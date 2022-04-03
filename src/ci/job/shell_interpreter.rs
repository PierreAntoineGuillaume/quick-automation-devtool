use crate::ci::job::env_bag::{EnvBag, SimpleEnvBag};
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
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
    pub fn interpret(&self) -> Arc<Mutex<(dyn EnvBag + Send + Sync)>> {
        let mut map = HashMap::default();

        map.insert("UID".to_string(), self.uid.clone());
        map.insert("GID".to_string(), self.gid.clone());
        map.insert("PWD".to_string(), self.pwd.clone());

        let regex = Regex::new(r"^\w+=").unwrap();

        if let Some(envtext) = &self.envtext {
            let output = Command::new("env")
                .args(&[
                    "sh",
                    "-c",
                    &format!(
                        "{}\nenv",
                        envtext
                            .split('\n')
                            .map(|line| {
                                if regex.is_match(line) {
                                    format!("export {}", line)
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    ),
                ])
                .output()
                .unwrap();

            let envlist = String::from(std::str::from_utf8(&output.stdout).unwrap())
                .trim()
                .to_string();

            let mut intermediate_map = HashMap::new();

            for item in envlist.split('\n').filter(|item| item.contains('=')) {
                let mut keyval = item.split('=');
                let key = keyval.next().unwrap();
                let val = keyval.next().unwrap();
                intermediate_map.insert(key, val);
            }

            for line in envtext.split('\n') {
                let mut keyval = line.split('=');
                if let Some(key) = keyval.next() {
                    if let Some(value) = intermediate_map.get(key) {
                        map.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        Arc::from(Mutex::new(SimpleEnvBag::new(
            self.uid.clone(),
            self.gid.clone(),
            self.pwd.clone(),
            map,
        )))
    }
}
