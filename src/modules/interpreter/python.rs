use crate::core::engine::{OS, Obfuscate, ObfuscatorType};

pub struct PythonWrapper;

impl Obfuscate for PythonWrapper {
    fn apply(&self, command: &str, _os: OS) -> String {
        command.to_string()
    }

    fn module_type(&self) -> ObfuscatorType {
        ObfuscatorType::InterpreterWrapper
    }
}
