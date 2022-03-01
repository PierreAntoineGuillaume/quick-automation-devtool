pub(crate) mod argh;
pub mod migrate;
mod toml_parser;
pub mod version_0x;
pub mod version_0y;
mod yaml_parser;

use crate::ci::CiConfig;
use crate::config::toml_parser::TomlParser;
use crate::config::yaml_parser::YamlParser;
use serde::Deserialize;
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

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version {
    pub version: String,
}

pub struct Config {
    possible_files: Vec<String>,
    options: OptionConfigPayload,
}

#[derive(Default)]
pub struct ConfigPayload {
    pub ci: CiConfig,
    pub quiet: bool,
}

#[derive(Default)]
pub struct OptionConfigPayload {
    pub display: Option<()>,
}

impl OptionConfigPayload {
    pub fn load_into(&self, config: &mut ConfigPayload) {
        if self.display.is_some() {
            config.quiet = true;
        }
    }
}

pub trait ConfigLoader {
    fn load(&self, payload: &mut ConfigPayload);
}

pub trait FormatParser {
    fn supports(&self, filename: &str) -> bool;
    fn version(&self, text: &str) -> Result<Version, ()>;
    fn version0x(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String>;
    fn version0y(&self, text: &str) -> Result<Box<dyn ConfigLoader>, String>;
}

impl Config {
    fn get_first_available_config_file(&self) -> Result<String, String> {
        let mut filename = None;
        for file in &self.possible_files {
            if Path::new(file).exists() {
                filename = Some(file.clone());
                break;
            }
        }

        filename.ok_or_else(|| String::from("no config file could be found (looked for {:?})"))
    }

    pub fn from(options: OptionConfigPayload, env: &str) -> Self {
        let possible_files: Vec<String> =
            ["toml", "yaml", "yml", "toml.dist", "yaml.dist", "yml.dist"]
                .iter()
                .map(|str| format!("{}.{}", env, str))
                .collect();

        Config {
            possible_files,
            options,
        }
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
        self.options.load_into(config);
        Ok(())
    }

    pub fn parse(
        &self,
        filename: &str,
        content: &str,
    ) -> Result<Box<dyn ConfigLoader>, ConfigError> {
        let parsers = [YamlParser::boxed(), TomlParser::boxed()];

        for parser in &parsers {
            if !parser.supports(filename) {
                continue;
            }
            let version = parser
                .version(content)
                .map_err(|_| ConfigError::NoVersion("0.y"))?;

            let ver = match version.version.as_str() {
                "0.x" => parser.version0x(content),
                "0.y" => parser.version0y(content),
                _ => return Err(ConfigError::BadVersion(version.version, "0.y")),
            }
            .map_err(|parse_error| ConfigError::ParseError(version.version.clone(), parse_error))?;

            return Ok(ver);
        }
        unreachable!("This could not be reached, else no content would be provided in parse");
    }
}
