use crate::ci::job::{Job, JobProgressTracker};

pub trait JobScheduler {
    fn schedule(&mut self, jobs: &[Job]) -> JobProgressTracker;
}
