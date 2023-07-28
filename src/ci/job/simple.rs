use crate::ci::job::container_configuration::ContainerConfiguration;
use crate::ci::job::container_configuration::ContainerConfiguration::Container;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::CommandRunner;
use crate::ci::job::{Job, Progress, ProgressConsumer};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Simple {
    name: String,
    group: Option<String>,
    container: ContainerConfiguration,
    instructions: Vec<String>,
    skip_if: Option<String>,
}

impl Job for Simple {
    fn name(&self) -> &str {
        &self.name
    }

    fn forward_env(&mut self, env: &HashMap<String, Vec<String>>) {
        if let Container(container) = &mut self.container {
            for key in env.keys() {
                container.forward_env(key);
            }
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
        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let command = self.container.compile(instruction);

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

impl Simple {
    pub fn long(
        name: String,
        instructions: Vec<String>,
        group: Option<String>,
        skip_if: Option<String>,
    ) -> Self {
        Self {
            name,
            group,
            container: ContainerConfiguration::None,
            instructions,
            skip_if,
        }
    }

    pub fn new(
        name: String,
        instructions: Vec<String>,
        container: ContainerConfiguration,
        group: Option<String>,
        skip_if: Option<String>,
    ) -> Self {
        Self {
            name,
            group,
            container,
            instructions,
            skip_if,
        }
    }
}
