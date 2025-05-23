use crate::{
    outputs::{OutputBuilder, OutputBuilderError, OutputBuilderFieldError, OutputDescription},
    syntax::FieldType,
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
    fn build(&self, description: &mut OutputDescription) -> Result<(), OutputBuilderError> {
        let mut sql = String::new();
        sql.push_str("BEGIN;\n\n");
        for object in description.objects() {
            sql.push_str("DROP TABLE IF EXISTS ");
            sql.push_str(object.table());
            sql.push_str(";\n");
        }

        for object in description.objects() {
            let mut constraints = String::new();
            sql.push_str(&format!("CREATE TABLE {} (\n", object.table()));
            for field in &object.fields {
                let nullability = if field.optional {
                    ""
                } else {
                    " NOT NULL"
                };
                let resolved_type = field.resolve_type(object, description)?;
                let typ = type_to_psql(resolved_type).ok_or(
                    OutputBuilderError::UnsupportedFieldType(OutputBuilderFieldError::new(
                        object, field,
                    )),
                )?;
                if let FieldType::Ref(foreign_obj, foreign_field) = &field.field_type {
                    let ref_field = description.field_result(object, field, foreign_obj, foreign_field)?;
                    constraints.push_str(&format!(
                        "\tFOREIGN KEY ({}) REFERENCES {}({}),\n",
                        field.name,
                        ref_field.object.table(),
                        ref_field.field.name,
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
