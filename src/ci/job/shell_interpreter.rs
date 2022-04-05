use crate::ci::job::env_bag::{EnvBag, SimpleEnvBag};
use crate::ci::job::env_parser::EnvParser;
use anyhow::{Error, Result};
use regex::Regex;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct ShellInterpreter {
    uid: String,
    gid: String,
    pwd: String,
    envtext: Option<String>,
}

impl ShellInterpreter {
    pub fn new(uid: String, gid: String, pwd: String, envtext: Option<String>) -> Self {
        Self {
            uid,
            gid,
            pwd,
            envtext,
        }
    }
    pub fn interpret(&self) -> Result<Arc<Mutex<(dyn EnvBag + Send + Sync)>>> {
        let mut map = HashMap::default();

        map.insert("UID".to_string(), vec![self.uid.clone()]);
        map.insert("GID".to_string(), vec![self.gid.clone()]);
        map.insert("PWD".to_string(), vec![self.pwd.clone()]);

        let regex = Regex::new(r"^\s*(\w+)=").unwrap();

        if let Some(envtext) = &self.envtext {
            let mut control = String::new();
            envtext
                .split('\n')
                .for_each(|line| match regex.captures(line) {
                    None => {}
                    Some(captures) => {
                        let name = captures.get(1).unwrap().as_str().to_string();
                        control.push_str(&format!("printf {}=; printf '%s\n' ${}\n", name, name))
                    }
                });
            let script = format!("{envtext}\n{control}");
            let default_shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("bash"));
            let output = Command::new(&default_shell)
                .args(&["-c", &script])
                .stderr(Stdio::inherit())
                .output()
                .unwrap();

            let envlist = String::from(std::str::from_utf8(&output.stdout).unwrap())
                .trim()
                .to_string();

            if !output.status.success() {
                return Err(Error::msg("env script failed"));
            }

            let parser = EnvParser {};
            let intermediate_map = parser.parse(envlist);

            for line in envtext.split('\n') {
                let mut keyval = line.split('=');
                if let Some(key) = keyval.next() {
                    if let Some(value) = intermediate_map.get(key) {
                        map.insert(key.to_string(), value.clone());
                    }
                }
            }
        }
        Ok(Arc::from(Mutex::new(SimpleEnvBag::new(
            self.uid.clone(),
            self.gid.clone(),
            self.pwd.clone(),
            map,
        ))))
    }
}
