use std::fmt::{Display, Formatter};

pub struct Spinner {
    ticks: u8,
    roll: usize,
    finished: bool,
    frames: &'static [&'static str],
}

impl Spinner {
    pub fn new(frames: &'static [&str]) -> Self {
        Spinner {
            frames,
            finished: false,
            ticks: 0,
            roll: frames.len() - 1,
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
    pub fn tick(&mut self) {
        self.ticks += 1;
        self.ticks %= self.roll as u8;
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
        };
        plus_one.tick();
        plus_one
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn spin() {
        let mut spinner = Spinner::new(&["titi", "tutu", "toto", "tata"]);

        assert_eq!("titi", format!("{spinner}"));
        spinner.tick();
        assert_eq!("tutu", format!("{spinner}"));
        spinner.tick();
        assert_eq!("toto", format!("{spinner}"));
        spinner.tick();
        assert_eq!("titi", format!("{spinner}"));
        spinner.finish();
        assert_eq!("tata", format!("{spinner}"));
        spinner.tick();
        assert_eq!("tata", format!("{spinner}"));

        let other_spinner = spinner.plus_one();
        assert_eq!("toto", format!("{other_spinner}"));
    }
}
