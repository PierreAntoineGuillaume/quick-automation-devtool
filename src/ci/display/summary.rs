use crate::ci::display::spinner::Spinner;
use crate::ci::display::term_wrapper::TermWrapper;
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::{InstructionState, JobProgressTracker, ProgressCollector};
use crate::ci::job::ports::UserFacade;
use std::io::Write;

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
        self.term.rewind();

        for (job_name, progress_collector) in &tracker.states {
            self.display(job_name, progress_collector);
        }
        self.term.clear_til_eo_screen();
        self.spin.tick(elapsed);
    }

    fn tear_down(&mut self, _: &JobProgressTracker) {
        self.clear();
    }

    fn display_error(&self, error: String) {
        eprintln!("{error}");
    }
}

impl<'a> Display<'a> {
    fn display(&mut self, job_name: &str, collector: &ProgressCollector) {
        self.term
            .write(&format!("{:1$} ", job_name, self.max_job_name_len));
        if let Some(result) = collector.terminated() {
            self.term.write(if result {
                &self.config.ok
            } else {
                &self.config.ko
            });
        }
        self.term.clear_til_eol();

        let spin_len = self.spin.current().len();
        for instruction in collector.instruction_list() {
            match instruction {
                InstructionState::Finished(instruction, success) => {
                    self.term.write(&format!(
                        "{:2$}    {}",
                        if success {
                            &self.config.ok
                        } else {
                            &self.config.ko
                        },
                        instruction,
                        spin_len,
                    ));
                }
                InstructionState::Running(instruction) => {
                    self.term.write(&format!("{}    {instruction}", self.spin));
                }
            }
            self.term.clear_til_eol();
        }
    }

    fn clear(&mut self) {
        self.term.clear();
    }

    pub fn new(config: &'a CiDisplayConfig, write: &'a mut dyn Write) -> Self {
        Self {
            term: TermWrapper::new(write),
            spin: Spinner::new(&config.spinner.0, config.spinner.1),
            config,
            max_job_name_len: 0,
        }
    }
}
