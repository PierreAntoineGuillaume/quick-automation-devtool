use crate::ci::display::spinner::Spinner;
use crate::ci::display::term_wrapper::TermWrapper;
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::{JobProgressTracker, ProgressCollector};
use crate::ci::job::ports::UserFacade;
use crate::ci::job::Progress;
use std::io::Write;

pub struct Display<'a> {
    spin: Spinner<'a>,
    term: TermWrapper<'a>,
    config: &'a CiDisplayConfig,
    max_job_name_len: usize,
}

impl<'a> UserFacade for Display<'a> {
    fn set_up(&mut self, tracker: &JobProgressTracker) {
        self.max_job_name_len = tracker
            .states
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap();
    }

    fn run(&mut self, tracker: &JobProgressTracker, elapsed: usize) {
        self.term.clear();
        for (job_name, progress_collector) in &tracker.states {
            self.display(job_name, progress_collector);
        }
        self.spin.tick(elapsed);
    }

    fn tear_down(&mut self, _: &JobProgressTracker) {
        self.term.clear();
    }

    fn display_error(&self, error: String) {
        eprintln!("{error}");
    }
}

impl<'a> Display<'a> {
    fn display(&mut self, job_name: &str, collector: &ProgressCollector) {
        let progress = collector.last();

        self.term
            .write(&format!("{:1$}", job_name, self.max_job_name_len));
        match progress {
            Progress::Available => {
                self.term.write("not started yet");
            }
            Progress::Terminated(state) => {
                self.term.write(&format!(
                    " {}",
                    if *state {
                        &self.config.ok
                    } else {
                        &self.config.ko
                    }
                ));
            }
            Progress::Partial(_, _) => {
                self.term.write(&format!(" {}", self.spin));
            }
            Progress::Skipped => {
                self.term
                    .write(&format!(" {} job was skipped", self.config.ok));
            }
            Progress::Blocked(blocked_by) => {
                self.term.write(" blocked by ");
                let mut len = blocked_by.len();
                for job in blocked_by {
                    self.term.write(job);
                    len -= 1;
                    if len > 0 {
                        self.term.write(", ");
                    }
                }
            }
            Progress::Cancelled => {
                self.term.write(&format!(" {}", self.config.cancelled));
            }
            Progress::Started(command) => {
                self.term.write(&format!(" {command} {}", self.spin));
            }
        }
        self.term.newline();
    }

    pub fn new(config: &'a CiDisplayConfig, write: &'a mut dyn Write) -> Self {
        Display {
            term: TermWrapper::new(write),
            spin: Spinner::new(&config.spinner.0, config.spinner.1),
            config,
            max_job_name_len: 0,
        }
    }
}
