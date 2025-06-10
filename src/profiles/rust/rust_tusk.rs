use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::{
    outputs::OutputBuilder,
    syntax::{
        FieldFunctionName, FieldReferenceKind, FieldType, FunctionNamespace, ObjectType,
        RepackError, RepackErrorKind,
    },
};

use super::type_to_rust;

const ENUM_DATA: &'static str = include_str!("enum_gen.txt");

pub struct RustTuskBuilder;

impl OutputBuilder for RustTuskBuilder {
    fn build(
        &self,
        description: &mut crate::outputs::OutputDescription,
    ) -> Result<(), RepackError> {
        let mut imports: HashSet<String> = HashSet::new();
        let mut output = String::new();

        for enm in description.enums() {
            imports.insert("use tusk_rs::ToSql;".to_string());
            output.push_str(&format!(
                "#[derive(Debug)]\npub enum {} {{\n{}\n}}\n\n",
                enm.name,
                enm.options
                    .iter()
                    .map(|x| format!("\t{}", x))
                    .collect::<Vec<_>>()
                    .join(",\n")
            ));
            let impl_data = ENUM_DATA
                .to_string()
                .replace("__typ", &enm.name)
                .replace(
                    "__to_sql_cases",
                    &enm.options
                        .iter()
                        .map(|x| format!("\t\tSelf::{} => \"{}\"", x, x))
                        .collect::<Vec<_>>()
                        .join(",\n"),
                )
                .replace(
                    "__accepts_cases",
                    &enm.options
                        .iter()
                        .map(|x| format!("\t\t\"{}\" => true", x))
                        .collect::<Vec<_>>()
                        .join(",\n"),
                );
            output.push_str(&impl_data);
        }

        for object in description.objects() {
            let mut derives = Vec::<String>::new();
            let mut write_fields = Vec::<String>::new();
            imports.insert("use tusk_rs::{FromJson,ToJson};".to_string());
            derives.push("Debug".to_string());
            derives.push("FromJson".to_string());
            derives.push("ToJson".to_string());
            if object.object_type == ObjectType::Record {
                imports.insert("use tusk_rs::{PostgresReadable,FromPostgres};".to_string());
                derives.push("PostgresReadable".to_string());
                derives.push("FromPostgres".to_string());
                if object.inherits.is_none() {
                    for f in &object.fields {
                        if object.inherits.is_none()
                            && !f
                                .functions_in_namespace(FunctionNamespace::Database)
                                .iter()
                                .any(|x| {
                                    x.name == FieldFunctionName::Generated
                                        || x.name == FieldFunctionName::GeneratedStored
                                })
                        {
                            write_fields.push(format!("\"{}\"", f.name));
                        }
                    }
                }
            }

            if !write_fields.is_empty() {
                imports.insert("use tusk_rs::{PostgresWriteFields,PostgresWriteable};".to_string());
                derives.push("PostgresWriteable".to_string());
            }
            output.push_str(&format!("#[derive({})]\n", derives.join(",")));
            output.push_str(&format!("pub struct {} {{\n", object.name));
            for field in &object.fields {
                let rust_type =
                    type_to_rust(field.field_type()).ok_or(RepackError::from_lang_with_msg(
                        RepackErrorKind::UnsupportedFieldType,
                        description.output,
                        field.field_type().to_string(),
                    ))?;
                if *field.field_type() == FieldType::DateTime {
                    imports.insert("use tusk_rs::chrono::{DateTime, Utc};".to_string());
                } else if *field.field_type() == FieldType::Uuid {
                    imports.insert("use tusk_rs::uuid::Uuid;".to_string());
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
            if object.object_type == ObjectType::Record {
                imports.insert(
                    "use tusk_rs::{PostgresTable,PostgresReadFields,PostgresField,PostgresJoins,PostgresJoin};"
                        .to_string(),
                );
                output.push_str(&format!(
                    "\
                    impl PostgresTable for {} {{ \
                        fn table_name() -> &'static str {{ \
                            return \"{}\" \
                        }} \
                        }} \n\
                    ",
                    object.name,
                    object.table(),
                ));
                let mut fields = Vec::<String>::new();
                let mut joins = HashMap::<String, String>::new();
                for f in &object.fields {
                    let field = match &f.location.reference {
                        FieldReferenceKind::Local | FieldReferenceKind::FieldType(_) => {
                            imports.insert("use tusk_rs::local;".to_string());
                            format!("local!(\"{}\")", f.name)
                        }
                        FieldReferenceKind::JoinData(join) => {
                            let join_name = format!("j_{}", join);
                            imports.insert("use tusk_rs::foreign_as;".to_string());
                            if let Entry::Vacant(e) = joins.entry(join_name.clone()) {
                                let local_join =
                                    object.fields.iter().find(|x| x.name == *join).unwrap();
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
                                    "&PostgresJoin {{\
                                        join_type: \"INNER JOIN\",
                                        join_name: \"{}\",\
                                        table: \"{}\",\
                                        local_field: \"{}\",\
                                        foreign_field: \"{}\",\
                                        condition: \"=\"\
                                    }}\
                                    ",
                                    join_name,
                                    foreign_object.table(),
                                    join,
                                    local_join.location.name,
                                );
                                e.insert(join);
                            };
                            format!(
                                "foreign_as!(\"{}\", \"{}\", \"{}\")",
                                join_name, f.location.name, f.name
                            )
                        }
                    };
                    fields.push(field);
                }
                output.push_str(&format!(
                    "\
                    impl PostgresReadFields for {} {{ \
                        fn read_fields() -> &'static [&'static PostgresField] {{ \
                            &[ \
                            {} \
                            ] \
                        }} \
                    }}\n\n\
                ",
                    object.name,
                    fields.join(", ")
                ));
                output.push_str(&format!(
                    "\
                    impl PostgresJoins for {} {{ \
                        fn joins() -> &'static [&'static PostgresJoin] {{ \
                            &[ \
                            {} \
                            ] \
                        }} \
                    }}\n\n\
                ",
                    object.name,
                    joins.into_values().collect::<Vec<_>>().join(",")
                ));
                if !write_fields.is_empty() {
                    output.push_str(&format!(
                        "\
                    impl PostgresWriteFields for {} {{ \
                        fn write_fields() -> &'static [&'static str] {{ \
                            &[ \
                            {} \
                            ] \
                        }} \
                    }}\n\n\
                ",
                        object.name,
                        write_fields.join(",")
                    ));
                }
            }
        }
        description.append(
            "model.rs",
            format!("{}\n\n", imports.into_iter().collect::<Vec<_>>().join("\n")),
        );
        description.append("model.rs", output);
        Ok(())
    }
}
