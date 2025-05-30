use crate::{
    outputs::{OutputBuilder, OutputDescription},
    syntax::{
        FieldCommand, FieldReferenceKind, FieldType, ObjectType, RepackError, RepackErrorKind,
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
                    &description.output,
                    object,
                ));
            }
            sql.push_str("DROP TABLE IF EXISTS ");
            sql.push_str(object.table());
            sql.push_str(";\n");
        }

        for object in description.objects() {
            if object.object_type != ObjectType::Record {
                return Err(RepackError::from_lang_with_obj(
                    RepackErrorKind::UnsupportedObjectType,
                    &description.output,
                    object,
                ));
            }
            if object.inherits.is_some() {
                return Err(RepackError::from_lang(
                    RepackErrorKind::CannotInherit,
                    &description.output,
                ));
            }
            let mut constraints = String::new();
            sql.push_str(&format!("CREATE TABLE {} (\n", object.table()));
            for field in &object.fields {
                let nullability = if field.optional { "" } else { " NOT NULL" };
                let typ =
                    type_to_psql(field.field_type()).ok_or(RepackError::from_lang_with_msg(
                        RepackErrorKind::UnsupportedFieldType,
                        &description.output,
                        field.field_type().to_string(),
                    ))?;
                if let FieldReferenceKind::FieldType(table_ref) = &field.location.reference {
                    let ref_obj = description.object_by_name(table_ref)?;
                    let ref_field = description.field_by_name(ref_obj, &field.location.name)?;
                    let cascade = if field.commands.contains(&FieldCommand::Cascade) {
                        " ON DELETE CASCADE"
                    } else {
                        ""
                    };
                    constraints.push_str(&format!(
                        "\tFOREIGN KEY ({}) REFERENCES {}({}){},\n",
                        field.name,
                        ref_obj.table(),
                        ref_field.name,
                        cascade
                    ));
                }
                sql.push_str(&format!("\t{} {}{},\n", field.name, typ, nullability));
            }
            sql.push_str(&constraints);
            sql.pop(); // Remove last comma
            sql.pop(); // Remove last newline
            sql.push_str("\n);\n");
        }
        sql.push_str("\nCOMMIT;\n");
        description.append("model.sql", sql);
        Ok(())
    }
}
