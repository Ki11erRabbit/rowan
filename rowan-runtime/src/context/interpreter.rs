use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;
use std::sync::TryLockError;
use rowan_shared::bytecode::linked::Bytecode;
use rowan_shared::TypeTag;
use crate::runtime;
use crate::context::{call_function_pointer, MethodName, Value, WrappedReference};
use crate::runtime::{FunctionDetails, Reference, Runtime, DO_GARBAGE_COLLECTION};
use crate::runtime::object::Object;

#[derive(Debug, Copy, Clone)]
pub enum CallContinueState {
    /// This means to continue what was being done
    Success,
    /// This means that we successfully called a JITted, Native, or Builtin function and it succeeded
    Return,
    /// This means that we just set up a bytecode method to be called
    ExecuteFunction,
    /// An exception was thrown and therefore unwind.
    Error,
}


pub struct StackFrame {
    operand_stack: Vec<Value>,
    ip: usize,
    current_block: usize,
    block_positions: HashMap<usize, usize>,
    variables: [Value; 256],
    call_args: [Value; 256],
    method_name: MethodName,
    is_for_bytecode: bool,
}

impl StackFrame {
    pub fn new(args: &[Value], bytecode: &[Bytecode], is_for_bytecode: bool, method_name: MethodName) -> Self {
        let mut block_positions = HashMap::new();
        for (i, bytecode) in bytecode.iter().enumerate() {
            match bytecode {
                Bytecode::StartBlock(name) => {
                    block_positions.insert(*name as usize, i);
                }
                _ => {}
            }
        }
        let mut variables = [Value::blank(); 256];
        for (arg, variable) in args.iter().zip(variables.iter_mut()) {
            if arg.is_blank() {
                break;
            }
            *variable = *arg;
        }
        Self {
            operand_stack: Vec::new(),
            ip: 0,
            current_block: 0,
            block_positions,
            variables,
            call_args: [Value::blank(); 256],
            is_for_bytecode,
            method_name,
        }
    }

    pub fn push(&mut self, stack_value: Value) {
        assert_ne!(stack_value.is_blank(), true, "Added a blank to the stack");
        self.operand_stack.push(stack_value);
    }
    pub fn pop(&mut self) -> Value {
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

    pub fn store_argument(&mut self, index: u8) {
        let value = self.operand_stack.pop().unwrap();
        self.call_args[index as usize] = value;
    }

    pub fn get_args(&self) -> &[Value] {
        &self.call_args
    }
    pub fn get_args_mut(&mut self) -> &mut [Value] {
        &mut self.call_args
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
                Value { tag: 4, value } => unsafe {
                    let value = value.r;
                    references.insert(WrappedReference(value));
                }
                Value { tag: 7, .. } => {
                    break;
                }
                _ => {}
            }
        }
        for call_arg in self.call_args.iter() {
            match call_arg {
                Value { tag: 4, value } => unsafe {
                    let value = value.r;
                    references.insert(WrappedReference(value));
                }
                Value { tag: 7, .. } => {
                    break;
                }
                _ => {}
            }
        }
        for operand in self.operand_stack.iter() {
            match operand {
                Value { tag: 4, value } => unsafe {
                    let value = value.r;
                    references.insert(WrappedReference(value));
                }
                Value { tag: 7, .. } => {
                    break;
                }
                _ => {}
            }
        }
    }
}


pub struct BytecodeContext {
    active_bytecodes: Vec<&'static [Bytecode]>,
    active_frames: Vec<StackFrame>,
    current_exception: Reference,
    sender: Sender<HashSet<WrappedReference>>,
}


impl BytecodeContext {
    pub fn new(sender: Sender<HashSet<WrappedReference>>) -> Self {
        BytecodeContext {
            active_bytecodes: Vec::new(),
            active_frames: Vec::new(),
            current_exception: std::ptr::null_mut(),
            sender,
        }
    }

