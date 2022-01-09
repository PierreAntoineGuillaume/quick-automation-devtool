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

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Job {
    pub name: String,
    instruction: String,
}

impl Job {
    pub fn start(&self, runner: Box<dyn JobRunner>) -> JobOutput {
        runner.run(&self.instruction)
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
        self.jobs.push(Job {
            name: key,
            instruction,
        });
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
