use super::{object::Object, Context, Reference, Symbol};
use rowan_shared::TypeTag;


/// This represents a class in the Virtual Machine.
pub struct VMClass {
    pub name: &'static str,
    pub parents: Vec<&'static str>,
    pub vtables: Vec<VMVTable>,
    pub members: Vec<VMMember>,
    pub signals: Vec<VMSignal>,
}

impl VMClass {
    pub fn new(
        name: &'static str,
        parents: Vec<&'static str>,
        vtables: Vec<VMVTable>,
        members: Vec<VMMember>,
        signals: Vec<VMSignal>,
    ) -> Self {
        VMClass {
            name,
            parents,
            vtables,
            members,
            signals,
        }
    }
}


pub struct VMVTable {
    pub class: &'static str,
    pub source_class: Option<&'static str>,
    pub methods: Vec<VMMethod>
}

impl VMVTable {
    pub fn new(class: &'static str, source_class: Option<&'static str>, methods: Vec<VMMethod>) -> Self {
        VMVTable {
            class,
            source_class,
            methods
        }
    }
}

pub struct VMMethod {
    pub name: &'static str,
    pub fn_pointer: *const (),
    pub signature: Vec<TypeTag>,
}

impl VMMethod {
    pub fn new(name: &'static str, fn_pointer: *const (), signature: Vec<TypeTag>) -> Self {
        VMMethod {
            name,
            fn_pointer,
            signature
        }
    }
}

pub struct VMMember {
    pub name: &'static str,
    pub ty: TypeTag,
}

impl VMMember {
    pub fn new(name: &'static str, ty: TypeTag) -> Self {
        VMMember {
            name,
            ty
        }
    }
}

pub struct VMSignal {
    pub name: &'static str,
    pub is_static: bool,
    pub arguments: Vec<TypeTag>
}

impl VMSignal {
    pub fn new(name: &'static str, is_static: bool, arguments: Vec<TypeTag>) -> Self {
        VMSignal {
            name,
            is_static,
            arguments
        }
    }
}



pub fn generate_object_class() -> VMClass {
    let vtable = VMVTable::new(
        "Object",
        None,
        vec![
            VMMethod::new(
                "tick",
                object_tick as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::F64]
                ),
            VMMethod::new(
                "ready",
                object_ready as *const (),
                vec![TypeTag::Void, TypeTag::Object]
                ),
            VMMethod::new(
                "upcast",
                object_upcast as *const (),
                vec![TypeTag::Object, TypeTag::Object]
                ),
            VMMethod::new(
                "get-child",
                object_get_child as *const (),
                vec![TypeTag::Object, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "remove-child",
                object_remove_child as *const (),
                vec![TypeTag::Object, TypeTag::Object, TypeTag::Object]
                ),
        ]
    );

    VMClass::new("Object", Vec::new(), vec![vtable], Vec::new(), Vec::new())
}


extern "C" fn object_tick(_: Reference, _: f64) {
    
}

extern "C" fn object_ready(_: Reference) {
    
}

// Possibly change this to take an additional parameter which is a class index
extern "C" fn object_upcast(this: Reference) -> Reference {
    this 
}


extern "C" fn object_get_child(this: Reference, nth: u64) -> Reference {
    todo!("get a context and look up the nth child of that object")
}

extern "C" fn object_remove_child(this: Reference, reference: Reference) -> Reference {
    todo!("get a context and find the child of that object that matches reference")
}

pub fn generate_printer_class() -> VMClass {
    let vtable = VMVTable::new(
        "Printer",
        None,
        vec![
            VMMethod::new(
                "println-int",
                printer_println_int as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "println-float",
                printer_println_float as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::F64]
                ),
            VMMethod::new(
                "println",
                printer_println as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
                ),
        ]
    );

    VMClass::new("Printer", vec!["Object"], vec![vtable], Vec::new(), Vec::new())
}


extern "C" fn printer_println_int(_: Reference, int: u64) {
    println!("{}", int);
}

extern "C" fn printer_println_float(_: Reference, float: f64) {
    println!("{}", float);
}

extern "C" fn printer_println(_: Reference, string: Reference) {
    let context = Context::new();
    let object = context.get_object(string);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    let slice = unsafe { std::slice::from_raw_parts(pointer, length as usize) };
    let string = unsafe { std::str::from_utf8_unchecked(slice) };
    println!("{}", string);
}

