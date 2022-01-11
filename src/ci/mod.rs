use job::{Job, JobOutput, JobProgress, JobProgressConsumer, JobRunner};
use schedule::JobStarter;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

pub(crate) mod display;
pub(crate) mod job;
pub(crate) mod schedule;

pub struct ParrallelJobStarter {
    threads: std::vec::Vec<JoinHandle<()>>,
}

impl ParrallelJobStarter {
    pub fn new() -> Self {
        ParrallelJobStarter { threads: vec![] }
    }
}

impl JobProgressConsumer for Sender<JobProgress> {
    fn consume(&self, job_progress: JobProgress) {
        self.send(job_progress).unwrap()
    }
}

impl JobStarter for ParrallelJobStarter {
    fn start_all_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>) {
        for real_job in jobs {
            let job = real_job.clone();
            let consumer = tx.clone();
            self.threads.push(thread::spawn(move || {
                job.start(&CommandJobRunner::new(), &consumer);
            }));
        }
    }

    fn join(&mut self) {
        while let Some(handle) = self.threads.pop() {
            handle.join().expect("Could not join handle")
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
        let mut parts = job.split(' ');
        if let Some(program) = parts.next() {
            let args: Vec<&str> = parts.into_iter().collect();
            match Command::new(&program).args(&args).output() {
                Ok(output) => {
                    return match (output.status.success(), std::str::from_utf8(&output.stdout)) {
                        (true, Ok(output)) => JobOutput::Success(output.to_string()),
                        (false, Ok(e)) => JobOutput::JobError(e.to_string()),
                        (_, Err(e)) => JobOutput::ProcessError(e.to_string()),
                    };
                }
                Err(e) => JobOutput::ProcessError(format!("{}: {}", job, e)),
            }
        } else {
            JobOutput::ProcessError(String::from("No jobs to be ran"))
        }
    }
}
