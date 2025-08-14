use rowan_shared::TypeTag;
use super::array8_init;
use crate::context::BytecodeContext;
use crate::runtime::core::{Array, VMClass, VMMember, VMMethod, VMVTable};
use crate::runtime::{Reference, Runtime, Symbol};
use crate::runtime::object::Object;

pub fn generate_string_class() -> VMClass {
    let main_vtable = VMVTable::new(
        "core::String",
        None,
        vec![
            VMMethod::new(
                "core::String::len",
                string_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
            ),
            VMMethod::new(
                "core::String::is-char-boundary",
                string_is_char_boundary as *const (),
                vec![TypeTag::U8, TypeTag::Object, TypeTag::U64]
            ),
            VMMethod::new(
                "core::String::as-bytes",
                string_as_bytes as *const (),
                vec![TypeTag::Object, TypeTag::Object]
            ),
        ]
    );
    let magic_vtable = VMVTable::new(
        "core::String",
        None,
        vec![
            VMMethod::new(
                "core::String::get-buffer",
                string_get_buffer as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let elements = vec![
    ];

    VMClass::new("core::String", "core::Object", vec![main_vtable, magic_vtable], elements, Vec::new(), Vec::new())
}

pub extern "C" fn string_len(_: &BytecodeContext, _: Reference) -> u64 {
    todo!("throw exception saying that this method needs to be implemented")
}

pub extern "C" fn string_is_char_boundary(_: &BytecodeContext, _: Reference, _: u64) -> u8 {
    todo!("throw exception saying that this method needs to be implemented")
}

pub extern "C" fn string_as_bytes(_: &BytecodeContext, _: Reference) -> Reference {
    todo!("throw exception saying that this method needs to be implemented")
}

pub extern "C" fn string_get_buffer(_: &BytecodeContext, _: Reference, _buf: &mut *const u8, _len: &mut u64)  {
    todo!("throw exception saying that this method needs to be implemented")
}

#[repr(C)]
pub struct InternedString {
    pub class: Symbol,
    pub parent_object: Reference,
    pub custom_drop: Option<fn(&mut Object)>,
    pub symbol: u64,
}

pub fn generate_interned_string_class() -> VMClass {
    let string_vtable = VMVTable::new(
        "core::InternedString",
        Some("core::String"),
        vec![
            VMMethod::new(
                "core::String::len",
                interned_string_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
            ),
            VMMethod::new(
                "core::String::is-char-boundary",
                interned_string_is_char_boundary as *const (),
                vec![TypeTag::U8, TypeTag::Object, TypeTag::U64]
            ),
            VMMethod::new(
                "core::String::as-bytes",
                interned_string_as_bytes as *const (),
                vec![TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let magic_vtable = VMVTable::new(
        "core::InternedString",
        Some("core::String"),
        vec![
            VMMethod::new(
                "core::String::get-buffer",
                interned_string_get_buffer as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let self_vtable = VMVTable::new(
        "core::InternedString",
        None,
        vec![
            VMMethod::new(
                "core::InternedString::to-buffer",
                interned_string_to_buffer as *const (),
                vec![TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let static_methods = vec![
        VMMethod::new(
            "core::InternedString::from-buffer",
            interned_string_from_buffer as *const (),
            vec![TypeTag::Object, TypeTag::Object]
        ),
    ];


    let elements = vec![
        VMMember::new("core::InternedString::symbol", TypeTag::U64),
    ];

    VMClass::new("core::InternedString", "core::String", vec![string_vtable, self_vtable, magic_vtable], elements, static_methods, Vec::new())
}

pub extern "C" fn interned_string_init(symbol: u64) -> *mut InternedString {
    let this = Runtime::new_object("core::InternedString");
    let this = this as *mut InternedString;
    let this = unsafe { this.as_mut().unwrap() };
    this.symbol = symbol;
    this
}

extern "C" fn interned_string_len(_: &BytecodeContext, this: *mut InternedString) -> u64 {
    let this = unsafe { this.as_ref().unwrap() };
    Runtime::get_string(this.symbol as Symbol).len() as u64
}

extern "C" fn interned_string_is_char_boundary(_: &BytecodeContext, this: *mut InternedString, index: u64) -> u8 {
    let this = unsafe { this.as_ref().unwrap() };
    Runtime::get_string(this.symbol as Symbol).is_char_boundary(index as usize) as u8
}

extern "C" fn interned_string_as_bytes(context: &mut BytecodeContext, this: *mut InternedString) -> Reference {
    let this = unsafe { this.as_ref().unwrap() };
    let string = Runtime::get_string(this.symbol as Symbol);
    let length = string.len() as u64;
    let pointer = string.as_bytes().as_ptr();
    
    let byte_array = Runtime::new_object("core::Array8");

    array8_init(context, byte_array, length);
    let array = byte_array;
    let array = array as *mut Array;
    let array = unsafe { array.as_ref().unwrap() };
    let array_pointer = array.buffer;

    unsafe {
        for i in 0..length {
            array_pointer.add(i as usize).write(*pointer.add(i as usize))
        }
    }
    byte_array
}

extern "C" fn interned_string_get_buffer(
    this: *mut InternedString, 
    buf: &mut *const u8, 
    len: &mut u64
) {
    let this = unsafe { this.as_ref().unwrap() };
    let string = Runtime::get_string(this.symbol as Symbol);
    *buf = string.as_bytes().as_ptr();
    *len = string.len() as u64;
    
}

extern "C" fn interned_string_to_buffer(_: &BytecodeContext, this: *mut InternedString) -> *mut StringBuffer {
    let this = unsafe { this.as_ref().unwrap() };
    let string = Runtime::get_string(this.symbol as Symbol);
    
    let buffer = Runtime::new_object("core::StringBuffer");
    let buffer = buffer as *mut StringBuffer;
    string_buffer_from_str(buffer, string);
    
    buffer
}

extern "C" fn interned_string_from_buffer(_: &BytecodeContext, buffer: *mut StringBuffer) -> *mut InternedString {
    let interned_string = Runtime::new_object("core::InternedString");
    let interned_string = interned_string as *mut InternedString;
    let interned_string = unsafe { interned_string.as_mut().unwrap() };
    let symbol = Runtime::intern_string(buffer);
    interned_string.symbol = symbol as u64;
    
    interned_string
}

#[repr(C)]
pub struct StringBuffer {
    pub class: Symbol,
    pub parent_object: Reference,
    pub custom_drop: Option<fn(&mut Object)>,
    pub length: u64,
    pub capacity: u64,
    pub buffer: *mut u8,
}

impl StringBuffer {

    pub fn initialize(&mut self, buffer: *mut u8, length: u64, capacity: u64) {
        self.buffer = buffer;
        self.length = length;
        self.capacity = capacity;
        self.custom_drop = Some(unsafe {
            std::mem::transmute::<_, fn(&mut Object)>(string_buffer_drop as *const ())
        });
    }

    fn resize_if_needed(&mut self, needed_size: usize) {
        use std::alloc::*;
        if needed_size <= (self.capacity - self.length) as usize {
            // We have enough space to just add the data needed
            return;
        }
        let new_capacity = self.capacity as f64 * 1.6;
        let mut new_capacity = new_capacity.ceil() as usize;
        if new_capacity == 0 {
            new_capacity = 1;
        }
        if new_capacity < needed_size {
            new_capacity += needed_size;
        }
        let layout = Layout::array::<u8>(new_capacity).expect("string layout is wrong or too big");
        let pointer = unsafe { alloc(layout) };
        if pointer.is_null() {
            eprintln!("Out of memory");
            handle_alloc_error(layout);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(self.buffer, pointer, self.length as usize);
        }
        if !self.buffer.is_null() {
            let old_layout = Layout::array::<u8>(self.capacity as usize).expect("string layout is wrong or too big");
            unsafe {
                dealloc(self.buffer, old_layout);
            }
        }
        self.buffer = pointer;
        self.capacity = new_capacity as u64;
    }

    fn push_char(&mut self, c: char) {
        let size = c.len_utf8();
        self.resize_if_needed(size);
        let mut char_buffer = [0; 4];
        let bytes = c.encode_utf8(&mut char_buffer).as_bytes();

        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.buffer.add(self.length as usize), bytes.len());
        }

        self.length += size as u64;
    }
}

pub fn generate_string_buffer_class() -> VMClass {
    let string_vtable = VMVTable::new(
        "core::StringBuffer",
        Some("core::String"),
        vec![
            VMMethod::new(
                "core::String::len",
                string_buffer_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
            ),
            VMMethod::new(
                "core::String::is-char-boundary",
                string_buffer_is_char_boundary as *const (),
                vec![TypeTag::U8, TypeTag::Object, TypeTag::U64]
            ),
            VMMethod::new(
                "core::String::as-bytes",
                string_buffer_as_bytes as *const (),
                vec![TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let magic_vtable = VMVTable::new(
        "core::StringBuffer",
        Some("core::String"),
        vec![
            VMMethod::new(
                "core::String::get-buffer",
                string_buffer_get_buffer as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let self_vtable = VMVTable::new(
        "core::StringBuffer",
        None,
        vec![
            VMMethod::new(
                "core::StringBuffer::push",
                string_buffer_push as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U32]
            ),
            VMMethod::new(
                "core::StringBuffer::intern",
                string_buffer_intern as *const (),
                vec![TypeTag::Object, TypeTag::Object]
            ),
        ]
    );

    let static_methods = vec![
        VMMethod::new(
            "core::StringBuffer::new",
            string_buffer_new as *const (),
            vec![TypeTag::Object]
        ),
        VMMethod::new(
            "core::StringBuffer::from-interned",
            string_buffer_from_interned as *const (),
            vec![TypeTag::Object, TypeTag::Object]
        ),
    ];


    let elements = vec![
        VMMember::new("core::StringBuffer::length", TypeTag::U64),
        VMMember::new("core::StringBuffer::capacity", TypeTag::U64),
        VMMember::new("core::StringBuffer::pointer", TypeTag::U64)
    ];

    VMClass::new("core::StringBuffer", "core::String", vec![string_vtable, self_vtable, magic_vtable], elements, static_methods, Vec::new())
}

extern "C" fn string_buffer_get_buffer(
    this: Reference,
    buf: &mut *const u8,
    len: &mut u64
) {
    let this = this as *mut StringBuffer;
    let this = unsafe { this.as_ref().unwrap() };
    *buf = this.buffer as *const u8;
    *len = this.length;
}

extern "C" fn string_buffer_intern(ctx: &mut BytecodeContext, this: *mut StringBuffer) -> *mut InternedString {
    interned_string_from_buffer(ctx, this)
}

extern "C" fn string_buffer_len(_: &mut BytecodeContext, this: *mut StringBuffer) -> u64 {
    let object = unsafe { this.as_ref().unwrap() };
    object.length
}

extern "C" fn string_buffer_from_interned(_: &mut BytecodeContext, interned_string: *mut InternedString) -> *mut StringBuffer {
    //println!("got: {this:p} {string_ref:p}");
    let interned_string = unsafe { interned_string.as_ref().unwrap() };
    let string = Runtime::get_string(interned_string.symbol as Symbol);
    let bytes = string.as_bytes();
    let object = Runtime::new_object("core::StringBuffer");
    let object = object as *mut StringBuffer;
    let object = unsafe { object.as_mut().unwrap() };
    object.initialize(std::ptr::null_mut(), 0, 0);
    object.resize_if_needed(bytes.len());
    object.length = bytes.len() as u64;
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), object.buffer, bytes.len())
    }
    
    object
}

extern "C" fn string_buffer_new(_: &mut BytecodeContext) -> *mut StringBuffer {
    let string_buffer = Runtime::new_object("core::StringBuffer");
    let string_buffer = string_buffer as *mut StringBuffer;
    let string_buffer = unsafe { string_buffer.as_mut().unwrap() };
    string_buffer.initialize(std::ptr::null_mut(), 0, 0);
    string_buffer
}


pub fn string_buffer_from_str(this: *mut StringBuffer, string: &str) {
    let object = unsafe { this.as_mut().unwrap() };
    object.initialize(std::ptr::null_mut(), 0, 0);
    let bytes = string.as_bytes();
    object.resize_if_needed(bytes.len());
    object.length = bytes.len() as u64;
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), object.buffer, bytes.len())
    }
}

pub fn string_buffer_drop(object: &mut StringBuffer) {
    use std::alloc::*;
    let capacity = object.capacity;
    let pointer = object.buffer;
    unsafe {
        let layout = Layout::array::<u8>(capacity as usize).expect("Wrong layout or too big");
        dealloc(pointer, layout);
    }
}

extern "C" fn string_buffer_is_char_boundary(_: &mut BytecodeContext, this: Reference, index: u64) -> u8 {
    let object = this;
    let object = object as *mut StringBuffer;
    let object = unsafe { object.as_ref().unwrap() };
    let length = object.length;
    let pointer = object.buffer;

    if index > length {
        return 0;
    }

    unsafe {
        if *pointer.add(index as usize) ^ 0b10000000 == 0b10000000 {
            1
        } else if (*pointer.add(index as usize) ^ 0b11100000) & 0b100000 == 0b100000 {
            1
        } else if (*pointer.add(index as usize) ^ 0b11110000) & 0b10000 == 0b10000 {
            1
        } else if (*pointer.add(index as usize) ^ 0b11111000) & 0b1000 == 0b1000 {
            1
        } else {
            0
        }
    }
}

extern "C" fn string_buffer_as_bytes(context: &mut BytecodeContext, this: Reference) -> Reference {
    let object = this;
    let object = object as *mut StringBuffer;
    let object = unsafe { object.as_ref().unwrap() };
    let length = object.length;
    let pointer = object.buffer;

    let byte_array = Runtime::new_object("Array8");

    array8_init(context, byte_array, length);
    let array = byte_array;
    let array = array as *mut Array;
    let array = unsafe { array.as_ref().unwrap() };
    let array_pointer = array.buffer;

    unsafe {
        for i in 0..length {
            array_pointer.add(i as usize).write(*pointer.add(i as usize))
        }
    }
    byte_array
}

extern "C" fn string_buffer_push(_: &mut BytecodeContext, this: Reference, character: u32) {
    let object = this;
    let object = object as *mut StringBuffer;
    let object = unsafe { object.as_mut().unwrap() };
    object.push_char(unsafe {
        char::from_u32_unchecked(character)
    })
}