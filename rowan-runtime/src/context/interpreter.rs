use std::collections::HashMap;
use paste::paste;
use rowan_shared::bytecode::linked::Bytecode;
use rowan_shared::TypeTag;
use crate::runtime;
use crate::context::call_function_pointer;
use crate::runtime::{Reference, Runtime};
use crate::runtime::object::Object;

#[derive(Clone, Copy, Debug)]
pub enum StackValue {
    Int8(u8),
    Int16(u16),
    Int32(u32),
    Int64(u64),
    Float32(f32),
    Float64(f64),
    Reference(Reference),
    Blank,
}

macro_rules! as_type {
    ($typ:ty) => {
        paste! {
            pub fn [<as_ $typ>](self) -> $typ {
                match self {
                    StackValue::Int8(v) => v as $typ,
                    StackValue::Int16(v) => v as $typ,
                    StackValue::Int32(v) => v as $typ,
                    StackValue::Int64(v) => v as $typ,
                    StackValue::Float32(v) => v as $typ,
                    StackValue::Float64(v) => v as $typ,
                    _ => todo!("Throw error for mismatched type")
                }
            }
        }
    };
}

macro_rules! into_type {
    ($typ:ty) => {
        paste! {
            pub fn [<into_ $typ>](self) -> $typ {
                let mut buffer = [0u8; std::mem::size_of::<$typ>()];
                match self {
                    StackValue::Int8(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf == *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Int16(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf == *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Int32(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf == *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Int64(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf == *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Float32(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf == *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Float64(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf == *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    _ => todo!("Throw error for mismatched type")
                }
            }
        }
    };
}
impl StackValue {
    as_type!(u8);
    as_type!(u16);
    as_type!(u32);
    as_type!(u64);
    as_type!(i8);
    as_type!(i16);
    as_type!(i32);
    as_type!(i64);
    as_type!(f32);
    as_type!(f64);
    into_type!(u8);
    into_type!(u16);
    into_type!(u32);
    into_type!(u64);
    into_type!(i8);
    into_type!(i16);
    into_type!(i32);
    into_type!(i64);
    into_type!(f32);
    into_type!(f64);

    pub fn is_blank(self) -> bool {
        match self {
            StackValue::Blank => true,
            _ => false,
        }
    }
}

impl From<u8> for StackValue {
    fn from(v: u8) -> Self {
        StackValue::Int8(v)
    }
}

impl From<u16> for StackValue {
    fn from(v: u16) -> Self {
        StackValue::Int16(v)
    }
}

impl From<u32> for StackValue {
    fn from(v: u32) -> Self {
        StackValue::Int32(v)
    }
}

impl From<u64> for StackValue {
    fn from(v: u64) -> Self {
        StackValue::Int64(v)
    }
}
impl From<i8> for StackValue {
    fn from(v: i8) -> Self {
        StackValue::Int8(u8::from_ne_bytes(v.to_ne_bytes()))
    }
}

impl From<i16> for StackValue {
    fn from(v: i16) -> Self {
        StackValue::Int16(u16::from_ne_bytes(v.to_ne_bytes()))
    }
}

impl From<i32> for StackValue {
    fn from(v: i32) -> Self {
        StackValue::Int32(u32::from_ne_bytes(v.to_ne_bytes()))
    }
}

impl From<i64> for StackValue {
    fn from(v: i64) -> Self {
        StackValue::Int64(u64::from_ne_bytes(v.to_ne_bytes()))
    }
}

impl From<f32> for StackValue {
    fn from(v: f32) -> Self {
        StackValue::Float32(v)
    }
}

impl From<f64> for StackValue {
    fn from(v: f64) -> Self {
        StackValue::Float64(v)
    }
}

impl From<Reference> for StackValue {
    fn from(v: Reference) -> Self {
        StackValue::Reference(v)
    }
}


pub struct StackFrame {
    operand_stack: Vec<StackValue>,
    ip: usize,
    current_block: usize,
    block_positions: HashMap<usize, usize>,
    variables: [StackValue; 256],
    call_args: [StackValue; 256],
    is_for_bytecode: bool,
}

impl StackFrame {
    pub fn new(args: &[StackValue], bytecode: &[Bytecode], is_for_bytecode: bool) -> Self {
        let mut block_positions = HashMap::new();
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
            operand_stack: Vec::new(),
            ip: 0,
            current_block: 0,
            block_positions,
            variables,
            call_args: [StackValue::Blank; 256],
            is_for_bytecode,
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

    pub fn store_argument(&mut self, index: u8) {
        let value = self.operand_stack.pop().unwrap();
        self.call_args[index as usize] = value;
    }

    pub fn get_args(&self) -> &[StackValue] {
        &self.call_args
    }
    pub fn get_args_mut(&mut self) -> &mut [StackValue] {
        &mut self.call_args
    }

    pub fn is_for_bytecode(&self) -> bool {
        self.is_for_bytecode
    }

    pub fn goto(&mut self, block_offset: usize) {
        let next_block = self.current_block + block_offset;
        let pc = self.block_positions[&next_block];
        self.ip = pc;
    }

    pub fn vars_len(&self) -> usize {
        for (i, var) in self.variables.iter().enumerate() {
            if var.is_blank() {
                return i;
            }
        }
        self.variables.len()
    }
}


pub struct BytecodeContext {
    active_bytecodes: Vec<&'static [Bytecode]>,
    active_frames: Vec<StackFrame>,
    current_exception: Reference,
}


impl BytecodeContext {
    pub fn new() -> Self {
        BytecodeContext {
            active_bytecodes: Vec::new(),
            active_frames: Vec::new(),
            current_exception: std::ptr::null_mut(),
        }
    }

    pub fn push(&mut self, bytecode: &'static [Bytecode], is_for_bytecode: bool) {
        self.active_bytecodes.push(bytecode);
        let frame = self.current_frame();
        let args = frame.get_args();
        self.active_frames.push(StackFrame::new(args, bytecode, is_for_bytecode));
        let frame_len = self.active_frames.len();
        for arg in self.active_frames[frame_len - 2].get_args_mut() {
            if arg.is_blank() {
                break
            }
            *arg = StackValue::Blank;
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
    /// The bool return dictates whether execution should continue or not.
    /// `true` means that we can continue from a bytecode context.
    /// `false` means that we can't continue from a bytecode context meaning that we have to unwind
    /// to a function call to either continue unwinding or catch the exception.
    pub fn handle_exception(&mut self) -> bool {
        true
    }

    /// The bool return dictates whether execution should continue or not.
    /// `true` means that we can continue from a bytecode context.
    /// `false` means that we can't continue from a bytecode context meaning that we have to unwind
    pub fn invoke_virtual(
        &mut self,
        specified: runtime::Symbol,
        origin: Option<runtime::Symbol>,
        method_name: runtime::Symbol
    ) -> bool {
        let object = self.current_frame().call_args[0];
        let object = match object {
            StackValue::Reference(object) => object,
            x => todo!("report error that first call arg must be an object. instead found: {x:?}")
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

        for pair in self.current_frame().call_args.iter().zip(details.arguments.iter()) {
            match pair {
                (StackValue::Int8(_), runtime::class::TypeTag::U8) |
                (StackValue::Int8(_), runtime::class::TypeTag::I8) => {}
                (StackValue::Int16(_), runtime::class::TypeTag::U16) |
                (StackValue::Int16(_), runtime::class::TypeTag::I16) => {}
                (StackValue::Int32(_), runtime::class::TypeTag::U32) |
                (StackValue::Int32(_), runtime::class::TypeTag::I32) => {}
                (StackValue::Int64(_), runtime::class::TypeTag::U64) |
                (StackValue::Int64(_), runtime::class::TypeTag::I64) => {}
                (StackValue::Float32(_), runtime::class::TypeTag::F32) => {}
                (StackValue::Float64(_), runtime::class::TypeTag::F64) => {}
                (StackValue::Reference(_), runtime::class::TypeTag::Object) => {}
                _ => {
                    todo!("report type error in typing")
                }
            }
        }

        self.push(details.bytecode, details.fn_ptr.is_none());

        match details.fn_ptr {
            Some(fn_ptr) => {
                let var_pointer = self.current_frame().variables.as_ptr();
                let var_len = self.current_frame().vars_len();
                let variables = unsafe {
                    std::slice::from_raw_parts(var_pointer, var_len)
                };
                let mut return_value = call_function_pointer(
                    self,
                    variables.as_ptr(),
                    variables.len(),
                    fn_ptr.as_ptr(),
                    details.return_type.tag(),
                );
                self.pop();
                if !return_value.is_blank() {
                    self.current_frame_mut().push(return_value);
                }
            }
            _ => {}
        }

        self.handle_exception()
    }

    /// The bool return dictates whether execution should continue or not.
    /// `true` means that we can continue from a bytecode context.
    /// `false` means that we can't continue from a bytecode context meaning that we have to unwind
    pub fn invoke_static(
        &mut self,
        class_name: runtime::Symbol,
        method_name: runtime::Symbol
    ) -> bool {
        let details = Runtime::get_static_method_details(
            class_name,
            method_name,
        );

        for pair in self.current_frame().call_args.iter().zip(details.arguments.iter()) {
            match pair {
                (StackValue::Int8(_), runtime::class::TypeTag::U8) |
                (StackValue::Int8(_), runtime::class::TypeTag::I8) => {}
                (StackValue::Int16(_), runtime::class::TypeTag::U16) |
                (StackValue::Int16(_), runtime::class::TypeTag::I16) => {}
                (StackValue::Int32(_), runtime::class::TypeTag::U32) |
                (StackValue::Int32(_), runtime::class::TypeTag::I32) => {}
                (StackValue::Int64(_), runtime::class::TypeTag::U64) |
                (StackValue::Int64(_), runtime::class::TypeTag::I64) => {}
                (StackValue::Float32(_), runtime::class::TypeTag::F32) => {}
                (StackValue::Float64(_), runtime::class::TypeTag::F64) => {}
                (StackValue::Reference(_), runtime::class::TypeTag::Object) => {}
                _ => {
                    todo!("report type error in typing")
                }
            }
        }

        self.push(details.bytecode, details.fn_ptr.is_none());

        match details.fn_ptr {
            Some(fn_ptr) => {
                let var_pointer = self.current_frame().variables.as_ptr();
                let var_len = self.current_frame().variables.len();
                let variables = unsafe {
                    std::slice::from_raw_parts(var_pointer, var_len)
                };
                let mut return_value = call_function_pointer(
                    self,
                    variables.as_ptr(),
                    variables.len(),
                    fn_ptr.as_ptr(),
                    details.return_type.tag(),
                );
                self.pop();
                if !return_value.is_blank() {
                    self.current_frame_mut().push(return_value);
                }
            }
            _ => {}
        }

        self.handle_exception()
    }

    /// TODO: add way to pass in cmdline args
    pub fn call_main(&mut self, class: runtime::Symbol, method: runtime::Symbol) {
        let details = Runtime::get_static_method_details(
            class,
            method,
        );
        self.active_bytecodes.push(details.bytecode);
        // TODO: add passing of cmdline args
        self.active_frames.push(StackFrame::new(&[], details.bytecode, details.fn_ptr.is_none()));
        self.current_frame_mut().variables[0] = StackValue::Reference(std::ptr::null_mut());
        self.main_loop();
    }

    pub fn main_loop(&mut self) {
        loop {
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

    /// The bool return dictates whether execution should continue or not.
    pub fn interpret(&mut self, bytecode: &Bytecode) -> bool {
        match bytecode {
            Bytecode::Nop => {}
            Bytecode::Breakpoint => {}
            Bytecode::LoadU8(value) => {
                self.current_frame_mut().push(StackValue::Int8(*value));
            }
            Bytecode::LoadU16(value) => {
                self.current_frame_mut().push(StackValue::Int16(*value));
            }
            Bytecode::LoadU32(value) => {
                self.current_frame_mut().push(StackValue::Int32(*value));
            }
            Bytecode::LoadU64(value) => {
                self.current_frame_mut().push(StackValue::Int64(*value));
            }
            Bytecode::LoadI8(value) => {
                self.current_frame_mut().push(StackValue::Int8(u8::from_ne_bytes(value.to_le_bytes())));
            }
            Bytecode::LoadI16(value) => {
                self.current_frame_mut().push(StackValue::Int16(u16::from_ne_bytes(value.to_le_bytes())));
            }
            Bytecode::LoadI32(value) => {
                self.current_frame_mut().push(StackValue::Int32(u32::from_ne_bytes(value.to_le_bytes())));
            }
            Bytecode::LoadI64(value) => {
                self.current_frame_mut().push(StackValue::Int64(u64::from_ne_bytes(value.to_le_bytes())));
            }
            Bytecode::LoadF32(value) => {
                self.current_frame_mut().push(StackValue::Float32(*value));
            }
            Bytecode::LoadF64(value) => {
                self.current_frame_mut().push(StackValue::Float64(*value));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.wrapping_add(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.wrapping_add(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.wrapping_add(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.wrapping_add(rhs)));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.wrapping_sub(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.wrapping_sub(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.wrapping_sub(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.wrapping_sub(rhs)));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.wrapping_mul(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.wrapping_mul(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.wrapping_mul(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.wrapping_mul(rhs)));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(StackValue::Int8(u8::from_ne_bytes(result.to_ne_bytes())));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(StackValue::Int16(u16::from_ne_bytes(result.to_ne_bytes())));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(StackValue::Int32(u32::from_ne_bytes(result.to_ne_bytes())));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.current_frame_mut().push(StackValue::Int64(u64::from_ne_bytes(result.to_ne_bytes())));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.wrapping_div(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.wrapping_div(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.wrapping_div(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.wrapping_div(rhs)));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(StackValue::Int8(u8::from_ne_bytes(result.to_ne_bytes())));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(StackValue::Int16(u16::from_ne_bytes(result.to_ne_bytes())));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(StackValue::Int32(u32::from_ne_bytes(result.to_ne_bytes())));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.current_frame_mut().push(StackValue::Int64(u64::from_ne_bytes(result.to_ne_bytes())));
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.wrapping_rem(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.wrapping_rem(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.wrapping_rem(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.wrapping_rem(rhs)));
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float32(lhs + rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float64(lhs + rhs))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float32(lhs - rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float64(lhs - rhs))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float32(lhs * rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float64(lhs * rhs))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float32(lhs / rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float64(lhs / rhs))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float32(lhs % rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Float64(lhs % rhs))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.saturating_add(rhs)))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.saturating_add(rhs)))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.saturating_add(rhs)))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.saturating_add(rhs)))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs.saturating_sub(rhs)))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs.saturating_sub(rhs)))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs.saturating_sub(rhs)))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs.saturating_sub(rhs)))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs & rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs & rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs & rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs & rhs))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs | rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs | rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs | rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs | rhs))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs ^ rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs ^ rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs ^ rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs ^ rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Not => {
                let value = self.current_frame_mut().pop();
                match value {
                    StackValue::Int8(value) => {
                        self.current_frame_mut().push(StackValue::Int8(!value))
                    }
                    StackValue::Int16(value) => {
                        self.current_frame_mut().push(StackValue::Int16(!value))
                    }
                    StackValue::Int32(value) => {
                        self.current_frame_mut().push(StackValue::Int32(!value))
                    }
                    StackValue::Int64(value) => {
                        self.current_frame_mut().push(StackValue::Int64(!value))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs << rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs << rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs << rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs << rhs))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = u8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(StackValue::Int8(u8::from_ne_bytes(value.to_ne_bytes())))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = u16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(StackValue::Int16(u16::from_ne_bytes(value.to_ne_bytes())))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(StackValue::Int32(u32::from_ne_bytes(value.to_ne_bytes())))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.current_frame_mut().push(StackValue::Int64(u64::from_ne_bytes(value.to_ne_bytes())))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int8(lhs >> rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int16(lhs >> rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int32(lhs >> rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.current_frame_mut().push(StackValue::Int64(lhs >> rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Neg => {
                let value = self.current_frame_mut().pop();
                match value {
                    StackValue::Int8(value) => {
                        let value = i8::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        let value = u8::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int8(value))
                    }
                    StackValue::Int16(value) => {
                        let value = i16::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        let value = u16::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int16(value))
                    }
                    StackValue::Int32(value) => {
                        let value = i32::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        let value = u32::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int32(value))
                    }
                    StackValue::Int64(value) => {
                        let value = i64::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        let value = u64::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int64(value))
                    }
                    StackValue::Float32(value) => {
                        let value = -value;
                        self.current_frame_mut().push(StackValue::Float32(value))
                    }
                    StackValue::Float64(value) => {
                        let value = -value;
                        self.current_frame_mut().push(StackValue::Float64(value))
                    }
                    StackValue::Reference(_) => {
                        todo!("Throw error saying that you can't negate references")
                    }
                    _ => unreachable!("You can't negate a blank")
                }
            }
            Bytecode::EqualSigned => {
                let lhs = self.current_frame_mut().pop();
                let rhs = self.current_frame_mut().pop();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs == rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs != rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs > rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs < rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs >= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs <= rhs;
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.eq(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.eq(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.ne(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.ne(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.gt(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.gt(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.lt(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.lt(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.ge(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.ge(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.le(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.le(&rhs);
                        self.current_frame_mut().push(StackValue::Int8(value as u8))
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
                        self.current_frame_mut().push(StackValue::Int8(value));
                    }
                    TypeTag::U16 => {
                        let value = value.as_u16();
                        self.current_frame_mut().push(StackValue::Int16(value));
                    }
                    TypeTag::U32 => {
                        let value = value.as_u32();
                        self.current_frame_mut().push(StackValue::Int32(value));
                    }
                    TypeTag::U64 => {
                        let value = value.as_u64();
                        self.current_frame_mut().push(StackValue::Int64(value));
                    }
                    TypeTag::I8 => {
                        let value = value.as_i8();
                        let value = u8::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int8(value));
                    }
                    TypeTag::I16 => {
                        let value = value.as_i16();
                        let value = u16::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int16(value));
                    }
                    TypeTag::I32 => {
                        let value = value.as_i32();
                        let value = u32::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int32(value));
                    }
                    TypeTag::I64 => {
                        let value = value.as_i64();
                        let value = u64::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int64(value));
                    }
                    TypeTag::F32 => {
                        let value = value.as_f32();
                        self.current_frame_mut().push(StackValue::Float32(value));
                    }
                    TypeTag::F64 => {
                        let value = value.as_f64();
                        self.current_frame_mut().push(StackValue::Float64(value));
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
                        self.current_frame_mut().push(StackValue::Int8(value));
                    }
                    TypeTag::U16 => {
                        let value = value.into_u16();
                        self.current_frame_mut().push(StackValue::Int16(value));
                    }
                    TypeTag::U32 => {
                        let value = value.into_u32();
                        self.current_frame_mut().push(StackValue::Int32(value));
                    }
                    TypeTag::U64 => {
                        let value = value.into_u64();
                        self.current_frame_mut().push(StackValue::Int64(value));
                    }
                    TypeTag::I8 => {
                        let value = value.into_i8();
                        let value = u8::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int8(value));
                    }
                    TypeTag::I16 => {
                        let value = value.into_i16();
                        let value = u16::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int16(value));
                    }
                    TypeTag::I32 => {
                        let value = value.into_i32();
                        let value = u32::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int32(value));
                    }
                    TypeTag::I64 => {
                        let value = value.into_i64();
                        let value = u64::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int64(value));
                    }
                    TypeTag::F32 => {
                        let value = value.into_f32();
                        self.current_frame_mut().push(StackValue::Float32(value));
                    }
                    TypeTag::F64 => {
                        let value = value.into_f64();
                        self.current_frame_mut().push(StackValue::Float64(value));
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
                    StackValue::Int64(size) => size,
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
                self.current_frame_mut().push(StackValue::Reference(object));
            }
            Bytecode::ArrayGet(tag) => {
                let index = self.current_frame_mut().pop();
                let index = match index {
                    StackValue::Int64(index) => index,
                    _ => todo!("report needing u64"),
                };
                let array = self.current_frame_mut().pop();
                let array = match array {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing reference for index"),
                };
                match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = runtime::core::array8_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int8(value));
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = runtime::core::array16_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int16(value));
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = runtime::core::array32_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int32(value));
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = runtime::core::array64_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int64(value));
                    }
                    TypeTag::F32 => {
                        let value = runtime::core::arrayf32_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Float32(value));
                    }
                    TypeTag::F64 => {
                        let value = runtime::core::arrayf64_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Float64(value));
                    }
                    TypeTag::Object => {
                        let value = runtime::core::arrayobject_get(self, array, index);
                        if !self.handle_exception() {
                            return false;
                        }
                        let value = value as usize as Reference;
                        self.current_frame_mut().push(StackValue::Reference(value));
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::ArraySet(tag) => {
                let value = self.current_frame_mut().pop();
                let index = self.current_frame_mut().pop();
                let index = match index {
                    StackValue::Int64(index) => index,
                    _ => todo!("report needing u64"),
                };
                let array = self.current_frame_mut().pop();
                let array = match array {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing reference for index"),
                };
                match (tag, value) {
                    (TypeTag::U8, StackValue::Int8(value)) | (TypeTag::I8, StackValue::Int8(value)) => {
                        runtime::core::array8_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::U16, StackValue::Int16(value)) | (TypeTag::I16, StackValue::Int16(value)) => {
                        runtime::core::array16_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::U32, StackValue::Int32(value)) | (TypeTag::I32, StackValue::Int32(value)) => {
                        runtime::core::array32_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::U64, StackValue::Int64(value)) | (TypeTag::I64, StackValue::Int64(value)) => {
                        runtime::core::array64_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::F32, StackValue::Float32(value)) => {
                        runtime::core::arrayf32_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::F64, StackValue::Float64(value)) => {
                        runtime::core::arrayf64_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::Object, StackValue::Reference(value)) => {
                        let value = value as usize as u64;
                        runtime::core::arrayobject_set(self, array, index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::NewObject(sym) => {
                let object = Runtime::new_object(*sym as usize);
                self.current_frame_mut().push(StackValue::Reference(object));
            }
            Bytecode::GetField(access, parent_name, index, tag) => {
                let object = self.current_frame_mut().pop();
                let object = match object {
                    StackValue::Reference(object) => object,
                    _ => todo!("report not accessing object correctly"),
                };

                match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = Object::get_8(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int8(value));
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = Object::get_16(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int16(value));
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = Object::get_32(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int32(value));
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = Object::get_64(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Int64(value));
                    }
                    TypeTag::F32 => {
                        let value = Object::get_f32(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Float32(value));
                    }
                    TypeTag::F64 => {
                        let value = Object::get_f64(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        self.current_frame_mut().push(StackValue::Float64(value));
                    }
                    TypeTag::Object => {
                        let value = Object::get_object(self, object, *access, *parent_name, *index);
                        if !self.handle_exception() {
                            return false;
                        }
                        let value = value as usize as Reference;
                        self.current_frame_mut().push(StackValue::Reference(value));
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::SetField(access, parent_name, index, tag) => {
                let value = self.current_frame_mut().pop();
                let object = self.current_frame_mut().pop();
                let object = match object {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing u64"),
                };
                match (tag, value) {
                    (TypeTag::U8, StackValue::Int8(value)) | (TypeTag::I8, StackValue::Int8(value)) => {
                        Object::set_8(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::U16, StackValue::Int16(value)) | (TypeTag::I16, StackValue::Int16(value)) => {
                        Object::set_16(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::U32, StackValue::Int32(value)) | (TypeTag::I32, StackValue::Int32(value)) => {
                        Object::set_32(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::U64, StackValue::Int64(value)) | (TypeTag::I64, StackValue::Int64(value)) => {
                        Object::set_64(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::F32, StackValue::Float32(value)) => {
                        Object::set_f32(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::F64, StackValue::Float64(value)) => {
                        Object::set_f64(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    (TypeTag::Object, StackValue::Reference(value)) => {
                        let value = value as usize as u64;
                        Object::set_object(self, object, *access, *parent_name, *index, value);
                        if !self.handle_exception() {
                            return false;
                        }
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::IsA(sym) => {
                let object = self.current_frame_mut().pop();
                let object = match object {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing object")
                };
                let object = unsafe {
                    object.as_ref().expect("check for null pointer")
                };
                let result = object.class as u64 == *sym;
                self.current_frame_mut().push(StackValue::Int8(result as u8));
            }
            Bytecode::InvokeVirt(specified, origin, method_name) => {
                return self.invoke_virtual(
                    *specified as runtime::Symbol,
                    origin.map(|s| s as runtime::Symbol),
                    *method_name as runtime::Symbol,
                )
            }
            Bytecode::InvokeVirtTail(..) => {
                todo!("Tail Recursion Virtual")
            }
            Bytecode::InvokeStatic(class_name, method_name) => {
                return self.invoke_static(
                    *class_name as runtime::Symbol,
                    *method_name as runtime::Symbol,
                )
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
                self.current_frame_mut().push(StackValue::Int64(*sym));
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
                    StackValue::Reference(exception) => exception,
                    _ => todo!("report exception needing to be an object"),
                };
                self.current_exception = exception;
            }
            Bytecode::StartBlock(_) => {}
            Bytecode::Goto(offset) => {
                self.current_frame_mut().goto(*offset as usize);
            }
            Bytecode::If(then_offset, else_offset) => {
                let value = self.current_frame_mut().pop();
                let boolean = match value {
                    StackValue::Int8(v) => v,
                    _ => todo!("report invalid type for boolean"),
                };

                if boolean != 0 {
                    self.current_frame_mut().goto(*then_offset as usize);
                } else {
                    self.current_frame_mut().goto(*else_offset as usize);
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
