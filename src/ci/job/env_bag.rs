use std::collections::HashMap;

pub trait EnvBag {
    fn user(&self) -> String;
    fn group(&self) -> String;
    fn pwd(&self) -> String;
    fn read(&mut self, key: &str) -> Option<Vec<String>>;
}

pub struct SimpleEnvBag {
    uid: String,
    gid: String,
    pwd: String,
    env: HashMap<String, Vec<String>>,
}

impl SimpleEnvBag {
    pub fn new<T: Into<String>>(uid: T, gid: T, pwd: T, env: HashMap<String, Vec<String>>) -> Self {
        Self {
            uid: uid.into(),
            gid: gid.into(),
            pwd: pwd.into(),
            env,
        }
    }
}

impl EnvBag for SimpleEnvBag {
    fn user(&self) -> String {
        self.uid.to_string()
    }

    fn group(&self) -> String {
        self.gid.to_string()
    }

    fn pwd(&self) -> String {
        self.pwd.to_string()
    }

    fn read(&mut self, key: &str) -> Option<Vec<String>> {
        self.env.get(key).cloned()
    }
}
