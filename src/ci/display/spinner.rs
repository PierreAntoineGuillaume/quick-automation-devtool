use std::fmt::{Display, Formatter};

pub struct Spinner {
    ticks: usize,
    roll: usize,
    finished: bool,
    frames: &'static [&'static str],
    current_frame: usize,
    per_frame: usize,
}

impl Spinner {
    pub fn new(frames: &'static [&str], per_frame: usize) -> Self {
        Spinner {
            frames,
            finished: false,
            ticks: 0,
            roll: frames.len() - 1,
            current_frame: 0,
            per_frame,
        }
    }
}

impl Display for Spinner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.frames[if self.finished {
                self.roll
            } else {
                self.ticks as usize
            }]
        )
    }
}

impl Spinner {
    pub fn tick(&mut self, frames: usize) {
        self.current_frame += frames;
        if self.current_frame >= self.per_frame {
            self.current_frame = 0;
            self.ticks = (self.ticks + 1) % self.roll;
        }
    }

    pub fn finish(&mut self) {
        self.finished = true
    }

    pub fn plus_one(&self) -> Self {
        let mut plus_one = Self {
            ticks: self.ticks,
            roll: self.roll,
            finished: false,
            frames: self.frames,
            per_frame: self.per_frame,
            current_frame: 0,
        };
        plus_one.tick(self.per_frame);
        plus_one
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn spin() {
        let mut spinner = Spinner::new(&["titi", "tutu", "toto", "tata"], 1);

        assert_eq!("titi", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("tutu", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("toto", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("titi", format!("{spinner}"));
        spinner.finish();
        assert_eq!("tata", format!("{spinner}"));
        spinner.tick(1);
        assert_eq!("tata", format!("{spinner}"));

        let other_spinner = spinner.plus_one();
        assert_eq!("toto", format!("{other_spinner}"));
    }
}
