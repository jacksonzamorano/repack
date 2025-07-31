use std::collections::{HashMap, HashSet};

use crate::syntax::{
    RepackEnum, RepackEnumCase, Field, FieldType, RepackStruct, Output, RepackError,
    RepackErrorKind,
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
    pub object: Option<&'a RepackStruct>,
    pub field: Option<&'a Field>,
    pub enm: Option<&'a RepackEnum>,
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
    pub fn with_object(&self, obj: &'a RepackStruct) -> Self {
        let mut variables = self.variables.clone();
        let flags = self.flags.clone();
        variables.insert("name".to_string(), obj.name.to_string());
        if let Some(tn) = obj.table_name.as_ref() {
            variables.insert("table_name".to_string(), tn.to_string());
        }

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
        obj: &'a RepackStruct,
        field: &'a Field,
        blueprint: &'a Blueprint,
        config: &Output,
        writer: &mut dyn TokenConsumer,
    ) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        let mut flags = self.flags.clone();

        let resolved_type = match field.field_type.as_ref().unwrap() {
            FieldType::Core(typ) => {
                if let Some(link) = blueprint.links.get(&typ.to_string()) {
                    writer.import(link.replace("$", &typ.to_string()))
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

        variables.insert("object_name".to_string(), obj.name.to_string());
        variables.insert("name".to_string(), field.name.to_string());
        variables.insert("type".to_string(), resolved_type.to_string());
        variables.insert(
            "type_raw".to_string(),
            field.field_type.as_ref().unwrap().to_string(),
        );
        flags.insert("optional", field.optional);
        flags.insert("array", field.array);

        Ok(Self {
            variables,
            flags,
            object: Some(obj),
            field: Some(field),
            enm: None,
            func_args: None,
        })
    }
    pub fn with_enum(&self, enm: &'a RepackEnum) -> Result<Self, RepackError> {
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
    pub fn with_enum_case(&self, enm: &'a RepackEnum, val: &'a RepackEnumCase) -> Result<Self, RepackError> {
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
}
