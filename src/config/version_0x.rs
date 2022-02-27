use crate::ci::job::Job;
use crate::ci::CiConfig;
use serde::Deserialize;

pub type JobSet = std::collections::HashMap<String, Vec<String>>;
pub type Constraints = std::collections::HashMap<String, Vec<String>>;

#[derive(Deserialize, Debug, PartialEq)]
struct CiSpinner {
    frames: Vec<String>,
    finished: String,
    blocked: String,
    per_frames: usize,
}

#[derive(Deserialize, Debug, PartialEq)]
struct CiIcons {
    ok: String,
    ko: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version0x {
    jobs: JobSet,
    constraints: Option<Constraints>,
    ci_spinner: Option<CiSpinner>,
    ci_icons: Option<CiIcons>,
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

        if let Some(spinner) = &self.ci_spinner {
            let mut vec = spinner.frames.clone();
            vec.push(spinner.finished.clone());
            vec.push(spinner.blocked.clone());
            ci_config.spinner = (vec, spinner.per_frames)
        }

        if let Some(icons) = &self.ci_icons {
            ci_config.icons.ok = icons.ok.clone();
            ci_config.icons.ko = icons.ko.clone();
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
                ci_spinner: None,
                ci_icons: None,
            }
        }
    }
}
