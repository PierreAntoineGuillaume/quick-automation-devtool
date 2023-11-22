use crate::ci::config::JobDesc;
use crate::ci::display::FinalDisplayMode;
use crate::ci::display::Running as RunningDisplay;
use crate::config::{Loader, Payload};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct FullJobDesc {
    script: Vec<String>,
    image: Option<String>,
    group: Option<String>,
    skip_if: Option<String>,
}

pub type JobSet = HashMap<String, FullJobDesc>;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Constraints {
    blocks: Option<HashMap<String, Vec<String>>>,
    needs: Option<HashMap<String, Vec<String>>>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Spinner {
    frames: Vec<String>,
    per_frames: usize,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Silent,
    Sequence,
    Summary,
}

impl From<DisplayMode> for RunningDisplay {
    fn from(mode: DisplayMode) -> Self {
        match mode {
            DisplayMode::Silent => Self::Silent,
            DisplayMode::Sequence => Self::Sequence,
            DisplayMode::Summary => Self::Summary,
        }
    }
}

impl From<RunningDisplay> for DisplayMode {
    fn from(mode: RunningDisplay) -> Self {
        match mode {
            RunningDisplay::Silent => Self::Silent,
            RunningDisplay::Sequence => Self::Sequence,
            RunningDisplay::Summary => Self::Summary,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FinalDisplay {
    Full,
    Silent,
    Interactive,
}

impl From<FinalDisplayMode> for FinalDisplay {
    fn from(mode: FinalDisplayMode) -> Self {
        match mode {
            FinalDisplayMode::Full => Self::Full,
            FinalDisplayMode::Silent => Self::Silent,
            FinalDisplayMode::Interactive => Self::Interactive,
        }
    }
}
impl From<FinalDisplay> for FinalDisplayMode {
    fn from(mode: FinalDisplay) -> Self {
        match mode {
            FinalDisplay::Full => Self::Full,
            FinalDisplay::Silent => Self::Silent,
            FinalDisplay::Interactive => Self::Interactive,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
struct Display {
    mode: Option<DisplayMode>,
    #[serde(rename = "final")]
    final_item: Option<FinalDisplay>,
    ok: Option<String>,
    ko: Option<String>,
    cancelled: Option<String>,
    spinner: Option<Spinner>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Version1 {
    version: String,
    jobs: JobSet,
    groups: Option<Vec<String>>,
    constraints: Option<Constraints>,
    display: Option<Display>,
    env: Option<String>,
    extra_files: Option<Vec<String>>,
}

impl Loader for Version1 {
    fn load(&self, payload: &mut Payload) {
        for (name, full_desc) in self.jobs.clone() {
            payload.ci.jobs.push(JobDesc {
                name,
                script: full_desc.script,
                image: full_desc.image,
                group: full_desc.group.iter().cloned().collect::<Vec<String>>(),
                skip_if: full_desc.skip_if,
            });
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
                            .push((blocker.clone(), blocked.clone()));
                    }
                }
            }
            if let Some(needs) = &constraint.needs {
                for (blocked, blockers) in needs {
                    for blocker in blockers {
                        payload
                            .ci
                            .constraints
                            .push((blocker.clone(), blocked.clone()));
                    }
                }
            }
        }

        if let Some(display) = &self.display {
            if let Some(ok) = &display.ok {
                payload.display.ok = ok.to_string();
            }
            if let Some(ko) = &display.ko {
                payload.display.ko = ko.to_string();
            }
            if let Some(cancelled) = &display.cancelled {
                payload.display.cancelled = cancelled.to_string();
            }
            if let Some(spinner) = &display.spinner {
                payload.display.spinner = (spinner.frames.clone(), spinner.per_frames);
            }
            if let Some(mode) = &display.mode {
                payload.display.running_display = RunningDisplay::from(*mode);
            }
            if let Some(final_display) = &display.final_item {
                payload.display.final_display = FinalDisplayMode::from(*final_display);
            }
        }

        if let Some(files) = &self.extra_files {
            payload.extra_files = files.clone();
        }

        payload.env = self.env.clone();
    }
}
