use crate::ci::job::Progress;
use indexmap::IndexMap;
use std::time::SystemTime;

pub struct JobProgress(String, pub Progress);

impl JobProgress {
    pub fn new(job_name: &str, progress: Progress) -> Self {
        Self(job_name.to_string(), progress)
    }
    pub const fn cancel(job_name: String) -> Self {
        Self(job_name, Progress::Cancelled)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub const fn failed(&self) -> bool {
        self.1.failed()
    }
}

#[derive(Default)]
pub struct ProgressCollector {
    pub progresses: Vec<Progress>,
}

impl ProgressCollector {
    fn push(&mut self, progress: Progress) {
        self.progresses.push(progress);
    }

    pub fn last(&self) -> &Progress {
        self.progresses.last().unwrap()
    }

    pub fn terminated(&self) -> Option<bool> {
        match self.last() {
            Progress::Terminated(result) => Some(*result),
            _ => None,
        }
    }

    pub fn instruction_list(&self) -> Vec<InstructionState> {
        let mut vec = vec![];
        let mut temp = None;
        for progress in &self.progresses {
            match progress {
                Progress::Partial(instruction, output) => {
                    temp = None;
                    vec.push(InstructionState::Finished(
                        instruction.clone(),
                        output.succeeded(),
                    ));
                }
                Progress::Started(instruction) => {
                    temp = Some(instruction.clone());
                }
                _ => {
                    temp = None;
                }
            }
        }
        if let Some(instruction) = temp {
            vec.push(InstructionState::Running(instruction));
        }
        vec
    }
}

pub enum InstructionState {
    Finished(String, bool),
    Running(String),
}

pub struct JobProgressTracker {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub states: IndexMap<String, ProgressCollector>,
    pub has_failed: bool,
}

impl JobProgressTracker {
    pub fn find_longest_jobname_size(&self) -> usize {
        self.states
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap()
    }

    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now(),
            end_time: None,
            states: IndexMap::new(),
            has_failed: false,
        }
    }
    pub fn record(&mut self, job_progress: JobProgress) {
        self.has_failed |= job_progress.failed();
        self.states
            .entry(job_progress.0)
            .or_default()
            .push(job_progress.1);
    }

    pub fn finish(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(SystemTime::now());
        }
    }
}
