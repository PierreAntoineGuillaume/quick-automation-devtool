mod spinner;

use crate::ci::job::inspection::{JobProgressTracker, ProgressCollector};
use crate::ci::job::schedule::CiDisplay;
use crate::ci::job::JobOutput;
use crate::ci::job::Progress;
use spinner::Spinner;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use term::StdoutTerminal;

pub struct CiDisplayDict {
    pub ok: String,
    pub ko: String,
}

impl Default for CiDisplayDict {
    fn default() -> Self {
        Self {
            ok: String::from("✔"),
            ko: String::from("✕"),
        }
    }
}

pub struct TermCiDisplay<'a> {
    spin: Spinner<'a>,
    term: Box<StdoutTerminal>,
    written_lines: u16,
    dict: CiDisplayDict,
}

impl<'a> TermCiDisplay<'a> {
    pub fn finish(&mut self, tracker: &JobProgressTracker) {
        self.refresh(tracker, 0);
        self.clear();
        for (job_name, progress_collector) in &tracker.states {
            println!("Running tasks for job {job_name}");
            for progress in &progress_collector.progresses {
                match progress {
                    Progress::Partial(instruction, job_output) => match job_output {
                        JobOutput::Success(stdout, stderr)
                        | JobOutput::JobError(stdout, stderr) => {
                            let symbol = if job_output.succeeded() {
                                &self.dict.ok
                            } else {
                                &self.dict.ko
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
                                self.dict.ko,
                                try_cleanup(stderr.clone())
                            );
                        }
                    },
                    Progress::Terminated(bool) => {
                        let emoji: &str = if *bool { &self.dict.ok } else { &self.dict.ko };
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
            (&self.dict.ok, "succeeded")
        } else {
            (&self.dict.ko, "failed")
        };
        println!(
            "\n{} ci {} in {:.2} seconds",
            status.0,
            status.1,
            elasped / 1000f64
        );
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
            self.term.delete_line().unwrap();
            writeln!(
                self.term,
                "{}",
                TempStatusLine::new(&spin, job_name, progress_collector, &self.dict)
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
    dict: &'a CiDisplayDict,
}

impl Display for TempStatusLine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let progress = self.progress_collector.last().unwrap();

        match progress {
            Progress::Available => {
                write!(f, "{:12} not started yet", self.job_name)
            }
            Progress::Terminated(true) => {
                write!(f, "{:12} {}", self.job_name, self.dict.ok)
            }
            Progress::Terminated(false) => {
                write!(f, "{:12} {}", self.job_name, self.dict.ko)
            }
            Progress::Partial(_, _) => {
                write!(f, "{:12} {}", self.job_name, self.spin)
            }
            Progress::Blocked(blocked_by) => {
                write!(f, "{:12} blocked by ", self.job_name)?;
                let mut len = blocked_by.len();
                for job in blocked_by {
                    write!(f, "{}", job)?;
                    len -= 1;
                    if len > 0 {
                        write!(f, ", ")?;
                    }
                }
                Ok(())
            }

            Progress::Started(command) => {
                write!(f, "{:12} {} {}", self.job_name, command, self.spin)
            }
        }
    }
}

impl<'a> TempStatusLine<'a> {
    fn new(
        spin: &'a Spinner,
        job_name: &'a str,
        progress_collector: &'a ProgressCollector,
        dict: &'a CiDisplayDict,
    ) -> Self {
        TempStatusLine {
            spin,
            job_name,
            progress_collector,
            dict,
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
    pub fn new(states: &'a Vec<String>, per_frame: usize, dict: CiDisplayDict) -> Self {
        TermCiDisplay {
            term: term::stdout().unwrap(),
            written_lines: 0,
            spin: Spinner::new(states, per_frame),
            dict,
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
