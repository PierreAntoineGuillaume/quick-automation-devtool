use crate::ci::job::docker_job::DockerJob;
use crate::ci::job::simple_job::SimpleJob;
use crate::ci::job::JobType;

#[derive(Default, Clone)]
pub struct JobDesc {
    pub name: String,
    pub script: Vec<String>,
    pub image: Option<String>,
    pub group: Option<String>,
    pub skip_if: Option<String>,
}

impl From<JobDesc> for JobType {
    fn from(desc: JobDesc) -> Self {
        match desc.image {
            None => JobType::Simple(SimpleJob::long(
                desc.name,
                desc.script,
                desc.group,
                desc.skip_if,
            )),
            Some(image) => JobType::Docker(DockerJob::long(
                desc.name,
                desc.script,
                image,
                desc.group,
                desc.skip_if,
            )),
        }
    }
}

#[derive(Default, Clone)]
pub struct CliConfig {
    pub job: Option<String>,
}

#[derive(Default, Clone)]
pub struct CiConfig {
    pub jobs: Vec<JobDesc>,
    pub groups: Vec<String>,
    pub constraints: Vec<(String, String)>,
}
