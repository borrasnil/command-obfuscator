pub mod core;
pub mod modules;
pub mod utils;

pub use core::engine::{OS, Obfuscate, ObfuscatorModule, ObfuscatorType, Pipeline};

#[cfg(test)]
mod tests {
    use super::*;
    use modules::command::reverse::ReverseObfuscator;
    use modules::string::hex::HexObfuscator;
    use modules::string::param::ParamObfuscator;
    use modules::string::quotes::QuotesObfuscator;

    // Strips all ${@...} param modifier injections — they evaluate to empty when $@ is unset.
    // Handles \} inside the expression so the scanner doesn't close early.
    fn strip_param_modifiers(s: &str) -> Option<String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume {
                if chars.peek() == Some(&'@') {
                    chars.next(); // consume @
                    loop {
                        match chars.next()? {
                            '\\' => { chars.next(); } // skip escaped char (e.g. \})
                            '}' => break,
                            _ => {}
                        }
                    }
                    // ${@...} = empty when no positional params — nothing added
                } else {
                    result.push_str("${");
                }
            } else {
                result.push(c);
            }
        }
        Some(result)
    }

    // Evaluates $'\xHH' hex escape sequences as bash ANSI-C quoting does.
    fn eval_hex_escapes(s: &str) -> Option<String> {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'\'') {
                chars.next(); // consume '
                let backslash = chars.next()?;
                let x = chars.next()?;
                if backslash != '\\' || x != 'x' { return None; }
                let h1 = chars.next()?;
                let h2 = chars.next()?;
                let close = chars.next()?;
                if close != '\'' { return None; }
                let byte = u8::from_str_radix(&format!("{h1}{h2}"), 16).ok()?;
                result.push(byte as char);
            } else {
                result.push(c);
            }
        }
        Some(result)
    }

    // Simulates how bash evaluates adjacent quoted strings.
    // Does not handle escape sequences — our obfuscator never emits them.
    fn eval_bash_quotes(s: &str) -> Option<String> {
        let mut result = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            match c {
                '\'' => {
                    let mut closed = false;
                    for inner in chars.by_ref() {
                        if inner == '\'' { closed = true; break; }
                        result.push(inner);
                    }
                    if !closed { return None; }
                }
                '"' => {
                    let mut closed = false;
                    for inner in chars.by_ref() {
                        if inner == '"' { closed = true; break; }
                        result.push(inner);
                    }
                    if !closed { return None; }
                }
                c => result.push(c),
            }
        }
        Some(result)
    }

    #[test]
    fn pipeline_no_modules_returns_original() {
        let result = Pipeline::new(OS::Linux).run("cat /etc/passwd");
        assert_eq!(result, "cat /etc/passwd");
    }

    #[test]
    fn pipeline_reverse() {
        let result = Pipeline::new(OS::Linux)
            .add(ReverseObfuscator)
            .run("cat /etc/passwd");
        assert_eq!(result, "$(rev <<< \"dwssap/cte/ tac\")");
    }

    #[test]
    fn pipeline_quotes_roundtrip() {
        let input = "cat /etc/passwd";
        for _ in 0..50 {
            let result = Pipeline::new(OS::Linux).add(QuotesObfuscator).run(input);
            assert_eq!(eval_bash_quotes(&result), Some(input.to_string()));
        }
    }

    #[test]
    fn pipeline_quotes_single_quote_roundtrip() {
        let input = "echo it's";
        for _ in 0..50 {
            let result = Pipeline::new(OS::Linux).add(QuotesObfuscator).run(input);
            assert_eq!(eval_bash_quotes(&result), Some(input.to_string()));
        }
    }

    #[test]
    fn pipeline_quotes_special_chars_roundtrip() {
        let input = "echo $HOME";
        for _ in 0..50 {
            let result = Pipeline::new(OS::Linux).add(QuotesObfuscator).run(input);
            assert_eq!(eval_bash_quotes(&result), Some(input.to_string()));
        }
    }

    #[test]
    fn pipeline_quotes_empty() {
        let result = Pipeline::new(OS::Linux).add(QuotesObfuscator).run("");
        assert_eq!(result, "");
    }

    #[test]
    fn pipeline_hex_roundtrip() {
        let input = "echo test";
        for _ in 0..50 {
            let result = Pipeline::new(OS::Linux).add(HexObfuscator).run(input);
            assert_eq!(eval_hex_escapes(&result), Some(input.to_string()));
        }
    }

    #[test]
    fn pipeline_hex_empty() {
        let result = Pipeline::new(OS::Linux).add(HexObfuscator).run("");
        assert_eq!(result, "");
    }

    #[test]
    fn pipeline_param_roundtrip() {
        let input = "echo test";
        for _ in 0..50 {
            let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run(input);
            assert_eq!(strip_param_modifiers(&result), Some(input.to_string()));
        }
    }

    #[test]
    fn pipeline_param_empty() {
        let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run("");
        assert_eq!(result, "");
    }

    #[test]
    fn pipeline_param_no_junk_in_injections() {
        let input = "cat /etc/passwd";
        for _ in 0..50 {
            let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run(input);
            assert!(strip_param_modifiers(&result).is_some());
        }
    }

    // --- param modifier structural tests ---

    // Parses every ${@modifier junk} block from a string.
    // Returns None if any block is malformed (bad modifier, unclosed, etc.).
    fn extract_param_blocks(s: &str) -> Option<Vec<(String, String)>> {
        let mut blocks = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c != '$' { continue; }
            if chars.peek() != Some(&'{') { continue; }
            chars.next(); // {
            if chars.peek() != Some(&'@') { continue; }
            chars.next(); // @

            // Must start with a valid modifier: # ## % %%
            let first = chars.next()?;
            let modifier = match first {
                '#' => if chars.peek() == Some(&'#') { chars.next(); "##" } else { "#" },
                '%' => if chars.peek() == Some(&'%') { chars.next(); "%%" } else { "%" },
                _ => return None,
            };

            // Collect junk until the first unescaped }
            let mut junk = String::new();
            loop {
                match chars.next()? {
                    '\\' => { junk.push('\\'); junk.push(chars.next()?); }
                    '}' => break,
                    c   => junk.push(c),
                }
            }

            blocks.push((modifier.to_string(), junk));
        }
        Some(blocks)
    }

    const VALID_MODIFIERS: &[&str] = &["#", "##", "%", "%%"];

    #[test]
    fn param_blocks_have_valid_modifiers() {
        for _ in 0..100 {
            let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run("echo test");
            let blocks = extract_param_blocks(&result)
                .unwrap_or_else(|| panic!("malformed param block in: {result}"));
            for (modifier, _) in &blocks {
                assert!(
                    VALID_MODIFIERS.contains(&modifier.as_str()),
                    "invalid modifier `{modifier}` in: {result}"
                );
            }
        }
    }

    #[test]
    fn param_blocks_are_properly_closed() {
        for _ in 0..100 {
            let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run("echo test");
            assert!(
                extract_param_blocks(&result).is_some(),
                "unclosed param block in: {result}"
            );
        }
    }

    #[test]
    fn param_blocks_junk_has_no_bare_close_brace() {
        for _ in 0..100 {
            let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run("echo test");
            let blocks = extract_param_blocks(&result).unwrap();
            for (_, junk) in &blocks {
                // \} is a legitimately escaped brace — strip it, then check for bare }
                let without_escaped = junk.replace("\\}", "");
                assert!(!without_escaped.contains('}'), "bare }} in junk `{junk}`");
            }
        }
    }

    #[test]
    fn param_blocks_junk_contains_brackets() {
        // [ and ] are valid in glob patterns — verify they appear across runs
        let mut found = false;
        for _ in 0..500 {
            let result = Pipeline::new(OS::Linux).add(ParamObfuscator).run("echo test");
            let blocks = extract_param_blocks(&result).unwrap();
            if blocks.iter().any(|(_, j)| j.contains('[') || j.contains(']')) {
                found = true;
                break;
            }
        }
        assert!(found, "expected [ or ] in junk at least once across 500 runs");
    }

    #[test]
    fn pipeline_quotes_then_param_roundtrip() {
        let input = "echo test";
        for _ in 0..100 {
            let result = Pipeline::new(OS::Linux)
                .add(QuotesObfuscator)
                .add(ParamObfuscator)
                .run(input);
            let stripped = strip_param_modifiers(&result)
                .unwrap_or_else(|| panic!("malformed param block in: {result}"));
            assert_eq!(
                eval_bash_quotes(&stripped),
                Some(input.to_string()),
                "round-trip failed for: {result}"
            );
        }
    }

    #[test]
    fn pipeline_hex_then_param_roundtrip() {
        let input = "echo test";
        for _ in 0..100 {
            let result = Pipeline::new(OS::Linux)
                .add(HexObfuscator)
                .add(ParamObfuscator)
                .run(input);
            let stripped = strip_param_modifiers(&result)
                .unwrap_or_else(|| panic!("malformed param block in: {result}"));
            assert_eq!(
                eval_hex_escapes(&stripped),
                Some(input.to_string()),
                "round-trip failed for: {result}"
            );
        }
    }

    #[test]
    fn module_types_are_correct() {
        assert_eq!(QuotesObfuscator.module_type(), ObfuscatorType::StringObfuscator);
        assert_eq!(HexObfuscator.module_type(),    ObfuscatorType::Encoder);
        assert_eq!(ParamObfuscator.module_type(),  ObfuscatorType::NoiseInjector);
        assert_eq!(ReverseObfuscator.module_type(), ObfuscatorType::CommandObfuscator);
    }
}
