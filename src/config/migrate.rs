use crate::config::versions::version_0y::Version0y;
use crate::{Config, ConfigPayload};
use anyhow::Result;

pub struct Migrate {
    config: Config,
}

#[derive(Debug)]
pub enum Migration {
    Version0y(Version0y),
}

impl Migrate {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn to0y(&self) -> Result<Migration> {
        let mut payload = ConfigPayload::default();
        self.config.load_into(&mut payload)?;
        Ok(Migration::Version0y(Version0y::from(payload)))
    }

    pub fn yaml(&self, migration: Migration) -> String {
        match migration {
            Migration::Version0y(version) => serde_yaml::to_string(&version).unwrap(),
        }
    }
}
