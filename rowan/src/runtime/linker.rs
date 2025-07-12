use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use cranelift::prelude::Signature;
use rowan_shared::classfile::{self, ClassFile, VTableEntry};
use rowan_shared::TypeTag;
use crate::runtime::class::{ClassMember, ClassMemberData};
use crate::runtime::jit::JITCompiler;
use crate::runtime::tables::native_object_table::NativeObjectTable;
use super::{class::{self, Class, MemberInfo}, jit::JITController, core::{VMClass, VMMember, VMMethod, VMVTable}, tables::{string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue, VTable, VTables}}, Context, Symbol, VTableIndex};

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
    class_locations: Vec<PathBuf>,
    jit_controller: &mut JITController,
    jit_compiler: &mut JITCompiler,
    symbol_table: &mut SymbolTable,
    class_table: &mut Vec<TableEntry<Class>>,
    string_table: &mut StringTable,
    vtables_table: &mut VTables,
    // The first hashmap is the class symbol which the vtable comes from.
    // The second hashmap is the class that has a custom version of the vtable
    // For example, two matching symbols means that that is the vtable of that particular class
    vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Vec<rowan_shared::TypeTag>, MethodLocation, Arc<RwLock<FunctionValue>>, Signature)>>>,
    string_map: &mut HashMap<String, Symbol>,
    class_map: &mut HashMap<String, Symbol>,
    library_table: &mut NativeObjectTable,
) -> Result<(Symbol, Symbol), ()> {

    let mut main_class_symbol = None;
    let mut main_method_symbol = None;

    for class in classes.iter() {
        let ClassFile { name, parents, vtables, .. } = class;
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
            let (_sub_class_name_str, _sub_class_name_symbol) = if *sub_class_name != 0 {
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

                let sig = jit_controller.create_signature(&signature[1..], &signature[0]);

                current_vtable.push(
                    (name_symbol, signature, bytecode, Arc::new(RwLock::new(FunctionValue::Blank)), sig)
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
    }

    let mut class_parts: Vec<(&str, PathBuf, Symbol, Symbol, Vec<Symbol>, Vec<MemberInfo>, Vec<(Symbol, Vec<TypeTag>, MethodLocation)>, &ClassFile, Vec<(Symbol, Option<Symbol>)>, Vec<ClassMember>, Vec<u8>)> = Vec::new();
    for (class, location) in classes.iter().zip(class_locations.into_iter()) {
        let ClassFile { name, parents, members, static_methods, vtables, static_members, static_init, .. } = &class;
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

        let mut static_method_functions = Vec::new();
        for function in static_methods.functions.iter() {
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

            if name_str.ends_with("main") {
                main_method_symbol = Some(name_symbol);
            }


            let signature = class.signature_table[*signature as usize].types.clone();
            //println!("{}'s signature: {:?}", name_str, signature);
            let function = if *bytecode == 0 {
                MethodLocation::Blank
            } else if *bytecode < 0 {
                let string = name_str.replace("::", "__")
                    .replace("-", "_dash_");
                MethodLocation::Native(string)
            } else {
                MethodLocation::Bytecode(class.index_bytecode_table(*bytecode).code.clone())
            };
            static_method_functions.push((name_symbol, signature, function))
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

        let static_members = static_members.iter()
            .map(|m| {
                let rowan_shared::classfile::Member { name, type_tag } = m;
                let name_str = class.index_string_table(*name);
                let data = match type_tag {
                    TypeTag::U8 | TypeTag::I8 => ClassMemberData::Byte(0),
                    TypeTag::U16 | TypeTag::I16 => ClassMemberData::Short(0),
                    TypeTag::U32 | TypeTag::I32 => ClassMemberData::Int(0),
                    TypeTag::U64 | TypeTag::I64 => ClassMemberData::Long(0),
                    TypeTag::F32 => ClassMemberData::Float(0.0),
                    TypeTag::F64 => ClassMemberData::Double(0.0),
                    _ => ClassMemberData::Object(std::ptr::null_mut()),
                };
                let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                    *symbol
                } else {
                    let string_table_index = string_table.add_string(name_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(name_str), symbol);
                    symbol
                };

                ClassMember {
                    name: name_symbol,
                    data,
                }
        }).collect::<Vec<_>>();


        let static_init = if *static_init != 0 {
            class.index_bytecode_table(*static_init).code.clone()
        } else {
            Vec::new()
        };

        class_parts.push((class_name_str, location, class_symbol, class_name_symbol, parent_symbols, class_members, static_method_functions, class, vtables_to_link, static_members, static_init));
    }
    let mut class_parts_to_try_again;
    loop {
        class_parts_to_try_again = Vec::new();
        'outer: for class_part in class_parts {
            let (class_name_str, mut location, class_symbol, class_name_symbol, parents, members, static_methods, class, vtables, static_members, static_init) = class_part;
            let mut vtables_to_add = Vec::new();
            // Source class is one of the parents of the derived class
            // This is used to disambiguate
            // So when this is some, we get the vtable from the class with the same symbol
            for (class_name, source_class) in vtables.iter() {

                if let Some(source_class) = source_class {
                    // In this block, this means that we likely have a diamond inheritance situation
                    // This means that we have 2 copies of the same vtable
                    // We use the class name to get the base vtable
                    // We then use the source class to look up the same vtable as class name but the implementation by source class
                    let derived_functions = vtables_map.get(class_name).unwrap().get(source_class).unwrap();
                    let base_functions = vtables_map.get(class_name).unwrap().get(class_name).unwrap();

                    for (_,_,_,value, _) in base_functions {
                        if value.read().expect("lock poisoned").is_blank() {
                            // We bail if any of base has not yet been linked
                            class_parts_to_try_again.push((class_name_str, location, class_symbol, class_name_symbol, parents, members, static_methods, class, vtables, static_members, static_init));
                            continue 'outer;
                        }
                    }

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (base_name_symbol,  base_signature, base_bytecode, base_value, sig) = base;
                            let (derived_name_symbol,  derived_signature, derived_bytecode, derived_value, _) = derived;
                            functions_mapper.insert(*derived_name_symbol, i);
                            let return_type = convert_type(&base_signature[0]);
                            let arguments = base_signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(*derived_name_symbol, derived_value.clone(), arguments, return_type, sig.clone())
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
                        .map(|(i, (name_symbol, signature, bytecode, _, sig))| {
                            functions_mapper.insert(*name_symbol, i);


                            let SymbolEntry::StringRef(name_index) = &symbol_table[*name_symbol] else {
                                unreachable!("Expected name symbol to be a string reference");
                            };
                            let name = &string_table[*name_index];


                            let func_id = jit_controller.declare_function(name, &sig).expect("Failed to declare function");

                            let (value, sig) = match bytecode {
                                MethodLocation::Bytecode(bytecode) => {
                                    let bytecode = link_bytecode(class, &bytecode, string_map, class_map, string_table, symbol_table, class_table);
                                    let value = FunctionValue::Bytecode(bytecode, func_id);
                                    (value, sig)
                                }
                                MethodLocation::Native(string) => {
                                    let name = class_name_str.split("::").collect::<Vec<&str>>().last().unwrap().to_string();
                                    let name = add_library_mod(&name);

                                    location.push(name);

                                    let value = if let Some(library) = library_table.get_mut(&location.to_str().unwrap()) {
                                        let symbol = unsafe {
                                            let symbol = library.get::<*const ()>(string.as_bytes()).expect("TODO: handle missing function reference");
                                            *symbol
                                        };
                                        FunctionValue::Native(symbol)
                                    } else {
                                        let (symbol, lib) = unsafe {
                                            let lib = libloading::Library::new(&location).expect("Handle Missing library");
                                            let symbol = lib.get::<*const ()>(string.as_bytes()).expect("TODO: handle missing function reference");

                                            (*symbol, lib)
                                        };
                                        library_table.insert(location.to_str().unwrap().to_string(), lib);
                                        FunctionValue::Native(symbol)
                                    };

                                    location.pop();

                                    (value, sig)
                                }
                                MethodLocation::Blank => {
                                    unreachable!("method location was blank")
                                }
                            };

                            let value = Arc::new(RwLock::new(value));
                            (*name_symbol, signature.clone(), MethodLocation::Blank, value, sig.clone())
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, signature, _, value, sig)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, arguments, return_type, sig)
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

                    for (_,_,_,value, _) in base_functions {
                        if value.read().expect("lock is poisoned").is_blank() {
                            // We bail if any of base has not yet been linked
                            class_parts_to_try_again.push((class_name_str, location, class_symbol, class_name_symbol, parents, members, static_methods, class, vtables, static_members, static_init));
                            continue 'outer;
                        }
                    }

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (_base_name_symbol, base_signature, _, base_value, sig) = base;
                            let (derived_name_symbol, derived_signature, derived_bytecode, _, _) = derived;

                            let SymbolEntry::StringRef(name_index) = &symbol_table[*derived_name_symbol] else {
                                unreachable!("Expected name symbol to be a string reference");
                            };

                            let value = match derived_bytecode {
                                MethodLocation::Bytecode(bytecode) => {
                                    let name = &string_table[*name_index];
                                    let func_id = jit_controller.declare_function(name, &sig).expect("Failed to declare function");

                                    let bytecode = link_bytecode(class, &bytecode, string_map, class_map, string_table, symbol_table, class_table);
                                    let value = FunctionValue::Bytecode(bytecode, func_id);
                                    Arc::new(RwLock::new(value))
                                }
                                MethodLocation::Native(string) => {
                                    let name = class_name_str.split("::").collect::<Vec<&str>>().last().unwrap().to_string();
                                    let name = add_library_mod(&name);

                                    location.push(name);

                                    let value = if let Some(library) = library_table.get_mut(&location.to_str().unwrap()) {
                                        let symbol = unsafe {
                                            let symbol = library.get::<*const ()>(string.as_bytes()).expect("TODO: handle missing function reference");
                                            *symbol
                                        };
                                        FunctionValue::Native(symbol)
                                    } else {
                                        let (symbol, lib) = unsafe {
                                            let lib = libloading::Library::new(&location).expect("Handle Missing library");
                                            let symbol = lib.get::<*const ()>(string.as_bytes()).expect("TODO: handle missing function reference");

                                            (*symbol, lib)
                                        };
                                        library_table.insert(location.to_str().unwrap().to_string(), lib);
                                        FunctionValue::Native(symbol)
                                    };

                                    location.pop();

                                    Arc::new(RwLock::new(value))
                                }
                                MethodLocation::Blank => {
                                    base_value.clone()
                                }
                            };

                            functions_mapper.insert(*derived_name_symbol, i);

                            (*derived_name_symbol, derived_signature.clone(), MethodLocation::Blank, value, sig.clone())
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, signature, _, value, sig)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, arguments, return_type, sig)
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
                    class_parts_to_try_again.push((class_name_str, location, class_symbol, class_name_symbol, parents, members, static_methods, class, vtables, static_members, static_init));
                    continue 'outer;
                }
                _ => {},
            }

            let mut static_method_mapper = HashMap::new();
            let functions = static_methods.into_iter()
                .enumerate()
                .map(|(i, (name, sig, location))| {
                    let name_symbol = name;
                    static_method_mapper.insert(name_symbol, i);
                    let SymbolEntry::StringRef(name_index) = &symbol_table[name] else {
                        unreachable!("Expected name symbol to be a string reference");
                    };
                    let name = &string_table[*name_index];
                    let cranelift_sig = jit_controller.create_signature(&sig[1..], &sig[0]);
                    let func_id = jit_controller.declare_function(name, &cranelift_sig).expect("Failed to declare function");

                    let return_type = convert_type(&sig[0]);
                    let arguments = sig[1..]
                        .iter()
                        .map(convert_type)
                        .collect::<Vec<_>>();

                    let (value, sig) = match location {
                        MethodLocation::Blank => panic!("we should be bytecode"),
                        MethodLocation::Native(_) => panic!("we should be bytecode"),
                        MethodLocation::Bytecode(code) => {
                            let bytecode = link_bytecode(class, &code, string_map, class_map, string_table, symbol_table, class_table);
                            let value = FunctionValue::Bytecode(bytecode, func_id);
                            (Arc::new(RwLock::new(value)), cranelift_sig)
                        }
                    };

                    Function::new(name_symbol, value, arguments, return_type, sig)
                })
                .collect::<Vec<_>>();

            let vtable = VTable::new(functions, static_method_mapper);
            let vtable_index = vtables_table.add_vtable(vtable);

            let static_init = if !static_init.is_empty() {
                let bytecode = link_bytecode(&class, &static_init, string_map, class_map, string_table, symbol_table, class_table);

                let cranelift_sig = jit_controller.create_signature(&[], &TypeTag::Void);
                let func_id = jit_controller.declare_function(format!("{class_name_str}::static-init").as_str(), &cranelift_sig).expect("Failed to declare function");
                let static_init = jit_compiler.compile_bytecode(&bytecode, &mut jit_controller.module, func_id).unwrap();
                let static_init = unsafe { std::mem::transmute::<_, fn(&mut Context)>(static_init) };

                static_init
            } else {
                |_: &mut Context| {}
            };


            // Create new class
            let class = Class::new(class_name_symbol, parents, class_vtable_mapper, members, vtable_index, static_members, static_init);

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


    match (main_class_symbol, main_method_symbol) {
        (Some(main_class_symbol), Some(main_method_symbol)) => Ok((main_class_symbol, main_method_symbol)),
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
        TypeTag::Native => unreachable!("Native Type not ABI compatible"),
    }
}


fn link_bytecode(
    class_file: &ClassFile,
    bytecode: &[u8],
    string_map: &mut HashMap<String, Symbol>,
    class_map: &mut HashMap<String, Symbol>,
    string_table: &mut StringTable,
    symbol_table: &mut SymbolTable,
    class_table: &mut Vec<TableEntry<Class>>,
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
            compiled::Bytecode::GetField(index, _, pos, tag) => {
                let class_str = class_file.index_string_table(index);
                let symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet"); 

                output.push(linked::Bytecode::GetField(symbol as u64, 0, pos, tag));
            }
            compiled::Bytecode::SetField(index, _, pos, tag) => {
                let class_str = class_file.index_string_table(index);
                let symbol: Symbol = *class_map.get(class_str).expect("Class not loaded yet"); 

                output.push(linked::Bytecode::SetField(symbol as u64, 0, pos, tag));
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
                if method_symbol == 71 {
                    panic!("weird method symbol");
                }

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
            compiled::Bytecode::GetStaticMember(class_index, member_index, type_tag) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol = if let Some(symbol) = class_map.get(class_str) {
                    *symbol
                } else {
                    let string_table_index = string_table.add_string(class_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(class_str), symbol);

                    let class_table_index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(class_table_index);

                    class_map.insert(String::from(class_str), symbol);
                    symbol
                };

                output.push(linked::Bytecode::GetStaticMember(class_symbol as u64, member_index, type_tag));
            }
            compiled::Bytecode::SetStaticMember(class_index, member_index, type_tag) => {
                let class_str = class_file.index_string_table(class_index);
                let class_symbol = if let Some(symbol) = class_map.get(class_str) {
                    *symbol
                } else {
                    let string_table_index = string_table.add_string(class_str);
                    let symbol = symbol_table.add_string(string_table_index);
                    string_map.insert(String::from(class_str), symbol);

                    let class_table_index = class_table.len();
                    class_table.push(TableEntry::Hole);
                    let symbol = symbol_table.add_class(class_table_index);

                    class_map.insert(String::from(class_str), symbol);
                    symbol
                };

                output.push(linked::Bytecode::SetStaticMember(class_symbol as u64, member_index, type_tag));
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
    vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Vec<rowan_shared::TypeTag>, MethodLocation, Arc<RwLock<FunctionValue>>, Signature)>>>,
    string_map: &mut HashMap<String, Symbol>,
    class_map: &mut HashMap<String, Symbol>
) {

    let mut class_parts: Vec<(Symbol, Vec<Symbol>, Vec<MemberInfo>, Vec<(Symbol, Vec<TypeTag>, Arc<RwLock<FunctionValue>>, Signature)>, Vec<(Symbol, Option<Symbol>)>, Vec<ClassMember>)> = Vec::new();
    for class in classes {
        let VMClass {
            name,
            parents,
            vtables,
            members,
            static_methods,
            static_members,
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

        let static_methods = static_methods.iter()
            .map(|method|{
                let cranelift_sig = jit_controller.create_signature(&method.signature[1..], &method.signature[0]);

                let index = string_table.add_static_string(method.name);
                let symbol = symbol_table.add_string(index);

                let value = FunctionValue::Builtin(method.fn_pointer);
                (symbol, method.signature.clone(), Arc::new(RwLock::new(value)), cranelift_sig)
            })
            .collect::<Vec<_>>();


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

                let value = FunctionValue::Builtin(fn_pointer);
                let value = Arc::new(RwLock::new(value));
                current_vtable.push((name_symbol, signature, MethodLocation::Blank, value, cranelift_sig));
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

        let static_members = static_members.into_iter()
            .map(|m| {
                let VMMember { name, ty } = m;
                let name_symbol = if let Some(symbol) = string_map.get(name) {
                    *symbol
                } else {
                    let index = string_table.add_static_string(name);
                    symbol_table.add_string(index)
                };

                let data = match ty {
                    TypeTag::U8 | TypeTag::I8 => ClassMemberData::Byte(0),
                    TypeTag::U16 | TypeTag::I16 => ClassMemberData::Short(0),
                    TypeTag::U32 | TypeTag::I32 => ClassMemberData::Int(0),
                    TypeTag::U64 | TypeTag::I64 => ClassMemberData::Long(0),
                    TypeTag::F32 => ClassMemberData::Float(0.0),
                    TypeTag::F64 => ClassMemberData::Double(0.0),
                    _ => ClassMemberData::Object(std::ptr::null_mut()),
                };

                ClassMember {
                    name: name_symbol,
                    data,
                }
            }).collect::<Vec<_>>();

        class_parts.push((class_symbol, parent_symbols, class_members, static_methods, vtables_to_link, static_members));
    }

    let mut class_parts_to_try_again;
    loop {
        class_parts_to_try_again = Vec::new();

        'outer: for class_part in class_parts {
            let (class_symbol, parents, members, static_methods, vtables, static_members) = class_part;
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
                            let (base_name_symbol,  base_signature, base_bytecode, base_value, sig) = base;
                            let (derived_name_symbol,  derived_signature, derived_bytecode, derived_value, _) = base;
                            functions_mapper.insert(*derived_name_symbol, i);
                            let return_type = convert_type(&base_signature[0]);
                            let arguments = base_signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(*derived_name_symbol, derived_value.clone(), arguments, return_type, sig.clone())
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
                        .map(|(i, (name_symbol, signature, bytecode, value, sig))| {
                            functions_mapper.insert(*name_symbol, i);

                            (*name_symbol, signature.clone(), MethodLocation::Blank, value.clone(), sig.clone())
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, signature, _, value, sig)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, arguments, return_type, sig)
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

                    for (_,_,_,value, _) in base_functions {
                        if value.read().expect("lock is poisoned").is_blank() {
                            // We bail if any of base has not yet been linked
                            class_parts_to_try_again.push((class_symbol, parents, members, static_methods, vtables, static_members));
                            continue 'outer;
                        }
                    }

                    let mut functions_mapper = HashMap::new();
                    let functions = base_functions.into_iter()
                        .zip(derived_functions.into_iter())
                        .enumerate()
                        .map(|(i, (base, derived))| {
                            let (_base_name_symbol,  _base_signature, _, base_value, sig) = base;
                            let (derived_name_symbol, derived_signature, _, value, _) = derived;
                            functions_mapper.insert(*derived_name_symbol, i);
                            
                            let value = match *value.read().expect("lock is poisoned") {
                                FunctionValue::Blank => {
                                    base_value.clone()
                                }
                                _ => value.clone()
                            };

                            (*derived_name_symbol, derived_signature.clone(), MethodLocation::Blank, value, sig.clone())
                        })
                        .collect::<Vec<_>>();
                    *vtables_map.get_mut(class_name).unwrap().get_mut(class_name).unwrap() = functions.clone();

                    let functions = functions.into_iter()
                        .map(|(name_symbol, signature, _, value, sig)| {
                            let return_type = convert_type(&signature[0]);
                            let arguments = signature[1..]
                                .iter()
                                .map(convert_type)
                                .collect::<Vec<_>>();

                            Function::new(name_symbol, value, arguments, return_type, sig)
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
                    class_parts_to_try_again.push((class_symbol, parents, members, static_methods, vtables, static_members));
                    continue 'outer;
                }
                _ => {},
            }
            let mut static_method_mapper = HashMap::new();
            let functions = static_methods.into_iter()
                .enumerate()
                .map(|(i, (name, signature, value, sig))| {
                    static_method_mapper.insert(name, i);
                    let name_symbol = name;
                    let arguments = signature[1..].into_iter()
                        .map(convert_type)
                        .collect();
                    let return_type = convert_type(&signature[0]);


                    Function::new(name_symbol, value, arguments, return_type, sig)
                })
                .collect::<Vec<_>>();

            let vtable = VTable::new(functions, static_method_mapper);
            let vtable_index = vtables_table.add_vtable(vtable);

            // Create new class
            let class = Class::new(class_symbol, parents, class_vtable_mapper, members, vtable_index, static_members, |_| {});

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


#[cfg(target_family = "windows")]
fn add_library_mod(name: &str) -> String {
    format!("{name}.dll")
}
#[cfg(target_family = "unix")]
fn add_library_mod(name: &str) -> String {
    format!("{name}.so")
}