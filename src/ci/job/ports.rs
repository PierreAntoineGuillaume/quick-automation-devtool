use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::{Output, Shared};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;

pub trait CommandRunner {
    fn precondition(&self, args: &str) -> Output;
    fn run(&self, args: &str) -> Output;
}

pub trait SystemFacade: CommandRunner {
    fn consume_job(&mut self, jobs: Arc<Shared>, tx: Sender<JobProgress>);
    fn delay(&mut self) -> usize;
    fn write_env(&self, env: HashMap<String, Vec<String>>);
    fn read_env(&self, key: &str, default: Option<&str>) -> anyhow::Result<String>;
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
