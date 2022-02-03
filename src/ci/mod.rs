use job::{Job, JobOutput, JobProgress, JobProgressConsumer, JobRunner};
use regex::Regex;
use schedule::JobStarter;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

pub(crate) mod config;
pub(crate) mod display;
pub(crate) mod job;
pub(crate) mod schedule;

pub struct ParrallelJobStarter {
    threads: std::vec::Vec<JoinHandle<()>>,
    last_occurence: SystemTime,
}

const AWAIT_TIME: Duration = std::time::Duration::from_millis(40);

impl ParrallelJobStarter {
    pub fn new() -> Self {
        ParrallelJobStarter {
            threads: vec![],
            last_occurence: SystemTime::now(),
        }
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

    fn delay(&mut self) -> usize {
        let time_for = AWAIT_TIME
            - SystemTime::now()
                .duration_since(self.last_occurence)
                .unwrap();
        let millis: usize = std::cmp::max(time_for.as_millis() as usize, 0);
        if millis != 0 {
            sleep(time_for);
        }
        self.last_occurence = SystemTime::now();
        millis
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
        let regex = Regex::new(r"\s+").unwrap();
        let mut parts = regex.split(job);
        if let Some(program) = parts.next() {
            let args: Vec<&str> = parts.into_iter().collect();
            match Command::new(&program).args(&args).output() {
                Ok(output) => {
                    let stdout = String::from(std::str::from_utf8(&output.stdout).unwrap());
                    let stderr = String::from(std::str::from_utf8(&output.stderr).unwrap());
                    if output.status.success() {
                        JobOutput::Success(stdout, stderr)
                    } else {
                        JobOutput::JobError(stdout, stderr)
                    }
                }
                Err(e) => JobOutput::ProcessError(format!("{}: {}", job, e)),
            }
        } else {
            JobOutput::ProcessError(String::from("No jobs to be ran"))
        }
    }
}
