mod version_0x;

use crate::ci::CiConfig;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use version_0x::Version0x;

#[derive(Debug)]
pub enum ConfigError {
    UnrecognizedFileformat,
    NoVersion(&'static str),
    BadVersion(String, &'static str),
    ParseError(String, String),
}

impl ConfigError {
    fn explain(&self, filename: String) -> String {
        match self {
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
            ConfigError::ParseError(version, prev) => {
                format!(
                    "could not parse {} with version {} ({})",
                    filename, version, prev
                )
            }
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version {
    pub version: String,
}

pub struct Config {
    possible_files: Vec<String>,
}

#[derive(Default)]
pub struct ConfigPayload {
    pub ci_config: CiConfig,
}

pub trait ConfigLoader {
    fn load(&self, payload: &mut ConfigPayload);
}

impl Config {
    pub fn from(env: &str) -> Self {
        let possible_files: Vec<String> = ["toml", "yaml", "yml"]
            .iter()
            .map(|str| format!("{}.{}", env, str))
            .collect();

        Config { possible_files }
    }

    pub fn load(&self) -> Result<ConfigPayload, String> {
        let filename = self.get_first_available_config_file()?;

        let content =
            fs::read_to_string(&filename).map_err(|_| format!("could not read {}", filename))?;

        let loader = if filename.ends_with(".toml") {
            Config::parse_toml(&content)
        } else if filename.ends_with(".yaml") || filename.ends_with(".yml") {
            Config::parse_yaml(&content)
        } else {
            Err(ConfigError::UnrecognizedFileformat)
        }
        .map_err(|error| error.explain(filename))?;

        let mut default = ConfigPayload::default();

        loader.load(&mut default);

        Ok(default)
    }

    fn get_first_available_config_file(&self) -> Result<String, String> {
        let mut filename = None;
        for file in &self.possible_files {
            if Path::new(file).exists() {
                filename = Some(file.clone());
                break;
            }
        }

        filename.ok_or_else(|| {
            format!(
                "no config file could be found (looked for {:?})",
                self.possible_files
            )
        })
    }

    fn parse_toml(content: &str) -> Result<Box<dyn ConfigLoader>, ConfigError> {
        let version =
            toml::from_str::<Version>(content).map_err(|_| ConfigError::NoVersion("0.x"))?;

        if version.version != "0.x" {
            return Err(ConfigError::BadVersion(version.version, "0.x"));
        }

        let v0x = toml::from_str::<Version0x>(content)
            .map_err(|e| ConfigError::ParseError(version.version.clone(), e.to_string()))?;

        Ok(Box::new(v0x))
    }

    fn parse_yaml(content: &str) -> Result<Box<dyn ConfigLoader>, ConfigError> {
        let version =
            serde_yaml::from_str::<Version>(content).map_err(|_| ConfigError::NoVersion("0.x"))?;
        if version.version != "0.x" {
            return Err(ConfigError::BadVersion(version.version, "0.x"));
        }

        let v0x = serde_yaml::from_str::<Version0x>(content)
            .map_err(|e| ConfigError::ParseError(version.version.clone(), e.to_string()))?;

        Ok(Box::new(v0x))
    }
}
