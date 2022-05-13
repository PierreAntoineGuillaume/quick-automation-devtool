pub mod argh;
pub mod migrate;
mod versions;
pub mod yaml_parser;

use crate::ci::config::Config as CiConfig;
use crate::ci::display::CiDisplayConfig;
use anyhow::Error as AnyError;
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use yaml_parser::YamlParser;

#[derive(Debug)]
pub enum Error {
    Parse(String),
    NoVersion(&'static str, String),
    BadVersion(String, &'static str),
    ParseError(String, String),
}

impl Error {
    fn explain(&self, filename: &str) -> String {
        match self {
            Error::Parse(error) => {
                format!("{} could not be parsed: {}", filename, error,)
            }
            Error::NoVersion(latest, previous) => {
                format!(
                    "{} could not parse version id (latest is {})\nbecause: {}",
                    filename, latest, previous,
                )
            }
            Error::BadVersion(version, latest) => {
                format!(
                    "unknown version {} in {} (latest is {})",
                    version, filename, latest
                )
            }
            Error::ParseError(version, prev) => {
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
pub struct Payload {
    pub ci: CiConfig,
    pub display: CiDisplayConfig,
    pub env: Option<String>,
}

pub trait Loader {
    fn load(&self, payload: &mut Payload);
}

pub trait FormatParser {
    fn supports(&self, filename: &str) -> bool;
    fn version(&self, text: &str) -> Result<Version, String>;
    fn version0x(&self, text: &str) -> Result<Box<dyn Loader>, String>;
    fn version1(&self, text: &str) -> Result<Box<dyn Loader>, String>;
    fn latest_with_warning(
        &self,
        text: &str,
        requested_version: &str,
    ) -> Result<Box<dyn Loader>, String>;
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
            AnyError::msg(format!(
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

    pub fn load_into(&self, config: &mut Payload) -> Result<()> {
        let filename = self.get_first_available_config_file()?;

        let content = fs::read_to_string(&filename)
            .map_err(|error| AnyError::msg(Error::Parse(error.to_string()).explain(&filename)))?;

        let loader = Self::parse(&filename, &content)
            .map_err(|error| AnyError::msg(error.explain(&filename)))?;
        loader.load(config);
        Ok(())
    }

    pub fn load_with_args_into(&self, config: &mut Payload) -> Result<()> {
        self.load_into(config)?;
        Ok(())
    }

    pub fn get_parser(filename: &str) -> Option<Box<dyn FormatParser>> {
        let parsers = [YamlParser::boxed()];
        for parser in parsers {
            if !parser.supports(filename) {
                continue;
            }
            return Some(parser);
        }
        None
    }

    pub fn parse(filename: &str, content: &str) -> Result<Box<dyn Loader>, Error> {
        let parser = Self::get_parser(filename)
            .expect("This could not be reached, else no content would be provided in parse");
        let version = parser
            .version(content)
            .map_err(|why| Error::NoVersion(LATEST, why))?;

        let regex = Regex::new(r#"^(\d+)(?:\.(\S+))?$"#).unwrap();
        let version_numbers = regex.captures(version.version.as_str());

        if version_numbers.is_none() {
            return Err(Error::BadVersion(version.version, LATEST));
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
            _ => return Err(Error::BadVersion(version.version, LATEST)),
        }
        .map_err(|parse_error| Error::ParseError(version.version.clone(), parse_error))?;

        Ok(ver)
    }
}
