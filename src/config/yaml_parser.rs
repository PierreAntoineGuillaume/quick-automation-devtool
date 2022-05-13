use crate::config::versions::version_0x::Version0x;
use crate::config::versions::version_1::Version1;
use crate::config::{FormatParser, Loader, Version};
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

    fn version(&self, text: &str) -> Result<Version, String> {
        serde_yaml::from_str::<Version>(text).map_err(|why| why.to_string())
    }

    fn version0x(&self, text: &str) -> Result<Box<dyn Loader>, String> {
        Ok(Box::new(
            serde_yaml::from_str::<Version0x>(text).map_err(|error| error.to_string())?,
        ))
    }

    fn version1(&self, text: &str) -> Result<Box<dyn Loader>, String> {
        Ok(Box::new(
            serde_yaml::from_str::<Version1>(text).map_err(|error| error.to_string())?,
        ))
    }

    fn latest_with_warning(
        &self,
        text: &str,
        requested_version: &str,
    ) -> anyhow::Result<Box<dyn Loader>, String> {
        const NAME: &str = crate::PACKAGE_NAME;
        eprintln!(
            "{NAME} config version {} is not available in {NAME} {} please check for new updates. Falling back to {NAME} config version {}",
            requested_version,
            crate::VERSION,
            crate::config::LATEST
        );
        self.version1(text)
    }

    fn format(&self) -> Format {
        Format::Yaml
    }
}
