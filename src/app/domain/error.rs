use std::fmt::{Display, Formatter};

pub fn err<T: ToString>(what: &T) -> Error {
    let location = std::panic::Location::caller();
    Error::new(
        what.to_string(),
        location.file().to_string(),
        location.line(),
    )
}

#[derive(Debug)]
pub struct Error {
    pub what: String,
    pub file: String,
    pub line: u32,
}

impl Error {
    #[must_use]
    const fn new(what: String, file: String, line: u32) -> Self {
        Self { what, file, line }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.what)
    }
}

impl std::error::Error for Error {}
