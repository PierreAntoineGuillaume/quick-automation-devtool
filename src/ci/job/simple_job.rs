use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::CommandRunner;
use crate::ci::job::{JobIntrospector, JobProgressConsumer, JobTrait, Progress};
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct SimpleJob {
    name: String,
    group: Option<String>,
    instructions: Vec<String>,
}

impl JobTrait for SimpleJob {
    fn introspect(&self, introspector: &mut dyn JobIntrospector) {
        introspector.basic_job(&self.name, &self.group, &self.instructions)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn forward_env(&mut self, _: &HashMap<String, Vec<String>>) {}

    fn group(&self) -> Option<&str> {
        match &self.group {
            None => None,
            Some(string) => Some(string.as_str()),
        }
    }

    fn start(&self, runner: &mut dyn CommandRunner, consumer: &dyn JobProgressConsumer) {
        let mut success = true;

        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let output = runner.run(&["bash", "-c", instruction]);

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

impl SimpleJob {
    pub fn long(name: String, instructions: Vec<String>, group: Option<String>) -> Self {
        Self {
            name,
            instructions,
            group,
        }
    }
}
