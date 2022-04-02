pub mod dag;
pub mod docker_job;
pub mod inspection;
pub mod schedule;
pub mod simple_job;
#[cfg(test)]
pub mod tests;

use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::schedule::JobRunner;
use std::sync::{Arc, Mutex};

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

pub trait EnvBag {
    fn parse(&mut self, key: &str) -> Vec<String>;
    fn user(&self) -> String;
    fn group(&self) -> String;
    fn pwd(&self) -> String;
}

pub struct SimpleEnvBag {
    uid: String,
    gid: String,
    pwd: String,
    _env_keys: Vec<String>,
}

impl SimpleEnvBag {
    pub fn new<T: Into<String>>(uid: T, gid: T, pwd: T, _env_keys: Vec<String>) -> Self {
        Self {
            uid: uid.into(),
            gid: gid.into(),
            pwd: pwd.into(),
            _env_keys,
        }
    }
}

impl EnvBag for SimpleEnvBag {
    fn parse(&mut self, instruction: &str) -> Vec<String> {
        return instruction
            .split(' ')
            .filter(|str| !str.is_empty())
            .map(|str| str.to_string())
            .collect();
    }

    fn user(&self) -> String {
        self.uid.to_string()
    }

    fn group(&self) -> String {
        self.gid.to_string()
    }

    fn pwd(&self) -> String {
        self.pwd.to_string()
    }
}

pub trait JobTrait {
    fn introspect(&self, introspector: &mut dyn JobIntrospector);
    fn name(&self) -> &str;
    fn group(&self) -> Option<&str>;
    fn start(
        &self,
        runner: &mut dyn JobRunner,
        envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        consumer: &dyn JobProgressConsumer,
    );
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
