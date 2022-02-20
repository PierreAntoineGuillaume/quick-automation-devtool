pub mod inspection;
pub mod job_output;
pub mod schedule;
pub mod state;

use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::JobRunner;
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
