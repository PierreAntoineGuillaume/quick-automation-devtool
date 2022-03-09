use std::fmt::{Display, Formatter};
use super::*;

#[derive(Default)]
struct JobTester {
    program: String,
    args: Vec<String>,
}

impl JobTester {
    fn run_job(instruction: &str, shell: Option<String>, image: Option<String>) -> String {
        let mut tester = JobTester::default();
        let job = Job {
            name: "name".to_string(),
            shell,
            image,
            instructions: vec![],
        };


        job.run(instruction, &mut tester);

        format!("{tester}")
    }
}

impl Display for JobTester {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.program)?;
        write!(f, "[")?;
        let mut len = self.args.len();
        for arg in &self.args {
            write!(f, "{}", arg)?;
            len-=1;
            if len > 0 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}


impl JobRunner for JobTester {
    fn run(&mut self, program: &str, args: &[&str], _: &str) -> JobOutput {
        self.program = program.to_string();
        self.args = args.iter().map(|str| str.to_string()).collect();
        JobOutput::ProcessError(String::default())
    }
}

#[test]
pub fn simple_job() {
    assert_eq!(
        JobTester::run_job("task", None, None).to_string(),
        "task\n[]"
    );
}

#[test]
pub fn simple_jobs_with_args() {
    assert_eq!(
        JobTester::run_job("task with args", None, None).to_string(),
        "task\n[with, args]"
    )
}

#[test]
pub fn bash_jobs_with_args() {
    assert_eq!(
        JobTester::run_job("task with args", Some("bash".to_string()), None).to_string(),
        "bash\n[-c, task with args]"
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
    Job {
        name: name.to_string(),
        shell: None,
        image: None,
        instructions: vec![],
    }
}

pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
    (blocking.to_string(), blocked.to_string())
}
