use crate::config::versions::version_0x::Version0x;
use crate::config::versions::version_0y::Version0y;
use crate::config::{ConfigLoader, FormatParser, Version};
use crate::Format;
use regex::Regex;

#[derive(Default)]
pub struct YamlParser {}

impl YamlParser {
    pub fn boxed() -> Box<dyn FormatParser> {
        Box::new(YamlParser {})
    }
}

impl FormatParser for YamlParser {
    fn supports(&self, filename: &str) -> bool {
        Regex::new(r"\.ya?ml(\.dist)?$").unwrap().is_match(filename)
    }

    fn version(&self, text: &str) -> Result<Version, ()> {
        serde_yaml::from_str::<Version>(text).map_err(|_| ())
    }

    fn version0x(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String> {
        Ok(Box::new(
            serde_yaml::from_str::<Version0x>(text).map_err(|error| error.to_string())?,
        ))
    }

    fn version0y(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String> {
        Ok(Box::new(
            serde_yaml::from_str::<Version0y>(text).map_err(|error| error.to_string())?,
        ))
    }

    fn format(&self) -> Format {
        Format::Yaml
    }
}
