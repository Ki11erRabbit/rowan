use std::collections::HashMap;
use paste::paste;
use rowan_shared::bytecode::linked::Bytecode;
use rowan_shared::TypeTag;
use crate::runtime::Reference;

#[derive(Clone, Copy)]
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
                let mut buffer = [u8; std::mem::size_of::<$typ>()];
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
    pub fn new(args: &[StackValue], bytecode: &[Bytecode]) -> Self {
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
            *variable = *arg;
        }
        Self {
            operand_stack: Vec::new(),
            ip: 0,
            current_block: 0,
            block_positions,
            variables: [StackValue::Blank; 256],
            call_args: [StackValue::Blank; 256],
            is_for_bytecode: bytecode.len() > 0,
        }
    }

    pub fn push(&mut self, stack_value: StackValue) {
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

    pub fn is_for_bytecode(&self) -> bool {
        self.is_for_bytecode
    }
}


pub struct BytecodeContext {
    active_bytecodes: Vec<&'static [Bytecode]>,
    active_frames: Vec<StackFrame>,
}


impl BytecodeContext {
    pub fn new() -> Self {
        BytecodeContext {
            active_bytecodes: Vec::new(),
            active_frames: Vec::new(),
        }
    }

    pub fn push(&mut self, bytecode: &'static [Bytecode]) {
        self.active_bytecodes.push(bytecode);
        let frame = self.current_frame();
        let args = frame.get_args();
        self.active_frames.push(StackFrame::new(args, bytecode));
    }

    pub fn pop(&mut self) {
        self.active_bytecodes.pop();
        self.active_frames.pop();
    }

    pub fn current_frame(&self) -> &StackFrame {
        &self.active_frames[self.active_bytecodes.len() - 1]
    }

    pub fn current_frame_mut(&mut self) -> &mut StackFrame {
        &mut self.active_frames[self.active_bytecodes.len() - 1]
    }

    pub fn interpret(&mut self, bytecode: &Bytecode) {
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
                self.current_frame_mut().load_local(*index);
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
                        let value = u32::from_ne_bytes(value.to_ne_bytes());
                        self.current_frame_mut().push(StackValue::Int32(value))
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

        }
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
