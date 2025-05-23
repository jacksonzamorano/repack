use crate::{outputs::{OutputBuilder, OutputBuilderError, OutputDescription}, syntax::Output};

pub struct DescriptionBuilder;

impl OutputBuilder for DescriptionBuilder {
    fn build(&self, output: &Output, description: &mut OutputDescription) -> Result<(), OutputBuilderError> {
        // Implement the logic to build the output description here
        Ok(())
    }
}