use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::{CiDisplay, JobScheduler, JobStarter};
use crate::ci::job::state::Progress;
use crate::ci::job::{Job, JobProgress};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

pub struct CompositeJobScheduler<'a, Starter: JobStarter, Displayer: CiDisplay> {
    job_starter: &'a mut Starter,
    job_display: &'a mut Displayer,
}

impl<Starter: JobStarter, Displayer: CiDisplay> JobScheduler
    for CompositeJobScheduler<'_, Starter, Displayer>
{
    fn schedule(&mut self, jobs: &[Job]) -> JobProgressTracker {
        let mut tracker = JobProgressTracker::new();
        if jobs.is_empty() {
            tracker.try_finish();
            self.job_display.finish(&tracker);
            return tracker;
        }

        let (tx, rx) = channel();

        self.signal_all_existing_jobs(jobs, &tx);

        while let Some(progress) = self.read(&rx) {
            tracker.record(progress);
        }

        let mut delay: usize = 0;

        loop {
            let available_jobs: Vec<Job> = jobs
                .iter()
                .filter(|job| tracker.job_is_available(&job.name))
                .cloned()
                .collect();

            if !available_jobs.is_empty() {
                self.job_starter
                    .consume_some_jobs(&available_jobs, tx.clone());
            }

            while let Some(progress) = self.read(&rx) {
                tracker.record(progress);
            }

            if tracker.try_finish() {
                break;
            }
            self.job_display.refresh(&tracker, delay);
            delay = self.job_starter.delay();
        }

        self.job_starter.join();

        self.job_display.finish(&tracker);

        tracker
    }
}

impl<Starter: JobStarter, Displayer: CiDisplay> CompositeJobScheduler<'_, Starter, Displayer> {
    pub fn new<'a, Ta: JobStarter, Tb: CiDisplay>(
        job_starter: &'a mut Ta,
        job_display: &'a mut Tb,
    ) -> CompositeJobScheduler<'a, Ta, Tb> {
        CompositeJobScheduler {
            job_starter,
            job_display,
        }
    }

    fn signal_all_existing_jobs(&self, jobs: &[Job], first_tx: &Sender<JobProgress>) {
        for job in jobs {
            first_tx
                .send(JobProgress::new(&job.name, Progress::Available))
                .unwrap();
        }
    }

    pub fn read(&self, rx: &Receiver<JobProgress>) -> Option<JobProgress> {
        match rx.try_recv() {
            Ok(state) => Some(state),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                panic!("State receiver has been disconnected, try restarting the program");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::job::schedule::test::{NullCiDisplay, TestJobStarter};

    impl Job {
        pub fn new(name: &str, instructions: &[&str]) -> Self {
            Job {
                name: name.to_string(),
                instructions: instructions
                    .iter()
                    .map(|item| String::from(*item))
                    .collect(),
            }
        }
    }

    fn test_that(callback: fn(&mut dyn JobScheduler)) {
        let mut job_start = TestJobStarter {};
        let mut job_display = NullCiDisplay {};
        let mut scheduler = CompositeJobScheduler::<TestJobStarter, NullCiDisplay>::new(
            &mut job_start,
            &mut job_display,
        );
        callback(&mut scheduler)
    }

    #[test]
    pub fn every_job_is_initialisated() {
        test_that(|scheduler| {
            let result = scheduler.schedule(&[Job::new("a", &["ok: result"])]);
            assert!(!result.has_failed);
        })
    }

    #[test]
    pub fn one_job_failure_fails_scheduling() {
        test_that(|scheduler| {
            let result = scheduler.schedule(&[Job::new("c", &["ko: result"])]);
            assert!(result.has_failed);
        })
    }

    #[test]
    pub fn one_job_crash_fails_scheduling() {
        test_that(|scheduler| {
            let result = scheduler.schedule(&[Job::new("c", &["crash: result"])]);
            assert!(result.has_failed);
        })
    }
}
