use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use crate::{
    blueprint::{Blueprint, BlueprintFileReader},
    syntax::{RepackError, RepackErrorKind},
};

const CORE_BLUEPRINTS: &[&str] = &[
    include_str!("core/rust.blueprint"),
    include_str!("core/postgres.blueprint"),
    include_str!("core/typescript.blueprint"),
];

pub struct BlueprintStore {
    languages: HashMap<String, Blueprint>,
}
impl BlueprintStore {
    pub fn new() -> Result<BlueprintStore, RepackError> {
        let mut store = BlueprintStore {
            languages: HashMap::new(),
        };

        for core in CORE_BLUEPRINTS {
            store.load_string(core)?
        }

        Ok(store)
    }

    pub fn load_file(&mut self, path: &PathBuf) -> Result<(), RepackError> {
        let mut file = File::open(path).map_err(|_| {
            RepackError::global(
                RepackErrorKind::CannotRead,
                path.to_str().unwrap().to_string(),
            )
        })?;
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let reader = BlueprintFileReader {
            reader: contents.iter().peekable(),
        };

        let lang = Blueprint::new(reader)?;
        self.languages.insert(lang.id.clone(), lang);

        Ok(())
    }

    pub fn load_string(&mut self, contents: &str) -> Result<(), RepackError> {
        let reader = BlueprintFileReader {
            reader: contents.as_bytes().iter().peekable(),
        };
        let lang = Blueprint::new(reader)?;
        self.languages.insert(lang.id.clone(), lang);

        Ok(())
    }

    pub fn blueprint(&self, tag: &str) -> Option<&Blueprint> {
        self.languages.get(tag)
    }
}
