use crate::ci::job::{Job, JobIntrospector};
use crate::config::{ConfigLoader, ConfigPayload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub type JobSet = HashMap<String, Vec<String>>;
pub type Constraints = HashMap<String, Vec<String>>;

fn from_vec(constraints: &[(String, String)]) -> Constraints {
    let mut map = Constraints::default();
    for (blocker, blocked) in constraints.iter().cloned() {
        map.entry(blocker).or_insert_with(Vec::new).push(blocked)
    }
    map
}

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
    display_commands: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Version0x {
    version: String,
    jobs: JobSet,
    constraints: Option<Constraints>,
    ci_spinner: Option<CiSpinner>,
    ci_icons: Option<CiIcons>,
}

impl ConfigLoader for Version0x {
    fn load(&self, payload: &mut ConfigPayload) {
        let mut ci_config = &mut payload.ci;
        for (name, instruction) in &self.jobs {
            ci_config
                .jobs
                .push(Arc::from(Job::short(name.clone(), instruction.clone())))
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
            ci_config.display.spinner = (spinner.frames.clone(), spinner.per_frames)
        }

        if let Some(icons) = &self.ci_icons {
            if let Some(ok) = &icons.ok {
                ci_config.display.ok = ok.clone()
            }
            if let Some(ko) = &icons.ko {
                ci_config.display.ko = ko.clone()
            }
            if let Some(cancelled) = &icons.cancelled {
                ci_config.display.cancelled = cancelled.clone()
            }
            if let Some(display_commands) = &icons.display_commands {
                ci_config.display.show_commands = *display_commands
            }
        }
    }
}

#[derive(Default)]
struct VersionXJobConverter {
    data: Option<(String, Vec<String>)>,
}

impl JobIntrospector for VersionXJobConverter {
    fn basic_job(
        &mut self,
        name: &str,
        _: &Option<String>,
        _: &Option<String>,
        instructions: &[String],
    ) {
        self.data = Some((name.to_string(), instructions.to_vec()))
    }
}

impl Version0x {
    pub fn from(payload: ConfigPayload) -> Self {
        Self {
            version: String::from("0.x"),
            jobs: payload
                .ci
                .jobs
                .iter()
                .map(|job| {
                    let mut converter = VersionXJobConverter::default();
                    job.introspect(&mut converter);
                    converter.data.expect("Visitor has been set")
                })
                .collect(),
            constraints: Some(from_vec(&payload.ci.constraints)),
            ci_spinner: Some(CiSpinner {
                frames: payload.ci.display.spinner.0.clone(),
                per_frames: payload.ci.display.spinner.1,
            }),
            ci_icons: Some(CiIcons {
                ok: Some(payload.ci.display.ok.clone()),
                ko: Some(payload.ci.display.ko.clone()),
                cancelled: Some(payload.ci.display.cancelled.clone()),
                display_commands: Some(payload.ci.display.show_commands),
            }),
        }
    }
}
