use crate::terminal_size::terminal_size;
use std::io::Write;

#[derive(Default)]
pub struct TermWrapper {
    written_lines: u16,
    written_chars: usize,
}

/// https://handwiki.org/wiki/ANSI_escape_code#Colors
const CLEAR_TIL_EOL: [u8; 5] = [27, b'[', b'0', b'K', b'\n'];
const CLEAR_TIL_EO_SCREEN: [u8; 4] = [27, b'[', b'0', b'J'];

impl TermWrapper {
    pub fn rewind(&mut self) {
        if self.written_lines == 0 && self.written_chars == 0 {
            return;
        }
        let mut term_seq = vec![27, b'['];

        term_seq.extend(self.written_lines.to_string().into_bytes());
        term_seq.extend(&[b'A']);

        std::io::stdout().write_all(&term_seq).unwrap();
        self.written_lines = 0;
        self.written_chars = 0;
    }

    pub fn clear_til_eol(&mut self) {
        self.written_lines += 1;
        self.written_chars = 0;
        std::io::stdout().write_all(&CLEAR_TIL_EOL).unwrap();
    }

    pub fn clear_til_eo_screen(&mut self) {
        std::io::stdout().write_all(&CLEAR_TIL_EO_SCREEN).unwrap()
    }

    pub fn newline(&mut self) {
        self.written_lines += 1;
        self.written_chars = 0;
        println!();
    }

    pub fn write(&mut self, message: &str) {
        let termsize = terminal_size().unwrap().0 .0 as usize;
        let mut redo = false;
        for sub in message.split('\n') {
            if redo {
                self.clear_til_eol();
                println!();
                self.written_lines += 1;
                self.written_chars = 0;
            }
            print!("{}", sub);
            if self.written_chars > termsize {
                self.written_chars %= termsize;
                self.written_lines += 1;
            }
            redo = true;
        }
    }

    pub fn clear(&mut self) {
        if self.written_lines == 0 && self.written_chars == 0 {
            return;
        }
        self.rewind();
        self.clear_til_eo_screen();
    }
}
