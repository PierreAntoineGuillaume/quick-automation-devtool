use crate::ci::job::constraint_matrix::ConstraintMatrix;
use crate::ci::job::Job;
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Constraint {
    Free,
    Indifferent,
    Blocked(usize),
}

impl Constraint {
    pub const fn constrain(&self) -> Result<Self, ()> {
        match self {
            Self::Free => Err(()),
            Self::Indifferent => Ok(Self::Blocked(1)),
            Self::Blocked(usize) => Ok(Self::Blocked(*usize + 1usize)),
        }
    }
}

impl Default for Constraint {
    fn default() -> Self {
        Self::Indifferent
    }
}

#[derive(Debug)]
pub enum Error {
    JobCannotBlockItself(String),
    UnknownJobInConstraint(String),
    CycleExistsBecauseOf(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::JobCannotBlockItself(jobname) => {
                write!(f, "job {jobname} is blocking itself")
            }
            Error::UnknownJobInConstraint(jobname) => {
                write!(f, "job {jobname} in constraint list doesn't exist")
            }
            Error::CycleExistsBecauseOf(blocking_job) => {
                write!(f, "a cycle exists in the job DAG because of {blocking_job}")
            }
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialOrd, Ord, Eq, PartialEq)]
pub enum JobResult {
    Success,
    Failure,
}

#[derive(Clone, PartialOrd, Ord, Eq, PartialEq)]
pub enum JobState {
    Pending,
    Started,
    Blocked,
    Terminated(JobResult),
    Cancelled(Vec<String>),
}

impl JobState {
    pub const fn is_unresolved(&self) -> bool {
        matches!(self, Self::Pending | Self::Started | Self::Blocked)
    }
}

impl Debug for JobState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Pending => "pending",
                Self::Started => "started",
                Self::Blocked => "blocked",
                Self::Terminated(JobResult::Success) => "success",
                Self::Terminated(JobResult::Failure) => "failure",
                Self::Cancelled(_) => "cancelled",
            }
        )
    }
}

impl Default for JobState {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug)]
pub struct JobList {
    vec: Vec<String>,
}

impl JobList {
    pub fn from(vec: &[String]) -> Self {
        Self {
            vec: vec.iter().rev().cloned().collect(),
        }
    }

    pub const fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn remove_job(&mut self, name: &str) {
        self.vec.retain(|contained| contained != name);
    }

    pub const fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn shift(&mut self) -> Option<String> {
        self.vec.pop()
    }
}

pub struct JobWatcher {
    job: Job,
    state: JobState,
    blocks_job: Vec<String>,
    blocked_by_jobs: JobList,
}

