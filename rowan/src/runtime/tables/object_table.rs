use std::collections::VecDeque;

use crate::runtime::{object::Object, Reference};

use super::{class_table::ClassTable, symbol_table::{SymbolEntry, SymbolTable}};



pub struct ObjectTable {
    /// This table is 1 indexed so that if we ever get a 0, then we know that we had a null reference
    table: Vec<*mut Object>,
    free_list: VecDeque<Reference>,

}

impl ObjectTable {
    pub fn new() -> Self {
        ObjectTable {
            table: Vec::new(),
            free_list: VecDeque::new(),
        }
    }

    pub fn add(&mut self, ptr: *mut Object) -> Reference {
        if let Some(front) = self.free_list.pop_front() {
            self[front] = ptr;
            return front as Reference;
        }
        self.table.push(ptr);
        self.table.len() as Reference
    }

    pub fn free(&mut self, reference: Reference, symbol_table: &SymbolTable, class_table: &ClassTable) {
        let pointer = self[reference];

        if pointer.is_null() {// We have already handled this object
            return;
        }
        println!("Freeing: {reference}");

        let (class_symbol, parent_objects) = unsafe {
            pointer.as_ref().expect("Non-null pointer was null").get_class_and_parents()
        };
        let SymbolEntry::ClassRef(class_ref) = symbol_table[class_symbol] else {
            panic!("Expected SymbolEntry::ClassRef");
        };

        for parent in parent_objects.iter() {
            self.free(*parent, symbol_table, class_table);
        }

        let class = &class_table[class_ref];
        let data_size = class.get_member_size();

        unsafe {
            Object::free(pointer, data_size);
        }

        self.free_list.push_back(reference);
        self[reference] = std::ptr::null::<Object>() as *mut Object;        
    }

    pub fn iter(&self) -> impl Iterator<Item = &*mut Object> {
        self.table.iter()
    }
}

impl std::ops::Index<Reference> for ObjectTable {
    type Output = *mut Object;
    fn index(&self, index: Reference) -> &Self::Output {
        if index == 0 {
            return &(std::ptr::null::<Object>() as *mut Object);
        }
        &self.table[(index - 1) as usize]
    }
}

impl std::ops::IndexMut<Reference> for ObjectTable {
    fn index_mut(&mut self, index: Reference) -> &mut Self::Output {
        if index == 0 {
            panic!("Null Reference");
        }
        &mut self.table[(index - 1) as usize]
    }
}

unsafe impl Send for ObjectTable {}
unsafe impl Sync for ObjectTable {}

