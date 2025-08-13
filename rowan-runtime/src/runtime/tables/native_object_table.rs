use std::collections::HashMap;
use libloading::Library;

pub struct NativeObjectTable {
    table: HashMap<String,Library>,
}

impl NativeObjectTable {
    pub fn new() -> Self {
        Self { table: HashMap::new() }
    }

    pub fn insert(&mut self, path: String, lib: Library) {
        self.table.insert(path, lib);
    }

    pub fn get_mut(&mut self, path: &str) -> Option<&mut Library> {
        self.table.get_mut(path)
    }

}

