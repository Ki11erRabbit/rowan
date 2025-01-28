use std::collections::HashMap;

use rowan_shared::bytecode::linked::Bytecode;

use crate::runtime::{class::TypeTag, Index, Symbol, VTableIndex};




pub struct VTable {
    pub symbol_mapper: HashMap<Symbol, Index>,
    pub table: Vec<Function>,
}


pub struct Function {
    pub name: Symbol,
    pub value: FunctionValue,
    pub responds_to: Option<Symbol>,
    pub arguments: Vec<TypeTag>,
    pub return_type: TypeTag,
}

pub enum FunctionValue {
    Builtin(*const ()),
    Bytecode(Vec<Bytecode>),
    Compiled(*const ()),
}



pub struct VTables {
    table: Vec<VTable>,
}


impl std::ops::Index<VTableIndex> for VTables {
    type Output = VTable;
    fn index(&self, index: VTableIndex) -> &Self::Output {
        &self.table[index]
    }
}