pub fn generate_array_8_class() -> VMClass {
    let vtable = VMVTable::new(
        "Array8",
        None,
        vec![
            VMMethod::new(
                "init",
                array8_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("Array8", vec!["Object"], vec![vtable], elements, Vec::new())
}

extern "C" fn array8_init(this: Reference, length: u64) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, length) };
    let layout = Layout::array::<u8>(length as usize).expect("Wrong layout or too big");
    let pointer = unsafe { alloc(layout) };
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        for i in 0..length {
            std::ptr::write(pointer.add(i as usize), 0);
        }
    }
    unsafe { object.set::<u64>(8, pointer as u64) };
    
}

pub extern "C" fn array8_get(this: Reference, index: u64) -> u8 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u8;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn array8_set(this: Reference, index: u64, value: u8) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u8;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}

pub fn generate_array_16_class() -> VMClass {
    let vtable = VMVTable::new(
        "Array16",
        None,
        vec![
            VMMethod::new(
                "init",
                array16_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("Array16", vec!["Object"], vec![vtable], elements, Vec::new())
}

extern "C" fn array16_init(this: Reference, length: u64) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, length) };
    let layout = Layout::array::<u16>(length as usize).expect("Wrong layout or too big");
    let pointer = unsafe { alloc(layout) } as *mut u16;
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        for i in 0..length {
            std::ptr::write(pointer.add(i as usize), 0);
        }
    }
    unsafe { object.set::<u64>(8, pointer as u64) };
    
}

pub extern "C" fn array16_get(this: Reference, index: u64) -> u16 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u16;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn array16_set(this: Reference, index: u64, value: u16) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u16;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}

pub fn generate_array_32_class() -> VMClass {
    let vtable = VMVTable::new(
        "Array32",
        None,
        vec![
            VMMethod::new(
                "init",
                array32_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("Array32", vec!["Object"], vec![vtable], elements, Vec::new())
}

extern "C" fn array32_init(this: Reference, length: u64) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, length) };
    let layout = Layout::array::<u32>(length as usize).expect("Wrong layout or too big");
    let pointer = unsafe { alloc(layout) } as *mut u32;
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        for i in 0..length {
            std::ptr::write(pointer.add(i as usize), 0);
        }
    }
    unsafe { object.set::<u64>(8, pointer as u64) };
    
}

pub extern "C" fn array32_get(this: Reference, index: u64) -> u32 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u32;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn array32_set(this: Reference, index: u64, value: u32) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u32;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}

pub fn generate_array_64_class() -> VMClass {
    let vtable = VMVTable::new(
        "Array64",
        None,
        vec![
            VMMethod::new(
                "init",
                array64_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("Array64", vec!["Object"], vec![vtable], elements, Vec::new())
}

pub extern "C" fn array64_init(this: Reference, length: u64) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, length) };
    let layout = Layout::array::<u64>(length as usize).expect("Wrong layout or too big");
    let pointer = unsafe { alloc(layout) } as *mut u64;
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        for i in 0..length {
            std::ptr::write(pointer.add(i as usize), 0);
        }
    }
    unsafe { object.set::<u64>(8, pointer as u64) };
    
}

pub extern "C" fn array64_get(this: Reference, index: u64) -> u64 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u64;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn array64_set(this: Reference, index: u64, value: u64) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u64;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}

pub fn generate_array_object_class() -> VMClass {
    let vtable = VMVTable::new(
        "ArrayObject",
        None,
        vec![
            VMMethod::new(
                "init",
                array64_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("ArrayObject", vec!["Object"], vec![vtable], elements, Vec::new())
}


pub extern "C" fn array_object_get(this: Reference, index: u64) -> Reference {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u64;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn array_object_set(this: Reference, index: u64, value: Reference) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut u64;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}


pub fn generate_array_f32_class() -> VMClass {
    let vtable = VMVTable::new(
        "Array32",
        None,
        vec![
            VMMethod::new(
                "init",
                arrayf32_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("Arrayf32", vec!["Object"], vec![vtable], elements, Vec::new())
}

pub extern "C" fn arrayf32_init(this: Reference, length: u64) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, length) };
    let layout = Layout::array::<f32>(length as usize).expect("Wrong layout or too big");
    let pointer = unsafe { alloc(layout) } as *mut f32;
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        for i in 0..length {
            std::ptr::write(pointer.add(i as usize), 0.0);
        }
    }
    unsafe { object.set::<u64>(8, pointer as u64) };
    
}

pub extern "C" fn arrayf32_get(this: Reference, index: u64) -> f32 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut f32;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn arrayf32_set(this: Reference, index: u64, value: f32) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut f32;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}

pub fn generate_array_f64_class() -> VMClass {
    let vtable = VMVTable::new(
        "Arrayf64",
        None,
        vec![
            VMMethod::new(
                "init",
                arrayf64_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "len",
                array_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("Arrayf64", vec!["Object"], vec![vtable], elements, Vec::new())
}

pub extern "C" fn arrayf64_init(this: Reference, length: u64) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, length) };
    let layout = Layout::array::<f64>(length as usize).expect("Wrong layout or too big");
    let pointer = unsafe { alloc(layout) } as *mut f64;
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        for i in 0..length {
            std::ptr::write(pointer.add(i as usize), 0.0);
        }
    }
    unsafe { object.set::<u64>(8, pointer as u64) };
    
}

