use rand::Rng;

use crate::core::engine::{Obfuscate, ObfuscatorType, OS};

pub struct QuotesObfuscator;

impl Obfuscate for QuotesObfuscator {
    fn apply(&self, command: &str, _os: OS) -> String {
        quote_bash(command)
    }

    fn module_type(&self) -> ObfuscatorType {
        ObfuscatorType::StringObfuscator
    }
}

#[derive(Clone, Copy)]
enum QuoteStyle {
    Single, // 'c'
    Double, // "c"
    Bare,   // c
}

// Safe to leave unquoted in bash
fn can_be_bare(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '/' | '.' | '-' | '_' | '@' | '%' | ':' | ',' | '+' | '=')
}

// These chars are special inside double quotes and must not be double-quoted
fn can_be_double_quoted(c: char) -> bool {
    !matches!(c, '"' | '$' | '`' | '\\' | '!')
}

// Single quotes cannot contain a literal single quote
fn can_be_single_quoted(c: char) -> bool {
    c != '\''
}

fn quote_bash(command: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut result = String::new();

    for c in command.chars() {
        // Whitespace must stay unquoted — quoted spaces suppress bash word splitting,
        // turning "echo test" into the single word "echo test" (command not found).
        if c.is_whitespace() {
            result.push(c);
            continue;
        }

        // Inject 0-2 random empty quote pairs as noise
        for _ in 0..rng.gen_range(0u8..=2) {
            result.push_str(if rng.gen_bool(0.5) { "''" } else { "\"\"" });
        }

        let mut options = Vec::new();
        if can_be_bare(c)          { options.push(QuoteStyle::Bare); }
        if can_be_single_quoted(c) { options.push(QuoteStyle::Single); }
        if can_be_double_quoted(c) { options.push(QuoteStyle::Double); }

        match options[rng.gen_range(0..options.len())] {
            QuoteStyle::Bare   => result.push(c),
            QuoteStyle::Single => { result.push('\''); result.push(c); result.push('\''); }
            QuoteStyle::Double => { result.push('"');  result.push(c); result.push('"'); }
        }
    }

    result
}
