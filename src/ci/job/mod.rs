pub mod inspection;
pub mod job_output;
pub mod schedule;
pub mod state;

use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::{JobRunner, JobScheduler};
use crate::ci::job::state::Progress;

pub trait JobProgressConsumer {
    fn consume(&self, job_progress: JobProgress);
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

pub struct Pipeline {
    jobs: Vec<Job>,
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
