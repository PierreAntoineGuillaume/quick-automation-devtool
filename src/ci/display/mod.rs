use super::job::{JobOutput, JobProgress, Progress};
use super::schedule::CiDisplay;
use std::collections::BTreeMap;
use std::fmt::Formatter;

pub struct PipelineProgress {
    states: BTreeMap<String, Progress>,
}

impl CiDisplay for PipelineProgress {
    fn record(&mut self, job_progress: JobProgress) {
        self.states
            .insert(job_progress.job_name, job_progress.progress);
    }

    fn is_finished(&self) -> bool {
        for progress in self.states.values() {
            if progress.is_pending() {
                return false;
            }
        }
        true
    }

    fn refresh(&mut self) {
        if self.is_finished() {
            println!("{}", self)
        }
    }
}

impl PipelineProgress {
    pub fn new() -> Self {
        PipelineProgress {
            states: BTreeMap::new(),
        }
    }
}

impl std::fmt::Display for PipelineProgress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut count = self.states.len();
        for (job_name, progress) in &self.states {
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
