use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::CommandRunner;
use crate::ci::job::{Introspector, Job, Progress, ProgressConsumer};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Docker {
    name: String,
    group: Option<String>,
    image: String,
    instructions: Vec<String>,
    env: Vec<String>,
    skip_if: Option<String>,
}

const DOCKER_RUN: &str =
    r#"docker run --rm --user "$USER_ID:$GROUP_ID" --volume "$PWD:$PWD" --workdir "$PWD""#;

impl Job for Docker {
    fn introspect(&self, introspector: &mut dyn Introspector) {
        introspector.docker_job(
            &self.name,
            &self.image,
            &self.group,
            &self.instructions,
            &self.skip_if,
        );
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>) {
        for key in env.keys() {
            self.env.push(key.clone());
        }
    }

    fn group(&self) -> Option<&str> {
        match &self.group {
            None => None,
            Some(string) => Some(string.as_str()),
        }
    }

    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn ProgressConsumer) {
        if let Some(condition) = &self.skip_if {
            if runner.run(condition).succeeded() {
                consumer.consume(JobProgress::new(&self.name, Progress::Skipped));
                consumer.consume(JobProgress::new(&self.name, Progress::Terminated(true)));
                return;
            }
        }

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

            let mut docker_args = self.image.split(' ');
            let image = docker_args.next().expect("there is at least an image");
            let args = docker_args.collect::<Vec<&str>>().join(" ").to_string();

            let command = format!(
                r#"{} {} {} {} {}"#,
                DOCKER_RUN, env, args, image, instruction
            );

            let output = runner.run(&command);
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

impl Docker {
    pub fn long(
        name: String,
        instructions: Vec<String>,
        image: String,
        group: Option<String>,
        skip_if: Option<String>,
    ) -> Self {
        Self {
            name,
            instructions,
            image,
            group,
            env: vec![],
            skip_if,
        }
    }
}
