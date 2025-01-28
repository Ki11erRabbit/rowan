use std::collections::HashMap;

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
        let index = self.symbol_mapper.get(&symbol).unwrap();
        &self.table[*index]
    }
    pub fn get_function_mut(&mut self, symbol: Symbol) -> &mut Function {
        let index = self.symbol_mapper.get(&symbol).unwrap();
        &mut self.table[*index]
    }
}


pub struct Function {
    pub name: Symbol,
    pub value: FunctionValue,
    pub responds_to: Option<Symbol>,
    pub arguments: Vec<TypeTag>,
    pub return_type: TypeTag,
}

impl Function {
    pub fn new(
        name: Symbol,
        value: FunctionValue,
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

#[derive(Clone)]
pub enum FunctionValue {
    Builtin(*const ()),
    Bytecode(Vec<Bytecode>),
    Compiled(*const ()),
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
}

impl std::ops::Index<VTableIndex> for VTables {
    type Output = VTable;
    fn index(&self, index: VTableIndex) -> &Self::Output {
        &self.table[index]
    }
}
