use crate::ci::job::Job;
use crate::Pipeline;
use serde::Deserialize;

pub type JobSet = std::collections::HashMap<String, std::vec::Vec<String>>;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version0x {
    jobs: JobSet,
}

impl Version0x {
    pub fn load_into(&self, pipeline: &mut Pipeline) {
        for (name, instruction) in &self.jobs {
            pipeline.push_job(Job {
                name: name.clone(),
                instructions: instruction.clone(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl Version0x {
        pub fn new(set: JobSet) -> Self {
            Version0x { jobs: set }
        }
    }
}
