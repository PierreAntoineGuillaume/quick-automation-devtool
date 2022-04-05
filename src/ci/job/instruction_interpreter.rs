use crate::ci::job::env_bag::EnvBag;
use std::sync::{Arc, Mutex};

pub struct InstructionInterpreter<'a> {
    envbag: &'a Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
    instructions: &'a Vec<String>,
    current_index: usize,
}

impl<'a> InstructionInterpreter<'a> {
    pub fn arc_mutex(
        envbag: &'a Arc<Mutex<(dyn EnvBag + Send + Sync)>>,
        instructions: &'a Vec<String>,
    ) -> Self {
        Self {
            envbag,
            instructions,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for InstructionInterpreter<'a> {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.instructions.len() {
            self.current_index += 1;
            let mut word_list = vec![];
            self.instructions[self.current_index - 1]
                .split(' ')
                .for_each(|str| {
                    if let Some(stripped) = str.strip_prefix('$') {
                        let mut mutex = self.envbag.lock().unwrap();
                        let opt = (*mutex).read(stripped);
                        if let Some(vec) = opt {
                            for value in vec {
                                word_list.push(value)
                            }
                        } else {
                            word_list.push(str.to_string())
                        }
                    } else {
                        word_list.push(str.to_string())
                    }
                });

            Some(word_list)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strvec;

    struct TestEnvBag;

    impl EnvBag for TestEnvBag {
        fn user(&self) -> String {
            "uid".into()
        }

        fn group(&self) -> String {
            "gid".into()
        }

        fn pwd(&self) -> String {
            "pwd".into()
        }

        fn read(&mut self, key: &str) -> Option<Vec<String>> {
            match key {
                "KEY" => Some(strvec!("VALUE")),
                _ => None,
            }
        }
    }

    impl TestEnvBag {
        fn arc_mutex() -> Arc<Mutex<(dyn EnvBag + Send + Sync)>> {
            Arc::from(Mutex::new(Self))
        }
    }

    #[test]
    pub fn iterate_normal() {
        let env = TestEnvBag::arc_mutex();
        let instructions = vec!["Parse this $KEY please".to_string()];
        let mut parser = InstructionInterpreter::arc_mutex(&env, &instructions);

        assert_eq!(
            Some(strvec!["Parse", "this", "VALUE", "please"]),
            parser.next()
        )
    }
}
