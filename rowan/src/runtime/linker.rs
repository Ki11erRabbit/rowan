use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use rowan_shared::classfile::{self, ClassFile, Signal, VTableEntry};

use super::{class::{self, Class, MemberInfo, SignalInfo}, jit::JITController, stdlib::{VMClass, VMMember, VMMethod, VMSignal, VMVTable}, tables::{string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue, VTable, VTables}}, Symbol, VTableIndex};

#[derive(Debug)]
pub enum TableEntry<T> {
    Hole,
    Entry(T),
}

#[derive(Debug, Clone)]
pub enum MethodLocation {
    Bytecode(Vec<u8>),
    Native(String),
    Blank,
}




pub fn link_class_files(
    classes: Vec<ClassFile>,
    jit_controller: &mut JITController,
    symbol_table: &mut SymbolTable,
    class_table: &mut Vec<TableEntry<Class>>,
    string_table: &mut StringTable,
    vtables_table: &mut VTables,
    // The first hashmap is the class symbol which the vtable comes from.
    // The second hashmap is the class that has a custom version of the vtable
    // For example, two matching symbols means that that is the vtable of that particular class
    vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Option<Symbol>, Vec<rowan_shared::TypeTag>, MethodLocation, Arc<RwLock<FunctionValue>>)>>>,
    string_map: &mut HashMap<String, Symbol>,
    class_map: &mut HashMap<String, Symbol>,
) -> Result<(Symbol, Symbol), ()> {

    let mut main_class_symbol = None;
    let mut main_class_ready_symbol = None;

    for class in classes.iter() {
        let ClassFile { name, parents, vtables, signals, .. } = class;
        let name_str = class.index_string_table(*name);
        let class_symbol = if let Some(symbol) = class_map.get(name_str) {
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
        if name_str == "Main" {
            main_class_symbol = Some(class_symbol);
        }

        for parent in parents.iter() {
            let name_str = class.index_string_table(*parent);
            let symbol = if let Some(symbol) = class_map.get(name_str) {
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
        }

        for vtable in vtables {
            let classfile::VTable { class_name, sub_class_name, .. } = vtable;

            let mut current_vtable = Vec::new();

            let class_name_str = class.index_string_table(*class_name);
            let vtable_class_name_symbol = if let Some(symbol) = class_map.get(class_name_str) {
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
                let VTableEntry { name, signature, bytecode, .. } = function;

                let name_str = class.index_string_table(*name);
                let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                    *symbol
                } else {
                    let string_table_index = string_table.add_string(name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(name_str), symbol);
                    symbol
                };

                if name_str == "ready" {
                    main_class_ready_symbol = Some(name_symbol);
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

                let signature = class.signature_table[*signature as usize].types.clone();
                let bytecode = if *bytecode == 0 {
                    MethodLocation::Blank
                } else if *bytecode < 0 {
                    let index = bytecode.abs() as u64;
                    let string = class.index_string_table(index).to_string();
                    MethodLocation::Native(string)
                } else {
                    MethodLocation::Bytecode(class.index_bytecode_table(*bytecode).code.clone())
                };
                
                current_vtable.push(
                    (name_symbol, responds_to_symbol, signature, bytecode, Arc::new(RwLock::new(FunctionValue::Blank)))
                );
            }
            vtables_map.entry(vtable_class_name_symbol)
                .and_modify(|map| {
                    map.insert(class_symbol, current_vtable.clone());
                })
                .or_insert({
                    let mut map = HashMap::new();
                    map.insert(class_symbol, current_vtable);
                    map
                });
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

    let mut class_parts: Vec<(Symbol, Symbol, Vec<Symbol>, Vec<MemberInfo>, Vec<SignalInfo>, &ClassFile, Vec<(Symbol, Option<Symbol>)>)> = Vec::new();
    for class in classes.iter() {
        let ClassFile { name, parents, members, signals, signature_table, vtables, .. } = &class;
        let class_name_str = class.index_string_table(*name);
        
        let class_symbol = *class_map.get(class_name_str).unwrap();
        let class_name_symbol = *string_map.get(class_name_str).unwrap();
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
        class_parts.push((class_symbol, class_name_symbol, parent_symbols, class_members, class_signals, class, vtables_to_link));
    }
    let mut class_parts_to_try_again;
    loop {
        class_parts_to_try_again = Vec::new();
        'outer: for class_part in class_parts {
            let (class_symbol, class_name_symbol, parents, members, signals, class, vtables) = class_part;
            let mut vtables_to_add = Vec::new();
            // Source class is one of the parents of the derived class
            // This is used to disambiguate
            // So when this is some, we get the vtable from the class with the same symbol
            for (class_name, source_class) in vtables.iter() {

                if let Some(source_class) = source_class {
                    // In this block, this means that we likely have a diamond inheritance situation
                    // This means that we have 2 copies of the same vtable
                    // We use the class name to get the base vtable
                    // We then use the source class to lookup the same vtable as class name but the implementation by source class
                    let derived_functions = vtables_map.get(class_name).unwrap().get(source_class).unwrap();
                    let base_functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    for (_,_,_,_,value) in base_functions {
                        if value.read().expect("lock poisoned").is_blank() {
                            // We bail if any of base has not yet been linked
                            class_parts_to_try_again.push((class_symbol, class_name_symbol, parents, members, signals, class, vtables));
                            continue 'outer;
                        }
                    }

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (base_name_symbol, base_responds_to, base_signature, base_bytecode, base_value) = base;
                            let (derived_name_symbol, derived_responds_to, derived_signature, derived_bytecode, derived_value) = derived;
                            functions_mapper.insert(*derived_name_symbol, i);
                            let return_type = convert_type(&base_signature[0]);
                            let arguments = base_signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(*derived_name_symbol, derived_value.clone(), *derived_responds_to, arguments, return_type)
                        })
                        .collect::<Vec<_>>();
                    vtables_to_add.push((*class_name, Some(*source_class), VTable::new(functions, functions_mapper)));
                } else if *class_name == class_symbol {
                    // Here we load in the current class' vtable
                    // Nothing fancy happens here other than that we link the bytecode
                    let functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    let mut functions_mapper = HashMap::new();
                    let functions = functions.into_iter()
                        .enumerate()
                        .map(|(i, (name_symbol, responds_to, signature, bytecode, _))| {
                            functions_mapper.insert(*name_symbol, i);


                            let SymbolEntry::StringRef(name_index) = &symbol_table[*name_symbol] else {
                                unreachable!("Expected name symbol to be a string reference");
                            };
                            let name = &string_table[*name_index];

                            let cranelift_sig = jit_controller.create_signature(&signature[1..], &signature[0]);
                            let func_id = jit_controller.declare_function(name, &cranelift_sig).expect("Failed to declare function");

                            let value = match bytecode {
                                MethodLocation::Bytecode(bytecode) => {
                                    let bytecode = link_bytecode(class, &bytecode, string_map, class_map, string_table, symbol_table);
                                    let value = FunctionValue::Bytecode(bytecode, func_id, cranelift_sig);
                                    value
                                }
                                MethodLocation::Native(_string) => {
                                    todo!("load shared object file and pull the function named by `string` from it")
                                }
                                MethodLocation::Blank => {
                                    unreachable!("method location was blank")
                                }
                            };

                            let value = Arc::new(RwLock::new(value));
                            (*name_symbol, *responds_to, signature.clone(), MethodLocation::Blank, value)
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, responds_to, signature, _, value)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, responds_to, arguments, return_type)
                        })
                        .collect::<Vec<_>>();

                    vtables_to_add.push((*class_name, None, VTable::new(functions, functions_mapper)));
                } else if *class_name != class_symbol {
                    // Here we do something similar to if source class is some
                    // we get the base vtable by going class name -> class name
                    // then get the derived vtable by going class name -> class symbol
                    // We also update vtables_map to hold updated function values so that we can link future vtables


                    let derived_functions = vtables_map.get(class_name).unwrap().get(&class_symbol).unwrap();
                    let base_functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    for (_,_,_,_,value) in base_functions {
                        if value.read().expect("lock is poisoned").is_blank() {
                            // We bail if any of base has not yet been linked
                            class_parts_to_try_again.push((class_symbol, class_name_symbol, parents, members, signals, class, vtables));
                            continue 'outer;
                        }
                    }

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (_base_name_symbol, base_responds_to, base_signature, _, _) = base;
                            let (derived_name_symbol, derived_responds_to, derived_signature, derived_bytecode, _) = derived;

                            let SymbolEntry::StringRef(name_index) = &symbol_table[*derived_name_symbol] else {
                                unreachable!("Expected name symbol to be a string reference");
                            };

                            let name = &string_table[*name_index];
                            let cranelift_sig = jit_controller.create_signature(&base_signature[1..], &base_signature[0]);
                            let func_id = jit_controller.declare_function(name, &cranelift_sig).expect("Failed to declare function");


                            let value = match derived_bytecode {
                                MethodLocation::Bytecode(bytecode) => {
                                    let bytecode = link_bytecode(class, &bytecode, string_map, class_map, string_table, symbol_table);
                                    let value = FunctionValue::Bytecode(bytecode, func_id, cranelift_sig);
                                    value
                                }
                                MethodLocation::Native(_string) => {
                                    todo!("load shared object file and pull the function named by `string` from it")
                                }
                                MethodLocation::Blank => {
                                    let value = FunctionValue::Bytecode(Vec::new(), func_id, cranelift_sig);
                                    value
                                }
                            };



                            let value = Arc::new(RwLock::new(value));
                            functions_mapper.insert(*derived_name_symbol, i);

                            (*derived_name_symbol, *base_responds_to, derived_signature.clone(), MethodLocation::Blank, value)
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, responds_to, signature, _, value)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, responds_to, arguments, return_type)
                        })
                        .collect::<Vec<_>>();

                    vtables_to_add.push((*class_name, None, VTable::new(functions, functions_mapper)));
                }
            }
            let mut class_vtable_mapper = HashMap::new();

            // Loop through vtables to add and put them in the vtable_table
            for (class_symbol, source_class, vtable) in vtables_to_add {
                let index = vtables_table.add_vtable(vtable);
                // store the position in class_vtable_mapper
                class_vtable_mapper.insert((class_symbol, source_class), index);
            }

            // a recursive algo that gives every parent/ancestor's vtable to the class
            match add_parent_vtables(&mut class_vtable_mapper, &parents, class_table, symbol_table, &mut HashSet::new()) {
                Err(_) => {
                    // We bail if any of base has not yet been linked
                    class_parts_to_try_again.push((class_symbol, class_name_symbol, parents, members, signals, class, vtables));
                    continue 'outer;
                }
                _ => {},
            }

            // Create new class
            let class = Class::new(class_name_symbol, parents, class_vtable_mapper, members, signals);

            let SymbolEntry::ClassRef(class_index) = &symbol_table[class_symbol] else {
                unreachable!("Class symbol should have been a symbol to a class");
            };

            class_table[*class_index] = TableEntry::Entry(class);
            
        }
        if class_parts_to_try_again.len() == 0 {
            break;
        }
        class_parts = class_parts_to_try_again;
    }


    match (main_class_symbol, main_class_ready_symbol) {
        (Some(main_class_symbol), Some(main_class_ready_symbol)) => Ok((main_class_symbol, main_class_ready_symbol)),
        _ => Err(()),
    }
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


