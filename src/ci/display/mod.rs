use super::job::{JobOutput, JobProgressTracker, Progress};
use super::schedule::CiDisplay;
use std::fmt::Formatter;

pub struct OneOffCiDisplay {}

impl CiDisplay for OneOffCiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker) {
        if tracker.is_finished() {
            println!("{}", tracker)
        }
    }
}

impl OneOffCiDisplay {
    pub fn new() -> Self {
        OneOffCiDisplay {}
    }
}

impl std::fmt::Display for JobProgressTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (job_name, progress_collector) in &self.states {
            writeln!(f, "job {}: ...", job_name)?;
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Partial(instruction, job_output) => match job_output {
                        JobOutput::Success(stdout, stderr) => {
                            write!(f, "✅ {}\n{}\n{}", instruction, stdout, stderr)?;
                        }
                        JobOutput::JobError(stdout, stderr) => {
                            write!(f, "❌ {}\n{}\n{}", instruction, stdout, stderr)?;
                        }
                        JobOutput::ProcessError(stderr) => {
                            write!(f, "💀 {}: {}", instruction, stderr)?;
                        }
                    },
                    Progress::Terminated(bool) => {
                        writeln!(f, "{} job {}", if *bool { "✅" } else { "❌" }, job_name)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
