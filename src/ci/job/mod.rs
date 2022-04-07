pub mod dag;
pub mod docker_job;
mod env_parser;
pub mod inspection;
pub mod schedule;
pub mod shell_interpreter;
pub mod simple_job;
#[cfg(test)]
pub mod tests;

use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::schedule::CommandRunner;

#[derive(Debug, PartialEq, Clone)]
pub enum JobOutput {
    Success(String, String),
    JobError(String, String),
    ProcessError(String),
}

impl JobOutput {
    pub fn succeeded(&self) -> bool {
        matches!(self, JobOutput::Success(_, _))
    }
}

pub trait JobProgressConsumer {
    fn consume(&self, job_progress: JobProgress);
}

pub trait JobIntrospector {
    fn basic_job(&mut self, name: &str, group: &Option<String>, instructions: &[String]);
    fn docker_job(
        &mut self,
        name: &str,
        image: &str,
        group: &Option<String>,
        instructions: &[String],
    );
}

pub type SharedJob = dyn JobTrait + Send + Sync;

pub trait JobTrait {
    fn introspect(&self, introspector: &mut dyn JobIntrospector);
    fn name(&self) -> &str;
    fn group(&self) -> Option<&str>;
    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn JobProgressConsumer);
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Available,
    Blocked(Vec<String>),
    Cancelled,
    Started(String),
    Partial(String, JobOutput),
    Terminated(bool),
}

impl Progress {
    pub fn failed(&self) -> bool {
        matches!(
            self,
            Progress::Partial(_, JobOutput::JobError(_, _))
                | Progress::Partial(_, JobOutput::ProcessError(_))
                | Progress::Terminated(false)
        )
    }
}
