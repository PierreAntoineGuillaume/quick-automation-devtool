use crate::ci::job::Job;
use crate::ci::job::Progress;
use std::collections::BTreeMap;
use std::time::SystemTime;

pub struct JobProgress(String, Progress);

impl JobProgress {
    pub fn new(job_name: &str, progress: Progress) -> Self {
        JobProgress(job_name.to_string(), progress)
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

    pub fn is_available(&self) -> bool {
        match self.last() {
            Some(progress) => progress.is_available(),
            _ => false,
        }
    }
}

pub struct JobProgressTracker<'a> {
    jobs: &'a [Job],
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub states: BTreeMap<String, ProgressCollector>,
    pub has_failed: bool,
}

impl<'a> JobProgressTracker<'a> {
    pub fn new(jobs: &'a [Job]) -> Self {
        let mut tracker = JobProgressTracker {
            jobs,
            start_time: SystemTime::now(),
            end_time: None,
            states: BTreeMap::new(),
            has_failed: false,
        };
        tracker.init();
        tracker
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

    pub fn job_is_available(&self, name: &String) -> bool {
        if let Some(collector) = self.states.get(name) {
            collector.is_available()
        } else {
            false
        }
    }
    fn init(&mut self) {
        for job in self.jobs {
            self.record(JobProgress::new(&job.name, Progress::Available))
        }
    }
}
