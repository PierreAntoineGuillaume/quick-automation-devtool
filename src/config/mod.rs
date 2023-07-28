pub mod argh;
pub mod migrate;
mod versions;
pub mod yaml_parser;

use crate::ci::config::Config as CiConfig;
use crate::ci::display::CiDisplayConfig;
use anyhow::Error as AnyError;
use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
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
                format!("{filename} could not be parsed: {error}")
            }
            Error::NoVersion(latest, previous) => {
                format!(
                    "{filename} could not parse version id (latest is {latest})\nbecause: {previous}"
                )
            }
            Error::BadVersion(version, latest) => {
                format!("unknown version {version} in {filename} (latest is {latest})",)
            }
            Error::ParseError(version, prev) => {
                format!("could not parse {filename} with version {version} ({prev})",)
            }
        }
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Version {
    pub version: String,
}

pub struct Config {
    candidates: Vec<String>,
}

#[derive(Default)]
pub struct Payload {
    pub ci: CiConfig,
    pub display: CiDisplayConfig,
    pub env: Option<String>,
    pub extra_files: Vec<String>,
}

impl Payload {
    pub fn absorb(&mut self, other: &Payload) {
        for new_job in &other.ci.jobs {
            if !self.ci.jobs.contains(new_job) {
                self.ci.jobs.push(new_job.clone());
            }
        }
    }
}

pub trait Loader {
    fn load(&self, payload: &mut Payload);
}

pub trait FormatParser {
    fn version(&self, text: &str) -> Result<Version, String>;
    fn version1(&self, text: &str) -> Result<Box<dyn Loader>, String>;
    fn latest_with_warning(
        &self,
        text: &str,
        requested_version: &str,
    ) -> Result<Box<dyn Loader>, String>;
}

pub const LATEST: &str = "1.0";

impl Config {
    pub fn get_first_available_config_file(&self) -> Result<String> {
        let mut filename = None;
        for file in &self.candidates {
            if Path::new(file).exists() {
                filename = Some(file.clone());
                break;
            }
        }

        filename.ok_or_else(|| {
            AnyError::msg(format!(
                "no config file could be found (looked in files {:?})",
                self.candidates
            ))
        })
    }

    pub fn from(env: &str) -> Self {
        let candidates: Vec<String> = ["yaml", "yml", "yaml.dist", "yml.dist"]
            .iter()
            .map(|str| format!("{env}.{str}"))
            .collect();

        Self { candidates }
    }

    pub fn from_name(env: &str) -> Self {
        Self {
            candidates: vec![env.to_string()],
        }
    }

    pub fn load_into(&self, config: &mut Payload) -> Result<()> {
        let filename = self.get_first_available_config_file()?;

        Self::load_unknown_file(config, &filename)?;

        for file in config.extra_files.clone() {
            let mut other = Payload::default();
            Self::load_unknown_file(&mut other, &file)?;
            config.absorb(&other);
        }

        Ok(())
    }

    fn load_unknown_file(config: &mut Payload, filename: &str) -> Result<()> {
        let content = fs::read_to_string(filename)
            .map_err(|error| AnyError::msg(Error::Parse(error.to_string()).explain(filename)))?;

        let loader =
            Self::parse(&content).map_err(|error| AnyError::msg(error.explain(filename)))?;
        loader.load(config);

        Ok(())
    }

    pub fn load_with_args_into(&self, config: &mut Payload) -> Result<()> {
        self.load_into(config)?;
        Ok(())
    }

    pub fn parse(content: &str) -> Result<Box<dyn Loader>, Error> {
        let parser = YamlParser {};
        let version = parser
            .version(content)
            .map_err(|why| Error::NoVersion(LATEST, why))?;

        let regex = Regex::new(r"^(\d+)(?:\.(\S+))?$").unwrap();
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
            ("1", None | Some("0")) => parser.version1(content),
            ("1", Some(_)) => parser.latest_with_warning(content, version.version.as_str()),
            _ => return Err(Error::BadVersion(version.version, LATEST)),
        }
        .map_err(|parse_error| Error::ParseError(version.version.clone(), parse_error))?;

        Ok(ver)
    }
}
