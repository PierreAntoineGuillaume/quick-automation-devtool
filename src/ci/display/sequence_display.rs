use crate::ci::display::full_final_display::FullFinalDisplay;
use crate::ci::display::spinner::Spinner;
use crate::ci::display::term_wrapper::TermWrapper;
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::CiDisplay;
use crate::ci::job::Progress;
use std::cmp::max;

pub struct SequenceDisplay<'a> {
    spin: Spinner<'a>,
    term: TermWrapper,
    config: &'a CiDisplayConfig,
    max_job_name_len: usize,
}

impl<'a> CiDisplay for SequenceDisplay<'a> {
    fn refresh(&mut self, tracker: &JobProgressTracker, elapsed: usize) {
        self.term.clear();
        for (job_name, _) in &tracker.states {
            self.max_job_name_len = max(self.max_job_name_len, job_name.len());
        }
        for (job_name, progress_collector) in &tracker.states {
            self.display(job_name, progress_collector.last());
        }
        self.term.flush();
        self.spin.tick(elapsed);
    }
    fn finish(&mut self, tracker: &JobProgressTracker) {
        self.refresh(tracker, 0);
        self.clear();
        self.term.flush();
        let mut final_display = FullFinalDisplay::new(self.config);
        final_display.finish(tracker);
    }
}

impl<'a> SequenceDisplay<'a> {
    fn display(&mut self, job_name: &str, progress: &Progress) {
        self.term
            .write(&format!("{:1$}", job_name, self.max_job_name_len));
        match progress {
            Progress::Available => {
                self.term.write("not started yet");
            }
            Progress::Terminated(true) => {
                self.term.write(&format!(" {}", self.config.ok));
            }
            Progress::Terminated(false) => {
                self.term.write(&format!(" {}", self.config.ko));
            }
            Progress::Partial(_, _) => {
                self.term.write(&format!(" {}", self.spin));
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
                self.term.write(&format!(" {} {}", command, self.spin));
            }
        }
        self.term.newline();
    }

    fn clear(&mut self) {
        self.term.clear()
    }

    pub fn new(config: &'a CiDisplayConfig) -> Self {
        SequenceDisplay {
            term: TermWrapper::default(),
            spin: Spinner::new(&config.spinner.0, config.spinner.1),
            config,
            max_job_name_len: 0,
        }
    }
}
