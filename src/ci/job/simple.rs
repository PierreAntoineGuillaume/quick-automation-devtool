use crate::ci::job::inspection::JobProgress;
use crate::ci::job::ports::CommandRunner;
use crate::ci::job::{Introspector, Job, Progress, ProgressConsumer};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Simple {
    name: String,
    group: Option<String>,
    instructions: Vec<String>,
    skip_if: Option<String>,
}

impl Job for Simple {
    fn introspect(&self, introspector: &mut dyn Introspector) {
        introspector.basic_job(&self.name, &self.group, &self.instructions, &self.skip_if);
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

            let output = runner.run(instruction);

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
            instructions,
            skip_if,
        }
    }
}
