mod spinner;

use crate::ci::job::inspection::JobProgressTracker;
use crate::ci::job::schedule::CiDisplay;
use crate::ci::job::JobOutput;
use crate::ci::job::Progress;
use crate::terminal_size::terminal_size;
use spinner::Spinner;
use std::cmp::max;
use std::time::SystemTime;
use term::StdoutTerminal;

pub struct CiDisplayDict {
    pub ok: String,
    pub ko: String,
    pub cancelled: String,
    pub display_command: bool,
}

impl Default for CiDisplayDict {
    fn default() -> Self {
        Self {
            ok: String::from("✔"),
            ko: String::from("✕"),
            cancelled: String::from("? cancelled"),
            display_command: true,
        }
    }
}

pub struct TermCiDisplay<'a> {
    spin: Spinner<'a>,
    term: TermWrapper,
    dict: CiDisplayDict,
    max_job_name_len: usize,
}

impl<'a> TermCiDisplay<'a> {
    pub fn finish(&mut self, tracker: &JobProgressTracker) {
        self.refresh(tracker, 0);
        self.clear();
        self.term.flush();
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
}

struct TermWrapper {
    term: Box<StdoutTerminal>,
    written_lines: u16,
    written_chars: usize,
}

impl Default for TermWrapper {
    fn default() -> Self {
        Self {
            term: term::stdout().unwrap(),
            written_lines: 0,
            written_chars: 0,
        }
    }
}

impl TermWrapper {
    pub fn newline(&mut self) {
        self.written_lines += 1;
        self.written_chars = 0;
        writeln!(self.term).unwrap();
    }
    pub fn write(&mut self, message: &str) {
        let termsize = terminal_size().unwrap().0 .0 as usize;
        write!(self.term, "{}", message).unwrap();
        self.written_chars += message.len();
        if self.written_chars > termsize {
            self.written_chars %= termsize;
            self.written_lines += 1;
        }
    }
    pub fn clear(&mut self) {
        (0..self.written_lines as usize).for_each(|_| {
            self.term.cursor_up().unwrap();
            self.term.carriage_return().unwrap();
            self.term.delete_line().unwrap();
        });
        self.written_lines = 0;
        self.written_chars = 0;
    }

    pub fn flush(&mut self) {
        self.term.reset().unwrap();
    }
}

impl<'a> TermCiDisplay<'a> {
    fn display(&mut self, job_name: &str, progress: &Progress) {
        self.term
            .write(&format!("{:1$}", job_name, self.max_job_name_len));
        match progress {
            Progress::Available => {
                self.term.write("not started yet");
            }
            Progress::Terminated(true) => {
                self.term.write(&format!(" {}", self.dict.ok));
            }
            Progress::Terminated(false) => {
                self.term.write(&format!(" {}", self.dict.ko));
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
                self.term.write(&format!(" {}", self.dict.cancelled));
            }
            Progress::Started(command) => {
                if self.dict.display_command {
                    self.term.write(&format!(" {} {}", command, self.spin));
                } else {
                    self.term.write(&format!(" {}", self.spin));
                }
            }
        }
        self.term.newline();
    }

    fn clear(&mut self) {
        self.term.clear()
    }

    pub fn new(states: &'a Vec<String>, per_frame: usize, dict: CiDisplayDict) -> Self {
        TermCiDisplay {
            term: TermWrapper::default(),
            spin: Spinner::new(states, per_frame),
            dict,
            max_job_name_len: 0,
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
