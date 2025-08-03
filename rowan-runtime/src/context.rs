mod interpreter;
/*#[cfg(target_arch = "x86_64")]
mod amd64;
#[cfg(target_arch = "x86_64")]
#[cfg(unix)]
pub(crate) use amd64::unix::*;*/

#[cfg(unix)]
const CALLING_CONVENTION: ffi_abi = libffi_sys::ffi_abi_FFI_UNIX64;

#[cfg(windows)]
const CALLING_CONVENTION: ffi_abi = libffi_sys::ffi_abi_FFI_WIN64;

pub fn call_function_pointer(context: &mut BytecodeContext, call_args: &mut [Value], fn_pointer: *const (), return_type: TypeTag) -> Value {
    let mut cif = ffi_cif::default();
    let mut types = Vec::new();
    let mut values = Vec::new();
    let call_args_len = call_args.len() as u32;
    for call_arg in call_args {
        match call_arg {
            Value { tag: 0, value } => unsafe {
                types.push(&raw mut ffi_type_uint8 as *mut _);
                values.push(&mut value.c as *mut _ as *mut c_void);
            }
            Value { tag: 1, value } => unsafe {
                types.push(&raw mut ffi_type_uint16 as *mut _);
                values.push(&mut value.s as *mut _ as *mut c_void);
            }
            Value { tag: 2, value } => unsafe {
                types.push(&raw mut ffi_type_uint32 as *mut _);
                values.push(&mut value.i as *mut _ as *mut c_void);
            }
            Value { tag: 3, value } => unsafe {
                types.push(&raw mut ffi_type_uint64 as *mut _);
                values.push(&mut value.l as *mut _ as *mut c_void);
            }
            Value { tag: 4, value } => unsafe {
                types.push(&raw mut ffi_type_uint64 as *mut _);
                values.push(&mut value.r as *mut _ as *mut c_void);
            }
            Value { tag: 5, value } => unsafe {
                types.push(&raw mut ffi_type_float as *mut _);
                values.push(&mut value.f as *mut _ as *mut c_void);
            }
            Value { tag: 6, value } => unsafe {
                types.push(&raw mut ffi_type_double as *mut _);
                values.push(&mut value.d as *mut _ as *mut c_void);
            }
            _ => unreachable!("as type")
        }
    }

    let mut ret_type = match return_type {
        TypeTag::I8 | TypeTag::U8 => unsafe {
            ffi_type_uint8
        }
        TypeTag::I16 | TypeTag::U16 => unsafe {
            ffi_type_uint16
        }
        TypeTag::I32 | TypeTag::U32 => unsafe {
            ffi_type_uint32
        }
        TypeTag::I64 | TypeTag::U64 | TypeTag::Object => unsafe {
            ffi_type_uint64
        }
        TypeTag::F32 => unsafe {
            ffi_type_float
        }
        TypeTag::F64 => unsafe {
            ffi_type_double
        }
        TypeTag::Void => unsafe {
            ffi_type_void
        }
        x => unreachable!("return type: {x:?}")

    };

    unsafe {
        ffi_prep_cif(&mut cif, CALLING_CONVENTION, call_args_len, &mut ret_type, types.as_mut_ptr());
    }

    unsafe {
        match return_type {
            TypeTag::I8 | TypeTag::U8 => unsafe {
                let out = call::<u8>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr());
                Value::from(out)
            }
            TypeTag::I16 | TypeTag::U16 => unsafe {
                let out = call::<u16>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr());
                Value::from(out)
            }
            TypeTag::I32 | TypeTag::U32 => unsafe {
                let out = call::<u32>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr());
                Value::from(out)
            }
            TypeTag::I64 | TypeTag::U64 | TypeTag::Object => unsafe {
                let out = call::<u64>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr());
                Value::from(out)
            }
            TypeTag::F32 => unsafe {
                let out = call::<f32>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr());
                Value::from(out)
            }
            TypeTag::F64 => unsafe {
                let out = call::<f64>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr());
                Value::from(out)
            }
            TypeTag::Void => unsafe {
                Value::blank()
            }
            x => unreachable!("return type: {x:?}")
        }
    }
}

use std::ffi::c_void;
use libffi::low::{call, ffi_cif, CodePtr};
use libffi::raw::{ffi_abi_FFI_UNIX64, ffi_prep_cif};
use libffi_sys::{ffi_abi, ffi_abi_FFI_WIN64, ffi_type_double, ffi_type_float, ffi_type_uint16, ffi_type_uint32, ffi_type_uint64, ffi_type_uint8, ffi_type_void};
pub use interpreter::BytecodeContext;
use crate::runtime::{Reference, Symbol};
use paste::paste;
use crate::runtime::class::TypeTag;

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

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct WrappedReference(pub Reference);

unsafe impl Send for WrappedReference {}
unsafe impl Sync for WrappedReference {}

#[derive(Copy, Clone, Debug)]
pub enum MethodName {
    StaticMethod {
        class_symbol: Symbol,
        method_name: Symbol,
    },
    VirtualMethod {
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        source_class: Option<Symbol>,
        method_name: Symbol,
    }
}

impl MethodName {
    pub fn get_method_name(&self) -> Symbol {
        match self {
            MethodName::StaticMethod { method_name, .. } => *method_name,
            MethodName::VirtualMethod { method_name, .. } => *method_name,
        }
    }
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