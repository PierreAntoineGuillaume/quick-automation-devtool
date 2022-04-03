use crate::ci::job::env_bag::EnvBag;
use std::sync::{Arc, Mutex};

pub struct InstructionParser<'a> {
    _envbag: &'a Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
    instructions: &'a Vec<String>,
    current_index: usize,
}

impl<'a> InstructionParser<'a> {
    pub fn new(
        envbag: &'a Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        instructions: &'a Vec<String>,
    ) -> Self {
        Self {
            _envbag: envbag,
            instructions,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for InstructionParser<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.instructions.len() {
            self.current_index += 1;
            let word_list = self.instructions[self.current_index - 1]
                .split(' ')
                .map(|str| str.to_string())
                .collect::<Vec<String>>();
            Some(word_list)
        } else {
            None
        }
    }
}
