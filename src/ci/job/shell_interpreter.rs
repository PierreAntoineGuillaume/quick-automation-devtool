use crate::ci::job::env_parser::parse_env_into_map;
use crate::ci::job::ports::{SystemFacade, UserFacade};
use crate::ci::job::Output;
use crate::strvec;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Write;

pub struct ShellInterpreter<'a> {
    user_facade: &'a dyn UserFacade,
    system_facade: &'a dyn SystemFacade,
}

impl<'a> ShellInterpreter<'a> {
    pub fn new(user_facade: &'a dyn UserFacade, system_facade: &'a dyn SystemFacade) -> Self {
        Self {
            user_facade,
            system_facade,
        }
    }

    pub fn interpret(
        &self,
        additionnal_envtext: Option<String>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let mut map = HashMap::default();

        let regex = Regex::new(r"^\s*(\w+)=").unwrap();

        let mut env_text = strvec!("USER_ID=$(id -u)", "GROUP_ID=$(id -g)").join("\n");

        if let Some(additionnal) = additionnal_envtext {
            env_text = format!("{env_text}\n{additionnal}\n");
        }

        let mut control = String::new();

        env_text
            .split('\n')
            .for_each(|line| match regex.captures(line) {
                None => {}
                Some(captures) => {
                    let name = captures.get(1).unwrap().as_str().to_string();
                    writeln!(control, "printf {name}=; printf '%s\n' ${name}").expect("write");
                }
            });

        let script = format!("{env_text}\n{control}");
        let out = self.system_facade.run(&script);

        let envlist = match out {
            Output::Success(stdout, stderr) => {
                if !stderr.is_empty() {
                    self.user_facade.display_error(stderr);
                }
                stdout.trim().to_string()
            }
            Output::JobError(_, stderr) => return Err(anyhow!(stderr)),
            Output::ProcessError(stderr) => return Err(anyhow!(stderr)),
        };

        let intermediate_map = parse_env_into_map(envlist);

        for line in env_text.split('\n') {
            let mut keyval = line.split('=');
            if let Some(key) = keyval.next() {
                if let Some(value) = intermediate_map.get(key) {
                    map.insert(key.to_string(), value.clone());
                }
            }
        }

        Ok(map)
    }
}
