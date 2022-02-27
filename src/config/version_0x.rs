use crate::ci::job::Job;
use crate::ci::CiConfig;
use serde::Deserialize;

pub type JobSet = std::collections::HashMap<String, Vec<String>>;
pub type Constraints = std::collections::HashMap<String, Vec<String>>;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version0x {
    jobs: JobSet,
    constraints: Option<Constraints>,
}

impl Version0x {
    pub fn load_into(&self, ci_config: &mut CiConfig) {
        for (name, instruction) in &self.jobs {
            ci_config.jobs.push(Job {
                name: name.clone(),
                instructions: instruction.clone(),
            })
        }
        if let Some(constraint) = &self.constraints {
            for (blocker, blocked_jobs) in constraint {
                for blocked in blocked_jobs {
                    ci_config
                        .constraints
                        .push((blocker.clone(), blocked.clone()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl Version0x {
        pub fn new(set: JobSet) -> Self {
            Version0x {
                jobs: set,
                constraints: None,
            }
        }
    }
}