pub extern "C" fn arrayf64_get(this: Reference, index: u64) -> f64 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut f64;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) }
}

pub extern "C" fn arrayf64_set(this: Reference, index: u64, value: f64) {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    let pointer = unsafe { object.get::<u64>(8) };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = pointer as *mut f64;
    if index >= length {
        todo!("provide way to handle out of bounds")
    }
    unsafe { *pointer.add(index as usize) = value }
}


extern "C" fn array_len(this: Reference) -> u64 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    length
}

pub fn array_8_drop(object: &mut Object) {
    use std::alloc::*;
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    unsafe {
        let layout = Layout::array::<u8>(length as usize).expect("Wrong layout or too big");
        dealloc(pointer, layout);
    }
}

pub fn array_16_drop(object: &mut Object) {
    use std::alloc::*;
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    unsafe {
        let layout = Layout::array::<u16>(length as usize).expect("Wrong layout or too big");
        dealloc(pointer, layout);
    }
}

pub fn array_32_drop(object: &mut Object) {
    use std::alloc::*;
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    unsafe {
        let layout = Layout::array::<u32>(length as usize).expect("Wrong layout or too big");
        dealloc(pointer, layout);
    }
}

pub fn array_64_drop(object: &mut Object) {
    use std::alloc::*;
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    unsafe {
        let layout = Layout::array::<u64>(length as usize).expect("Wrong layout or too big");
        dealloc(pointer, layout);
    }
}


pub fn generate_string_class() -> VMClass {
    let vtable = VMVTable::new(
        "String",
        None,
        vec![
            VMMethod::new(
                "load-str",
                string_load_str as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "init",
                string_init as *const (),
                vec![TypeTag::Void, TypeTag::Object]
                ),
            VMMethod::new(
                "len",
                string_len as *const (),
                vec![TypeTag::U64, TypeTag::Object]
                ),
        ]
    );

    let elements = vec![
        VMMember::new("length", TypeTag::U64),
        VMMember::new("capacity", TypeTag::U64),
        VMMember::new("pointer", TypeTag::U64)
    ];

    VMClass::new("String", vec!["Object"], vec![vtable], elements, Vec::new())
}

extern "C" fn string_len(this: Reference) -> u64 {
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    length
}

extern "C" fn string_load_str(this: Reference, string_ref: Reference) {
    use std::alloc::*;
    let context = Context::new();
    let string = context.get_string(string_ref as Symbol);
    let bytes = string.as_bytes();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, bytes.len() as u64) };
    unsafe { object.set::<u64>(8, bytes.len() as u64) };
    let layout = Layout::array::<u8>(bytes.len()).expect("string layout is wrong or too big");
    let pointer = unsafe { alloc(layout) };
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len())
    }
    unsafe { object.set::<u64>(16, pointer as u64) };
}

extern "C" fn string_init(this: Reference) {
    use std::alloc::*;
    let context = Context::new();
    let object = context.get_object(this);
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, 0) };
    unsafe { object.set::<u64>(8, 4) };
    let layout = Layout::array::<u8>(4).expect("string layout is wrong or too big");
    let pointer = unsafe { alloc(layout) };
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(b"\0\0\0\0".as_ptr(), pointer, 4)
    }
    unsafe { object.set::<u64>(16, pointer as u64) };
}

pub fn string_drop(object: &mut Object) {
    use std::alloc::*;
    let capacity = unsafe { object.get::<u64>(8) };
    let pointer = unsafe { object.get::<u64>(16) };
    let pointer = pointer as *mut u8;
    unsafe {
        let layout = Layout::array::<u8>(capacity as usize).expect("Wrong layout or too big");
        dealloc(pointer, layout);
    }
}
