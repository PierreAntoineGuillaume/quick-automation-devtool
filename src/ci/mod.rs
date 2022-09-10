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
use crate::ci::job::{Output, ProgressConsumer, Shared};
use crate::config::{Config, Payload};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::{Duration, SystemTime};

pub mod config;
pub mod display;
pub mod job;

pub struct Ci {}

impl Ci {
    pub fn run(config: &Config, cli_option: &CliOption) -> Result<bool> {
        let mut payload = Payload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut stdout = std::io::stdout();
        let output_is_non_interactive = !atty::is(atty::Stream::Stdout);

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
            println!("{}", name);
        }
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
    fn consume_job(&mut self, job: Arc<Shared>, tx: Sender<JobProgress>) {
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
            std::env::set_var(key, vals.join("\n"));
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
    fn run(&self, args: &str) -> Output {
        use std::str::from_utf8;
        let default_shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        let mut cmd = Command::new(&default_shell);

        let mut child = match cmd
            .args(["-c", args])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(p) => p,
            Err(e) => return Output::ProcessError(e.to_string()),
        };

        let mut out = String::new();
        let mut err = String::new();

        let mut res: Option<ExitStatus> = None;

        {
            let mut stdout = match child.stdout.take() {
                Some(out) => BufReader::new(out),
                None => return Output::ProcessError(String::from("no Child stdout")),
            };
            let mut stderr = match child.stderr.take() {
                Some(err) => BufReader::new(err),
                None => return Output::ProcessError(String::from("no Child output")),
            };

            while res.is_none() {
                sleep(Duration::from_millis(10));
                let (stdout_bytes, stderr_bytes) = match (stdout.fill_buf(), stderr.fill_buf()) {
                    (Ok(stdout), Ok(stderr)) => {
                        out.push_str(from_utf8(stdout).expect("error from UTF8"));
                        err.push_str(from_utf8(stderr).expect("error from UTF8"));

                        (stdout.len(), stderr.len())
                    }
                    e => return Output::ProcessError(format!("Error: {:#?}", e)),
                };

                if let Ok(proc) = child.try_wait() {
                    res = proc;
                }

                stdout.consume(stdout_bytes);
                stderr.consume(stderr_bytes);
            }
        }

        if res.expect("already opened").success() {
            Output::Success(out, err)
        } else {
            Output::JobError(out, err)
        }
    }
}
