use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::{JobOutput, Progress};
use std::time::SystemTime;

pub fn try_cleanup(input: String) -> String {
    let cleaned = input.trim_end();
    if cleaned.is_empty() {
        String::new()
    } else {
        format!("{cleaned}\n")
    }
}

pub struct FullFinalDisplay<'a> {
    config: &'a CiDisplayConfig,
}

impl<'a> FullFinalDisplay<'a> {
    pub fn new(config: &'a CiDisplayConfig) -> Self {
        Self { config }
    }

    pub fn finish(&mut self, tracker: &JobProgressTracker) {
        for (job_name, progress_collector) in &tracker.states {
            println!("Running tasks for job {job_name}");
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Partial(instruction, job_output) => match job_output {
                        JobOutput::Success(stdout, stderr)
                        | JobOutput::JobError(stdout, stderr) => {
                            let symbol = if job_output.succeeded() {
                                &self.config.ok
                            } else {
                                &self.config.ko
                            };
                            print!(
                                "{} {}",
                                symbol,
                                try_cleanup(format!(
                                    "{}\n{}{}",
                                    instruction,
                                    try_cleanup(stdout.clone()),
                                    try_cleanup(stderr.clone())
                                ))
                            );
                        }
                        JobOutput::ProcessError(stderr) => {
                            print!(
                                "{} {instruction}: {}",
                                self.config.ko,
                                try_cleanup(stderr.clone())
                            );
                        }
                    },
                    Progress::Terminated(bool) => {
                        let emoji: &str = if *bool {
                            &self.config.ok
                        } else {
                            &self.config.ko
                        };
                        println!("{} all tasks done for job {}", emoji, job_name);
                    }
                    _ => {}
                }
            }
        }
        let time = tracker
            .end_time
            .or_else(|| Some(SystemTime::now()))
            .unwrap();
        let elasped = time.duration_since(tracker.start_time).unwrap().as_millis() as f64;
        let status = if !tracker.has_failed {
            (&self.config.ok, "succeeded")
        } else {
            (&self.config.ko, "failed")
        };
        println!(
            "\n{} ci {} in {:.2} seconds",
            status.0,
            status.1,
            elasped / 1000f64
        );
    }
}
