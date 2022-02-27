use crate::ci::job::dag::Dag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::{JobRunner, JobStarter, Pipeline};
use crate::ci::job::JobOutput;
use crate::{Config, TermCiDisplay};
use job::{Job, JobProgressConsumer};
use regex::Regex;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

pub mod display;
pub mod job;

pub struct CiConfig {
    pub jobs: Vec<Job>,
    pub constraints: Vec<(String, String)>,
    pub spinner: (Vec<String>, usize),
}

impl CiConfig {
    fn new() -> Self {
        CiConfig {
            jobs: Vec::new(),
            constraints: Vec::new(),
            spinner: (
                vec![".  ", " . ", "  .", " . ", "...", "   "]
                    .iter()
                    .map(|str| str.to_string())
                    .collect(),
                80,
            ),
        }
    }
}

pub struct Ci {}

impl Ci {
    pub fn run(&mut self, config: Config) -> Result<(), ()> {
        let mut ci_config = CiConfig::new();
        config.load_into(&mut ci_config);

        let mut starter = ParrallelJobStarter::new();

        let mut display = TermCiDisplay::new(&ci_config.spinner.0, ci_config.spinner.1);

        let mut pipeline = Pipeline {};

        let dag = Dag::new(&ci_config.jobs, &ci_config.constraints).unwrap();

        let tracker = pipeline.schedule(dag, &mut starter, &mut display);

        display.finish(&tracker);

        if tracker.has_failed {
            Err(())
        } else {
            Ok(())
        }
    }
}

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
    fn consume_some_jobs(&mut self, jobs: &mut Dag, tx: Sender<JobProgress>) {
        while let Some(job) = jobs.poll() {
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
