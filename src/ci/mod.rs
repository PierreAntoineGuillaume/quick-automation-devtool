use crate::ci::display::{CiDisplayConfig, NullCiDisplay};
use crate::ci::job::dag::Dag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::{schedule, CiDisplay, JobRunner, JobStarter};
use crate::ci::job::JobOutput;
use crate::config::{Config, ConfigPayload};
use crate::TermCiDisplay;
use job::{Job, JobProgressConsumer};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

pub mod display;
pub mod job;

#[derive(Default)]
pub struct CiConfig {
    pub jobs: Vec<Job>,
    pub constraints: Vec<(String, String)>,
    pub display: CiDisplayConfig,
}

pub struct Ci {}

impl Ci {
    pub fn run(&mut self, config: Config) -> Result<(), Option<String>> {
        let mut payload = ConfigPayload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut display: Box<dyn CiDisplay> = if payload.quiet {
            Box::new(NullCiDisplay {})
        } else {
            Box::new(TermCiDisplay::new(&ci_config.display))
        };

        let dag = Dag::new(&ci_config.jobs, &ci_config.constraints).unwrap();

        let tracker = schedule(dag, &mut ParrallelJobStarter::new(), &mut *display);

        display.finish(&tracker);

        if tracker.has_failed {
            Err(None)
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
    fn run(&self, _: &Job, instruction: &str) -> JobOutput {
        let mut parts = instruction.split(' ');
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
                Err(e) => JobOutput::ProcessError(format!("{}: {}", instruction, e)),
            }
        } else {
            JobOutput::ProcessError(String::from("No jobs to be ran"))
        }
    }
}
