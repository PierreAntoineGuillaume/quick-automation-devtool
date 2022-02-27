mod spinner;

use crate::ci::job::inspection::{JobProgressTracker, ProgressCollector};
use crate::ci::job::schedule::CiDisplay;
use crate::ci::job::JobOutput;
use crate::ci::job::Progress;
use spinner::Spinner;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use term::StdoutTerminal;

use super::super::term;

mod dict {
    pub const AWAIT: &str = "🔜";
    pub const BLOCKED: &str = "🔐";
    pub const CHECK: &str = "🆗";
    pub const CROSS: &str = "✕";
    pub const CRASH: &str = "↺";
}

pub struct TermCiDisplay<'a> {
    spin: Spinner<'a>,
    term: Box<StdoutTerminal>,
    written_lines: u16,
}

impl<'a> TermCiDisplay<'a> {
    pub fn finish(&mut self, tracker: &JobProgressTracker) {
        self.refresh(tracker, 0);
        self.clear();
        print!("{tracker}")
    }
}

impl<'a> CiDisplay for TermCiDisplay<'a> {
    fn refresh(&mut self, tracker: &JobProgressTracker, elapsed: usize) {
        let previous_written_lines = self.written_lines;
        self.written_lines = 0;
        (0..previous_written_lines).for_each(|_| {
            self.term.cursor_up().unwrap();
        });
        self.term.carriage_return().unwrap();
        let mut spin = self.spin.plus_one();
        for (job_name, progress_collector) in &tracker.states {
            if !progress_collector.progresses.last().unwrap().is_pending() {
                spin.finish()
            }
            self.term.delete_line().unwrap();
            writeln!(
                self.term,
                "{}",
                TempStatusLine::new(&spin, job_name, progress_collector)
            )
            .unwrap();
            self.written_lines += 1;
            spin = spin.plus_one();
        }
        (previous_written_lines..self.written_lines).for_each(|_| {
            self.term.delete_line().unwrap();
        });
        self.spin.tick(elapsed);
    }
}

struct TempStatusLine<'a> {
    spin: &'a Spinner<'a>,
    job_name: &'a str,
    progress_collector: &'a ProgressCollector,
}

impl Display for TempStatusLine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use dict::*;
        let progress = self.progress_collector.last().unwrap();

        match progress {
            Progress::Terminated(true) => {
                write!(f, "{:12} {} {CHECK}", self.job_name, self.spin)
            }
            Progress::Terminated(false) => {
                write!(f, "{:12} {} {CROSS}", self.job_name, self.spin)
            }
            Progress::Blocked => {
                write!(
                    f,
                    "{:12} {}   {BLOCKED}",
                    self.job_name,
                    self.spin.blocked()
                )
            }
            _ => {
                write!(f, "{:12} {} {AWAIT}", self.job_name, self.spin)
            }
        }
    }
}

impl<'a> TempStatusLine<'a> {
    fn new(
        spin: &'a Spinner,
        job_name: &'a str,
        progress_collector: &'a ProgressCollector,
    ) -> Self {
        TempStatusLine {
            spin,
            job_name,
            progress_collector,
        }
    }
}

impl<'a> TermCiDisplay<'a> {
    fn clear(&mut self) {
        (0..self.written_lines as usize).for_each(|_| {
            self.term.cursor_up().unwrap();
            self.term.carriage_return().unwrap();
            self.term.delete_line().unwrap();
        });
        self.term.reset().unwrap();
        self.written_lines = 0;
    }
    pub fn new(states: &'a Vec<String>, per_frame: usize) -> Self {
        TermCiDisplay {
            term: term::stdout().unwrap(),
            written_lines: 0,
            spin: Spinner::new(states, per_frame),
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

impl Display for JobProgressTracker {
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
                                "{} {}",
                                symbol,
                                try_cleanup(format!(
                                    "{}\n{}{}",
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
                        writeln!(f, "{} all tasks done for job {}", emoji, job_name)?;
                    }
                    _ => {}
                }
            }
        }
        let time = self.end_time.or_else(|| Some(SystemTime::now())).unwrap();
        let elasped = time.duration_since(self.start_time).unwrap().as_millis() as f64;
        let status = if !self.has_failed {
            (dict::CHECK, "succeeded")
        } else {
            (dict::CROSS, "failed")
        };
        writeln!(
            f,
            "\n{} ci {} in {:.2} seconds",
            status.0,
            status.1,
            elasped / 1000f64
        )?;
        Ok(())
    }
}
