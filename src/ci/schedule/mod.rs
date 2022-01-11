use super::job::{Job, JobProgress, JobProgressTracker, JobScheduler, Progress};
use std::sync::mpsc::{channel, Sender};

pub trait JobStarter {
    fn start_all_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>);
    fn join(&mut self);
}

pub trait CiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker);
}

pub struct CompositeJobScheduler<'a> {
    job_starter: &'a mut dyn JobStarter,
    job_display: &'a mut dyn CiDisplay,
}

impl JobScheduler for CompositeJobScheduler<'_> {
    fn schedule(&mut self, jobs: &[Job]) -> Result<JobProgressTracker, JobProgressTracker> {
        let (tx, rx) = channel();

        Self::signal_all_existing_jobs(jobs, &tx);
        self.job_starter.start_all_jobs(jobs, tx);
        let mut tracker = JobProgressTracker::new();

        let mut is_error = false;
        loop {
            if let Ok(state) = rx.try_recv() {
                is_error |= state.failed();
                tracker.record(state);
            }

            if tracker.is_finished() {
                break;
            }

            self.job_display.refresh(&tracker);
        }

        self.job_starter.join();

        self.job_display.refresh(&tracker);

        if is_error {
            return Err(tracker);
        }

        Ok(tracker)
    }
}

impl CompositeJobScheduler<'_> {
    fn signal_all_existing_jobs(jobs: &[Job], first_tx: &Sender<JobProgress>) {
        for job in jobs {
            first_tx
                .send(JobProgress::new(job.name.clone(), Progress::Awaiting))
                .unwrap();
        }
    }
    pub fn new<'a>(
        job_starter: &'a mut dyn JobStarter,
        job_display: &'a mut dyn CiDisplay,
    ) -> CompositeJobScheduler<'a> {
        CompositeJobScheduler {
            job_starter,
            job_display,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::job::{JobOutput, JobRunner};

    struct TestJobRunner {}

    impl JobRunner for TestJobRunner {
        fn run(&self, job: &str) -> JobOutput {
            if let Some(stripped) = job.strip_prefix("ok:") {
                JobOutput::Success(stripped.into())
            } else if let Some(stripped) = job.strip_prefix("ko:") {
                JobOutput::JobError(stripped.into())
            } else if let Some(stripped) = job.strip_prefix("crash:") {
                JobOutput::ProcessError(stripped.into())
            } else {
                panic!("Job should begin with ok:, ko, or crash:")
            }
        }
    }

    struct TestJobStarter {}

    impl JobStarter for TestJobStarter {
        fn start_all_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>) {
            for job in jobs {
                job.start(&TestJobRunner {}, &tx.clone());
            }
        }

        fn join(&mut self) {}
    }

    struct NullCiDisplay {}

    impl CiDisplay for NullCiDisplay {
        fn refresh(&mut self, _: &JobProgressTracker) {}
    }

    fn test_that(callback: fn(&mut dyn JobScheduler)) {
        let mut job_start = TestJobStarter {};
        let mut job_display = NullCiDisplay {};
        let mut scheduler = CompositeJobScheduler::new(&mut job_start, &mut job_display);
        callback(&mut scheduler)
    }

    #[test]
    pub fn every_job_is_initialisated() {
        test_that(|scheduler| {
            let result = scheduler.schedule(&[Job::new("a".into(), vec!["ok: result".into()])]);
            assert!(result.is_ok());
        })
    }

    #[test]
    pub fn one_job_failure_fails_scheduling() {
        test_that(|scheduler| {
            let result = scheduler.schedule(&[Job::new("c".into(), vec!["ko: result".into()])]);
            assert!(result.is_err());
        })
    }

    #[test]
    pub fn one_job_crash_fails_scheduling() {
        test_that(|scheduler| {
            let result = scheduler.schedule(&[Job::new("c".into(), vec!["crash: result".into()])]);
            assert!(result.is_err());
        })
    }
}
