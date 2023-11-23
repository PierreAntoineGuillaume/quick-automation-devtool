use crate::ci::config::JobDesc;
use crate::ci::display::FinalDisplayMode;
use crate::ci::display::Running as RunningDisplay;
use crate::ci::job::container_configuration::DockerContainer;
use crate::config::{Loader, Payload};
use serde::de::{MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
struct ContainerReference {
    image: String,
    env: Vec<String>,
    volumes: Vec<String>,
    user: String,
    workdir: String,
}

impl FromStr for ContainerReference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            image: s.to_string(),
            env: Default::default(),
            volumes: vec!["$PWD:$PWD:rw".to_string()],
            user: "$USER_ID:$GROUP_ID".to_string(),
            workdir: "$PWD".to_string(),
        })
    }
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = ()>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = ()>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
struct ContainerWrapper(#[serde(deserialize_with = "string_or_struct")] ContainerReference);

#[derive(Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct FullJobDesc {
    script: Vec<String>,
    #[serde(rename = "image")]
    container_reference: Option<ContainerWrapper>,
    group: Option<String>,
    skip_if: Option<String>,
}

pub type JobSet = HashMap<String, FullJobDesc>;

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Constraints {
    blocks: Option<HashMap<String, Vec<String>>>,
    needs: Option<HashMap<String, Vec<String>>>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Spinner {
    frames: Vec<String>,
    per_frames: usize,
}

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Display {
    mode: Option<DisplayMode>,
    #[serde(rename = "final")]
    final_item: Option<FinalDisplay>,
    ok: Option<String>,
    ko: Option<String>,
    cancelled: Option<String>,
    spinner: Option<Spinner>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
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
            let image =
                full_desc
                    .container_reference
                    .map(|container_reference: ContainerWrapper| {
                        DockerContainer::new(
                            &container_reference.0.image,
                            &container_reference.0.user,
                            &container_reference.0.workdir,
                            &container_reference.0.volumes,
                            &container_reference.0.env,
                        )
                    });
            payload.ci.jobs.push(JobDesc {
                name,
                script: full_desc.script,
                image,
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
