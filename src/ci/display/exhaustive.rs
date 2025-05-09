use crate::ci::clean::try_cleanup;
use crate::ci::display::ansi_control_sequence::{ResetChar, UnderlineChar};
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::ports::FinalCiDisplay;
use crate::ci::job::{Output, Progress};
use std::fmt::Write;
use std::time::SystemTime;

pub struct FullFinalDisplay<'a> {
    config: &'a CiDisplayConfig,
}

impl<'a> FullFinalDisplay<'a> {
    pub const fn new(config: &'a CiDisplayConfig) -> Self {
        Self { config }
    }
}

impl FinalCiDisplay for FullFinalDisplay<'_> {
    fn finish(&mut self, tracker: &JobProgressTracker) {
        for (job_name, progress_collector) in &tracker.states {
            let mut string = String::new();
            let mut icon = String::new();
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Cancelled => {
                        icon.clone_from(&self.config.cancelled);
                    }
                    Progress::Skipped => string.push_str("  job was skipped\n"),
                    Progress::Partial(instruction, job_output) => match job_output {
                        Output::Success(stdout, stderr) | Output::JobError(stdout, stderr) => {
                            let symbol = if job_output.succeeded() {
                                &self.config.ok
                            } else {
                                &self.config.ko
                            };
                            writeln!(string, "  {symbol} {instruction}").expect("write");
                            string.push_str(&try_cleanup(&format!(
                                "  {}\n  {}",
                                try_cleanup(stdout).replace('\n', "\n    "),
                                try_cleanup(&stderr.replace('\n', "\n    "))
                            )));
                        }
                        Output::ProcessError(stderr) => write!(
                            string,
                            "  {} {instruction}: {}",
                            self.config.ko,
                            try_cleanup(stderr).replace('\n', "\n    ")
                        )
                        .expect("write"),
                    },
                    Progress::Terminated(bool) => {
                        let emoji: &str = if *bool {
                            icon.clone_from(&self.config.ok);
                            &self.config.ok
                        } else {
                            icon.clone_from(&self.config.ko);
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
                print!("{string}");
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

impl FullFinalDisplay<'_> {
    /// I'd rather have no info over a system error when reporting time
    fn elapsed(tracker: &JobProgressTracker) -> Option<f64> {
        let time = tracker.end_time.or_else(|| Some(SystemTime::now()))?;

        let since = time.duration_since(tracker.start_time).ok()?;
        #[allow(clippy::cast_precision_loss)]
        Some(since.as_millis() as f64)
    }
}
