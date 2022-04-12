use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::CommandRunner;
use crate::ci::job::{JobIntrospector, JobProgressConsumer, JobTrait, Progress};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct DockerJob {
    name: String,
    group: Option<String>,
    image: String,
    instructions: Vec<String>,
    env: Vec<String>,
}

const DOCKER_RUN: &str =
    r#"docker run --rm --user "$USER_ID:$GROUP_ID" --volume "$PWD:$PWD" --workdir "$PWD""#;

impl JobTrait for DockerJob {
    fn introspect(&self, introspector: &mut dyn JobIntrospector) {
        introspector.docker_job(&self.name, &self.image, &self.group, &self.instructions)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>) {
        for key in env.keys() {
            self.env.push(key.clone())
        }
    }

    fn group(&self) -> Option<&str> {
        match &self.group {
            None => None,
            Some(string) => Some(string.as_str()),
        }
    }

    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn JobProgressConsumer) {
        let mut success = true;

        let env = self
            .env
            .iter()
            .map(|key| format!(r#"--env "{key}=${key}""#))
            .collect::<Vec<String>>()
            .join(" ");

        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let output = runner.run(&[
                "bash",
                "-c",
                &format!(r#"{} {} {} {}"#, DOCKER_RUN, env, self.image, instruction),
            ]);

            success = output.succeeded();
            let partial = Progress::Partial(instruction.clone(), output);
            consumer.consume(JobProgress::new(&self.name, partial));
            if !success {
                break;
            }
        }

        consumer.consume(JobProgress::new(&self.name, Progress::Terminated(success)));
    }
}

impl DockerJob {
    pub fn long(
        name: String,
        instructions: Vec<String>,
        image: String,
        group: Option<String>,
    ) -> Self {
        Self {
            name,
            instructions,
            image,
            group,
            env: vec![],
        }
    }
}
