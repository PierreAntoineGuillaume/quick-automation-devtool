#[cfg(test)]
pub mod tests {
    use crate::ci::job::Job;

    pub fn complex_list() -> (Vec<Job>, Vec<(String, String)>) {
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
            instructions: vec![],
        }
    }

    pub fn cons(blocking: &str, blocked: &str) -> (String, String) {
        (blocking.to_string(), blocked.to_string())
    }
}
