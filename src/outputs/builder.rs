use super::OutputDescription;

pub enum OutputBuilderError {
    CannotOpenFile,
}

impl OutputBuilderError {
    pub fn description(&self) -> String {
        match self {
            OutputBuilderError::CannotOpenFile => "Cannot open file.".to_string(),
        }
    }
}

pub trait OutputBuilder {
    fn build(&self, description: &mut OutputDescription) -> Result<(), OutputBuilderError>;
}