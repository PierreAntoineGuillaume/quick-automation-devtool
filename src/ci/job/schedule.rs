use crate::ci::job::job_output::JobOutput;
use crate::ci::job::{Job, JobProgress, JobProgressTracker};
use std::sync::mpsc::Sender;

pub trait JobScheduler {
    fn schedule(&mut self, jobs: &[Job]) -> JobProgressTracker;
}

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

#[cfg(test)]
pub mod test {
    use super::*;

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
