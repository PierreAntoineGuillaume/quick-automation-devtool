use std::fmt::{Display, Formatter};

pub struct UnderlineChar();

impl Display for UnderlineChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[4m", 27 as char)
    }
}

pub struct ResetChar();

impl Display for ResetChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[0m", 27 as char)
    }
}
