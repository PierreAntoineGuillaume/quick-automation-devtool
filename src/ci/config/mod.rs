use super::job::Pipeline;
use crate::ci::job::Job;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct JobSection {
    jobs: std::collections::HashMap<String, std::vec::Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    version: String,
}

impl Config {
    /// Will compile and load pipeline config from multiple sources with descending priority order
    /// ./dt/config.tml
    pub fn load_into(pipeline: &mut Pipeline) {
        let file = "dt.toml";
        if let Ok(content) = fs::read_to_string(&file) {
            let res = toml::from_str::<Config>(content.as_str());
            if res.is_err() {
                eprintln!(
                    "dt: could not parse {} config file, missing version=<int>",
                    file
                );
                std::process::exit(2);
            }
            let config = res.unwrap();
            if config.version == "0.x" {
                let v1: JobSection = toml::from_str(content.as_str()).unwrap();
                for (name, instructions) in &v1.jobs {
                    pipeline.push_job(Job {
                        name: name.clone(),
                        instructions: instructions.clone(),
                    });
                }
            } else {
                eprintln!(
                    "dt: version {} is unrecognised in file {}",
                    config.version, file
                );
                std::process::exit(2);
            }
        }
    }
}

#[cfg(test)]
mod tests {}