    pub fn is_current_exception_set(&self) -> bool {
        !self.current_exception.is_null()
    }

    pub fn push(&mut self, bytecode: &'static [Bytecode], is_for_bytecode: bool, method_name: MethodName) {
        self.active_bytecodes.push(bytecode);
        let frame = self.current_frame();
        let args = frame.get_args();
        self.active_frames.push(StackFrame::new(args, bytecode, is_for_bytecode, method_name));
        let frame_len = self.active_frames.len();
        for arg in self.active_frames[frame_len - 2].get_args_mut() {
            if arg.is_blank() {
                break
            }
            *arg = Value::blank();
        }
    }

    pub fn pop(&mut self) {
        self.active_bytecodes.pop();
        self.active_frames.pop();
    }

    pub fn current_frame(&self) -> &StackFrame {
        let len = self.active_frames.len();
        &self.active_frames[len - 1]
    }

    pub fn current_frame_mut(&mut self) -> &mut StackFrame {
        let len = self.active_frames.len();
        &mut self.active_frames[len - 1]
    }

    /// This function will unwind the stack if needed when an exception is thrown.
    pub fn handle_exception(&mut self) -> CallContinueState {
        CallContinueState::Success
    }


    pub fn invoke_virtual(
        &mut self,
        specified: runtime::Symbol,
        origin: Option<runtime::Symbol>,
        method_name: runtime::Symbol,
        return_slot: Option<&mut Value>,
    ) -> CallContinueState {
        let object = self.current_frame().call_args[0];
        let object = match object {
            Value { tag: 4, value: object} => unsafe { object.r },
            _ => todo!("report error that first call arg must be an object.")
        };
        let object = unsafe {
            object.as_ref().expect("report null pointer")
        };

        let details = Runtime::get_virtual_method_details(
            object.class,
            specified,
            origin,
            method_name,
        );

        let method_name = MethodName::VirtualMethod {
            object_class_symbol: object.class,
            class_symbol: specified,
            source_class: origin,
            method_name,
        };

        self.call_function(details, method_name, return_slot)
    }


    pub fn invoke_static(
        &mut self,
        class_name: runtime::Symbol,
        method_name: runtime::Symbol,
        return_slot: Option<&mut Value>,
    ) -> CallContinueState {
        let details = Runtime::get_static_method_details(
            class_name,
            method_name,
        );

        let method_name = MethodName::StaticMethod {
            class_symbol: class_name,
            method_name
        };

        self.call_function(details, method_name, return_slot)
    }

    pub fn call_function(
        &mut self,
        details: FunctionDetails,
        method_name: MethodName,
        return_slot: Option<&mut Value>
    ) -> CallContinueState {
        for pair in self.current_frame().call_args.iter().zip(details.arguments.iter()) {
            match pair {
                (Value { tag: 0, .. }, runtime::class::TypeTag::U8) |
                (Value { tag: 0, .. }, runtime::class::TypeTag::I8) => {}
                (Value { tag: 1, .. }, runtime::class::TypeTag::U16) |
                (Value { tag: 1, .. }, runtime::class::TypeTag::I16) => {}
                (Value { tag: 2, .. }, runtime::class::TypeTag::U32) |
                (Value { tag: 2, .. }, runtime::class::TypeTag::I32) => {}
                (Value { tag: 3, .. }, runtime::class::TypeTag::U64) |
                (Value { tag: 3, .. }, runtime::class::TypeTag::I64) => {}
                (Value { tag: 5, .. }, runtime::class::TypeTag::F32) => {}
                (Value { tag: 6, .. }, runtime::class::TypeTag::F64) => {}
                (Value { tag: 4, .. }, runtime::class::TypeTag::Object) => {}
                _ => {
                    todo!("report type error in typing")
                }
            }
        }

        self.push(details.bytecode, details.fn_ptr.is_none(), method_name);

        match details.fn_ptr {
            Some(fn_ptr) => {
                let var_len = self.current_frame().vars_len();
                let mut variables = self.current_frame()
                    .variables[..var_len]
                    .to_vec();
                let need_padding = super::need_padding(&variables);
                super::sort_call_args(&mut variables);
                let mut return_value = call_function_pointer(
                    self,
                    variables.as_ptr(),
                    var_len,
                    fn_ptr.as_ptr(),
                    details.return_type.tag(),
                    need_padding as u8
                );
                self.pop();
                if let Some(return_slot) = return_slot {
                    *return_slot = return_value;
                }
                if !return_value.is_blank() {
                    self.current_frame_mut().push(return_value);
                }
                //self.handle_exception()
                CallContinueState::Return
            }
            _ => CallContinueState::ExecuteFunction
        }

    }

