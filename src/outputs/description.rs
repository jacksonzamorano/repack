use crate::syntax::{Enum, Field, Object, Output, ParseResult, RepackError, RepackErrorKind};
use std::{collections::HashMap, env::current_dir, fs};

pub struct OutputDescription<'a> {
    objects: Vec<&'a Object>,
    enums: Vec<&'a Enum>,
    pub output: &'a Output,
    pub buffers: HashMap<String, String>,
}

impl<'a> OutputDescription<'a> {
    pub fn new(result: &'a ParseResult, output: &'a Output) -> Result<Self, RepackError> {
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
        let enums = result
            .enums
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

        let mut included_types: Vec<String> = objs.iter().map(|x| x.name.to_string()).collect();
        included_types.append(&mut enums.iter().map(|x| x.name.to_string()).collect());

        for o in &objs {
            for f in &o.fields {
                if let crate::syntax::FieldType::Custom(typ, _) = f.field_type() {
                    if !included_types.contains(typ) {
                        return Err(RepackError::from_field_with_msg(
                            RepackErrorKind::ObjectNotIncluded,
                            o,
                            f,
                            typ.to_string(),
                        ));
                    }
                }
            }
        }

        Ok(Self {
            objects: objs,
            enums,
            output,
            buffers: HashMap::new(),
        })
    }

    pub fn append(&mut self, name: &str, contents: String) {
        if let Some(existing) = self.buffers.get_mut(name) {
            existing.push_str(&contents);
        } else {
            self.buffers.insert(name.to_string(), contents);
        }
    }

    pub fn flush(&mut self) -> Result<(), RepackError> {
        let mut root_path = current_dir()
            .map_err(|_| RepackError::from_lang(RepackErrorKind::CannotWriteFile, self.output))?;
        if let Some(path) = &self.output.location {
            root_path.push(path);
        }
        for (name, contents) in &self.buffers {
            let mut file_path = root_path.clone();
            file_path.push(name);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).map_err(|_| {
                    RepackError::from_lang(RepackErrorKind::CannotWriteFile, self.output)
                })?;
            }
            fs::write(&file_path, contents).map_err(|_| {
                RepackError::from_lang(RepackErrorKind::CannotWriteFile, self.output)
            })?;
        }
        Ok(())
    }

    pub fn clean(&mut self) -> Result<(), RepackError> {
        let mut root_path = current_dir()
            .map_err(|_| RepackError::from_lang(RepackErrorKind::CannotWriteFile, self.output))?;
        if let Some(path) = &self.output.location {
            root_path.push(path);
        }
        for name in self.buffers.keys() {
            let mut file_path = root_path.clone();
            file_path.push(name);
            _ = fs::remove_file(file_path);
        }
        if let Ok(mut entries) = fs::read_dir(&root_path) {
            if entries.next().is_none() {
                _ = fs::remove_dir_all(&root_path);
            }
        }
        Ok(())
    }

    pub fn objects(&self) -> Vec<&'a Object> {
        self.objects.clone()
    }

    pub fn enums(&self) -> Vec<&'a Enum> {
        self.enums.clone()
    }

    pub fn object_by_name(&self, obj_name: &str) -> Result<&'a Object, RepackError> {
        self.objects
            .iter()
            .find(|obj| obj.name == obj_name)
            .copied()
            .ok_or(RepackError::from_lang_with_msg(
                RepackErrorKind::ObjectNotIncluded,
                self.output,
                obj_name.to_string(),
            ))
    }

    pub fn field_by_name(
        &self,
        obj: &'a Object,
        field_name: &str,
    ) -> Result<&'a Field, RepackError> {
        Ok(obj
            .fields
            .iter()
            .find(|field| field.name == field_name)
            .unwrap())
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        match self.output.options.get(key) {
            Some(value) => value == "true",
            None => default,
        }
    }
}
