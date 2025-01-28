use std::{collections::HashMap, sync::{LazyLock, RwLock}};

use rowan_shared::classfile::{BytecodeIndex, ClassFile, SignatureIndex, VTableEntry};
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

        let mut vtables: Vec<(Vec<(&str, Symbol, Option<Symbol>, SignatureIndex, BytecodeIndex)>, HashMap<Symbol, Index>)> = Vec::new();
        for vtable in class_file.vtables.iter() {
            let mut virt_table = Vec::new();
            let mut mapper = HashMap::new();
            for (i, function) in vtable.functions.iter().enumerate() {
                let VTableEntry { class_name: vtable_class_name, name, responds_to, signature, bytecode, .. } = function;
                let vtable_class_name = class_file.index_string_table(*vtable_class_name);
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
                let responds_to: Option<Symbol> = if *responds_to == 0 {
                    None
                } else {
                    let name_str = class_file.index_string_table(*responds_to);
                    if let Some(symbol) = string_map.get(name_str) {
                        Some(*symbol)
                    } else {
                        let str_table_index = string_table.add_string(name_str);
                        let sym = symbol_table.add_string(str_table_index);
                        string_map.insert(String::from(name_str), sym);
                        Some(sym)
                    }
                };

                virt_table.push((vtable_class_name, name_symbol, responds_to, *signature, *bytecode));
                mapper.insert(name_symbol, i);
            }
            vtables.push((virt_table, mapper));
        }
        
    }
}
