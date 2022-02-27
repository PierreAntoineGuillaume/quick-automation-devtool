use crate::ci::job::Progress;
use indexmap::IndexMap;
use std::time::SystemTime;

pub struct JobProgress(String, pub(crate) Progress);

impl JobProgress {
    pub fn new(job_name: &str, progress: Progress) -> Self {
        JobProgress(job_name.to_string(), progress)
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn failed(&self) -> bool {
        self.1.failed()
    }
}

pub struct ProgressCollector {
    pub progresses: std::vec::Vec<Progress>,
}

impl ProgressCollector {
    fn new() -> Self {
        ProgressCollector { progresses: vec![] }
    }
    fn push(&mut self, progress: Progress) {
        self.progresses.push(progress)
    }

    pub fn last(&self) -> Option<&Progress> {
        self.progresses.last()
    }
}

pub struct JobProgressTracker {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub states: IndexMap<String, ProgressCollector>,
    pub has_failed: bool,
}

impl JobProgressTracker {
    pub fn new() -> Self {
        JobProgressTracker {
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
            .or_insert_with(ProgressCollector::new)
            .push(job_progress.1);
    }

    pub fn try_finish(&mut self) -> bool {
        if self.end_time.is_some() {
            return true;
        }
        for progress in self.states.values() {
            if let Some(progress) = progress.last() {
                if progress.is_pending() {
                    return false;
                }
            }
        }
        self.end_time = Some(SystemTime::now());
        true
    }
}
