use crate::ci::job::{Job, JobIntrospector};
use crate::config::instructions::InstructionCompiler;
use crate::config::{ConfigLoader, ConfigPayload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FullJobDesc {
    script: Vec<String>,
    image: Option<String>,
}

pub type JobSet = std::collections::HashMap<String, FullJobDesc>;

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
        for (name, full_desc) in &self.jobs {
            let compiler = InstructionCompiler::default();
            let instructions = full_desc
                .script
                .iter()
                .map(|str| compiler.compile(str))
                .collect();
            let name = name.clone();
            payload.ci.jobs.push(Arc::from(Job::long(
                name,
                instructions,
                full_desc.image.clone(),
            )))
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

#[derive(Default)]
struct JobTraitConverter {
    data: Option<(String, FullJobDesc)>,
}

impl JobIntrospector for JobTraitConverter {
    fn basic_job(&mut self, name: &str, image: &Option<String>, instructions: &[String]) {
        self.data = Some((
            name.to_string(),
            FullJobDesc {
                image: image.clone(),
                script: instructions.to_vec(),
            },
        ))
    }
}

impl Version0y {
    pub fn from(payload: ConfigPayload) -> Self {
        let job_ref = payload.ci.jobs;
        let jobs = job_ref
            .iter()
            .map(|job| {
                let mut convertor = JobTraitConverter::default();
                job.introspect(&mut convertor);
                convertor.data.expect("Visitor has been set")
            })
            .collect();
        Self {
            version: String::from("unstable"),
            jobs,
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
