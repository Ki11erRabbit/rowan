use std::cell::Ref;
use std::ops::Add;
use std::ptr::slice_from_raw_parts;
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
    pub static_methods: Vec<VMMethod>,
    pub static_members: Vec<VMMember>,
}

impl VMClass {
    pub fn new(
        name: &'static str,
        parents: Vec<&'static str>,
        vtables: Vec<VMVTable>,
        members: Vec<VMMember>,
        static_methods: Vec<VMMethod>,
        static_members: Vec<VMMember>,
    ) -> Self {
        VMClass {
            name,
            parents,
            vtables,
            members,
            static_methods,
            static_members,
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
                "downcast",
                object_downcast as *const (),
                vec![TypeTag::Object, TypeTag::Object, TypeTag::U64]
                ),
        ]
    );

    VMClass::new("Object", Vec::new(), vec![vtable], Vec::new(), Vec::new(), Vec::new())
}


extern "C" fn object_downcast(context: &mut Context, this: Reference, class_index: u64) -> Reference {
    let Some(object) = context.get_object(this) else {
        return 0;
    };
    let object = unsafe { object.as_mut().unwrap() };
    if object.class == class_index as Symbol {
        this
    } else {
        for obj in object.parent_objects.iter() {
            if object_downcast(context, *obj, class_index) != 0 {
                return this;
            }
        }
        0
    }
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

    VMClass::new("Printer", vec!["Object"], vec![vtable], Vec::new(), Vec::new(), Vec::new())
}


extern "C" fn printer_println_int(context: &mut Context, _: Reference, int: u64) {
    println!("{}", int);
}

extern "C" fn printer_println_float(context: &mut Context, _: Reference, float: f64) {
    println!("{}", float);
}

extern "C" fn printer_println(context: &mut Context, _: Reference, string: Reference) {
    let Some(object) = context.get_object(string) else {
        return
    };
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    let slice = unsafe { std::slice::from_raw_parts(pointer, length as usize) };
    let string = unsafe { std::str::from_utf8_unchecked(slice) };
    println!("{}", string);
}

macro_rules! array_downcast_contents {
    (object, $ty:ty, $context:ident, $this:ident, $class_symbol:ident) => {
        {
            let Some(object) = $context.get_object($this) else {
                return 0;
            };
            let object = unsafe { object.as_mut().unwrap() };
            let pointer = unsafe { object.get::<u64>(8) };
            let length = unsafe { object.get::<u64>(0) };
            let pointer = pointer as *mut u64;
            unsafe {
                for i in 0..length as usize {
                    if object_downcast($context, *pointer.add(i), $class_symbol) == 0 {
                        return 0;
                    }
                }
            }

            $this
        }
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
                            "downcast-contents",
                            [< array $variant _upcast_contents >] as *const (),
                            vec![TypeTag::U64, TypeTag::Object, TypeTag::U64]
                            ),
                    ]
                );

                let elements = vec![
                    VMMember::new("length", TypeTag::U64),
                    VMMember::new("pointer", TypeTag::U64)
                ];

                VMClass::new(std::stringify!($array_name), vec!["Object"], vec![vtable], elements, Vec::new(), Vec::new())
            }
        }
    };
}

macro_rules! array_create_init {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, length: u64) {
                use std::alloc::*;
                let Some(object) = context.get_object(this) else {
                    return
                };
                let object = object as *mut Array;
                let object = unsafe { object.as_mut().unwrap() };
                object.length = length;
                let layout = Layout::array::<$ty>(length as usize).expect("Wrong layout or too big");
                let pointer = unsafe { alloc(layout) };
                if pointer.is_null() {
                    eprintln!("Out of memory");
                    handle_alloc_error(layout);
                }
                unsafe {
                    for i in 0..(length as usize * std::mem::size_of::<$ty>()) {
                        std::ptr::write(pointer.add(i), 0);
                    }
                }
                object.buffer = pointer as *mut u8;

                object.custom_drop = Some([< array $variant _drop >]);
            }
        }
    };
}

