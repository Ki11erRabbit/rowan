
pub enum SymbolEntry {
    StringRef(usize),
    ClassRef(usize),
}


pub struct SymbolTable {
    table: Vec<SymbolEntry>,
}


impl std::ops::Index<usize> for SymbolTable {
    type Output = SymbolEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}
