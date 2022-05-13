use crate::ci::config::JobDesc;
use crate::config::{Loader, Payload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type JobSet = HashMap<String, Vec<String>>;
pub type Constraints = HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CiSpinner {
    frames: Vec<String>,
    per_frames: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct CiIcons {
    ok: Option<String>,
    ko: Option<String>,
    cancelled: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Version0x {
    version: String,
    jobs: JobSet,
    constraints: Option<Constraints>,
    ci_spinner: Option<CiSpinner>,
    ci_icons: Option<CiIcons>,
}

impl Loader for Version0x {
    fn load(&self, payload: &mut Payload) {
        let ci_config = &mut payload.ci;

        for (name, instruction) in self.jobs.clone() {
            ci_config.jobs.push(JobDesc {
                name,
                script: instruction,
                image: None,
                group: None,
                skip_if: None,
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
            payload.display.spinner = (spinner.frames.clone(), spinner.per_frames)
        }

        if let Some(icons) = &self.ci_icons {
            if let Some(ok) = &icons.ok {
                payload.display.ok = ok.clone()
            }
            if let Some(ko) = &icons.ko {
                payload.display.ko = ko.clone()
            }
            if let Some(cancelled) = &icons.cancelled {
                payload.display.cancelled = cancelled.clone()
            }
        }
    }
}