macro_rules! array_create_get {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, index: u64) -> $ty {
                let Some(object) = context.get_object(this) else {
                    return 0 as $ty;
                };
                let object = object as *mut Array;
                let object = unsafe { object.as_ref().expect("array get") };
                let pointer = object.buffer;
                let length = object.length;
                let pointer = pointer as *mut $ty;
                if index >= length {
                    let exception = Context::new_object("IndexOutOfBounds");
                    out_of_bounds_init(context, exception, length, index);
                    context.set_exception(exception);
                    return 0 as $ty;
                }

                let index = index as usize;

                unsafe { *pointer.add(index) }
            }
        }
    };
}

macro_rules! array_create_set {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, index: u64, value: $ty) {
                let Some(object) = context.get_object(this) else {
                    return
                };
                let object = object as *mut Array;
                let object = unsafe { object.as_mut().expect("array set") };
                let pointer = object.buffer;
                let length = object.length;
                let pointer = pointer as *mut $ty;
                if index >= length {
                    let exception = Context::new_object("IndexOutOfBounds");
                    out_of_bounds_init(context, exception, length, index);
                    context.set_exception(exception);
                    return;
                }
                unsafe { *pointer.add(index as usize) = value }
            }
        }
    };
}

macro_rules! array_create_downcast {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(context: &mut Context, this: Reference, class_symbol: u64) -> Reference {
                array_downcast_contents!($variant, $ty, context, this, class_symbol)
            }
        }
    };
}

macro_rules! array_create_drop {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub fn $fn_name(object: &mut Object) {
                use std::alloc::*;
                let object = unsafe { std::mem::transmute::<_, &mut Array>(object) };
                let pointer = object.buffer;
                let length = object.length;
                let pointer = pointer as *mut u8;
                unsafe {
                    let layout = Layout::array::<$ty>(length as usize).expect("Wrong layout or too big");
                    dealloc(pointer, layout);
                }
            }
        }
    };
}

macro_rules! create_array_class {
    ($variant:expr, $ty:ty) => {

        paste!{
        array_create_class!($variant, [< generate_array $variant _class >], [<Array $variant >]);
        }

        paste!{
        array_create_init!($variant, [< array $variant _init >], $ty);
        }

        paste!{
        array_create_get!($variant, [< array $variant _get >], $ty);
        }

        paste!{
        array_create_set!($variant, [< array $variant _set >], $ty);
        }

        paste!{
        array_create_downcast!($variant, [< array $variant _upcast_contents >], $ty);
        }

        paste!{
        array_create_drop!($variant, [< array $variant _drop >], $ty);
        }

    };
}

#[repr(C)]
pub struct Array {
    pub class: Symbol,
    pub parent_objects: Box<[Reference]>,
    pub custom_drop: Option<fn(&mut Object)>,
    pub length: u64,
    pub buffer: *mut u8,
}

create_array_class!(8, u8);
create_array_class!(16, u16);
create_array_class!(32, u32);
create_array_class!(64, u64);
create_array_class!(object, u64);
create_array_class!(f32, f32);
create_array_class!(f64, f64);

extern "C" fn array_len(context: &mut Context, this: Reference) -> u64 {
    let Some(object) = context.get_object(this) else {
        return 0
    };
    let object = object as *mut Array;
    let object = unsafe { object.as_ref().unwrap() };
    let length = object.length;
    length
}

