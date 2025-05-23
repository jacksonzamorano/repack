use crate::outputs::OutputBuilder;

use super::{DescriptionBuilder, PostgresBuilder};

#[derive(Debug)]
pub enum OutputProfile {
    Description,
    PostgresInit,
}

impl OutputProfile {
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "description" => Some(OutputProfile::Description),
            "postgres" => Some(OutputProfile::PostgresInit),
            _ => None,
        }
    }
    pub fn builder(&self) -> Box<dyn OutputBuilder> {
        match self {
            OutputProfile::Description => Box::new(DescriptionBuilder {}),
            OutputProfile::PostgresInit => Box::new(PostgresBuilder {}),
        }
    }
}
