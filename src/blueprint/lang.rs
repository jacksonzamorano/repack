use std::collections::HashMap;

use crate::{
    blueprint::{BlueprintToken, TemplateDefineSection, BlueprintCommand, BlueprintFileReader},
    syntax::CoreType,
};

#[derive(Debug)]
pub enum BlueprintError {
    CannotRead,
    InvalidCommandSyntax(BlueprintCommand),
    NoSections,
    InconsistentContexts,
}

impl BlueprintError {
    pub fn output(&self) -> String {
        match self {
            Self::CannotRead => "Cannot read the requested file.".to_string(),
            Self::InvalidCommandSyntax(_) => "This command expected different syntax, please check the documentation.".to_string(),
            Self::NoSections => "This define block didn't specify any known sections.".to_string(),
            Self::InconsistentContexts => "This define block isn't valid because it relies on more than one context.".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Blueprint {
    pub id: String,
    pub name: String,
    pub types: HashMap<CoreType, String>,
    pub sections: HashMap<TemplateDefineSection, Vec<BlueprintToken>>,
    pub optional: Option<Vec<BlueprintToken>>,
    pub array: Option<Vec<BlueprintToken>>,
}
impl Blueprint {
    pub fn new(
        mut reader: BlueprintFileReader,
    ) -> Result<Blueprint, BlueprintError> {
        let mut lang = Blueprint {
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
            match next {
                BlueprintToken::Command(cmd) => {
                    cmd.handle(&mut reader, &mut lang)?;
                }
                _ => {}
            }
        }

        return Ok(lang);
    }
}
