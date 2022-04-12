pub mod constraint_matrix;
pub mod dag;
pub mod docker_job;
mod env_parser;
pub mod inspection;
pub mod ports;
pub mod schedule;
pub mod shell_interpreter;
pub mod simple_job;
#[cfg(test)]
pub mod tests;

use crate::ci::job::docker_job::DockerJob;
use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::ports::CommandRunner;
use crate::ci::job::simple_job::SimpleJob;
use std::collections::HashMap;
use std::sync::Arc;

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

#[derive(Clone)]
pub enum JobType {
    Simple(SimpleJob),
    Docker(DockerJob),
}

impl JobType {
    pub fn to_arc(&self) -> Arc<SharedJob> {
        match self {
            JobType::Simple(job) => {
                Arc::from(Box::new(job.clone()) as Box<dyn JobTrait + Send + Sync>)
            }
            JobType::Docker(job) => {
                Arc::from(Box::new(job.clone()) as Box<dyn JobTrait + Send + Sync>)
            }
        }
    }
}

impl JobTrait for JobType {
    fn introspect(&self, introspector: &mut dyn JobIntrospector) {
        match self {
            JobType::Docker(job) => job.introspect(introspector),
            JobType::Simple(job) => job.introspect(introspector),
        }
    }

    fn name(&self) -> &str {
        match self {
            JobType::Docker(job) => job.name(),
            JobType::Simple(job) => job.name(),
        }
    }

    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>) {
        match self {
            JobType::Simple(job) => job.forward_env(env),
            JobType::Docker(job) => job.forward_env(env),
        }
    }

    fn group(&self) -> Option<&str> {
        match self {
            JobType::Docker(job) => job.group(),
            JobType::Simple(job) => job.group(),
        }
    }

    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn JobProgressConsumer) {
        match self {
            JobType::Simple(job) => job.start(runner, consumer),
            JobType::Docker(job) => job.start(runner, consumer),
        }
    }
}

pub trait JobTrait {
    fn introspect(&self, introspector: &mut dyn JobIntrospector);
    fn name(&self) -> &str;
    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>);
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
