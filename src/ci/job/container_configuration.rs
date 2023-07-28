#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct DockerContainer {
    image: String,
    env: Vec<String>,
    volumes: Vec<String>,
    user: String,
    workdir: String,
}

impl DockerContainer {
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

    pub fn new(
        image: &impl ToString,
        user: &impl ToString,
        workdir: &impl ToString,
        volumes: &[impl ToString],
    ) -> Self {
        Self {
            image: image.to_string(),
            env: vec![],
            workdir: workdir.to_string(),
            user: user.to_string(),
            volumes: volumes.iter().map(ToString::to_string).collect(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum ContainerConfiguration {
    None,
    Container(DockerContainer),
}

impl ContainerConfiguration {
    pub fn compile(&self, instruction: &str) -> String {
        match self {
            ContainerConfiguration::None => instruction.to_string(),
            ContainerConfiguration::Container(docker) => docker.compile(instruction),
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
    use super::DockerContainer;

    impl DockerContainer {
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
        let container = DockerContainer::test(
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
