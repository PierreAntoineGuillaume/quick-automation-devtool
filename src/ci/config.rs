use crate::ci::job::docker::Docker;
use crate::ci::job::simple::Simple;
use crate::ci::job::Type;

#[derive(Default, Clone)]
pub struct JobDesc {
    pub name: String,
    pub script: Vec<String>,
    pub image: Option<String>,
    pub group: Vec<String>,
    pub skip_if: Option<String>,
}

impl From<JobDesc> for Type {
    fn from(desc: JobDesc) -> Self {
        match desc.image {
            None => Self::Simple(Simple::long(
                desc.name,
                desc.script,
                desc.group.get(0).cloned(),
                desc.skip_if,
            )),
            Some(image) => Self::Docker(Docker::long(
                desc.name,
                desc.script,
                image,
                desc.group.get(0).cloned(),
                desc.skip_if,
            )),
        }
    }
}

#[derive(Default, Clone)]
pub struct CliOption {
    pub job: Option<String>,
}

#[derive(Default, Clone)]
pub struct Config {
    pub jobs: Vec<JobDesc>,
    pub groups: Vec<String>,
    pub constraints: Vec<(String, String)>,
}
