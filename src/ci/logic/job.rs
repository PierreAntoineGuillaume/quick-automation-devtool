#[derive(Debug, PartialEq, Clone)]
pub enum JobOutput {
    Success(String),
    JobError(String),
    ProcessError(String),
}

impl JobOutput {
    pub fn failed(&self) -> bool {
        !matches!(self, JobOutput::Success(_))
    }
}

pub trait JobRunner {
    fn run(&self, job: &str) -> JobOutput;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Job {
    pub name: String,
    instruction: String,
}

impl Job {
    pub fn start(&self, runner: Box<dyn JobRunner>) -> JobOutput {
        runner.run(&self.instruction)
    }
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Awaiting,
    Started,
    Terminated(JobOutput),
}

impl Progress {
    pub fn failed(&self) -> bool {
        match self {
            Progress::Terminated(job_output) => job_output.failed(),
            _ => false,
        }
    }
}

impl Progress {
    pub fn is_pending(&self) -> bool {
        matches!(*self, Progress::Awaiting | Progress::Started)
    }
}

pub struct Pipeline {
    jobs: Vec<Job>,
}

pub trait JobScheduler {
    fn schedule(&mut self, jobs: &[Job]) -> Result<(), ()>;
}

impl Pipeline {
    pub fn run(&mut self, scheduler: &mut dyn JobScheduler) -> Result<(), ()> {
        scheduler.schedule(&self.jobs)
    }

    pub fn push(&mut self, key: String, instruction: String) {
        self.jobs.push(Job {
            name: key,
            instruction,
        });
    }

    pub fn new() -> Pipeline {
        Pipeline { jobs: Vec::new() }
    }
}

pub struct JobProgress {
    pub job_name: String,
    pub progress: Progress,
}

impl JobProgress {
    pub fn new(job: String, progress: Progress) -> Self {
        JobProgress {
            job_name: job,
            progress,
        }
    }

    pub fn failed(&self) -> bool {
        self.progress.failed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct NaiveScheduler {
        pub states: HashMap<Job, Progress>,
    }

    impl JobScheduler for NaiveScheduler {
        fn schedule(&mut self, jobs: &[Job]) -> Result<(), ()> {
            let mut err = false;
            for job in jobs {
                self.states.insert(job.clone(), Progress::Awaiting);
                self.states.insert(job.clone(), Progress::Started);
                let res = job.start(FakeJobRunner::new(JobOutput::Success("Ok".into())));
                err |= res.failed();
                self.states
                    .insert(job.clone(), Progress::Terminated(res))
                    .unwrap();
            }
            if err {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    impl NaiveScheduler {
        fn new() -> NaiveScheduler {
            NaiveScheduler {
                states: HashMap::new(),
            }
        }
    }

    struct FakeJobRunner {
        out: JobOutput,
    }

    impl FakeJobRunner {
        fn new(out: JobOutput) -> Box<FakeJobRunner> {
            Box::new(FakeJobRunner { out })
        }
    }

    impl From<Box<NaiveScheduler>> for Box<dyn JobScheduler> {
        fn from(scheduler: Box<NaiveScheduler>) -> Self {
            Box::new(*scheduler)
        }
    }

    impl JobRunner for FakeJobRunner {
        fn run(&self, job: &str) -> JobOutput {
            match &self.out {
                JobOutput::Success(item) => JobOutput::Success(format!("{}:{}", job, item.clone())),
                JobOutput::JobError(item) => {
                    JobOutput::JobError(format!("{}:{}", job, item.clone()))
                }
                JobOutput::ProcessError(item) => {
                    JobOutput::ProcessError(format!("{}:{}", job, item.clone()))
                }
            }
        }
    }

    #[test]
    pub fn all_jobs_are_ran() {
        let mut pipeline = Pipeline::new();
        pipeline.push("first".into(), "first".into());
        pipeline.push("second".into(), "second".into());
        pipeline.push("third".into(), "third".into());
        pipeline.push("fourth".into(), "fourth".into());

        let mut scheduler = NaiveScheduler::new();
        pipeline.run(&mut scheduler).unwrap();

        let mut job_count = 0;
        for (job, status) in scheduler.states {
            assert_eq!(
                status,
                Progress::Terminated(JobOutput::Success(format!("{}:Ok", job.name)))
            );
            job_count += 1;
        }
        assert_eq!(job_count, 4)
    }
}
