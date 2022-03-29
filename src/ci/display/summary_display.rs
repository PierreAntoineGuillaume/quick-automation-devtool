use crate::ci::display::spinner::Spinner;
use crate::ci::display::term_wrapper::TermWrapper;
use crate::ci::display::CiDisplayConfig;
use crate::ci::job::inspection::{InstructionState, JobProgressTracker, ProgressCollector};
use crate::ci::job::schedule::RunningCiDisplay;
use std::cmp::max;

pub struct SummaryDisplay<'a> {
    spin: Spinner<'a>,
    term: TermWrapper,
    config: &'a CiDisplayConfig,
    max_job_name_len: usize,
}

impl<'a> RunningCiDisplay for SummaryDisplay<'a> {
    fn refresh(&mut self, tracker: &JobProgressTracker, elapsed: usize) {
        self.term.clear();

        for (job_name, _) in &tracker.states {
            self.max_job_name_len = max(self.max_job_name_len, job_name.len());
        }
        for (job_name, progress_collector) in &tracker.states {
            self.display(job_name, progress_collector);
        }
        self.term.flush();
        self.spin.tick(elapsed);
    }

    fn clean_up(&mut self) {
        self.clear();
        self.term.flush();
    }
}

impl<'a> SummaryDisplay<'a> {
    fn display(&mut self, job_name: &str, collector: &ProgressCollector) {
        self.term
            .write(&format!("{:1$}  ", job_name, self.max_job_name_len));
        if let Some(result) = collector.terminated() {
            self.term.write(if result {
                &self.config.ok
            } else {
                &self.config.ko
            })
        }
        self.term.newline();
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
                    self.term
                        .write(&format!("{}    {}", self.spin, instruction));
                }
            }
            self.term.newline();
        }

        self.term.newline();
    }

    fn clear(&mut self) {
        self.term.clear()
    }

    pub fn new(config: &'a CiDisplayConfig) -> Self {
        SummaryDisplay {
            term: TermWrapper::default(),
            spin: Spinner::new(&config.spinner.0, config.spinner.1),
            config,
            max_job_name_len: 0,
        }
    }
}
