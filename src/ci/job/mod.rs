use std::collections::BTreeMap;

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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Job {
    pub name: String,
    instructions: Vec<String>,
}

impl Job {
    pub fn new(name: &str, instruction: Vec<String>) -> Self {
        Job {
            name: name.to_string(),
            instructions: instruction,
        }
    }
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
    fn schedule(&mut self, jobs: &[Job]) -> Result<JobProgressTracker, JobProgressTracker>;
}

impl Pipeline {
    pub fn run(
        &mut self,
        scheduler: &mut dyn JobScheduler,
    ) -> Result<JobProgressTracker, JobProgressTracker> {
        scheduler.schedule(&self.jobs)
    }

    pub fn push(&mut self, key: &str, instruction: Vec<String>) {
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
