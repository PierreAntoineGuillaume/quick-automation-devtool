use crate::ci::job::job_output::JobOutput;
use crate::ci::job::{Job, JobProgressTracker};

pub trait JobScheduler {
    fn schedule(&mut self, jobs: &[Job]) -> JobProgressTracker;
}

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

#[cfg(test)]
pub mod test {
    use super::*;

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
