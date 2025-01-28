use std::{collections::HashMap, env::var, sync::{LazyLock, RwLock}};

use class::{Class, MemberInfo, SignalInfo};
use rowan_shared::classfile::{BytecodeEntry, BytecodeIndex, ClassFile, Member, Signal, SignatureIndex, VTableEntry};
use tables::{class_table::ClassTable, object_table::ObjectTable, string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue, VTable, VTables}};


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

        let Ok(mut vtable_tables) = VTABLES.write() else {
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
        let mut members = Vec::new();
        for member in class_file.members.iter() {
            let Member { name, type_tag } = member;
            let name_str = class_file.index_string_table(*name);
            let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                *symbol
            } else {
                let str_table_index = string_table.add_string(name_str);
                let sym = symbol_table.add_string(str_table_index);
                string_map.insert(String::from(name_str), sym);
                sym
            };

            let type_tag = self.convert_type(type_tag);

            members.push(MemberInfo::new(name_symbol, type_tag));
        }

        let mut signals = Vec::new();
        for signal in class_file.signals.iter() {
            let Signal { name, is_static, signature } = signal;
            let name_str = class_file.index_string_table(*name);
            let name_symbol = if let Some(symbol) = string_map.get(name_str) {
                *symbol
            } else {
                let str_table_index = string_table.add_string(name_str);
                let sym = symbol_table.add_string(str_table_index);
                string_map.insert(String::from(name_str), sym);
                sym
            };
            let signature = &class_file.signature_table[*signature as usize];
            let signature = signature.types[1..].iter().map(|t| self.convert_type(t)).collect();

            signals.push(SignalInfo::new(name_symbol, *is_static, signature));
        }

        let next_class_index = class_table.get_next_index();

        let class_symbol = symbol_table.add_class(next_class_index);
        class_map.insert(String::from(class_name), class_symbol);

        let mut class_vtables = Vec::new();
        let mut class_vtable_map = HashMap::new();
        for (i, (functions, mapper)) in vtables.into_iter().enumerate() {
            let mut table = Vec::new();
            let mut vtable_class_symbol = 0;
            for (vtable_class_name, name_symbol, responds_to, signature, bytecode) in functions {
                let function = if vtable_class_name == class_name {
                    let bytecode = self.link_bytecode(class_file.index_bytecode_table(bytecode), string_map, class_map);
                    let value = FunctionValue::Bytecode(bytecode);

                    let signature = class_file.signature_table[signature as usize];

                    let arguments = signature.types[1..].iter().map(|t| self.convert_type(t)).collect();
                    let return_type = self.convert_type(&signature.types[0]);

                    vtable_class_symbol = class_symbol;
                    
                    Function::new(name_symbol, value, responds_to, arguments, return_type)
                } else if bytecode == 0 {
                    let class_name_symbol = class_map.get(vtable_class_name).expect("We haven't linked a class file yet");
                    let SymbolEntry::ClassRef(class_index) = symbol_table[*class_name_symbol] else {
                        panic!("class wasn't a class");
                    };

                    let class = &class_table[class_index];

                    let vtable_index = class.get_vtable(*class_name_symbol);

                    let vtable = &vtable_tables[vtable_index];

                    vtable_class_symbol = *class_name_symbol;

                    vtable.get_function(name_symbol).clone()
                } else {
                    let class_name_symbol = class_map.get(vtable_class_name).expect("We haven't linked a class file yet");
                    let bytecode = self.link_bytecode(class_file.index_bytecode_table(bytecode), string_map, class_map);
                    let value = FunctionValue::Bytecode(bytecode);

                    let signature = class_file.signature_table[signature as usize];

                    let arguments = signature.types[1..].iter().map(|t| self.convert_type(t)).collect();
                    let return_type = self.convert_type(&signature.types[0]);

                    vtable_class_symbol = *class_name_symbol;

                    Function::new(name_symbol, value, responds_to, arguments, return_type)
                };

                table.push(function);
            }

            let vtable = VTable::new(table, mapper);
            let vtable_index = vtable_tables.add_vtable(vtable);
            
            class_vtable_map.insert(vtable_class_symbol, vtable_index);
        }

        let class = Class::new(class_name_symbol, parents, class_vtable_map, members, signals);

        let class_index = class_table.insert_class(class);
        assert!(class_index == next_class_index, "inserted class index doesn't match index");

    }

    fn convert_type(&self, tag: &rowan_shared::TypeTag) -> super::runtime::class::TypeTag {
        match tag {
            rowan_shared::TypeTag::U8 => super::runtime::class::TypeTag::U8,
            rowan_shared::TypeTag::U16 => super::runtime::class::TypeTag::U16,
            rowan_shared::TypeTag::U32 => super::runtime::class::TypeTag::U32,
            rowan_shared::TypeTag::U64 => super::runtime::class::TypeTag::U64,
            rowan_shared::TypeTag::I8 => super::runtime::class::TypeTag::I8,
            rowan_shared::TypeTag::I16 => super::runtime::class::TypeTag::I16,
            rowan_shared::TypeTag::I32 => super::runtime::class::TypeTag::I32,
            rowan_shared::TypeTag::I64 => super::runtime::class::TypeTag::I64,
            rowan_shared::TypeTag::F32 => super::runtime::class::TypeTag::F32,
            rowan_shared::TypeTag::F64 => super::runtime::class::TypeTag::F64,
            rowan_shared::TypeTag::Object => super::runtime::class::TypeTag::Object,
            rowan_shared::TypeTag::Str => super::runtime::class::TypeTag::Str,
            rowan_shared::TypeTag::Void => super::runtime::class::TypeTag::Void,

        }
    }

    fn link_bytecode(
        &self,
        BytecodeEntry { code }: &BytecodeEntry,
        string_map: &mut HashMap<String, Symbol>,
        class_map: &mut HashMap<String, Symbol>
    ) -> Vec<rowan_shared::bytecode::linked::Bytecode> {
        let mut output = Vec::new();
        let compiled_code = rowan_shared::bytecode::compiled::Bytecode::try_from(code).unwrap();

        output
    }
}
