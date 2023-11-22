use std::cmp::max;

#[derive(Clone)]
pub struct StatefulText {
    pub scroll: u16,
    pub text: String,
    lc: u16,
}

impl StatefulText {
    pub fn with_text(text: String) -> Self {
        let lc: u16 = text
            .lines()
            .count()
            .try_into()
            .expect("lines should be small");
        Self {
            scroll: 0,
            text,
            lc,
        }
    }

    pub fn next(&mut self) {
        self.scroll = match self.scroll {
            i if i >= self.lc => 0,
            i => i + max(1, self.lc / 100),
        };
    }

    pub fn previous(&mut self) {
        self.scroll = match self.scroll {
            0 => self.lc - 1,
            i => i - max(1, self.lc / 100),
        };
    }
}
