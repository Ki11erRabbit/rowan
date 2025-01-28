use std::{collections::HashMap, sync::{LazyLock, RwLock}};

use rowan_shared::classfile::{ClassFile, VTableEntry};
use tables::{class_table::ClassTable, object_table::ObjectTable, string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue, VTables}};


mod tables;
pub mod class;
pub mod object;

pub type Symbol = usize;

pub type Reference = u64;

pub type Index = usize;

pub type VTableIndex = usize;


static VTABLES: LazyLock<RwLock<VTables>> = LazyLock::new(|| {
    let table = VTables::new();
    RwLock::new(table)
});

static CLASS_TABLE: LazyLock<RwLock<ClassTable>> = LazyLock::new(|| {
    let table = ClassTable::new();
    RwLock::new(table)
});

static STRING_TABLE: LazyLock<RwLock<StringTable>> = LazyLock::new(|| {
    let table = StringTable::new();
    RwLock::new(table)
});

static SYMBOL_TABLE: LazyLock<RwLock<SymbolTable>> = LazyLock::new(|| {
    let table = SymbolTable::new();
    RwLock::new(table)
});

static OBJECT_TABLE: LazyLock<RwLock<ObjectTable>> = LazyLock::new(|| {
    let table = ObjectTable::new();
    RwLock::new(table)
});



pub struct Context {}

impl Context {
    pub const fn new() -> Self {
        Context {}
    }

    pub fn link_class(
        &self,
        class_file: ClassFile,
        string_map: &mut HashMap<String, Symbol>,
        class_map: &mut HashMap<String, Symbol>
    ) {
        let class_name = class_file.index_string_table(class_file.name);
        let Ok(mut string_table) = STRING_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut symbol_table) = SYMBOL_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class_name_symbol = if let Some(str_table_index) = string_map.get(class_name) {
            *str_table_index
        } else {
            let str_table_index = string_table.add_string(class_name);
            let sym = symbol_table.add_string(str_table_index);
            string_map.insert(String::from(class_name), sym);
            sym
        };

        let mut parents: Vec<Symbol> = Vec::new();
        
        for parent in class_file.parents.iter() {
            let parent_name = class_file.index_string_table(*parent);
            if let Some(_) = string_map.get(parent_name) {
            } else {
                let str_table_index = string_table.add_string(parent_name);
                let sym = symbol_table.add_string(str_table_index);
                string_map.insert(String::from(parent_name), sym);
            };
            parents.push(*class_map.get(parent_name).expect("parent wasn't linked first"));
        }

        let Ok(mut vtables) = VTABLES.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };

        for vtable in class_file.vtables.iter() {
            let mut virt_table = Vec::new();
            let mut mapper = HashMap::new();
            let mut parent_class_table = None;
            let mut self_vtable: bool = false;
            for function in vtable.functions.iter() {
                let VTableEntry { class_name: vtable_class_name, name, responds_to, signature, bytecode, .. } = function;
                if parent_class_table.is_none() && !self_vtable {
                    let vtable_class_name = class_file.index_string_table(*vtable_class_name);
                    if vtable_class_name == class_name {
                        self_vtable = true;
                    } else {
                        if let Some(parent_class_symbol) = class_map.get(vtable_class_name) {
                            let SymbolEntry::ClassRef(parent_class_index) = symbol_table[*parent_class_symbol] else {
                                panic!("Symbol was not a class reference");
                            };
                            let parent_class = &class_table[parent_class_index];

                            let vtable_index = parent_class.get_vtable(*parent_class_symbol);
                            parent_class_table = Some(&vtables[vtable_index])
                        }
                    }
                }

                let name_str = class_file.index_string_table(*name);
                let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                    *symbol
                } else {
                    let str_table_index = string_table.add_string(name_str);
                    let sym = symbol_table.add_string(str_table_index);
                    string_map.insert(String::from(name_str), sym);
                    sym
                };

                // TODO: Add responds to code
                let responds_to: Option<Symbol> = None;

                if *bytecode == 0 {
                    if let Some(parent_class_table) = parent_class_table {
                        let function = parent_class_table.get_function(name_symbol);
                        virt_table.push(function.clone());
                        mapper.insert(name_symbol, virt_table.len() - 1);
                    } else {
                        panic!("parent class table isn't set for a missing method");
                    }
                } else if self_vtable {
                } else {
                    panic!("Invalid state");
                }
                

            }
        }
        
    }
}
