use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum JobOutput {
    Success(String),
    JobError(String),
    ProcessError(String),
}

pub trait JobRunner {
    fn run(&self, job: String) -> JobOutput;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Job {
    pub name: String,
    instruction: String,
}

impl Job {
    pub fn start(&self, runner: Box<dyn JobRunner>) -> JobOutput {
        runner.run(self.instruction.clone())
    }
}

#[derive(Debug, PartialEq)]
pub enum Progress {
    Awaiting,
    Started,
    Terminated(JobOutput),
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
    fn schedule(&mut self, jobs: &[Job]);
}

impl Pipeline {
    pub fn run(&mut self, scheduler: &mut dyn JobScheduler) {
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
    job_name: String,
    progress: Progress,
}

impl JobProgress {
    pub fn new(job: String, progress: Progress) -> Self {
        JobProgress {
            job_name: job,
            progress,
        }
    }
}

#[derive(Debug)]
pub struct PipelineProgress {
    states: HashMap<String, Progress>,
}

impl PipelineProgress {
    pub fn new() -> Self {
        PipelineProgress {
            states: HashMap::new(),
        }
    }
    pub fn push(&mut self, job_progress: JobProgress) {
        self.states
            .insert(job_progress.job_name, job_progress.progress);
    }

    pub fn is_finished(&self) -> bool {
        for progress in self.states.values() {
            if progress.is_pending() {
                return false;
            }
        }
        true
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
        fn schedule(&mut self, jobs: &[Job]) {
            for job in jobs {
                self.states.insert(job.clone(), Progress::Awaiting);
                self.states.insert(job.clone(), Progress::Started);
                self.states
                    .insert(
                        job.clone(),
                        Progress::Terminated(
                            job.start(FakeJobRunner::new(JobOutput::Success("Ok".into()))),
                        ),
                    )
                    .unwrap();
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
        fn run(&self, job: String) -> JobOutput {
            match self.out {
                JobOutput::Success(ref item) => {
                    JobOutput::Success(format!("{}:{}", job, item.clone()))
                }
                JobOutput::JobError(ref item) => {
                    JobOutput::JobError(format!("{}:{}", job, item.clone()))
                }
                JobOutput::ProcessError(ref item) => {
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
        pipeline.run(&mut scheduler);

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
