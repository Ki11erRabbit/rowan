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

    pub fn get_next_index(&self) -> usize {
        self.table.len()
    }

    pub fn insert_class(&mut self, class: Class) -> usize {
        let out = self.table.len();
        self.table.push(class);
        out
    }
}

impl std::ops::Index<usize> for ClassTable {
    type Output = Class;
    fn index(&self, index: usize) -> &Class {
        &self.table[index]
    }
}
