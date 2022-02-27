mod version_0x;
mod wrapped_content;

use crate::ci::CiConfig;
use serde::Deserialize;
use std::fs;
use version_0x::Version0x;
use wrapped_content::WrappedContent;

#[derive(Debug)]
pub enum ConfigError {
    UnrecognizedFileformat,
    NoVersion(&'static str),
    BadVersion(String, &'static str),
    ParseError(String),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version {
    pub version: String,
}

#[derive(Debug, PartialEq)]
pub struct Config {
    pub version: Version,
    pub content: WrappedContent,
}

impl Config {
    pub fn parse(filename: &str) -> Result<Config, String> {
        let content =
            fs::read_to_string(&filename).map_err(|_| format!("could not read {}", filename))?;

        let content = if filename.ends_with(".toml") {
            Config::parse_toml(&content)
        } else if filename.ends_with(".yaml") || filename.ends_with(".yml") {
            Config::parse_yaml(&content)
        } else {
            Err(ConfigError::UnrecognizedFileformat)
        };

        content.map_err(|error| match error {
            ConfigError::UnrecognizedFileformat => {
                format!(
                    "{} could not be parsed, expected file types are .toml, .yml or .yaml",
                    filename
                )
            }
            ConfigError::NoVersion(latest) => {
                format!(
                    "{} should contain version id (latest is {})",
                    filename, latest
                )
            }
            ConfigError::BadVersion(version, latest) => {
                format!(
                    "unknown version {} in {} (latest is {})",
                    version, filename, latest
                )
            }
            ConfigError::ParseError(version) => {
                format!("could not parse {} with version {}", filename, version)
            }
        })
    }

    fn parse_toml(content: &str) -> Result<Config, ConfigError> {
        let version =
            toml::from_str::<Version>(content).map_err(|_| ConfigError::NoVersion("0.x"))?;

        if version.version != "0.x" {
            return Err(ConfigError::BadVersion(version.version, "0.x"));
        }

        let v0x = toml::from_str::<Version0x>(content)
            .map_err(|_| ConfigError::ParseError(version.version.clone()))?;

        Ok(Config {
            version,
            content: WrappedContent::V0x(v0x),
        })
    }

    fn parse_yaml(content: &str) -> Result<Config, ConfigError> {
        let version =
            serde_yaml::from_str::<Version>(content).map_err(|_| ConfigError::NoVersion("0.x"))?;
        if version.version != "0.x" {
            return Err(ConfigError::BadVersion(version.version, "0.x"));
        }

        let v0x = serde_yaml::from_str::<Version0x>(content)
            .map_err(|_| ConfigError::ParseError(version.version.clone()))?;

        Ok(Config {
            version,
            content: WrappedContent::V0x(v0x),
        })
    }

    pub fn load_into(&self, pipeline: &mut CiConfig) {
        self.content.load_into(pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use version_0x::JobSet;

    impl FromStr for Version {
        type Err = ();
        fn from_str(s: &str) -> Result<Self, ()> {
            Ok(Version {
                version: s.to_string(),
            })
        }
    }

    #[test]
    pub fn parse_good_v0x_toml() -> Result<(), ()> {
        let mut job_set = JobSet::new();
        job_set.insert(
            String::from("jobname"),
            vec![String::from("inst1"), String::from("inst2")],
        );

        assert_eq!(
            Config::parse_toml("version = \"0.x\"\n[jobs]\njobname = [\"inst1\", \"inst2\"]")
                .unwrap(),
            Config {
                version: "0.x".parse::<Version>()?,
                content: WrappedContent::V0x(Version0x::new(job_set)),
            }
        );

        Ok(())
    }

    #[test]
    pub fn parse_good_v0x_yaml() -> Result<(), ()> {
        let mut job_set = JobSet::new();
        job_set.insert(
            String::from("jobname"),
            vec![String::from("inst1"), String::from("inst2")],
        );

        assert_eq!(
            Config::parse_yaml("{ version: 0.x, jobs: { jobname: [inst1, inst2] } }").unwrap(),
            Config {
                version: "0.x".parse::<Version>()?,
                content: WrappedContent::V0x(Version0x::new(job_set)),
            }
        );

        Ok(())
    }
}
