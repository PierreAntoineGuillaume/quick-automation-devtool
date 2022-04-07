use crate::ci::display::full_final_display::FullFinalDisplay;
use crate::ci::display::silent_display::SilentDisplay;
use crate::ci::display::summary_display::SummaryDisplay;
use crate::ci::display::{CiDisplayConfig, Mode};
use crate::ci::job::dag::Dag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::{schedule, CommandRunner, FinalCiDisplay, SystemFacade, UserFacade};
use crate::ci::job::{JobOutput, JobProgressConsumer, SharedJob};
use crate::config::{Config, ConfigPayload};
use crate::SequenceDisplay;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

pub mod display;
pub mod job;

#[derive(Default)]
pub struct CiConfig {
    pub jobs: Vec<Arc<SharedJob>>,
    pub groups: Vec<String>,
    pub constraints: Vec<(String, String)>,
    pub display: CiDisplayConfig,
}

pub struct Ci {}

impl Ci {
    pub fn run(&mut self, config: Config) -> Result<bool> {
        let mut payload = ConfigPayload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut display: Box<dyn UserFacade> = match &ci_config.display.mode {
            Mode::Silent => Box::new(SilentDisplay {}),
            Mode::AllOutput => Box::new(SequenceDisplay::new(&ci_config.display)),
            Mode::Summary => Box::new(SummaryDisplay::new(&ci_config.display)),
        };

        let dag = Dag::new(&ci_config.jobs, &ci_config.constraints, &ci_config.groups).unwrap();

        let tracker = schedule(
            dag,
            &mut ParrallelJobStarter::new(),
            &mut *display,
            payload.env,
        )?;

        (&mut FullFinalDisplay::new(&ci_config.display) as &mut dyn FinalCiDisplay)
            .finish(&tracker);

        Ok(!tracker.has_failed)
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

impl CommandRunner for ParrallelJobStarter {
    fn run(&self, args: &[&str]) -> JobOutput {
        CommandJobRunner {}.run(args)
    }
}

impl SystemFacade for ParrallelJobStarter {
    fn consume_some_jobs(&mut self, jobs: &mut Dag, tx: Sender<JobProgress>) {
        while let Some(job) = jobs.poll() {
            let consumer = tx.clone();
            let arc: Arc<SharedJob> = job.clone();
            self.threads.push(thread::spawn(move || {
                arc.start(&mut CommandJobRunner, &consumer);
            }));
        }
    }

    fn join(&mut self) {
        while let Some(handle) = self.threads.pop() {
            handle.join().expect("Could not join handle")
        }
    }

    fn delay(&mut self) -> usize {
        let time_for = match AWAIT_TIME.checked_sub(self.last_occurence.elapsed().unwrap()) {
            None => Duration::new(0, 0),
            Some(duration) => duration,
        };

        let millis: usize = std::cmp::max(time_for.as_millis() as usize, 0);
        if millis != 0 {
            sleep(time_for);
        }
        self.last_occurence = SystemTime::now();
        millis
    }

    fn write_env(&self, env: HashMap<String, Vec<String>>) {
        for (key, vals) in env {
            std::env::set_var(key, vals.join("\n"))
        }
    }

    fn read_env(&self, key: &str, default: Option<&str>) -> anyhow::Result<String> {
        if let Ok(env) = std::env::var(key) {
            Ok(env)
        } else if let Some(env) = default {
            Ok(env.to_string())
        } else {
            Err(anyhow!("no env value for {}", key))
        }
    }
}

pub struct CommandJobRunner;

impl CommandRunner for CommandJobRunner {
    fn run(&self, args: &[&str]) -> JobOutput {
        let program = args[0];
        let args: Vec<String> = args.iter().skip(1).map(|str| str.to_string()).collect();

        match Command::new(program).args(args).output() {
            Ok(output) => {
                let stdout = String::from(std::str::from_utf8(&output.stdout).unwrap());
                let stderr = String::from(std::str::from_utf8(&output.stderr).unwrap());
                if output.status.success() {
                    JobOutput::Success(stdout, stderr)
                } else {
                    JobOutput::JobError(stdout, stderr)
                }
            }
            Err(e) => JobOutput::ProcessError(e.to_string()),
        }
    }
}