impl JobWatcher {
    pub const fn new(
        job: Job,
        state: JobState,
        blocks_job: Vec<String>,
        blocked_by_jobs: JobList,
    ) -> Self {
        Self {
            job,
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

pub struct JobEnumeration {
    pub name: String,
    pub state: JobState,
    pub block: Vec<String>,
}

const fn jobenum(name: String, state: JobState, block: Vec<String>) -> JobEnumeration {
    JobEnumeration { name, state, block }
}

impl Eq for JobEnumeration {}

impl PartialEq<Self> for JobEnumeration {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd<Self> for JobEnumeration {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JobEnumeration {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.name == other.name {
            return Ordering::Equal;
        }
        match self.block.cmp(&other.block) {
            Ordering::Equal => self.name.cmp(&other.name),
            ord => ord,
        }
    }
}

impl Dag {
    pub fn new(
        jobs: &[Job],
        constraints: &[(String, String)],
        groups: &[String],
        env: &HashMap<String, Vec<String>>,
    ) -> Result<Self, Error> {
        let jobs: Vec<Job> = jobs.to_vec();
        let mut constraints: Vec<(String, String)> = constraints.to_vec();

        constraints.extend(Self::compute_group_constraints(&jobs, groups));

        let matrix = ConstraintMatrix::new(&jobs, &constraints)?;

        let mut all_jobs = BTreeMap::<String, JobWatcher>::new();

        for mut job in jobs {
            job.forward_env(env);
            let blocking = matrix.blocked_by(job.name());
            let blocked_by_jobs: Vec<String> = matrix.blocking(job.name()).collect();
            let state = if blocked_by_jobs.is_empty() {
                JobState::Pending
            } else {
                for blocked_by_job in &blocked_by_jobs {
                    if blocked_by_job == job.name() {
                        return Err(Error::CycleExistsBecauseOf(blocked_by_job.clone()));
                    }
                }
                JobState::Blocked
            };
            all_jobs.insert(
                job.name().to_string(),
                JobWatcher::new(
                    job,
                    state,
                    blocking.collect(),
                    JobList::from(&blocked_by_jobs),
                ),
            );
        }

        let mut dag = Self {
            all_jobs,
            available_jobs: JobList::new(),
        };

        dag.actualize_job_list();

        Ok(dag)
    }

    fn compute_group_constraints(jobs: &[Job], groups: &[String]) -> Vec<(String, String)> {
        let mut group_constraints = vec![];
        let mut blocking_jobs_by_groups = IndexMap::<String, Vec<String>>::new();

        for group in groups {
            blocking_jobs_by_groups.insert(group.to_string(), vec![]);
        }

        for job in jobs {
            if let Some(group) = job.group() {
                if let Some(collection) = blocking_jobs_by_groups.get_mut(group) {
                    collection.push(job.name().to_string());
                }
            }
        }

        for job in jobs {
            if let Some(job_group) = job.group() {
                for (group, blocking_job) in &blocking_jobs_by_groups {
                    if group == job_group {
                        break;
                    }
                    for blocking_job in blocking_job {
                        group_constraints.push((blocking_job.to_string(), job.name().to_string()));
                    }
                }
            }
        }
        group_constraints
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
    /// `JobResult::Failure` or `JobResult::Success`
    pub fn record_event(&mut self, job: &str, result: JobResult) {
        let job_went_wrong = matches!(result, JobResult::Failure);

        if let Some(watcher) = self.all_jobs.get_mut(job) {
            if !matches!(watcher.state, JobState::Started) {
                unreachable!(
                    "bad state {:?} for job {:?} should be {:?}",
                    &watcher.state,
                    watcher.job.name(),
                    JobState::Pending
                )
            }

            watcher.state = JobState::Terminated(result);
        } else {
            unreachable!("recorded job not in all_jobs");
        }

        if job_went_wrong {
            self.cancel_next_jobs(job);
            return;
        }

        self.unlock_next_jobs(job);
        self.actualize_job_list();
    }

    /// A query method to know if all possible jobs have been ran
    pub fn is_finished(&self) -> bool {
        for job in self.all_jobs.values() {
            if job.state.is_unresolved() {
                return false;
            }
        }
        true
    }

    /// Query all job states by job name
    pub fn enumerate(&self) -> Vec<JobEnumeration> {
        let mut vec: Vec<JobEnumeration> = self
            .all_jobs
            .values()
            .map(|watcher| {
                jobenum(
                    watcher.job.name().to_string(),
                    watcher.state.clone(),
                    watcher.blocked_by_jobs.vec.clone(),
                )
            })
            .collect();
        vec.sort();
        vec
    }

    fn actualize_job_list(&mut self) {
        self.available_jobs = JobList::from(
            &self
                .all_jobs
                .values()
                .filter(|job_watcher| matches!(job_watcher.state, JobState::Pending))
                .map(|job_watcher| job_watcher.job.name().to_string())
                .collect::<Vec<String>>(),
        );
    }

    fn unlock_next_jobs(&mut self, name: &str) {
        let watcher = self.all_jobs.get(name).unwrap();

        let blocking_job_name = watcher.job.name().to_string();
        let blocked_job_list = watcher.blocks_job.clone();

        for blocked_job_name in blocked_job_list {
            let blocked_job = self.all_jobs.get_mut(blocked_job_name.as_str()).unwrap();
            if matches!(blocked_job.state, JobState::Cancelled(_)) {
                return;
            }
            debug_assert!(
                matches!(blocked_job.state, JobState::Blocked),
                "Only blocked or Cancelled jobs should be in blocked list"
            );
            blocked_job.blocked_by_jobs.remove_job(&blocking_job_name);
            if blocked_job.blocked_by_jobs.is_empty() {
                blocked_job.state = JobState::Pending;
            }
        }
    }

    fn cancel_next_jobs(&mut self, name: &str) {
        let watcher = self.all_jobs.get(name).unwrap();
        let blocking_job_name = watcher.job.name().to_string();
        let blocked_job_list = watcher.blocks_job.clone();
        for blocked_job_name in blocked_job_list {
            let blocked_job = self.all_jobs.get_mut(blocked_job_name.as_str()).unwrap();
            debug_assert!(
                matches!(
                    blocked_job.state,
                    JobState::Blocked | JobState::Cancelled(_)
                ),
                "Only blocked or Cancelled jobs should be in blocked list"
            );
            let list = match &blocked_job.state {
                JobState::Blocked => {
                    vec![blocking_job_name.clone()]
                }
                JobState::Cancelled(old) => {
                    let mut vec = old.clone();
                    vec.push(blocking_job_name.clone());
                    vec
                }
                _ => {
                    unreachable!("This statement cannot happen")
                }
            };
            blocked_job.state = JobState::Cancelled(list);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ci::job::dag::{Dag, Error, JobEnumeration, JobList, JobResult};
    use crate::ci::job::tests::{
        complex_job_schedule, cons, group_job_schedule, job, simple_job_schedule,
    };
    use std::collections::HashMap;
    use std::fmt::{Debug, Display, Formatter};

    impl Display for JobList {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[")?;
            let mut left = self.vec.len();
            for item in self.vec.iter().rev() {
                write!(f, "{item}")?;
                left -= 1;
                if left > 0 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")
        }
    }

    impl Debug for JobEnumeration {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}({:?})", self.name, self.state)
        }
    }

    #[test]
    pub fn record_good() {
        let (jobs, constraints, groups) = simple_job_schedule();
        let mut dag = Dag::new(&jobs, &constraints, &groups, &HashMap::new()).unwrap();
        let build = dag.poll().expect("this is not None");

        assert_eq!("build", build.name());

        dag.record_event(build.name(), JobResult::Success);

        let test = dag.poll();

        assert_eq!(
            "test",
            test.expect("This is supposed to be test job").name()
        );
    }

    #[test]
    pub fn record_bad() {
        let (jobs, constraints, groups) = simple_job_schedule();
        let mut dag = Dag::new(&jobs, &constraints, &groups, &HashMap::new()).unwrap();

        let job = dag.poll().expect("this is not None");

        dag.record_event(job.name(), JobResult::Failure);

        let none = dag.poll();

        assert!(none.is_none());
    }

    #[test]
    pub fn test_complex() {
        let (jobs, constraints, groups) = complex_job_schedule();
        let mut dag = Dag::new(&jobs, &constraints, &groups, &HashMap::new()).unwrap();
        full_dag_test(&mut dag);
    }

    #[test]
    pub fn test_group() {
        let (jobs, constraints, groups) = group_job_schedule();
        let mut dag = Dag::new(&jobs, &constraints, &groups, &HashMap::new()).unwrap();
        full_dag_test(&mut dag);
    }

    fn full_dag_test(dag: &mut Dag) {
        assert!(!dag.is_finished());

        let build1 = dag.poll().unwrap();
        let build2 = dag.poll().unwrap();

        assert_eq!("build1", build1.name());
        assert_eq!("build2", build2.name());
        assert!(dag.poll().is_none());

        dag.record_event(build1.name(), JobResult::Success);
        dag.record_event(build2.name(), JobResult::Success);

        let test1 = dag.poll().unwrap();
        let test2 = dag.poll().unwrap();

        assert_eq!("test1", test1.name());
        assert_eq!("test2", test2.name());
        assert!(dag.poll().is_none());

        dag.record_event(test1.name(), JobResult::Success);
        dag.record_event(test2.name(), JobResult::Success);

        let deploy = dag.poll().unwrap();
        assert_eq!("deploy", deploy.name());
        dag.record_event(deploy.name(), JobResult::Success);
        assert!(dag.is_finished());
    }

    #[test]
    pub fn test_enumerate_base() {
        let (jobs, constraints, groups) = complex_job_schedule();
        let dag = Dag::new(&jobs, &constraints, &groups, &HashMap::new()).unwrap();
        let actual = dag.enumerate();
        assert_eq!(
            String::from("[build1(pending), build2(pending), test1(blocked), test2(blocked), deploy(blocked)]"),
            format!("{actual:?}")
        );
    }

    #[test]
    pub fn test_enumerate_failure() {
        let (jobs, constraints, groups) = complex_job_schedule();
        let mut dag = Dag::new(&jobs, &constraints, &groups, &HashMap::new()).unwrap();

        dag.poll();
        dag.record_event("build1", JobResult::Failure);

        let actual = dag.enumerate();

        assert_eq!(
            String::from("[build1(failure), build2(pending), test1(cancelled), test2(cancelled), deploy(cancelled)]"),
            format!("{actual:?}"));

        dag.poll();
        dag.record_event("build2", JobResult::Failure);

        let actual = dag.enumerate();

        assert_eq!(
            String::from("[build1(failure), build2(failure), test1(cancelled), test2(cancelled), deploy(cancelled)]"),
            format!("{actual:?}"));
    }

    #[test]
    pub fn test_cycle() {
        let jobs = vec![job("A"), job("B"), job("C")];
        let cons = vec![cons("A", "B"), cons("B", "C"), cons("C", "A")];
        let error = Dag::new(&jobs, &cons, &[], &HashMap::new()).err().unwrap();

        if let Error::CycleExistsBecauseOf(letter) = error {
            assert_eq!(&letter, "A");
        } else {
            panic!(
                "{error:?} should be a {:?}",
                Error::CycleExistsBecauseOf(String::from("A"))
            )
        }
    }
}
