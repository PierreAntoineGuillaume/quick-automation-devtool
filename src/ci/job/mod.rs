use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub enum JobOutput {
    Success(String),
    JobError(String),
    ProcessError(String),
}

impl JobOutput {
    pub fn failed(&self) -> bool {
        !matches!(self, JobOutput::Success(_))
    }
}

pub trait JobProgressConsumer {
    fn consume(&self, job_progress: JobProgress);
}

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Job {
    pub name: String,
    instruction: String,
}

impl Job {
    pub fn new(name: String, instruction: String) -> Self {
        Job { name, instruction }
    }
    pub fn start(&self, runner: &dyn JobRunner, consumer: &dyn JobProgressConsumer) {
        consumer.consume(JobProgress::new(self.name.clone(), Progress::Started));
        let terminated = Progress::Terminated(runner.run(&self.instruction));
        consumer.consume(JobProgress::new(self.name.clone(), terminated));
    }
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Awaiting,
    Started,
    Terminated(JobOutput),
}

impl Progress {
    pub fn failed(&self) -> bool {
        match self {
            Progress::Terminated(job_output) => job_output.failed(),
            _ => false,
        }
    }
}

impl Progress {
    pub fn is_pending(&self) -> bool {
        matches!(*self, Progress::Awaiting | Progress::Started)
    }
}

pub struct Pipeline {
    jobs: Vec<Job>,
}

pub trait JobScheduler {
    fn schedule(&mut self, jobs: &[Job]) -> Result<(), ()>;
}

impl Pipeline {
    pub fn run(&mut self, scheduler: &mut dyn JobScheduler) -> Result<(), ()> {
        scheduler.schedule(&self.jobs)
    }

    pub fn push(&mut self, key: String, instruction: String) {
        self.jobs.push(Job::new(key, instruction));
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
    pub fn new(job: String, progress: Progress) -> Self {
        JobProgress {
            job_name: job,
            progress,
        }
    }

    pub fn failed(&self) -> bool {
        self.progress.failed()
    }
}

pub struct ProgressCollector {
    progresses: std::vec::Vec<Progress>,
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
    pub states: BTreeMap<String, ProgressCollector>,
}

impl JobProgressTracker {
    pub fn new() -> Self {
        JobProgressTracker {
            states: BTreeMap::new(),
        }
    }
    pub fn record(&mut self, job_progress: JobProgress) {
        self.states
            .entry(job_progress.job_name)
            .or_insert_with(ProgressCollector::new)
            .push(job_progress.progress)
    }

    pub fn is_finished(&self) -> bool {
        for progress in self.states.values() {
            if let Some(progress) = progress.last() {
                if progress.is_pending() {
                    return false;
                }
            }
        }
        true
    }
}