    /// TODO: add way to pass in cmdline args
    pub fn call_main(&mut self, class: runtime::Symbol, method: runtime::Symbol) {
        let details = Runtime::get_static_method_details(
            class,
            method,
        );
        let method_name = MethodName::StaticMethod {
            class_symbol: class,
            method_name: method,
        };
        self.active_bytecodes.push(details.bytecode);
        // TODO: add passing of cmdline args
        self.active_frames.push(StackFrame::new(&[], details.bytecode, details.fn_ptr.is_none(), method_name));
        self.current_frame_mut().variables[0] = Value::from(std::ptr::null_mut());
        self.main_loop();
    }

    /// returns true if call finished without any errors
    /// returns false if an exception was thrown
    pub fn invoke_virtual_extern(
        &mut self,
        specified: runtime::Symbol,
        origin: Option<runtime::Symbol>,
        method_name: runtime::Symbol,
        return_slot: Option<&mut Value>,
    ) -> bool {
        let result = self.invoke_virtual(specified, origin, method_name, return_slot);
        match result {
            CallContinueState::Success => false,
            CallContinueState::Return => true,
            CallContinueState::ExecuteFunction => {
                self.main_loop();
                true
            }
            CallContinueState::Error => false,
        }
    }


    pub fn invoke_static_extern(
        &mut self,
        class_name: runtime::Symbol,
        method_name: runtime::Symbol,
        return_slot: Option<&mut Value>,
    ) -> bool {
        let result = self.invoke_static(class_name, method_name, return_slot);
        match result {
            CallContinueState::Success => false,
            CallContinueState::Return => true,
            CallContinueState::ExecuteFunction => {
                self.main_loop();
                true
            }
            CallContinueState::Error => false,
        }
    }

    pub fn main_loop(&mut self) {
        if !self.current_frame().is_for_bytecode {
            self.check_and_do_garbage_collection();
            return;
        }
        loop {
            self.check_and_do_garbage_collection();
            let bytecode = &self.active_bytecodes[self.active_bytecodes.len() - 1][self.current_frame().ip];
            self.current_frame_mut().ip += 1;

            if !self.interpret(bytecode) {
                break;
            }
            if self.active_frames.is_empty() {
                break;
            }
        }
    }

    fn check_for_garbage_collection(&mut self) -> bool {
        match DO_GARBAGE_COLLECTION.try_read() {
            Ok(_) => true,
            Err(TryLockError::WouldBlock) => {
                false
            }
            Err(TryLockError::Poisoned(_)) => {
                panic!("Lock poisoned");
            }
        }
    }
    pub fn check_and_do_garbage_collection(&mut self) {
        if self.check_for_garbage_collection() {
            return
        }

        self.collect_garbage()

    }

    pub fn collect_garbage(&mut self) {
        let mut references = HashSet::new();

        self.collect_interpreter_references(&mut references);
        self.collect_jit_references(&mut references);


        self.sender.send(references).unwrap();
        loop {
            if self.check_for_garbage_collection() {
                return
            }
            std::thread::yield_now();
        }
    }

    fn collect_interpreter_references(&mut self, references: &mut HashSet<WrappedReference>) {
        for frame in self.active_frames.iter() {
            frame.collect(references);
        }
    }

