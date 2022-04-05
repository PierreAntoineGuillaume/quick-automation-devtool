use crate::config::versions::version_0x::Version0x;
use crate::config::versions::version_0y::Version0y;
use crate::{Config, ConfigPayload};
use anyhow::Result;

pub struct Migrate {
    config: Config,
}

#[derive(Debug)]
pub enum Migration {
    Version0x(Version0x),
    Version0y(Version0y),
}

impl Migrate {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn to0x(&self) -> Result<Migration> {
        let mut payload = ConfigPayload::default();
        self.config.load_into(&mut payload)?;
        Ok(Migration::Version0x(Version0x::from(payload)))
    }

    pub fn to0y(&self) -> Result<Migration> {
        let mut payload = ConfigPayload::default();
        self.config.load_into(&mut payload)?;
        Ok(Migration::Version0y(Version0y::from(payload)))
    }

    pub fn toml(&self, migration: Migration) -> String {
        let str = match migration {
            Migration::Version0x(version) => toml::to_string(&version),
            Migration::Version0y(version) => toml::to_string(&version),
        };
        if str.is_err() {
            eprintln!("dt serialization error: {}", str.unwrap_err());
            std::process::exit(1);
        }

        str.unwrap()
    }

    pub fn yaml(&self, migration: Migration) -> String {
        match migration {
            Migration::Version0x(version) => serde_yaml::to_string(&version).unwrap(),
            Migration::Version0y(version) => serde_yaml::to_string(&version).unwrap(),
        }
    }
}
