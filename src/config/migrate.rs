use crate::config::versions::version_1::Version1;
use crate::{Config, Payload};
use anyhow::Result;

pub struct Migrate {
    config: Config,
}

#[derive(Debug)]
pub enum Migration {
    Version1(Version1),
}

impl Migrate {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn to0y(&self) -> Result<Migration> {
        let mut payload = Payload::default();
        self.config.load_into(&mut payload)?;
        Ok(Migration::Version1(Version1::from(payload)))
    }

    pub fn yaml(migration: Migration) -> String {
        match migration {
            Migration::Version1(version) => serde_yaml::to_string(&version).unwrap(),
        }
    }
}
