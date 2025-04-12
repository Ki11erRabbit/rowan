use crate::runtime::Symbol;

#[derive(Copy, Clone, Debug)]
pub enum SymbolEntry {
    StringRef(usize),
    ClassRef(usize),
}


#[derive(Debug)]
pub struct SymbolTable {
    table: Vec<SymbolEntry>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: Vec::new()
        }
    }

    pub fn add_string(&mut self, index: usize) -> Symbol {
        let out = self.table.len();
        self.table.push(SymbolEntry::StringRef(index));
        out
    }

    pub fn add_class(&mut self, index: usize) -> Symbol {
        let out = self.table.len();
        self.table.push(SymbolEntry::ClassRef(index));
        out
    }
}


impl std::ops::Index<usize> for SymbolTable {
    type Output = SymbolEntry;
    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}
