use super::job::{Job, JobProgress, JobProgressTracker, JobScheduler, Progress};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};

pub trait JobStarter {
    fn start_all_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>);
    fn join(&mut self);
    fn delay(&mut self) -> usize;
}

pub trait CiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker, elapsed: usize);
    fn finish(&mut self, tracker: &JobProgressTracker);
}

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

        Self::signal_all_existing_jobs(jobs, &tx);

        while let Some(progress) = self.read(&rx) {
            tracker.record(progress);
        }

        self.job_starter.start_all_jobs(jobs, tx.clone());
        let mut delay: usize = 0;

        loop {
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

        drop(tx);

        self.job_display.finish(&tracker);

        tracker
    }
}

impl<Starter: JobStarter, Displayer: CiDisplay> CompositeJobScheduler<'_, Starter, Displayer> {
    fn signal_all_existing_jobs(jobs: &[Job], first_tx: &Sender<JobProgress>) {
        for job in jobs {
            first_tx
                .send(JobProgress::new(&job.name, Progress::Awaiting))
                .unwrap();
        }
    }

    pub fn new<'a, Ta: JobStarter, Tb: CiDisplay>(
        job_starter: &'a mut Ta,
        job_display: &'a mut Tb,
    ) -> CompositeJobScheduler<'a, Ta, Tb> {
        CompositeJobScheduler {
            job_starter,
            job_display,
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
    use super::super::job::test::TestJobRunner;
    use super::*;

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

    struct TestJobStarter {}

    impl JobStarter for TestJobStarter {
        fn start_all_jobs(&mut self, jobs: &[Job], tx: Sender<JobProgress>) {
            for job in jobs {
                job.start(&TestJobRunner {}, &tx.clone());
            }
        }

        fn join(&mut self) {}
        fn delay(&mut self) -> usize {
            0
        }
    }

    struct NullCiDisplay {}

    impl CiDisplay for NullCiDisplay {
        fn refresh(&mut self, _: &JobProgressTracker, _: usize) {}
        fn finish(&mut self, _: &JobProgressTracker) {}
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
