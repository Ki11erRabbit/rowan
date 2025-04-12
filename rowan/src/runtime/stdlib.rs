use std::cell::Ref;
use std::ops::Add;
use paste::paste;
use quote::format_ident;
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
                vec![TypeTag::Object, TypeTag::Object, TypeTag::U64]
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
            VMMethod::new(
                "add-child",
                object_add_child as *const (),
                vec![TypeTag::U64, TypeTag::Object, TypeTag::Object]
            ),
            VMMethod::new(
                "add-child-at",
                object_add_child_at as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object, TypeTag::U64]
            ),
            VMMethod::new(
                "child-died",
                object_child_died as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64, TypeTag::Object]
            ),
        ]
    );

    VMClass::new("Object", Vec::new(), vec![vtable], Vec::new(), Vec::new())
}


extern "C" fn object_tick(context: &mut Context, _: Reference, _: f64) {
    
}

extern "C" fn object_ready(context: &mut Context, _: Reference) {
    
}

// Possibly change this to take an additional parameter which is a class index
extern "C" fn object_upcast(context: &mut Context, this: Reference, class_index: u64) -> Reference {
    this 
}


extern "C" fn object_get_child(context: &mut Context, this: Reference, nth: u64) -> Reference {
    todo!("get a context and look up the nth child of that object")
}

extern "C" fn object_remove_child(context: &mut Context, this: Reference, reference: Reference) -> Reference {
    todo!("get a context and find the child of that object that matches reference")
}

extern "C" fn object_add_child(context: &mut Context, this: Reference, child: Reference) -> u64 {
    todo!("get a context and call ready on the object, then attach it to this object, returning the index of the child")
}
extern "C" fn object_add_child_at(context: &mut Context, this: Reference, child: Reference, nth: u64) {
    todo!("get a context and call ready on the object, then attach it to the this object")
}

extern "C" fn object_child_died(context: &mut Context, this: Reference, child_index: Reference, exception: Reference) -> Reference {
    todo!("take in a context parameter and propagate exception")
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


extern "C" fn printer_println_int(context: &mut Context, _: Reference, int: u64) {
    println!("{}", int);
}

extern "C" fn printer_println_float(context: &mut Context, _: Reference, float: f64) {
    println!("{}", float);
}

extern "C" fn printer_println(context: &mut Context, _: Reference, string: Reference) {
    let object = context.get_object(string);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    let slice = unsafe { std::slice::from_raw_parts(pointer, length as usize) };
    let string = unsafe { std::str::from_utf8_unchecked(slice) };
    println!("{}", string);
}

macro_rules! array_upcast_contents {
    (object, $ty:ty, $context:ident, $this:ident, $class_symbol:ident) => {
        let object = $context.get_object($this);
        let object = unsafe { object.as_mut().unwrap() };
        let pointer = unsafe { object.get::<u64>(8) };
        let length = unsafe { object.get::<u64>(0) };
        let pointer = pointer as *mut u64;
        unsafe {
            for i in 0..length as usize {
                if object_upcast(*pointer.add(i), $class_symbol) == 0 {
                    return 0;
                }
            }
        }

        this
    };
    ($variant:expr, $ty:ty, $context:ident, $this:ident, $class_symbol:ident) => {
        todo!("create unable to upcast exception and set it in context")
    };
}

macro_rules! array_create_class {
    ($variant:expr, $fn_name:ident, $array_name:ident) => {
        paste! {
            pub fn $fn_name() -> VMClass {
                let vtable = VMVTable::new(
                    std::stringify!($array_name),
                    None,
                    vec![
                        VMMethod::new(
                            "init",
                            [< array $variant _init>] as *const (),
                            vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                            ),
                        VMMethod::new(
                            "len",
                            array_len as *const (),
                            vec![TypeTag::U64, TypeTag::Object]
                            ),
                        VMMethod::new(
                            "upcast-contents",
                            [< array $variant _upcast_contents >] as *const (),
                            vec![TypeTag::U64, TypeTag::Object, TypeTag::U64]
                            ),
                    ]
                );

                let elements = vec![
                    VMMember::new("length", TypeTag::U64),
                    VMMember::new("pointer", TypeTag::U64)
                ];

                VMClass::new(std::stringify!($array_name), vec!["Object"], vec![vtable], elements, Vec::new())
            }
        }
    };
}

macro_rules! array_create_init {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, length: u64) {
                use std::alloc::*;
                let object = context.get_object(this);
                let object = unsafe { object.as_mut().unwrap() };
                unsafe { object.set::<u64>(0, length) };
                let layout = Layout::array::<$ty>(length as usize).expect("Wrong layout or too big");
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

                object.custom_drop = Some([< array $variant _drop >]);
            }
        }
    };
}

