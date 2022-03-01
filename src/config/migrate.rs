use crate::config::version_0x::Version0x;
use crate::config::version_0y::Version0y;
use crate::{Config, ConfigPayload};

pub struct Migrate {
    config: Config,
}

pub enum Migration {
    Version0x(Version0x),
    Version0y(Version0y),
}

impl Migrate {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn to0x(&self) -> Result<Migration, String> {
        let mut payload = ConfigPayload::default();
        self.config.load_into(&mut payload)?;
        Ok(Migration::Version0x(Version0x::from(payload)))
    }

    pub fn to0y(&self) -> Result<Migration, String> {
        let mut payload = ConfigPayload::default();
        self.config.load_into(&mut payload)?;
        Ok(Migration::Version0y(Version0y::from(payload)))
    }
}
