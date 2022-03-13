use super::*;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};

#[derive(Default)]
struct JobInput {
    program: String,
    args: Vec<String>,
}

#[derive(Default)]
struct JobTester {
    inputs: RefCell<JobInput>,
}

impl JobTester {
    fn run_job(instruction: &str, shell: Option<String>, image: Option<String>) -> String {
        let mut tester = JobTester::default();
        let job = Job::long("name".to_string(), vec![], shell, image);

        job.run(instruction, &mut tester);

        format!("{tester}")
    }
}

impl Display for JobInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.program)?;
        write!(f, "[")?;
        let mut len = self.args.len();
        for arg in &self.args {
            write!(f, "{}", arg)?;
            len -= 1;
            if len > 0 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl Display for JobTester {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inputs.borrow())
    }
}

impl JobRunner for JobTester {
    fn run(&self, program: &str, args: &[&str]) -> JobOutput {
        let mut input = self.inputs.borrow_mut();
        input.program = program.to_string();
        input.args = args.iter().map(|str| str.to_string()).collect();
        JobOutput::ProcessError(String::default())
    }
}

#[test]
pub fn simple_job() {
    assert_eq!(JobTester::run_job("task", None, None), "task\n[]");
}

#[test]
pub fn simple_jobs_with_args() {
    assert_eq!(
        JobTester::run_job("task with args", None, None),
        "task\n[with, args]"
    )
}

#[test]
pub fn bash_jobs_with_args() {
    assert_eq!(
        JobTester::run_job("task with args", Some("bash".to_string()), None),
        "bash\n[-c, task with args]"
    )
}

#[test]
pub fn docker_jobs_with_args() {
    assert_eq!(
        JobTester::run_job("task with args", None, Some("alpine".to_string())),
        "docker\n[run, --rm, --user, $USER:$GROUPS, --volume, $PWD:$PWD, --workdir, $PWD, alpine, task, with, args]"
    )
}

pub fn simple_job_schedule() -> (Vec<Job>, Vec<(String, String)>) {
    let jobs = vec![job("deploy"), job("build"), job("test")];

    let constraints = vec![cons("build", "test"), cons("test", "deploy")];

    (jobs, constraints)
}

pub fn complex_job_schedule() -> (Vec<Job>, Vec<(String, String)>) {
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

    (jobs, constraints)
}

pub fn job(name: &str) -> Job {
    Job::short(name.to_string(), vec![])
}

pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
    (blocking.to_string(), blocked.to_string())
}
