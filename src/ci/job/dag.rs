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
    CycleExistsBetween(String, String),
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
    pub fn from_vec(vec: Vec<String>) -> Self {
        JobList { vec }
    }

    pub fn new() -> Self {
        Self::from_vec(vec![])
    }

    pub fn remove_job(&mut self, name: &str) {
        self.vec.retain(|contained| contained != name)
    }

    pub fn add_job(&mut self, name: &str) {
        self.vec.push(name.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

#[derive(Debug)]
pub struct JobWatcher {
    job: Job,
    state: JobState,
    blocks_job: Vec<String>,
    blocking_jobs: JobList,
}

pub struct Dag {
    all_jobs: BTreeMap<String, JobWatcher>,
    available_jobs: JobList,
}

impl Dag {
    pub fn new(jobs: &[Job], constraints: &[(String, String)]) -> Result<Self, DagError> {
        let jobs: Vec<Job> = jobs.to_vec();
        let _matrix = ConstraintMatrix::new(&jobs, constraints)?;

        let all_jobs = BTreeMap::<String, JobWatcher>::new();
        let available_jobs = JobList::new();

        Ok(Dag {
            all_jobs,
            available_jobs,
        })
    }

    pub fn record_event(&mut self, job: &str, result: JobResult) {
        let watcher = self.all_jobs.get_mut(job).unwrap();

        if !matches!(watcher.state, JobState::Started) {
            panic!("bad state for terminated job {:?}", watcher)
        }

        let job_went_wrong = matches!(result, JobResult::Failure);

        watcher.state = JobState::Terminated(result);

        self.available_jobs.remove_job(job);

        if job_went_wrong {
            self.cancel_next_jobs(job);
            return;
        }

        self.unlock_next_jobs(job);
        self.actualize_job_list();
    }

    fn actualize_job_list(&mut self) {
        self.available_jobs = JobList::from_vec(
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
            if !matches!(blocked_job.state, JobState::Blocked) {
                continue;
            }
            blocked_job.blocking_jobs.remove_job(&blocking_job_name);
            if blocked_job.blocking_jobs.is_empty() {
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
            if !matches!(blocked_job.state, JobState::Blocked) {
                continue;
            }
            blocked_job.blocking_jobs.remove_job(&blocking_job_name);
            if blocked_job.blocking_jobs.is_empty() {
                blocked_job.state = JobState::Cancelled(blocking_job_name.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ci::job::tests::tests::complex_job_schedule;

    #[test]
    pub fn record() {
        let list = complex_job_schedule();

    }
}
