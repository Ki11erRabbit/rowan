use std::ffi::{c_char, c_void, CStr};
use crate::context::{BytecodeContext, StackValue};
use crate::runtime::{Runtime, Reference};
use crate::runtime::core::{array16_init, array32_init, array64_init, array8_init, arrayf32_init, arrayf64_init, arrayobject_init, string_buffer_from_str, Array, InternedString, StringBuffer};

#[repr(C)]
#[derive(Copy, Clone)]
pub union FFIValueUnion {
    blank: u8,
    byte: u8,
    short: u16,
    int: u32,
    long: u64,
    float: f32,
    double: f64,
    reference: Reference,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFIValue {
    tag: u64,
    value: FFIValueUnion,
}

impl FFIValue {
    #[unsafe(no_mangle)]
    pub extern "C"  fn rowan_new_value() -> FFIValue {
        FFIValue {
            tag: 0,
            value: FFIValueUnion { blank: 0 },
        }
    }
}

impl From<StackValue> for FFIValue {
    fn from(value: StackValue) -> Self {
        match value {
            StackValue::Blank => {
                FFIValue {
                    tag: 0,
                    value: FFIValueUnion { blank: 0 },
                }
            }
            StackValue::Int8(v) => {
                FFIValue {
                    tag: 1,
                    value: FFIValueUnion { byte: v },
                }
            }
            StackValue::Int16(v) => {
                FFIValue {
                    tag: 2,
                    value: FFIValueUnion { short: v },
                }
            }
            StackValue::Int32(v) => {
                FFIValue {
                    tag: 3,
                    value: FFIValueUnion { int: v },
                }
            }
            StackValue::Int64(v) => {
                FFIValue {
                    tag: 4,
                    value: FFIValueUnion { long: v },
                }
            }
            StackValue::Float32(v) => {
                FFIValue {
                    tag: 5,
                    value: FFIValueUnion { float: v },
                }
            }
            StackValue::Float64(v) => {
                FFIValue {
                    tag: 6,
                    value: FFIValueUnion { double: v },
                }
            }
            StackValue::Reference(v) => {
                FFIValue {
                    tag: 7,
                    value: FFIValueUnion { reference: v },
                }
            }
        }
    }
}

impl Into<StackValue> for FFIValue {
    fn into(self) -> StackValue {
        match self.tag {
            0 => StackValue::Blank,
            1 => unsafe {
                StackValue::Int8(self.value.byte)
            }
            2 => unsafe {
                StackValue::Int16(self.value.short)
            }
            3 => unsafe {
                StackValue::Int32(self.value.int)
            }
            4 => unsafe {
                StackValue::Int64(self.value.long)
            }
            5 => unsafe {
                StackValue::Float32(self.value.float)
            }
            6 => unsafe {
                StackValue::Float64(self.value.double)
            }
            7 => unsafe {
                StackValue::Reference(self.value.reference)
            }
            _ => panic!("Invalid stack value type"),
        }
    }
}

/// This function constructs an object from a given class name from a CStr.
/// The CStr should be valid utf-8 as to prevent misses.
/// Returns a valid reference to an object
#[unsafe(no_mangle)]
pub extern "C" fn rowan_create_object(class_name: *const c_char) -> Reference {
    let class_name = unsafe { CStr::from_ptr(class_name) };
    let name = class_name.to_string_lossy();
    Runtime::new_object(name.as_ref())
}

#[unsafe(no_mangle)]
pub extern "C" fn rowan_create_array(context: &mut BytecodeContext, array_type: *const c_char, length: u64) -> Reference {
    let class_name = unsafe { CStr::from_ptr(array_type) };
    let name = class_name.to_string_lossy();
    match name.as_ref() {
        "8" => {
            let object = Runtime::new_object("core::Array8");
            array8_init(context, object, length);
            object
        }
        "16" => {
            let object = Runtime::new_object("core::Array16");
            array16_init(context, object, length);
            object
        }
        "32" => {
            let object = Runtime::new_object("core::Array32");
            array32_init(context, object, length);
            object
        }
        "64" => {
            let object = Runtime::new_object("core::Array64");
            array64_init(context, object, length);
            object
        }
        "f32" => {
            let object = Runtime::new_object("core::Arrayf32");
            arrayf32_init(context, object, length);
            object
        }
        "f64" => {
            let object = Runtime::new_object("core::Arrayf64");
            arrayf64_init(context, object, length);
            object
        }
        "object" => {
            let object = Runtime::new_object("core::ArrayObject");
            arrayobject_init(context, object, length);
            object
        }
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rowan_get_array_buffer(array: &mut Array, buf: *mut *mut c_void, length: &mut u64) {
    unsafe {
        *buf = array.buffer as *mut c_void;
    }
    *length = array.length;
}

/// This function will mark an object and its parent objects to be uncollectable.
/// This is to allow for the passing of objects into FFI boundaries where a GC might
/// not be able to find the memory, especially if the object is used for a callback.
#[unsafe(no_mangle)]
pub extern "C" fn rowan_block_collection(object: Reference) {
    Runtime::block_collection(object);
}

/// This function will mark an object and its parents as being collectable again.
/// This is so that there are no leaks across an FFI boundary.
#[unsafe(no_mangle)]
pub extern "C" fn rowan_allow_collection(object: Reference) {
    Runtime::allow_collection(object);
}

/// This function is a convenience function to allow for quickly making strings from a CStr.
/// The CStr should be valid utf-8.
/// Returns a valid reference to a string object
#[unsafe(no_mangle)]
pub extern "C" fn rowan_create_string_buffer(string_contents: *const c_char) -> *mut StringBuffer {
    let string = Runtime::new_object("core::StringBuffer");
    let string = string as *mut StringBuffer;
    let string_contents = unsafe { CStr::from_ptr(string_contents) };
    let contents = string_contents.to_string_lossy();
    string_buffer_from_str(string, contents.as_ref());
    string
}

/// This function is a convenience function to allow for quickly making empty strings.
/// Returns a valid reference to a string object
#[unsafe(no_mangle)]
pub extern "C" fn rowan_create_empty_string_buffer() -> *mut StringBuffer {
    let string = Runtime::new_object("core::StringBuffer");
    let string = string as *mut StringBuffer;
    let string = unsafe { string.as_mut().unwrap() };
    string.initialize(std::ptr::null_mut(), 0, 0);
    string
}

#[unsafe(no_mangle)]
pub extern "C" fn rowan_get_string_buffer(string: Reference, buf: &mut *const u8, length: &mut u64) {
    let string = unsafe { string.as_mut().unwrap() };
    let (class_symbol, method_symbol) = Runtime::get_virtual_method_name(
        "core::String", 
        "core::String::get-buffer"
    ).unwrap();
    let get_buffer_details = Runtime::get_virtual_method_details(string.class, class_symbol, method_symbol);
    let get_buffer = get_buffer_details.fn_ptr.unwrap();
    let get_buffer = unsafe {
        std::mem::transmute::<_, extern "C" fn(Reference, &mut *const u8, &mut u64)>(get_buffer)
    };
    
    get_buffer(string, buf, length);
}

pub extern "C" fn rowan_set_call_argument(context: &mut BytecodeContext, index: u8, value: FFIValue) {
    let stack_value: StackValue = value.into();
    context.store_argument_raw(index, stack_value);
}


/// This function retrieves the function pointer for a virtual function for a given object.
/// object: the object to get the function pointer
/// class: the class with the particular method
/// source_class: the parent class of the object to start looking. Can be null.
/// method_name: the name of the method to return
/// Returns a pointer to a function. It is up to the caller to cast it correctly
#[unsafe(no_mangle)]
pub extern "C" fn rowan_call_virtual_function(
    context: &mut BytecodeContext,
    class: *const c_char,
    method_name: *const c_char,
    return_slot: Option<&mut StackValue>,
) -> i32 {
    let class = unsafe { CStr::from_ptr(class) };
    let class = class.to_string_lossy();
    let method_name = unsafe { CStr::from_ptr(method_name) };
    let method_name = method_name.to_string_lossy();

    let Some((class, method_name)) = Runtime::get_virtual_method_name(&class, &method_name) else {
        return 2;
    };

    let mut return_value = StackValue::Blank;

    let result = if return_slot.is_some() {
        context.invoke_virtual_extern(class, method_name, Some(&mut return_value))
    } else {
        context.invoke_virtual_extern(class, method_name, None)
    };

    match return_slot {
        Some(slot) => {
            *slot = return_value.into();
        }
        _ => {}
    }

    if result {
        0
    } else {
        1
    }
}

/// This function retrieves the function pointer for a static function for a given class.
/// class: the class with the particular method
/// method_name: the name of the method
/// return_slot: an optional return parameter
/// returns an i32 indicating success and failure
/// `0` success
/// `1` unwinding failure
/// `2` unwinding failure from unknown method
#[unsafe(no_mangle)]
pub extern "C" fn rowan_call_static_function(
    context: &mut BytecodeContext,
    class: *const c_char,
    method_name: *const c_char,
    return_slot: Option<&mut FFIValue>,
) -> i32 {
    let class = unsafe { CStr::from_ptr(class) };
    let class = class.to_string_lossy();
    let method_name = unsafe { CStr::from_ptr(method_name) };
    let method_name = method_name.to_string_lossy();

    let Some((class_name, method_name)) = Runtime::get_static_method_name(&class, &method_name) else {
        return 2;
    };

    let mut return_value = StackValue::Blank;

    let result = if return_slot.is_some() {
        context.invoke_static_extern(class_name, method_name, Some(&mut return_value))
    } else {
        context.invoke_static_extern(class_name, method_name, None)
    };

    match return_slot {
        Some(slot) => {
            *slot = return_value.into();
        }
        _ => {}
    }

    if result {
        0
    } else {
        1
    }
}

/// This function retrieves the function pointer for a virtual function for a given object.
/// object: the object to get the function pointer
/// class: the class with the particular method
/// source_class: the parent class of the object to start looking. Can be null.
/// method_name: the name of the method to return
/// Returns a pointer to a function. It is up to the caller to cast it correctly
#[unsafe(no_mangle)]
pub extern "C" fn rowan_call_interface_function(
    context: &mut BytecodeContext,
    interface: *const c_char,
    method_name: *const c_char,
    return_slot: Option<&mut StackValue>,
) -> i32 {
    let interface = unsafe { CStr::from_ptr(interface) };
    let interface = interface.to_string_lossy();
    let method_name = unsafe { CStr::from_ptr(method_name) };
    let method_name = method_name.to_string_lossy();

    let Some((interface, method_name)) = Runtime::get_interface_method_name(&interface, &method_name) else {
        return 2;
    };

    let mut return_value = StackValue::Blank;

    let result = if return_slot.is_some() {
        context.invoke_interface_extern(interface, method_name, Some(&mut return_value))
    } else {
        context.invoke_interface_extern(interface, method_name, None)
    };

    match return_slot {
        Some(slot) => {
            *slot = return_value.into();
        }
        _ => {}
    }

    if result {
        0
    } else {
        1
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rowan_set_exception(_context: &mut Runtime, _exception: Reference) {
    //context.set_exception(exception);
}

#[unsafe(no_mangle)]
pub extern "C" fn rowan_set_object_field(
    context: &mut BytecodeContext,
    object: Reference,
    field: *const c_char,
    value: FFIValue,
) -> i32 {
    let field = unsafe { CStr::from_ptr(field) };
    let field_name = field.to_string_lossy();

    let mut value: StackValue = value.into();

    let result = Runtime::set_object_field(context, object, &field_name, &mut value);

    match result {
        Some(_) => 0,
        _ => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rowan_get_object_field(
    context: &mut BytecodeContext,
    object: Reference,
    field: *const c_char,
    value: &mut FFIValue,
) -> i32 {
    let field = unsafe { CStr::from_ptr(field) };
    let field_name = field.to_string_lossy();

    let mut slot: StackValue = (*value).into();

    let result = Runtime::get_object_field(context, object, &field_name, &mut slot);

    *value = FFIValue::from(slot);

    match result {
        Some(_) => 0,
        _ => 1,
    }
}
