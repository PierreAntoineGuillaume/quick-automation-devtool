use crate::config::instructions::InstructionCompilerPass;
use regex::Regex;

pub struct WhitespacesPass {}

impl InstructionCompilerPass for WhitespacesPass {
    fn interpret(&self, instruction: &str) -> String {
        let regex = Regex::new(r"\s+").expect("Regex error cannot be expected");
        let vec: Vec<&str> = regex.split(instruction.trim()).collect();
        vec.join(" ")
    }
}
