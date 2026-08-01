use super::ObfuscatorType;

#[derive(Clone, Debug)]
pub struct ObfuscatorModule {
    pub id: u8,
    pub name: String,
    pub description: String,
    pub obfuscator_type: ObfuscatorType,
}

impl ObfuscatorModule {
    pub fn new(
        id: u8,
        name: String,
        description: String,
        obfuscator_type: ObfuscatorType,
    ) -> ObfuscatorModule {
        ObfuscatorModule {
            id,
            name,
            description,
            obfuscator_type,
        }
    }
}
