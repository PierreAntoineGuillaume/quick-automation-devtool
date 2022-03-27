use crate::terminal_size::terminal_size;
use term::StdoutTerminal;

pub struct TermWrapper {
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
