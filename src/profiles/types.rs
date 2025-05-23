use crate::outputs::OutputBuilder;

use super::DescriptionBuilder;

#[derive(Debug)]
pub enum OutputProfile {
    Description,
}

impl OutputProfile {
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "description" => Some(OutputProfile::Description),
            _ => None,
        }
    }
    pub fn builder(&self) -> impl OutputBuilder {
        match self {
            OutputProfile::Description => DescriptionBuilder{},
        }
    }
}