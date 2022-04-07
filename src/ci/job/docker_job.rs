use crate::ci::job::env_bag::EnvBag;
use crate::ci::job::inspection::JobProgress;
use crate::ci::job::instruction_interpreter::InstructionInterpreter;
use crate::ci::job::schedule::CommandRunner;
use crate::ci::job::{JobIntrospector, JobProgressConsumer, JobTrait, Progress};
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
        runner: &mut dyn CommandRunner,
        envbag: Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        consumer: &dyn JobProgressConsumer,
    ) {
        let mut success = true;

        let (uid, gid, pwd) = Self::get_env(&envbag);

        let user_string = format!("{}:{}", uid, gid);
        let volume_string = format!("{}:{}", pwd, pwd);

        let base_instructions = vec![
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

        let parser = InstructionInterpreter::arc_mutex(&envbag, &self.instructions);

        for word_list in parser {
            let executed = word_list.join(" ");
            consumer.consume(JobProgress::new(
                &self.name,
                Progress::Started(executed.clone()),
            ));

            let mut args = base_instructions.clone();
            args.extend(word_list.iter().map(|str| str.as_str()));
            let output = runner.run(&args);

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

    fn get_env(envbag: &Arc<Mutex<dyn EnvBag + Send + Sync>>) -> (String, String, String) {
        let lock = envbag.lock().unwrap();
        let uid = (*lock).user();
        let gid = (*lock).group();
        let pwd = (*lock).pwd();
        (uid, gid, pwd)
    }
}
