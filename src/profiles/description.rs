use crate::outputs::{OutputBuilder, OutputBuilderError, OutputDescription};

const DESCRIPTION_FILE: &str = "description.txt";
pub struct DescriptionBuilder;

impl OutputBuilder for DescriptionBuilder {
    fn build(&self, description: &mut OutputDescription) -> Result<(), OutputBuilderError> {
        for object in description.objects() {
            description.append(DESCRIPTION_FILE, format!("{}\n", object.name));
            for field in &object.fields {
                description.append(DESCRIPTION_FILE, format!("\t- {}: {}\n", field.name, field.field_type().to_string()));
                if field.optional {
                    description.append(DESCRIPTION_FILE, "\t\t- optional\n".to_string());
                }
                if description.bool("print_commands", true) {
                    for commands in &field.commands {
                        description.append(
                            DESCRIPTION_FILE,
                            format!("\t\t- {}\n", commands.string()),
                        );
                    }
                }
            }
            description.append(DESCRIPTION_FILE, "---\n\n".to_string());
        }
        Ok(())
    }
}
