use std::collections::HashSet;

use crate::{
    outputs::OutputBuilder,
    syntax::{
        FieldReferenceKind, FieldType, FunctionName, FunctionNamespace, ObjectType, RepackError,
    },
};

pub struct TypescriptDrizzleBuilder;

fn drizzle_type(typ: &FieldType) -> Option<(&'static str, &'static str)> {
    Some(match typ {
        FieldType::Int32 => ("integer", "integer()"),
        FieldType::Int64 => ("bigint", "bigint({ mode: 'number' })"),
        FieldType::Float64 => ("doublePrecision", "doublePrecision()"),
        FieldType::DateTime => ("timestamp", "timestamp({ withTimezone: true })"),
        FieldType::String => ("varchar", "varchar()"),
        FieldType::Boolean => ("boolean", "boolean()"),
        _ => return None,
    })
}

const F_NAME: &str = "schema.ts";
const PRIMARY_KEY: &str = "primaryKey";

impl OutputBuilder for TypescriptDrizzleBuilder {
    fn build(
        &self,
        description: &mut crate::outputs::OutputDescription,
    ) -> Result<(), crate::syntax::RepackError> {
        let mut tables = Vec::<String>::new();

        let table_type = "pgTable".to_string();
        let mut drizzle_imports = HashSet::<String>::new();
        drizzle_imports.insert(table_type.clone());

        for obj in description.objects() {
            if obj.object_type != ObjectType::Record || obj.inherits.is_some() {
                return Err(RepackError::from_lang_with_obj(
                    crate::syntax::RepackErrorKind::UnsupportedObjectType,
                    description.output,
                    obj,
                ));
            }

            let mut fields: Vec<String> = Vec::new();

            for f in &obj.fields {
                let typ = drizzle_type(f.field_type()).ok_or(RepackError::from_lang_with_msg(
                    crate::syntax::RepackErrorKind::UnsupportedFieldType,
                    description.output,
                    f.field_type().to_string(),
                ))?;

                let mut modifiers: Vec<String> = Vec::new();
                if !f.optional {
                    modifiers.push("notNull()".to_string());
                };
                for function in &f.functions_in_namespace(FunctionNamespace::Database) {
                    match function.name {
                        FunctionName::PrimaryKey => {
                            modifiers.push(format!("{}()", PRIMARY_KEY));
                        }
                        FunctionName::Identity => {
                            modifiers.push("generatedAlwaysAsIdentity()".to_string())
                        }
                        _ => {}
                    }
                }
                if let FieldReferenceKind::FieldType(table_ref) = &f.location.reference {
                    let ref_obj = description.object_by_name(table_ref)?;
                    let ref_field = description.field_by_name(ref_obj, &f.location.name)?;
                    modifiers.push(format!(
                        "references(() => {}.{})",
                        ref_obj.name, ref_field.name
                    ));
                }

                let modifier_prefix = if modifiers.is_empty() { "" } else { "." };
                fields.push(format!(
                    "\t{}: {}{}{}",
                    f.name,
                    typ.1,
                    modifier_prefix,
                    modifiers.join(".")
                ));
                drizzle_imports.insert(typ.0.to_string());
            }

            let def = format!(
                "export const {} = {}(\"{}\", {{\n{}\n}})\n\n",
                obj.name,
                table_type,
                obj.table(),
                fields.join(",\n"),
            );
            tables.push(def);
        }

        description.append(
            F_NAME,
            format!(
                "import {{ {} }} from 'drizzle-orm/pg-core'\n\n",
                drizzle_imports.into_iter().collect::<Vec<_>>().join(", ")
            ),
        );

        description.append(F_NAME, tables.join("\n\n"));
        Ok(())
    }
}
