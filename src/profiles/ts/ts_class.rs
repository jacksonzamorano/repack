use crate::{outputs::{OutputBuilder, OutputBuilderError}, syntax::FieldCommand};

use super::{make_index, type_to_ts};

pub struct TypescriptClassBuilder;

impl OutputBuilder for TypescriptClassBuilder {
    fn build(&self, description: &mut crate::outputs::OutputDescription) -> Result<(), OutputBuilderError> {
        for object in description.objects() {
            let mut imports: Vec<String> = Vec::new();
            let mut output = String::new();
            output.push_str(&format!("export class {} {{\n", object.name));
            for field in &object.fields {
                let resolved_typ = field.resolve_type(object, description)?;
                let ts_type = type_to_ts(resolved_typ).ok_or(
                    OutputBuilderError::UnsupportedFieldType(crate::outputs::OutputBuilderFieldError::new(
                        object, field,
                    )),
                )?;
                let optional = if field.optional {
                    "?"
                } else {
                    ""
                };
                let arr = if field.commands.contains(&FieldCommand::Many) {
                    "[]"
                } else {
                    ""
                };
                if let crate::syntax::FieldType::Custom(name) = resolved_typ {
                    if !imports.contains(name) {
                        imports.push(name.clone());
                    }
                }

                output.push_str(&format!("\t{}{}: {}{};\n", field.name, optional, ts_type, arr));
            }
            output.push_str("}\n");
            let file_name = format!("{}.ts", object.name);
            for import in imports {
                description.append(&file_name, format!("import {{ {} }} from './{}';\n", import, import));
            }
            description.append(&file_name, output);
            if make_index(description) {
                description.append("index.ts", format!("export {{ {} }} from './{}';\n", object.name, object.name));
            }
        }

        Ok(())
    }
}