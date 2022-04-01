use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::JobRunner;
use crate::ci::job::{JobIntrospector, JobOutput, JobProgressConsumer, JobTrait, Progress};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct DockerJob {
    name: String,
    group: Option<String>,
    image: String,
    instructions: Vec<String>,
}

impl JobTrait for DockerJob {
    fn introspect(&self, introspector: &mut dyn JobIntrospector) {
        introspector.docker_job(&self.name, &self.image, &self.group, &self.instructions)
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
        }
    }

    pub fn run(&self, instruction: &str, runner: &dyn JobRunner) -> JobOutput {
        let mut args = vec![
            "docker",
            "run",
            "--rm",
            "--user",
            "$USER:$GROUPS",
            "--volume",
            "$PWD:$PWD",
            "--workdir",
            "$PWD",
            self.image.as_str(),
        ];
        args.extend(instruction.split(' ').into_iter());
        runner.run(&args)
    }
}
