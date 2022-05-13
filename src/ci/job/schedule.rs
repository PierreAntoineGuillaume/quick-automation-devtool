use crate::ci::config::{CliOption, Config};
use crate::ci::job::dag::{Dag, JobResult, JobState};
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::{SystemFacade, UserFacade};
use crate::ci::job::shell_interpreter::ShellInterpreter;
use crate::ci::job::{JobProgressTracker, Progress, Type};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

pub fn schedule(
    cli_option: &CliOption,
    ci_config: Config,
    system_facade: &mut dyn SystemFacade,
    user_facade: &mut dyn UserFacade,
    envtext: Option<String>,
) -> anyhow::Result<JobProgressTracker> {
    let env = {
        let parser = ShellInterpreter::new(user_facade, system_facade);
        parser.interpret(envtext)?
    };

    let jobs = if cli_option.job.is_none() {
        ci_config
            .jobs
            .iter()
            .cloned()
            .map(std::convert::Into::into)
            .collect::<Vec<Type>>()
    } else {
        let filter = cli_option.job.as_ref().unwrap();
        if let Some(group) = filter.strip_prefix("group:") {
            ci_config
                .jobs
                .iter()
                .cloned()
                .filter(|job| job.group.is_some() && group == job.group.as_ref().unwrap())
                .map(std::convert::Into::into)
                .collect::<Vec<Type>>()
        } else {
            ci_config
                .jobs
                .iter()
                .cloned()
                .filter(|job| filter == &job.name)
                .map(std::convert::Into::into)
                .collect::<Vec<Type>>()
        }
    };

    let constraints = if cli_option.job.is_none() {
        ci_config.constraints
    } else {
        vec![]
    };

    let mut jobs = Dag::new(&jobs, &constraints, &ci_config.groups, &env)?;

    let mut tracker = JobProgressTracker::new();

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
        ));
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
                tracker.record(JobProgress::cancel(cancel));
            }
        }

        if jobs.is_finished() {
            tracker.finish();
            break;
        }
        user_facade.run(&tracker, delay);
        delay = system_facade.delay();
    }

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
    use crate::ci::job::ports::CommandRunner;
    use crate::ci::job::{Output, Shared};
    use std::collections::HashMap;
    use std::sync::mpsc::Sender;
    use std::sync::Arc;

    pub struct TestJobStarter {}

    impl CommandRunner for TestJobStarter {
        fn run(&self, _: &str) -> Output {
            Output::Success("".to_string(), "".to_string())
        }
    }

    impl SystemFacade for TestJobStarter {
        fn consume_job(&mut self, job: Arc<Shared>, tx: Sender<JobProgress>) {
            job.start(&mut TestJobRunner {}, &tx);
        }

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
        fn run(&self, job: &str) -> Output {
            // clippy::option_if_let_else makes a bad suggestion
            // https://github.com/rust-lang/rust-clippy/issues/8829
            if let Some(stripped) = job.strip_prefix("ok:") {
                Output::Success(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("ko:") {
                Output::JobError(stripped.into(), "".into())
            } else if let Some(stripped) = job.strip_prefix("crash:") {
                Output::ProcessError(stripped.into())
            } else {
                panic!(
                    "Job should begin with ok:, ko, or crash: (actual: '{}')",
                    job
                )
            }
        }
    }
}
