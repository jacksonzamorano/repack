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
}