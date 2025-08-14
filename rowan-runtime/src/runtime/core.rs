mod strings;
mod boxes;

pub use strings::*;
pub use boxes::*;

use std::ptr::slice_from_raw_parts;
use paste::paste;
use super::{object::Object, Runtime, Reference, Symbol};
use rowan_shared::TypeTag;
use crate::context::BytecodeContext;

/// This represents a class in the Virtual Machine.
pub struct VMClass {
    pub name: &'static str,
    pub parent: &'static str,
    pub vtables: Vec<VMVTable>,
    pub members: Vec<VMMember>,
    pub static_methods: Vec<VMMethod>,
    pub static_members: Vec<VMMember>,
}

impl VMClass {
    pub fn new(
        name: &'static str,
        parent: &'static str,
        vtables: Vec<VMVTable>,
        members: Vec<VMMember>,
        static_methods: Vec<VMMethod>,
        static_members: Vec<VMMember>,
    ) -> Self {
        VMClass {
            name,
            parent,
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

pub fn generate_object_class() -> VMClass {
    let vtable = VMVTable::new(
        "core::Object",
        None,
        vec![
            VMMethod::new(
                "core::Object::downcast",
                object_downcast as *const (),
                vec![TypeTag::Object, TypeTag::Object, TypeTag::U64]
                ),
        ]
    );

    VMClass::new("core::Object", "", vec![vtable], Vec::new(), Vec::new(), Vec::new())
}


extern "C" fn object_downcast(context: &mut BytecodeContext, this: Reference, class_index: u64) -> Reference {
    let object = this;
    let object = unsafe { object.as_mut().unwrap() };
    if object.class == class_index as Symbol {
        this
    } else {
        if !object_downcast(context, object.parent_object, class_index).is_null() {
            return this;
        }
        std::ptr::null_mut()
    }
}

pub fn generate_printer_class() -> VMClass {
    let vtable = VMVTable::new(
        "core::Printer",
        None,
        vec![
            VMMethod::new(
                "core::Printer::println-int",
                printer_println_int as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "core::Printer::println-float",
                printer_println_float as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::F64]
                ),
            VMMethod::new(
                "core::Printer::println",
                printer_println as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
                ),
            VMMethod::new(
                "core::Printer::println-ints",
                printer_println_ints as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64]
            ),
        ]
    );

    VMClass::new("core::Printer", "core::Object", vec![vtable], Vec::new(), Vec::new(), Vec::new())
}


extern "C" fn printer_println_int(_: &mut BytecodeContext, _: Reference, int: u64) {
    println!("{}", int);
}

extern "C" fn printer_println_float(context: &mut BytecodeContext, _: Reference, float: f64) {
    println!("{}", float);
}

extern "C" fn printer_println(context: &mut BytecodeContext, _: Reference, string: Reference) {
    let object = string;
    let object = unsafe { object.as_ref().unwrap() };
    let length = unsafe { object.get::<u64>(0) };
    let pointer = unsafe { object.get::<u64>(8) };
    let pointer = pointer as *mut u8;
    let slice = unsafe { std::slice::from_raw_parts(pointer, length as usize) };
    let string = unsafe { std::str::from_utf8_unchecked(slice) };
    println!("{}", string);
}

extern "C" fn printer_println_ints(ctx: &mut BytecodeContext, this: Reference, int1: u64, int2: u64, int3: u64, int4: u64, int5: u64, int6: u64, int7: u64) {
    //let ints = [int1, int2, int3, int4, int5, int6, int7];
    println!("ctx: {ctx:p}");
    println!("this: {this:p}");
    println!("{}", int1);
    println!("{}", int2);
    println!("{}", int3);
    println!("{}", int4);
    println!("{}", int5);
    println!("{}", int6);
    println!("{}", int7);
    /*for (i, int) in ints.iter().enumerate() {
        println!("{i}: {}", int);
    }*/
}

macro_rules! array_downcast_contents {
    (object, $ty:ty, $context:ident, $this:ident, $class_symbol:ident) => {
        {
            let object = $this;
            let object = unsafe { object.as_mut().unwrap() };
            let pointer = unsafe { object.get::<u64>(8) };
            let length = unsafe { object.get::<u64>(0) };
            let pointer = pointer as *mut Reference;
            unsafe {
                for i in 0..length as usize {
                    if object_downcast($context, *pointer.add(i), $class_symbol).is_null() {

                        //Runtime::normal_return($context);
                        return std::ptr::null_mut();
                    }
                }
            }

            //Runtime::normal_return($context);
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
                    concat!("core::",std::stringify!($array_name)),
                    None,
                    vec![
                        VMMethod::new(
                            concat!("core::", std::stringify!($array_name), "::init"),
                            [< array $variant _init>] as *const (),
                            vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                            ),
                        VMMethod::new(
                            concat!("core::", std::stringify!($array_name), "::len"),
                            array_len as *const (),
                            vec![TypeTag::U64, TypeTag::Object]
                            ),
                        VMMethod::new(
                            concat!("core::", std::stringify!($array_name), "::downcast-contents"),
                            [< array $variant _upcast_contents >] as *const (),
                            vec![TypeTag::U64, TypeTag::Object, TypeTag::U64]
                            ),
                    ]
                );

                let elements = vec![
                    VMMember::new(concat!("core::", std::stringify!($array_name), "::length"), TypeTag::U64),
                    VMMember::new(concat!("core::", std::stringify!($array_name), "::pointer"), TypeTag::U64)
                ];

                VMClass::new(concat!("core::", std::stringify!($array_name)), "core::Object", vec![vtable], elements, Vec::new(), Vec::new())
            }
        }
    };
}

