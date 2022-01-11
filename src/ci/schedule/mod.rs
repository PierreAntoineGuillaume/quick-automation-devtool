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
    fn schedule(&mut self, jobs: &[Job]) -> Result<(), ()> {
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
            return Err(());
        }

        Ok(())
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
