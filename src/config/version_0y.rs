use crate::ci::job::Job;
use crate::config::{ConfigLoader, ConfigPayload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type JobSet = std::collections::HashMap<String, Vec<String>>;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Constraints {
    blocks: Option<HashMap<String, Vec<String>>>,
    needs: Option<HashMap<String, Vec<String>>>,
}

fn from_vec(constraints: &[(String, String)]) -> Constraints {
    let mut map = HashMap::new();
    for (blocker, blocked) in constraints.iter().cloned() {
        map.entry(blocker).or_insert_with(Vec::new).push(blocked)
    }
    Constraints {
        blocks: Some(map),
        needs: Some(HashMap::new()),
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Verbose {
    All,
    Process,
    Failed,
    Result,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Spinner {
    frames: Vec<String>,
    per_frames: usize,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Display {
    ok: Option<String>,
    ko: Option<String>,
    cancelled: Option<String>,
    show_command: Option<bool>,
    display: Option<Vec<Verbose>>,
    spinner: Option<Spinner>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Version0y {
    version: String,
    jobs: JobSet,
    constraints: Option<Constraints>,
    display: Option<Display>,
}

impl ConfigLoader for Version0y {
    fn load(&self, payload: &mut ConfigPayload) {
        for (name, instruction) in &self.jobs {
            payload.ci.jobs.push(Job {
                name: name.clone(),
                instructions: instruction.clone(),
            })
        }

        if let Some(constraint) = &self.constraints {
            if let Some(blocks) = &constraint.blocks {
                for (blocker, blocked_jobs) in blocks {
                    for blocked in blocked_jobs {
                        payload
                            .ci
                            .constraints
                            .push((blocker.clone(), blocked.clone()))
                    }
                }
            }
            if let Some(needs) = &constraint.needs {
                for (blocked, blockers) in needs {
                    for blocker in blockers {
                        payload
                            .ci
                            .constraints
                            .push((blocker.clone(), blocked.clone()))
                    }
                }
            }
        }

        if let Some(icons) = &self.display {
            if let Some(ok) = &icons.ok {
                payload.ci.display.ok = ok.to_string()
            }
            if let Some(ko) = &icons.ko {
                payload.ci.display.ko = ko.to_string()
            }
            if let Some(cancelled) = &icons.cancelled {
                payload.ci.display.cancelled = cancelled.to_string()
            }
            if let Some(spinner) = &icons.spinner {
                payload.ci.display.spinner = (spinner.frames.clone(), spinner.per_frames)
            }
        }
    }
}

impl Version0y {
    pub fn from(payload: ConfigPayload) -> Self {
        Self {
            version: String::from("0.y"),
            jobs: payload
                .ci
                .jobs
                .iter()
                .cloned()
                .map(|job| (job.name, job.instructions))
                .collect(),
            constraints: Some(from_vec(&payload.ci.constraints)),
            display: Some(Display {
                ok: Some(payload.ci.display.ok.to_string()),
                ko: Some(payload.ci.display.ko.to_string()),
                cancelled: Some(payload.ci.display.cancelled.to_string()),
                show_command: Some(payload.ci.display.show_commands),
                display: None,
                spinner: Some(Spinner {
                    frames: payload.ci.display.spinner.0.clone(),
                    per_frames: payload.ci.display.spinner.1,
                }),
            }),
        }
    }
}
