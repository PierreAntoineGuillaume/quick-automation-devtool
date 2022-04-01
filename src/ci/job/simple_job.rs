use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::JobRunner;
use crate::ci::job::{JobIntrospector, JobOutput, JobProgressConsumer, JobTrait, Progress};

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

    fn start(&self, runner: &mut dyn JobRunner, consumer: &dyn JobProgressConsumer) {
        let mut success = true;

        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let output = self.run(instruction, runner);
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

    pub fn short(name: String, instructions: Vec<String>) -> Self {
        Self::long(name, instructions, None)
    }

    pub fn run(&self, instruction: &str, runner: &dyn JobRunner) -> JobOutput {
        runner.run(&instruction.split(' ').into_iter().collect::<Vec<&str>>())
    }
}
