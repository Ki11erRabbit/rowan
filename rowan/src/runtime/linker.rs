use std::collections::HashMap;

use rowan_shared::classfile::{self, BytecodeIndex, ClassFile, Member, Signal, SignatureIndex, VTableEntry};

use super::{class::{self, Class, MemberInfo, SignalInfo}, tables::{string_table::StringTable, symbol_table::SymbolTable, vtable::{Function, FunctionValue, VTable, VTables}}, Symbol, VTableIndex};


pub enum TableEntry<T> {
    Hole,
    Entry(T),
}






pub fn link_class_files(
    classes: Vec<ClassFile>,
    symbol_table: &mut SymbolTable,
    mut class_table: Vec<TableEntry<Class>>,
    string_table: &mut StringTable,
) -> Result<(), ()> {
    let mut string_map: HashMap<String, Symbol> = HashMap::new();
    let mut class_map: HashMap<String, Symbol> = HashMap::new();
    let mut vtables_map: HashMap<(Symbol, Option<Symbol>), Vec<(Symbol, Option<Symbol>, SignatureIndex, BytecodeIndex, FunctionValue)>> = HashMap::new();
    let mut vtable_table: Vec<VTable> = Vec::new();

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

        let mut class_vtables = HashMap::new();
        for vtable in vtables {
            let classfile::VTable { class_name, sub_class_name, functions } = vtable;

            let mut current_vtable = Vec::new();

            let class_name_str = class.index_string_table(*class_name);
            let class_name_symbol = if let Some(symbol) = class_map.get(class_name_str) {
                *symbol
            } else {
                let string_table_index = string_table.add_string(class_name_str);
                let symbol = symbol_table.add_string(string_table_index);
                string_map.insert(String::from(class_name_str), symbol);

                let class_table_index = class_table.len();
                class_table.push(TableEntry::Hole);
                let symbol = symbol_table.add_class(class_table_index);

                class_map.insert(String::from(class_name_str), symbol);
                symbol
            };
            let (sub_class_name_str, sub_class_name_symbol) = if *sub_class_name != 0 {
                let sub_class_name_str = class.index_string_table(*sub_class_name);
                let symbol = if let Some(symbol) = class_map.get(sub_class_name_str) {
                    Some(*symbol)
                } else {
                    let string_table_index = string_table.add_string(sub_class_name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(sub_class_name_str), symbol);

                    let class_table_index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(class_table_index);

                    class_map.insert(String::from(sub_class_name_str), symbol);
                    Some(symbol)
                };
                (Some(sub_class_name_str), symbol)
            } else {
                (None, None)
            };

            for function in vtable.functions.iter() {
                let VTableEntry { name, responds_to, signature, bytecode, .. } = function;

                let name_str = class.index_string_table(*name);
                let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                    *symbol
                } else {
                    let string_table_index = string_table.add_string(name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(name_str), symbol);
                    symbol
                };

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

                current_vtable.push((name_symbol, responds_to_symbol, *signature, *bytecode, FunctionValue::Blank));
            }
            vtables_map.insert((class_name_symbol, sub_class_name_symbol), current_vtable);
        }
        
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

    let mut class_parts: Vec<(Symbol, Vec<Symbol>, Vec<MemberInfo>, Vec<SignalInfo>, &ClassFile, Vec<(Symbol, Option<Symbol>)>)> = Vec::new();
    for class in classes.iter() {
        let ClassFile { name, parents, members, signals, signature_table, vtables, .. } = &class;
        let class_name_str = class.index_string_table(*name);
        
        let class_symbol = *class_map.get(class_name_str).unwrap();
        let parent_symbols = parents.iter().map(|p| {
            let p_str = class.index_string_table(*p);
            *class_map.get(p_str).unwrap()
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

        let mut vtables_to_link = Vec::new();

        for vtable in vtables {
            let classfile::VTable { class_name, sub_class_name, .. } = vtable;
            let class_name_str = class.index_string_table(*class_name);
            let class_name_symbol = if let Some(symbol) = class_map.get(class_name_str) {
                *symbol
            } else {
                let string_table_index = string_table.add_string(class_name_str);
                let symbol = symbol_table.add_string(string_table_index);
                string_map.insert(String::from(class_name_str), symbol);

                let class_table_index = class_table.len();
                class_table.push(TableEntry::Hole);
                let symbol = symbol_table.add_class(class_table_index);

                class_map.insert(String::from(class_name_str), symbol);
                symbol
            };
            let (sub_class_name_str, sub_class_name_symbol) = if *sub_class_name != 0 {
                let sub_class_name_str = class.index_string_table(*sub_class_name);
                let symbol = if let Some(symbol) = class_map.get(sub_class_name_str) {
                    Some(*symbol)
                } else {
                    let string_table_index = string_table.add_string(sub_class_name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(sub_class_name_str), symbol);

                    let class_table_index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(class_table_index);

                    class_map.insert(String::from(sub_class_name_str), symbol);
                    Some(symbol)
                };
                (Some(sub_class_name_str), symbol)
            } else {
                (None, None)
            };

            vtables_to_link.push((class_name_symbol, sub_class_name_symbol));
        }
        class_parts.push((class_symbol, parent_symbols, class_members, class_signals, class, vtables_to_link));
    }
    let mut class_parts_to_try_again = Vec::new();
    loop {
        'outer: for class_part in class_parts {
            let (class_symbol, parents, members, signals, class, vtables) = class_part;
            let mut vtables_to_add = Vec::new();
            for (class_name, modifier_class) in vtables.iter() {
                if modifier_class.is_none() {
                    let functions = vtables_map.get(&(*class_name, None)).unwrap();
                    let mut functions_mapper = HashMap::new();
                    let functions = functions.into_iter()
                        .enumerate()
                        .map(|(i, (name_symbol, responds_to, signature, bytecode, value))| {
                            let return_type = convert_type(&class.signature_table[*signature as usize].types[0]);
                            let arguments = class.signature_table[*signature as usize]
                                .types[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            let bytecode = link_bytecode(class.index_bytecode_table(*bytecode));
                            let value = FunctionValue::Bytecode(bytecode);
                            functions_mapper.insert(*name_symbol, i);

                            Function::new(*name_symbol, value, *responds_to, arguments, return_type)

                        })
                        .collect::<Vec<_>>();
                    vtables_to_add.push((*class_name, *modifier_class, VTable::new(functions, functions_mapper)));
                } else {
                    let base_functions = vtables_map.get(&(*class_name, None)).unwrap();
                    for (_, _, _, _, value) in base_functions {
                        if value.is_blank() {
                            class_parts_to_try_again.push(class_part);
                            continue 'outer;
                        }
                    }
                    let derived_functions = vtables_map.get(&(*class_name, *modifier_class)).unwrap();
                    let mut functions_mapper = HashMap::new();
                    let functions = derived_functions.into_iter()
                        .zip(base_functions.into_iter())
                        .enumerate()
                        .map(|(i, (derived, base))| {
                            let return_type = convert_type(&class.signature_table[derived.2 as usize].types[0]);
                            let arguments = class.signature_table[derived.2 as usize]
                                .types[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            let value = if derived.3 == 0 {
                                base.4
                            } else {
                                let bytecode = link_bytecode(class.index_bytecode_table(derived.3));
                                FunctionValue::Bytecode(bytecode)
                            };
                            functions_mapper.insert(derived.0, i);

                            Function::new(derived.0, value, derived.1, arguments, return_type)
                        })
                        .collect::<Vec<_>>();
                    vtables_to_add.push((*class_name, *modifier_class, VTable::new(functions, functions_mapper)));
                }
            }
            let mut class_vtable_mapper = HashMap::new();

            // Loop through vtables to add and put them in the vtable_table
            // store the position in class_vtable_mapper
            // Create new class
            
        }
        if class_parts_to_try_again.len() == 0 {
            break;
        }
        class_parts = class_parts_to_try_again;
    }


    Ok(())
}

fn convert_type(tag: &rowan_shared::TypeTag) -> class::TypeTag {
    match tag {
        rowan_shared::TypeTag::U8 => class::TypeTag::U8,
        rowan_shared::TypeTag::U16 => class::TypeTag::U16,
        rowan_shared::TypeTag::U32 => class::TypeTag::U32,
        rowan_shared::TypeTag::U64 => class::TypeTag::U64,
        rowan_shared::TypeTag::I8 => class::TypeTag::I8,
        rowan_shared::TypeTag::I16 => class::TypeTag::I16,
        rowan_shared::TypeTag::I32 => class::TypeTag::I32,
        rowan_shared::TypeTag::I64 => class::TypeTag::I64,
        rowan_shared::TypeTag::F32 => class::TypeTag::F32,
        rowan_shared::TypeTag::F64 => class::TypeTag::F64,
        rowan_shared::TypeTag::Object => class::TypeTag::Object,
        rowan_shared::TypeTag::Str => class::TypeTag::Str,
        rowan_shared::TypeTag::Void => class::TypeTag::Void,
    }
}

