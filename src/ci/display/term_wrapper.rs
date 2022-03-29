use crate::terminal_size::terminal_size;
use std::io::Write;

#[derive(Default)]
pub struct TermWrapper {
    written_lines: u16,
    written_chars: usize,
}

const CLEAR_TIL_EO_SCREEN: [u8; 4] = [27, b'[', b'0', b'J'];

impl TermWrapper {
    pub fn newline(&mut self) {
        self.written_lines += 1;
        self.written_chars = 0;
        println!();
    }

    pub fn write(&mut self, message: &str) {
        let termsize = terminal_size().unwrap().0 .0 as usize;
        print!("{}", message);
        self.written_chars += message.len();
        if self.written_chars > termsize {
            self.written_chars %= termsize;
            self.written_lines += 1;
        }
    }

    pub fn clear(&mut self) {
        if self.written_lines == 0 && self.written_chars == 0 {
            return;
        }
        let mut term_seq = vec![27, b'['];

        term_seq.extend(self.written_lines.to_string().into_bytes());
        term_seq.extend(&[b'A']);
        term_seq.extend(CLEAR_TIL_EO_SCREEN);

        std::io::stdout().write_all(&term_seq).unwrap();
        self.written_lines = 0;
        self.written_chars = 0;
    }
}
