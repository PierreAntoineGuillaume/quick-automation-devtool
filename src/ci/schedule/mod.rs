use super::job::{Job, JobProgress, JobScheduler, Progress};
use super::CommandJobRunner;
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub trait JobStarter {
    fn start_all_jobs(&self, jobs: &[Job], first_tx: Sender<JobProgress>);
}

pub trait CiDisplay {
    fn record(&mut self, job_progress: JobProgress);
    fn is_finished(&self) -> bool;
    fn refresh(&mut self);
}

#[derive(Clone, Copy)]
pub struct ParrallelJobStarter {}

impl JobStarter for ParrallelJobStarter {
    fn start_all_jobs(&self, jobs: &[Job], first_tx: Sender<JobProgress>) {
        for real_job in jobs {
            let job = real_job.clone();
            let tx = first_tx.clone();
            thread::spawn(move || {
                tx.send(JobProgress::new(job.name.clone(), Progress::Started))
                    .unwrap();
                let terminated = Progress::Terminated(job.start(&CommandJobRunner::new()));
                tx.send(JobProgress::new(job.name, terminated)).unwrap();
            });
        }
    }
}

pub struct CompositeJobScheduler<'a> {
    job_starter: &'a dyn JobStarter,
    job_display: &'a mut dyn CiDisplay,
}

impl JobScheduler for CompositeJobScheduler<'_> {
    fn schedule(&mut self, jobs: &[Job]) -> Result<(), ()> {
        let (first_tx, rx) = channel();

        Self::signal_all_existing_jobs(jobs, &first_tx);
        self.job_starter.start_all_jobs(jobs, first_tx);

        let mut is_error = false;
        loop {
            if let Ok(state) = rx.try_recv() {
                is_error |= state.failed();
                self.job_display.record(state);
            }
            self.job_display.refresh();
            if self.job_display.is_finished() {
                break;
            }
        }

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
