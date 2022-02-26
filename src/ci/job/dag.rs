use crate::ci::job::dag::constraint_matrix::ConstraintMatrix;
use crate::ci::job::Job;
use std::collections::BTreeMap;

pub mod constraint_matrix;
mod constraint_matrix_constraint_iterator;

#[derive(Debug)]
enum Constraint {
    Free,
    Indifferent,
    Blocked(usize),
}

impl Constraint {
    fn constrain(&self) -> Result<Self, ()> {
        match self {
            Constraint::Free => Err(()),
            Constraint::Indifferent => Ok(Constraint::Blocked(1)),
            Constraint::Blocked(usize) => Ok(Constraint::Blocked(*usize + 1usize)),
        }
    }
}

impl Default for Constraint {
    fn default() -> Self {
        Self::Indifferent
    }
}

#[derive(Debug)]
pub enum DagError {
    JobCannotBlockItself(String),
    UnknownJobInConstraint(String),
    CycleExistsBecauseOf(String),
}

#[derive(Debug)]
pub enum JobResult {
    Success,
    Failure,
}

#[derive(Debug)]
pub enum JobState {
    Pending,
    Started,
    Blocked,
    Terminated(JobResult),
    Cancelled(String),
}

impl Default for JobState {
    fn default() -> Self {
        JobState::Pending
    }
}

#[derive(Debug)]
pub struct JobList {
    vec: Vec<String>,
}

impl JobList {
    pub fn from(vec: Vec<String>) -> Self {
        JobList {
            vec: vec.iter().rev().cloned().collect(),
        }
    }

    pub fn new() -> Self {
        JobList { vec: Vec::new() }
    }

    pub fn remove_job(&mut self, name: &str) {
        self.vec.retain(|contained| contained != name)
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn shift(&mut self) -> Option<String> {
        self.vec.pop()
    }
}

#[derive(Debug)]
pub struct JobWatcher {
    job: Job,
    state: JobState,
    blocks_job: Vec<String>,
    blocked_by_jobs: JobList,
}

impl JobWatcher {
    pub fn new(
        job: &Job,
        state: JobState,
        blocks_job: Vec<String>,
        blocked_by_jobs: JobList,
    ) -> Self {
        JobWatcher {
            job: job.clone(),
            state,
            blocks_job,
            blocked_by_jobs,
        }
    }
}

pub struct Dag {
    all_jobs: BTreeMap<String, JobWatcher>,
    available_jobs: JobList,
}

impl Dag {
    pub fn new(jobs: &[Job], constraints: &[(String, String)]) -> Result<Self, DagError> {
        let jobs: Vec<Job> = jobs.to_vec();
        let matrix = ConstraintMatrix::new(&jobs, constraints)?;

        let mut all_jobs = BTreeMap::<String, JobWatcher>::new();
        let available_jobs = JobList::new();

        for job in &jobs {
            let blocking = matrix.blocked_by(&job.name);
            let blocked_by_jobs: Vec<String> = matrix.blocking(&job.name).collect();
            let state = if blocked_by_jobs.is_empty() {
                JobState::Pending
            } else {
                for blocked_by_job in &blocked_by_jobs {
                    if blocked_by_job == &job.name {
                        return Err(DagError::CycleExistsBecauseOf(blocked_by_job.clone()));
                    }
                }
                JobState::Blocked
            };
            all_jobs.insert(
                job.name.to_string(),
                JobWatcher::new(
                    job,
                    state,
                    blocking.collect(),
                    JobList::from(blocked_by_jobs),
                ),
            );
        }

        let mut dag = Dag {
            all_jobs,
            available_jobs,
        };

        dag.actualize_job_list();

        Ok(dag)
    }

    /// Poll will return a job if a job is available
    /// Available jobs are Pending
    /// When a job is polled, it is considered Started
    pub fn poll(&mut self) -> Option<Job> {
        let jobname = self.available_jobs.shift()?;
        let job = self.all_jobs.get_mut(&jobname);

        debug_assert!(
            job.is_some(),
            "There is an unknown job {:?} in all_jobs",
            &jobname
        );

        let job = job.unwrap();
        debug_assert!(
            matches!(job.state, JobState::Pending),
            "Logic error : job {:?} should be {:?}, is actually {:?}",
            jobname,
            JobState::Pending,
            job.state
        );
        job.state = JobState::Started;
        Some(job.job.clone())
    }

    /// After being issued a job by `poll`
    /// inform the Dag of the result of the job `job`
    /// with `result` either:
    /// JobResult::Failure or JobResult::Success
    pub fn record_event(&mut self, job: &str, result: JobResult) {
        let job_went_wrong = matches!(result, JobResult::Failure);

        if let Some(watcher) = self.all_jobs.get_mut(job) {
            if !matches!(watcher.state, JobState::Started) {
                panic!(
                    "bad state {:?} for job {:?} should be {:?}",
                    &watcher.state,
                    watcher.job.name,
                    JobState::Pending
                )
            }

            watcher.state = JobState::Terminated(result);
        } else {
            panic!("recorded job not in all_jobs");
        }

        self.available_jobs.remove_job(job);

        if job_went_wrong {
            self.cancel_next_jobs(job);
            return;
        }

        self.unlock_next_jobs(job);
        self.actualize_job_list();
    }

