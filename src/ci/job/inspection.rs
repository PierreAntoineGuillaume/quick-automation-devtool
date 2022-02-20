use crate::ci::job::state::Progress;
use crate::ci::job::JobProgress;
use std::collections::BTreeMap;
use std::time::SystemTime;

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

    pub fn is_available(&self) -> bool {
        match self.last() {
            Some(progress) => progress.is_available(),
            _ => false,
        }
    }
}

pub struct JobProgressTracker {
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub states: BTreeMap<String, ProgressCollector>,
    pub has_failed: bool,
}

impl JobProgressTracker {
    pub fn new() -> Self {
        JobProgressTracker {
            start_time: SystemTime::now(),
            end_time: None,
            states: BTreeMap::new(),
            has_failed: false,
        }
    }
    pub fn record(&mut self, job_progress: JobProgress) {
        self.has_failed |= job_progress.failed();
        self.states
            .entry(job_progress.job_name)
            .or_insert_with(ProgressCollector::new)
            .push(job_progress.progress);
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

    pub fn job_is_available(&self, name: &String) -> bool {
        if let Some(collector) = self.states.get(name) {
            collector.is_available()
        } else {
            false
        }
    }
}
