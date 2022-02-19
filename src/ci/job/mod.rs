use std::collections::BTreeMap;
use std::time::SystemTime;

#[derive(Debug, PartialEq, Clone)]
pub enum JobOutput {
    Success(String, String),
    JobError(String, String),
    ProcessError(String),
}

impl JobOutput {
    pub fn succeeded(&self) -> bool {
        matches!(self, JobOutput::Success(_, _))
    }
}

pub trait JobProgressConsumer {
    fn consume(&self, job_progress: JobProgress);
}

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Job {
    pub name: String,
    pub instructions: Vec<String>,
}

impl Job {
    pub fn start(&self, runner: &dyn JobRunner, consumer: &dyn JobProgressConsumer) {
        consumer.consume(JobProgress::new(&self.name, Progress::Started));
        let mut success = true;
        for instruction in &self.instructions {
            let output = runner.run(instruction);
            success = output.succeeded();
            let partial = Progress::Partial(instruction.clone(), output);
            consumer.consume(JobProgress::new(&self.name, partial));
            if !success {
                break;
            }
        }
        consumer.consume(JobProgress::new(&self.name, Progress::Terminated(success)));
    }
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Awaiting,
    Started,
    Partial(String, JobOutput),
    Terminated(bool),
}

impl Progress {
    pub fn failed(&self) -> bool {
        matches!(
            self,
            Progress::Partial(_, JobOutput::JobError(_, _))
                | Progress::Partial(_, JobOutput::ProcessError(_))
                | Progress::Terminated(false)
        )
    }

    pub fn is_awaiting(&self) -> bool {
        matches!(self, Progress::Awaiting)
    }
}

impl Progress {
    pub fn is_pending(&self) -> bool {
        !matches!(*self, Progress::Terminated(_))
    }
}

pub struct Pipeline {
    jobs: Vec<Job>,
}

pub trait JobScheduler {
    fn schedule(&mut self, jobs: &[Job]) -> JobProgressTracker;
}

impl Pipeline {
    pub fn run(
        &mut self,
        scheduler: &mut dyn JobScheduler,
    ) -> Result<JobProgressTracker, JobProgressTracker> {
        let tracker = scheduler.schedule(&self.jobs);
        if tracker.has_failed {
            Err(tracker)
        } else {
            Ok(tracker)
        }
    }

    pub fn push_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn new() -> Pipeline {
        Pipeline { jobs: Vec::new() }
    }
}

pub struct JobProgress {
    pub job_name: String,
    pub progress: Progress,
}

impl JobProgress {
    pub fn new(job_name: &str, progress: Progress) -> Self {
        JobProgress {
            job_name: job_name.to_string(),
            progress,
        }
    }

    pub fn failed(&self) -> bool {
        self.progress.failed()
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

    pub fn is_awaiting(&self) -> bool {
        match self.last() {
            Some(progress) => progress.is_awaiting(),
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

    pub fn is_waiting_for(&self, name: &String) -> bool {
        if let Some(collector) = self.states.get(name) {
            collector.is_awaiting()
        } else {
            false
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub struct TestJobRunner {}

    impl JobRunner for TestJobRunner {
        fn run(&self, job: &str) -> JobOutput {
            if let Some(stripped) = job.strip_prefix("ok:") {
                JobOutput::Success(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("ko:") {
                JobOutput::JobError(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("crash:") {
                JobOutput::ProcessError(stripped.into())
            } else {
                panic!("Job should begin with ok:, ko, or crash:")
            }
        }
    }
}
