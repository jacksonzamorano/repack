use crate::outputs::OutputBuilder;

use super::{DescriptionBuilder, PostgresBuilder, TypescriptClassBuilder, TypescriptInterfaceBuilder};

#[derive(Debug)]
pub enum OutputProfile {
    Description,
    PostgresInit,
    TypescriptClass,
    TypescriptInterface
}

impl OutputProfile {
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "description" => Some(OutputProfile::Description),
            "postgres" => Some(OutputProfile::PostgresInit),
            "typescript_class" => Some(OutputProfile::TypescriptClass),
            "typescript_interface" => Some(OutputProfile::TypescriptInterface),
            _ => None,
        }
    }
    pub fn builder(&self) -> Box<dyn OutputBuilder> {
        match self {
            OutputProfile::Description => Box::new(DescriptionBuilder {}),
            OutputProfile::PostgresInit => Box::new(PostgresBuilder {}),
            OutputProfile::TypescriptClass => Box::new(TypescriptClassBuilder {}),
            OutputProfile::TypescriptInterface => Box::new(TypescriptInterfaceBuilder {}),
        }
    }
}
