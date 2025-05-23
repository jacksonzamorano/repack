use crate::outputs::OutputBuilder;

use super::{DescriptionBuilder, PostgresBuilder, RustBuilder, TypescriptClassBuilder, TypescriptInterfaceBuilder};

#[derive(Debug)]
pub enum OutputProfile {
    Description,
    PostgresInit,
    TypescriptClass,
    TypescriptInterface,
    Rust
}

impl OutputProfile {
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "description" => Some(OutputProfile::Description),
            "postgres" => Some(OutputProfile::PostgresInit),
            "typescript_class" => Some(OutputProfile::TypescriptClass),
            "typescript_interface" => Some(OutputProfile::TypescriptInterface),
            "rust" => Some(OutputProfile::Rust),
            _ => None,
        }
    }
    pub fn builder(&self) -> Box<dyn OutputBuilder> {
        match self {
            OutputProfile::Description => Box::new(DescriptionBuilder {}),
            OutputProfile::PostgresInit => Box::new(PostgresBuilder {}),
            OutputProfile::TypescriptClass => Box::new(TypescriptClassBuilder {}),
            OutputProfile::TypescriptInterface => Box::new(TypescriptInterfaceBuilder {}),
            OutputProfile::Rust => Box::new(RustBuilder {}),
        }
    }
}
