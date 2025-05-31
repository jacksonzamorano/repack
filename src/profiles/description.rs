use crate::{outputs::{OutputBuilder, OutputDescription}, syntax::RepackError};

const DESCRIPTION_FILE: &str = "description.txt";
pub struct DescriptionBuilder;

impl OutputBuilder for DescriptionBuilder {
    fn build(&self, description: &mut OutputDescription) -> Result<(), RepackError> {
        for object in description.objects() {
            description.append(DESCRIPTION_FILE, format!("{}\n", object.name));
            for field in &object.fields {
                description.append(DESCRIPTION_FILE, format!("\t- {}: {}\n", field.name, field.field_type()));
                if field.optional {
                    description.append(DESCRIPTION_FILE, "\t\t- optional\n".to_string());
                }
                // if description.bool("print_functions", true) {
                //     for function in &field.functions {
                //         let args = function.args.iter().map(|x| format!("\t\t\t- '{}'", x)).collect::<Vec<_>>().join("\n");
                //         description.append(DESCRIPTION_FILE,
                //             format!("\t\t- Function: '{}'\n{}\n", function.name, args)
                //         );
                //     }
                // }
            }
            description.append(DESCRIPTION_FILE, "---\n\n".to_string());
        }
        Ok(())
    }
}
