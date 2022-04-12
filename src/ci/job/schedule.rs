use crate::ci::ci_config::CiConfig;
use crate::ci::job::dag::{Dag, JobResult, JobState};
use crate::ci::job::docker_job::DockerJob;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::{SystemFacade, UserFacade};
use crate::ci::job::shell_interpreter::ShellInterpreter;
use crate::ci::job::simple_job::SimpleJob;
use crate::ci::job::{JobProgressTracker, JobType, Progress};
use std::sync::mpsc::{channel, Receiver, TryRecvError};

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

    let jobs = ci_config
        .jobs
        .iter()
        .cloned()
        .map(|job| match job.image {
            None => JobType::Simple(SimpleJob::long(job.name, job.script, job.group)),
            Some(image) => JobType::Docker(DockerJob::long(job.name, job.script, image, job.group)),
        })
        .collect::<Vec<JobType>>();

    let mut jobs = Dag::new(&jobs, &ci_config.constraints, &ci_config.groups, &env).unwrap();

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
    use crate::ci::job::ports::CommandRunner;
    use crate::ci::job::{JobOutput, SharedJob};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::mpsc::Sender;

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
