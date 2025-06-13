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
        for enm in description.enums() {
            output.push_str(&format!(
                "pub enum {} {{\n\
                    {}\n\
                    }}\n\n\
                    impl {} {{\n\
                        fn from_string(val: &str) -> Option<{}> {{\n\
                            match val {{\n\
                                {}\n\
                                _ => None
                            }}\n\
                        }}\n\
                        fn to_string(&self) -> &'static str {{\n\
                            match self {{\n\
                                {}\n\
                            }}\n\
                        }}\n\
                    }}\n",
                enm.name,
                enm.options
                    .iter()
                    .map(|x| format!("\t{}", x))
                    .collect::<Vec<_>>()
                    .join(",\n"),
                enm.name,
                enm.name,
                enm.options
                    .iter()
                    .map(|x| format!("\t\t\t\"{}\" => Some(Self::{})", x, x))
                    .collect::<Vec<_>>()
                    .join(",\n"),
                enm.options
                    .iter()
                    .map(|x| format!("\t\t\tSelf::{} => \"{}\"", x, x))
                    .collect::<Vec<_>>()
                    .join(",\n")
            ));
        }
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
                    imports.insert("use chrono::{DateTime, Utc};".to_string());
                } else if *field.field_type() == FieldType::Uuid {
                    imports.insert("use uuid::Uuid;".to_string());
                }
                let optional = if field.optional { "Option<" } else { "" };
                let arr = if field.array { "Vec<" } else { "" };
                let optional_close = if field.optional { ">" } else { "" };
                let arr_close = if field.array { ">" } else { "" };
                output.push_str(&format!(
                    "\tpub {}: {}{}{}{}{},\n",
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