#[repr(C)]
struct Exception {
    pub class: Symbol,
    pub parent_objects: Box<[Reference]>,
    pub custom_drop: Option<fn(&mut Object)>,
    pub message: Reference,
    pub stack_length: u64,
    pub stack_capacity: u64,
    pub stack_pointer: *mut Reference,
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
                "print-stack-trace",
                exception_print_stack_trace as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
        ]
    );

    let elements = vec![
        VMMember::new("message", TypeTag::Object),
        VMMember::new("stack-length", TypeTag::U64),
        VMMember::new("stack-capacity", TypeTag::U64),
        VMMember::new("stack-trace-pointer", TypeTag::U64),
    ];

    VMClass::new("Exception", vec!["Object"], vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn exception_init(context: &Context, this: Reference, message: Reference) {
    use std::alloc::*;
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = object as *mut Exception;
    let object = unsafe { object.as_mut().unwrap() };
    object.message = message;
    object.stack_length = 0;
    object.stack_capacity = 4;
    let layout = Layout::array::<u64>(4).expect("stack-trace layout is wrong or too big");
    let pointer = unsafe { alloc(layout) };
    if pointer.is_null() {
        eprintln!("Out of memory");
        handle_alloc_error(layout);
    }
    unsafe {
        std::ptr::copy_nonoverlapping::<u64>([0,0,0,0].as_ptr(), pointer as *mut u64, 4)
    }
    object.stack_pointer = pointer as *mut Reference;
}

pub extern "C" fn exception_fill_in_stack_trace(context: &mut Context, this: Reference) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = object as *mut Exception;
    let object = unsafe { object.as_mut().unwrap() };
    let length = object.stack_length;
    let mut capacity = object.stack_capacity;
    let pointer = object.stack_pointer;

    let (pointer, capacity) = if length == capacity {
        use std::alloc::*;
        let layout = Layout::array::<u64>(capacity as usize * 2).expect("stack-trace layout is wrong or too big");
        let new_pointer = unsafe { alloc(layout) };
        if new_pointer.is_null() {
            eprintln!("Out of memory");
            handle_alloc_error(layout);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(pointer, new_pointer as *mut Reference, capacity as usize);
        }
        let layout = Layout::array::<u64>(capacity as usize).expect("stack-trace layout is wrong or too big");
        unsafe { dealloc(pointer as *mut u8, layout) };
        capacity = capacity * 2;
        (new_pointer as *mut u64, capacity)
    } else {
        (pointer as *mut u64, capacity)
    };
    object.stack_capacity = capacity;
    object.stack_pointer = pointer;

    let backtrace = Context::new_object("Backtrace");

    let method_name = context.get_current_method();

    backtrace_init(context, backtrace, method_name,0, 0);

    unsafe {
        pointer.add(length as usize).write(backtrace);
    }
    object.stack_length = length + 1;
}

pub extern "C" fn exception_print_stack_trace(context: &mut Context, this: Reference) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = object as *mut Exception;
    let object = unsafe { object.as_mut().unwrap() };
    let length = object.stack_length;
    let pointer = object.stack_pointer;

    unsafe {
        let slice = slice_from_raw_parts(pointer, length as usize).as_ref().unwrap();

        for backtrace in slice {
            backtrace_display(context, *backtrace);
        }
    }
}

#[repr(C)]
struct Backtrace {
    pub class: Symbol,
    pub parent_objects: Box<[Reference]>,
    pub custom_drop: Option<fn(&mut Object)>,
    pub function_name: Reference,
    pub line_number: u64,
    pub column_number: u64,
}

pub fn generate_backtrace_class() -> VMClass {
    let vtable = VMVTable::new(
        "Backtrace",
        None,
        vec![
            VMMethod::new(
                "init",
                backtrace_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object, TypeTag::U64, TypeTag::U64]
            ),
            VMMethod::new(
                "display",
                backtrace_display as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
        ]
    );

    let elements = vec![
        VMMember::new("function-name", TypeTag::Object),
        VMMember::new("line-number", TypeTag::Object),
        VMMember::new("column-number", TypeTag::Object),
    ];

    VMClass::new("Backtrace", vec!["Object"], vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn backtrace_init(context: &mut Context, this: Reference, function_name: Reference, line: u64, column: u64) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = object as *mut Backtrace;
    let object = unsafe { object.as_mut().unwrap() };
    object.function_name = function_name;
    object.line_number = line;
    object.column_number = column;
}

extern "C" fn backtrace_display(context: &mut Context, this: Reference) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = object as *mut Backtrace;
    let object = unsafe { object.as_ref().unwrap() };
    let function_name = object.function_name;
    let line = object.line_number;
    let column = object.column_number;

    let Some(string) = context.get_object(function_name) else {
        return
    };
    let string = unsafe { string.as_ref().unwrap() };
    let string_length = unsafe { string.get::<u64>(8) };
    let string_pointer = unsafe { string.get::<*const u8>(16) };
    let string_slice = slice_from_raw_parts(string_pointer as *const u8, string_length as usize);
    let str = unsafe { std::str::from_utf8_unchecked(string_slice.as_ref().unwrap()) };

    println!("{} {}:{}", str, line, column);
}

