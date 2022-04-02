pub trait EnvBag {
    fn parse(&mut self, key: &str) -> Vec<String>;
    fn user(&self) -> String;
    fn group(&self) -> String;
    fn pwd(&self) -> String;
}

pub struct SimpleEnvBag {
    uid: String,
    gid: String,
    pwd: String,
    _env_keys: Vec<String>,
}

impl SimpleEnvBag {
    pub fn new<T: Into<String>>(uid: T, gid: T, pwd: T, _env_keys: Vec<String>) -> Self {
        Self {
            uid: uid.into(),
            gid: gid.into(),
            pwd: pwd.into(),
            _env_keys,
        }
    }
}

impl EnvBag for SimpleEnvBag {
    fn parse(&mut self, instruction: &str) -> Vec<String> {
        return instruction
            .split(' ')
            .filter(|str| !str.is_empty())
            .map(|str| str.to_string())
            .collect();
    }

    fn user(&self) -> String {
        self.uid.to_string()
    }

    fn group(&self) -> String {
        self.gid.to_string()
    }

    fn pwd(&self) -> String {
        self.pwd.to_string()
    }
}
