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

    pub fn insert_class(&mut self, class: Class) -> usize {
        let out = self.table.len();
        self.table.push(class);
        out
    }
    
    pub fn iter(&self) -> impl Iterator<Item=&Class> {
        self.table.iter()
    }
}

impl std::ops::Index<usize> for ClassTable {
    type Output = Class;
    fn index(&self, index: usize) -> &Class {
        &self.table[index]
    }
}

impl std::ops::IndexMut<usize> for ClassTable {
    fn index_mut(&mut self, index: usize) -> &mut Class {
        &mut self.table[index]
    }
}
