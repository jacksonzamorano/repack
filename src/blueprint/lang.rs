use super::SnippetDetails;
use crate::{
    blueprint::{BlueprintFileReader, FlyToken},
    syntax::CoreType,
};
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SnippetMainTokenName {
    Meta,
    If,
    Ifn,
    Each,
    TypeDef,
    Func,
    Join,
    Ref,
    Variable(String),
}
impl SnippetMainTokenName {
    fn from_string(val: &str) -> SnippetMainTokenName {
        match val {
            "meta" => Self::Meta,
            "if" => Self::If,
            "ifn" => Self::Ifn,
            "each" => Self::Each,
            "define" => Self::TypeDef,
            "func" => Self::Func,
            "join" => Self::Join,
            "ref" => Self::Ref,
            _ => Self::Variable(val.to_string()),
        }
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SnippetSecondaryTokenName {
    // Define
    Id,
    Name,
    Object,
    Field,
    Enum,
    Case,

    // TypeDef
    String,
    Int32,
    Int64,
    Float64,
    Uuid,
    DateTime,
    Boolean,

    Arbitrary(String),
}
impl SnippetSecondaryTokenName {
    fn from_string(val: &str) -> Self {
        if let Some(ct) = CoreType::from_string(val) {
            return Self::from_type(&ct);
        }
        match val {
            "id" => Self::Id,
            "name" => Self::Name,
            "object" => Self::Object,
            "field" => Self::Field,
            "enum" => Self::Enum,
            "case" => Self::Case,
            _ => Self::Arbitrary(val.to_string()),
        }
    }
    pub fn from_type(typ: &CoreType) -> SnippetSecondaryTokenName {
        match typ {
            CoreType::Uuid => Self::Uuid,
            CoreType::Int64 => Self::Int64,
            CoreType::Int32 => Self::Int32,
            CoreType::String => Self::String,
            CoreType::Float64 => Self::Float64,
            CoreType::Boolean => Self::Boolean,
            CoreType::DateTime => Self::DateTime,
        }
    }
}
type SnippetIdentifier = (SnippetMainTokenName, SnippetSecondaryTokenName);

#[allow(dead_code)]
#[derive(Debug)]
pub enum BlueprintError {
    CannotRead,
    InvalidFile,
    UnknownCommand(String),
    NoSections,
    InconsistentContexts,
    CouldNotCreateContext(&'static str),
    TypeNotSupported(String),
    VariableNotFound(String),
    InvalidFunctionSyntax,
    FunctionMissingArgument(String, String),
}
impl BlueprintError {
    pub fn output(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug)]
pub struct SectionContent {
    pub details: SnippetDetails,
    pub contents: Vec<FlyToken>,
    pub literal_string_value: String,
}

#[derive(Debug)]
pub struct SnippetReference<'a> {
    pub details: &'a SnippetDetails,
    pub contents: &'a [FlyToken],
}
impl<'a> SnippetReference<'a> {
    pub fn main_token(&self) -> SnippetMainTokenName {
        SnippetMainTokenName::from_string(&self.details.main_token)
    }
    pub fn secondary_token(&self) -> SnippetSecondaryTokenName {
        SnippetSecondaryTokenName::from_string(&self.details.secondary_token)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Blueprint {
    pub id: String,
    pub name: String,
    pub utilities: HashMap<SnippetIdentifier, SectionContent>,
    pub tokens: Vec<FlyToken>,
}
impl Blueprint {
    pub fn new(mut reader: BlueprintFileReader) -> Result<Blueprint, BlueprintError> {
        let mut lang = Blueprint {
            id: String::new(),
            name: String::new(),
            utilities: HashMap::new(),
            tokens: Vec::new(),
        };

        loop {
            let Some(next) = reader.next() else {
                break;
            };
            if let FlyToken::Snippet(snip) = &next {
                let (main, secondary) = (
                    SnippetMainTokenName::from_string(&snip.main_token),
                    SnippetSecondaryTokenName::from_string(&snip.secondary_token),
                );

                match main {
                    SnippetMainTokenName::TypeDef | SnippetMainTokenName::Meta => {
                        let mut participating_tokens = Vec::new();
                        if !snip.is_ended {
                            while let Some(in_block) = reader.next() {
                                match &in_block {
                                    FlyToken::Snippet(det)
                                        if det.main_token == "end"
                                            && det.secondary_token == snip.main_token =>
                                    {
                                        break;
                                    }
                                    _ => {
                                        participating_tokens.push(in_block);
                                    }
                                }
                            }
                        }
                        let mut literal_string_value = snip.contents.clone();
                        for t in &participating_tokens {
                            match t {
                                FlyToken::Literal(val) => {
                                    literal_string_value.push_str(val);
                                }
                                _ => {}
                            }
                        }

                        let contents = SectionContent {
                            contents: participating_tokens,
                            details: snip.clone(),
                            literal_string_value,
                        };
                        lang.utilities.insert((main, secondary), contents);
                    }
                    _ => lang.tokens.push(next),
                }
            } else {
                lang.tokens.push(next);
            }
        }

        if let Some(id) = lang
            .utilities
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Id))
        {
            lang.id = id.literal_string_value.clone();
        }
        if let Some(name) = lang
            .utilities
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Name))
        {
            lang.name = name.literal_string_value.clone();
        }

        dbg!(&lang.tokens);

        Ok(lang)
    }
}
