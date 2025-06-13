use std::collections::HashSet;

use crate::{
    outputs::OutputBuilder,
    syntax::{CustomFieldType, RepackError, RepackErrorKind},
};

use super::{enum_type, make_index, type_to_ts};

pub struct TypescriptClassBuilder;

impl OutputBuilder for TypescriptClassBuilder {
    fn build(
        &self,
        description: &mut crate::outputs::OutputDescription,
    ) -> Result<(), RepackError> {
        for enm in description.enums() {
            let file_name = format!("{}.ts", enm.name);
            description.append(&file_name, enum_type(enm));
            if make_index(description) {
                description.append(
                    "index.ts",
                    format!("export type {{ {} }} from './{}';\n", enm.name, enm.name),
                );
            }
        }
        for object in description.objects() {
            let mut imports: Vec<String> = Vec::new();
            let mut import_as_types: HashSet<String> = HashSet::new();
            let mut output = String::new();
            output.push_str(&format!("export class {} {{\n", object.name));
            for field in &object.fields {
                let ts_type =
                    type_to_ts(field.field_type()).ok_or(RepackError::from_lang_with_msg(
                        RepackErrorKind::UnsupportedFieldType,
                        description.output,
                        field.field_type().to_string(),
                    ))?;
                let optional = if field.optional { "?" } else { "" };
                let arr = if field.array { "[]" } else { "" };
                if let crate::syntax::FieldType::Custom(name, typ) = &field.field_type() {
                    match typ {
                        CustomFieldType::Enum => {
                            import_as_types.insert(name.clone());
                        }
                        _ => {}
                    };
                    if !imports.contains(name) {
                        imports.push(name.clone());
                    }
                }

                output.push_str(&format!(
                    "\t{}{}: {}{};\n",
                    field.name, optional, ts_type, arr
                ));
            }
            output.push_str("}\n");
            let file_name = format!("{}.ts", object.name);
            for import in imports {
                description.append(
                    &file_name,
                    format!(
                        "import{} {{ {} }} from './{}';\n",
                        if import_as_types.contains(&import) {
                            " type"
                        } else {
                            ""
                        },
                        import,
                        import
                    ),
                );
            }
            description.append(&file_name, output);
            if make_index(description) {
                description.append(
                    "index.ts",
                    format!("export {{ {} }} from './{}';\n", object.name, object.name),
                );
            }
        }

        Ok(())
    }
}
