use crate::ci::ci_config::JobDesc;
use crate::ci::display::Mode;
use crate::ci::job::JobIntrospector;
use crate::config::{ConfigLoader, ConfigPayload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct FullJobDesc {
    script: Vec<String>,
    image: Option<String>,
    group: Option<String>,
    skip_if: Option<String>,
}

pub type JobSet = HashMap<String, FullJobDesc>;

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
struct Spinner {
    frames: Vec<String>,
    per_frames: usize,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum DisplayMode {
    silent,
    sequence,
    summary,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Display {
    mode: Option<DisplayMode>,
    ok: Option<String>,
    ko: Option<String>,
    cancelled: Option<String>,
    spinner: Option<Spinner>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Version1 {
    version: String,
    jobs: JobSet,
    groups: Option<Vec<String>>,
    constraints: Option<Constraints>,
    display: Option<Display>,
    env: Option<String>,
}

impl ConfigLoader for Version1 {
    fn load(&self, payload: &mut ConfigPayload) {
        for (name, full_desc) in self.jobs.clone() {
            payload.ci.jobs.push(JobDesc {
                name,
                script: full_desc.script,
                image: full_desc.image,
                group: full_desc.group,
                skip_if: full_desc.skip_if,
            })
        }
        if let Some(groups) = &self.groups {
            payload.ci.groups = groups.clone();
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

        if let Some(display) = &self.display {
            if let Some(ok) = &display.ok {
                payload.display.ok = ok.to_string()
            }
            if let Some(ko) = &display.ko {
                payload.display.ko = ko.to_string()
            }
            if let Some(cancelled) = &display.cancelled {
                payload.display.cancelled = cancelled.to_string()
            }
            if let Some(spinner) = &display.spinner {
                payload.display.spinner = (spinner.frames.clone(), spinner.per_frames)
            }
            if let Some(mode) = &display.mode {
                match mode {
                    DisplayMode::silent => payload.display.mode = Mode::Silent,
                    DisplayMode::sequence => payload.display.mode = Mode::AllOutput,
                    DisplayMode::summary => payload.display.mode = Mode::Summary,
                }
            }
        }

        payload.env = self.env.clone();
    }
}

#[derive(Default)]
struct VersionYJobConverter {
    data: Option<(String, FullJobDesc)>,
}

impl JobIntrospector for VersionYJobConverter {
    fn basic_job(
        &mut self,
        name: &str,
        group: &Option<String>,
        instructions: &[String],
        skip_if: &Option<String>,
    ) {
        self.data = Some((
            name.to_string(),
            FullJobDesc {
                image: None,
                group: group.clone(),
                script: instructions.to_vec(),
                skip_if: skip_if.clone(),
            },
        ))
    }

    fn docker_job(
        &mut self,
        name: &str,
        image: &str,
        group: &Option<String>,
        instructions: &[String],
        skip_if: &Option<String>,
    ) {
        self.data = Some((
            name.to_string(),
            FullJobDesc {
                image: Some(image.to_string()),
                group: group.clone(),
                script: instructions.to_vec(),
                skip_if: skip_if.clone(),
            },
        ))
    }
}

impl Version1 {
    pub fn from(payload: ConfigPayload) -> Self {
        let job_ref = payload.ci.jobs;
        let jobs = job_ref
            .iter()
            .cloned()
            .map(|job| {
                (
                    job.name,
                    FullJobDesc {
                        script: job.script,
                        image: job.image,
                        group: job.group,
                        skip_if: job.skip_if,
                    },
                )
            })
            .collect();
        Self {
            version: String::from("1"),
            jobs,
            constraints: Some(from_vec(&payload.ci.constraints)),
            groups: Some(payload.ci.groups.clone()),
            display: Some(Display {
                mode: Some(match payload.display.mode {
                    Mode::Silent => DisplayMode::silent,
                    Mode::AllOutput => DisplayMode::sequence,
                    Mode::Summary => DisplayMode::summary,
                }),
                ok: Some(payload.display.ok.to_string()),
                ko: Some(payload.display.ko.to_string()),
                cancelled: Some(payload.display.cancelled.to_string()),
                spinner: Some(Spinner {
                    frames: payload.display.spinner.0.clone(),
                    per_frames: payload.display.spinner.1,
                }),
            }),
            env: payload.env,
        }
    }
}
