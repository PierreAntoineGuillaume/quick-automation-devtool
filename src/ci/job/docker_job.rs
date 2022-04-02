use crate::ci::job::env_bag::EnvBag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::schedule::JobRunner;
use crate::ci::job::{JobIntrospector, JobOutput, JobProgressConsumer, JobTrait, Progress};
use std::sync::{Arc, Mutex};

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

    fn start(
        &self,
        runner: &mut dyn JobRunner,
        envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        consumer: &dyn JobProgressConsumer,
    ) {
        let mut success = true;

        for instruction in &self.instructions {
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(instruction.clone()),
            ));

            let output = self.run(instruction, runner, &envbag);

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

    pub fn run(
        &self,
        instruction: &str,
        runner: &dyn JobRunner,
        envbag: &Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
    ) -> JobOutput {
        let (uid, gid, pwd, parsed) = Self::get_instructions(instruction, envbag);

        let user_string = format!("{}:{}", uid, gid);
        let volume_string = format!("{}:{}", pwd, pwd);

        let mut args = vec![
            "docker",
            "run",
            "--rm",
            "--user",
            &user_string,
            "--volume",
            &volume_string,
            "--workdir",
            &pwd,
            self.image.as_str(),
        ];

        args.extend(parsed.iter().map(|string| string.as_str()));

        runner.run(&args)
    }

    fn get_instructions(
        instruction: &str,
        envbag: &Arc<Mutex<dyn EnvBag + Send + Sync>>,
    ) -> (String, String, String, Vec<String>) {
        let mut lock = envbag.lock().unwrap();
        let uid = (*lock).user();
        let gid = (*lock).group();
        let pwd = (*lock).pwd();
        let parsed = (*lock).parse(instruction);
        (uid, gid, pwd, parsed)
    }
}
