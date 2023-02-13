#[derive(Default, Debug)]
pub struct Bag {
    _env: Option<String>,
    _volume: Option<String>,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum ContainerConfiguration {
    None,
    _Docker {
        name: String,
        env: Vec<String>,
        volumes: Vec<String>,
        user: String,
        workdir: String,
    },
}

const _DOCKER_RUN: &str =
    r#"docker run --rm --user "$USER_ID:$GROUP_ID" --volume "$PWD:$PWD" --workdir "$PWD""#;

impl ContainerConfiguration {
    pub fn make(&self, instruction: &str, _bag: &mut Bag) -> String {
        match self {
            ContainerConfiguration::None => instruction.to_string(),
            ContainerConfiguration::_Docker { .. } => String::new(),
        }
    }
}

pub fn _env(env: &[String]) -> String {
    env.iter()
        .map(|key| format!(r#"--env "{key}=${key}""#))
        .collect::<Vec<String>>()
        .join(" ")
}

impl Default for ContainerConfiguration {
    fn default() -> Self {
        Self::None
    }
}

impl ContainerConfiguration {}
