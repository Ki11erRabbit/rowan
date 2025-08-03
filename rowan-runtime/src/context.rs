mod interpreter;
use std::ffi::c_void;
use libffi::low::{call, ffi_cif, CodePtr};
use libffi::raw::{ffi_prep_cif};
use libffi_sys::{ffi_abi, ffi_abi_FFI_WIN64, ffi_type_double, ffi_type_float, ffi_type_pointer, ffi_type_uint16, ffi_type_uint32, ffi_type_uint64, ffi_type_uint8, ffi_type_void};
pub use interpreter::BytecodeContext;
use crate::runtime::{Reference, Symbol};
use crate::runtime::class::TypeTag;

pub use interpreter::StackValue;


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

#[cfg(unix)]
const CALLING_CONVENTION: ffi_abi = libffi::raw::ffi_abi_FFI_SYSV;

#[cfg(windows)]
const CALLING_CONVENTION: ffi_abi = libffi::raw::ffi_abi_FFI_WIN64;

pub fn call_function_pointer(context: &mut BytecodeContext, call_args: &mut [StackValue], fn_pointer: *const (), return_type: TypeTag) -> StackValue {
    let mut cif = ffi_cif::default();
    let mut types = Vec::new();
    let mut values = Vec::new();
    let call_args_len = call_args.len() as u32 + 1;
    // Adding context
    unsafe {
        types.push(&raw mut ffi_type_pointer);
    }
    values.push(context as *mut _ as *mut c_void);
    for call_arg in call_args {
        match call_arg {
            StackValue::Int8(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_uint8 as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            StackValue::Int16(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_uint16 as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            StackValue::Int32(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_uint32 as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            StackValue::Int64(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_uint64 as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            StackValue::Reference(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_pointer as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            StackValue::Float32(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_float as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            StackValue::Float64(value) => {
                unsafe {
                    types.push(&raw mut ffi_type_double as *mut _);
                }
                values.push(value as *mut _ as *mut c_void);
            }
            _ => unreachable!("argument conversion")
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

    match return_type {
        TypeTag::I8 | TypeTag::U8 => {
            let out = unsafe {
                call::<u8>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::I16 | TypeTag::U16 => {
            let out = unsafe {
                call::<u16>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::I32 | TypeTag::U32 => {
            let out = unsafe {
                call::<u32>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::I64 | TypeTag::U64=> {
            let out = unsafe {
                call::<u64>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::F32 => {
            let out = unsafe {
                call::<f32>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::F64 => {
            let out = unsafe {
                call::<f64>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::Object => {
            let out = unsafe {
                call::<Reference>(&mut cif, CodePtr(fn_pointer as *mut _), values.as_mut_ptr())
            };
            StackValue::from(out)
        }
        TypeTag::Void => {
            StackValue::Blank
        }
        x => unreachable!("return type: {x:?}")
    }

}
