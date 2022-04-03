pub mod argh;
pub mod instructions;
pub mod migrate;
mod serialization;
mod versions;

use crate::ci::CiConfig;
use crate::config::serialization::toml_parser::TomlParser;
use crate::config::serialization::yaml_parser::YamlParser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum ConfigError {
    FileError(String),
    NoVersion(&'static str),
    BadVersion(String, &'static str),
    ParseError(String, String),
}

impl ConfigError {
    fn explain(&self, filename: &str) -> String {
        match self {
            ConfigError::FileError(error) => {
                format!("{} could not be parsed: {}", filename, error,)
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

#[derive(Serialize, Deserialize, Debug, PartialOrd, PartialEq, Clone)]
pub enum Format {
    Yaml,
    Toml,
}

impl Default for Format {
    fn default() -> Self {
        Self::Toml
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
    pub ci: CiConfig,
    pub env: Option<String>,
}

pub trait ConfigLoader {
    fn load(&self, payload: &mut ConfigPayload);
}

pub trait FormatParser {
    fn supports(&self, filename: &str) -> bool;
    fn version(&self, text: &str) -> Result<Version, ()>;
    fn version0x(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String>;
    fn version0y(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String>;
    fn format(&self) -> Format;
}

impl Config {
    pub fn get_first_available_config_file(&self) -> Result<String, String> {
        let mut filename = None;
        for file in &self.possible_files {
            if Path::new(file).exists() {
                filename = Some(file.clone());
                break;
            }
        }

        filename.ok_or_else(|| {
            format!(
                "no config file could be found (looked in files {:?})",
                self.possible_files
            )
        })
    }

    pub fn from(env: &str) -> Self {
        let possible_files: Vec<String> =
            ["toml", "yaml", "yml", "toml.dist", "yaml.dist", "yml.dist"]
                .iter()
                .map(|str| format!("{}.{}", env, str))
                .collect();

        Config { possible_files }
    }

    pub fn load_into(&self, config: &mut ConfigPayload) -> Result<(), String> {
        let filename = self.get_first_available_config_file()?;

        let content = fs::read_to_string(&filename)
            .map_err(|error| ConfigError::FileError(error.to_string()).explain(&filename))?;

        let loader = self
            .parse(&filename, &content)
            .map_err(|error| error.explain(&filename))?;
        loader.load(config);
        Ok(())
    }

    pub fn load_with_args_into(&self, config: &mut ConfigPayload) -> Result<(), String> {
        self.load_into(config)?;
        Ok(())
    }

    pub fn get_parser(&self, filename: &str) -> Option<Box<dyn FormatParser>> {
        let parsers = [YamlParser::boxed(), TomlParser::boxed()];
        for parser in parsers {
            if !parser.supports(filename) {
                continue;
            }
            return Some(parser);
        }
        None
    }

    pub fn parse(
        &self,
        filename: &str,
        content: &str,
    ) -> Result<Box<dyn ConfigLoader>, ConfigError> {
        let parser = self
            .get_parser(filename)
            .expect("This could not be reached, else no content would be provided in parse");
        let version = parser
            .version(content)
            .map_err(|_| ConfigError::NoVersion("unstable"))?;

        let ver = match version.version.as_str() {
            "0.x" => parser.version0x(content),
            "unstable" => parser.version0y(content),
            _ => return Err(ConfigError::BadVersion(version.version, "unstable")),
        }
        .map_err(|parse_error| ConfigError::ParseError(version.version.clone(), parse_error))?;

        Ok(ver)
    }
}
