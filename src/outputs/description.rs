use std::{collections::HashMap, env::current_dir, fs};

use crate::syntax::{Field, Object, Output, ParseResult};

use super::{OutputBuilderError, OutputBuilderFieldError};

pub struct FieldResult<'a> {
    pub field: &'a Field,
    pub object: &'a Object,
}
pub struct OutputDescription<'a> {
    objects: Vec<&'a Object>,
    pub output: &'a Output,
    pub buffers: HashMap<String, String>,
}

impl<'a> OutputDescription<'a> {
    pub fn new(result: &'a ParseResult, output: &'a Output) -> Self {
        let mut objs = result
            .objects
            .iter()
            .filter(|obj| {
                // If the output has categories, filter the objects.
                if !output.categories.is_empty()
                    && !obj
                        .categories
                        .iter()
                        .any(|cat| output.categories.contains(cat))
                {
                    return false;
                }
                if output.exclude.contains(&obj.name) {
                    return false;
                }
                true
            })
            .collect::<Vec<_>>();

        let mut i = 0;
        while i < objs.len() {
            let mut found_issue = false;
            'dep_search: for dependancy in objs[i].depends_on() {
                let mut x = i;
                while x < objs.len() {
                    if objs[x].name == dependancy {
                        found_issue = true;
                        break 'dep_search;
                    }
                    x += 1;
                }
            }
            if found_issue {
                let dep = objs.remove(i);
                objs.push(dep);
                i = 0
            } else {
                i += 1;
            }
        }
        Self {
            objects: objs,
            output,
            buffers: HashMap::new(),
        }
    }

    pub fn append(&mut self, name: &str, contents: String) {
        if let Some(existing) = self.buffers.get_mut(name) {
            existing.push_str(&contents);
        } else {
            self.buffers.insert(name.to_string(), contents);
        }
    }

    pub fn flush(&mut self) -> Result<(), OutputBuilderError> {
        let mut root_path = current_dir().map_err(|_| OutputBuilderError::CannotOpenFile)?;
        if let Some(path) = &self.output.location {
            root_path.push(path);
        }
        for (name, contents) in &self.buffers {
            let mut file_path = root_path.clone();
            file_path.push(name);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).map_err(|_| OutputBuilderError::CannotOpenFile)?;
            }
            fs::write(&file_path, contents).map_err(|_| OutputBuilderError::CannotOpenFile)?;
        }
        Ok(())
    }

    pub fn objects(&self) -> Vec<&'a Object> {
        self.objects.clone()
    }

    pub fn object(
        &self,
        obj: &Object,
        field: &Field,
        name: &str,
    ) -> Result<&'a Object, OutputBuilderError> {
        self.objects
            .iter()
            .find(|obj| obj.name == name)
            .copied()
            .ok_or(OutputBuilderError::FieldReferenceNotIncluded(
                OutputBuilderFieldError::new(obj, field),
            ))
    }

    pub fn fields(&self, obj: &'a Object) -> Result<Vec<&'a Field>, OutputBuilderError> {
        let mut fields = Vec::<&'a Field>::new();
        for field in &obj.fields {
            fields.push(field);
        }
        if let Some(inherit_name) = &obj.inherits {
            let parent = self.objects.iter().find(|o| o.name == *inherit_name).ok_or(
                OutputBuilderError::InheritenceReferenceNotIncluded(
                    obj.name.clone(),
                    inherit_name.clone(),
                ),
            );
            if obj.reuse_all {
                let mut parent_fields = self.fields(parent?)?;
                parent_fields.retain(|x| !obj.reuse_exclude.contains(&x.name));
                fields.append(&mut parent_fields);
            }
        }
        return Ok(fields);
    }

    pub fn field(
        &self,
        obj: &Object,
        field: &Field,
        name: &str,
        field_name: &str,
    ) -> Result<&'a Field, OutputBuilderError> {
        self.object(obj, field, name).and_then(|obj| {
            obj.fields.iter().find(|f| f.name == field_name).ok_or(
                OutputBuilderError::FieldNotFound(OutputBuilderFieldError::new(obj, field)),
            )
        })
    }

    pub fn field_result(
        &self,
        obj: &Object,
        field: &Field,
        name: &str,
        field_name: &str,
    ) -> Result<FieldResult<'a>, OutputBuilderError> {
        self.object(obj, field, name).and_then(|obj| {
            Ok(FieldResult {
                field: obj.fields.iter().find(|f| f.name == field_name).ok_or(
                    OutputBuilderError::FieldNotFound(OutputBuilderFieldError::new(obj, field)),
                )?,
                object: obj,
            })
        })
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        match self.output.options.get(key) {
            Some(value) => value == "true",
            None => default,
        }
    }
}
