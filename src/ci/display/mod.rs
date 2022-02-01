use super::job::{JobOutput, JobProgressTracker, Progress};
use super::schedule::CiDisplay;
use std::fmt::Formatter;
use term::StdoutTerminal;

use super::super::term;

mod dict {
    pub const CHECK: &str = "✔";
    pub const CROSS: &str = "✕";
    pub const CRASH: &str = "↺";
    pub const AWAIT: &str = "�";
}

pub struct TermCiDisplay {
    term: Box<StdoutTerminal>,
    lines_written: u16,
}

impl CiDisplay for TermCiDisplay {
    fn refresh(&mut self, tracker: &JobProgressTracker) {
        self.clear();
        for (job_name, progress_collector) in &tracker.states {
            let progress = progress_collector.last().unwrap();
            let pending = progress.is_pending();
            let failed = progress.failed();

            let symbol = if pending {
                dict::AWAIT
            } else if failed {
                dict::CROSS
            } else {
                dict::CHECK
            };

            writeln!(self.term, "{job_name} {symbol}").unwrap();
            self.lines_written += 1;
        }
    }

    fn finish(&mut self, tracker: &JobProgressTracker) {
        self.refresh(tracker);
        self.clear();
        print!("{tracker}")
    }
}

impl TermCiDisplay {
    fn clear(&mut self) {
        (0..self.lines_written as usize).for_each(|_| {
            self.term.cursor_up().unwrap();
            self.term.carriage_return().unwrap();
            self.term.delete_line().unwrap();
        });
        self.term.reset().unwrap();
        self.lines_written = 0;
    }
    pub fn new() -> Self {
        TermCiDisplay {
            term: term::stdout().unwrap(),
            lines_written: 0,
        }
    }
}

fn try_cleanup(input: String) -> String {
    let cleaned = input.trim_end();
    if cleaned.is_empty() {
        String::new()
    } else {
        format!("{cleaned}\n")
    }
}

impl std::fmt::Display for JobProgressTracker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (job_name, progress_collector) in &self.states {
            writeln!(f, "Running tasks for job {job_name}")?;
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Partial(instruction, job_output) => match job_output {
                        JobOutput::Success(stdout, stderr)
                        | JobOutput::JobError(stdout, stderr) => {
                            let symbol = if job_output.succeeded() {
                                dict::CHECK
                            } else {
                                dict::CROSS
                            };
                            write!(
                                f,
                                "{symbol} {}",
                                try_cleanup(format!(
                                    "{}{}{}",
                                    instruction,
                                    try_cleanup(stdout.clone()),
                                    try_cleanup(stderr.clone())
                                ))
                            )?;
                        }
                        JobOutput::ProcessError(stderr) => {
                            write!(
                                f,
                                "{} {instruction}: {}",
                                dict::CRASH,
                                try_cleanup(stderr.clone())
                            )?;
                        }
                    },
                    Progress::Terminated(bool) => {
                        let emoji: &str = if *bool { dict::CHECK } else { dict::CROSS };
                        writeln!(f, "{emoji} all tasks done for job {job_name}")?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
