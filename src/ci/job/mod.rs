pub mod constraint_matrix;
pub mod dag;
pub mod docker;
mod env_parser;
pub mod inspection;
pub mod ports;
pub mod schedule;
pub mod shell_interpreter;
pub mod simple;
#[cfg(test)]
pub mod tests;

use crate::ci::job::docker::Docker;
use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::ports::CommandRunner;
use crate::ci::job::simple::Simple;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub enum Output {
    Success(String, String),
    JobError(String, String),
    ProcessError(String),
}

impl Output {
    pub fn succeeded(&self) -> bool {
        matches!(self, Output::Success(_, _))
    }
}

pub trait ProgressConsumer {
    fn consume(&self, job_progress: JobProgress);
}

pub trait Introspector {
    fn basic_job(
        &mut self,
        name: &str,
        group: &Option<String>,
        instructions: &[String],
        skip_if: &Option<String>,
    );
    fn docker_job(
        &mut self,
        name: &str,
        image: &str,
        group: &Option<String>,
        instructions: &[String],
        skip_if: &Option<String>,
    );
}

pub type Shared = dyn JobTrait + Send + Sync;

#[derive(Clone)]
pub enum Type {
    Simple(Simple),
    Docker(Docker),
}

impl Type {
    pub fn to_arc(&self) -> Arc<Shared> {
        match self {
            Type::Simple(job) => {
                Arc::from(Box::new(job.clone()) as Box<dyn JobTrait + Send + Sync>)
            }
            Type::Docker(job) => {
                Arc::from(Box::new(job.clone()) as Box<dyn JobTrait + Send + Sync>)
            }
        }
    }
}

impl JobTrait for Type {
    fn introspect(&self, introspector: &mut dyn Introspector) {
        match self {
            Type::Docker(job) => job.introspect(introspector),
            Type::Simple(job) => job.introspect(introspector),
        }
    }

    fn name(&self) -> &str {
        match self {
            Type::Docker(job) => job.name(),
            Type::Simple(job) => job.name(),
        }
    }

    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>) {
        match self {
            Type::Simple(job) => job.forward_env(env),
            Type::Docker(job) => job.forward_env(env),
        }
    }

    fn group(&self) -> Option<&str> {
        match self {
            Type::Docker(job) => job.group(),
            Type::Simple(job) => job.group(),
        }
    }

    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn ProgressConsumer) {
        match self {
            Type::Simple(job) => job.start(runner, consumer),
            Type::Docker(job) => job.start(runner, consumer),
        }
    }
}

pub trait JobTrait {
    fn introspect(&self, introspector: &mut dyn Introspector);
    fn name(&self) -> &str;
    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>);
    fn group(&self) -> Option<&str>;
    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn ProgressConsumer);
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Available,
    Blocked(Vec<String>),
    Cancelled,
    Started(String),
    Partial(String, Output),
    Skipped,
    Terminated(bool),
}

impl Progress {
    pub fn failed(&self) -> bool {
        matches!(
            self,
            Progress::Partial(_, Output::JobError(_, _) | Output::ProcessError(_))
                | Progress::Terminated(false)
        )
    }
}
