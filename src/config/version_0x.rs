use crate::ci::job::Job;
use crate::config::{ConfigLoader, ConfigPayload};
use serde::Deserialize;

pub type JobSet = std::collections::HashMap<String, Vec<String>>;
pub type Constraints = std::collections::HashMap<String, Vec<String>>;

#[derive(Deserialize, Debug, PartialEq)]
struct CiSpinner {
    frames: Vec<String>,
    per_frames: usize,
}

#[derive(Deserialize, Debug, PartialEq)]
struct CiIcons {
    ok: Option<String>,
    ko: Option<String>,
    cancelled: Option<String>,
    display_commands: Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Version0x {
    jobs: JobSet,
    constraints: Option<Constraints>,
    ci_spinner: Option<CiSpinner>,
    ci_icons: Option<CiIcons>,
}

impl ConfigLoader for Version0x {
    fn load(&self, payload: &mut ConfigPayload) {
        let mut ci_config = &mut payload.ci_config;
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
            ci_config.spinner = (spinner.frames.clone(), spinner.per_frames)
        }

        if let Some(icons) = &self.ci_icons {
            if let Some(ok) = &icons.ok {
                ci_config.icons.ok = ok.clone()
            }
            if let Some(ko) = &icons.ko {
                ci_config.icons.ko = ko.clone()
            }
            if let Some(cancelled) = &icons.cancelled {
                ci_config.icons.cancelled = cancelled.clone()
            }
            if let Some(display_commands) = &icons.display_commands {
                ci_config.icons.display_command = *display_commands
            }
        }
    }
}
