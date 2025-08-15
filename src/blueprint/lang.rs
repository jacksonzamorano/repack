use super::BlueprintSnippetDetails;
use crate::{
    blueprint::{BlueprintFileReader, BlueprintToken},
    syntax::{CoreType, RepackError},
};
use std::collections::HashMap;

/// Main blueprint template tokens that control template flow and content generation.
///
/// These tokens define the primary template constructs available in blueprint files.
/// They control iteration, conditionals, file output, and variable substitution.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SnippetMainTokenName {
    Meta,
    File,
    If,
    Ifn,
    Each,
    Eachr,
    TypeDef,
    Func,
    Nfunc,
    Join,
    Ref,
    Link,
    Import,
    Trim,
    PlaceImports,
    Break,
    Exec,
    Increment,
    Snippet,
    Render,
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
            "define" => Self::TypeDef,
            "func" => Self::Func,
            "nfunc" => Self::Nfunc,
            "join" => Self::Join,
            "ref" => Self::Ref,
            "file" => Self::File,
            "link" => Self::Link,
            "import" => Self::Import,
            "imports" => Self::PlaceImports,
            "br" => Self::Break,
            "exec" => Self::Exec,
            "increment" => Self::Increment,
            "snippet" => Self::Snippet,
            "render" => Self::Render,
            "trim" => Self::Trim,
            _ => Self::Variable(val.to_string()),
        }
    }
}
/// Secondary blueprint template tokens that provide context and type information.
///
/// These tokens are used within primary template constructs to specify details
/// like object types, field types, metadata keys, and template parameters.
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum SnippetSecondaryTokenName {
    // Define
    Id,
    Name,
    Kind,
    Struct,
    Field,
    Enum,
    Case,
    Debug,

    // TypeDef
    String,
    Int32,
    Int64,
    Float64,
    Uuid,
    DateTime,
    Boolean,
    Bytes,

    Join,
    Arg,
    Query,

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
            "kind" => Self::Kind,
            "struct" => Self::Struct,
            "field" => Self::Field,
            "enum" => Self::Enum,
            "case" => Self::Case,
            "join" => Self::Join,
            "arg" => Self::Arg,
            "debug" => Self::Debug,
            "query" => Self::Query,
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
            CoreType::Bytes => Self::Bytes,
        }
    }
}
type SnippetIdentifier = (SnippetMainTokenName, SnippetSecondaryTokenName);

#[derive(Debug)]
pub struct SnippetReference<'a> {
    pub details: &'a BlueprintSnippetDetails,
    pub contents: &'a [BlueprintToken],
}
impl<'a> SnippetReference<'a> {
    pub fn main_token(&self) -> SnippetMainTokenName {
        SnippetMainTokenName::from_string(&self.details.main_token)
    }
    pub fn secondary_token(&self) -> SnippetSecondaryTokenName {
        SnippetSecondaryTokenName::from_string(&self.details.secondary_token)
    }
}

/// Defines the category of blueprint and its intended purpose.
///
/// BlueprintKind determines which operations will use this blueprint and
/// what type of output it generates.
#[derive(Debug)]
pub enum BlueprintKind {
    /// Generates source code files (structs, interfaces, schemas, etc.)
    /// Used by the build command to create language-specific code
    Code,
    /// Generates configuration files (env files, docker configs, etc.)
    /// Used by the configure command for environment-specific deployments
    Configure,
    /// Generates documentation files (markdown, HTML, etc.)
    /// Used by the document command to create human-readable docs
    Document,
}
impl BlueprintKind {
    pub fn from_string(x: &str) -> BlueprintKind {
        match x {
            "code" => Self::Code,
            "configure" => Self::Configure,
            "document" => Self::Document,
            _ => panic!("Unknown blueprint kind {x}"),
        }
    }
}

/// Represents a complete blueprint definition for code generation.
///
/// Blueprint contains all the template logic, type mappings, and metadata needed
/// to generate code for a specific target language or format. Blueprints are loaded
/// from template files and used by the renderer to produce output files.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Blueprint {
    /// Unique identifier for this blueprint (e.g., "rust", "typescript")
    pub id: String,
    /// Human-readable name for this blueprint
    pub name: String,
    /// The blueprint category (Code, Document, or Configure)
    pub kind: BlueprintKind,
    /// Import statements and dependencies needed for generated code
    pub links: HashMap<String, String>,
    /// Type mappings from repack types to target language types
    pub utilities: HashMap<SnippetIdentifier, String>,
    /// Parsed template tokens that define the generation logic
    pub tokens: Vec<BlueprintToken>,
    /// Named code snippets for reuse within the template
    pub snippets: HashMap<String, String>,
}
impl Blueprint {
    pub fn new(mut reader: BlueprintFileReader) -> Result<Blueprint, RepackError> {
        let mut lang = Blueprint {
            id: String::new(),
            name: String::new(),
            kind: BlueprintKind::Code,
            links: HashMap::new(),
            utilities: HashMap::new(),
            tokens: Vec::new(),
            snippets: HashMap::new(),
        };

        loop {
            let Some(next) = reader.next() else {
                break;
            };
            if let BlueprintToken::Snippet(snip) = &next {
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
                                    BlueprintToken::Close(det) if *det == snip.main_token => {
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
                            if let BlueprintToken::Literal(val) = t {
                                literal_string_value.push_str(val);
                            }
                        }

                        lang.utilities
                            .insert((main, secondary), literal_string_value);
                    }
                    SnippetMainTokenName::Snippet => {
                        let mut participating_tokens = Vec::new();
                        if !snip.autoclose {
                            while let Some(in_block) = reader.next() {
                                match &in_block {
                                    BlueprintToken::Close(det) if *det == snip.main_token => {
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
                            if let BlueprintToken::Literal(val) = t {
                                literal_string_value.push_str(val);
                            }
                        }
                        lang.snippets
                            .insert(snip.secondary_token.to_string(), literal_string_value);
                    }
                    SnippetMainTokenName::Link => {
                        let mut participating_tokens = Vec::new();
                        if !snip.autoclose {
                            while let Some(in_block) = reader.next() {
                                match &in_block {
                                    BlueprintToken::Close(det) if *det == snip.main_token => {
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
                            if let BlueprintToken::Literal(val) = t {
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
                BlueprintToken::Snippet(snip) => {
                    let autoclose = snip.autoclose;
                    if let BlueprintToken::Literal(lit) = &mut lang.tokens[i] {
                        if !autoclose {
                            while lit.ends_with('\n') || lit.ends_with('\t') {
                                lit.pop();
                            }
                        }
                    }
                }
                BlueprintToken::Close(_) => {
                    if let BlueprintToken::Literal(lit) = &mut lang.tokens[i] {
                        while lit.ends_with('\n') || lit.ends_with('\t') {
                            lit.pop();
                        }
                    }
                }
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

        if let Some(kind) = lang
            .utilities
            .get(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Kind))
        {
            lang.kind = BlueprintKind::from_string(kind)
        }

        if lang
            .utilities
            .contains_key(&(SnippetMainTokenName::Meta, SnippetSecondaryTokenName::Debug))
        {
            dbg!(&lang.tokens);
        }

        Ok(lang)
    }
}