    fn collect_jit_references(&mut self, references: &mut HashSet<WrappedReference>) {
        let mut info = Vec::new();

        let mut frame_iter = self.active_frames.iter().rev();

        rowan_unwind::backtrace(|frame| {
            if frame.is_jitted() {
                let sp = frame.sp();
                let ip = frame.ip();
                //println!("RSP: {:x}, RIP: {:x}", sp, ip);
                let Some(frame) = frame_iter.next() else {
                    return false;
                };

                info.push((frame.method_name, sp, ip));
            }
            true
        });

        self.dereference_stack_pointer(&info, references);
    }

    fn dereference_stack_pointer(
        &mut self,
        info: &[(MethodName, usize, usize)],
        references: &mut HashSet<WrappedReference>
    ) {
        Runtime::dereference_stack_pointer(info, references);
    }

    /// The bool return dictates whether execution should continue or not.
    pub fn interpret(&mut self, bytecode: &Bytecode) -> bool {
        match bytecode {
            Bytecode::Nop => {}
            Bytecode::Breakpoint => {}
            Bytecode::LoadU8(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadU16(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadU32(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadU64(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadI8(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadI16(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadI32(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadI64(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadF32(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadF64(value) => {
                self.current_frame_mut().push(Value::from(*value));
            }
            Bytecode::LoadSymbol(_sym) => {
                todo!("LoadSymbol")
            }
            Bytecode::Pop => {
                self.current_frame_mut().pop();
            }
            Bytecode::Dup => {
                self.current_frame_mut().dup();
            }
            Bytecode::Swap => {
                self.current_frame_mut().swap();
            }
            Bytecode::StoreLocal(index) => {
                self.current_frame_mut().store_local(*index);
            }
            Bytecode::LoadLocal(index) => {
                self.current_frame_mut().load_local(*index);
            }
            Bytecode::StoreArgument(index) => {
                self.current_frame_mut().store_argument(*index);
            }
            Bytecode::AddInt => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();

                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_add(rhs)));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_add(rhs)));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_add(rhs)));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_add(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SubInt => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();

                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_sub(rhs)));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_sub(rhs)));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_sub(rhs)));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_sub(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::MulInt => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();

                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_mul(rhs)));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_mul(rhs)));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_mul(rhs)));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_mul(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::DivSigned => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::DivUnsigned => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_div(rhs)));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_div(rhs)));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_div(rhs)));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_div(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::ModSigned => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(Value::from(result));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::ModUnsigned => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_rem(rhs)));
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_rem(rhs)));
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_rem(rhs)));
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.wrapping_rem(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::AddFloat => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        self.current_frame_mut().push(Value::from(lhs + rhs))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        self.current_frame_mut().push(Value::from(lhs + rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SubFloat => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        self.current_frame_mut().push(Value::from(lhs - rhs))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        self.current_frame_mut().push(Value::from(lhs - rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::MulFloat => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        self.current_frame_mut().push(Value::from(lhs * rhs))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        self.current_frame_mut().push(Value::from(lhs * rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::DivFloat => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        self.current_frame_mut().push(Value::from(lhs / rhs))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        self.current_frame_mut().push(Value::from(lhs / rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::ModFloat => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        self.current_frame_mut().push(Value::from(lhs % rhs))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        self.current_frame_mut().push(Value::from(lhs % rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SatAddIntUnsigned => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.saturating_add(rhs)))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.saturating_add(rhs)))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.saturating_add(rhs)))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.saturating_add(rhs)))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SatSubIntUnsigned => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs.saturating_sub(rhs)))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs.saturating_sub(rhs)))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs.saturating_sub(rhs)))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs.saturating_sub(rhs)))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::And => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs & rhs))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs & rhs))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs & rhs))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs & rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Or => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs | rhs))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs | rhs))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs | rhs))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs | rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Xor => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs ^ rhs))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs ^ rhs))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs ^ rhs))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs ^ rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Not => {
                let value = self.current_frame_mut().pop();
                match value {
                    Value { tag: 0, value } => {
                        let value = unsafe { value.c };
                        self.current_frame_mut().push(Value::from(!value))
                    }
                    Value { tag: 1, value } => {
                        let value = unsafe { value.s };
                        self.current_frame_mut().push(Value::from(!value))
                    }
                    Value { tag: 2, value } => {
                        let value = unsafe { value.i };
                        self.current_frame_mut().push(Value::from(!value))
                    }
                    Value { tag: 3, value } => {
                        let value = unsafe { value.l };
                        self.current_frame_mut().push(Value::from(!value))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Shl => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs << rhs))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs << rhs))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs << rhs))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs << rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::AShr => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LShr => {
                let rhs = self.current_frame_mut().pop();
                let lhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        self.current_frame_mut().push(Value::from(lhs >> rhs))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        self.current_frame_mut().push(Value::from(lhs >> rhs))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        self.current_frame_mut().push(Value::from(lhs >> rhs))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        self.current_frame_mut().push(Value::from(lhs >> rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Neg => {
                let value = self.current_frame_mut().pop();
                match value {
                    Value { tag: 0, value } => {
                        let value = unsafe { value.c };
                        let value = i8::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    Value { tag: 1, value } => {
                        let value = unsafe { value.s };
                        let value = i16::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    Value { tag: 2, value } => {
                        let value = unsafe { value.i };
                        let value = i32::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    Value { tag: 3, value } => {
                        let value = unsafe { value.l };
                        let value = i64::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    Value { tag: 5, value } => {
                        let value = unsafe { value.f };
                        let value = -value;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    Value { tag: 6, value } => {
                        let value = unsafe { value.d };
                        let value = -value;
                        self.current_frame_mut().push(Value::from(value))
                    }
                    Value { tag: 4, .. } => {
                        todo!("Throw error saying that you can't negate references")
                    }
                    _ => unreachable!("You can't negate a blank")
                }
            }
            Bytecode::EqualSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::NotEqualSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::EqualUnsigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let value = lhs == rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::NotEqualUnsigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let value = lhs != rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterUnsigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let value = lhs > rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessUnsigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let value = lhs < rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterOrEqualSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessOrEqualSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterOrEqualUnsigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessOrEqualUnsigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 0, value: lhs }, Value { tag: 0, value: rhs }) => {
                        let lhs = unsafe { lhs.c };
                        let rhs = unsafe { rhs.c };
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 1, value: lhs }, Value { tag: 1, value: rhs }) => {
                        let lhs = unsafe { lhs.s };
                        let rhs = unsafe { rhs.s };
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 2, value: lhs }, Value { tag: 2, value: rhs }) => {
                        let lhs = unsafe { lhs.i };
                        let rhs = unsafe { rhs.i };
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 3, value: lhs }, Value { tag: 3, value: rhs }) => {
                        let lhs = unsafe { lhs.l };
                        let rhs = unsafe { rhs.l };
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::EqualFloat => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        let value = lhs.eq(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        let value = lhs.eq(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::NotEqualFloat => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        let value = lhs.ne(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        let value = lhs.ne(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterFloat => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        let value = lhs.gt(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        let value = lhs.gt(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessFloat => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        let value = lhs.lt(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        let value = lhs.lt(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterOrEqualFloat => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        let value = lhs.ge(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        let value = lhs.ge(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessOrEqualFloat => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (Value { tag: 5, value: lhs }, Value { tag: 5, value: rhs }) => {
                        let lhs = unsafe { lhs.f };
                        let rhs = unsafe { rhs.f };
                        let value = lhs.le(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    (Value { tag: 6, value: lhs }, Value { tag: 6, value: rhs }) => {
                        let lhs = unsafe { lhs.d };
                        let rhs = unsafe { rhs.d };
                        let value = lhs.le(&rhs);
                        self.current_frame_mut().push(Value::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Convert(tag) => {
                let value = self.current_frame_mut().pop();
                match tag {
                    TypeTag::U8 => {
                        let value = value.as_u8();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U16 => {
                        let value = value.as_u16();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U32 => {
                        let value = value.as_u32();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U64 => {
                        let value = value.as_u64();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I8 => {
                        let value = value.as_i8();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I16 => {
                        let value = value.as_i16();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I32 => {
                        let value = value.as_i32();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I64 => {
                        let value = value.as_i64();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F32 => {
                        let value = value.as_f32();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F64 => {
                        let value = value.as_f64();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::Object => {
                        todo!("report object conversion errors")
                    }
                    _ => unreachable!("Invalid Type Tag")
                }
            }
            Bytecode::BinaryConvert(tag) => {
                let value = self.current_frame_mut().pop();
                match tag {
                    TypeTag::U8 => {
                        let value = value.into_u8();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U16 => {
                        let value = value.into_u16();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U32 => {
                        let value = value.into_u32();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U64 => {
                        let value = value.into_u64();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I8 => {
                        let value = value.into_i8();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I16 => {
                        let value = value.into_i16();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I32 => {
                        let value = value.into_i32();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::I64 => {
                        let value = value.into_i64();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F32 => {
                        let value = value.into_f32();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F64 => {
                        let value = value.into_f64();
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::Object => {
                        todo!("report object conversion errors")
                    }
                    _ => unreachable!("Invalid Type Tag")
                }
            }
            Bytecode::CreateArray(tag) => {
                let size = self.current_frame_mut().pop();
                let size = match size {
                    Value { tag: 3, value: size } => unsafe { size.l }
                    _ => todo!("report needing u64 for array alloc"),
                };
                let (class_name, init): (&str, fn(&mut BytecodeContext, Reference, u64)) = match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        ("core::Array8", runtime::core::array8_init_internal)
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        ("core::Array16", runtime::core::array16_init_internal)
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        ("core::Array32", runtime::core::array32_init_internal)
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        ("core::Array64", runtime::core::array64_init_internal)
                    }
                    TypeTag::F32 => {
                        ("core::Arrayf32", runtime::core::arrayf32_init_internal)
                    }
                    TypeTag::F64 => {
                        ("core::Arrayf64", runtime::core::arrayf64_init_internal)
                    }
                    TypeTag::Object => {
                        ("core::Arrayobject", runtime::core::arrayobject_init_internal)
                    }
                    _ => unreachable!("Invalid Type Tag")
                };
                // TODO: add call to stack so that it can record the backtrace correctly
                let object = Runtime::new_object(class_name);
                init(self, object, size);
                self.current_frame_mut().push(Value::from(object));
            }
            Bytecode::ArrayGet(tag) => {
                let index = self.current_frame_mut().pop();
                let index = match index {
                    Value { tag: 3, value: index } => unsafe { index.l }
                    _ => todo!("report needing u64"),
                };
                let array = self.current_frame_mut().pop();
                let array = match array {
                    Value { tag: 4, value: object } => unsafe { object.r }
                    _ => todo!("report needing reference for index"),
                };
                match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = runtime::core::array8_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = runtime::core::array16_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = runtime::core::array32_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = runtime::core::array64_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F32 => {
                        let value = runtime::core::arrayf32_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F64 => {
                        let value = runtime::core::arrayf64_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::Object => {
                        let value = runtime::core::arrayobject_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        let value = value as usize as Reference;
                        self.current_frame_mut().push(Value::from(value));
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::ArraySet(tag) => {
                let value = self.current_frame_mut().pop();
                let index = self.current_frame_mut().pop();
                let index = match index {
                    Value { tag: 3, value: index } => unsafe { index.l }
                    _ => todo!("report needing u64"),
                };
                let array = self.current_frame_mut().pop();
                let array = match array {
                    Value { tag: 4, value: object } => unsafe { object.r }
                    _ => todo!("report needing reference for index"),
                };
                match (tag, value) {
                    (TypeTag::U8, Value { tag: 0, value }) | (TypeTag::I8, Value { tag: 0, value }) => {
                        let value = unsafe { value.c };
                        runtime::core::array8_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U16, Value { tag: 1, value }) | (TypeTag::I16, Value { tag: 1, value }) => {
                        let value = unsafe { value.s };
                        runtime::core::array16_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U32, Value { tag: 2, value }) | (TypeTag::I32, Value { tag: 2, value }) => {
                        let value = unsafe { value.i };
                        runtime::core::array32_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U64, Value { tag: 3, value }) | (TypeTag::I64, Value { tag: 32, value }) => {
                        let value = unsafe { value.l };
                        runtime::core::array64_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F32, Value { tag: 5, value }) => {
                        let value = unsafe { value.f };
                        runtime::core::arrayf32_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F64, Value { tag: 5, value }) => {
                        let value = unsafe { value.d };
                        runtime::core::arrayf64_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::Object, Value { tag: 4, value }) => {
                        let value = unsafe { value.r };
                        let value = value as usize as u64;
                        runtime::core::arrayobject_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::NewObject(sym) => {
                let object = Runtime::new_object(*sym as usize);
                self.current_frame_mut().push(Value::from(object));
            }
            Bytecode::GetField(access, parent_name, index, tag) => {
                let object = self.current_frame_mut().pop();
                let object = match object {
                    Value { tag: 4, value: object } => unsafe { object.r }
                    _ => todo!("report not accessing object correctly"),
                };

                match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = Object::get_8(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = Object::get_16(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = Object::get_32(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = Object::get_64(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F32 => {
                        let value = Object::get_f32(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::F64 => {
                        let value = Object::get_f64(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.current_frame_mut().push(Value::from(value));
                    }
                    TypeTag::Object => {
                        let value = Object::get_object(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        let value = value as usize as Reference;
                        self.current_frame_mut().push(Value::from(value));
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::SetField(access, parent_name, index, tag) => {
                let value = self.current_frame_mut().pop();
                let object = self.current_frame_mut().pop();
                let object = match object {
                    Value { tag: 4, value: object } => unsafe { object.r }
                    _ => todo!("report needing u64"),
                };
                match (tag, value) {
                    (TypeTag::U8, Value { tag: 0, value }) | (TypeTag::I8, Value { tag: 0, value }) => {
                        let value = unsafe { value.c };
                        Object::set_8(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U16, Value { tag: 1, value }) | (TypeTag::I16, Value { tag: 1, value }) => {
                        let value = unsafe { value.s };
                        Object::set_16(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U32, Value { tag: 2, value }) | (TypeTag::I32, Value { tag: 2, value }) => {
                        let value = unsafe { value.i };
                        Object::set_32(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U64, Value { tag: 3, value }) | (TypeTag::I64, Value { tag: 3, value }) => {
                        let value = unsafe { value.l };
                        Object::set_64(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F32, Value { tag: 5, value }) => {
                        let value = unsafe { value.f };
                        Object::set_f32(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F32, Value { tag: 6, value }) => {
                        let value = unsafe { value.d };
                        Object::set_f64(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::Object, Value { tag: 4, value }) => {
                        let value = unsafe { value.r };
                        let value = value as usize as u64;
                        Object::set_object(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::IsA(sym) => {
                let object = self.current_frame_mut().pop();
                let object = match object {
                    Value { tag: 4, value: object } => unsafe { object.r }
                    _ => todo!("report needing object")
                };
                let object = unsafe {
                    object.as_ref().expect("check for null pointer")
                };
                let result = object.class as u64 == *sym;
                self.current_frame_mut().push(Value::from(result as u8));
            }
            Bytecode::InvokeVirt(specified, origin, method_name) => {
                match self.invoke_virtual(
                    *specified as runtime::Symbol,
                    origin.map(|s| s as runtime::Symbol),
                    *method_name as runtime::Symbol,
                    None
                ) {
                    CallContinueState::Error => return false,
                    CallContinueState::Return => return false,
                    _ => return true,
                }
            }
            Bytecode::InvokeVirtTail(..) => {
                todo!("Tail Recursion Virtual")
            }
            Bytecode::InvokeStatic(class_name, method_name) => {
                match self.invoke_static(
                    *class_name as runtime::Symbol,
                    *method_name as runtime::Symbol,
                    None,
                ) {
                    CallContinueState::Error => return false,
                    CallContinueState::Return => return false,
                    _ => return true,
                }
            }
            Bytecode::InvokeStaticTail(..) => {
                todo!("Tail Recursion Static")
            }
            Bytecode::GetStaticMethod(..) => {
                todo!("conversion of a static method into an object")
            }
            Bytecode::GetStaticMember(..) => {
                todo!("access of static members")
            }
            Bytecode::SetStaticMember(..) => {
                todo!("access of static members")
            }
            Bytecode::GetStrRef(sym) => {
                self.current_frame_mut().push(Value::from(*sym));
            }
            Bytecode::Return => {
                let value = self.current_frame_mut().pop();
                self.pop();
                self.current_frame_mut().push(value);
            }
            Bytecode::ReturnVoid => {
                if self.active_frames.len() == 1 {
                    return false;
                }
                self.pop();
            }
            Bytecode::RegisterException(..) => {
                todo!("registering of exceptions");
            }
            Bytecode::UnregisterException(..) => {
                todo!("unregistering of exceptions");
            }
            Bytecode::Throw => {
                let exception = self.current_frame_mut().pop();
                let exception = match exception {
                    Value { tag: 4, value: exception } => unsafe { exception.r }
                    _ => todo!("report exception needing to be an object"),
                };
                self.current_exception = exception;
            }
            Bytecode::StartBlock(_) => {}
            Bytecode::Goto(offset) => {
                self.current_frame_mut().goto(*offset as isize);
            }
            Bytecode::If(then_offset, else_offset) => {
                let value = self.current_frame_mut().pop();
                let boolean = match value {
                    Value { tag: 0, value: value } => unsafe { value.c }
                    _ => todo!("report invalid type for boolean"),
                };

                if boolean != 0 {
                    self.current_frame_mut().goto(*then_offset as isize);
                } else {
                    self.current_frame_mut().goto(*else_offset as isize);
                }
            }
            Bytecode::Switch(..) => {
                todo!("switching conditional");
            }
        }
        true
    }

    pub extern "C" fn store_argument_int8(&mut self, index: u8, value: u8) {
        self.current_frame_mut().push(value.into());
        self.current_frame_mut().store_argument(index);
    }

    pub extern "C" fn store_argument_int16(&mut self, index: u8, value: u16) {
        self.current_frame_mut().push(value.into());
        self.current_frame_mut().store_argument(index);
    }

    pub extern "C" fn store_argument_int32(&mut self, index: u8, value: u32) {
        self.current_frame_mut().push(value.into());
        self.current_frame_mut().store_argument(index);
    }

    pub extern "C" fn store_argument_int64(&mut self, index: u8, value: u64) {
        self.current_frame_mut().push(value.into());
        self.current_frame_mut().store_argument(index);
    }

    pub extern "C" fn store_argument_float32(&mut self, index: u8, value: f32) {
        self.current_frame_mut().push(value.into());
        self.current_frame_mut().store_argument(index);
    }

    pub extern "C" fn store_argument_float64(&mut self, index: u8, value: f64) {
        self.current_frame_mut().push(value.into());
        self.current_frame_mut().store_argument(index);
    }
}
