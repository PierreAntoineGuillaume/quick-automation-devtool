pub mod argh;
pub mod migrate;
mod versions;
pub mod yaml_parser;

use crate::ci::config::CiConfig;
use crate::ci::display::CiDisplayConfig;
use anyhow::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use yaml_parser::YamlParser;

#[derive(Debug)]
pub enum ConfigError {
    FileError(String),
    NoVersion(&'static str, String),
    BadVersion(String, &'static str),
    ParseError(String, String),
}

impl ConfigError {
    fn explain(&self, filename: &str) -> String {
        match self {
            ConfigError::FileError(error) => {
                format!("{} could not be parsed: {}", filename, error,)
            }
            ConfigError::NoVersion(latest, previous) => {
                format!(
                    "{} could not parse version id (latest is {})\nbecause: {}",
                    filename, latest, previous,
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
    pub display: CiDisplayConfig,
    pub env: Option<String>,
}

pub trait ConfigLoader {
    fn load(&self, payload: &mut ConfigPayload);
}

pub trait FormatParser {
    fn supports(&self, filename: &str) -> bool;
    fn version(&self, text: &str) -> Result<Version, String>;
    fn version0x(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String>;
    fn version1(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String>;
    fn latest_with_warning(
        &self,
        text: &str,
        requested_version: &str,
    ) -> Result<Box<dyn ConfigLoader>, String>;
    fn format(&self) -> Format;
}

pub const LATEST: &str = "1.0";

impl Config {
    pub fn get_first_available_config_file(&self) -> Result<String> {
        let mut filename = None;
        for file in &self.possible_files {
            if Path::new(file).exists() {
                filename = Some(file.clone());
                break;
            }
        }

        filename.ok_or_else(|| {
            Error::msg(format!(
                "no config file could be found (looked in files {:?})",
                self.possible_files
            ))
        })
    }

    pub fn from(env: &str) -> Self {
        let possible_files: Vec<String> = ["yaml", "yml", "yaml.dist", "yml.dist"]
            .iter()
            .map(|str| format!("{}.{}", env, str))
            .collect();

        Config { possible_files }
    }

    pub fn load_into(&self, config: &mut ConfigPayload) -> Result<()> {
        let filename = self.get_first_available_config_file()?;

        let content = fs::read_to_string(&filename).map_err(|error| {
            Error::msg(ConfigError::FileError(error.to_string()).explain(&filename))
        })?;

        let loader = self
            .parse(&filename, &content)
            .map_err(|error| Error::msg(error.explain(&filename)))?;
        loader.load(config);
        Ok(())
    }

    pub fn load_with_args_into(&self, config: &mut ConfigPayload) -> Result<()> {
        self.load_into(config)?;
        Ok(())
    }

    pub fn get_parser(&self, filename: &str) -> Option<Box<dyn FormatParser>> {
        let parsers = [YamlParser::boxed()];
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
            .map_err(|why| ConfigError::NoVersion(LATEST, why))?;

        let regex = Regex::new(r#"^(\d+)(?:\.(\S+))?$"#).unwrap();
        let version_numbers = regex.captures(version.version.as_str());

        if version_numbers.is_none() {
            return Err(ConfigError::BadVersion(version.version, LATEST));
        }

        let version_numbers = version_numbers.unwrap();

        let ver = match (
            version_numbers
                .get(1)
                .expect("Filtered Previously")
                .as_str(),
            version_numbers.get(2).map(|item| item.as_str()),
        ) {
            ("0", _) => parser.version0x(content),
            ("1", None | Some("0")) => parser.version1(content),
            ("1", Some(_)) => parser.latest_with_warning(content, version.version.as_str()),
            _ => return Err(ConfigError::BadVersion(version.version, LATEST)),
        }
        .map_err(|parse_error| ConfigError::ParseError(version.version.clone(), parse_error))?;

        Ok(ver)
    }
}
