use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::Job;
use crate::ci::job::Output;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

pub trait CommandRunner {
    fn run(&self, args: &str) -> Output;
}

pub trait SystemFacade: CommandRunner {
    fn consume_job(&mut self, jobs: Job, tx: Sender<JobProgress>);
    fn delay(&mut self) -> usize;
    fn write_env(&self, env: HashMap<String, Vec<String>>);
}

pub trait FinalCiDisplay {
    fn finish(&mut self, tracker: &JobProgressTracker);
}

pub trait UserFacade {
    fn set_up(&mut self, tracker: &JobProgressTracker);
    fn run(&mut self, tracker: &JobProgressTracker, elapsed: usize);
    fn tear_down(&mut self, tracker: &JobProgressTracker);
    fn display_error(&self, error: String);
}
