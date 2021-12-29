pub(crate) mod logic;

use logic::display::PipelineProgress;
use logic::job::*;
use std::process::Command;
use std::sync::mpsc::channel;
use std::thread;

#[derive(Clone)]
pub struct NThreadedJobScheduler {}

impl JobScheduler for NThreadedJobScheduler {
    fn schedule(&mut self, jobs: &[Job]) {
        let (first_tx, rx) = channel();

        for job in jobs {
            first_tx
                .send(JobProgress::new(job.name.clone(), Progress::Awaiting))
                .unwrap();
        }

        for real_job in jobs {
            let job = real_job.clone();
            let tx = first_tx.clone();
            thread::spawn(move || {
                tx.send(JobProgress::new(job.name.clone(), Progress::Started))
                    .unwrap();
                let terminated = Progress::Terminated(job.start(Box::new(CommandJobRunner::new())));
                tx.send(JobProgress::new(job.name, terminated)).unwrap();
            });
        }
        let mut pipeline_progress = PipelineProgress::new();
        loop {
            if let Ok(state) = rx.try_recv() {
                pipeline_progress.push(state);
            }
            if pipeline_progress.is_finished() {
                println!("{}", pipeline_progress);
                break;
            }
        }
    }
}

pub struct CommandJobRunner {}

impl JobRunner for CommandJobRunner {
    fn run(&self, job: String) -> JobOutput {
        match Command::new(&job).output() {
            Ok(output) => {
                return match (output.status.success(), std::str::from_utf8(&output.stdout)) {
                    (true, Ok(output)) => JobOutput::Success(output.to_string()),
                    (false, Ok(e)) => JobOutput::JobError(e.to_string()),
                    (_, Err(e)) => JobOutput::ProcessError(e.to_string()),
                };
            }
            Err(e) => JobOutput::ProcessError(format!("{}: {}", job, e)),
        }
    }
}

impl CommandJobRunner {
    pub fn new() -> CommandJobRunner {
        CommandJobRunner {}
    }
}
