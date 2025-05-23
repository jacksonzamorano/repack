use std::{collections::HashMap, env::current_dir, fs};

use crate::syntax::{Object, Output, ParseResult};

use super::OutputBuilderError;

pub struct OutputDescription<'a> {
    objects: &'a Vec<Object>,
    pub output: &'a Output,
    pub buffers: HashMap<String, String>,
}

impl<'a> OutputDescription<'a> {
    pub fn new(result: &'a ParseResult, output: &'a Output) -> Self {
        Self {
            objects: &result.objects,
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
        self.objects.iter().flat_map(|obj| {
            if !obj.categories.is_empty() && !obj.categories.iter().any(|cat| self.output.categories.contains(cat)) {
                return None
            }
            if self.output.exclude.contains(&obj.name) {
                return None
            }
            Some(obj)
        }).collect()
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        match self.output.options.get(key) {
            Some(value) => value == "true",
            None => default
        }
    }
}