pub fn generate_index_out_of_bounds_class() -> VMClass {
    let vtable = VMVTable::new(
        "IndexOutOfBounds",
        None,
        vec![
            VMMethod::new(
                "init",
                out_of_bounds_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64, TypeTag::U64]
            ),
        ]
    );

    let elements = vec![
    ];

    VMClass::new("IndexOutOfBounds", vec!["Exception"], vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn out_of_bounds_init(context: &mut Context, this: Reference, bounds: u64, index: u64) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = unsafe { object.as_mut().unwrap() };

    let message = Context::new_object("String"); // String Class Symbol

    string_from_str(context, message, format!("Index was {index} but length was {bounds}"));

    let base_exception = object.parent_objects[0];
    exception_init(context, base_exception, message);
}

pub fn generate_null_pointer_class() -> VMClass {
    let vtable = VMVTable::new(
        "NullPointerException",
        None,
        vec![
            VMMethod::new(
                "init",
                null_pointer_init as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
        ]
    );

    let elements = vec![
    ];

    VMClass::new("NullPointerException", vec!["Exception"], vec![vtable], elements, Vec::new(), Vec::new())
}

pub extern "C" fn null_pointer_init(context: &Context, this: Reference) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = unsafe { object.as_mut().unwrap() };

    let message = Context::new_object("String"); // String Class Symbol

    string_from_str(context, message, String::from("NullPointerException"));

    let base_exception = object.parent_objects[0];
    exception_init(context, base_exception, message);
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

    VMClass::new("String", vec!["Object"], vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn string_len(context: &mut Context, this: Reference) -> u64 {
    let Some(object) = context.get_object(this) else {
        return 0
    };
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    length
}

extern "C" fn string_load_str(context: &mut Context, this: Reference, string_ref: Reference) {
    use std::alloc::*;
    let string = Context::get_string(string_ref as Symbol);
    let bytes = string.as_bytes();
    let Some(object) = context.get_object(this) else {
        return
    };
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
    let Some(object) = context.get_object(this) else {
        return
    };
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

pub fn string_from_str(context: &Context, this: Reference, string: String) {
    let Some(object) = context.get_object(this) else {
        return
    };
    let object = unsafe { object.as_mut().unwrap() };
    unsafe { object.set::<u64>(0, string.len() as u64) };
    unsafe { object.set::<u64>(8, string.len() as u64) };
    let mut string_box = string.into_boxed_str();
    let string_pointer = string_box.as_mut_ptr();
    let _ = Box::into_raw(string_box);
    unsafe { object.set::<*mut u8>(16, string_pointer) };
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
    let Some(object) = context.get_object(this) else {
        return 0
    };
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(8) };
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
    let Some(object) = context.get_object(this) else {
        return 0
    };
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(8) };
    let pointer = unsafe { object.get::<*mut u8>(16) };

    let byte_array = Context::new_object("Array8");

    array8_init(context, byte_array, length);
    let Some(array) = context.get_object(byte_array) else {
        unreachable!("array just created is invalid")
    };
    let array = unsafe { array.as_ref().unwrap() };
    let array_pointer = unsafe { array.get::<*mut u8>(8) };

    unsafe {
        for i in 0..length {
            array_pointer.add(i as usize).write(*pointer.add(i as usize))
        }
    }
    byte_array
}