use crate::ci::job::env_bag::EnvBag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::instruction_interpreter::InstructionInterpreter;
use crate::ci::job::schedule::CommandRunner;
use crate::ci::job::{JobIntrospector, JobProgressConsumer, JobTrait, Progress};
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Eq, Hash, Debug)]
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

    fn group(&self) -> Option<&str> {
        match &self.group {
            None => None,
            Some(string) => Some(string.as_str()),
        }
    }

    fn start(
        &self,
        runner: &mut dyn CommandRunner,
        envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        consumer: &dyn JobProgressConsumer,
    ) {
        let mut success = true;
        let parser = InstructionInterpreter::arc_mutex(&envbag, &self.instructions);

        for word_list in parser {
            let executed = word_list.join(" ");
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(executed.clone()),
            ));

            let output = runner.run(
                &(word_list
                    .iter()
                    .map(|str| str.as_str())
                    .collect::<Vec<&str>>()),
            );

            success = output.succeeded();
            let partial = Progress::Partial(executed, output);
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

    pub fn short(name: String, instructions: Vec<String>) -> Self {
        Self::long(name, instructions, None)
    }
}
