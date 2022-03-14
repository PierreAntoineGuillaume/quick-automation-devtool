pub mod dag;
pub mod inspection;
pub mod schedule;
#[cfg(test)]
pub mod tests;

use crate::ci::job::inspection::{JobProgress, JobProgressTracker};
use crate::ci::job::schedule::JobRunner;

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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Job {
    pub name: String,
    pub image: Option<String>,
    pub instructions: Vec<String>,
}

impl Job {
    pub fn long(name: String, instructions: Vec<String>, image: Option<String>) -> Self {
        Self {
            name,
            instructions,
            image,
        }
    }

    pub fn short(name: String, instructions: Vec<String>) -> Self {
        Self::long(name, instructions, None)
    }

    pub fn start(&self, runner: &mut dyn JobRunner, consumer: &dyn JobProgressConsumer) {
        let mut success = true;

        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let output = self.run(instruction, runner);
            success = output.succeeded();
            let partial = Progress::Partial(instruction.clone(), output);
            consumer.consume(JobProgress::new(&self.name, partial));
            if !success {
                break;
            }
        }

        consumer.consume(JobProgress::new(&self.name, Progress::Terminated(success)));
    }

    fn run(&self, instruction: &str, runner: &dyn JobRunner) -> JobOutput {
        if let Some(image) = &self.image {
            let mut args = vec![
                "docker",
                "run",
                "--rm",
                "--user",
                "$USER:$GROUPS",
                "--volume",
                "$PWD:$PWD",
                "--workdir",
                "$PWD",
                image.as_str(),
            ];
            args.extend(instruction.split(' ').into_iter());
            return runner.run(&args);
        }

        runner.run(&instruction.split(' ').into_iter().collect::<Vec<&str>>())
    }
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Available,
    Started(String),
    Blocked(Vec<String>),
    Cancelled,
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

    pub fn is_pending(&self) -> bool {
        !matches!(*self, Progress::Terminated(_))
    }
}
