use std::{collections::HashMap, fs::File, io::Read, path::PathBuf, process::exit};

use crate::blueprint::{Blueprint, BlueprintError, BlueprintFileReader};

const CORE_BLUEPRINTS: &[&str] = &[include_str!("core/rust.blueprint")];

pub struct BlueprintStore {
    languages: HashMap<String, Blueprint>,
}
impl BlueprintStore {
    pub fn new() -> BlueprintStore {
        let mut store = BlueprintStore {
            languages: HashMap::new(),
        };

        for core in CORE_BLUEPRINTS {
            if let Err(err) = store.load_string(core) {
                println!("[CORE] Could not load core blueprints: {}", err.output());
                exit(1);
            }
        }

        store
    }

    pub fn load_file(&mut self, path: &PathBuf) -> Result<(), BlueprintError> {
        let mut file = File::open(path).map_err(|_| BlueprintError::CannotRead)?;
        let mut contents = vec![];
        _ = file.read_to_end(&mut contents);

        let reader = BlueprintFileReader {
            reader: contents.iter().peekable(),
        };

        let lang = Blueprint::new(reader)?;
        self.languages.insert(lang.id.clone(), lang);

        Ok(())
    }

    pub fn load_string(&mut self, contents: &str) -> Result<(), BlueprintError> {
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
