use super::job::{JobOutput, JobProgressTracker, Progress};
use super::schedule::CiDisplay;
use std::fmt::Formatter;

pub struct OneOffCiDisplay {}

impl CiDisplay for OneOffCiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker) {
        if tracker.is_finished() {
            print!("{}", tracker)
        }
    }
}

impl OneOffCiDisplay {
    pub fn new() -> Self {
        OneOffCiDisplay {}
    }
}

const CHECK: &str = "✔";
const CROSS: &str = "✕";
const SKULL: &str = "�";

fn try_cleanup(input: String) -> String {
    let cleaned = input.trim_end();
    if cleaned.is_empty() {
        String::new()
    } else {
        format!("{cleaned}\n")
    }
}

impl std::fmt::Display for JobProgressTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (job_name, progress_collector) in &self.states {
            writeln!(f, "Running tasks for job {job_name}")?;
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Partial(instruction, job_output) => match job_output {
                        JobOutput::Success(stdout, stderr)
                        | JobOutput::JobError(stdout, stderr) => {
                            let symbol = if job_output.succeeded() { CHECK } else { CROSS };
                            write!(
                                f,
                                "{symbol} {}",
                                try_cleanup(format!(
                                    "{}{}{}",
                                    instruction,
                                    try_cleanup(stdout.clone()),
                                    try_cleanup(stderr.clone())
                                ))
                            )?;
                        }
                        JobOutput::ProcessError(stderr) => {
                            write!(f, "{SKULL} {instruction}: {}", try_cleanup(stderr.clone()))?;
                        }
                    },
                    Progress::Terminated(bool) => {
                        let emoji: &str = if *bool { CHECK } else { CROSS };
                        writeln!(f, "{emoji} all tasks done for job {job_name}")?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
