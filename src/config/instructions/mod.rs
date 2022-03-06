use crate::config::instructions::whitespaces::WhitespacesPass;

mod whitespaces;

pub trait InstructionCompilerPass {
    fn interpret(&self, instruction: &str) -> String;
}

pub struct InstructionCompiler {
    passes: Vec<Box<dyn InstructionCompilerPass>>,
}

impl InstructionCompiler {
    pub fn compile(&self, instruction: &str) -> String {
        let mut instruction = instruction.to_string();
        for pass in &self.passes {
            instruction = pass.interpret(&instruction);
        }
        instruction
    }
}

impl Default for InstructionCompiler {
    fn default() -> Self {
        InstructionCompiler {
            passes: vec![Box::new(WhitespacesPass {})],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::instructions::InstructionCompiler;

    #[test]
    pub fn test_cleanup() {
        let compiler = InstructionCompiler::default();

        assert_eq!(
            &compiler.compile("  string with bad  spacing"),
            "string with bad spacing"
        );
    }
}
