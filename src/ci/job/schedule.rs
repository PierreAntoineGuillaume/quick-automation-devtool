use crate::ci::job::dag::{Dag, JobResult, JobState};
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::shell_interpreter::ShellInterpreter;
use crate::ci::job::{JobOutput, JobProgressTracker, Progress, SharedJob};
use crate::ci::CiConfig;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::Arc;

pub trait CommandRunner {
    fn run(&self, args: &[&str]) -> JobOutput;
}

pub trait SystemFacade: CommandRunner {
    fn consume_job(&mut self, jobs: Arc<SharedJob>, tx: Sender<JobProgress>);
    fn join(&mut self);
    fn delay(&mut self) -> usize;
    fn write_env(&self, env: HashMap<String, Vec<String>>);
    fn read_env(&self, key: &str, default: Option<&str>) -> anyhow::Result<String>;
}

pub trait UserFacade {
    fn set_up(&mut self, tracker: &JobProgressTracker);
    fn run(&mut self, tracker: &JobProgressTracker, elapsed: usize);
    fn tear_down(&mut self, tracker: &JobProgressTracker);
    fn display_error(&self, error: String);
}

pub trait FinalCiDisplay {
    fn finish(&mut self, tracker: &JobProgressTracker);
}

pub fn schedule(
    ci_config: CiConfig,
    system_facade: &mut dyn SystemFacade,
    user_facade: &mut dyn UserFacade,
    envtext: Option<String>,
) -> anyhow::Result<JobProgressTracker> {
    let mut tracker = JobProgressTracker::new();

    let env = {
        let parser = ShellInterpreter::new(user_facade, system_facade);
        parser.interpret(envtext)?
    };

    let mut jobs = Dag::new(
        &ci_config.jobs,
        &ci_config.constraints,
        &ci_config.groups,
        &env
    ).unwrap();

    if jobs.is_finished() {
        tracker.finish();
        return Ok(tracker);
    }

    system_facade.write_env(env);

    for job in jobs.enumerate() {
        tracker.record(JobProgress::new(
            &job.name,
            match job.state {
                JobState::Pending => Progress::Available,
                JobState::Blocked => Progress::Blocked(job.block.clone()),
                _ => {
                    unreachable!("This state is impossible with no poll yet")
                }
            },
        ))
    }

    let (tx, rx) = channel();

    user_facade.set_up(&tracker);

    let mut delay: usize = 0;
    loop {
        while let Some(job) = jobs.poll() {
            system_facade.consume_job(job.clone(), tx.clone());
        }

        while let Some(progress) = read(&rx) {
            let mut cancel_list: Vec<String> = vec![];
            let name = progress.name();
            if let Progress::Terminated(success) = progress.1 {
                jobs.record_event(
                    name,
                    if success {
                        JobResult::Success
                    } else {
                        JobResult::Failure
                    },
                );
                if !success {
                    cancel_list = jobs
                        .enumerate()
                        .iter()
                        .filter(|job| matches!(job.state, JobState::Cancelled(_)))
                        .map(|job| job.name.clone())
                        .collect();
                }
            }
            tracker.record(progress);
            for cancel in cancel_list {
                tracker.record(JobProgress::cancel(cancel))
            }
        }

        if jobs.is_finished() {
            tracker.finish();
            break;
        }
        user_facade.run(&tracker, delay);
        delay = system_facade.delay();
    }

    system_facade.join();
    user_facade.tear_down(&tracker);

    Ok(tracker)
}

pub fn read(rx: &Receiver<JobProgress>) -> Option<JobProgress> {
    match rx.try_recv() {
        Ok(state) => Some(state),
        Err(TryRecvError::Empty) => None,
        Err(TryRecvError::Disconnected) => {
            panic!("State receiver has been disconnected, try restarting the program");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::display::silent_display::SilentDisplay;
    use crate::ci::job::simple_job::SimpleJob;
    use crate::ci::job::SharedJob;
    use std::collections::HashMap;
    use std::sync::Arc;

    impl SimpleJob {
        pub fn new(name: &str, instructions: &[&str]) -> Self {
            SimpleJob::short(
                name.to_string(),
                instructions
                    .iter()
                    .map(|item| String::from(*item))
                    .collect(),
            )
        }
    }

    #[test]
    pub fn every_job_is_initialisated() {
        assert!(!test_pipeline(&[Arc::from(SimpleJob::new("a", &["ok: result"]))]).has_failed)
    }

    #[test]
    pub fn one_job_failure_fails_scheduling() {
        assert!(test_pipeline(&[Arc::from(SimpleJob::new("c", &["ko: result"]))]).has_failed)
    }

    #[test]
    pub fn one_job_crash_fails_scheduling() {
        assert!(test_pipeline(&[Arc::from(SimpleJob::new("c", &["crash: result"]))]).has_failed)
    }

    pub struct TestJobStarter {}

    impl CommandRunner for TestJobStarter {
        fn run(&self, _: &[&str]) -> JobOutput {
            JobOutput::Success("".to_string(), "".to_string())
        }
    }

    impl SystemFacade for TestJobStarter {
        fn consume_job(&mut self, job: Arc<SharedJob>, tx: Sender<JobProgress>) {
            job.start(&mut TestJobRunner {}, &tx);
        }

        fn join(&mut self) {}

        fn delay(&mut self) -> usize {
            0
        }

        fn write_env(&self, _: HashMap<String, Vec<String>>) {}

        fn read_env(&self, _: &str, _: Option<&str>) -> anyhow::Result<String> {
            Ok("".to_string())
        }
    }

    pub struct TestJobRunner {}
    impl CommandRunner for TestJobRunner {
        fn run(&self, args: &[&str]) -> JobOutput {
            let job = args[2].to_string();
            if let Some(stripped) = job.strip_prefix("ok:") {
                JobOutput::Success(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("ko:") {
                JobOutput::JobError(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("crash:") {
                JobOutput::ProcessError(stripped.into())
            } else {
                unreachable!(
                    "Job should begin with ok:, ko, or crash: (actual: '{}')",
                    job
                )
            }
        }
    }
}
