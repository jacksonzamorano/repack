use std::collections::HashSet;

use crate::{
    outputs::OutputBuilder,
    syntax::{FieldType, RepackError, RepackErrorKind},
};

use super::type_to_rust;

pub struct RustBuilder;

impl OutputBuilder for RustBuilder {
    fn build(
        &self,
        description: &mut crate::outputs::OutputDescription,
    ) -> Result<(), RepackError> {
        let mut imports: HashSet<String> = HashSet::new();
        let mut output = String::new();
        for object in description.objects() {
            output.push_str(&format!("pub struct {} {{\n", object.name));
            for field in &object.fields {
                let rust_type =
                    type_to_rust(field.field_type()).ok_or(RepackError::from_lang_with_msg(
                        RepackErrorKind::UnsupportedFieldType,
                        description.output,
                        field.field_type().to_string(),
                    ))?;
                if *field.field_type() == FieldType::DateTime {
                    imports.insert("use chrono::NaiveDateTime;".to_string());
                }
                let optional = if field.optional { "Option<" } else { "" };
                let arr = if field.array {
                    "Vec<"
                } else {
                    ""
                };
                let optional_close = if field.optional { ">" } else { "" };
                let arr_close = if field.array {
                    ">"
                } else {
                    ""
                };
                output.push_str(&format!(
                    "\t{}: {}{}{}{}{},\n",
                    field.name, optional, arr, rust_type, optional_close, arr_close
                ));
            }
            output.push_str("}\n\n");
        }
        description.append(
            "model.rs",
            format!("{}\n\n", imports.into_iter().collect::<Vec<_>>().join("\n")),
        );
        description.append("model.rs", output);
        Ok(())
    }
}
