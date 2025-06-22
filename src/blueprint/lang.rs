use std::collections::HashMap;

use crate::{
    blueprint::{BlueprintCommand, BlueprintFileReader, BlueprintToken, TemplateDefineSection},
    syntax::CoreType,
};

#[derive(Debug)]
pub enum BlueprintError {
    CannotRead,
    InvalidCommandSyntax(BlueprintCommand),
    NoSections,
    InconsistentContexts,
    EnumsNotSupported,
    RecordsNotSupported,
    StructsNotSupported,
    TypeNotSupported(String),
    ArraysNotSupported,
}

impl BlueprintError {
    pub fn output(&self) -> String {
        match self {
            Self::CannotRead => "Cannot read the requested file.".to_string(),
            Self::InvalidCommandSyntax(_) => {
                "This command expected different syntax, please check the documentation."
                    .to_string()
            }
            Self::NoSections => "This define block didn't specify any known sections.".to_string(),
            Self::InconsistentContexts => {
                "This define block isn't valid because it relies on more than one context."
                    .to_string()
            }
            Self::EnumsNotSupported => {
                "This blueprint doesn't provide a template for enums, so enums cannot be used."
                    .to_string()
            }
            Self::RecordsNotSupported => {
                "This blueprint doesn't provide a template for records, so records cannot be used."
                    .to_string()
            }
            Self::StructsNotSupported => {
                "This blueprint doesn't provide a template for structs, so structs cannot be used."
                    .to_string()
            }
            Self::TypeNotSupported(typ) => {
                format!("This blueprint doesn't support the '{}' type.", typ)
            }
            Self::ArraysNotSupported => {
                "This blueprint doesn't support arrays.".to_string()
            }
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
    pub fn new(mut reader: BlueprintFileReader) -> Result<Blueprint, BlueprintError> {
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
