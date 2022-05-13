use crate::ci::job::env_parser::Capturing::ValueWithKey;
use std::collections::HashMap;

#[derive(Debug)]
enum Capturing {
    Key,
    ValueWithKey(String),
}

pub fn parse_env_into_map<T: Into<String>>(env_string: T) -> HashMap<String, Vec<String>> {
    let env_string = env_string.into();
    let mut pos = 0;
    let mut last_symbol_start = 0;
    let mut last_new_line = 0;
    let mut capturing = Capturing::Key;
    let mut map = HashMap::<String, Vec<String>>::new();
    for char in env_string.chars() {
        match (&capturing, char) {
            (Capturing::Key, '=') => {
                capturing = ValueWithKey(env_string[last_symbol_start..pos].to_string());
                last_symbol_start = pos + 1;
            }
            (ValueWithKey(key), '=') => {
                if last_symbol_start < last_new_line {
                    map.insert(
                        key.to_string(),
                        env_string[last_symbol_start..last_new_line]
                            .to_string()
                            .split('\n')
                            .map(std::string::ToString::to_string)
                            .collect(),
                    );
                } else {
                    map.insert(key.to_string(), vec![]);
                }
                capturing = ValueWithKey(env_string[last_new_line + 1..pos].to_string());
                last_symbol_start = pos + 1;
            }
            (_, '\n') => last_new_line = pos,
            _ => {}
        }
        pos += 1;
    }
    if let ValueWithKey(key) = capturing {
        map.insert(
            key,
            env_string[last_symbol_start..pos]
                .to_string()
                .split('\n')
                .map(std::string::ToString::to_string)
                .collect(),
        );
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strvec;

    fn extract_map(map: HashMap<String, Vec<String>>) -> String {
        let mut result = vec![];
        for (key, value) in map {
            result.push(format!("{}: [{}]", key, value.join(", ")));
        }
        result.sort();
        result.join("")
    }

    #[test]
    pub fn empty() {
        let map = parse_env_into_map("");
        assert!(map.is_empty());
    }

    #[test]
    pub fn multiple() {
        let map = parse_env_into_map("KEY1=value\nKEY2=value");
        assert_eq!(
            extract_map(map),
            strvec!("KEY1: [value]", "KEY2: [value]").join("")
        );
    }

    #[test]
    pub fn multiple_with_multiline() {
        let map = parse_env_into_map("KEY1=value\nwith\nnew\nlines\nKEY2=value");
        assert_eq!(
            extract_map(map),
            strvec!("KEY1: [value, with, new, lines]", "KEY2: [value]").join("")
        );
    }

    #[test]
    pub fn empty_var() {
        let map = parse_env_into_map("KEY1=\nKEY2=value");
        assert_eq!(
            extract_map(map),
            strvec!("KEY1: []", "KEY2: [value]").join("")
        );
    }
}
