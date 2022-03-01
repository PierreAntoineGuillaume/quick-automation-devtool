use crate::ci::job::dag::{Dag, JobResult, JobState};
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::{JobOutput, JobProgressTracker, Progress};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

pub trait JobStarter {
    fn consume_some_jobs(&mut self, jobs: &mut Dag, tx: Sender<JobProgress>);
    fn join(&mut self);
    fn delay(&mut self) -> usize;
}

pub trait CiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker, elapsed: usize);
    fn finish(&mut self, tracker: &JobProgressTracker);
}

pub fn schedule(
    mut jobs: Dag,
    job_starter: &mut dyn JobStarter,
    job_display: &mut dyn CiDisplay,
) -> JobProgressTracker {
    let mut tracker = JobProgressTracker::new();

    if jobs.is_finished() {
        tracker.try_finish();
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

    loop {
        job_starter.consume_some_jobs(&mut jobs, tx.clone());

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
            tracker.try_finish();
            break;
        }
        job_display.refresh(&tracker, delay);
        delay = job_starter.delay();
    }

    job_starter.join();

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
    use crate::ci::display::NullCiDisplay;
    use crate::ci::job::Job;

    impl Job {
        pub fn new(name: &str, instructions: &[&str]) -> Self {
            Job {
                name: name.to_string(),
                instructions: instructions
                    .iter()
                    .map(|item| String::from(*item))
                    .collect(),
            }
        }
    }

    fn pipeline(jobs: &[Job]) -> JobProgressTracker {
        let mut job_start = TestJobStarter {};
        let mut job_display = NullCiDisplay {};
        let dag = Dag::new(jobs, &[]).unwrap();
        schedule(dag, &mut job_start, &mut job_display)
    }

    #[test]
    pub fn every_job_is_initialisated() {
        assert!(!pipeline(&[Job::new("a", &["ok: result"])]).has_failed)
    }

    #[test]
    pub fn one_job_failure_fails_scheduling() {
        assert!(pipeline(&[Job::new("c", &["ko: result"])]).has_failed)
    }

    #[test]
    pub fn one_job_crash_fails_scheduling() {
        assert!(pipeline(&[Job::new("c", &["crash: result"])]).has_failed)
    }

    pub struct TestJobStarter {}
    impl JobStarter for TestJobStarter {
        fn consume_some_jobs(&mut self, jobs: &mut Dag, tx: Sender<JobProgress>) {
            while let Some(job) = jobs.poll() {
                job.start(&TestJobRunner {}, &tx.clone());
            }
        }

        fn join(&mut self) {}

        fn delay(&mut self) -> usize {
            0
        }
    }

    pub struct TestJobRunner {}
    impl JobRunner for TestJobRunner {
        fn run(&self, job: &str) -> JobOutput {
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
