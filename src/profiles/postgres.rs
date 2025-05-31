use std::collections::{HashMap, hash_map::Entry};

use crate::{
    outputs::{OutputBuilder, OutputDescription},
    syntax::{
        FieldFunctionName, FieldReferenceKind, FieldType, FunctionNamespace, ObjectFunctionName,
        ObjectType, RepackError, RepackErrorKind,
    },
};

fn type_to_psql(field_type: &FieldType) -> Option<String> {
    match field_type {
        FieldType::Boolean => Some("BOOLEAN".to_string()),
        FieldType::Int32 => Some("INT4".to_string()),
        FieldType::Int64 => Some("INT8".to_string()),
        FieldType::String => Some("TEXT".to_string()),
        FieldType::Float64 => Some("FLOAT8".to_string()),
        FieldType::DateTime => Some("TIMESTAMPTZ".to_string()),
        _ => None,
    }
}

pub struct PostgresBuilder;

impl OutputBuilder for PostgresBuilder {
    fn build(&self, description: &mut OutputDescription) -> Result<(), RepackError> {
        let mut sql = String::new();
        sql.push_str("BEGIN;\n\n");
        for object in description.objects().iter().rev() {
            if object.object_type != ObjectType::Record {
                return Err(RepackError::from_lang_with_obj(
                    RepackErrorKind::UnsupportedObjectType,
                    description.output,
                    object,
                ));
            }
            if object.inherits.is_none() {
                sql.push_str("DROP TABLE IF EXISTS ");
                sql.push_str(object.table());
            } else {
                sql.push_str("DROP VIEW IF EXISTS ");
                sql.push_str(&object.name);
            }
            sql.push_str(";\n");
        }

        for object in description.objects() {
            if object.object_type != ObjectType::Record {
                return Err(RepackError::from_lang(
                    RepackErrorKind::CannotInherit,
                    description.output,
                ));
            }
            if object.inherits.is_none() {
                // Root object = table
                let mut fields = Vec::<String>::new();
                let mut constraints = Vec::<String>::new();
                let mut indicies = Vec::<String>::new();
                sql.push_str(&format!("CREATE TABLE {} (\n", object.table()));
                for field in &object.fields {
                    let nullability = if field.optional { "" } else { " NOT NULL" };
                    let typ =
                        type_to_psql(field.field_type()).ok_or(RepackError::from_lang_with_msg(
                            RepackErrorKind::UnsupportedFieldType,
                            description.output,
                            field.field_type().to_string(),
                        ))?;
                    if let FieldReferenceKind::FieldType(table_ref) = &field.location.reference {
                        let ref_obj = description.object_by_name(table_ref)?;
                        let ref_field = description.field_by_name(ref_obj, &field.location.name)?;
                        let cascade = if field
                            .functions_in_namespace(FunctionNamespace::Database)
                            .iter()
                            .any(|x| x.name == FieldFunctionName::Cascade)
                        {
                            " ON DELETE CASCADE"
                        } else {
                            ""
                        };
                        constraints.push(format!(
                            "\tFOREIGN KEY ({}) REFERENCES {}({}){}",
                            field.name,
                            ref_obj.table(),
                            ref_field.name,
                            cascade
                        ));
                    }
                    let mut modifiers: Vec<String> = Vec::new();
                    for f in &field.functions_in_namespace(FunctionNamespace::Database) {
                        match f.name {
                            FieldFunctionName::Default => {
                                let arg = f.arg(description.output, object, field, 0)?;
                                modifiers.push(format!("DEFAULT {}", arg));
                            }
                            FieldFunctionName::Identity => {
                                modifiers.push("GENERATED ALWAYS AS IDENTITY".to_string());
                            }
                            FieldFunctionName::Generated => {
                                let arg = f.arg(description.output, object, field, 0)?;
                                modifiers.push(format!("GENERATED ALWAYS AS ({})", arg));
                            }
                            FieldFunctionName::Unique => {
                                modifiers.push("UNIQUE".to_string());
                            }
                            FieldFunctionName::PrimaryKey => {
                                modifiers.push("PRIMARY KEY".to_string());
                            }
                            _ => {}
                        }
                    }
                    fields.push(format!(
                        "\t{} {}{}{}{}",
                        field.name,
                        typ,
                        nullability,
                        if !modifiers.is_empty() { " " } else { "" },
                        modifiers.join(" ")
                    ));
                }
                for o in &object.functions_in_namespace(FunctionNamespace::Database) {
                    if o.name == ObjectFunctionName::Index {
                        let column_name = o.arg(description.output, object, 0)?;
                        indicies.push(format!(
                            "CREATE INDEX idx_{} ON {} ({});",
                            column_name,
                            object.table(),
                            column_name
                        ));
                    }
                }
                sql.push_str(&fields.join(",\n"));
                if !constraints.is_empty() {
                    sql.push_str(",\n");
                    sql.push_str(&constraints.join(",\n"));
                }
                sql.push('\n');
                sql.push_str(");\n");
                sql.push_str(&indicies.join("\n"));
                sql.push_str("\n\n");
            } else {
                // Make view
                let mut fields = Vec::<String>::new();
                let mut joins = HashMap::<String, String>::new();
                for field in &object.fields {
                    match &field.location.reference {
                        FieldReferenceKind::Local | FieldReferenceKind::FieldType(_) => {
                            fields.push(format!(
                                "\t{}.{} as {}",
                                object.table(),
                                field.name,
                                field.name
                            ));
                        }
                        FieldReferenceKind::JoinData(local_join_key) => {
                            let join_name = format!("j_{}", local_join_key);
                            if let Entry::Vacant(e) = joins.entry(join_name.clone()) {
                                let local_join = object
                                    .fields
                                    .iter()
                                    .find(|x| x.name == *local_join_key)
                                    .unwrap();
                                let foreign_object_name = match &local_join.location.reference {
                                    FieldReferenceKind::FieldType(foreign_table) => foreign_table,
                                    _ => {
                                        return Err(RepackError::from_lang_with_obj(
                                            RepackErrorKind::ExpectedReference,
                                            description.output,
                                            object,
                                        ));
                                    }
                                };
                                let foreign_object =
                                    description.object_by_name(foreign_object_name)?;
                                let join = format!(
                                    "INNER JOIN {} {} ON {}.{} = {}.{}",
                                    foreign_object.table(),
                                    join_name,
                                    join_name,
                                    local_join.location.name,
                                    object.table(),
                                    local_join.name
                                );
                                fields.push(format!(
                                    "\t{}.{} as {}",
                                    join_name, field.location.name, field.name
                                ));
                                e.insert(join);
                            }
                        }
                    }
                }
                sql.push_str(&format!(
                    "CREATE VIEW {} AS SELECT\n{}\nFROM {}\n{};\n",
                    object.name,
                    fields.join(",\n"),
                    object.table(),
                    joins.into_values().collect::<Vec<String>>().join("\n")
                ));
            }
        }
        sql.push_str("\nCOMMIT;\n");
        description.append("model.sql", sql);
        Ok(())
    }
}
