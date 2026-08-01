use crate::core::engine::{Obfuscate, ObfuscatorType, OS};

pub struct ReverseObfuscator;

impl Obfuscate for ReverseObfuscator {
    fn apply(&self, command: &str, _os: OS) -> String {
        let reversed: String = command.chars().rev().collect();
        format!("$(rev <<< \"{}\")", reversed)
    }

    fn module_type(&self) -> ObfuscatorType {
        ObfuscatorType::CommandObfuscator
    }
}
