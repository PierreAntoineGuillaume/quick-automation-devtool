use crate::ci::display::spinner::Spinner;
use crate::ci::display::term_wrapper::TermWrapper;
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::{JobProgressTracker, ProgressCollector};
use crate::ci::job::ports::UserFacade;
use crate::ci::job::Progress;
use std::fmt::Write;

pub struct Display<'a> {
    spin: Spinner<'a>,
    term: TermWrapper<'a>,
    config: &'a CiDisplayConfig,
    max_job_name_len: usize,
}

impl UserFacade for Display<'_> {
    fn set_up(&mut self, tracker: &JobProgressTracker) {
        self.max_job_name_len = tracker.find_longest_jobname_size();
    }

    fn run(&mut self, tracker: &JobProgressTracker, elapsed: usize) {
        self.term.clear();
        for (job_name, progress_collector) in &tracker.states {
            let result = self.display(job_name, progress_collector);
            self.term.write(&result);
            self.term.newline();
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
    fn display(&self, job_name: &str, collector: &ProgressCollector) -> String {
        let progress = collector.last();
        let mut str = String::new();

        write!(str, "{:1$}", job_name, self.max_job_name_len).expect("Can't write");

        match progress {
            Progress::Available => {
                str.push_str("not started yet");
            }
            Progress::Terminated(state) => {
                write!(
                    str,
                    " {}",
                    if *state {
                        &self.config.ok
                    } else {
                        &self.config.ko
                    }
                )
                .expect("Can't write");
            }
            Progress::Partial(_, _) => {
                write!(str, " {}", self.spin).expect("Can't write");
            }
            Progress::Skipped => {
                write!(str, " {} job was skipped", self.config.ok).expect("Can't write");
            }
            Progress::Blocked(blocked_by) => {
                write!(str, " blocked by ").expect("Can't write");
                let mut len = blocked_by.len();
                for job in blocked_by {
                    str.push_str(job);
                    len -= 1;
                    if len > 0 {
                        str.push_str(", ");
                    }
                }
            }
            Progress::Cancelled => {
                write!(str, " {}", self.config.cancelled).expect("Can't write");
            }
            Progress::Started(command) => {
                write!(str, " {command} {}", self.spin).expect("Can't write");
            }
        }
        str
    }

    pub fn new(config: &'a CiDisplayConfig, write: &'a mut dyn std::io::Write) -> Self {
        Self {
            term: TermWrapper::new(write),
            spin: Spinner::new(&config.spinner.0, config.spinner.1),
            config,
            max_job_name_len: 0,
        }
    }
}
