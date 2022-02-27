use std::fmt::{Display, Formatter};

pub struct Spinner<'a> {
    ticks: usize,
    roll: usize,
    finished: bool,
    frames: &'a Vec<String>,
    current_frame: usize,
    per_frame: usize,
}

impl<'a> Spinner<'a> {
    pub fn current(&self) -> &'a str {
        self.frames[if self.finished {
            self.roll
        } else {
            self.ticks as usize
        }]
        .as_str()
    }

    pub fn finish(&mut self) {
        self.finished = true
    }

    pub fn blocked(&self) -> &'a str {
        self.frames[self.roll + 1].as_str()
    }

    pub fn new(frames: &'a Vec<String>, per_frame: usize) -> Self {
        Spinner {
            frames,
            finished: false,
            ticks: 0,
            roll: frames.len() - 2,
            current_frame: 0,
            per_frame,
        }
    }

    pub fn tick(&mut self, frames: usize) {
        let up = self.current_frame + frames >= self.per_frame;
        self.ticks = if up {
            (self.ticks + 1) % self.roll
        } else {
            self.ticks
        };
        self.current_frame = if up { 0 } else { self.current_frame + frames };
    }

    pub fn plus_one(&self) -> Self {
        let mut clone = self.clone();
        clone.tick(self.per_frame);
        clone
    }
}

impl<'a> Display for Spinner<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.current())
    }
}

impl<'a> Clone for Spinner<'a> {
    fn clone(&self) -> Self {
        Spinner {
            ticks: self.ticks,
            roll: self.roll,
            finished: false,
            frames: self.frames,
            current_frame: self.current_frame,
            per_frame: self.per_frame,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn spin() {
        let strings = vec!["titi", "tutu", "toto", "tata", "    "]
            .iter()
            .map(|str| str.to_string())
            .collect();
        let mut spinner = Spinner::new(&strings, 1);

        assert_eq!("titi", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("tutu", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("toto", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("titi", format!("{spinner}"));
        spinner.finish();
        assert_eq!("tata", format!("{spinner}"));

        let other_spinner = spinner.plus_one();
        assert_eq!("tutu", format!("{other_spinner}"));
    }
}
