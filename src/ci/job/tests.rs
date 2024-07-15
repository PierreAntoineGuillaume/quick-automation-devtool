use super::*;

pub type ScheduleType = (Vec<Job>, Vec<(String, String)>, Vec<String>);

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

pub fn job(name: &str) -> Job {
    Job::long(name.to_string(), vec![], None, None)
}

fn job_group(name: &str, group: &str) -> Job {
    Job::long(name.to_string(), vec![], Some(group.to_string()), None)
}

pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
    (blocking.to_string(), blocked.to_string())
}
