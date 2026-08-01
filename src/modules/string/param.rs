use rand::Rng;

use crate::core::engine::{Obfuscate, ObfuscatorType, OS};
use crate::utils;

pub struct ParamObfuscator;

// # ## % %% (prefix/suffix stripping) are valid since bash 2 and zsh.
// ^^ ,, ^ , (case modifiers) require bash 4+ — not available on macOS default bash 3.2.
const MODIFIERS: &[&str] = &["#", "##", "%", "%%"];

impl Obfuscate for ParamObfuscator {
    fn apply(&self, command: &str, _os: OS) -> String {
        let mut rng = rand::thread_rng();
        tokenize(command)
            .iter()
            .flat_map(|tok| {
                let mut parts: Vec<String> = Vec::new();
                if rng.gen_bool(0.3) {
                    parts.push(random_param_junk(&mut rng));
                }
                parts.push(tok.clone());
                parts
            })
            .collect()
    }

    fn module_type(&self) -> ObfuscatorType {
        ObfuscatorType::NoiseInjector
    }
}

fn random_param_junk(rng: &mut impl Rng) -> String {
    let modifier = MODIFIERS[rng.gen_range(0..MODIFIERS.len())];
    format!("${{@{}{}}}", modifier, utils::junk())
}

// Splits a bash string into atomic tokens so junk is never injected inside a quoted region.
// Recognises: $'...' (ANSI-C), '...' (single), "..." (double), and bare chars.
fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // $'...' — ANSI-C quoting used by HexObfuscator
            '$' if chars.peek() == Some(&'\'') => {
                chars.next(); // consume '
                let mut tok = String::from("$'");
                loop {
                    match chars.next() {
                        Some('\'') => { tok.push('\''); break; }
                        Some(inner) => tok.push(inner),
                        None => break,
                    }
                }
                tokens.push(tok);
            }
            '\'' => {
                let mut tok = String::from('\'');
                loop {
                    match chars.next() {
                        Some('\'') => { tok.push('\''); break; }
                        Some(inner) => tok.push(inner),
                        None => break,
                    }
                }
                tokens.push(tok);
            }
            '"' => {
                let mut tok = String::from('"');
                loop {
                    match chars.next() {
                        Some('"') => { tok.push('"'); break; }
                        Some(inner) => tok.push(inner),
                        None => break,
                    }
                }
                tokens.push(tok);
            }
            other => tokens.push(other.to_string()),
        }
    }
    tokens
}
