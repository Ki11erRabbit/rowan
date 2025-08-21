use std::collections::HashSet;
use fxhash::FxHashMap;
use crate::context::{MethodName, StackValue, WrappedReference};


pub enum StackFrame {
    Full {
        ip: usize,
        current_block: usize,
        block_positions: &'static FxHashMap<usize, usize>,
        variables: [StackValue; 256],
        method_name: MethodName,
    },
    Light {
        method_name: MethodName,
    }
}

impl StackFrame {
    pub fn new(
        args: &[StackValue],
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
        Self::Full {
            ip: 0,
            current_block: 0,
            block_positions,
            variables,
            method_name,
        }
    }

    pub fn is_for_bytecode(&self) -> bool {
        match self {
            StackFrame::Full { .. } => true,
            StackFrame::Light { .. } => false,
        }
    }

    pub fn new_light(method_name: MethodName) -> Self {
        StackFrame::Light { method_name }
    }

    pub fn method_name(&self) -> &MethodName {
        match self {
            StackFrame::Full { method_name, .. } => method_name,
            StackFrame::Light { method_name } => method_name,
        }
    }

    pub fn variables(&self) -> &[StackValue] {
        match self {
            StackFrame::Full { variables, .. } => variables,
            StackFrame::Light { .. } => unreachable!("Can't get variables from a light frame"),
        }
    }

    pub fn variables_mut(&mut self) -> &mut [StackValue] {
        match self {
            StackFrame::Full { variables, .. } => variables,
            StackFrame::Light { .. } => unreachable!("Can't get variables from a light frame"),
        }
    }

    pub fn ip(&self) -> &usize {
        match self {
            StackFrame::Full { ip, .. } => ip,
            StackFrame::Light { .. } => unreachable!("cannot get ip of a light frame"),
        }
    }

    pub fn ip_mut(&mut self) -> &mut usize {
        match self {
            StackFrame::Full { ip, .. } => ip,
            StackFrame::Light { .. } => unreachable!("cannot get ip of a light frame"),
        }
    }

    pub fn goto(&mut self, block_offset: isize) {
        match self {
            StackFrame::Full {
                block_positions,
                ip,
                current_block,
                ..
            } => {
                let next_block = *current_block as isize + block_offset;
                let next_block = next_block as usize;
                let pc = block_positions[&next_block];
                *ip = pc;
                *current_block = next_block;
            },
            StackFrame::Light { .. } => unreachable!("can't change instruction pointer with light stack frame")
        }
    }

    pub fn vars_len(&self) -> usize {
        match self {
            StackFrame::Full {
                variables,
                ..
            } => {
                for (i, var) in variables.iter().enumerate() {
                    if var.is_blank() {
                        return i;
                    }
                }
                variables.len()
            }
            StackFrame::Light { .. } => 0,
        }
    }

    pub fn collect(&self, references: &mut HashSet<WrappedReference>) {
        match self {
            StackFrame::Full {
                variables,
                ..
            } => {
                for variable in variables.iter() {
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
            StackFrame::Light { .. } => {}
        }
    }
}
