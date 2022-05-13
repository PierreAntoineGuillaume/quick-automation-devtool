use crate::ci::display::ansi_control_sequence::{ResetChar, UnderlineChar};
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::ports::FinalCiDisplay;
use crate::ci::job::{JobOutput, Progress};
use regex::Regex;
use std::fmt::Write;
use std::time::SystemTime;

pub fn try_cleanup(input: &str) -> String {
    let cleaned = input.trim_end().replace(27 as char, r"\E");
    if cleaned.is_empty() {
        String::new()
    } else {
        let regex = Regex::new(r"\\E\[.[KG]").unwrap();
        format!(
            "{}\n",
            regex.replace_all(&cleaned, (27 as char).to_string())
        )
        .replace(r"\E", &(27 as char).to_string())
    }
}

pub struct FullFinalDisplay<'a> {
    config: &'a CiDisplayConfig,
}

impl<'a> FullFinalDisplay<'a> {
    pub fn new(config: &'a CiDisplayConfig) -> Self {
        Self { config }
    }
}

impl<'a> FinalCiDisplay for FullFinalDisplay<'a> {
    fn finish(&mut self, tracker: &JobProgressTracker) {
        for (job_name, progress_collector) in &tracker.states {
            let mut string = String::new();
            let mut icon = String::new();
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Cancelled => {
                        icon = self.config.cancelled.clone();
                    }
                    Progress::Skipped => string.push_str("  job was skipped\n"),
                    Progress::Partial(instruction, job_output) => match job_output {
                        JobOutput::Success(stdout, stderr)
                        | JobOutput::JobError(stdout, stderr) => {
                            let symbol = if job_output.succeeded() {
                                &self.config.ok
                            } else {
                                &self.config.ko
                            };
                            writeln!(string, "  {} {}", symbol, instruction).expect("write");
                            string.push_str(&try_cleanup(&format!(
                                "  {}\n  {}",
                                try_cleanup(stdout).replace('\n', "\n    "),
                                try_cleanup(&stderr.replace('\n', "\n    "))
                            )));
                        }
                        JobOutput::ProcessError(stderr) => write!(
                            string,
                            "  {} {instruction}: {}",
                            self.config.ko,
                            try_cleanup(stderr).replace('\n', "\n    ")
                        )
                        .expect("write"),
                    },
                    Progress::Terminated(bool) => {
                        let emoji: &str = if *bool {
                            icon = self.config.ok.clone();
                            &self.config.ok
                        } else {
                            icon = self.config.ko.clone();
                            &self.config.ko
                        };
                        writeln!(
                            string,
                            "  {} {}all tasks done for job {}{}",
                            emoji,
                            UnderlineChar(),
                            job_name,
                            ResetChar()
                        )
                        .expect("write");
                    }
                    _ => {}
                }
            }
            println!("{icon} tasks for job {job_name}:");
            if !string.is_empty() {
                print!("{}", string);
            }
        }

        let status = if tracker.has_failed {
            (&self.config.ko, "failed")
        } else {
            (&self.config.ok, "succeeded")
        };

        println!(
            "\n{} ci {} in {:.2} seconds",
            status.0,
            status.1,
            Self::elapsed(tracker).unwrap_or(0f64) / 1000f64
        );
    }
}

impl<'a> FullFinalDisplay<'a> {
    /// I'd rather have no info over a system error when reporting time
    fn elapsed(tracker: &JobProgressTracker) -> Option<f64> {
        let time = tracker.end_time.or_else(|| Some(SystemTime::now()))?;

        let since = time.duration_since(tracker.start_time).ok()?;

        Some(since.as_millis() as f64)
    }
}
