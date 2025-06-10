use crate::outputs::OutputBuilder;

use super::{
    DescriptionBuilder, PostgresBuilder, RustBuilder, RustTuskBuilder, TypescriptClassBuilder,
    TypescriptDrizzleBuilder, TypescriptInterfaceBuilder,
};

#[derive(Debug)]
pub enum OutputProfile {
    Description,
    PostgresInit,
    TypescriptClass,
    TypescriptInterface,
    TypescriptDrizzle,
    Rust,
    RustTusk,
}

impl OutputProfile {
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword {
            "description" => OutputProfile::Description,
            "postgres" => OutputProfile::PostgresInit,
            "typescript_class" => OutputProfile::TypescriptClass,
            "typescript_interface" => OutputProfile::TypescriptInterface,
            "typescript_drizzle" => OutputProfile::TypescriptDrizzle,
            "rust" => OutputProfile::Rust,
            "rust_tusk" => OutputProfile::RustTusk,
            _ => return None,
        })
    }
    pub fn builder(&self) -> Box<dyn OutputBuilder> {
        match self {
            OutputProfile::Description => Box::new(DescriptionBuilder {}),
            OutputProfile::PostgresInit => Box::new(PostgresBuilder {}),
            OutputProfile::TypescriptClass => Box::new(TypescriptClassBuilder {}),
            OutputProfile::TypescriptInterface => Box::new(TypescriptInterfaceBuilder {}),
            Self::TypescriptDrizzle => Box::new(TypescriptDrizzleBuilder {}),
            Self::RustTusk => Box::new(RustTuskBuilder {}),
            OutputProfile::Rust => Box::new(RustBuilder {}),
        }
    }
}
