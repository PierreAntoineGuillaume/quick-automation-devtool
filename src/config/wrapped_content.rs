use super::version_0x::Version0x;
use crate::ci::CiConfig;

#[derive(Debug, PartialEq)]
pub enum WrappedContent {
    V0x(Version0x),
}

impl WrappedContent {
    pub fn load_into(&self, pipeline: &mut CiConfig) {
        match self {
            WrappedContent::V0x(v) => v.load_into(pipeline),
        }
    }
}
