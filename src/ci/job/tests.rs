use super::*;
use crate::ci::job::simple::Simple;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};

#[derive(Default)]
struct JobInput {
    args: Option<String>,
}

#[derive(Default)]
struct JobTester {
    inputs: RefCell<Vec<JobInput>>,
}

impl Display for JobInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.args.clone().unwrap_or_default())?;
        write!(f, "(")?;
        if let Some(arg) = &self.args {
            write!(f, "{arg}")?;
        }
        write!(f, ")")
    }
}

impl Display for JobTester {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for job in self.inputs.borrow().iter() {
            writeln!(f, "{job}")?;
        }
        Ok(())
    }
}

impl CommandRunner for JobTester {
    fn precondition(&self, args: &str) -> Output {
        self.run(args)
    }
    fn run(&self, args: &str) -> Output {
        self.inputs.borrow_mut().push(JobInput {
            args: Some(args.to_string()),
        });
        Output::ProcessError(String::default())
    }
}

pub type ScheduleType = (Vec<Type>, Vec<(String, String)>, Vec<String>);

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

pub fn job(name: &str) -> Type {
    Type::Simple(Simple::long(name.to_string(), vec![], None, None))
}

fn job_group(name: &str, group: &str) -> Type {
    Type::Simple(Simple::long(
        name.to_string(),
        vec![],
        Some(group.to_string()),
        None,
    ))
}

pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
    (blocking.to_string(), blocked.to_string())
}
