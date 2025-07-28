mod interpreter;
#[cfg(target_arch = "x86_64")]
mod amd64;
#[cfg(target_arch = "x86_64")]
#[cfg(unix)]
pub(crate) use amd64::unix::*;

pub use interpreter::BytecodeContext;
use crate::runtime::Reference;
use paste::paste;

macro_rules! as_type {
    ($typ:ty) => {
        paste! {
            pub fn [<as_ $typ>](self) -> $typ {
                match self {
                    Value { tag: 0, value } => unsafe {
                        value.c as $typ
                    }
                    Value { tag: 1, value } => unsafe {
                        value.s as $typ
                    }
                    Value { tag: 2, value } => unsafe {
                        value.i as $typ
                    }
                    Value { tag: 3, value } => unsafe {
                        value.l as $typ
                    }
                    Value { tag: 4, .. } => {
                        panic!("cannot cast object");
                    }
                    Value { tag: 5, value } => unsafe {
                        value.f as $typ
                    }
                    Value { tag: 6, value } => unsafe {
                        value.c as $typ
                    }
                    _ => unreachable!("as type")
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
                    Value { tag: 0, value } => unsafe {
                        for (buf, v) in buffer.iter_mut().zip(value.c.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    Value { tag: 1, value } => unsafe {
                        for (buf, v) in buffer.iter_mut().zip(value.s.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    Value { tag: 2, value } => unsafe {
                        for (buf, v) in buffer.iter_mut().zip(value.i.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    Value { tag: 3, value } => unsafe {
                        for (buf, v) in buffer.iter_mut().zip(value.l.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    Value { tag: 4, .. } => {
                        panic!("cannot cast object");
                    }
                    Value { tag: 5, value } => unsafe {
                        for (buf, v) in buffer.iter_mut().zip(value.f.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    Value { tag: 6, value } => unsafe {
                        for (buf, v) in buffer.iter_mut().zip(value.d.to_le_bytes().iter()) {
                            *buf = *v;
                        }
                        $typ::from_le_bytes(buffer)
                    }
                    _ => unreachable!("cannot cast blank into a type")
                }
            }
        }
    };
}


#[derive(Copy, Clone)]
pub union ValueUnion {
    c: u8,
    s: u16,
    i: u32,
    l: u64,
    r: Reference,
    f: f32,
    d: f64,
    blank: u64,
}

#[derive(Copy, Clone)]
pub struct Value {
    tag: u64,
    value: ValueUnion,
}

impl Value {
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

    pub fn new(tag: u64, value: ValueUnion) -> Self {
        Self { tag, value }
    }

    pub fn tag(&self) -> u64 {
        self.tag
    }

    pub fn is_blank(&self) -> bool {
        match self {
            Value { tag: 7, .. } => true,
            _ => false,
        }
    }

    pub fn blank() -> Self {
        Self { tag: 7, value: ValueUnion { blank: 0 } }
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Value::new(0, ValueUnion { c: v })
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Value::new(0, ValueUnion { c: u8::from_ne_bytes(v.to_ne_bytes()) })
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Value::new(1, ValueUnion { s: v })
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Value::new(1, ValueUnion { s: u16::from_ne_bytes(v.to_ne_bytes()) })
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Value::new(2, ValueUnion { i: v })
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::new(2, ValueUnion { i: u32::from_ne_bytes(v.to_ne_bytes()) })
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Value::new(3, ValueUnion { l: v })
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::new(3, ValueUnion { l: u64::from_ne_bytes(v.to_ne_bytes()) })
    }
}

impl From<Reference> for Value {
    fn from(r: Reference) -> Self {
        Value::new(4, ValueUnion { r })
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::new(5, ValueUnion { f: v })
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::new(6, ValueUnion { d: v })
    }
}