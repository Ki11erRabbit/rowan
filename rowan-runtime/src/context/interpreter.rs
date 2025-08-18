mod stackframe;

use std::collections::HashSet;
use std::sync::TryLockError;
use fxhash::FxHashMap;
use rowan_shared::bytecode::linked::Bytecode;
use rowan_shared::TypeTag;
use crate::runtime;
use crate::context::{call_function_pointer, MethodName, WrappedReference};
use crate::runtime::{FunctionDetails, Reference, Runtime, DO_GARBAGE_COLLECTION};
use crate::runtime::object::Object;
use paste::paste;
use crate::context::interpreter::stackframe::{StackFrame};
use crate::runtime::core::interned_string_init;
use crate::runtime::garbage_collection::GarbageCollection;

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
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Int16(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Int32(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Int64(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Float32(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    StackValue::Float64(v) => {
                        for (buf, v) in buffer.iter_mut().zip(v.to_le_bytes().iter()) {
                            *buf = *v;
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





pub struct BytecodeContext {
    operand_stack: Vec<StackValue>,
    active_bytecodes: Vec<&'static [Bytecode]>,
    active_frames: Vec<StackFrame>,
    current_exception: Reference,
    call_args: [StackValue; 256],
}


impl BytecodeContext {
    pub fn new() -> Self {
        BytecodeContext {
            operand_stack: Vec::with_capacity(10),
            active_bytecodes: Vec::new(),
            active_frames: Vec::new(),
            current_exception: std::ptr::null_mut(),
            call_args: [StackValue::Blank; 256],
        }
    }
    
    pub fn push_value(&mut self, stack_value: StackValue) {
        assert_ne!(stack_value.is_blank(), true, "Added a blank to the stack");
        self.operand_stack.push(stack_value);
    }
    
    pub fn pop_value(&mut self) -> StackValue {
        self.operand_stack.pop().unwrap()
    }

    pub fn dup(&mut self) {
        let value = self.operand_stack.last().unwrap();
        self.operand_stack.push(*value);
    }

    pub fn swap(&mut self) {
        let value1 = self.pop_value();
        let value2 = self.pop_value();
        self.push_value(value2);
        self.push_value(value1);
    }

    pub fn store_local(&mut self, index: u8) {
        let value = self.pop_value();
        self.current_frame_mut().variables[index as usize] = value;
    }

    pub fn load_local(&mut self, index: u8) {
        let value = self.current_frame_mut().variables[index as usize];
        self.push_value(value);
    }

    pub fn store_argument(&mut self, index: u8, value: StackValue) {
        self.call_args[index as usize] = value;
    }

    pub fn store_argument_raw<V: Into<StackValue>>(&mut self, index: u8, value: V) {
        self.store_argument(index, value.into());
    }

    pub fn fetch_argument(&mut self, index: u8) -> StackValue {
        self.call_args[index as usize]
    }

    pub fn get_args(&self) -> &[StackValue] {
        &self.call_args
    }
    pub fn get_args_mut(&mut self) -> &mut [StackValue] {
        &mut self.call_args
    }


    pub fn is_current_exception_set(&self) -> bool {
        !self.current_exception.is_null()
    }

    pub fn push(&mut self, bytecode: &'static [Bytecode], is_for_bytecode: bool, method_name: MethodName, block_positions: &'static FxHashMap<usize, usize>) {
        self.active_bytecodes.push(bytecode);
        let args = self.get_args();
        self.active_frames.push(StackFrame::new(args, is_for_bytecode, method_name, block_positions));
        for arg in self.get_args_mut() {
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
    pub fn handle_exception(&mut self) -> CallContinueState {
        CallContinueState::Success
    }


    pub fn invoke_virtual(
        &mut self,
        specified: runtime::Symbol,
        method_name: runtime::Symbol,
        return_slot: Option<&mut StackValue>,
    ) -> CallContinueState {
        let object = self.call_args[0];
        let object = match object {
            StackValue::Reference(object) => object,
            _ => todo!("report error that first call arg must be an object.")
        };
        let object = unsafe {
            object.as_ref().expect("report null pointer")
        };

        let details = Runtime::get_virtual_method_details(
            object.class,
            specified,
            method_name,
        );

        let method_name = MethodName::VirtualMethod {
            object_class_symbol: object.class,
            class_symbol: specified,
            method_name,
        };

        self.call_function(details, method_name, return_slot)
    }


    pub fn invoke_static(
        &mut self,
        class_name: runtime::Symbol,
        method_name: runtime::Symbol,
        return_slot: Option<&mut StackValue>,
    ) -> CallContinueState {
        let details = Runtime::get_static_method_details(
            class_name,
            method_name,
        );
        println!("bytecode: {:#?}", details.bytecode);

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
        return_slot: Option<&mut StackValue>
    ) -> CallContinueState {
        for (i, pair) in self.call_args.iter().zip(details.arguments.iter()).enumerate() {
            println!("call arg{i}: {pair:?}");
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
                (value, type_tag) => {
                    todo!("report type error in typing for tag: {:?} and type_tag: {:?}", value, type_tag);
                }
            }
        }

        self.push(details.bytecode, details.fn_ptr.is_none(), method_name, details.block_positions);

        match details.fn_ptr {
            Some(fn_ptr) => {
                //println!("calling function pointer");
                let var_len = self.current_frame().vars_len();
                let mut variables = self.current_frame()
                    .variables[..var_len]
                    .to_vec();
                //super::sort_call_args(&mut variables);
                //let need_padding = super::need_padding(&variables);
                let return_value = call_function_pointer(
                    self,
                    &mut variables,
                    fn_ptr.as_ptr(),
                    details.return_type,
                );
                self.pop();
                if let Some(return_slot) = return_slot {
                    *return_slot = return_value;
                } else {
                    if !return_value.is_blank() {
                        self.push_value(return_value);
                    }
                }
                //self.handle_exception()
                CallContinueState::Return
            }
            _ => {
                CallContinueState::ExecuteFunction
            }
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
        self.active_frames.push(StackFrame::new(&[], details.fn_ptr.is_none(), method_name, details.block_positions));
        self.current_frame_mut().variables[0] = StackValue::Reference(std::ptr::null_mut());
        self.main_loop();
    }

    /// returns true if call finished without any errors
    /// returns false if an exception was thrown
    pub fn invoke_virtual_extern(
        &mut self,
        specified: runtime::Symbol,
        method_name: runtime::Symbol,
        return_slot: Option<&mut StackValue>,
    ) -> bool {
        let result = self.invoke_virtual(specified, method_name, return_slot);
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
        return_slot: Option<&mut StackValue>,
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
            let active_bytecode = self.active_bytecodes[self.active_bytecodes.len() - 1];
            let bytecode = &active_bytecode[self.current_frame().ip];
            self.current_frame_mut().ip += 1;

            if !self.interpret(bytecode) {
                break;
            }
            if self.active_frames.is_empty() {
                break;
            }
        }
    }

    pub fn run_bytecode(&mut self, bytecode: &'static [Bytecode], block_positions: &'static FxHashMap<usize, usize>) {
        self.active_bytecodes.push(bytecode);
        self.active_frames.push(StackFrame::new(&[], true, MethodName::StaticMethod { method_name: 0, class_symbol: 0 }, block_positions));
        self.main_loop();
        self.pop();
    }

    fn check_for_garbage_collection(&mut self) -> bool {
        //println!("attempting to read");
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


        GarbageCollection::send_references(references);
        loop {
            //println!("spinlock");
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
        for call_arg in self.call_args.iter() {
            match call_arg {
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
        println!("Bytecode: {bytecode:?}");
        match bytecode {
            Bytecode::Nop => {}
            Bytecode::Breakpoint => {}
            Bytecode::LoadU8(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadU16(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadU32(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadU64(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadI8(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadI16(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadI32(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadI64(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadF32(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadF64(value) => {
                self.push_value(StackValue::from(*value));
            }
            Bytecode::LoadSymbol(_sym) => {
                todo!("LoadSymbol")
            }
            Bytecode::Pop => {
                self.pop_value();
            }
            Bytecode::Dup => {
                self.dup();
            }
            Bytecode::Swap => {
                self.swap();
            }
            Bytecode::StoreLocal(index) => {
                self.store_local(*index);
            }
            Bytecode::LoadLocal(index) => {
                self.load_local(*index);
            }
            Bytecode::StoreArgument(index) => {
                let value = self.pop_value();
                self.store_argument(*index, value);
            }
            Bytecode::AddInt => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();

                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_add(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_add(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_add(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_add(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SubInt => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();

                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_sub(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_sub(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_sub(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs.wrapping_sub(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::MulInt => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();

                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_mul(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_mul(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_mul(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_mul(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::DivSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_div(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::DivUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_div(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_div(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_div(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_div(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::ModSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let result = lhs.wrapping_rem(rhs);
                        self.push_value(StackValue::from(result));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::ModUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_rem(rhs)));
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_rem(rhs)));
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_rem(rhs)));
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.push_value(StackValue::from(lhs.wrapping_rem(rhs)));
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::AddFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.push_value(StackValue::from(lhs + rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.push_value(StackValue::from(lhs + rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SubFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.push_value(StackValue::from(lhs - rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.push_value(StackValue::from(lhs - rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::MulFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.push_value(StackValue::from(lhs * rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.push_value(StackValue::from(lhs * rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::DivFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.push_value(StackValue::from(lhs / rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.push_value(StackValue::from(lhs / rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::ModFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        self.push_value(StackValue::from(lhs % rhs))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        self.push_value(StackValue::from(lhs % rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SatAddIntUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_add(rhs)))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_add(rhs)))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_add(rhs)))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_add(rhs)))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::SatSubIntUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_sub(rhs)))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_sub(rhs)))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_sub(rhs)))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        self.push_value(StackValue::from(lhs.saturating_sub(rhs)))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::And => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs & rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs & rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs & rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs & rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Or => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs | rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs | rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs | rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs | rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Xor => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs ^ rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs ^ rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs ^ rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs ^ rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Not => {
                let value = self.pop_value();
                match value {
                    StackValue::Int8(value) => {
                        self.push_value(StackValue::from(!value))
                    }
                    StackValue::Int16(value) => {
                        self.push_value(StackValue::from(!value))
                    }
                    StackValue::Int32(value) => {
                        self.push_value(StackValue::from(!value))
                    }
                    StackValue::Int64(value) => {
                        self.push_value(StackValue::from(!value))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Shl => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs << rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs << rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs << rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        self.push_value(StackValue::from(lhs << rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::AShr => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.push_value(StackValue::from(value))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.push_value(StackValue::from(value))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.push_value(StackValue::from(value))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >> rhs;
                        self.push_value(StackValue::from(value))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LShr => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        self.push_value(StackValue::from(lhs >> rhs))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        self.push_value(StackValue::from(lhs >> rhs))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        self.push_value(StackValue::from(lhs >> rhs))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {                        
                        self.push_value(StackValue::from(lhs >> rhs))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Neg => {
                let value = self.pop_value();
                match value {
                    StackValue::Int8(value) => {
                        let value = i8::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.push_value(StackValue::from(value))
                    }
                    StackValue::Int16(value) => {
                        let value = i16::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.push_value(StackValue::from(value))
                    }
                    StackValue::Int32(value) => {
                        let value = i32::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.push_value(StackValue::from(value))
                    }
                    StackValue::Int64(value) => {
                        let value = i64::from_ne_bytes(value.to_ne_bytes());
                        let value = -value;
                        self.push_value(StackValue::from(value))
                    }
                    StackValue::Float32(value) => {
                        let value = -value;
                        self.push_value(StackValue::from(value))
                    }
                    StackValue::Float64(value) => {
                        let value = -value;
                        self.push_value(StackValue::from(value))
                    }
                    StackValue::Reference(_) => {
                        todo!("Throw error saying that you can't negate references")
                    }
                    _ => unreachable!("You can't negate a blank")
                }
            }
            Bytecode::EqualSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::NotEqualSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::EqualUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let value = lhs == rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::NotEqualUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let value = lhs != rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        
                        
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        
                        
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        
                        
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        
                        
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs > rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs < rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterOrEqualSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessOrEqualSigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let lhs = i8::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i8::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let lhs = i16::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i16::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let lhs = i32::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i32::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let lhs = i64::from_ne_bytes(lhs.to_ne_bytes());
                        let rhs = i64::from_ne_bytes(rhs.to_ne_bytes());
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterOrEqualUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs >= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessOrEqualUnsigned => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Int8(lhs), StackValue::Int8(rhs)) => {
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int16(lhs), StackValue::Int16(rhs)) => {
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int32(lhs), StackValue::Int32(rhs)) => {
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Int64(lhs), StackValue::Int64(rhs)) => {
                        let value = lhs <= rhs;
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::EqualFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {                        
                        let value = lhs.eq(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.eq(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::NotEqualFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.ne(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.ne(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.gt(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.gt(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.lt(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.lt(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::GreaterOrEqualFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.ge(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.ge(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::LessOrEqualFloat => {
                let rhs = self.pop_value();
                let lhs = self.pop_value();
                match (lhs, rhs) {
                    (StackValue::Float32(lhs), StackValue::Float32(rhs)) => {
                        let value = lhs.le(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    (StackValue::Float64(lhs), StackValue::Float64(rhs)) => {
                        let value = lhs.le(&rhs);
                        self.push_value(StackValue::from(value as u8))
                    }
                    _ => {
                        todo!("Throw error saying that types should match if they are different")
                    }
                }
            }
            Bytecode::Convert(tag) => {
                let value = self.pop_value();
                match tag {
                    TypeTag::U8 => {
                        let value = value.as_u8();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U16 => {
                        let value = value.as_u16();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U32 => {
                        let value = value.as_u32();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U64 => {
                        let value = value.as_u64();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I8 => {
                        let value = value.as_i8();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I16 => {
                        let value = value.as_i16();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I32 => {
                        let value = value.as_i32();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I64 => {
                        let value = value.as_i64();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F32 => {
                        let value = value.as_f32();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F64 => {
                        let value = value.as_f64();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::Object => {
                        todo!("report object conversion errors")
                    }
                    _ => unreachable!("Invalid Type Tag")
                }
            }
            Bytecode::BinaryConvert(tag) => {
                let value = self.pop_value();
                match tag {
                    TypeTag::U8 => {
                        let value = value.into_u8();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U16 => {
                        let value = value.into_u16();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U32 => {
                        let value = value.into_u32();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U64 => {
                        let value = value.into_u64();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I8 => {
                        let value = value.into_i8();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I16 => {
                        let value = value.into_i16();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I32 => {
                        let value = value.into_i32();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::I64 => {
                        let value = value.into_i64();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F32 => {
                        let value = value.into_f32();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F64 => {
                        let value = value.into_f64();
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::Object => {
                        todo!("report object conversion errors")
                    }
                    _ => unreachable!("Invalid Type Tag")
                }
            }
            Bytecode::CreateArray(tag) => {
                let size = self.pop_value();
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
                self.push_value(StackValue::from(object));
            }
            Bytecode::ArrayGet(tag) => {
                let index = self.pop_value();
                let index = match index {
                    StackValue::Int64(index) => index,
                    _ => todo!("report needing u64"),
                };
                let array = self.pop_value();
                let array = match array {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing reference for index"),
                };
                match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = runtime::core::array8_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = runtime::core::array16_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = runtime::core::array32_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = runtime::core::array64_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F32 => {
                        let value = runtime::core::arrayf32_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F64 => {
                        let value = runtime::core::arrayf64_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::Object => {
                        let value = runtime::core::arrayobject_get(self, array, index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        let value = value as usize as Reference;
                        self.push_value(StackValue::from(value));
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::ArraySet(tag) => {
                let value = self.pop_value();
                let index = self.pop_value();
                let index = match index {
                    StackValue::Int64(index) => index,
                    _ => todo!("report needing u64"),
                };
                let array = self.pop_value();
                let array = match array {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing reference for index"),
                };
                match (tag, value) {
                    (TypeTag::U8, StackValue::Int8(value)) | (TypeTag::I8, StackValue::Int8(value)) => {

                        runtime::core::array8_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U16, StackValue::Int16(value)) | (TypeTag::I16, StackValue::Int16(value)) => {

                        runtime::core::array16_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U32, StackValue::Int32(value)) | (TypeTag::I32, StackValue::Int32(value)) => {

                        runtime::core::array32_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U64, StackValue::Int64(value)) | (TypeTag::I64, StackValue::Int64(value)) => {

                        runtime::core::array64_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F32, StackValue::Float32(value)) => {

                        runtime::core::arrayf32_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F64, StackValue::Float64(value)) => {

                        runtime::core::arrayf64_set(self, array, index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::Object, StackValue::Reference(value)) => {
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
                self.push_value(StackValue::from(object));
            }
            Bytecode::GetField(access, parent_name, index, tag) => {
                let object = self.pop_value();
                let object = match object {
                    StackValue::Reference(object) => object,
                    _ => todo!("report not accessing object correctly"),
                };

                match tag {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = Object::get_8(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = Object::get_16(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = Object::get_32(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = Object::get_64(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F32 => {
                        let value = Object::get_f32(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::F64 => {
                        let value = Object::get_f64(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        self.push_value(StackValue::from(value));
                    }
                    TypeTag::Object => {
                        let value = Object::get_object(self, object, *access, *parent_name, *index);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                        let value = value as usize as Reference;
                        self.push_value(StackValue::from(value));
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::SetField(access, parent_name, index, tag) => {
                let value = self.pop_value();
                let object = self.pop_value();
                let object = match object {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing u64"),
                };
                match (tag, value) {
                    (TypeTag::U8, StackValue::Int8(value)) | (TypeTag::I8, StackValue::Int8(value)) => {

                        Object::set_8(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U16, StackValue::Int16(value)) | (TypeTag::I16, StackValue::Int16(value)) => {

                        Object::set_16(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U32, StackValue::Int32(value)) | (TypeTag::I32, StackValue::Int32(value)) => {

                        Object::set_32(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::U64, StackValue::Int64(value)) | (TypeTag::I64, StackValue::Int64(value)) => {

                        Object::set_64(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F32, StackValue::Float32(value)) => {

                        Object::set_f32(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::F32, StackValue::Float64(value)) => {

                        Object::set_f64(self, object, *access, *parent_name, *index, value);
                        match self.handle_exception() {
                            CallContinueState::Error => return false,
                            _ => {}
                        }
                    }
                    (TypeTag::Object, StackValue::Reference(value)) => {
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
                let object = self.pop_value();
                let object = match object {
                    StackValue::Reference(object) => object,
                    _ => todo!("report needing object")
                };
                let object = unsafe {
                    object.as_ref().expect("check for null pointer")
                };
                let result = object.class as u64 == *sym;
                self.push_value(StackValue::from(result as u8));
            }
            Bytecode::InvokeVirt(specified, method_name) => {
                match self.invoke_virtual(
                    *specified as runtime::Symbol,
                    *method_name as runtime::Symbol,
                    None
                ) {
                    CallContinueState::Error => return false,
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
                    _ => return true,
                }
            }
            Bytecode::InvokeStaticTail(..) => {
                todo!("Tail Recursion Static")
            }
            Bytecode::GetStaticMethod(..) => {
                todo!("conversion of a static method into an object")
            }
            Bytecode::GetStaticMember(class, index, ty) => {
                match ty {
                    TypeTag::U8 | TypeTag::I8 => {
                        let value = Runtime::get_static_member::<u8>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    TypeTag::U16 | TypeTag::I16 => {
                        let value = Runtime::get_static_member::<u16>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    TypeTag::U32 | TypeTag::I32 => {
                        let value = Runtime::get_static_member::<u32>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    TypeTag::U64 | TypeTag::I64 => {
                        let value = Runtime::get_static_member::<u64>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    TypeTag::F32 => {
                        let value = Runtime::get_static_member::<f32>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    TypeTag::F64 => {
                        let value = Runtime::get_static_member::<f64>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    TypeTag::Object => {
                        let value = Runtime::get_static_member::<Reference>(self, *class as runtime::Symbol, *index);
                        let value = StackValue::from(value);
                        self.push_value(value);
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::SetStaticMember(class, index, ty) => {
                let value = self.pop_value();
                match (value, ty) {
                    (StackValue::Int8(value), TypeTag::U8) | (StackValue::Int8(value), TypeTag::I8) => {
                        Runtime::set_static_member::<u8>(self, *class as runtime::Symbol, *index, value);
                    }
                    (StackValue::Int16(value), TypeTag::U16) | (StackValue::Int16(value), TypeTag::I16) => {
                        Runtime::set_static_member::<u16>(self, *class as runtime::Symbol, *index, value);
                    }
                    (StackValue::Int32(value), TypeTag::U32) | (StackValue::Int32(value), TypeTag::I32) => {
                        Runtime::set_static_member::<u32>(self, *class as runtime::Symbol, *index, value);
                    }
                    (StackValue::Int64(value), TypeTag::U64) | (StackValue::Int64(value), TypeTag::I64) => {
                        Runtime::set_static_member::<u64>(self, *class as runtime::Symbol, *index, value);
                    }
                    (StackValue::Float32(value), TypeTag::F32) => {
                        Runtime::set_static_member::<f32>(self, *class as runtime::Symbol, *index, value);
                    }
                    (StackValue::Float64(value), TypeTag::F64) => {
                        Runtime::set_static_member::<f64>(self, *class as runtime::Symbol, *index, value);
                    }
                    (StackValue::Reference(value), TypeTag::Object) => {
                        Runtime::set_static_member::<Reference>(self, *class as runtime::Symbol, *index, value);
                    }
                    _ => unreachable!("Invalid Type Tag"),
                }
            }
            Bytecode::GetStrRef(sym) => {
                let interned_string = interned_string_init(*sym) as Reference;

                self.push_value(StackValue::from(interned_string));
            }
            Bytecode::Return => {
                self.check_and_do_garbage_collection();
                let value = self.pop_value();
                self.pop();
                self.push_value(value);
                if self.active_frames.len() == 1 {
                    return false;
                }
            }
            Bytecode::ReturnVoid => {
                self.check_and_do_garbage_collection();
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
                let exception = self.pop_value();
                let exception = match exception {
                    StackValue::Reference(exception) => exception,
                    _ => todo!("report exception needing to be an object"),
                };
                self.current_exception = exception;
            }
            Bytecode::StartBlock(_) => {
                self.check_and_do_garbage_collection();
            }
            Bytecode::Goto(offset) => {
                self.current_frame_mut().goto(*offset as isize);
            }
            Bytecode::If(then_offset, else_offset) => {
                let value = self.pop_value();
                let boolean = match value {
                    StackValue::Int8(value) => value,
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
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn store_argument_int16(&mut self, index: u8, value: u16) {
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn store_argument_int32(&mut self, index: u8, value: u32) {
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn store_argument_int64(&mut self, index: u8, value: u64) {
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn store_argument_float32(&mut self, index: u8, value: f32) {
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn store_argument_float64(&mut self, index: u8, value: f64) {
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn store_argument_object(&mut self, index: u8, value: Reference) {
        self.store_argument_raw(index, value);
    }

    pub extern "C" fn fetch_argument_int8(&mut self, index: u8) -> u8 {
        match self.fetch_argument(index) {
            StackValue::Int8(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_argument_int16(&mut self, index: u8) -> u16 {
        match self.fetch_argument(index) {
            StackValue::Int16(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_argument_int32(&mut self, index: u8) -> u32 {
        match self.fetch_argument(index) {
            StackValue::Int32(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_argument_int64(&mut self, index: u8) -> u64 {
        match self.fetch_argument(index) {
            StackValue::Int64(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_argument_object(&mut self, index: u8) -> Reference {
        match self.fetch_argument(index) {
            StackValue::Reference(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_argument_float32(&mut self, index: u8) -> f32 {
        match self.fetch_argument(index) {
            StackValue::Float32(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_argument_float64(&mut self, index: u8) -> f64 {
        match self.fetch_argument(index) {
            StackValue::Float64(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_int8(&mut self) -> u8 {
        match self.pop_value() {
            StackValue::Int8(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_int16(&mut self) -> u16 {
        match self.pop_value() {
            StackValue::Int16(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_int32(&mut self) -> u32 {
        match self.pop_value() {
            StackValue::Int32(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_int64(&mut self) -> u64 {
        match self.pop_value() {
            StackValue::Int64(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_object(&mut self) -> Reference {
        match self.pop_value() {
            StackValue::Reference(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_float32(&mut self) -> f32 {
        match self.pop_value() {
            StackValue::Float32(value) => value,
            _ => panic!("invalid type")
        }
    }

    pub extern "C" fn fetch_return_float64(&mut self) -> f64 {
        match self.pop_value() {
            StackValue::Float64(value) => value,
            _ => panic!("invalid type")
        }
    }
}
