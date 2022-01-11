use super::job::{JobOutput, JobProgress, JobProgressTracker, Progress};
use super::schedule::CiDisplay;
use std::fmt::Formatter;

pub struct OneOffCiDisplay {
    tracker: JobProgressTracker,
}

impl CiDisplay for OneOffCiDisplay {
    fn record(&mut self, job_progress: JobProgress) {
        self.tracker.record(job_progress);
    }

    fn is_finished(&self) -> bool {
        self.tracker.is_finished()
    }

    fn refresh(&mut self) {
        if self.is_finished() {
            println!("{}", self)
        }
    }
}

impl OneOffCiDisplay {
    pub fn new() -> Self {
        OneOffCiDisplay {
            tracker: JobProgressTracker::new(),
        }
    }
}

impl std::fmt::Display for OneOffCiDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut count = self.tracker.states.len();
        for (job_name, progress) in &self.tracker.states {
            if let Progress::Terminated(job_output) = progress {
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