fn link_bytecode(
    class_file: &ClassFile,
    bytecode: &[u8],
    string_map: &mut HashMap<String, Symbol>,
    class_map: &mut HashMap<String, Symbol>,
    string_table: &mut StringTable,
    symbol_table: &mut SymbolTable,
) -> Vec<rowan_shared::bytecode::linked::Bytecode> {
    let mut output = Vec::new();
    let compiled_code: Vec<rowan_shared::bytecode::compiled::Bytecode> =
        rowan_shared::bytecode::compiled::Bytecode::try_from(&mut bytecode.iter()).unwrap();

    for code in compiled_code {
        use rowan_shared::bytecode::*;
        match code {
            compiled::Bytecode::Nop => {
                output.push(linked::Bytecode::Nop);
            }
            compiled::Bytecode::Breakpoint => {
                output.push(linked::Bytecode::Breakpoint);
            }
            compiled::Bytecode::LoadU8(x) => {
                output.push(linked::Bytecode::LoadU8(x));
            }
            compiled::Bytecode::LoadU16(x) => {
                output.push(linked::Bytecode::LoadU16(x));
            }
            compiled::Bytecode::LoadU32(x) => {
                output.push(linked::Bytecode::LoadU32(x));
            }
            compiled::Bytecode::LoadU64(x) => {
                output.push(linked::Bytecode::LoadU64(x));
            }
            compiled::Bytecode::LoadI8(x) => {
                output.push(linked::Bytecode::LoadI8(x));
            }
            compiled::Bytecode::LoadI16(x) => {
                output.push(linked::Bytecode::LoadI16(x));
            }
            compiled::Bytecode::LoadI32(x) => {
                output.push(linked::Bytecode::LoadI32(x));
            }
            compiled::Bytecode::LoadI64(x) => {
                output.push(linked::Bytecode::LoadI64(x));
            }
            compiled::Bytecode::LoadF32(x) => {
                output.push(linked::Bytecode::LoadF32(x));
            }
            compiled::Bytecode::LoadF64(x) => {
                output.push(linked::Bytecode::LoadF64(x));
            }
            compiled::Bytecode::LoadSymbol(index) => {
                let class_name = class_file.index_string_table(index);
                let symbol = class_map.get(class_name).expect("class not loaded yet");

                output.push(linked::Bytecode::LoadSymbol(*symbol as u64));
            }
            compiled::Bytecode::Pop => {
                output.push(linked::Bytecode::Pop);
            }
            compiled::Bytecode::Dup => {
                output.push(linked::Bytecode::Dup);
            }
            compiled::Bytecode::Swap => {
                output.push(linked::Bytecode::Swap);
            }
            compiled::Bytecode::StoreLocal(pos) => {
                output.push(linked::Bytecode::StoreLocal(pos));
            }
            compiled::Bytecode::LoadLocal(pos) => {
                output.push(linked::Bytecode::LoadLocal(pos));
            }
            compiled::Bytecode::StoreArgument(pos) => {
                output.push(linked::Bytecode::StoreArgument(pos));
            }
            compiled::Bytecode::AddInt => {
                output.push(linked::Bytecode::AddInt);
            }
            compiled::Bytecode::SubInt => {
                output.push(linked::Bytecode::SubInt);
            }
            compiled::Bytecode::MulInt => {
                output.push(linked::Bytecode::MulInt);
            }
            compiled::Bytecode::DivSigned => {
                output.push(linked::Bytecode::DivSigned);
            }
            compiled::Bytecode::DivUnsigned => {
                output.push(linked::Bytecode::DivUnsigned);
            }
            compiled::Bytecode::ModSigned => {
                output.push(linked::Bytecode::ModSigned);
            }
            compiled::Bytecode::ModUnsigned => {
                output.push(linked::Bytecode::ModUnsigned);
            }
            compiled::Bytecode::AddFloat => {
                output.push(linked::Bytecode::AddFloat);
            }
            compiled::Bytecode::SubFloat => {
                output.push(linked::Bytecode::SubFloat);
            }
            compiled::Bytecode::MulFloat => {
                output.push(linked::Bytecode::MulFloat);
            }
            compiled::Bytecode::DivFloat => {
                output.push(linked::Bytecode::DivFloat);
            }
            compiled::Bytecode::ModFloat => {
                output.push(linked::Bytecode::ModFloat);
            }
            compiled::Bytecode::SatAddIntUnsigned => {
                output.push(linked::Bytecode::SatAddIntUnsigned);
            }
            compiled::Bytecode::SatSubIntUnsigned => {
                output.push(linked::Bytecode::SatSubIntUnsigned);
            }
            compiled::Bytecode::And => {
                output.push(linked::Bytecode::And);
            }
            compiled::Bytecode::Or => {
                output.push(linked::Bytecode::Or);
            }
            compiled::Bytecode::Xor => {
                output.push(linked::Bytecode::Xor);
            }
            compiled::Bytecode::Not => {
                output.push(linked::Bytecode::Not);
            }
            compiled::Bytecode::Shl => {
                output.push(linked::Bytecode::Shl);
            }
            compiled::Bytecode::AShr => {
                output.push(linked::Bytecode::AShr);
            }
            compiled::Bytecode::LShr => {
                output.push(linked::Bytecode::LShr);
            }
            compiled::Bytecode::Neg => {
                output.push(linked::Bytecode::Neg);
            }
            compiled::Bytecode::EqualSigned => {
                output.push(linked::Bytecode::EqualSigned);
            }
            compiled::Bytecode::NotEqualSigned => {
                output.push(linked::Bytecode::NotEqualSigned);
            }
            compiled::Bytecode::EqualUnsigned => {
                output.push(linked::Bytecode::EqualUnsigned);
            }
            compiled::Bytecode::NotEqualUnsigned => {
                output.push(linked::Bytecode::NotEqualUnsigned);
            }
            compiled::Bytecode::GreaterUnsigned => {
                output.push(linked::Bytecode::GreaterUnsigned);
            }
            compiled::Bytecode::GreaterOrEqualUnsigned => {
                output.push(linked::Bytecode::GreaterOrEqualUnsigned);
            }
            compiled::Bytecode::LessUnsigned => {
                output.push(linked::Bytecode::LessUnsigned);
            }
            compiled::Bytecode::LessOrEqualUnsigned => {
                output.push(linked::Bytecode::LessOrEqualUnsigned);
            }
            compiled::Bytecode::GreaterSigned => {
                output.push(linked::Bytecode::GreaterSigned);
            }
            compiled::Bytecode::GreaterOrEqualSigned => {
                output.push(linked::Bytecode::GreaterOrEqualSigned);
            }
            compiled::Bytecode::LessSigned => {
                output.push(linked::Bytecode::LessSigned);
            }
            compiled::Bytecode::LessOrEqualSigned => {
                output.push(linked::Bytecode::LessOrEqualSigned);
            }
            compiled::Bytecode::EqualFloat => {
                output.push(linked::Bytecode::EqualFloat);
            }
            compiled::Bytecode::NotEqualFloat => {
                output.push(linked::Bytecode::NotEqualFloat);
            }
            compiled::Bytecode::GreaterFloat => {
                output.push(linked::Bytecode::GreaterFloat);
            }
            compiled::Bytecode::GreaterOrEqualFloat => {
                output.push(linked::Bytecode::GreaterOrEqualFloat);
            }
            compiled::Bytecode::LessFloat => {
                output.push(linked::Bytecode::LessFloat);
            }
            compiled::Bytecode::LessOrEqualFloat => {
                output.push(linked::Bytecode::LessOrEqualFloat);
            }
            compiled::Bytecode::Convert(ty) => {
                output.push(linked::Bytecode::Convert(ty));
            }
            compiled::Bytecode::BinaryConvert(ty) => {
                output.push(linked::Bytecode::BinaryConvert(ty));
            }
            compiled::Bytecode::CreateArray(ty) => {
                output.push(linked::Bytecode::CreateArray(ty));
            }
            compiled::Bytecode::ArrayGet(ty) => {
                output.push(linked::Bytecode::ArrayGet(ty));
            }
            compiled::Bytecode::ArraySet(ty) => {
                output.push(linked::Bytecode::ArraySet(ty));
            }
            compiled::Bytecode::NewObject(index) => {
                let class_str = class_file.index_string_table(index);
                let symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet"); 

                output.push(linked::Bytecode::NewObject(symbol as u64));
            }
            compiled::Bytecode::GetField(index, _, pos) => {
                let class_str = class_file.index_string_table(index);
                let symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet"); 

                output.push(linked::Bytecode::GetField(symbol as u64, 0, pos));
            }
            compiled::Bytecode::SetField(index, _, pos) => {
                let class_str = class_file.index_string_table(index);
                let symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet"); 

                output.push(linked::Bytecode::SetField(symbol as u64, 0, pos));
            }
            compiled::Bytecode::IsA(index) => {
                let class_str = class_file.index_string_table(index);
                let symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet"); 

                output.push(linked::Bytecode::IsA(symbol as u64));
            }
            compiled::Bytecode::InvokeVirt(class_index, source_class, method_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let source_class = if source_class != 0 {
                    let class_str = class_file.index_string_table(source_class);
                    let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");
                    Some(class_symbol as u64)
                } else {
                    None
                };

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                }; 

                output.push(linked::Bytecode::InvokeVirt(class_symbol as u64, source_class, method_symbol as u64));
            }
            compiled::Bytecode::InvokeVirtTail(class_index, source_class, method_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let source_class = if source_class == 0 {
                    let class_str = class_file.index_string_table(source_class);
                    let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");
                    Some(class_symbol as u64)
                } else {
                    None
                };

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                }; 

                output.push(linked::Bytecode::InvokeVirtTail(class_symbol as u64, source_class, method_symbol as u64));
            }
            compiled::Bytecode::InvokeStatic(class_index, method_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                output.push(linked::Bytecode::InvokeStatic(class_symbol as u64, method_symbol as u64));
            }
            compiled::Bytecode::InvokeStaticTail(class_index, method_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                output.push(linked::Bytecode::InvokeStaticTail(class_symbol as u64, method_symbol as u64));
            }
            compiled::Bytecode::GetStaticMethod(class_index, method_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                output.push(linked::Bytecode::GetStaticMethod(class_symbol as u64, method_symbol as u64));
            }
            compiled::Bytecode::EmitSignal(class_index, name_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let signal_str = class_file.index_string_table(name_index);
                let signal_symbol: Symbol = if let Some(index) = string_map.get(signal_str) {
                    *index
                } else {
                    let index = string_table.add_string(signal_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                output.push(linked::Bytecode::EmitSignal(class_symbol as u64, signal_symbol as u64));
            }
            compiled::Bytecode::EmitStaticSignal(class_index, name_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let signal_str = class_file.index_string_table(name_index);
                let signal_symbol: Symbol = if let Some(index) = string_map.get(signal_str) {
                    *index
                } else {
                    let index = string_table.add_string(signal_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                output.push(linked::Bytecode::EmitStaticSignal(class_symbol as u64, signal_symbol as u64));
            }
            compiled::Bytecode::ConnectSignal(signal_index, class_index, class_index2, method_index) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let class_str = class_file.index_string_table(class_index2);
                let class_symbol2: Symbol = *class_map.get(class_str).expect("Class not loaded yet");

                let signal_str = class_file.index_string_table(signal_index);
                let signal_symbol: Symbol = if let Some(index) = string_map.get(signal_str) {
                    *index
                } else {
                    let index = string_table.add_string(signal_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                }; 

                output.push(
                    linked::Bytecode::ConnectSignal(
                        signal_symbol as u64,
                        class_symbol as u64,
                        class_symbol2 as u64,
                        method_symbol as u64,
                    ));
            }
            compiled::Bytecode::DisconnectSignal(signal_index, method_index) => {
                let signal_str = class_file.index_string_table(signal_index);
                let signal_symbol: Symbol = if let Some(index) = string_map.get(signal_str) {
                    *index
                } else {
                    let index = string_table.add_string(signal_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                };

                let method_str = class_file.index_string_table(method_index);
                let method_symbol: Symbol = if let Some(index) = string_map.get(method_str) {
                    *index
                } else {
                    let index = string_table.add_string(method_str);
                    let symbol = symbol_table.add_string(index);
                    symbol
                }; 

                output.push(
                    linked::Bytecode::DisconnectSignal(
                        signal_symbol as u64,
                        method_symbol as u64,
                    ));
            }
            compiled::Bytecode::GetStrRef(str_index) => {
                let str_str = class_file.index_string_table(str_index);
                let str_index = if let Some(index) = string_map.get(str_str) {
                    *index
                } else {
                    let str_index = string_table.add_string(str_str);
                    let symbol = symbol_table.add_string(str_index);
                    symbol
                };

                output.push(linked::Bytecode::GetStrRef(str_index as u64));
            }
            compiled::Bytecode::Return => {
                output.push(linked::Bytecode::Return);
            }
            compiled::Bytecode::ReturnVoid => {
                output.push(linked::Bytecode::ReturnVoid);
            }
            compiled::Bytecode::RegisterException(index, offset) => {
                let class_name = class_file.index_string_table(index);
                let symbol = class_map.get(class_name).expect("class not loaded yet");

                output.push(linked::Bytecode::RegisterException(*symbol as u64, offset));
            }
            compiled::Bytecode::UnregisterException(index) => {
                let class_name = class_file.index_string_table(index);
                let symbol = class_map.get(class_name).expect("class not loaded yet");

                output.push(linked::Bytecode::UnregisterException(*symbol as u64));
            }
            compiled::Bytecode::Throw => {
                output.push(linked::Bytecode::Throw);
            }
            compiled::Bytecode::StartBlock(x) => {
                output.push(linked::Bytecode::StartBlock(x));
            }
            compiled::Bytecode::Goto(x) => {
                output.push(linked::Bytecode::Goto(x));
            }
            compiled::Bytecode::If(x, y) => {
                output.push(linked::Bytecode::If(x, y));
            }
            compiled::Bytecode::Switch(branches, default) => {
                output.push(linked::Bytecode::Switch(branches, default));
            }
        }
    }

    // TODO: perform semantic analysis on bytecode to ensure that references are not messed with
    output
}

pub fn link_vm_classes(
    classes: Vec<VMClass>,
    jit_controller: &mut JITController,
    symbol_table: &mut SymbolTable,
    class_table: &mut Vec<TableEntry<Class>>,
    string_table: &mut StringTable,
    vtables_table: &mut VTables,
    // The first hashmap is the class symbol which the vtable comes from.
    // The second hashmap is the class that has a custom version of the vtable
    // For example, two matching symbols means that that is the vtable of that particular class
    vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Option<Symbol>, Vec<rowan_shared::TypeTag>, MethodLocation, Arc<RwLock<FunctionValue>>)>>>,
    string_map: &mut HashMap<String, Symbol>,
    class_map: &mut HashMap<String, Symbol>
) {

    let mut class_parts: Vec<(Symbol, Vec<Symbol>, Vec<MemberInfo>, Vec<SignalInfo>, Vec<(Symbol, Option<Symbol>)>)> = Vec::new();
    for class in classes {
        let VMClass {
            name,
            parents,
            vtables,
            members,
            signals
        } = class;

        let class_symbol = {
            if let Some(symbol) = class_map.get(name) {
                *symbol
            } else {
                let index = string_table.add_static_string(name);
                let symbol = symbol_table.add_string(index);

                string_map.insert(String::from(name), symbol);

                if let Some(symbol) = class_map.get(name) {
                    *symbol
                } else {
                    let index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(index);
                    class_map.insert(String::from(name), symbol);

                    symbol
                }
            }
        };

        let mut parent_symbols = Vec::new();
        for parent in parents {
            let symbol = if let Some(symbol) = class_map.get(parent) {
                *symbol
            } else {
                let index = string_table.add_static_string(parent);
                let symbol = symbol_table.add_string(index);

                string_map.insert(String::from(parent), symbol);

                if let Some(symbol) = class_map.get(parent) {
                    *symbol
                } else {
                    let index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(index);
                    class_map.insert(String::from(parent), symbol);

                    symbol
                }
            };
            parent_symbols.push(symbol);
        }

        let mut class_signals = Vec::new();
        for signal in signals {
            let VMSignal { name, is_static, arguments } = signal;

            let name_symbol = if let Some(symbol) = string_map.get(name) {
                *symbol
            } else {
                let index = string_table.add_static_string(name);
                let symbol = symbol_table.add_string(index);
                string_map.insert(String::from(name), symbol);
                symbol
            };

            let arguments = arguments.iter().map(convert_type).collect::<Vec<_>>();

            class_signals.push(SignalInfo::new(name_symbol, is_static, arguments));
        }

        let mut class_members = Vec::new();
        for member in members {
            let VMMember { name, ty } = member;
            let name_symbol = if let Some(symbol) = string_map.get(name) {
                *symbol
            } else {
                let index = string_table.add_static_string(name);
                symbol_table.add_string(index)
            };

            let ty = convert_type(&ty);

            class_members.push(MemberInfo::new(name_symbol, ty));
        }
        let mut vtables_to_link = Vec::new();
        for vtable in vtables {
            let VMVTable { class, source_class, methods } = vtable;

            let vtable_class_name_symbol = if let Some(symbol) = class_map.get(class) {
                *symbol
            } else {
                let index = string_table.add_static_string(class);
                let symbol = symbol_table.add_string(index);

                string_map.insert(String::from(class), symbol);

                if let Some(symbol) = class_map.get(class) {
                    *symbol
                } else {
                    let index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(index);
                    class_map.insert(String::from(class), symbol);

                    symbol
                }
            };
            let source_class_name = if let Some(source_class) = source_class {
                if let Some(symbol) = class_map.get(source_class) {
                    Some(*symbol)
                } else {
                    let index = string_table.add_static_string(source_class);
                    let symbol = symbol_table.add_string(index);

                    string_map.insert(String::from(source_class), symbol);

                    if let Some(symbol) = class_map.get(source_class) {
                        Some(*symbol)
                    } else {
                        let index = class_table.len();
                        class_table.push(TableEntry::Hole);
                        let symbol = symbol_table.add_class(index);
                        class_map.insert(String::from(source_class), symbol);

                        Some(symbol)
                    }
                }
            } else {
                None
            };

            let mut current_vtable = Vec::new();
            for method in methods {
                let VMMethod { name, fn_pointer, signature } = method;

                let name_symbol = if let Some(symbol) = string_map.get(name) {
                    *symbol
                } else {
                    let index = string_table.add_static_string(name);
                    let symbol = symbol_table.add_string(index);

                    string_map.insert(String::from(name), symbol);

                    symbol
                };
                let cranelift_sig = jit_controller.create_signature(&signature[1..], &signature[0]);

                let value = FunctionValue::Builtin(fn_pointer, cranelift_sig);
                let value = Arc::new(RwLock::new(value));
                current_vtable.push((name_symbol, None, signature, MethodLocation::Blank, value));
            }
            vtables_map.entry(vtable_class_name_symbol)
                .and_modify(|map| {
                    map.insert(class_symbol, current_vtable.clone());
                })
                .or_insert({
                    let mut map = HashMap::new();
                    map.insert(class_symbol, current_vtable);
                    map
                });

            vtables_to_link.push((vtable_class_name_symbol, source_class_name));
        }
        class_parts.push((class_symbol, parent_symbols, class_members, class_signals, vtables_to_link));
    }

    let mut class_parts_to_try_again;
    loop {
        class_parts_to_try_again = Vec::new();

        'outer: for class_part in class_parts {
            let (class_symbol, parents, members, signals, vtables) = class_part;
            let mut vtables_to_add = Vec::new();
            // Source class is one of the parents of the derived class
            // This is used to disambiguate
            // So when this is some, we get the vtable from the class with the same symbol
            for (class_name, source_class) in vtables.iter() {
                if let Some(source_class) = source_class {
                    // In this block, this means that we likely have a diamond inheritance situation
                    // This means that we have 2 copies of the same vtable
                    // We use the class name to get the base vtable
                    // We then use the source class to lookup the same vtable as class name but the implementation by source class
                    let derived_functions = vtables_map.get(class_name).unwrap().get(source_class).unwrap();
                    let base_functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (base_name_symbol, base_responds_to, base_signature, base_bytecode, base_value) = base;
                            let (derived_name_symbol, derived_responds_to, derived_signature, derived_bytecode, derived_value) = base;
                            functions_mapper.insert(*derived_name_symbol, i);
                            let return_type = convert_type(&base_signature[0]);
                            let arguments = base_signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(*derived_name_symbol, derived_value.clone(), *derived_responds_to, arguments, return_type)
                        })
                        .collect::<Vec<_>>();
                    vtables_to_add.push((*class_name, Some(*source_class), VTable::new(functions, functions_mapper)));
                } else if *class_name == class_symbol {
                    // Here we load in the current class' vtable
                    // Nothing fancy happens here other than that we link the bytecode
                    let functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    let mut functions_mapper = HashMap::new();
                    let functions = functions.into_iter()
                        .enumerate()
                        .map(|(i, (name_symbol, responds_to, signature, bytecode, value))| {
                            functions_mapper.insert(*name_symbol, i);

                            (*name_symbol, *responds_to, signature.clone(), MethodLocation::Blank, value.clone())
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, responds_to, signature, _, value)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, responds_to, arguments, return_type)
                        })
                        .collect::<Vec<_>>();

                    vtables_to_add.push((*class_name, None, VTable::new(functions, functions_mapper)));
                } else if *class_name != class_symbol {
                    // Here we do something similar to if source class is some
                    // we get the base vtable by going class name -> class name
                    // then get the derived vtable by going class name -> class symbol
                    // We also update vtables_map to hold updated function values so that we can link future vtables

                    let derived_functions = vtables_map.get(class_name).unwrap().get(&class_symbol).unwrap();
                    let base_functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    for (_,_,_,_,value) in base_functions {
                        if value.read().expect("lock is poisoned").is_blank() {
                            // We bail if any of base has not yet been linked
                            class_parts_to_try_again.push((class_symbol, parents, members, signals, vtables));
                            continue 'outer;
                        }
                    }

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (_base_name_symbol, base_responds_to, _base_signature, _, _) = base;
                            let (derived_name_symbol, _derived_responds_to, derived_signature, _, value) = derived;
                            functions_mapper.insert(*derived_name_symbol, i);

                            (*derived_name_symbol, *base_responds_to, derived_signature.clone(), MethodLocation::Blank, value.clone())
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, responds_to, signature, _, value)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, responds_to, arguments, return_type)
                        })
                        .collect::<Vec<_>>();

                    vtables_to_add.push((*class_name, None, VTable::new(functions, functions_mapper)));
                }
            }
            let mut class_vtable_mapper = HashMap::new();

            // Loop through vtables to add and put them in the vtable_table
            for (class_symbol, source_class, vtable) in vtables_to_add {
                let index = vtables_table.add_vtable(vtable);
                // store the position in class_vtable_mapper
                class_vtable_mapper.insert((class_symbol, source_class), index);
            }

            // a recursive algo that gives every parent/ancestor's vtable to the class
            match add_parent_vtables(&mut class_vtable_mapper, &parents, class_table, symbol_table, &mut HashSet::new()) {
                Err(_) => {
                    // We bail if any of base has not yet been linked
                    class_parts_to_try_again.push((class_symbol, parents, members, signals, vtables));
                    continue 'outer;
                }
                _ => {},
            }
            

            // Create new class
            let class = Class::new(class_symbol, parents, class_vtable_mapper, members, signals);

            let SymbolEntry::ClassRef(class_index) = &symbol_table[class_symbol] else {
                unreachable!("Class symbol should have been a symbol to a class");
            };

            class_table[*class_index] = TableEntry::Entry(class);
            
        }
        if class_parts_to_try_again.len() == 0 {
            break;
        }
        class_parts = class_parts_to_try_again;
    }
}

fn add_parent_vtables(
    class_table_mapper: &mut HashMap<(Symbol, Option<Symbol>), VTableIndex>,
    parents: &[Symbol],
    class_table: &Vec<TableEntry<Class>>,
    symbol_table: &SymbolTable,
    seen_classes: &mut HashSet<Symbol>,
) -> Result<(), ()> {

    for parent in parents.iter() {
        if seen_classes.contains(parent) {
            continue;
        }
        seen_classes.insert(*parent);
        
        let SymbolEntry::ClassRef(index) = symbol_table[*parent] else {
            unreachable!("Class was not a class");
        };

        let TableEntry::Entry(class) = &class_table[index] else {
            return Err(());
        };

        for (k, v) in class.vtables.iter() {
            class_table_mapper.entry(*k).or_insert(*v);
        }

       add_parent_vtables(class_table_mapper, &class.parents, class_table, symbol_table, seen_classes)?;
    }
    
    Ok(())
}
