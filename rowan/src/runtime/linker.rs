use std::collections::HashMap;

use rowan_shared::classfile::{BytecodeIndex, ClassFile};

use super::{class::Class, tables::{string_table::StringTable, symbol_table::SymbolTable}, Symbol};


pub enum TableEntry<T> {
    Hole,
    Entry(T),
}






pub fn link_class_files(
    classes: Vec<ClassFile>,
    symbol_table: &mut SymbolTable,
    mut class_table: Vec<TableEntry<Class>>,
    mut vtables_table: Vec<TableEntry<Vec<TableEntry<(&str, Option<&str>, &str, BytecodeIndex)>>>>,
    string_table: &mut StringTable
) -> Result<(), ()> {
    let mut string_map: HashMap<String, Symbol> = HashMap::new();
    let mut class_map: HashMap<String, Symbol> = HashMap::new();

    for class in classes.iter() {
        let ClassFile { name, parents, vtables, members, signals, signature_table, .. } = class;
        let name_str = class.index_string_table(*name);
        if let Some(_) = class_map.get(name_str) {
        } else {
            let string_table_index = string_table.add_string(name_str);
            let symbol = symbol_table.add_string(string_table_index);
            string_map.insert(String::from(name_str), symbol);

            let class_table_index = class_table.len();
            class_table.push(TableEntry::Hole);
            let symbol = symbol_table.add_class(class_table_index);
            
            class_map.insert(String::from(name_str), symbol);
        }

        for parent in parents.iter() {
            let name_str = class.index_string_table(*parent);
            if let Some(_) = class_map.get(name_str) {
            } else {
                let string_table_index = string_table.add_string(name_str);
                let symbol = symbol_table.add_string(string_table_index);
                string_map.insert(String::from(name_str), symbol);

                let class_table_index = class_table.len();
                class_table.push(TableEntry::Hole);
                let symbol = symbol_table.add_class(class_table_index);

                class_map.insert(String::from(name_str), symbol);
            }
        }

        

    }


    Ok(())
}
        
