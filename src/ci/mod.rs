use crate::ci::config::CliOption;
use crate::ci::display::exhaustive::FullFinalDisplay;
use crate::ci::display::interactive::Interactive;
use crate::ci::display::sequence::Display as SequenceDisplay;
use crate::ci::display::silent::Display as SilentDisplay;
use crate::ci::display::summary::Display as SummaryDisplay;
use crate::ci::display::{FinalDisplayMode, Running};
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::{CommandRunner, FinalCiDisplay, SystemFacade, UserFacade};
use crate::ci::job::schedule::schedule;
use crate::ci::job::Job;
use crate::ci::job::{Output, ProgressConsumer};
use crate::config::{Config, Payload};
use anyhow::Result;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

pub mod clean;
pub mod config;
pub mod display;
pub mod job;

pub struct Ci {}

impl Ci {
    pub fn debug(config: &Config, job: String) -> Result<bool> {
        println!("Debuging {job}");
        let mut payload = Payload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut display: Box<dyn UserFacade> = Box::new(SilentDisplay {});

        let cli_option = CliOption {
            job: Some(job),
            ..Default::default()
        };

        let tracker = schedule(
            &cli_option,
            ci_config,
            &mut DebugJobStarter::default(),
            &mut *display,
            payload.env,
        )?;

        Ok(!tracker.has_failed)
    }

    pub fn run(config: &Config, cli_option: &CliOption) -> Result<bool> {
        let mut payload = Payload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut stdout = std::io::stdout();
        let output_is_non_interactive = cli_option.no_tty || !atty::is(atty::Stream::Stdout);

        let mut display: Box<dyn UserFacade> = if output_is_non_interactive {
            Box::new(SilentDisplay {})
        } else {
            match &payload.display.running_display {
                Running::Silent => Box::new(SilentDisplay {}),
                Running::Sequence => Box::new(SequenceDisplay::new(&payload.display, &mut stdout)),
                Running::Summary => Box::new(SummaryDisplay::new(&payload.display, &mut stdout)),
            }
        };

        let tracker = schedule(
            cli_option,
            ci_config,
            &mut ParrallelJobStarter::new(),
            &mut *display,
            payload.env,
        )?;

        let mut display: Box<dyn FinalCiDisplay> = match payload.display.final_display {
            FinalDisplayMode::Silent => Box::new(SilentDisplay {}),
            FinalDisplayMode::Full => Box::new(FullFinalDisplay::new(&payload.display)),
            FinalDisplayMode::Interactive => {
                if output_is_non_interactive {
                    Box::new(FullFinalDisplay::new(&payload.display))
                } else {
                    Box::new(Interactive::new(&payload.display))
                }
            }
        };

        display.finish(&tracker);

        Ok(!tracker.has_failed)
    }

    pub fn list(config: &Config) -> Result<()> {
        let mut payload = Payload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut jobs = ci_config
            .jobs
            .iter()
            .cloned()
            .map(|job| job.name)
            .collect::<Vec<String>>();

        jobs.sort();

        for name in jobs {
            println!("{name}");
        }
        ci_config
            .groups
            .iter()
            .for_each(|name| println!("group:{name}"));

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct DebugJobStarter {}

impl CommandRunner for DebugJobStarter {
    fn run(&self, args: &str) -> Output {
        std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        println!("Command: {args}");
        let default_shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        let process = Command::new(default_shell)
            .args(["-xc", args])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();
        println!();
        match process {
            Ok(output) => {
                if output.status.success() {
                    Output::Success(String::default(), String::default())
                } else {
                    Output::JobError(String::default(), String::default())
                }
            }
            Err(_) => Output::ProcessError(String::default()),
        }
    }
}

impl SystemFacade for DebugJobStarter {
    fn consume_job(&mut self, job: Job, tx: Sender<JobProgress>) {
        eprintln!("Consuming job: {}", job.name());
        job.start(self, &tx);
    }

    fn delay(&mut self) -> usize {
        sleep(Duration::from_millis(200));
        200
    }

    fn write_env(&self, env: HashMap<String, Vec<String>>) {
        for (key, vals) in env {
            std::env::set_var(key, vals.join("\n"));
        }
    }
}

pub struct ParrallelJobStarter {
    threads: Vec<JoinHandle<()>>,
    last_occurence: SystemTime,
}

const AWAIT_TIME: Duration = Duration::from_millis(40);

impl ParrallelJobStarter {
    pub fn new() -> Self {
        Self {
            threads: vec![],
            last_occurence: SystemTime::now(),
        }
    }
}

impl ProgressConsumer for Sender<JobProgress> {
    fn consume(&self, job_progress: JobProgress) {
        self.send(job_progress).unwrap();
    }
}

impl CommandRunner for ParrallelJobStarter {
    fn run(&self, args: &str) -> Output {
        CommandJobRunner {}.run(args)
    }
}

impl SystemFacade for ParrallelJobStarter {
    fn consume_job(&mut self, job: Job, tx: Sender<JobProgress>) {
        self.threads.push(thread::spawn(move || {
            job.start(&CommandJobRunner, &tx);
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
            std::env::set_var(key, vals.join("\n"));
        }
    }
}

fn mute(args: &str) -> Output {
    let default_shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    match Command::new(default_shell).args(["-c", args]).output() {
        Ok(output) => {
            let stdout = String::from(std::str::from_utf8(&output.stdout).unwrap());
            let stderr = String::from(std::str::from_utf8(&output.stderr).unwrap());
            if output.status.success() {
                Output::Success(stdout, stderr)
            } else {
                Output::JobError(stdout, stderr)
            }
        }
        Err(e) => Output::ProcessError(e.to_string()),
    }
}

pub struct CommandJobRunner;

impl CommandRunner for CommandJobRunner {
    fn run(&self, args: &str) -> Output {
        mute(args)
    }
}
