#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct _DockerContainer {
    image: String,
    env: Vec<String>,
    volumes: Vec<String>,
    user: String,
    workdir: String,
}

impl _DockerContainer {
    fn env(&self) -> String {
        self.env
            .iter()
            .map(|key| format!(r#"--env "{key}""#))
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn volumes(&self) -> String {
        self.volumes
            .iter()
            .map(|name| format!(r#"--volume "{name}""#))
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn user(&self) -> String {
        format!(r#"--user "{}""#, self.user)
    }

    fn workdir(&self) -> String {
        format!(r#"--workdir "{}""#, self.workdir)
    }

    fn compile(&self, instruction: &str) -> String {
        format!(
            "docker run --rm {} {} {} {} {} {instruction}",
            self.user(),
            self.volumes(),
            self.workdir(),
            self.env(),
            self.image,
        )
    }

    pub fn forward_env(&mut self, key: &impl ToString) {
        self.env.push(key.to_string());
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum ContainerConfiguration {
    None,
    _Docker(_DockerContainer),
}

impl ContainerConfiguration {
    pub fn compile(&self, instruction: &str) -> String {
        match self {
            ContainerConfiguration::None => instruction.to_string(),
            ContainerConfiguration::_Docker(docker) => docker.compile(instruction),
        }
    }
}

impl Default for ContainerConfiguration {
    fn default() -> Self {
        Self::None
    }
}

impl ContainerConfiguration {}

#[cfg(test)]
mod tests {
    use super::_DockerContainer;

    impl _DockerContainer {
        fn test(image: &str, env: &[&str], volumes: &[&str], user: &str, workdir: &str) -> Self {
            Self {
                image: image.to_string(),
                env: env.iter().map(ToString::to_string).collect(),
                volumes: volumes.iter().map(ToString::to_string).collect(),
                user: user.to_string(),
                workdir: workdir.to_string(),
            }
        }
    }

    #[test]
    fn parse() {
        let container = _DockerContainer::test(
            "rust:latest",
            &["CHANGED_FILES", "HAS_RUST"],
            &["PWD:PWD:rw"],
            "$USER_ID:$GROUP_ID",
            "$PWD",
        );

        assert_eq!(
            r#"docker run --rm --user "$USER_ID:$GROUP_ID" --volume "PWD:PWD:rw" --workdir "$PWD" --env "CHANGED_FILES" --env "HAS_RUST" rust:latest cargo fmt"#,
            &container.compile("cargo fmt")
        );
    }
}
