use std::collections::HashSet;

use crate::runtime::{object::Object, Reference};

use super::{class_table::ClassTable, symbol_table::{SymbolEntry, SymbolTable}};



pub struct ObjectTable {
    /// This is a hashset for ease of freeing up references
    table: HashSet<*mut Object>,

}

impl ObjectTable {
    pub fn new() -> Self {
        ObjectTable {
            table: HashSet::new(),
        }
    }

    pub fn add(&mut self, ptr: *mut Object) -> Reference {
        if ptr as usize == 0x1b {
            panic!("not in address space")
        }
        self.table.insert(ptr);
        ptr
    }

    pub fn free(&mut self, reference: Reference, symbol_table: &SymbolTable, class_table: &ClassTable) {
        let pointer = reference;

        if pointer.is_null() || !self.table.contains(&pointer) {// We have already handled this object or it isn't collectable
            return;
        }
        //println!("Freeing: {reference}");

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

        self.table.remove(&pointer);
    }

    pub fn iter(&self) -> impl Iterator<Item = &*mut Object> {
        self.table.iter()
    }
}


unsafe impl Send for ObjectTable {}
unsafe impl Sync for ObjectTable {}

