use libloading::Library;

pub struct NativeObjectTable {
    table: Vec<Library>,
}

impl NativeObjectTable {
    pub fn new() -> Self {
        Self { table: Vec::new() }
    }

    pub fn insert(&mut self, lib: Library) {
        self.table.push(lib);
    }

}

impl std::ops::Index<usize> for NativeObjectTable {
    type Output = Library;
    fn index(&self, index: usize) -> &Library {
        &self.table[index]
    }
}