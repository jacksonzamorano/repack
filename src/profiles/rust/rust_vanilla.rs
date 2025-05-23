use std::collections::HashSet;

use crate::outputs::OutputBuilder;

use super::type_to_rust;

pub struct RustBuilder;

impl OutputBuilder for RustBuilder {
    fn build(
        &self,
        description: &mut crate::outputs::OutputDescription,
    ) -> Result<(), crate::outputs::OutputBuilderError> {
        let mut imports: HashSet<String> = HashSet::new();
        let mut output = String::new();
        for object in description.objects() {
            output.push_str(&format!("pub struct {} {{\n", object.name));
            for field in &object.fields {
                let resolved_typ = field.resolve_type(object, description)?;
                let rust_type = type_to_rust(resolved_typ).ok_or(
                    crate::outputs::OutputBuilderError::UnsupportedFieldType(
                        crate::outputs::OutputBuilderFieldError::new(object, field),
                    ),
                )?;
                if resolved_typ == &crate::syntax::FieldType::DateTime {
                    imports.insert("use chrono::NaiveDateTime;".to_string());
                }
                let optional = if field.optional { "Option<" } else { "" };
                let arr = if field.commands.contains(&crate::syntax::FieldCommand::Many) {
                    "Vec<"
                } else {
                    ""
                };
                let optional_close = if field.optional { ">" } else { "" };
                let arr_close = if field.commands.contains(&crate::syntax::FieldCommand::Many) {
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
