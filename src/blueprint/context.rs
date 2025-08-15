use std::collections::{HashMap, HashSet};

use crate::syntax::{
    CoreType, Field, FieldType, Output, ParseResult, Query, QueryArg, QueryReturn, RepackEnum,
    RepackEnumCase, RepackError, RepackErrorKind, RepackStruct,
};

use super::{Blueprint, SnippetMainTokenName, SnippetSecondaryTokenName};

pub(crate) trait TokenConsumer {
    fn set_file_name(&mut self, filename: &str);
    fn import_point(&mut self);
    fn write(&mut self, value: &dyn AsRef<str>);
    fn delete_trailing(&mut self, value: &dyn AsRef<str>);
    fn import(&mut self, value: String);
}
impl TokenConsumer for HashSet<String> {
    fn set_file_name(&mut self, filename: &str) {
        self.insert(filename.to_string());
    }
    fn delete_trailing(&mut self, _value: &dyn AsRef<str>) {}
    fn write(&mut self, _value: &dyn AsRef<str>) {}
    fn import(&mut self, _value: String) {}
    fn import_point(&mut self) {}
}
impl TokenConsumer for String {
    fn set_file_name(&mut self, _filename: &str) {}
    fn write(&mut self, value: &dyn AsRef<str>) {
        self.push_str(value.as_ref());
    }
    fn delete_trailing(&mut self, value: &dyn AsRef<str>) {
        if self.ends_with(value.as_ref()) {
            let mut del_ct = 0;
            let len = value.as_ref().chars().count();
            if let Some(cutoff) = self.char_indices().rev().find_map(|(idx, _)| {
                del_ct += 1;
                if del_ct == len {
                    Some(idx)
                } else {
                    None
                }
            }) {
                self.truncate(cutoff);
            }
        }
    }
    fn import(&mut self, _value: String) {}
    fn import_point(&mut self) {}

}

#[derive(Debug, Clone, Default)]
pub(crate) struct BlueprintExecutionContext<'a> {
    pub variables: HashMap<String, String>,
    pub flags: HashMap<&'a str, bool>,
    pub strct: Option<&'a RepackStruct>,
    pub field: Option<&'a Field>,
    pub enm: Option<&'a RepackEnum>,
    pub func_args: Option<&'a Vec<String>>,
    pub query: Option<&'a Query>,
}
impl<'a> BlueprintExecutionContext<'a> {
    pub fn new() -> BlueprintExecutionContext<'a> {
        BlueprintExecutionContext {
            variables: HashMap::new(),
            flags: HashMap::new(),
            strct: None,
            field: None,
            enm: None,
            func_args: None,
            query: None,
        }
    }
    pub fn with_strct(&self, obj: &'a RepackStruct) -> Self {
        let mut variables = self.variables.clone();
        let mut flags = self.flags.clone();
        variables.insert("name".to_string(), obj.name.to_string());
        if let Some(tn) = obj.table_name.as_ref() {
            variables.insert("table_name".to_string(), tn.to_string());
        }
        flags.insert("queries", !obj.queries.is_empty());

        Self {
            variables,
            flags,
            strct: Some(obj),
            ..Default::default()
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

        let resolved_type = match field.field_type.as_ref() {
            Some(field_type) => match field_type {
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
            }
            None => {
                return Err(RepackError::from_field(
                    RepackErrorKind::TypeNotResolved,
                    obj,
                    field
                ));
            }
        };

        variables.insert("struct_name".to_string(), obj.name.to_string());
        variables.insert("name".to_string(), field.name.to_string());
        variables.insert("type".to_string(), resolved_type.to_string());
        variables.insert(
            "type_raw".to_string(),
            field.field_type.as_ref().unwrap_or(&FieldType::Core(crate::syntax::CoreType::String)).to_string(),
        );
        flags.insert("optional", field.optional);
        flags.insert("array", field.array);

        Ok(Self {
            variables,
            flags,
            strct: Some(obj),
            field: Some(field),
            ..Default::default()
        })
    }
    pub fn with_query(
        &self,
        obj: &'a RepackStruct,
        q: &'a Query,
        result: &'a ParseResult,
    ) -> Result<Self, RepackError> {
        let mut new = self.clone();
        new.variables
            .insert("query".to_string(), q.render(obj, &result.strcts)?);
        new.variables.insert("name".to_string(), q.name.to_string());
        new.variables
            .insert("struct_name".to_string(), obj.name.to_string());
        new.flags
            .insert("returns_many", matches!(q.ret_type, QueryReturn::Many));
        new.flags
            .insert("returns_one", matches!(q.ret_type, QueryReturn::One));
        new.flags
            .insert("returns_none", matches!(q.ret_type, QueryReturn::None));
        new.query = Some(q);

        Ok(new)
    }
    pub fn with_query_arg(
        &self,
        arg: &'a QueryArg,
        blueprint: &'a Blueprint,
        writer: &mut dyn TokenConsumer,
    ) -> Result<Self, RepackError> {
        let mut new = self.clone();
        new.variables
            .insert("name".to_string(), arg.name.to_string());
        let resolved_type = match CoreType::from_string(&arg.typ) {
            Some(typ) => {
                if let Some(link) = blueprint.links.get(&typ.to_string()) {
                    writer.import(link.replace("$", &typ.to_string()))
                }
                blueprint
                    .utilities
                    .get(&(
                        SnippetMainTokenName::TypeDef,
                        SnippetSecondaryTokenName::from_type(&typ),
                    ))
                    .ok_or_else(|| {
                        RepackError::global(RepackErrorKind::TypeNotSupported, typ.to_string())
                    })?
            }
            None => {
                if let Some(link) = blueprint.links.get("custom") {
                    writer.import(link.replace("$", &arg.typ))
                }
                &arg.typ
            }
        };
        new.variables
            .insert("type".to_string(), resolved_type.to_string());

        Ok(new)
    }
    pub fn with_enum(&self, enm: &'a RepackEnum) -> Result<Self, RepackError> {
        let mut variables = self.variables.clone();
        variables.insert("name".to_string(), enm.name.to_string());
        Ok(Self {
            variables,
            flags: HashMap::new(),
            enm: Some(enm),
            ..Default::default()
        })
    }
    pub fn with_enum_case(
        &self,
        enm: &'a RepackEnum,
        val: &'a RepackEnumCase,
    ) -> Result<Self, RepackError> {
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
            ..Default::default()
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
