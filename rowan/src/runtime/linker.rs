use std::collections::HashMap;

use rowan_shared::classfile::{BytecodeIndex, ClassFile, Member, Signal, SignatureIndex, VTableEntry};

use super::{class::{Class, MemberInfo, SignalInfo}, tables::{string_table::StringTable, symbol_table::SymbolTable, vtable::{Function, VTables}}, Symbol, VTableIndex};


pub enum TableEntry<T> {
    Hole,
    Entry(T),
}






pub fn link_class_files(
    classes: Vec<ClassFile>,
    symbol_table: &mut SymbolTable,
    mut class_table: Vec<TableEntry<Class>>,
    string_table: &mut StringTable,
    virtual_tables: &mut VTables,
) -> Result<(), ()> {
    let mut string_map: HashMap<String, Symbol> = HashMap::new();
    let mut class_map: HashMap<String, Symbol> = HashMap::new();
    let mut vtables_table: HashMap<Symbol, Vec<Vec<(&str, Option<&str>, &str, Option<Symbol>, SignatureIndex, BytecodeIndex)>>> = HashMap::new();

    for class in classes.iter() {
        let ClassFile { name, parents, vtables, members, signals, signature_table, .. } = class;
        let name_str = class.index_string_table(*name);
        let class_name_symbol = if let Some(symbol) = class_map.get(name_str) {
            *symbol
        } else {
            let string_table_index = string_table.add_string(name_str);
            let symbol = symbol_table.add_string(string_table_index);
            string_map.insert(String::from(name_str), symbol);

            let class_table_index = class_table.len();
            class_table.push(TableEntry::Hole);
            let symbol = symbol_table.add_class(class_table_index);
            
            class_map.insert(String::from(name_str), symbol);

            symbol
        };

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

        let mut class_vtables = Vec::new();
        for vtable in vtables {
            let mut virtual_table: Vec<(&str, Option<&str>, &str, Option<Symbol>, SignatureIndex, BytecodeIndex)> = Vec::new();
            for function in vtable.functions.iter() {
                let VTableEntry { class_name, sub_class_name, name, responds_to, signature, bytecode, .. } = function;
                let class_name_str = class.index_string_table(*class_name);
                if let Some(_) = class_map.get(class_name_str) {
                } else {
                    let string_table_index = string_table.add_string(class_name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(name_str), symbol);

                    let class_table_index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(class_table_index);

                    class_map.insert(String::from(class_name_str), symbol);
                }
                let sub_class_name_str = if *sub_class_name != 0 {
                    let sub_class_name_str = class.index_string_table(*sub_class_name);
                    if let Some(_) = class_map.get(sub_class_name_str) {
                    } else {
                        let string_table_index = string_table.add_string(sub_class_name_str);
                        let symbol = symbol_table.add_string(string_table_index);
                        string_map.insert(String::from(name_str), symbol);

                        let class_table_index = class_table.len();
                        class_table.push(TableEntry::Hole);
                        let symbol = symbol_table.add_class(class_table_index);

                        class_map.insert(String::from(sub_class_name_str), symbol);
                    }
                    Some(sub_class_name_str)
                } else {
                    None
                };

                let name_str = class.index_string_table(*name);
                if let Some(_) = string_map.get(name_str) {
                } else {
                    let string_table_index = string_table.add_string(name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(name_str), symbol);
                }

                let responds_to_symbol = if *sub_class_name != 0 {
                    let responds_to_str = class.index_string_table(*sub_class_name);
                    if let Some(symbol) = class_map.get(responds_to_str) {
                        Some(*symbol)
                    } else {
                        let string_table_index = string_table.add_string(responds_to_str);
                        let symbol = symbol_table.add_string(string_table_index);
                        string_map.insert(String::from(name_str), symbol);

                        Some(symbol)
                    }
                } else {
                    None
                };

                virtual_table.push((class_name_str, sub_class_name_str, name_str, responds_to_symbol, *signature, *bytecode));
            }
            class_vtables.push(virtual_table);
        }

        vtables_table.insert(class_name_symbol, class_vtables);
        
        for signal in signals {
            let Signal { name, .. } = signal;
            let name_str = class.index_string_table(*name);
            
            if let Some(_) = string_map.get(name_str) {
            } else {
                let string_table_index = string_table.add_string(name_str);
                let symbol = symbol_table.add_string(string_table_index);
                string_map.insert(String::from(name_str), symbol);
            }
        }
    }

    
    let mut vtable_to_index: HashMap<VTable, VTableIndex> = HashMap::new();
    for class in classes {
        let ClassFile { name, parents, members, signals, signature_table, .. } = &class;
        let class_name_str = class.index_string_table(*name);
        
        let class_symbol = *class_map.get(class_name_str).unwrap();
        let parent_symbols = parents.iter().map(|p| {
            let p_str = class.index_string_table(*p);
            class_map.get(p_str).unwrap()
        }).collect::<Vec<_>>();

        let mut class_members = Vec::new();
        for member in members {
            let rowan_shared::classfile::Member { name, type_tag } = member;

            let name_str = class.index_string_table(*name);
            let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                *symbol
            } else {
                let string_table_index = string_table.add_string(name_str);
                let symbol = symbol_table.add_string(string_table_index);
                string_map.insert(String::from(name_str), symbol);
                symbol
            };

            let type_tag = convert_type(type_tag);

            class_members.push(MemberInfo::new(name_symbol, type_tag));
        }

        let mut class_signals = Vec::new();
        for signal in signals {
            let rowan_shared::classfile::Signal { name, is_static, signature } = signal;

            let name_str = class.index_string_table(*name);
            let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                *symbol
            } else {
                let string_table_index = string_table.add_string(name_str);
                let symbol = symbol_table.add_string(string_table_index);
                string_map.insert(String::from(name_str), symbol);
                symbol
            };

            let signature = &signature_table[*signature as usize];
            let signature = signature.types[1..].iter().map(convert_type).collect::<Vec<_>>();


            class_signals.push(SignalInfo::new(name_symbol, *is_static, signature));
        }

        let mut vtable_mapper = HashMap::new();
        for vtable in vtables_table.get(&class_symbol).unwrap() {
            let mut functions = Vec::new();

            for (class_name, sub_class_name, method_name, responds_to, signature_index, bytecode_index) in vtable {
                let method_name_symbol = string_map.get(method_name).get();

                let arguments = signature_table[1..].iter().map(convert_type).collect::<Vec<_>>();
                let return_type = convert_type(&signature_table[0]);

                let function = if false {
                    // Lookup class method that already exists
                } else if bytecode_index != 0 {
                    // Link function
                } else {
                    // use a RefCell to allow for updating a missing functions in the future
                };
                
            }

        }
    }


    Ok(())
}
        
