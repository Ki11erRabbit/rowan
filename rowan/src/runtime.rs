use std::{collections::HashMap, sync::{LazyLock, RwLock}};

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

        let mut class_vtable_map = HashMap::new();
        for (i, (functions, mapper)) in vtables.into_iter().enumerate() {
            let mut table = Vec::new();
            let mut vtable_class_symbol = 0;
            for (vtable_class_name, name_symbol, responds_to, signature, bytecode) in functions {
                let function = if vtable_class_name == class_name {
                    let bytecode = self.link_bytecode(
                        &class_file,
                        class_file.index_bytecode_table(bytecode),
                        string_map,
                        class_map,
                        &mut string_table,
                        &mut symbol_table,
                    );
                    let value = FunctionValue::Bytecode(bytecode);

                    let signature = &class_file.signature_table[signature as usize];

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
                    let class_name_symbol = *class_map.get(vtable_class_name).expect("We haven't linked a class file yet");
                    let bytecode = self.link_bytecode(
                        &class_file,
                        class_file.index_bytecode_table(bytecode),
                        string_map,
                        class_map,
                        &mut string_table,
                        &mut symbol_table,
                    );
                    let value = FunctionValue::Bytecode(bytecode);

                    let signature = &class_file.signature_table[signature as usize];

                    let arguments = signature.types[1..].iter().map(|t| self.convert_type(t)).collect();
                    let return_type = self.convert_type(&signature.types[0]);

                    vtable_class_symbol = class_name_symbol;

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
        class_file: &ClassFile,
        BytecodeEntry { code }: &BytecodeEntry,
        string_map: &mut HashMap<String, Symbol>,
        class_map: &mut HashMap<String, Symbol>,
        string_table: &mut StringTable,
        symbol_table: &mut SymbolTable,
    ) -> Vec<rowan_shared::bytecode::linked::Bytecode> {
        let mut output = Vec::new();
        let compiled_code: Vec<rowan_shared::bytecode::compiled::Bytecode> =
            rowan_shared::bytecode::compiled::Bytecode::try_from(&mut code.iter()).unwrap();

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
                compiled::Bytecode::Add => {
                    output.push(linked::Bytecode::Add);
                }
                compiled::Bytecode::Sub => {
                    output.push(linked::Bytecode::Sub);
                }
                compiled::Bytecode::Mul => {
                    output.push(linked::Bytecode::Mul);
                }
                compiled::Bytecode::Div => {
                    output.push(linked::Bytecode::Div);
                }
                compiled::Bytecode::Mod => {
                    output.push(linked::Bytecode::Mod);
                }
                compiled::Bytecode::SatAdd => {
                    output.push(linked::Bytecode::SatAdd);
                }
                compiled::Bytecode::SatSub => {
                    output.push(linked::Bytecode::SatSub);
                }
                compiled::Bytecode::SatMul => {
                    output.push(linked::Bytecode::SatMul);
                }
                compiled::Bytecode::SatDiv => {
                    output.push(linked::Bytecode::SatDiv);
                }
                compiled::Bytecode::SatMod => {
                    output.push(linked::Bytecode::SatMod);
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
                compiled::Bytecode::AShl => {
                    output.push(linked::Bytecode::AShl);
                }
                compiled::Bytecode::LShl => {
                    output.push(linked::Bytecode::LShl);
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
                compiled::Bytecode::Equal => {
                    output.push(linked::Bytecode::Equal);
                }
                compiled::Bytecode::NotEqual => {
                    output.push(linked::Bytecode::NotEqual);
                }
                compiled::Bytecode::Greater => {
                    output.push(linked::Bytecode::Greater);
                }
                compiled::Bytecode::GreaterOrEqual => {
                    output.push(linked::Bytecode::GreaterOrEqual);
                }
                compiled::Bytecode::Less => {
                    output.push(linked::Bytecode::Less);
                }
                compiled::Bytecode::LessOrEqual => {
                    output.push(linked::Bytecode::LessOrEqual);
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
                compiled::Bytecode::InvokeVirt(class_index, _, method_index) => {
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

                    output.push(linked::Bytecode::InvokeVirt(class_symbol as u64, 0, method_symbol as u64));
                }
                compiled::Bytecode::InvokeVirtTail(class_index, _, method_index) => {
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

                    output.push(linked::Bytecode::InvokeVirtTail(class_symbol as u64, 0, method_symbol as u64));
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

        output
    }
}