macro_rules! array_create_get {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, index: u64) -> $ty {
                let object = context.get_object(this);
                let object = unsafe { object.as_ref().unwrap() };
                let pointer = unsafe { object.get::<u64>(8) };
                let length = unsafe { object.get::<u64>(0) };
                let pointer = pointer as *mut $ty;
                if index >= length {
                    todo!("provide way to handle out of bounds")
                }
                unsafe { *pointer.add(index as usize) }
            }
        }
    };
}

macro_rules! array_create_set {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, index: u64, value: $ty) {
                let object = context.get_object(this);
                let object = unsafe { object.as_mut().unwrap() };
                let pointer = unsafe { object.get::<u64>(8) };
                let length = unsafe { object.get::<u64>(0) };
                let pointer = pointer as *mut $ty;
                if index >= length {
                    todo!("provide way to handle out of bounds")
                }
                unsafe { *pointer.add(index as usize) = value }
            }
        }
    };
}

macro_rules! array_create_upcast {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, class_symbol: u64) -> Reference {
                array_upcast_contents!($variant, $ty, context, this, class_symbol)
            }
        }
    };
}

macro_rules! array_create_drop {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub fn $fn_name(object: &mut Object) {
                use std::alloc::*;
                let length = unsafe { object.get::<u64>(0) };
                let pointer = unsafe { object.get::<u64>(8) };
                let pointer = pointer as *mut u8;
                unsafe {
                    let layout = Layout::array::<$ty>(length as usize).expect("Wrong layout or too big");
                    dealloc(pointer, layout);
                }
            }
        }
    };
}
paste! { [<generate_array 8 _class>]}

macro_rules! create_array_class {
    ($variant:expr, $ty:ty) => {

        paste!{
        array_create_class!($variant, [<generate_array $variant _class >], [<Array $variant >]);
        }

        paste!{
        array_create_init!($variant, [< array $variant _init >], $ty)
        }

        array_create_get!($variant, [< array $variant _get >], $ty)

        array_create_set!($variant, [< array $variant _set >], $ty)

        array_create_upcast!($variant, [< array $variant _upcast_contents >], $ty)

        array_create_drop!($variant, [< array $variant _drop >], $ty)

    };
}

create_array_class!(8, u8);
create_array_class!(16, u16);
create_array_class!(32, u32);
create_array_class!(64, u64);
create_array_class!(object, u64);
create_array_class!(f32, f32);
create_array_class!(f64, f64);

extern "C" fn array_len(context: &mut Context, this: Reference) -> u64 {
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    length
}

pub fn generate_exception_class() -> VMClass {
    let vtable = VMVTable::new(
        "Exception",
        None,
        vec![
            VMMethod::new(
                "init",
                exception_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
            ),
            VMMethod::new(
                "fill-in-stack-trace",
                exception_fill_in_stack_trace as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
            VMMethod::new(
                "fill-in-stack-trace",
                exception_fill_in_stack_trace as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
            VMMethod::new(
                "print-stack-trace",
                exception_print_stack_trace as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
        ]
    );

    let elements = vec![
        VMMember::new("message", TypeTag::Object),
        VMMember::new("stack-trace", TypeTag::Object),
    ];

    VMClass::new("Exception", vec!["Object"], vec![vtable], elements, Vec::new())
}

extern "C" fn exception_init(context: &mut Context, this: Reference, message: Reference) {

}

extern "C" fn exception_fill_in_stack_trace(context: &mut Context, this: Reference) {
    
}

extern "C" fn exception_print_stack_trace(context: &mut Context, this: Reference) {}

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
            VMMethod::new(
                "is-char-boundary",
                string_is_char_boundary as *const (),
                vec![TypeTag::U8, TypeTag::Object, TypeTag::U64]
            ),
            VMMethod::new(
                "as-bytes",
                string_is_char_boundary as *const (),
                vec![TypeTag::Object, TypeTag::Object]
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

extern "C" fn string_len(context: &mut Context, this: Reference) -> u64 {
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    length
}

extern "C" fn string_load_str(context: &mut Context, this: Reference, string_ref: Reference) {
    use std::alloc::*;
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

extern "C" fn string_init(context: &mut Context, this: Reference) {
    use std::alloc::*;
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

extern "C" fn string_is_char_boundary(context: &mut Context, this: Reference, index: u64) -> u8 {
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(4) };
    let pointer = unsafe { object.get::<*mut u8>(16) };

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

extern "C" fn string_as_bytes(context: &mut Context, this: Reference) -> Reference {
    let object = context.get_object(this);
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(4) };
    let pointer = unsafe { object.get::<*mut u8>(16) };

    let byte_array = crate::runtime::new_object(8); // Array8 Class Symbol

    /*array8_init(byte_array, length);
    let array = context.get_object(byte_array);
    let array = unsafe { array.as_ref().unwrap() };
    let array_pointer = unsafe { array.get::<*mut u8>(8) };

    unsafe {
        for i in 0..length {
            array_pointer.add(i as usize).write(*pointer.add(i as usize))
        }
    }*/
    byte_array
}