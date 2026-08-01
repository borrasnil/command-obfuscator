use rand::Rng;

use crate::core::engine::{Obfuscate, ObfuscatorType, OS};

pub struct HexObfuscator;

impl Obfuscate for HexObfuscator {
    fn apply(&self, command: &str, _os: OS) -> String {
        let mut rng = rand::thread_rng();
        command
            .chars()
            .map(|c| {
                if c.is_ascii() && rng.gen_bool(0.4) {
                    format!("$'\\x{:02x}'", c as u8)
                } else {
                    c.to_string()
                }
            })
            .collect()
    }

    fn module_type(&self) -> ObfuscatorType {
        ObfuscatorType::Encoder
    }
}
