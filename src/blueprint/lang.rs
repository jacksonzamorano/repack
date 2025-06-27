use super::SnippetDetails;
use crate::{
    blueprint::{BlueprintFileReader, FlyToken},
    syntax::{CoreType, RepackError},
};
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SnippetMainTokenName {
    Meta,
    File,
    If,
    Ifn,
    Each,
    Eachr,
    EachInline,
    TypeDef,
    Func,
    Join,
    Ref,
    Link,
    Import,
    PlaceImports,
    Variable(String),
}
impl SnippetMainTokenName {
    pub(crate) fn from_string(val: &str) -> SnippetMainTokenName {
        match val {
            "meta" => Self::Meta,
            "if" => Self::If,
            "ifn" => Self::Ifn,
            "each" => Self::Each,
            "eachr" => Self::Eachr,
            "eachl" => Self::EachInline,
            "define" => Self::TypeDef,
            "func" => Self::Func,
            "join" => Self::Join,
            "ref" => Self::Ref,
            "file" => Self::File,
            "link" => Self::Link,
            "import" => Self::Import,
            "imports" => Self::PlaceImports,
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

    Join,

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
            "join" => Self::Join,
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
    pub links: HashMap<String, String>,
    pub utilities: HashMap<SnippetIdentifier, String>,
    pub tokens: Vec<FlyToken>,
}
impl Blueprint {
    pub fn new(mut reader: BlueprintFileReader) -> Result<Blueprint, RepackError> {
        let mut lang = Blueprint {
            id: String::new(),
            name: String::new(),
            links: HashMap::new(),
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
                        if !snip.autoclose {
                            while let Some(in_block) = reader.next() {
                                match &in_block {
                                    FlyToken::Close(det) if *det == snip.main_token => {
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
                            if let FlyToken::Literal(val) = t {
                                literal_string_value.push_str(val);
                            }
                        }

                        lang.utilities
                            .insert((main, secondary), literal_string_value);
                    }
                    SnippetMainTokenName::Link => {
                        let mut participating_tokens = Vec::new();
                        if !snip.autoclose {
                            while let Some(in_block) = reader.next() {
                                match &in_block {
                                    FlyToken::Close(det) if *det == snip.main_token => {
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
                            if let FlyToken::Literal(val) = t {
                                literal_string_value.push_str(val);
                            }
                        }
                        lang.links
                            .insert(snip.secondary_token.to_string(), literal_string_value);
                    }
                    _ => lang.tokens.push(next),
                }
            } else {
                lang.tokens.push(next);
            }
        }

        // Trim extra chars
        let mut i = 0;
        while i + 1 < lang.tokens.len() {
            match &lang.tokens[i + 1] {
                FlyToken::Snippet(snip) => {
                    let autoclose = snip.autoclose;
                    match &mut lang.tokens[i] {
                        FlyToken::Literal(lit) => {
                            if !autoclose {
                                while lit.ends_with('\n') || lit.ends_with('\t') {
                                    lit.pop();
                                }
                            }
                        }
                        _ => {}
                    }
                }
                FlyToken::Close(_) => match &mut lang.tokens[i] {
                    FlyToken::Literal(lit) => {
                        while lit.ends_with('\n') || lit.ends_with('\t') {
                            lit.pop();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
            i += 1;
        }

        if let Some(id) = lang
            .utilities
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Id))
        {
            lang.id = id.clone();
        }
        if let Some(name) = lang
            .utilities
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Name))
        {
            lang.name = name.clone();
        }

        Ok(lang)
    }
}
