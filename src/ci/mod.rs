pub(crate) mod display;
pub(crate) mod job;
pub(crate) mod schedule;

use job::*;
use std::process::Command;

pub struct CommandJobRunner {}

impl JobRunner for CommandJobRunner {
    fn run(&self, job: &str) -> JobOutput {
        match Command::new(&job).output() {
            Ok(output) => {
                return match (output.status.success(), std::str::from_utf8(&output.stdout)) {
                    (true, Ok(output)) => JobOutput::Success(output.to_string()),
                    (false, Ok(e)) => JobOutput::JobError(e.to_string()),
                    (_, Err(e)) => JobOutput::ProcessError(e.to_string()),
                };
            }
            Err(e) => JobOutput::ProcessError(format!("{}: {}", job, e)),
        }
    }
}

impl CommandJobRunner {
    pub fn new() -> CommandJobRunner {
        CommandJobRunner {}
    }
}
