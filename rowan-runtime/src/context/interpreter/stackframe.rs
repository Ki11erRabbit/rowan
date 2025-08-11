use std::collections::HashSet;
use std::ops::DerefMut;
use std::sync::{LazyLock, Mutex};
use fxhash::FxHashMap;
use pool_box::{Complete, Pool, PoolBoxAllocator};
use rowan_shared::bytecode::linked::Bytecode;
use crate::context::{MethodName, StackValue, WrappedReference};


static STACKFRAME_POOL: LazyLock<Mutex<Pool<StackFrame>>> = LazyLock::new(|| {
    let pool = Pool::new(256);
    Mutex::new(pool)
});


pub struct StackFrame {
    operand_stack: Vec<StackValue>,
    pub(crate) ip: usize,
    current_block: usize,
    block_positions: FxHashMap<usize, usize>,
    pub(crate) variables: [StackValue; 256],
    pub(crate) method_name: MethodName,
    pub(crate) is_for_bytecode: bool,
}

impl StackFrame {
    pub fn new(args: &[StackValue], bytecode: &[Bytecode], is_for_bytecode: bool, method_name: MethodName) -> Self {
        let mut block_positions = FxHashMap::default();
        for (i, bytecode) in bytecode.iter().enumerate() {
            match bytecode {
                Bytecode::StartBlock(name) => {
                    block_positions.insert(*name as usize, i);
                }
                _ => {}
            }
        }
        let mut variables = [StackValue::Blank; 256];
        for (arg, variable) in args.iter().zip(variables.iter_mut()) {
            if arg.is_blank() {
                break;
            }
            *variable = *arg;
        }
        Self {
            operand_stack: Vec::with_capacity(10),
            ip: 0,
            current_block: 0,
            block_positions,
            variables,
            is_for_bytecode,
            method_name,
        }
    }

    pub fn push(&mut self, stack_value: StackValue) {
        assert_ne!(stack_value.is_blank(), true, "Added a blank to the stack");
        self.operand_stack.push(stack_value);
    }
    pub fn pop(&mut self) -> StackValue {
        self.operand_stack.pop().unwrap()
    }

    pub fn dup(&mut self) {
        let value = self.operand_stack.last().unwrap();
        self.operand_stack.push(*value);
    }

    pub fn swap(&mut self) {
        let value1 = self.operand_stack.pop().unwrap();
        let value2 = self.operand_stack.pop().unwrap();
        self.operand_stack.push(value2);
        self.operand_stack.push(value1);
    }

    pub fn store_local(&mut self, index: u8) {
        let value = self.operand_stack.pop().unwrap();
        self.variables[index as usize] = value;
    }

    pub fn load_local(&mut self, index: u8) {
        let value = self.variables[index as usize];
        self.operand_stack.push(value);
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
        for operand in self.operand_stack.iter() {
            match operand {
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

unsafe impl Complete for StackFrame {}

#[derive(Default)]
pub struct StackFramePoolBoxAllocator {}

impl PoolBoxAllocator<StackFrame> for StackFramePoolBoxAllocator {
    fn fetch_pool(&self) -> impl DerefMut<Target = Pool<StackFrame>> {
        STACKFRAME_POOL.lock().unwrap()
    }
}