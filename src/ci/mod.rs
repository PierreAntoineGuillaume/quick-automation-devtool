use job::{Job, JobOutput, JobProgress, JobProgressConsumer, JobRunner};
use schedule::JobStarter;
use std::sync::mpsc::Sender;
use std::thread;

pub(crate) mod display;
pub(crate) mod job;
pub(crate) mod schedule;

#[derive(Clone, Copy)]
pub struct ParrallelJobStarter {}

impl ParrallelJobStarter {
    pub fn new() -> Self {
        ParrallelJobStarter {}
    }
}

impl JobProgressConsumer for Sender<JobProgress> {
    fn consume(&self, job_progress: JobProgress) {
        self.send(job_progress).unwrap()
    }
}

impl JobStarter for ParrallelJobStarter {
    fn start_all_jobs(&self, jobs: &[Job], tx: Sender<JobProgress>) {
        for real_job in jobs {
            let job = real_job.clone();
            let consumer = tx.clone();
            thread::spawn(move || {
                job.start(&CommandJobRunner::new(), &consumer);
            });
        }
    }
}

pub struct CommandJobRunner {}

impl CommandJobRunner {
    pub fn new() -> CommandJobRunner {
        CommandJobRunner {}
    }
}

impl JobRunner for CommandJobRunner {
    fn run(&self, job: &str) -> JobOutput {
        match std::process::Command::new(&job).output() {
            Ok(output) => {
                return match (output.status.success(), std::str::from_utf8(&output.stdout)) {
                    (true, Ok(output)) => JobOutput::Success(output.to_string()),
                    (false, Ok(e)) => JobOutput::JobError(e.to_string()),
                    (_, Err(e)) => JobOutput::ProcessError(e.to_string()),
                };
            }
            Err(e) => JobOutput::ProcessError(format!("{}: {}", job, e)),
        }
    }
}
