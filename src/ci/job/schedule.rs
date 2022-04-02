use crate::ci::job::dag::{Dag, JobResult, JobState};
use crate::ci::job::env_bag::EnvBag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::{JobOutput, JobProgressTracker, Progress};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

pub trait JobRunner {
    fn run(&self, args: &[&str]) -> JobOutput;
}

pub trait JobStarter {
    fn consume_some_jobs(
        &mut self,
        jobs: &mut Dag,
        envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        tx: Sender<JobProgress>,
    );
    fn join(&mut self);
    fn delay(&mut self) -> usize;
}

pub trait RunningCiDisplay {
    fn set_up(&mut self, tracker: &JobProgressTracker);
    fn run(&mut self, tracker: &JobProgressTracker, elapsed: usize);
    fn tear_down(&mut self, tracker: &JobProgressTracker);
}

pub trait FinalCiDisplay {
    fn finish(&mut self, tracker: &JobProgressTracker);
}

pub fn schedule(
    mut jobs: Dag,
    job_starter: &mut dyn JobStarter,
    job_display: &mut dyn RunningCiDisplay,
    envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
) -> JobProgressTracker {
    let mut tracker = JobProgressTracker::new();

    if jobs.is_finished() {
        tracker.finish();
        return tracker;
    }

    for job in jobs.enumerate() {
        tracker.record(JobProgress::new(
            &job.name,
            match job.state {
                JobState::Pending => Progress::Available,
                JobState::Blocked => Progress::Blocked(job.block.clone()),
                _ => {
                    unreachable!("This state is impossible with no poll yet")
                }
            },
        ))
    }

    let (tx, rx) = channel();

    let mut delay: usize = 0;

    job_display.set_up(&tracker);

    loop {
        job_starter.consume_some_jobs(&mut jobs, envbag.clone(), tx.clone());

        while let Some(progress) = read(&rx) {
            let mut cancel_list: Vec<String> = vec![];
            let name = progress.name();
            if let Progress::Terminated(success) = progress.1 {
                jobs.record_event(
                    name,
                    if success {
                        JobResult::Success
                    } else {
                        JobResult::Failure
                    },
                );
                if !success {
                    cancel_list = jobs
                        .enumerate()
                        .iter()
                        .filter(|job| matches!(job.state, JobState::Cancelled(_)))
                        .map(|job| job.name.clone())
                        .collect();
                }
            }
            tracker.record(progress);
            for cancel in cancel_list {
                tracker.record(JobProgress::cancel(cancel))
            }
        }

        if jobs.is_finished() {
            tracker.finish();
            break;
        }
        job_display.run(&tracker, delay);
        delay = job_starter.delay();
    }

    job_starter.join();
    job_display.tear_down(&tracker);

    tracker
}

pub fn read(rx: &Receiver<JobProgress>) -> Option<JobProgress> {
    match rx.try_recv() {
        Ok(state) => Some(state),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => {
            panic!("State receiver has been disconnected, try restarting the program");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::display::silent_display::SilentDisplay;
    use crate::ci::job::env_bag::SimpleEnvBag;
    use crate::ci::job::simple_job::SimpleJob;
    use crate::ci::job::SharedJob;
    use std::sync::{Arc, Mutex};

    impl SimpleJob {
        pub fn new(name: &str, instructions: &[&str]) -> Self {
            SimpleJob::short(
                name.to_string(),
                instructions
                    .iter()
                    .map(|item| String::from(*item))
                    .collect(),
            )
        }
    }

    fn test_pipeline(jobs: &[Arc<SharedJob>]) -> JobProgressTracker {
        let mut job_start = TestJobStarter {};
        let mut job_display = SilentDisplay {};
        let dag = Dag::new(jobs, &[], &[]).unwrap();
        let envbag = Arc::from(Mutex::new(SimpleEnvBag::new("uid", "gid", "/dir", vec![])));
        schedule(dag, &mut job_start, &mut job_display, envbag)
    }

    #[test]
    pub fn every_job_is_initialisated() {
        assert!(!test_pipeline(&[Arc::from(SimpleJob::new("a", &["ok: result"]))]).has_failed)
    }

    #[test]
    pub fn one_job_failure_fails_scheduling() {
        assert!(test_pipeline(&[Arc::from(SimpleJob::new("c", &["ko: result"]))]).has_failed)
    }

    #[test]
    pub fn one_job_crash_fails_scheduling() {
        assert!(test_pipeline(&[Arc::from(SimpleJob::new("c", &["crash: result"]))]).has_failed)
    }

    pub struct TestJobStarter {}
    impl JobStarter for TestJobStarter {
        fn consume_some_jobs(
            &mut self,
            jobs: &mut Dag,
            envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
            tx: Sender<JobProgress>,
        ) {
            let new_env_bag = envbag.clone();
            while let Some(job) = jobs.poll() {
                job.start(&mut TestJobRunner {}, new_env_bag.clone(), &tx.clone());
            }
        }

        fn join(&mut self) {}

        fn delay(&mut self) -> usize {
            0
        }
    }

    pub struct TestJobRunner {}
    impl JobRunner for TestJobRunner {
        fn run(&self, args: &[&str]) -> JobOutput {
            let job = args[0].to_string();
            if let Some(stripped) = job.strip_prefix("ok:") {
                JobOutput::Success(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("ko:") {
                JobOutput::JobError(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("crash:") {
                JobOutput::ProcessError(stripped.into())
            } else {
                unreachable!("Job should begin with ok:, ko, or crash:")
            }
        }
    }
}
