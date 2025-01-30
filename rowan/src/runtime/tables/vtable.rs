use std::{collections::HashMap, hash::{Hash, Hasher}};

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

impl PartialEq for VTable {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
    }
}

impl Hash for VTable {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.table.hash(hasher)
    }

}

#[derive(Clone)]
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

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.responds_to == other.responds_to && self.arguments == other.arguments && self.return_type == other.return_type && self.value == other.value
    }
}

impl Hash for Function {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.name.hash(hasher);
        self.responds_to.hash(hasher);
        self.arguments.hash(hasher);
        self.return_type.hash(hasher);
    }

}

#[derive(Clone)]
pub enum FunctionValue {
    Builtin(*const ()),
    Bytecode(Vec<Bytecode>),
    Compiled(*const ()),
    Blank,
}

impl PartialEq for FunctionValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FunctionValue::Bytecode(b1), FunctionValue::Bytecode(b2)) => {
                b1 == b2
            }
            (FunctionValue::Builtin(b1), FunctionValue::Builtin(b2)) => {
                b1 == b2
            }
            (FunctionValue::Blank, FunctionValue::Builtin(_)) => {
                true
            }
            (FunctionValue::Builtin(_), FunctionValue::Bank) => {
                true
            }
            (FunctionValue::Blank, FunctionValue::Bytecode(_)) => {
                true
            }
            (FunctionValue::Bytecode(_), FunctionValue::Bank) => {
                true
            }
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
