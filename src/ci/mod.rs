use crate::ci::ci_config::CliConfig;
use crate::ci::display::full_final_display::FullFinalDisplay;
use crate::ci::display::silent_display::SilentDisplay;
use crate::ci::display::summary_display::SummaryDisplay;
use crate::ci::display::Mode;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::{CommandRunner, FinalCiDisplay, SystemFacade, UserFacade};
use crate::ci::job::schedule::schedule;
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

pub mod ci_config;
pub mod display;
pub mod job;

pub struct Ci {}

impl Ci {
    pub fn run(&mut self, config: Config, cli_config: CliConfig) -> Result<bool> {
        let mut payload = ConfigPayload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut stdout = std::io::stdout();

        let mut display: Box<dyn UserFacade> = if !atty::is(atty::Stream::Stdout) {
            Box::new(SilentDisplay {})
        } else {
            match &payload.display.mode {
                Mode::Silent => Box::new(SilentDisplay {}),
                Mode::AllOutput => Box::new(SequenceDisplay::new(&payload.display, &mut stdout)),
                Mode::Summary => Box::new(SummaryDisplay::new(&payload.display, &mut stdout)),
            }
        };

        let tracker = schedule(
            cli_config,
            ci_config,
            &mut ParrallelJobStarter::new(),
            &mut *display,
            payload.env,
        )?;

        (&mut FullFinalDisplay::new(&payload.display) as &mut dyn FinalCiDisplay).finish(&tracker);

        Ok(!tracker.has_failed)
    }

    pub fn list(&mut self, config: Config) -> Result<()> {
        let mut payload = ConfigPayload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut jobs = ci_config
            .jobs
            .iter()
            .cloned()
            .map(|job| job.name)
            .collect::<Vec<String>>();

        jobs.sort();

        jobs.iter().for_each(|name| println!("{}", name));
        ci_config
            .groups
            .iter()
            .for_each(|name| println!("group:{}", name));

        Ok(())
    }
}

pub struct ParrallelJobStarter {
    threads: Vec<JoinHandle<()>>,
    last_occurence: SystemTime,
}

const AWAIT_TIME: Duration = Duration::from_millis(40);

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
    fn run(&self, args: &str) -> JobOutput {
        CommandJobRunner {}.run(args)
    }
}

impl SystemFacade for ParrallelJobStarter {
    fn consume_job(&mut self, job: Arc<SharedJob>, tx: Sender<JobProgress>) {
        self.threads.push(thread::spawn(move || {
            job.start(&mut CommandJobRunner, &tx);
        }));
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

    fn read_env(&self, key: &str, default: Option<&str>) -> Result<String> {
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
    fn run(&self, args: &str) -> JobOutput {
        let default_shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        match Command::new(&default_shell).args(["-c", args]).output() {
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
