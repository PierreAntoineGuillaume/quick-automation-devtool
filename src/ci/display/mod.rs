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
        let mut count = self.states.len();
        for (job_name, progress) in &self.states {
            if let Some(Progress::Terminated(job_output)) = progress.last() {
                match job_output {
                    JobOutput::Success(string) => {
                        write!(f, "✅ {}\n{}", job_name, string)?;
                    }
                    JobOutput::JobError(string) => {
                        write!(f, "❌ {}\n{}", job_name, string)?;
                    }
                    JobOutput::ProcessError(string) => {
                        write!(f, "💀 {}: {}", job_name, string)?;
                    }
                }
            }
            count -= 1;
            if count != 0 {
                writeln!(f)?
            }
        }
        Ok(())
    }
}
