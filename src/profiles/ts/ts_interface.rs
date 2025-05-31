use crate::{outputs::OutputBuilder, syntax::{FieldType, RepackError, RepackErrorKind}};

use super::{make_index, type_to_ts};

pub struct TypescriptInterfaceBuilder;

impl OutputBuilder for TypescriptInterfaceBuilder {
    fn build(&self, description: &mut crate::outputs::OutputDescription) -> Result<(), RepackError> {
        for object in description.objects() {
            let mut imports: Vec<String> = Vec::new();
            let mut output = String::new();
            output.push_str(&format!("export interface {} {{\n", object.name));
            for field in &object.fields {
                let ts_type = type_to_ts(field.field_type()).ok_or(
                    RepackError::from_lang_with_msg(
                        RepackErrorKind::UnsupportedFieldType,
                        description.output,
                        field.field_type().to_string(),
                    )
                )?;
                let optional = if field.optional {
                    "?"
                } else {
                    ""
                };
                let arr = if field.array {
                    "[]"
                } else {
                    ""
                };
                if let FieldType::Custom(name) = field.field_type() {
                    if !imports.contains(name) {
                        imports.push(name.clone());
                    }
                }

                output.push_str(&format!("\t{}{}: {}{};\n", field.name, optional, ts_type, arr));
            }
            output.push_str("}\n");
            let file_name = format!("{}.ts", object.name);
            for import in imports {
                description.append(&file_name, format!("import type {{ {} }} from './{}';\n", import, import));
            }
            description.append(&file_name, output);
            if make_index(description) {
                description.append("index.ts", format!("export type {{ {} }} from './{}';\n", object.name, object.name));
            }
        }

        Ok(())
    }
}
