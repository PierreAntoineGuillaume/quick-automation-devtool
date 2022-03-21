use crate::config::versions::version_0x::Version0x;
use crate::config::versions::version_0y::Version0y;
use crate::config::{ConfigLoader, FormatParser, Version};
use crate::Format;
use regex::Regex;

#[derive(Default)]
pub struct TomlParser {}

impl TomlParser {
    pub fn boxed() -> Box<dyn FormatParser> {
        Box::new(TomlParser {})
    }
}

impl FormatParser for TomlParser {
    fn supports(&self, filename: &str) -> bool {
        Regex::new(r"\.toml(\.dist)?$").unwrap().is_match(filename)
    }

    fn version(&self, text: &str) -> Result<Version, ()> {
        toml::from_str::<Version>(text).map_err(|_| ())
    }

    fn version0x(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String> {
        Ok(Box::new(
            toml::from_str::<Version0x>(text).map_err(|error| error.to_string())?,
        ))
    }

    fn version0y(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String> {
        Ok(Box::new(
            toml::from_str::<Version0y>(text).map_err(|error| error.to_string())?,
        ))
    }

    fn format(&self) -> Format {
        Format::Toml
    }
}
