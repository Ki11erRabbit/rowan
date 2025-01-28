use crate::runtime::class::Class;



pub struct ClassTable {
    table: Vec<Class>,
}

impl ClassTable {
    pub fn new() -> Self {
        ClassTable {
            table: Vec::new()
        }
    }
}


impl std::ops::Index<usize> for ClassTable {
    type Output = Class;
    fn index(&self, index: usize) -> &Class {
        &self.table[index]
    }
}
