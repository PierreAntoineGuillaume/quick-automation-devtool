use crate::ci::job::container_configuration::{ContainerConfiguration, DockerContainer};
use crate::ci::job::Job;

#[derive(Default, Clone)]
pub struct JobDesc {
    pub name: String,
    pub script: Vec<String>,
    pub image: Option<String>,
    pub group: Vec<String>,
    pub skip_if: Option<String>,
}

impl From<JobDesc> for Job {
    fn from(desc: JobDesc) -> Self {
        match desc.image {
            None => Job::long(
                desc.name,
                desc.script,
                desc.group.get(0).cloned(),
                desc.skip_if,
            ),
            Some(image) => Job::new(
                desc.name,
                desc.script,
                ContainerConfiguration::Container(DockerContainer::new(
                    &image,
                    &"$USER_ID:$GROUP_ID",
                    &"$PWD",
                    &[&"$PWD:$PWD:rw"],
                )),
                desc.group.get(0).cloned(),
                desc.skip_if,
            ),
        }
    }
}

impl PartialEq<Self> for JobDesc {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Default, Clone)]
pub struct CliOption {
    pub job: Option<String>,
    pub no_tty: bool,
}

#[derive(Default, Clone)]
pub struct Config {
    pub jobs: Vec<JobDesc>,
    pub groups: Vec<String>,
    pub constraints: Vec<(String, String)>,
}
