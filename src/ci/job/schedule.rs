use crate::ci::job::job_output::JobOutput;
use crate::ci::job::state::Progress;
use crate::ci::job::{Job, JobProgress, JobProgressTracker};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

pub trait JobStarter {
    fn consume_some_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>);
    fn join(&mut self);
    fn delay(&mut self) -> usize;
}

pub trait CiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker, elapsed: usize);
    fn finish(&mut self, tracker: &JobProgressTracker);
}

pub struct Pipeline {}
impl Pipeline {
    pub fn schedule(
        &mut self,
        jobs: &[Job],
        job_starter: &mut dyn JobStarter,
        job_display: &mut dyn CiDisplay,
    ) -> JobProgressTracker {
        let mut tracker = JobProgressTracker::new();
        if jobs.is_empty() {
            tracker.try_finish();
            job_display.finish(&tracker);
            return tracker;
        }

        let (tx, rx) = channel();

        self.signal_all_existing_jobs(jobs, &tx);

        while let Some(progress) = self.read(&rx) {
            tracker.record(progress);
        }

        let mut delay: usize = 0;

        loop {
            let available_jobs: Vec<Job> = jobs
                .iter()
                .filter(|job| tracker.job_is_available(&job.name))
                .cloned()
                .collect();

            if !available_jobs.is_empty() {
                job_starter.consume_some_jobs(&available_jobs, tx.clone());
            }

            while let Some(progress) = self.read(&rx) {
                tracker.record(progress);
            }

            if tracker.try_finish() {
                break;
            }
            job_display.refresh(&tracker, delay);
            delay = job_starter.delay();
        }

        job_starter.join();

        job_display.finish(&tracker);

        tracker
    }

    fn signal_all_existing_jobs(&self, jobs: &[Job], first_tx: &Sender<JobProgress>) {
        for job in jobs {
            first_tx
                .send(JobProgress::new(&job.name, Progress::Available))
                .unwrap();
        }
    }

    pub fn read(&self, rx: &Receiver<JobProgress>) -> Option<JobProgress> {
        match rx.try_recv() {
            Ok(state) => Some(state),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                panic!("State receiver has been disconnected, try restarting the program");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut scheduler = Pipeline {};
        scheduler.schedule(jobs, &mut job_start, &mut job_display)
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
        fn consume_some_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>) {
            for job in jobs {
                job.start(&TestJobRunner {}, &tx.clone());
            }
        }

        fn join(&mut self) {}
        fn delay(&mut self) -> usize {
            0
        }
    }

    pub struct NullCiDisplay {}
    impl CiDisplay for NullCiDisplay {
        fn refresh(&mut self, _: &JobProgressTracker, _: usize) {}
        fn finish(&mut self, _: &JobProgressTracker) {}
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
                panic!("Job should begin with ok:, ko, or crash:")
            }
        }
    }
}
