use std::collections::{HashMap, HashSet};

use crate::syntax::{
    ConfigurationInstance, CoreType, Enum, EnumCase, Field, FieldReferenceKind, FieldType, Object,
    ObjectJoin, ObjectType, Output, RepackError, RepackErrorKind,
};

use super::{Blueprint, SnippetMainTokenName, SnippetSecondaryTokenName};

pub(crate) trait TokenConsumer {
    fn set_file_name(&mut self, filename: &str);
    fn import_point(&mut self);
    fn write(&mut self, value: &dyn AsRef<str>);
    fn import(&mut self, value: String);
}
impl TokenConsumer for HashSet<String> {
    fn set_file_name(&mut self, filename: &str) {
        self.insert(filename.to_string());
    }
    fn write(&mut self, _value: &dyn AsRef<str>) {}
    fn import(&mut self, _value: String) {}
    fn import_point(&mut self) {}
}
impl TokenConsumer for String {
    fn set_file_name(&mut self, _filename: &str) {}
    fn write(&mut self, value: &dyn AsRef<str>) {
        self.push_str(value.as_ref());
    }
    fn import(&mut self, _value: String) {}
    fn import_point(&mut self) {}
}

#[derive(Debug, Clone)]
pub(crate) struct BlueprintExecutionContext<'a> {
    pub variables: HashMap<String, String>,
    pub flags: HashMap<&'a str, bool>,
    pub object: Option<&'a Object>,
    pub field: Option<&'a Field>,
    pub enm: Option<&'a Enum>,
    pub func_args: Option<&'a Vec<String>>,
}
impl<'a> BlueprintExecutionContext<'a> {
    pub fn new() -> BlueprintExecutionContext<'a> {
        BlueprintExecutionContext {
            variables: HashMap::new(),
            flags: HashMap::new(),
            object: None,
            field: None,
            enm: None,
            func_args: None,
        }
    }
    pub fn with_object(&self, obj: &'a Object) -> Self {
        let mut variables = self.variables.clone();
        let mut flags = self.flags.clone();
        variables.insert("name".to_string(), obj.name.to_string());
        if let Some(tn) = obj.table_name.as_ref() {
            variables.insert("table_name".to_string(), tn.to_string());
        }
        flags.insert("record", matches!(obj.object_type, ObjectType::Record));
        flags.insert("syn", matches!(obj.object_type, ObjectType::Synthetic));
        flags.insert(
            "synthetic",
            matches!(obj.object_type, ObjectType::Synthetic),
        );
        flags.insert("struct", matches!(obj.object_type, ObjectType::Struct));
        flags.insert("has_joins", !obj.joins.is_empty());

        Self {
            variables,
            flags,
            object: Some(obj),
            field: None,
            enm: None,
            func_args: None,
        }
    }
    pub fn with_field(
        &self,
        obj: &'a Object,
        field: &'a Field,
        blueprint: &'a Blueprint,
        config: &Output,
        writer: &mut dyn TokenConsumer,
    ) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        let mut flags = self.flags.clone();

        let resolved_type = match field.field_type() {
            FieldType::Core(typ) => {
                if let Some(link) = blueprint.links.get(&typ.to_string()) {
                    writer.import(link.to_string());
                }

                blueprint
                    .utilities
                    .get(&(
                        SnippetMainTokenName::TypeDef,
                        SnippetSecondaryTokenName::from_type(typ),
                    ))
                    .ok_or_else(|| {
                        RepackError::from_lang_with_obj_field_msg(
                            RepackErrorKind::TypeNotSupported,
                            config,
                            obj,
                            field,
                            typ.to_string(),
                        )
                    })?
            }
            FieldType::Custom(typ, _) => {
                if let Some(link) = blueprint.links.get("custom") {
                    writer.import(link.replace("$", typ))
                }
                typ
            }
        };

        let (name, loc) = match &field.location.reference {
            FieldReferenceKind::Local | FieldReferenceKind::FieldType(_) => (
                field.name.clone(),
                obj.table_name
                    .as_ref()
                    .map(|x| x.to_string())
                    .unwrap_or_default(),
            ),
            FieldReferenceKind::ImplicitJoin(local_field_name) => {
                (field.location.name.clone(), format!("j_{local_field_name}"))
            }
            FieldReferenceKind::ExplicitJoin(jn) => (field.location.name.clone(), jn.to_string()),
        };
        variables.insert("ref_table".to_string(), loc);
        variables.insert("ref_field".to_string(), name);
        variables.insert("object_name".to_string(), obj.name.to_string());
        variables.insert("name".to_string(), field.name.to_string());
        variables.insert("type".to_string(), resolved_type.to_string());
        flags.insert(
            "enum",
            matches!(
                field.field_type(),
                FieldType::Custom(_, crate::syntax::CustomFieldType::Enum)
            ),
        );
        flags.insert("optional", field.optional);
        flags.insert("array", field.array);
        flags.insert(
            "custom",
            matches!(field.field_type(), FieldType::Custom(_, _)),
        );
        flags.insert(
            "local",
            matches!(
                field.location.reference,
                FieldReferenceKind::Local | FieldReferenceKind::FieldType(_)
            ),
        );
        flags.insert(
            "uuid",
            matches!(field.field_type(), FieldType::Core(CoreType::Uuid)),
        );
        flags.insert(
            "string",
            matches!(field.field_type(), FieldType::Core(CoreType::String)),
        );
        flags.insert(
            "int32",
            matches!(field.field_type(), FieldType::Core(CoreType::Int32)),
        );
        flags.insert(
            "int64",
            matches!(field.field_type(), FieldType::Core(CoreType::Int64)),
        );
        flags.insert(
            "float64",
            matches!(field.field_type(), FieldType::Core(CoreType::Float64)),
        );
        flags.insert(
            "boolean",
            matches!(field.field_type(), FieldType::Core(CoreType::Boolean)),
        );
        flags.insert(
            "datetime",
            matches!(field.field_type(), FieldType::Core(CoreType::DateTime)),
        );
        flags.insert(
            "bytes",
            matches!(field.field_type(), FieldType::Core(CoreType::Bytes)),
        );

        Ok(Self {
            variables,
            flags,
            object: Some(obj),
            field: Some(field),
            enm: None,
            func_args: None,
        })
    }
    pub fn with_join(&self, obj: &'a Object, join: &'a ObjectJoin) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        let flags = self.flags.clone();

        variables.insert("name".to_string(), join.join_name.to_string());
        variables.insert(
            "local_entity".to_string(),
            join.local_base.as_ref().unwrap().to_string(),
        );
        variables.insert(
            "local_base".to_string(),
            join.local_base.as_ref().unwrap().to_string(),
        );
        variables.insert("ref_entity".to_string(), join.foreign_entity.to_string());
        variables.insert("local_field".to_string(), join.local_field.to_string());
        variables.insert("ref_field".to_string(), join.foreign_field.to_string());
        variables.insert(
            "ref_table".to_string(),
            join.foreign_table.as_ref().unwrap().to_string(),
        );
        variables.insert("condition".to_string(), join.condition.to_string());
        variables.insert("join_type".to_string(), join.join_type.to_string());

        Ok(Self {
            variables,
            flags,
            object: Some(obj),
            field: None,
            enm: None,
            func_args: None,
        })
    }

    pub fn with_enum(&self, enm: &'a Enum) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        variables.insert("name".to_string(), enm.name.to_string());
        Ok(Self {
            variables,
            flags: HashMap::new(),
            object: None,
            field: None,
            enm: Some(enm),
            func_args: None,
        })
    }
    pub fn with_enum_case(&self, enm: &'a Enum, val: &'a EnumCase) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let flags = HashMap::new();

        variables.insert("enum_name".to_string(), enm.name.to_string());
        variables.insert("name".to_string(), val.name.to_string());
        variables.insert(
            "value".to_string(),
            val.value.as_ref().unwrap_or(&val.name).to_string(),
        );

        Ok(Self {
            variables,
            flags,
            object: None,
            field: None,
            enm: None,
            func_args: None,
        })
    }
    pub fn with_func_args(&self, args: &'a Vec<String>) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        let mut flags = HashMap::new();

        for (idx, arg) in args.iter().enumerate() {
            variables.insert(format!("{idx}"), arg.to_string());
        }

        flags.insert("has_args", !args.is_empty());

        Ok(Self {
            variables,
            flags,
            func_args: Some(args),
            ..self.clone()
        })
    }
    pub fn with_func_arg(&self, arg: &'a String) -> Result<Self, RepackError> {
        let mut variables = HashMap::new();
        let flags = HashMap::new();

        variables.insert("arg".to_string(), arg.to_string());

        Ok(Self {
            variables,
            flags,
            ..self.clone()
        })
    }
    pub fn with_instance(&self, instance: &'a ConfigurationInstance) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        let flags = self.flags.clone();

        variables.insert("name".to_string(), instance.name.to_string());
        for (k, v) in &instance.values {
            variables.insert(k.to_string(), v.to_string());
        }

        Ok(Self {
            variables,
            flags,
            ..self.clone()
        })
    }
}
