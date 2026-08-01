#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObfuscatorType {
    CommandObfuscator,
    Compressor,
    Encoder,
    NoiseInjector,
    StringObfuscator,
    TokenObfuscator,
}

impl ObfuscatorType {
    /// Execution order: lower weight runs first.
    /// StringObfuscator/Encoder (1) → NoiseInjector (2) → CommandObfuscator (3)
    pub fn weight(self) -> u8 {
        match self {
            ObfuscatorType::Encoder          => 1,
            ObfuscatorType::StringObfuscator => 1,
            ObfuscatorType::TokenObfuscator  => 1,
            ObfuscatorType::NoiseInjector    => 2,
            ObfuscatorType::Compressor       => 2,
            ObfuscatorType::CommandObfuscator => 3,
        }
    }
}