macro_rules! array_create_init {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub extern "C" fn $fn_name(_: &mut BytecodeContext, this: Reference, length: u64) {
                use std::alloc::*;
                let object = this;
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

macro_rules! array_create_init_internal {
    ($variant:expr, $fn_name:ident, $ty:ty) => {
        paste! {
            pub fn $fn_name(context: &mut BytecodeContext, this: Reference, length: u64) {
                use std::alloc::*;
                let object = this;
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
            pub extern "C" fn $fn_name(context: &mut BytecodeContext, this: Reference, index: u64) -> $ty {
                let object = this;
                let object = object as *mut Array;
                let object = unsafe { object.as_ref().expect("array get") };
                let pointer = object.buffer;
                let length = object.length;
                let pointer = pointer as *mut $ty;
                if index >= length {
                    let exception = Runtime::new_object("IndexOutOfBounds");
                    out_of_bounds_init(context, exception, length, index);
                    //context.set_exception(exception);
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
            pub extern "C" fn $fn_name(context: &mut BytecodeContext, this: Reference, index: u64, value: $ty) {
                let object = this;
                let object = object as *mut Array;
                let object = unsafe { object.as_mut().expect("array set") };
                let pointer = object.buffer;
                let length = object.length;
                let pointer = pointer as *mut $ty;
                if index >= length {
                    let exception = Runtime::new_object("IndexOutOfBounds");
                    out_of_bounds_init(context, exception, length, index);
                    //context.set_exception(exception);
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
            pub extern "C" fn $fn_name(context: &mut BytecodeContext, this: Reference, class_symbol: u64) -> Reference {
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
        array_create_init_internal!($variant, [< array $variant _init_internal >], $ty);
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

extern "C" fn array_len(_: &mut Runtime, this: Reference) -> u64 {
    let object = this;
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
        "core::Exception",
        None,
        vec![
            VMMethod::new(
                "core::Exception::init",
                exception_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object]
            ),
            VMMethod::new(
                "core::Exception::fill-in-stack-trace",
                exception_fill_in_stack_trace as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
            VMMethod::new(
                "core::Exception::print-stack-trace",
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

    VMClass::new("core::Exception", "core::Object", vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn exception_init(_: &BytecodeContext, this: Reference, message: Reference) {
    use std::alloc::*;
    let object = this;
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

pub extern "C" fn exception_fill_in_stack_trace(_context: &mut Runtime, this: Reference) {
    let object = this;
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
        (new_pointer as *mut Reference, capacity)
    } else {
        (pointer as *mut Reference, capacity)
    };
    object.stack_capacity = capacity;
    object.stack_pointer = pointer;

    let backtrace = Runtime::new_object("Backtrace");

    /*let method_name = context.get_current_method();

    backtrace_init(context, backtrace, method_name,0, 0);*/

    unsafe {
        pointer.add(length as usize).write(backtrace);
    }
    object.stack_length = length + 1;
}

pub extern "C" fn exception_print_stack_trace(context: &mut Runtime, this: Reference) {
    let object = this;
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
        "core::Backtrace",
        None,
        vec![
            VMMethod::new(
                "core::Backtrace::init",
                backtrace_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::Object, TypeTag::U64, TypeTag::U64]
            ),
            VMMethod::new(
                "core::Backtrace::display",
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

    VMClass::new("core::Backtrace", "core::Object", vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn backtrace_init(_context: &mut Runtime, this: Reference, function_name: Reference, line: u64, column: u64) {
    let object = this;
    let object = object as *mut Backtrace;
    let object = unsafe { object.as_mut().unwrap() };
    object.function_name = function_name;
    object.line_number = line;
    object.column_number = column;
}

extern "C" fn backtrace_display(_context: &mut Runtime, this: Reference) {
    let object = this;
    let object = object as *mut Backtrace;
    let object = unsafe { object.as_ref().unwrap() };
    let function_name = object.function_name;
    let line = object.line_number;
    let column = object.column_number;

    let string = function_name;
    let string = string as *mut StringBuffer;
    let string = unsafe { string.as_ref().unwrap() };
    let string_length = string.length;
    let string_pointer = string.buffer;
    let string_slice = slice_from_raw_parts(string_pointer as *const u8, string_length as usize);
    let str = unsafe { std::str::from_utf8_unchecked(string_slice.as_ref().unwrap()) };

    println!("{} {}:{}", str, line, column);
}

pub fn generate_index_out_of_bounds_class() -> VMClass {
    let vtable = VMVTable::new(
        "core::IndexOutOfBounds",
        None,
        vec![
            VMMethod::new(
                "core::IndexOutOfBounds::init",
                out_of_bounds_init as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64, TypeTag::U64]
            ),
        ]
    );

    let elements = vec![
    ];

    VMClass::new("core::IndexOutOfBounds", "core::Exception", vec![vtable], elements, Vec::new(), Vec::new())
}

extern "C" fn out_of_bounds_init(context: &mut BytecodeContext, this: Reference, bounds: u64, index: u64) {
    let object = this;
    let object = unsafe { object.as_mut().unwrap() };

    let message = Runtime::new_object("core::StringBuffer"); // String Class Symbol
    let message = message as *mut StringBuffer;

    string_buffer_from_str(message, &format!("Index was {index} but length was {bounds}"));

    let base_exception = object.parent_object;
    exception_init(context, base_exception, message as Reference);
}

pub fn generate_null_pointer_class() -> VMClass {
    let vtable = VMVTable::new(
        "core::NullPointerException",
        None,
        vec![
            VMMethod::new(
                "core::NullPointerException::init",
                null_pointer_init as *const (),
                vec![TypeTag::Void, TypeTag::Object]
            ),
        ]
    );

    let elements = vec![
    ];

    VMClass::new("core::NullPointerException", "core::Exception", vec![vtable], elements, Vec::new(), Vec::new())
}

pub extern "C" fn null_pointer_init(context: &BytecodeContext, this: Reference) {
    let object = this;
    let object = unsafe { object.as_mut().unwrap() };

    let message = Runtime::new_object("core::StringBuffer"); // String Class Symbol
    let message = message as *mut StringBuffer;

    string_buffer_from_str(message, "NullPointerException");

    let base_exception = object.parent_object;
    exception_init(context, base_exception, message as Reference);
}

