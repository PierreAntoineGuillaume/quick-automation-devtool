use crate::config::versions::version_1::Version1;
use crate::config::{FormatParser, Loader, Version};

#[derive(Default)]
pub struct YamlParser {}

impl FormatParser for YamlParser {
    fn version(&self, text: &str) -> Result<Version, String> {
        serde_yaml::from_str::<Version>(text).map_err(|why| why.to_string())
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
}
