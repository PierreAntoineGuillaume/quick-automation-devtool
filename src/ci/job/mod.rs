pub mod constraint_matrix;
pub mod container_configuration;
pub mod dag;
mod env_parser;
pub mod inspection;
pub mod ports;
pub mod schedule;
pub mod shell_interpreter;
#[cfg(test)]
pub mod tests;

use crate::ci::job::inspection::JobProgressTracker;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Output {
    Success(String, String),
    JobError(String, String),
    ProcessError(String),
}

impl Output {
    pub const fn succeeded(&self) -> bool {
        matches!(self, Output::Success(_, _))
    }
}

pub trait ProgressConsumer {
    fn consume(&self, job_progress: JobProgress);
}

#[derive(Debug, Eq, PartialEq)]
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
    pub const fn failed(&self) -> bool {
        matches!(
            self,
            Progress::Partial(_, Output::JobError(_, _) | Output::ProcessError(_))
                | Progress::Terminated(false)
        )
    }
}

use crate::ci::job::container_configuration::ContainerConfiguration;
use crate::ci::job::container_configuration::ContainerConfiguration::Container;
use crate::ci::job::inspection::JobProgress;
use ports::CommandRunner;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Job {
    name: String,
    group: Option<String>,
    container: ContainerConfiguration,
    instructions: Vec<String>,
    skip_if: Option<String>,
}

impl Job {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn forward_env(&mut self, env: &HashMap<String, Vec<String>>) {
        if let Container(container) = &mut self.container {
            for key in env.keys() {
                container.forward_env(key);
            }
        }
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    pub fn start(&self, runner: &impl CommandRunner, consumer: &dyn ProgressConsumer) {
        if let Some(condition) = &self.skip_if {
            if runner.run(condition).succeeded() {
                consumer.consume(JobProgress::new(&self.name, Progress::Skipped));
                consumer.consume(JobProgress::new(&self.name, Progress::Terminated(true)));
                return;
            }
        }

        let mut success = true;
        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let command = self.container.compile(instruction);

            let output = runner.run(&command);

            success = output.succeeded();
            let partial = Progress::Partial(instruction.clone(), output);
            consumer.consume(JobProgress::new(&self.name, partial));
            if !success {
                break;
            }
        }

        consumer.consume(JobProgress::new(&self.name, Progress::Terminated(success)));
    }

    pub fn long(
        name: String,
        instructions: Vec<String>,
        group: Option<String>,
        skip_if: Option<String>,
    ) -> Self {
        Self {
            name,
            group,
            container: ContainerConfiguration::None,
            instructions,
            skip_if,
        }
    }

    pub fn new(
        name: String,
        instructions: Vec<String>,
        container: ContainerConfiguration,
        group: Option<String>,
        skip_if: Option<String>,
    ) -> Self {
        Self {
            name,
            group,
            container,
            instructions,
            skip_if,
        }
    }
}
