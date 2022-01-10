use job::{Job, JobOutput, JobProgress, JobRunner, Progress};
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

impl JobStarter for ParrallelJobStarter {
    fn start_all_jobs(&self, jobs: &[Job], first_tx: Sender<JobProgress>) {
        for real_job in jobs {
            let job = real_job.clone();
            let tx = first_tx.clone();
            thread::spawn(move || {
                tx.send(JobProgress::new(job.name.clone(), Progress::Started))
                    .unwrap();
                let terminated = Progress::Terminated(job.start(&CommandJobRunner::new()));
                tx.send(JobProgress::new(job.name, terminated)).unwrap();
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
