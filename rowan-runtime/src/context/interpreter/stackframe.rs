use std::collections::HashSet;
use fxhash::FxHashMap;
use crate::context::{MethodName, StackValue, WrappedReference};


pub struct StackFrame {
    pub(crate) ip: usize,
    current_block: usize,
    block_positions: &'static FxHashMap<usize, usize>,
    pub(crate) variables: [StackValue; 256],
    pub(crate) method_name: MethodName,
    pub(crate) is_for_bytecode: bool,
}

impl StackFrame {
    pub fn new(
        args: &[StackValue],
        is_for_bytecode: bool,
        method_name: MethodName,
        block_positions: &'static FxHashMap<usize, usize>,
    ) -> Self {
        let mut variables = [StackValue::Blank; 256];
        for (arg, variable) in args.iter().zip(variables.iter_mut()) {
            if arg.is_blank() {
                break;
            }
            *variable = *arg;
        }
        Self {
            ip: 0,
            current_block: 0,
            block_positions,
            variables,
            is_for_bytecode,
            method_name,
        }
    }

    pub fn is_for_bytecode(&self) -> bool {
        self.is_for_bytecode
    }

    pub fn goto(&mut self, block_offset: isize) {
        let next_block = self.current_block as isize + block_offset;
        let next_block = next_block as usize;
        let pc = self.block_positions[&next_block];
        self.ip = pc;
        self.current_block = next_block;
    }

    pub fn vars_len(&self) -> usize {
        for (i, var) in self.variables.iter().enumerate() {
            if var.is_blank() {
                return i;
            }
        }
        self.variables.len()
    }

    pub fn collect(&self, references: &mut HashSet<WrappedReference>) {
        for variable in self.variables.iter() {
            match variable {
                StackValue::Reference(value) =>  {
                    references.insert(WrappedReference(*value));
                }
                StackValue::Blank => {
                    break;
                }
                _ => {}
            }
        }
    }
}
