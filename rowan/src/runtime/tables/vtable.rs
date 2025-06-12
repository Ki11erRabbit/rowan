use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use cranelift::prelude::Signature;
use cranelift_module::FuncId;
use rowan_shared::bytecode::linked::Bytecode;

use crate::runtime::{class::TypeTag, Index, Symbol, VTableIndex};




pub struct VTable {
    pub symbol_mapper: HashMap<Symbol, Index>,
    pub table: Vec<Function>,
}

impl VTable {
    pub fn new(table: Vec<Function>, mapper: HashMap<Symbol, Index>) -> Self {
        VTable {
            symbol_mapper: mapper,
            table
        }
    }

    pub fn get_function(&self, symbol: Symbol) -> &Function {
        //println!("[VTable] Looking for symbol: {:?}", symbol);
        //println!("[VTable] Mapper: {:?}", self.symbol_mapper);
        let index = self.symbol_mapper.get(&symbol).unwrap();
        &self.table[*index]
    }
    pub fn get_function_mut(&mut self, symbol: Symbol) -> &mut Function {
        let index = self.symbol_mapper.get(&symbol).unwrap();
        &mut self.table[*index]
    }
}


#[derive(Clone)]
pub struct Function {
    pub name: Symbol,
    pub value: Arc<RwLock<FunctionValue>>,
    pub arguments: Vec<TypeTag>,
    pub return_type: TypeTag,
}

impl Function {
    pub fn new(
        name: Symbol,
        value: Arc<RwLock<FunctionValue>>,
        arguments: Vec<TypeTag>,
        return_type: TypeTag
    ) -> Self {
        Function {
            name,
            value,
            arguments,
            return_type
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Function")
        .field("name", &self.name)
        .field("value", &self.value)
        .field("arguments", &self.arguments)
        .field("return_type", &self.return_type)
        .finish()
    }
}


#[derive(Clone)]
pub enum FunctionValue {
    Builtin(*const (), Signature),
    Bytecode(Vec<Bytecode>, FuncId, Signature),
    Compiled(*const (), Signature),
    Native(*const (), Signature),
    Blank,
}

impl FunctionValue {
    pub fn is_blank(&self) -> bool {
        match self {
            FunctionValue::Blank => true,
            _ => false,
        }
    }
}

impl Debug for FunctionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            FunctionValue::Builtin(ptr, signature) => {
                f.debug_struct("Builtin")
                .field("ptr", ptr)
                    .finish()
            }
            FunctionValue::Blank => f.debug_struct("Blank").finish(),
            FunctionValue::Compiled(ptr, signature) => {
                f.debug_struct("Compiled")
                .field("ptr", ptr)
                .finish()
            }
            FunctionValue::Native(ptr, signature) => {
                f.debug_struct("Native")
                .field("ptr", ptr)
                .finish()
            }
            FunctionValue::Bytecode(bytecode,_,  _) => {
                f.debug_struct("Bytecode")
                .field("bytecode", bytecode)
                .finish()
            }
        }
    }
}


unsafe impl Send for FunctionValue {}
unsafe impl Sync for FunctionValue {}

pub struct VTables {
    table: Vec<VTable>,
}

impl VTables {
    pub fn new() -> Self {
        VTables {
            table: Vec::new()
        }
    }

    pub fn add_vtable(&mut self, table: VTable) -> VTableIndex {
        let out = self.table.len();
        self.table.push(table);
        out
    }
}

impl std::ops::Index<VTableIndex> for VTables {
    type Output = VTable;
    fn index(&self, index: VTableIndex) -> &Self::Output {
        &self.table[index]
    }
}
