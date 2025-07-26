use std::collections::HashMap;
use std::fmt::Debug;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use cranelift::prelude::Signature;
use cranelift_module::FuncId;
use rowan_shared::bytecode::linked::Bytecode;

use crate::runtime::{class::TypeTag, Index, Symbol, VTableIndex};

pub struct FunctionDetails {
    pub bytecode: &'static [Bytecode],
    pub arguments: &'static [TypeTag],
    pub return_type: TypeTag,
    pub fn_ptr: Option<NonNull<()>>
}

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

    pub fn get_function(&self, symbol: Symbol) -> Option<&Function> {
        //println!("[VTable] Looking for symbol: {:?}", symbol);
        //println!("[VTable] Mapper: {:?}", self.symbol_mapper);
        if self.table.len() < 10 {
            for func in self.table.iter() {
                if func.name == symbol {
                    return Some(func);
                }
            }
            None
        } else {
            let index = self.symbol_mapper.get(&symbol).unwrap();
            Some(&self.table[*index])
        }
    }
    pub fn get_function_mut(&mut self, symbol: Symbol) -> &mut Function {
        let index = self.symbol_mapper.get(&symbol).unwrap();
        &mut self.table[*index]
    }
}


#[derive(Clone)]
pub struct Function {
    pub name: Symbol,
    pub bytecode: Box<[Bytecode]>,
    pub value: FunctionValue,
    pub arguments: Box<[TypeTag]>,
    pub return_type: TypeTag,
    pub signature: Signature,
}

impl Function {
    pub fn new(
        name: Symbol,
        bytecode: Box<[Bytecode]>,
        value: FunctionValue,
        arguments: Box<[TypeTag]>,
        return_type: TypeTag,
        signature: Signature,
    ) -> Self {
        Function {
            name,
            bytecode,
            value,
            arguments,
            return_type,
            signature,
        }
    }

    pub fn create_details(&self) -> FunctionDetails {
        let bytecode_ptr = self.bytecode.as_ptr();
        let bytecode_len = self.bytecode.len();
        let bytecode_ref = unsafe {
            std::slice::from_raw_parts(bytecode_ptr, bytecode_len)
        };
        let arguments_ptr = self.arguments.as_ptr();
        let arguments_len = self.arguments.len();
        let arguments_ref = unsafe {
            std::slice::from_raw_parts(arguments_ptr, arguments_len)
        };

        let fn_ptr = match self.value {
            FunctionValue::Builtin(ptr) => {
                NonNull::new(ptr as *mut ())
            }
            FunctionValue::Compiled(ptr, _) => {
                NonNull::new(ptr as *mut ())
            }
            FunctionValue::Native(ptr) => {
                NonNull::new(ptr as *mut ())
            }
            _ => None,
        };

        FunctionDetails {
            bytecode: bytecode_ref,
            arguments: arguments_ref,
            return_type: self.return_type,
            fn_ptr,
        }
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Function")
        .field("name", &self.name)
        .field("arguments", &self.arguments)
        .field("return_type", &self.return_type)
        .finish()
    }
}


#[derive(Clone)]
pub enum FunctionValue {
    Builtin(*const ()),
    Bytecode(FuncId),
    /// The hashmap's key is a stack pointer address where we have spilled objects onto the stack
    Compiled(*const (), HashMap<usize, Vec<u32>>),
    Native(*const ()),
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
            FunctionValue::Builtin(ptr) => {
                f.debug_struct("Builtin")
                .field("ptr", ptr)
                    .finish()
            }
            FunctionValue::Blank => f.debug_struct("Blank").finish(),
            FunctionValue::Compiled(ptr, _) => {
                f.debug_struct("Compiled")
                .field("ptr", ptr)
                .finish()
            }
            FunctionValue::Native(ptr) => {
                f.debug_struct("Native")
                .field("ptr", ptr)
                .finish()
            }
            FunctionValue::Bytecode(bytecode,_) => {
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
