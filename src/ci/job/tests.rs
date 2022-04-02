use super::*;
use crate::ci::job::docker_job::DockerJob;
use crate::ci::job::env_bag::{EnvBag, SimpleEnvBag};
use crate::ci::job::simple_job::SimpleJob;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Default)]
struct JobInput {
    args: Vec<String>,
}

#[derive(Default)]
struct JobTester {
    inputs: RefCell<Vec<JobInput>>,
}

impl JobTester {
    pub fn run_job<T: Into<String>>(instruction: &str, image: T) -> String {
        let tester = JobTester::default();
        let envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>> =
            Arc::from(Mutex::new(SimpleEnvBag::new("uid", "gid", "/dir", vec![])));
        let job = DockerJob::long("name".to_string(), vec![], image.into(), None);
        job.run(instruction, &tester, &envbag);
        format!("{tester}")
    }
}

impl Display for JobInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.args[0])?;
        write!(f, "(")?;
        let mut len = self.args.len() - 1;
        for arg in self.args.iter().skip(1) {
            write!(f, "{}", arg)?;
            len -= 1;
            if len > 0 {
                write!(f, ", ")?;
            }
        }
        write!(f, ")")
    }
}

impl Display for JobTester {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for job in self.inputs.borrow().iter() {
            writeln!(f, "{}", job)?;
        }
        Ok(())
    }
}

impl JobRunner for JobTester {
    fn run(&self, args: &[&str]) -> JobOutput {
        self.inputs.borrow_mut().push(JobInput {
            args: args.iter().map(|str| str.to_string()).collect(),
        });
        JobOutput::ProcessError(String::default())
    }
}

#[test]
pub fn docker_jobs_with_args() {
    assert_eq!(
        JobTester::run_job("task with args", "alpine"),
        "docker(run, --rm, --user, uid:gid, --volume, /dir:/dir, --workdir, /dir, alpine, task, with, args)\n"
    )
}

pub type ScheduleType = (Vec<Arc<SharedJob>>, Vec<(String, String)>, Vec<String>);

pub fn simple_job_schedule() -> ScheduleType {
    let jobs = vec![job("deploy"), job("build"), job("test")];

    let constraints = vec![cons("build", "test"), cons("test", "deploy")];

    (jobs, constraints, vec![])
}

pub fn complex_job_schedule() -> ScheduleType {
    let jobs = vec![
        job("deploy"),
        job("build1"),
        job("build2"),
        job("test1"),
        job("test2"),
    ];

    let constraints = vec![
        cons("build1", "test1"),
        cons("build1", "test2"),
        cons("build2", "test1"),
        cons("build2", "test2"),
        cons("test1", "deploy"),
        cons("test2", "deploy"),
    ];

    (jobs, constraints, vec![])
}

pub fn group_job_schedule() -> ScheduleType {
    let jobs = vec![
        job_group("build1", "build"),
        job_group("build2", "build"),
        job_group("test1", "test"),
        job_group("test2", "test"),
        job_group("deploy", "deploy"),
    ];

    let config = vec![
        "build".to_string(),
        "test".to_string(),
        "deploy".to_string(),
    ];

    (jobs, vec![], config)
}

pub fn job(name: &str) -> Arc<SharedJob> {
    Arc::from(SimpleJob::short(name.to_string(), vec![]))
}

fn job_group(name: &str, group: &str) -> Arc<SharedJob> {
    Arc::from(SimpleJob::long(
        name.to_string(),
        vec![],
        Some(group.to_string()),
    ))
}

pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
    (blocking.to_string(), blocked.to_string())
}
