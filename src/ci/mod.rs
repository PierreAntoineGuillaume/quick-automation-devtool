use crate::ci::display::full_final_display::FullFinalDisplay;
use crate::ci::display::silent_display::SilentDisplay;
use crate::ci::display::summary_display::SummaryDisplay;
use crate::ci::display::{CiDisplayConfig, Mode};
use crate::ci::job::dag::Dag;
use crate::ci::job::env_bag::{EnvBag, SimpleEnvBag};
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::{schedule, FinalCiDisplay, JobRunner, JobStarter, RunningCiDisplay};
use crate::ci::job::{JobOutput, JobProgressConsumer, SharedJob};
use crate::config::{Config, ConfigPayload};
use crate::SequenceDisplay;
use std::collections::HashMap;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
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
    fn id(flag: &str) -> String {
        let output = Command::new("id")
            .args(vec![flag])
            .output()
            .expect("id -u cannot fail");
        String::from(std::str::from_utf8(&output.stdout).expect("This is an int"))
            .trim()
            .to_string()
    }

    pub fn run(&mut self, config: Config) -> Result<(), Option<String>> {
        let mut payload = ConfigPayload::default();
        config.load_with_args_into(&mut payload)?;
        let ci_config = payload.ci;

        let mut display: Box<dyn RunningCiDisplay> = match &ci_config.display.mode {
            Mode::Silent => Box::new(SilentDisplay {}),
            Mode::AllOutput => Box::new(SequenceDisplay::new(&ci_config.display)),
            Mode::Summary => Box::new(SummaryDisplay::new(&ci_config.display)),
        };

        let dag = Dag::new(&ci_config.jobs, &ci_config.constraints, &ci_config.groups).unwrap();

        let uid = Self::id("-u");
        let gid = Self::id("-g");
        let pwd = std::env::var("PWD").expect("PWD always exists");
        let envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>> = Arc::from(Mutex::new(
            SimpleEnvBag::new(uid, gid, pwd, HashMap::default()),
        ));

        let tracker = schedule(dag, &mut ParrallelJobStarter::new(), &mut *display, envbag);

        (&mut FullFinalDisplay::new(&ci_config.display) as &mut dyn FinalCiDisplay)
            .finish(&tracker);

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
    fn consume_some_jobs(
        &mut self,
        jobs: &mut Dag,
        envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        tx: Sender<JobProgress>,
    ) {
        while let Some(job) = jobs.poll() {
            let consumer = tx.clone();
            let arc: Arc<SharedJob> = job.clone();
            let envbag = envbag.clone();
            self.threads.push(thread::spawn(move || {
                arc.start(&mut CommandJobRunner, envbag, &consumer);
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

pub struct CommandJobRunner;

impl JobRunner for CommandJobRunner {
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
            Err(e) => JobOutput::ProcessError(format!("{}", e)),
        }
    }
}
