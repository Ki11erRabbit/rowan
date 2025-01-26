use std::collections::HashMap;

use rowan_shared::bytecode::linked::Bytecode;

use crate::runtime::{class::TypeTag, Index, Symbol};




pub struct VTable {
    symbol_mapper: HashMap<Symbol, Index>,
    table: Vec<Function>,
}


pub struct Function {
    name: Symbol,
    value: FunctionValue,
    responds_to: Option<Symbol>,
    arguments: Vec<TypeTag>,
    return_type: TypeTag,
}

pub enum FunctionValue {
    Builtin(*const ()),
    Bytecode(Vec<Bytecode>),
    Compiled(*const ()),

}
