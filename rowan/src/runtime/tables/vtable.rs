use std::collections::HashMap;
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
        println!("[VTable] Looking for symbol: {:?}", symbol);
        println!("[VTable] Mapper: {:?}", self.symbol_mapper);
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
    pub responds_to: Option<Symbol>,
    pub arguments: Vec<TypeTag>,
    pub return_type: TypeTag,
}

impl Function {
    pub fn new(
        name: Symbol,
        value: Arc<RwLock<FunctionValue>>,
        responds_to: Option<Symbol>,
        arguments: Vec<TypeTag>,
        return_type: TypeTag
    ) -> Self {
        Function {
            name,
            value,
            responds_to,
            arguments,
            return_type
        }
    }
}



#[derive(Clone, Debug)]
pub enum FunctionValue {
    Builtin(*const (), Signature),
    Bytecode(Vec<Bytecode>, FuncId, Signature),
    Compiled(*const (), Signature),
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
