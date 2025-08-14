use std::ffi::{c_char, CStr};
use crate::context::{BytecodeContext, StackValue};
use crate::runtime::{Runtime, Reference};
use crate::runtime::core::{string_buffer_from_str, InternedString, StringBuffer};

/// This function constructs an object from a given class name from a CStr.
/// The CStr should be valid utf-8 as to prevent misses.
/// Returns a valid reference to an object
#[unsafe(no_mangle)]
pub extern "C" fn rowan_create_object(class_name: *const c_char) -> Reference {
    let class_name = unsafe { CStr::from_ptr(class_name) };
    let name = class_name.to_string_lossy();
    Runtime::new_object(name.as_ref())
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

    let result = context.invoke_virtual_extern(class, method_name, return_slot);

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
    return_slot: Option<&mut StackValue>,
) -> i32 {
    let class = unsafe { CStr::from_ptr(class) };
    let class = class.to_string_lossy();
    let method_name = unsafe { CStr::from_ptr(method_name) };
    let method_name = method_name.to_string_lossy();

    let Some((class_name, method_name)) = Runtime::get_static_method_name(&class, &method_name) else {
        return 2;
    };

    let result = context.invoke_static_extern(class_name, method_name, return_slot);

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
