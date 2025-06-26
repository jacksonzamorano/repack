use super::SnippetDetails;
use crate::{
    blueprint::{BlueprintFileReader, FlyToken},
    syntax::{CoreType, FieldType},
};
use std::collections::HashMap;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SnippetMainTokenName {
    Meta,
    If,
    Each,
    TypeDef,
    Variable(String),
}
impl SnippetMainTokenName {
    fn from_string(val: &str) -> SnippetMainTokenName {
        match val {
            "meta" => Self::Meta,
            "if" => Self::If,
            "each" => Self::Each,
            "type" => Self::TypeDef,
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
        if let Some(ct) = CoreType::from_string(&val) {
            return Self::from_type(&FieldType::Core(ct))
        }
        match val {
            "id" => Self::Id,
            "name" => Self::Name,
            "object" => Self::Object,
            _ => Self::Arbitrary(val.to_string()),
        }
    }
    pub fn from_type(typ: &FieldType) -> SnippetSecondaryTokenName {
        match typ {
            FieldType::Core(core) => match core {
                CoreType::Uuid => Self::Uuid,
                CoreType::Int64 => Self::Int64,
                CoreType::Int32 => Self::Int32,
                CoreType::String => Self::String,
                CoreType::Float64 => Self::Float64,
                CoreType::Boolean => Self::Boolean,
                CoreType::DateTime => Self::DateTime,
            },
            FieldType::Custom(name, _) => {
                return SnippetSecondaryTokenName::Arbitrary(name.clone());
            }
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

pub struct SnippetReference<'a> {
    pub details: &'a SnippetDetails,
    pub contents: &'a [FlyToken],
}
impl<'a> SnippetReference<'a> {
    pub fn from_content(content: &'a SectionContent) -> Self {
        Self {
            details: &content.details,
            contents: &content.contents,
        }
    }
    pub fn slice_content(content: &'a SectionContent, starting_at: usize) -> Option<Self> {
        let content_details = match &content.contents[starting_at] {
            FlyToken::Snippet(snip_details) => snip_details,
            _ => return None,
        };
        Some(Self {
            details: &content_details,
            contents: &[],
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Blueprint {
    pub id: String,
    pub name: String,
    pub sections: HashMap<SnippetIdentifier, SectionContent>,
}
impl Blueprint {
    pub fn new(mut reader: BlueprintFileReader) -> Result<Blueprint, BlueprintError> {
        let mut lang = Blueprint {
            id: String::new(),
            name: String::new(),
            sections: HashMap::new(),
        };

        loop {
            let Some(next) = reader.next() else {
                break;
            };
            if let FlyToken::Snippet(snip) = next {
                let (main, secondary) = (
                    SnippetMainTokenName::from_string(&snip.main_token),
                    SnippetSecondaryTokenName::from_string(&snip.secondary_token),
                );
                let mut content = SectionContent {
                    details: snip,
                    contents: Vec::new(),
                    literal_string_value: String::new(),
                };
                let mut embed_count = 1;
                if !content.details.is_ended {
                    while let Some(in_block) = reader.next() {
                        match &in_block {
                            FlyToken::SnippetEnd(end_name)
                                if *end_name == content.details.main_token =>
                            {
                                embed_count -= 1;
                                if embed_count == 0 {
                                    break;
                                }
                            }
                            FlyToken::Snippet(embedded)
                                if embedded.main_token == content.details.main_token =>
                            {
                                embed_count += 1;
                                content.contents.push(in_block);
                            }
                            _ => content.contents.push(in_block),
                        }
                    }
                }
                let mut content_literal_value = String::new();
                content_literal_value.push_str(&content.details.contents);
                for c in &content.contents {
                    if let FlyToken::Literal(lit) = c {
                        content_literal_value.push_str(lit)
                    }
                }
                content.literal_string_value = content_literal_value;
                lang.sections.insert((main, secondary), content);
            }
        }

        if let Some(id) = lang
            .sections
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Id))
        {
            lang.id = id.literal_string_value.clone();
        }
        if let Some(name) = lang
            .sections
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Name))
        {
            lang.name = name.literal_string_value.clone();
        }

        dbg!(&lang);

        Ok(lang)
    }
}
