use std::{collections::HashMap, fs::File, io::Read};

use crate::{
    profiles::{
        FlyContextualizedVariable, FlyToken, FlyTokenType, TemplatedLanguageReader,
        fly::{TemplateDefineSection, TemplateToken},
    },
    syntax::CoreType,
};

#[derive(Debug)]
pub enum TemplatedLanguageError {
    CannotRead,
    InvalidFile,
    UnknownCommand(String),
    InvalidCommandSyntax(TemplateToken),
    NoSections,
    InconsistentContexts,
}

#[derive(Debug)]
pub struct TemplatedLanguage {
    pub id: String,
    pub name: String,
    pub types: HashMap<CoreType, String>,
    pub sections: HashMap<TemplateDefineSection, Vec<FlyToken>>,
    pub optional: Option<String>,
    pub array: Option<String>,
}
impl TemplatedLanguage {
    pub fn from_file(path: &str) -> Result<TemplatedLanguage, TemplatedLanguageError> {
        let mut file = File::open(&path).map_err(|_| TemplatedLanguageError::CannotRead)?;
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let mut reader = TemplatedLanguageReader {
            reader: contents.iter().peekable(),
        };

        let mut lang = TemplatedLanguage {
            id: String::new(),
            name: String::new(),
            types: HashMap::new(),
            sections: HashMap::new(),
            optional: None,
            array: None,
        };

        loop {
            let Some(next) = reader.next_token() else {
                break;
            };
            match next.token_type {
                FlyTokenType::Command(cmd) => {
                    dbg!(&cmd);
                    cmd.handle(&mut reader, &mut lang)?;
                }
                _ => {
                    // dbg!(next);
                }
            }
        }

        return Ok(lang);
    }
}