    fn actualize_job_list(&mut self) {
        self.available_jobs = JobList::from(
            self.all_jobs
                .values()
                .filter(|job_watcher| matches!(job_watcher.state, JobState::Pending))
                .map(|job_watcher| job_watcher.job.name.to_string())
                .collect(),
        );
    }

    fn unlock_next_jobs(&mut self, name: &str) {
        let watcher = self.all_jobs.get(name).unwrap();

        let blocking_job_name = watcher.job.name.to_string();
        let blocked_job_list = watcher.blocks_job.clone();

        for blocked_job_name in blocked_job_list {
            let blocked_job = self.all_jobs.get_mut(blocked_job_name.as_str()).unwrap();
            debug_assert!(
                matches!(blocked_job.state, JobState::Blocked),
                "Only blocked jobs should be in blocked list"
            );
            blocked_job.blocked_by_jobs.remove_job(&blocking_job_name);
            if blocked_job.blocked_by_jobs.is_empty() {
                blocked_job.state = JobState::Pending;
            }
        }
    }

    fn cancel_next_jobs(&mut self, name: &str) {
        let watcher = self.all_jobs.get(name).unwrap();
        let blocking_job_name = watcher.job.name.to_string();
        let blocked_job_list = watcher.blocks_job.clone();
        for blocked_job_name in blocked_job_list {
            let blocked_job = self.all_jobs.get_mut(blocked_job_name.as_str()).unwrap();
            debug_assert!(
                matches!(blocked_job.state, JobState::Blocked),
                "Only blocked jobs should be in blocked list"
            );
            blocked_job.blocked_by_jobs.remove_job(&blocking_job_name);
            if blocked_job.blocked_by_jobs.is_empty() {
                blocked_job.state = JobState::Cancelled(blocking_job_name.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ci::job::dag::{Dag, DagError, JobList, JobResult};
    use crate::ci::job::tests::{complex_job_schedule, cons, job, simple_job_schedule};
    use std::fmt::{Display, Formatter};

    impl Display for JobList {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[")?;
            let mut left = self.vec.len();
            for item in self.vec.iter().rev() {
                write!(f, "{}", item)?;
                left -= 1;
                if left > 0 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")
        }
    }

    #[test]
    pub fn record_good() {
        let list = simple_job_schedule();
        let mut dag = Dag::new(&list.0, &list.1).unwrap();
        let build = dag.poll().expect("this is not None");

        assert_eq!("build", &build.name);

        dag.record_event(&build.name, JobResult::Success);

        let test = dag.poll();

        assert_eq!("test", &test.expect("This is supposed to be test job").name);
    }

    #[test]
    pub fn record_bad() {
        let list = simple_job_schedule();
        let mut dag = Dag::new(&list.0, &list.1).unwrap();

        let job = dag.poll().expect("this is not None");

        dag.record_event(&job.name, JobResult::Failure);

        let none = dag.poll();

        assert!(none.is_none())
    }

    #[test]
    pub fn test_complex() {
        let list = complex_job_schedule();
        let mut dag = Dag::new(&list.0, &list.1).unwrap();

        let build1 = dag.poll().unwrap();
        let build2 = dag.poll().unwrap();

        assert_eq!("build1", &build1.name);
        assert_eq!("build2", &build2.name);
        assert!(dag.poll().is_none());

        dag.record_event(&build1.name, JobResult::Success);
        dag.record_event(&build2.name, JobResult::Success);

        let test1 = dag.poll().unwrap();
        let test2 = dag.poll().unwrap();

        assert_eq!("test1", &test1.name);
        assert_eq!("test2", &test2.name);
        assert!(dag.poll().is_none());

        dag.record_event(&test1.name, JobResult::Success);
        dag.record_event(&test2.name, JobResult::Success);

        let deploy = dag.poll().unwrap();
        assert_eq!("deploy", &deploy.name);
        assert!(dag.poll().is_none());
    }

    #[test]
    pub fn test_cycle() {
        let jobs = vec![job("A"), job("B"), job("C")];
        let cons = vec![cons("A", "B"), cons("B", "C"), cons("C", "A")];
        let error = Dag::new(&jobs, &cons);

        assert!(error.is_err());

        let error = error.err().unwrap();

        match error {
            DagError::CycleExistsBecauseOf(letter) => {
                assert_eq!(&letter, "A");
            }
            err => {
                panic!(
                    "{:?} should be a {:?}",
                    err,
                    DagError::CycleExistsBecauseOf(String::from("A"))
                )
            }
        }
    }
}
