use std::{collections::HashMap, io::Write, path::PathBuf};
use std::cmp::Ordering;
use either::Either;
use itertools::Itertools;
use rowan_shared::{bytecode::compiled::Bytecode, classfile::{Member, SignatureEntry, VTable, VTableEntry}, TypeTag};
use rowan_shared::classfile::{SignatureIndex, StaticMethods};
use crate::{trees::ir, trees::ir::{Class, Constant, Expression, File, Literal, Method, Parameter, Pattern, Statement, TopLevelStatement}, backend::compiler_utils::Frame};
use crate::trees::ir::{ClosureParameter, IfExpression, ParentDec};
use crate::trees::{BinaryOperator, PathName, Type, UnaryOperator, Text, Annotation, Span, Visibility};
use super::compiler_utils::{PartialClass, StaticMember};



fn create_stdlib() -> HashMap<Vec<String>, PartialClass> {
    let mut classes = HashMap::new();

    let mut object = PartialClass::new();
    object.set_name("core::Object");
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Object::downcast",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    object.add_vtable(&vec![String::from("core"), String::from("Object")], vtable, &names, &signatures);
    object.make_not_printable();
    classes.insert(vec![String::from("Object")], object);

    let mut printer = PartialClass::new();
    printer.set_name("core::Printer");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Printer::println-int",
        "core::Printer::println-float",
        "core::Printer::println",
        "core::Printer::println-ints",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::F64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64, TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    printer.add_vtable(&vec![String::from("core"), String::from("Printer")], vtable, &names, &signatures);
    printer.make_not_printable();
    classes.insert(vec![String::from("Printer")], printer);

    
    let mut string = PartialClass::new();
    string.set_name("core::String");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::String::is-char-boundary",
        "core::String::as-bytes",
        "core::String::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
        SignatureEntry::new(vec![TypeTag::U64, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    
    string.add_vtable(&vec![String::from("core"), String::from("String")], vtable, &names, &signatures);
    string.make_not_printable();
    classes.insert(vec![String::from("String")], string);

    let mut string_buffer = PartialClass::new();
    string_buffer.set_name("core::StringBuffer");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::String::is-char-boundary",
        "core::String::as-bytes",
        "core::String::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
        SignatureEntry::new(vec![TypeTag::U64, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);

    string_buffer.add_vtable(&vec![String::from("core"), String::from("String")], vtable, &names, &signatures);

    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::StringBuffer::push",
        "core::StringBuffer::intern",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object, TypeTag::U32]),
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    string_buffer.add_vtable(&vec![String::from("core"), String::from("StringBuffer")], vtable, &names, &signatures);

    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::StringBuffer::from-interned",
        "core::StringBuffer::new",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
        SignatureEntry::new(vec![TypeTag::Object]),
    ];
    let static_methods = StaticMethods::new(functions);
    
    string_buffer.add_static_methods(&vec![String::from("core"), String::from("StringBuffer")], static_methods, &names, &signatures);
    string_buffer.make_not_printable();
    classes.insert(vec![String::from("StringBuffer")], string_buffer);


    let mut interned_string = PartialClass::new();
    interned_string.set_name("core::InternedString");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::String::is-char-boundary",
        "core::String::as-bytes",
        "core::String::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
        SignatureEntry::new(vec![TypeTag::U64, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);

    interned_string.add_vtable(&vec![String::from("core"), String::from("String")], vtable, &names, &signatures);

    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::InternedString::to-buffer",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    interned_string.add_vtable(&vec![String::from("core"), String::from("InternedString")], vtable, &names, &signatures);

    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::InternedString::from-buffer",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
    ];
    let static_methods = StaticMethods::new(functions);

    interned_string.add_static_methods(&vec![String::from("core"), String::from("InternedString")], static_methods, &names, &signatures);
    interned_string.make_not_printable();
    classes.insert(vec![String::from("InternedString")], interned_string);

    let mut u8_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::U8::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::U8]),
    ];
    let static_methods = StaticMethods::new(functions);

    u8_box.add_static_methods(&vec![String::from("core"), String::from("U8")], static_methods, &names, &signatures);
    u8_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::U8,
    }, "value");
    u8_box.make_not_printable();
    classes.insert(vec![String::from("U8")], u8_box);

    let mut u16_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::U16::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::U16]),
    ];
    let static_methods = StaticMethods::new(functions);

    u16_box.add_static_methods(&vec![String::from("core"), String::from("U16")], static_methods, &names, &signatures);
    u16_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::U16,
    }, "value");
    u16_box.make_not_printable();
    classes.insert(vec![String::from("U16")], u16_box);

    let mut u32_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::U32::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::U32]),
    ];
    let static_methods = StaticMethods::new(functions);

    u32_box.add_static_methods(&vec![String::from("core"), String::from("U32")], static_methods, &names, &signatures);
    u32_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::U32,
    }, "value");
    u32_box.make_not_printable();
    classes.insert(vec![String::from("U32")], u32_box);

    let mut u64_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::U64::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::U64]),
    ];
    let static_methods = StaticMethods::new(functions);

    u64_box.add_static_methods(&vec![String::from("core"), String::from("U64")], static_methods, &names, &signatures);
    u64_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::U64,
    }, "value");
    u64_box.make_not_printable();
    classes.insert(vec![String::from("U64")], u64_box);



    let mut i8_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::I8::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::I8]),
    ];
    let static_methods = StaticMethods::new(functions);

    i8_box.add_static_methods(&vec![String::from("core"), String::from("I8")], static_methods, &names, &signatures);
    i8_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::I8,
    }, "value");
    i8_box.make_not_printable();
    classes.insert(vec![String::from("I8")], i8_box);

    let mut i16_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::I16::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::I16]),
    ];
    let static_methods = StaticMethods::new(functions);

    i16_box.add_static_methods(&vec![String::from("core"), String::from("I16")], static_methods, &names, &signatures);
    i16_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::I16,
    }, "value");
    i16_box.make_not_printable();
    classes.insert(vec![String::from("I16")], i16_box);

    let mut i32_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::I32::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::I32]),
    ];
    let static_methods = StaticMethods::new(functions);

    i32_box.add_static_methods(&vec![String::from("core"), String::from("I32")], static_methods, &names, &signatures);
    i32_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::I32,
    }, "value");
    i32_box.make_not_printable();
    classes.insert(vec![String::from("I32")], i32_box);

    let mut i64_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::I64::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::I64]),
    ];
    let static_methods = StaticMethods::new(functions);

    i64_box.add_static_methods(&vec![String::from("core"), String::from("I64")], static_methods, &names, &signatures);
    i64_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::I64,
    }, "value");
    i64_box.make_not_printable();
    classes.insert(vec![String::from("I64")], i64_box);

    let mut f32_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::F32::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::F32]),
    ];
    let static_methods = StaticMethods::new(functions);

    f32_box.add_static_methods(&vec![String::from("core"), String::from("F32")], static_methods, &names, &signatures);
    f32_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::F32,
    }, "value");
    f32_box.make_not_printable();
    classes.insert(vec![String::from("F32")], f32_box);

    let mut f64_box = PartialClass::new();
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "core::F64::create",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::F64]),
    ];
    let static_methods = StaticMethods::new(functions);

    f64_box.add_static_methods(&vec![String::from("core"), String::from("F64")], static_methods, &names, &signatures);
    f64_box.add_member(Member {
        name: 0,
        type_tag: TypeTag::F64,
    }, "value");
    f64_box.make_not_printable();
    classes.insert(vec![String::from("F64")], f64_box);


    let mut array = PartialClass::new();
    array.set_name("core::Array8");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Array8::init",
        "core::Array8::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("core"), String::from("Array8")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Array8")], array);

    let mut array = PartialClass::new();
    array.set_name("core::Array16");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Array16::init",
        "core::Array16::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);

    array.add_vtable(&vec![String::from("core"), String::from("Array16")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Array16")], array);

    let mut array = PartialClass::new();
    array.set_name("core::Array32");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Array32::init",
        "core::Array32::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("core"), String::from("Array32")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Array32")], array);

    let mut array = PartialClass::new();
    array.set_name("core::Array64");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Array64::init",
        "core::Array64::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("core"), String::from("Array64")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Array64")], array);

    let mut array = PartialClass::new();
    array.set_name("core::Arrayf32");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Arrayf32::init",
        "core::Arrayf32::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("core"), String::from("Arrayf32")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Arrayf32")], array);

    let mut array = PartialClass::new();
    array.set_name("core::Arrayf64");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Arrayf64::init",
        "core::Arrayf64::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("core"), String::from("Arrayf64")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Arrayf64")], array);

    let mut array = PartialClass::new();
    array.set_name("core::Arrayobject");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "core::Arrayobject::init",
        "core::Arrayobject::len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("core"), String::from("Arrayobject")], vtable, &names, &signatures);
    array.make_not_printable();
    classes.insert(vec![String::from("Arrayobject")], array);
    
    classes
}

#[derive(Debug)]
pub enum CompilerError {
    UnboundIdentifer(String, usize, usize),
    MalformedCharacter(String, usize, usize),
    
}


pub struct Compiler {
    scopes: Vec<Frame>,
    pub(crate) classes: HashMap<Vec<String>, PartialClass>,
    current_block: u64,
    method_returned: bool,
    current_type_args: HashMap<String, TypeTag>,
    current_module: Vec<String>,
    active_imports: HashMap<String, Vec<String>>,
    imports_to_change: HashMap<String, Vec<String>>,
    current_block_returned: bool,
    closures: HashMap<String, Vec<String>>,
    closures_under_path: HashMap<String, usize>,
}


impl Compiler {

    pub fn new() -> Compiler {
        Compiler {
            scopes: Vec::new(),
            classes: create_stdlib(),
            current_block: 0,
            method_returned: false,
            current_type_args: HashMap::new(),
            current_module: Vec::new(),
            active_imports: HashMap::new(),
            imports_to_change: HashMap::new(),
            current_block_returned: false,
            closures: HashMap::new(),
            closures_under_path: HashMap::new(),
        }
    }

    fn increment_block(&mut self) {
        self.current_block += 1;
    }

    fn push_scope(&mut self) {
        if self.scopes.len() == 0 {
            self.scopes.push(Frame::new());
        } else {
            let frame = Frame::new_with_location(self.scopes.last().expect("No frames despite just checking for them").get_location());
            self.scopes.push(frame);
        }
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() == 0 {
            unreachable!("Popped with no scopes");
        } else {
            self.scopes.pop();
        }
    }

    fn bind_variable(&mut self, name: impl AsRef<str>) -> u8 {
        
        let mut binding = None;
        for frame in self.scopes.iter().rev() {
            match frame.get_binding(name.as_ref()) {
                Some(pos) => {
                    binding = Some(pos);
                }
                None => {}
            }
        }
        match binding {
            Some(pos) => {
                self.scopes.last_mut().expect("No scopes").set_binding(name, pos);
                pos
            }
            None => {
                self.scopes.last_mut().expect("No scopes").add_binding(name)
            }
        }
    }

    fn get_variable(&self, name: impl AsRef<str>) -> Option<u8> {
        for frame in self.scopes.iter().rev() {
            match frame.get_binding(name.as_ref()) {
                Some(pos) => {
                    return Some(pos);
                }
                None => {}
            }
        }
        None
    }

    fn add_path_if_needed(&self, class: String) -> Vec<String> {
        if class.contains("::") {
            return class.split("::").map(|s| s.to_string()).collect();
        }
        let path = self.active_imports.get(&class);
        if let Some(path) = path {
            let module = path.clone();
            module
        } else if self.classes.get(&vec![class.clone()]).is_some() {
            return vec![class]
        } else {
            let mut module = self.current_module.clone();
            module.push(class);
            module
        }
    }

    fn alter_imports_if_needed(&mut self) {
        let mut imports_to_change = Vec::new();
        for import in self.active_imports.keys() {
            if self.imports_to_change.contains_key(import) {
                imports_to_change.push((import.clone(), self.active_imports.get(import).unwrap().clone()));
            }
        }
        for (import, mut path) in imports_to_change {
            self.active_imports.remove(&import);
            for change in self.imports_to_change.get(&import).unwrap() {
                let last_item = path.last_mut().unwrap();
                *last_item = change.clone();
                self.active_imports.insert(change.clone(), path.clone());
            }
        }
    }

    fn create_closure_class(&mut self, args: &[ClosureParameter], return_type: &Type) -> String {
        let mut closure_name = String::from("Closure");
        let ret_type = match return_type {
            Type::I8 => TypeTag::I8,
            Type::U8 => TypeTag::U8,
            Type::I16 => TypeTag::I16,
            Type::U16 => TypeTag::U16,
            Type::I32 => TypeTag::I32,
            Type::U32 => TypeTag::U32,
            Type::I64 => TypeTag::I64,
            Type::U64 => TypeTag::U64,
            Type::F32 => TypeTag::F32,
            Type::F64 => TypeTag::F64,
            Type::Void => TypeTag::Void,
            _ => TypeTag::Object,
        };
        let mut types = vec![ret_type, TypeTag::Object];
        for param in args {
            match param {
                ClosureParameter { parameter: Parameter::Pattern { ty, ..} } => {

                    match ty {
                        Type::I8 => {
                            types.push(TypeTag::I8);
                            closure_name.push_str("i8")
                        },
                        Type::U8 => {
                            types.push(TypeTag::U8);
                            closure_name.push_str("u8")
                        },
                        Type::I16 => {
                            types.push(TypeTag::I16);
                            closure_name.push_str("i16")
                        },
                        Type::U16 => {
                            types.push(TypeTag::U16);
                            closure_name.push_str("u16")
                        },
                        Type::I32 => {
                            types.push(TypeTag::I32);
                            closure_name.push_str("i32")
                        },
                        Type::U32 => {
                            types.push(TypeTag::U32);
                            closure_name.push_str("u32")
                        },
                        Type::I64 => {
                            types.push(TypeTag::I64);
                            closure_name.push_str("i64")
                        },
                        Type::U64 => {
                            types.push(TypeTag::U64);
                            closure_name.push_str("u64")
                        },
                        Type::F32 => {
                            types.push(TypeTag::F32);
                            closure_name.push_str("f32")
                        },
                        Type::F64 => {
                            types.push(TypeTag::F64);
                            closure_name.push_str("f64")
                        },
                        _ => {
                            types.push(TypeTag::Object);
                            closure_name.push_str("object")
                        },
                    }
                }
                _ => unreachable!("self can't be in a closure parameter")
            }
        }
        match return_type {
            Type::I8  => closure_name.push_str("i8"),
            Type::U8 => closure_name.push_str("u8"),
            Type::I16  => closure_name.push_str("i16"),
            Type::U16 => closure_name.push_str("u16"),
            Type::I32 => closure_name.push_str("i32"),
            Type::U32 => closure_name.push_str("u32"),
            Type::I64 => closure_name.push_str("i64"),
            Type::U64 => closure_name.push_str("u64"),
            Type::F32 => closure_name.push_str("f32"),
            Type::F64 => closure_name.push_str("f64"),
            Type::Void => closure_name.push_str("void"),
            _ => closure_name.push_str("object"),
        }

        if self.closures.contains_key(&closure_name) {
            return closure_name;
        }
        self.active_imports.insert(closure_name.clone(), vec![
            String::from("std"),
            String::from("closure"),
            closure_name.clone()
        ]);

        let mut partial_class = PartialClass::new();
        partial_class.set_name(&format!("std::closure::{}", closure_name));
        partial_class.set_parent("core::Object");
        let functions = vec![
            VTableEntry::default(),
        ];
        let names = vec![
            format!("std::closure::{}::call", closure_name),
        ];
        let signatures = vec![
            SignatureEntry::new(types),
        ];
        let vtable = VTable::new(functions);

        partial_class.add_vtable(&vec![String::from("std"), String::from("closure"), closure_name.clone()], vtable, &names, &signatures);
        let path = vec![
            String::from("std"),
            String::from("closure"),
            closure_name.clone(),
        ];

        let closure_path = [
            String::from("std"),
            String::from("closure"),
            closure_name.clone(),
        ];

        partial_class.attach_bytecode(
            &closure_path,
            format!("std::closure::{closure_name}::call"),
            &[0],
            false,
        ).unwrap();

        self.classes.insert(path.clone(), partial_class);

        self.closures.insert(closure_name.clone(), path);

        closure_name
    }

    /// files should be sorted in a way that means we don't need to do each file incrementally
    pub fn compile_files(
        mut self, 
        files: Vec<File>,
    ) -> Result<(), CompilerError> {

        for file in files {
            let File { path, content, .. } = file;
            self.current_module = path.segments.into_iter().map(|x| x.to_string()).collect();
            let mut content = content;

            content.sort_by(|a, b| {
                match (a, b) {
                    (TopLevelStatement::Class(_), TopLevelStatement::Import(_)) => {
                        Ordering::Greater
                    }
                    (TopLevelStatement::Import(_), TopLevelStatement::Class(_)) => {
                        Ordering::Less
                    }
                    _ => Ordering::Equal,
                }
            });

            for statement in content {
                match statement {
                    TopLevelStatement::Class(class) => {
                        self.alter_imports_if_needed();
                        self.compile_class(class)?;
                    }
                    TopLevelStatement::Import(import) => {
                        let path = import.path.segments.iter().map(ToString::to_string).collect();

                        self.active_imports.insert(import.path.segments.last().map(ToString::to_string).unwrap(), path);
                    }
                }
            }
        }

        for (path, file) in self.classes.into_iter() {
            if file.is_printable() && file.get_class_name().contains(&String::from("Closure0")) {
                println!("closure: {file:#?}");
            }
            if let Some((file, native_definitions)) = file.create_class_file() {
                if !native_definitions.is_empty() {
                    let path = format!("output/{}.h", path.join("/"));
                    let header = native_definitions.as_c_header();
                    let bytes = header.as_bytes();
                    let path = PathBuf::from(path);
                    if let Some(parents) = path.parent() {
                        let _ = std::fs::create_dir_all(parents);
                    }
                    let _ = std::fs::remove_file(&path);
                    let mut file = std::fs::File::create(path).unwrap();
                    file.write_all(&bytes).unwrap();
                }
                let path = format!("output/{}.class", path.join("/"));
                let bytes = file.as_binary();
                let path = PathBuf::from(path);
                if let Some(parents) = path.parent() {
                    let _ = std::fs::create_dir_all(parents);
                }
                let _ = std::fs::remove_file(&path);
                let mut file = std::fs::File::create(path).unwrap();
                file.write_all(&bytes).unwrap();
            }
        }
        Ok(())
    }

    fn compile_class(&mut self, class: Class) -> Result<(), CompilerError> {
        let Class {
            name,
            type_params,
            parent,
            members,
            methods,
            static_members,
            ..
        } = class;

        let class_name = self.add_path_if_needed(name.to_string());

        if type_params.is_empty() {
            let new_parents = self.create_new_parent(&parent);
            self.compile_class_inner(&class_name, &new_parents, &methods, &members, &static_members)?;
        } else {
            let mut name_order = Vec::new();
            for type_param in type_params.iter() {
                name_order.push(type_param.name.to_string());
            }

            let permutations = vec![
                TypeTag::I8,
                TypeTag::I16,
                TypeTag::I32,
                TypeTag::I64,
                TypeTag::F32,
                TypeTag::F64,
                TypeTag::Object,
            ].into_iter().permutations(type_params.len()).collect::<Vec<_>>();

            for permutation in permutations {
                let mut modifier_string = String::new();
                for (name, typ) in name_order.iter().zip(permutation.into_iter()) {
                    if let Some(value) = self.current_type_args.get_mut(name) {
                        *value = typ;
                    } else {
                        self.current_type_args.insert(name.to_string(), typ);
                    }
                    let modifier = match typ {
                        TypeTag::I8 => "8",
                        TypeTag::I16 => "16",
                        TypeTag::I32 => "32",
                        TypeTag::I64 => "64",
                        TypeTag::F32 => "f32",
                        TypeTag::F64 => "f64",
                        TypeTag::Object => "object",
                        _ => unreachable!("bizarre possible type"),
                    };
                    modifier_string.push_str(modifier);
                }

                let new_parents = self.create_new_parent(&parent);

                self.imports_to_change.entry(class_name.last().unwrap().to_string())
                    .or_insert(Vec::new())
                    .push(format!("{}{modifier_string}", class_name.last().unwrap()));

                let mut new_path = class_name.clone();
                new_path.last_mut().unwrap().push_str(&modifier_string);

                self.compile_class_inner(&new_path, &new_parents, &methods, &members, &static_members)?;
            }
        }
        
        Ok(())
    }

    fn create_new_parent<'a>(&mut self, parent: &'a Option<ParentDec<'a>>) -> Option<ParentDec<'a>> {
        parent.as_ref().map(|parent| {
            let mut string = parent.name.to_string();
            for type_arg in parent.type_args.iter() {
                let str_value = match type_arg {
                    Type::I8 | Type::U8 => "8",
                    Type::I16 | Type::U16 => "16",
                    Type::I32 | Type::U32 => "32",
                    Type::I64 | Type::U64 => "64",
                    Type::F32 => "f32",
                    Type::F64 => "f64",
                    Type::Object(name, _) => {
                        if self.current_type_args.contains_key(name.as_str()) {
                            match type_arg {
                                Type::I8 | Type::U8 => "8",
                                Type::I16 | Type::U16 => "16",
                                Type::I32 | Type::U32 => "32",
                                Type::I64 | Type::U64 => "64",
                                Type::F32 => "f32",
                                Type::F64 => "f64",
                                _ => "object",
                            }
                        } else {
                            "object"
                        }
                    }
                    _ => "object",
                };
                string.push_str(str_value);
            }
            ParentDec {
                name: Text::Owned(string),
                type_args: Vec::new(),
                type_params: parent.type_params.clone(),
                span: parent.span,
            }
        })
    }

    fn compile_class_inner(
        &mut self,
        name: &Vec<String>,
        parent: &Option<ParentDec>,
        methods: &Vec<Method>,
        members: &Vec<ir::Member>,
        static_members: &Vec<ir::StaticMember>,
    ) -> Result<(), CompilerError> {
        let mut partial_class = PartialClass::new();
        let path_name = name.join("::");
        partial_class.set_name(&path_name);

        let parent_vtables = parent.as_ref().map(|parent_name| {
            let path = self.add_path_if_needed(parent_name.name.clone().to_string());
            let partial_class = self.classes.get(&path).expect("Order of files is wrong");
            let vtables = partial_class.get_vtables(&path);
            vtables.into_iter().map(|(table, names, signatures)| {
                let class_name = partial_class.index_string_table(table.class_name).split("::")
                    .map(|name| name.to_string()).collect::<Vec<String>>();
                let source_class = if table.sub_class_name == 0 {
                    None
                } else {
                    Some(partial_class.index_string_table(table.sub_class_name))
                };
                (class_name, source_class, table, names, signatures)
            }).collect::<Vec<_>>()

        });
        
        parent.as_ref().map(|parent| {
            let path = self.add_path_if_needed(parent.name.clone().to_string()).join("::");
            partial_class.set_parent(&path);
        });

        let (
            vtable,
            names,
            signatures,
            static_method_map,
            static_signatures,
        ) = self.construct_vtable(&name, &methods)?;

        partial_class.add_signatures(static_signatures);

        partial_class.set_static_method_to_sig(static_method_map);

        if vtable.functions.len() != 0 {
            partial_class.add_vtable(&name, vtable, &names, &signatures);
        } else {
            drop(vtable);
            drop(names);
            drop(signatures);
        }

        if parent.is_none() {
            let object_class = self.classes.get(&vec!["Object".to_string()]).expect("Object not added to known classes");

            let vtables = object_class.get_vtables(&[
                String::from("core"),
                String::from("Object"),
            ]);
            let (vtable, names, signatures) = &vtables[0];
            let names = names.iter()
                .map(|n| n.clone())
                .collect::<Vec<String>>();

            let vtable = vtable.clone();

            partial_class.add_vtable(&vec![String::from("core"), String::from("Object")], vtable, &names, signatures);
            partial_class.set_parent("core::Object");
        }

        if let Some(vtables) = parent_vtables {
            for (class_name, source_class, vtable, names, signatures) in vtables {
                let names = names.into_iter()
                    .map(|n| self.add_path_if_needed(n).join("::"))
                    .collect::<Vec<String>>();
                let class_name = if let Some(source) = source_class {
                    let path = self.add_path_if_needed(source.to_string());
                    path
                } else {
                    class_name
                };
                partial_class.add_vtable(&class_name, vtable.clone(), &names, &signatures);
            }
        }

        members.into_iter().map(|member| {
            (member.name.clone(), Member::new(self.convert_type(&member.ty)))
        }).for_each(|(name, member)| {
            partial_class.add_member(member, name);
        });

        let mut static_init_bytecode = Vec::new();
        static_init_bytecode.push(Bytecode::StartBlock(0));

        let class_name = name;
        static_members.into_iter().map(|member| {
            let new_ty = self.convert_type(&member.ty);
            (&member.name, StaticMember::new(member.is_const, new_ty), &member.value)
        })
            .collect::<Vec<(&Text, StaticMember, &Option<Expression>)>>()
            .into_iter()
            .enumerate()
            .flat_map(|(i, (name, member, value))| {
                let ty = member.type_tag.clone();
                let mut member_name = class_name.clone();
                member_name.push(name.to_string());
                let name = member_name.join("::");
                partial_class.add_static_member(member, name);
                value.as_ref().map(|value| {
                    self.compile_expression(
                        &class_name,
                        &mut partial_class,
                        &value,
                        &mut static_init_bytecode,
                        true
                    ).map(|_| {
                        let index = partial_class.add_string(&class_name.join("::"));
                        static_init_bytecode.push(Bytecode::SetStaticMember(index, i as u64, ty));
                        Ok::<(), CompilerError>(())
                    })
                })
            }).collect::<Result<Result<(), CompilerError>, CompilerError>>()??;

        static_init_bytecode.push(Bytecode::ReturnVoid);

        let static_init_bytecode = static_init_bytecode.into_iter().flat_map(|code| {
            code.into_binary()
        }).collect::<Vec<_>>();

        partial_class.attach_static_init_bytecode(static_init_bytecode).expect("attaching bytecode error");

        self.compile_methods(&class_name, &mut partial_class, methods)?;

        self.classes.insert(class_name.clone(), partial_class);

        Ok(())
    }

    fn construct_vtable(&self, class_name: &Vec<String>, methods: &Vec<Method>) -> Result<(
        VTable,
        Vec<String>,
        Vec<SignatureEntry>,
        HashMap<String,SignatureIndex>,
        Vec<SignatureEntry>,
    ), CompilerError> {


        let mut entries = Vec::new();
        let mut names = Vec::new();
        let mut signatures = Vec::new();
        let mut static_signatures = Vec::new();
        let mut static_method_to_signature = HashMap::new();

        'methods: for method in methods.iter() {
            let Method {
                name,
                annotations,
                parameters,
                return_type,
                span: _span,
                ..
            } = method;

            for annotation in annotations.iter() {
                if annotation.name == "Override" {
                    continue 'methods;
                }
            }

            let mut signature = Vec::new();
            signature.push(self.convert_type(return_type));
            
            let mut static_method = true;

            parameters.iter().for_each(|param| {
                match param {
                    Parameter::This(_, _) => {
                        static_method = false;
                        signature.push(TypeTag::Object);
                    }
                    Parameter::Pattern { ty, .. } => {
                        signature.push(self.convert_type(ty));
                    }
                }

            });

            let signature_index = signatures.len() + static_signatures.len(); 
            
            if static_method {
                let mut class_name = class_name.clone();
                class_name.push(name.to_string());
                let name = class_name.join("::");
                static_method_to_signature.insert(name, signature_index as SignatureIndex);
                static_signatures.push(SignatureEntry::new(signature));
            } else {

                let name = if name.contains("::") {
                    name.to_string()
                } else {
                    let mut class_name = class_name.clone();
                    class_name.push(name.to_string());
                    class_name.join("::")
                };

                names.push(name);
                entries.push(VTableEntry::default());
                signatures.push(SignatureEntry::new(signature));
            }
        }
        let vtable = VTable::new(entries);

        Ok((vtable, names, signatures, static_method_to_signature, static_signatures))
    }

    fn convert_type(&self, ty: &Type) -> TypeTag {
        match ty {
            Type::Void => TypeTag::Void,
            Type::U8 => TypeTag::U8,
            Type::U16 => TypeTag::U16,
            Type::U32 => TypeTag::U32,
            Type::U64 => TypeTag::U64,
            Type::I8 => TypeTag::I8,
            Type::I16 => TypeTag::I16,
            Type::I32 => TypeTag::I32,
            Type::I64 => TypeTag::I64,
            Type::Char => TypeTag::U32,
            Type::Str => TypeTag::Str,
            Type::F32 => TypeTag::F32,
            Type::F64 => TypeTag::F64,
            Type::Array(_, _) => TypeTag::Object,
            Type::Object(text, _) => {
                let tag = self.current_type_args.get(text.as_str()).unwrap_or(&TypeTag::Object);
                *tag
            },
            Type::TypeArg(_, _, _) => TypeTag::Object,
            Type::Tuple(_, _) => TypeTag::Object,
            Type::Function(_, _, _) => TypeTag::Object,
            Type::Native => TypeTag::Native,
        }

    }


    pub fn compile_methods(&mut self, class_name: &Vec<String>, partial_class: &mut PartialClass, methods: &Vec<Method>) -> Result<(), CompilerError> {

        for method in methods {
            self.method_returned = false;
            let Method {
                name,
                parameters,
                body,
                is_native,
                ..
            } = method;

            self.push_scope();
            let mut is_static = true;
            for parameter in parameters {
                let Parameter::Pattern { name, .. } = parameter else {
                    self.bind_variable("self");
                    is_static = false;
                    continue;
                };
                self.bind_patterns(&name);
            }
            
            
            let mut bytecode = self.compile_method_body(class_name, partial_class, body)?;

            if !self.method_returned {
                bytecode.push(Bytecode::ReturnVoid);
            }

            let bytecode = bytecode.into_iter().flat_map(|code| {
                code.into_binary()
            }).collect::<Vec<_>>();

            if is_static {
                let mut class_name = partial_class.get_class_name();
                class_name.push(name.to_string());
                let name = class_name.join("::");
                partial_class.add_static_method(name, bytecode, *is_native);
            } else {
                //println!("{}", name);
                let path_name = if name.contains("::") {
                    name.to_string()
                } else {
                    let mut path_name = class_name.clone();
                    path_name.push(name.to_string());
                    path_name.join("::")
                };

                let vtable = partial_class.get_vtable(&path_name).unwrap();
                let method_class_name = partial_class.index_string_table(vtable.sub_class_name).split("::")
                    .map(|name| name.to_string())
                    .collect::<Vec<String>>();
                //println!("{}", method_class_name);

                let method_name = if name.contains("::") {
                    name.to_string()
                } else {
                    let mut method_name = method_class_name.clone();
                    method_name.push(name.to_string());
                    method_name.join("::")
                };

                partial_class.attach_bytecode(&method_class_name, method_name, bytecode, *is_native).expect("Handle partial class error");
            }

            self.pop_scope();
        }

        Ok(())
    }

    fn bind_patterns(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Variable(name, _, _) => {
                self.bind_variable(name);
            }
            Pattern::Tuple(bindings, _) => {
                for binding in bindings {
                    self.bind_patterns(binding);
                }
            }
            _ => {}
        }
    }

    fn compile_method_body(&mut self, class_name: &Vec<String>, partial_class: &mut PartialClass, body: &Vec<Statement>) -> Result<Vec<Bytecode>, CompilerError> {
        let mut output = Vec::new();
        //output.push(Bytecode::StartBlock(block));
        //output.push(Bytecode::Goto(1));
        self.current_block = 0;
        self.compile_block(class_name, partial_class, body, &mut output)?;
        //println!("Bytecode output: {:#?}", output);

        Ok(output)
    }

    fn compile_block(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass,
        body: &Vec<Statement>,
        output: &mut Vec<Bytecode>
    ) -> Result<(), CompilerError> {

        self.push_scope();
        let block = self.current_block;
        output.push(Bytecode::StartBlock(block));
        
        for statement in body {
            match statement {
                Statement::Expression(expr, _) => {
                    self.compile_expression(class_name, partial_class, &expr, output, false)?;
                }
                Statement::Let { bindings, value, .. } => {
                    self.compile_expression(class_name, partial_class, &value, output, false)?;
                    match bindings {
                        Pattern::Variable(var, _, _) => {
                            let index = self.bind_variable(var);
                            output.push(Bytecode::StoreLocal(index));

                        }
                        _ => todo!("let bindings"),
                    }
                }
                Statement::While { test, body, .. } => {
                    output.push(Bytecode::Goto(1));
                    self.increment_block();
                    let while_test_block = self.current_block;
                    output.push(Bytecode::StartBlock(while_test_block));
                    self.compile_expression(class_name, partial_class, test, output, false)?;
                    output.push(Bytecode::If(1, 2));
                    self.increment_block();
                    self.compile_block(class_name, partial_class, body, output)?;
                    let while_loop_block = while_test_block as i64 - self.current_block as i64;
                    output.push(Bytecode::Goto(while_loop_block));
                    self.increment_block();
                    let exit_block = self.current_block;
                    output.push(Bytecode::StartBlock(exit_block));
                }
                Statement::Assignment { target, value, .. } => {
                    match target {
                        Expression::Variable(name, _,  _) => {
                            let var_index = self.get_variable(name).expect("report unbound variable");
                            self.compile_expression(class_name, partial_class, value, output, false)?;
                            output.push(Bytecode::StoreLocal(var_index));
                        }
                        Expression::BinaryOperation { operator: BinaryOperator::Index, .. } => {
                            let defer = match self.compile_expression(class_name, partial_class, target, output, true) {
                                Ok(defer) => defer,
                                Err(err) => return Err(err),
                            };
                            self.compile_expression(class_name, partial_class, value, output, false)?;
                            if let Some(defer) = defer {
                                defer(output);
                            }
                        }
                        Expression::MemberAccess {
                             ..
                        } => {
                            let defer = self.compile_member_set(class_name, partial_class, target, output)?;
                            self.compile_expression(class_name, partial_class, value, output, false)?;
                            if let Some(defer) = defer {
                                defer(output);
                            }
                        }
                        _ => todo!("lhs assignment")
                    }
                }
                _ => unimplemented!("compile_block statement: {:?}", statement),
            }
        }

        self.pop_scope();

        Ok(())
    }

    fn compile_expression<'a>(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass, 
        expr: &'a Expression,
        output: &mut Vec<Bytecode>,
        lhs : bool,
    ) -> Result<Option<Box<dyn Fn(&mut Vec<Bytecode>) + 'a>>, CompilerError> {
        // println!("lhs {}", lhs);
        // println!("Expression: {:#?}", expr);
        match expr {
            Expression::Variable(var, _, span) => {
                let index = self.get_variable(var)
                    .ok_or(CompilerError::UnboundIdentifer(var.clone().to_string(), span.start, span.end))?;
                output.push(Bytecode::LoadLocal(index));
                
            }
            Expression::Literal(lit) => {
                match lit {
                    Literal::Constant(constant) => {
                        match constant {
                            Constant::Bool(val, span) => {
                                if *val {
                                    output.push(Bytecode::LoadU8(1));
                                } else {
                                    output.push(Bytecode::LoadU8(0));
                                }
                            }
                            Constant::Character(char_str, span) => {
                                let chr = match char_str.as_str() {
                                    "\\n" => '\n',
                                    "\\t" => '\t',
                                    "\\r" => '\r',
                                    "\\\\" => '\\',
                                    x if x.contains("\\x") => {
                                        x.parse::<char>().expect("Char parse error")
                                    }
                                    x if x.chars().count() == 1 => {
                                        x.chars().next().unwrap()
                                    }
                                    x => return Err(CompilerError::MalformedCharacter(x.to_string(), span.start, span.end)),
                                };

                                let value = chr as u32;

                                output.push(Bytecode::LoadU32(value));
                            }
                            Constant::String(string, _) => {
                                let string = string.replace("\\n", "\n");
                                let string = string.replace("\\r", "\r");
                                let string = string.replace("\\t", "\t");
                                let string = string.replace("\\\\", "\\");

                                let string_ref = partial_class.add_string(string);
                                output.push(Bytecode::GetStrRef(string_ref));
                            }
                            Constant::Float(value, ty, _) => {
                                match ty {
                                    Type::F32 => {
                                        let value = value.parse::<f32>().expect("malformed f32");
                                        output.push(Bytecode::LoadF32(value));
                                    }
                                    Type::F64 => {
                                        let value = value.parse::<f64>().expect("malformed f64");
                                        output.push(Bytecode::LoadF64(value));
                                    }
                                    _ => unreachable!("float literal"),
                                }
                            }
                            Constant::Integer(value, ty, _) => {
                                match ty {
                                    Type::F32 => {
                                        let value = value.parse::<f32>().expect("malformed f32");
                                        output.push(Bytecode::LoadF32(value));
                                    }
                                    Type::F64 => {
                                        let value = value.parse::<f64>().expect("malformed f64");
                                        output.push(Bytecode::LoadF64(value));
                                    }
                                    Type::U8 => {
                                        let value = value.parse::<u8>().expect("malformed u8");
                                        output.push(Bytecode::LoadU8(value));
                                    }
                                    Type::U16 => {
                                        let value = value.parse::<u16>().expect("malformed u16");
                                        output.push(Bytecode::LoadU16(value));
                                    }
                                    Type::U32 => {
                                        let value = value.parse::<u32>().expect("malformed u32");
                                        output.push(Bytecode::LoadU32(value));
                                    }
                                    Type::U64 => {
                                        let value = value.parse::<u64>().expect("malformed u64");
                                        output.push(Bytecode::LoadU64(value));
                                    }
                                    Type::I8 => {
                                        let value = value.parse::<i8>().expect("malformed i8");
                                        output.push(Bytecode::LoadI8(value));
                                    }
                                    Type::I16 => {
                                        let value = value.parse::<i16>().expect("malformed i16");
                                        output.push(Bytecode::LoadI16(value));
                                    }
                                    Type::I32 => {
                                        let value = value.parse::<i32>().expect("malformed i32");
                                        output.push(Bytecode::LoadI32(value));
                                    }
                                    Type::I64 => {
                                        let value = value.parse::<i64>().expect("malformed i64");
                                        output.push(Bytecode::LoadI64(value));
                                    }
                                    x => unreachable!("integer literal {:?}", x)
                                }
                            }
                        }
                    }
                    Literal::Array(exprs, ty, _) => {
                        let type_tag = match ty {
                            Type::U8 => TypeTag::U8,
                            Type::I8 => TypeTag::I8,
                            Type::U16 => TypeTag::U16,
                            Type::I16 => TypeTag::I16,
                            Type::U32 => TypeTag::U32,
                            Type::I32 => TypeTag::I32,
                            Type::U64 => TypeTag::U64,
                            Type::F32 => TypeTag::F32,
                            Type::F64 => TypeTag::F64,
                            Type::I64 => TypeTag::I64,
                            Type::Array(_, _) => TypeTag::Object,
                            Type::Tuple(_, _) => TypeTag::Object,
                            Type::Char => TypeTag::U32,
                            Type::Str => TypeTag::Str,
                            Type::Object(_, _) => TypeTag::Object,
                            Type::Void => TypeTag::Void,
                            Type::TypeArg(_ ,_, _) => TypeTag::Object,
                            Type::Function(_, _, _) => TypeTag::Object,
                            Type::Native => unreachable!("Native should not ever occur in here"),
                        };
                        output.push(Bytecode::LoadU64(exprs.len() as u64));
                        output.push(Bytecode::CreateArray(type_tag));
                        for (i, expr) in exprs.into_iter().enumerate() {
                            output.push(Bytecode::Dup);
                            output.push(Bytecode::LoadU64(i as u64));
                            self.compile_expression(class_name, partial_class, expr, output, lhs)?;
                            output.push(Bytecode::ArraySet(type_tag));
                        }
                    }
                    _ => todo!("all other literals")
                }
            }
            Expression::This(_) => {
                output.push(Bytecode::LoadLocal(0));
            }
            Expression::BinaryOperation { operator, left, right, span } => {
                self.compile_expression(class_name, partial_class, left.as_ref(), output, lhs)?;
                self.compile_expression(class_name, partial_class, right.as_ref(), output, lhs)?;

                match (left.get_type(), operator, right.get_type()) {
                    (Either::Left(lhs), BinaryOperator::Add, Either::Left(rhs)) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::AddInt)
                    }
                    (Either::Left(lhs), BinaryOperator::Sub, Either::Left(rhs)) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::SubInt)
                    }
                    (Either::Left(lhs), BinaryOperator::Mul, Either::Left(rhs)) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::MulInt)
                    }
                    (Either::Left(lhs), BinaryOperator::Div, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::DivUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Div, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::DivSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Mod, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::ModUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Mod, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::ModSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Add, Either::Left(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::AddFloat)
                    }
                    (Either::Left(lhs), BinaryOperator::Sub, Either::Left(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::SubFloat)
                    }
                    (Either::Left(lhs), BinaryOperator::Mul, Either::Left(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::MulFloat)
                    }
                    (Either::Left(lhs), BinaryOperator::Div, Either::Left(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::DivFloat)
                    }
                    (Either::Left(lhs), BinaryOperator::Mod, Either::Left(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::ModFloat)
                    }
                    (Either::Left(lhs), BinaryOperator::Eq, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::EqualUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Eq, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::EqualSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Ne, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::NotEqualUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Ne, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::NotEqualSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Lt, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::LessUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Lt, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::LessSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Le, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::LessOrEqualUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Le, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::LessOrEqualSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Gt, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::GreaterUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Gt, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::GreaterSigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Ge, Either::Left(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::GreaterOrEqualUnsigned)
                    }
                    (Either::Left(lhs), BinaryOperator::Ge, Either::Left(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::GreaterOrEqualSigned)
                    }
                    (Either::Left(Type::U8), BinaryOperator::And, Either::Left(Type::U8)) => {
                        output.push(Bytecode::And)
                    }
                    (Either::Left(Type::U8), BinaryOperator::Or, Either::Left(Type::U8)) => {
                        output.push(Bytecode::Or)
                    }
                    (Either::Left(Type::Array(generic, _)), BinaryOperator::Index, _) => {
                        let type_tag = match generic.as_ref() {
                            Type::U8 => TypeTag::U8,
                            Type::I8 => TypeTag::I8,
                            Type::U16 => TypeTag::U16,
                            Type::I16 => TypeTag::I16,
                            Type::U32 => TypeTag::U32,
                            Type::I32 => TypeTag::I32,
                            Type::U64 => TypeTag::U64,
                            Type::I64 => TypeTag::I64,
                            Type::F32 => TypeTag::F32,
                            Type::F64 => TypeTag::F64,
                            Type::Object(_, _) => TypeTag::Object,
                            Type::TypeArg(_, _, _) => TypeTag::Object,
                            Type::Void => TypeTag::Void,
                            Type::Str => TypeTag::Str,
                            Type::Tuple(_, _) => TypeTag::Object,
                            Type::Array(_, _) => TypeTag::Object,
                            Type::Char => TypeTag::U32,
                            Type::Function(_, _, _) => TypeTag::Object,
                            Type::Native => unreachable!("Should not be able to get a native type"),
                        };

                        if lhs {
                            return Ok(Some(
                                Box::new(move |output: &mut Vec<Bytecode>| {
                                    output.push(Bytecode::ArraySet(type_tag));
                                })
                            ));
                        } else {
                            output.push(Bytecode::ArrayGet(type_tag));
                        }
                    }
                    (l, op, r) => todo!("binary operator {:?}: ({:?}: {:?}) ({:?}: {:?}) spanned: {:?}", op, left, l, right, r, span),
                }
                
            }
            Expression::UnaryOperation { operator, operand, span } => {
                self.compile_expression(class_name, partial_class, operand.as_ref(), output, lhs)?;

                match operator {
                    UnaryOperator::Neg => {
                        output.push(Bytecode::Neg);
                    }
                    UnaryOperator::Not => {
                        output.push(Bytecode::Not);
                    }
                    _ => unreachable!("try operator"),
                }
            }
            Expression::Parenthesized(expr, _) => {
                self.compile_expression(class_name, partial_class, expr.as_ref(), output, lhs)?;
            }
            Expression::Call { .. } => {
                self.compile_call_expression(class_name, partial_class, expr, output, lhs)?;
            }
            Expression::StaticCall { name, type_args, args, .. } => {

                for (i, arg) in args.iter().enumerate().rev() {
                    self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                    self.bind_variable(format!("arg{i}"));
                }

                for i in 0..args.len() {
                    self.get_variable(format!("arg{}", i));
                    output.push(Bytecode::StoreArgument(i as u8));
                }

                let method_name = name.segments.last().unwrap();
                let method_class = name.segments.iter().rev().skip(1).next().unwrap();
                let method_class = method_class.to_string();

                let mut path = self.add_path_if_needed(method_class);
                let method_class = partial_class.add_string(path.join("::"));
                path.push(method_name.to_string());

                let method_name = partial_class.add_string(path.join("::"));
                
                output.push(Bytecode::InvokeStatic(method_class, method_name));
            }
            Expression::MemberAccess {
                ..
            } => {
                self.compile_member_get(class_name, partial_class, expr, output)?;
            }
            Expression::New(ty, arr_size, _) => {
                if let Some(arr_size) = arr_size {
                    let name = match ty {
                        Type::I8 => TypeTag::I8,
                        Type::U8 => TypeTag::U8,
                        Type::I16 => TypeTag::I16,
                        Type::U16 => TypeTag::U16,
                        Type::I32 => TypeTag::I32,
                        Type::U32 => TypeTag::U32,
                        Type::I64 => TypeTag::I64,
                        Type::U64 => TypeTag::U64,
                        Type::F32 => TypeTag::F32,
                        Type::F64 => TypeTag::F64,
                        Type::Object(name, _) =>  {
                            let path = self.add_path_if_needed(name.to_string());
                            if self.classes.contains_key(&path) {
                                TypeTag::Object
                            } else {
                                *self.current_type_args.get(name.as_str()).unwrap()
                            }
                        },
                        Type::Native => unreachable!("new array can't have native"),
                        _ => TypeTag::Object,
                    };
                    self.compile_expression(class_name, partial_class, arr_size.as_ref(), output, lhs)?;
                    output.push(Bytecode::CreateArray(name));
                } else {
                    let name = match ty {
                        Type::Object(name, _) => name,
                        Type::TypeArg(name, args, _) => {
                            let mut name_string = String::new();
                            let Type::Object(name, _) = name.as_ref() else {
                                unreachable!("only objects can be in a type arg")
                            };
                            name_string.push_str(name);
                            for arg in args {
                                let name_mod = match arg {
                                    Type::I8 | Type::U8 => "8",
                                    Type::I16 | Type::U16 => "16",
                                    Type::I32 | Type::U32 => "32",
                                    Type::I64 | Type::U64 => "64",
                                    Type::F32 => "f32",
                                    Type::F64 => "f64",
                                    Type::Object(ty, _) => {
                                        let path = self.add_path_if_needed(ty.to_string());
                                        if self.classes.contains_key(&path) {
                                            "object"
                                        } else {
                                            match self.current_type_args.get(ty.as_str()).unwrap() {
                                                TypeTag::I8 | TypeTag::U8 => "8",
                                                TypeTag::I16 | TypeTag::U16 => "16",
                                                TypeTag::I32 | TypeTag::U32 => "32",
                                                TypeTag::I64 | TypeTag::U64 => "64",
                                                TypeTag::F32 => "f32",
                                                TypeTag::F64 => "f64",
                                                _ => "object",
                                            }
                                        }
                                    }
                                    _ => "object",
                                };
                                name_string.push_str(name_mod);
                            }
                            &Text::Owned(name_string)
                        }
                        _ => todo!("handle array new")
                    };

                    // This is a nasty hack that shouldn't be here.
                    // However, this only applies to Printer objects for some reason
                    // which will be removed at some point so this is fine.
                    let name = if name.as_str() == "Printer" {
                        vec![String::from("core"), name.to_string()]
                    } else {
                        self.add_path_if_needed(name.to_string())
                    };

                    let string_ref = partial_class.add_string(name.join("::"));

                    output.push(Bytecode::NewObject(string_ref));
                }
            }
            Expression::IfExpression(if_expr, _) => {
                self.compile_if_expression(class_name, partial_class, if_expr, output, lhs)?;
            }
            Expression::Return(value, _) => {
                self.method_returned = true;
                let result = value.as_ref().map(|expr| {
                    self.compile_expression(class_name, partial_class, expr.as_ref(), output, lhs)
                });

                if let Some(result) = result {
                    let _ = result?;
                    output.push(Bytecode::Return)
                } else {
                    output.push(Bytecode::ReturnVoid)
                }
                self.current_block_returned = true;
            }
            Expression::Closure { params, return_type, body, captures, span, .. } => {
                // create closure class with matching types if it doesn't already exist
                // create implementation closure from base closure class
                // if captures is non-empty, call a static function that makes the closure with each of the captures
                // if captures is empty, create the closure object
                let closure_class_name = self.create_closure_class(params, return_type);
                self.compile_closure_expression(
                    class_name,
                    partial_class,
                    closure_class_name,
                    params,
                    return_type,
                    body,
                    captures,
                    *span,
                    output,
                )?;
            }
            _ => todo!("add remaining expressions")
        }
        Ok(None)
    }

    fn compile_if_expression(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass,
        expr: &IfExpression,
        output: &mut Vec<Bytecode>,
        lhs: bool,
    ) -> Result<(), CompilerError> {
        match expr {
            IfExpression { condition, then_branch, else_branch: None, ..} => {
                self.compile_expression(class_name, partial_class, condition.as_ref(), output, lhs)?;
                output.push(Bytecode::If(1, 2));
                self.increment_block();
                self.compile_block(class_name, partial_class, then_branch, output)?;
                self.increment_block();
                if !self.current_block_returned {
                    output.push(Bytecode::Goto(1));
                }
                self.current_block_returned = false;
                let block = self.current_block;
                output.push(Bytecode::StartBlock(block));
            }
            IfExpression { condition, then_branch, else_branch: Some(Either::Right(else_branch)), ..} => {
                self.compile_expression(class_name, partial_class, condition.as_ref(), output, lhs)?;
                output.push(Bytecode::If(1, 2));
                self.increment_block();
                self.compile_block(class_name, partial_class, then_branch, output)?;
                output.push(Bytecode::Goto(2));
                self.increment_block();
                self.compile_block(class_name, partial_class, else_branch, output)?;
                if !self.current_block_returned {
                    output.push(Bytecode::Goto(1));
                }
                self.current_block_returned = false;
                let block = self.current_block;
                output.push(Bytecode::StartBlock(block));
            }
            IfExpression { condition, then_branch, else_branch: Some(Either::Left(else_branch)), ..} => {
                self.compile_expression(class_name, partial_class, condition.as_ref(), output, lhs)?;
                output.push(Bytecode::If(1, 2));
                self.increment_block();
                self.compile_block(class_name, partial_class, then_branch, output)?;
                let mut temp_output = Vec::new();
                let then_block = self.current_block;
                self.compile_if_expression(class_name, partial_class, else_branch.as_ref(), &mut temp_output, lhs)?;
                let escape_block = self.current_block;
                if !self.current_block_returned {
                    output.push(Bytecode::Goto((escape_block - then_block) as i64));
                }
                self.current_block_returned = false;
                output.extend(temp_output);
            }
        }
        Ok(())
    }

    fn compile_member_get<'a>(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass,
        expr: &'a Expression<'a>,
        output: &mut Vec<Bytecode>
    ) -> Result<(), CompilerError> {
        let Expression::MemberAccess {
            object, field, ..
        } = expr else {
            unreachable!("We have already checked for expr being a MemberAccess");
        };

        match object.as_ref() {
            Expression::ClassAccess { class_name, ..} => {
                let class_name = if self.active_imports.contains_key(class_name.segments[0].as_str()) {
                    let mut active_path = self.active_imports.get(class_name.segments[0].as_str()).unwrap().clone();
                    active_path.extend(
                        class_name.segments[1..class_name.segments.len()].iter()
                            .map(ToString::to_string)
                    );
                    active_path
                } else {
                    let path = class_name.segments[..class_name.segments.len()].iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>();

                    if path.len() == 1 {
                        self.add_path_if_needed(path[0].clone())
                    } else {
                        path
                    }
                };
                let path = partial_class.add_string(class_name.join("::"));
                let class = self.classes.get(&class_name).unwrap_or(partial_class);
                let mut field_name = class.get_class_name();
                field_name.push(field.segments.last().unwrap().to_string());
                let (member_index, member_type) = class.get_static_member_offset(&field_name.join("::"));

                output.push(Bytecode::GetStaticMember(path, member_index, member_type));
                return Ok(());
            }
            _ => {}
        }

        self.compile_expression(class_name, partial_class, object.as_ref(), output, false)?;

        let annotation = object.get_type();

        let name = match annotation  {
            Either::Left(Type::Object(name, _)) => self.add_path_if_needed(name.to_string()),
            Either::Left(Type::TypeArg(name, args, _)) => {
                let mut string_name = String::new();
                let Type::Object(name, _) = name.as_ref() else {
                    unreachable!("Type arg should always have an object type");
                };
                string_name.push_str(&name);
                for arg in args {
                    let mod_string = match arg {
                        Type::I8 | Type::U8 => "8",
                        Type::I16 | Type::U16 => "16",
                        Type::I32 | Type::U32 => "32",
                        Type::I64 | Type::U64 => "64",
                        Type::F32 => "f32",
                        Type::F64 => "f64",
                        _ => "object",
                    };
                    string_name.push_str(mod_string);
                }
                self.add_path_if_needed(string_name)
            }
            Either::Right(()) => {
                class_name.clone()
            }
            _ => todo!("report error about method output not being an object {annotation:?}"),
        };

        let class = match self.classes.get(&name) {
            Some(class) => class,
            _ => partial_class,
        };
        let (class_name, parent_name) = if class.contains_field(field.to_string().as_str()) {
            (class.get_class_name(), Vec::new())
        } else {
            let Some((name, parent)) = class.find_class_with_field(self, field.to_string().as_str()) else {
                todo!("report error about being unable to find field")
            };

            (name, parent)
        };
        let (offset, type_tag) = if !parent_name.is_empty() {
            let class = self.classes.get(&parent_name).unwrap();
            class.get_member_offset(field.to_string().as_str())
        } else {
            class.get_member_offset(field.to_string().as_str())
        };
        let class_name = class_name.join("::");

        let class_name = partial_class.add_string(class_name);
        let parent_name = partial_class.add_string(parent_name.join("::"));

        output.push(Bytecode::GetField(class_name, parent_name, offset, type_tag));

        Ok(())
    }

    fn compile_member_set<'a>(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass,
        expr: &'a Expression<'a>,
        output: &mut Vec<Bytecode>
    ) -> Result<Option<Box<dyn Fn(&mut Vec<Bytecode>) + 'a>>, CompilerError> {
        let Expression::MemberAccess {
            object, field, ..
        } = expr else {
            unreachable!("We have already checked for expr being a MemberAccess");
        };

        match object.as_ref() {
            Expression::ClassAccess { class_name, ..} => {
                let class_name = if self.active_imports.contains_key(class_name.segments[0].as_str()) {
                    let mut active_path = self.active_imports.get(class_name.segments[0].as_str()).unwrap().clone();
                    active_path.extend(
                        class_name.segments[1..class_name.segments.len() - 1].iter()
                            .map(ToString::to_string)
                    );
                    active_path
                } else {
                    class_name.segments[..class_name.segments.len() - 1].iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                };
                let path = partial_class.add_string(class_name.join("::"));
                let class = self.classes.get(&class_name).unwrap_or(partial_class);
                let (member_index, member_type) = class.get_static_member_offset(field.segments.last().unwrap().as_str());

                return Ok(Some(Box::new(move |output: &mut Vec<Bytecode>| {
                    output.push(Bytecode::SetStaticMember(path, member_index, member_type));
                })));
            }
            _ => {}
        }

        self.compile_expression(class_name, partial_class, object.as_ref(), output, false)?;

        let annotation = object.get_type();

        let name = match annotation  {
            Either::Left(Type::Object(name, _)) => self.add_path_if_needed(name.to_string()),
            Either::Left(Type::TypeArg(name, args, _)) => {
                let mut string_name = String::new();
                let Type::Object(name, _) = name.as_ref() else {
                    unreachable!("Type arg should always have an object type");
                };
                string_name.push_str(&name);
                for arg in args {
                    let mod_string = match arg {
                        Type::I8 | Type::U8 => "8",
                        Type::I16 | Type::U16 => "16",
                        Type::I32 | Type::U32 => "32",
                        Type::I64 | Type::U64 => "64",
                        Type::F32 => "f32",
                        Type::F64 => "f64",
                        _ => "object",
                    };
                    string_name.push_str(mod_string);
                }
                self.add_path_if_needed(string_name)
            }
            Either::Right(()) => {
                class_name.clone()
            }
            _ => todo!("report error about method output not being an object: {:?} {:?}", object, field),
        };

        let class = self.classes.get(&name).unwrap_or(partial_class);
        let (class_name, parent_name) = if class.contains_field(field.to_string().as_str()) {
            (class.get_class_name(), Vec::new())
        } else {
            let Some((name, parent)) = class.find_class_with_field(self, field.to_string().as_str()) else {
                todo!("report error about being unable to find field")
            };

            (name, parent)
        };
        let (offset, type_tag) = if !parent_name.is_empty() {
            let class = self.classes.get(&parent_name).unwrap();
            class.get_member_offset(field.to_string().as_str())
        } else {
            class.get_member_offset(field.to_string().as_str())
        };
        let class_name = class_name.join("::");
        let parent_name = parent_name.join("::");

        let class_name = partial_class.add_string(class_name);
        let parent_name = partial_class.add_string(parent_name);



        Ok(Some(Box::new(move |output: &mut Vec<Bytecode>| {
            output.push(Bytecode::SetField(class_name, parent_name, offset, type_tag));
        })))
    }

    fn compile_call_expression<'a>(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass,
        expr: &'a Expression<'a>,
        output: &mut Vec<Bytecode>,
        lhs : bool,
    ) -> Result<(), CompilerError> {
        let Expression::Call { name, type_args, args, span, .. } = expr else {
            unreachable!("We have already checked for expr being a Call");
        };
        let (name, ty) = 'setup_args: loop {
            let (name, ty, var): (&PathName, Vec<String>, Text) = match name.as_ref() {
                Expression::MemberAccess { object, field, .. } => {
                    match object.as_ref() {
                        Expression::Variable(var, Type::Object(ty, _), _) => {
                            let path = self.add_path_if_needed(ty.to_string());
                            (field, path, var.clone())
                        }
                        Expression::Variable(var, Type::Array(ty, _), _) => {
                            let ty = match ty.as_ref() {
                                Type::U8 | Type::I8 => Text::Borrowed("Array8"),
                                Type::U16 | Type::I16 => Text::Borrowed("Array16"),
                                Type::U32 | Type::I32 | Type::Char => Text::Borrowed("Array32"),
                                Type::U64 | Type::I64 => Text::Borrowed("Array64"),
                                Type::Str | Type::Function(_, _, _) | Type::Array(_, _) | Type::Void | Type::Tuple(_, _) | Type::TypeArg(_, _, _) => Text::Borrowed("ArrayObject"),
                                Type::Object(ty, _) => {
                                    let path = self.add_path_if_needed(ty.to_string());
                                    if self.classes.contains_key(&path) {
                                        Text::Borrowed("Arrayobject")
                                    } else {
                                        match self.current_type_args.get(ty.as_str()).unwrap() {
                                            TypeTag::I8 | TypeTag::U8 => Text::Borrowed("Array8"),
                                            TypeTag::I16 | TypeTag::U16 => Text::Borrowed("Array16"),
                                            TypeTag::I32 | TypeTag::U32 => Text::Borrowed("Array32"),
                                            TypeTag::I64 | TypeTag::U64 => Text::Borrowed("Array64"),
                                            TypeTag::F32 => Text::Borrowed("Arrayf32"),
                                            TypeTag::F64 => Text::Borrowed("Arrayf64"),
                                            _ => Text::Borrowed("Arrayobject"),
                                        }
                                    }
                                }
                                Type::F32 => Text::Borrowed("Arrayf32"),
                                Type::F64 => Text::Borrowed("Arrayf64"),
                                Type::Native => unreachable!("Native shouldn't be a constructable type in Rowan"),
                            };
                            (field, vec![ty.to_string()], var.clone())
                        }
                        Expression::This(_) => {
                            (field, class_name.clone(), Text::Borrowed("self"))
                        }
                        Expression::Variable(var, Type::TypeArg(obj, args, _), _) => {
                            let Type::Object(ty, _) = obj.as_ref() else {
                                unreachable!("type arg should contain an object");
                            };
                            let mut ty_name = ty.to_string();
                            for arg in args {
                                let modifier = match arg {
                                    Type::U8 | Type::I8 => "8",
                                    Type::U16 | Type::I16 => "16",
                                    Type::U32 | Type::I32 => "32",
                                    Type::U64 | Type::I64 => "64",
                                    Type::F32 => "f32",
                                    Type::F64 => "f64",
                                    Type::Object(ty, _) => {
                                        let path = self.add_path_if_needed(ty.to_string());
                                        if self.classes.contains_key(&path) {
                                            "object"
                                        } else {
                                            match self.current_type_args.get(ty.as_str()).unwrap() {
                                                TypeTag::I8 | TypeTag::U8 => "8",
                                                TypeTag::I16 | TypeTag::U16 => "16",
                                                TypeTag::I32 | TypeTag::U32 => "32",
                                                TypeTag::I64 | TypeTag::U64 => "64",
                                                TypeTag::F32 => "f32",
                                                TypeTag::F64 => "f64",
                                                _ => "object",
                                            }
                                        }
                                    }
                                    _ => "object",
                                };
                                ty_name.push_str(modifier);
                            }
                            (field, self.add_path_if_needed(ty_name), var.clone())
                        }
                        Expression::MemberAccess { annotation, .. } => {
                            self.compile_expression(class_name, partial_class, object.as_ref(), output, lhs)?;
                            for (i, arg) in args.iter().enumerate() {
                                self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                                self.bind_variable(format!("arg{i}"));
                            }

                            for i in 1..=args.len() { // 1..len for leaving space for object
                                self.get_variable(format!("arg{}", i - 1));
                                output.push(Bytecode::StoreArgument(i as u8));
                            }

                            output.push(Bytecode::StoreArgument(0));

                            let annotation = match annotation {
                                Type::Object(name, _) => {
                                    name.clone()
                                }
                                Type::Array(ty, _) => {
                                    let ty = match ty.as_ref() {
                                        Type::U8 | Type::I8 => Text::Borrowed("Array8"),
                                        Type::U16 | Type::I16 => Text::Borrowed("Array16"),
                                        Type::U32 | Type::I32 | Type::Char => Text::Borrowed("Array32"),
                                        Type::U64 | Type::I64 => Text::Borrowed("Array64"),
                                        Type::Str | Type::Function(_, _, _) | Type::Array(_, _) | Type::Void | Type::Tuple(_, _) | Type::TypeArg(_, _, _) => Text::Borrowed("ArrayObject"),
                                        Type::Object(ty, _) => {
                                            let path = self.add_path_if_needed(ty.to_string());
                                            if self.classes.contains_key(&path) {
                                                Text::Borrowed("Arrayobject")
                                            } else {
                                                match self.current_type_args.get(ty.as_str()).unwrap() {
                                                    TypeTag::I8 | TypeTag::U8 => Text::Borrowed("Array8"),
                                                    TypeTag::I16 | TypeTag::U16 => Text::Borrowed("Array16"),
                                                    TypeTag::I32 | TypeTag::U32 => Text::Borrowed("Array32"),
                                                    TypeTag::I64 | TypeTag::U64 => Text::Borrowed("Array64"),
                                                    TypeTag::F32 => Text::Borrowed("Arrayf32"),
                                                    TypeTag::F64 => Text::Borrowed("Arrayf64"),
                                                    _ => Text::Borrowed("Arrayobject"),
                                                }
                                            }
                                        }
                                        Type::F32 => Text::Borrowed("Arrayf32"),
                                        Type::F64 => Text::Borrowed("Arrayf64"),
                                        Type::Native => unreachable!("Native shouldn't be a constructable type in Rowan"),
                                    };
                                    ty
                                }
                                Type::TypeArg(obj, args, _) => {
                                    let Type::Object(ty, _) = obj.as_ref() else {
                                        unreachable!("type arg should contain an object");
                                    };
                                    let mut ty_name = ty.to_string();
                                    for arg in args {
                                        let modifier = match arg {
                                            Type::U8 | Type::I8 => "8",
                                            Type::U16 | Type::I16 => "16",
                                            Type::U32 | Type::I32 => "32",
                                            Type::U64 | Type::I64 => "64",
                                            Type::F32 => "f32",
                                            Type::F64 => "f64",
                                            Type::Object(ty, _) => {
                                                let path = self.add_path_if_needed(ty.to_string());
                                                if self.classes.contains_key(&path) {
                                                    "object"
                                                } else {
                                                    match self.current_type_args.get(ty.as_str()).unwrap() {
                                                        TypeTag::I8 | TypeTag::U8 => "8",
                                                        TypeTag::I16 | TypeTag::U16 => "16",
                                                        TypeTag::I32 | TypeTag::U32 => "32",
                                                        TypeTag::I64 | TypeTag::U64 => "64",
                                                        TypeTag::F32 => "f32",
                                                        TypeTag::F64 => "f64",
                                                        _ => "object",
                                                    }
                                                }
                                            }
                                            _ => "object",
                                        };
                                        ty_name.push_str(modifier);
                                    }
                                    Text::Owned(ty_name)
                                }
                                _ => unreachable!("type arg should contain an object"),
                            };

                            let annotation = self.add_path_if_needed(annotation.to_string());

                            break 'setup_args (field, annotation);
                        }
                        Expression::Variable(value, Type::Function(function_args, return_ty, ..), ..) => {
                            self.compile_expression(class_name, partial_class, object.as_ref(), output, lhs)?;
                            for (i, arg) in args.iter().enumerate() {
                                self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                                self.bind_variable(format!("arg{i}"));
                            }

                            for i in 1..=args.len() { // 1..len for leaving space for object
                                self.get_variable(format!("arg{}", i - 1));
                                output.push(Bytecode::StoreArgument(i as u8));
                            }

                            output.push(Bytecode::StoreArgument(0));

                            let mut closure_name = String::from("Closure");

                            for arg in function_args {
                                match arg {
                                    Type::U8 => closure_name.push_str("u8"),
                                    Type::I8 => closure_name.push_str("i8"),
                                    Type::U16 => closure_name.push_str("u16"),
                                    Type::I16 => closure_name.push_str("i16"),
                                    Type::U32 => closure_name.push_str("u32"),
                                    Type::I32 => closure_name.push_str("i32"),
                                    Type::U64 => closure_name.push_str("u64"),
                                    Type::I64 => closure_name.push_str("i64"),
                                    Type::F32 => closure_name.push_str("f32"),
                                    Type::F64 => closure_name.push_str("f64"),
                                    _ => closure_name.push_str("object"),
                                }
                            }

                            match return_ty.as_ref() {
                                Type::U8 => closure_name.push_str("u8"),
                                Type::I8 => closure_name.push_str("i8"),
                                Type::U16 => closure_name.push_str("u16"),
                                Type::I16 => closure_name.push_str("i16"),
                                Type::U32 => closure_name.push_str("u32"),
                                Type::I32 => closure_name.push_str("i32"),
                                Type::U64 => closure_name.push_str("u64"),
                                Type::I64 => closure_name.push_str("i64"),
                                Type::F32 => closure_name.push_str("f32"),
                                Type::F64 => closure_name.push_str("f64"),
                                Type::Void => closure_name.push_str("void"),
                                _ => closure_name.push_str("object"),
                            }

                            let annotation = vec![
                                String::from("std"),
                                String::from("closure"),
                                closure_name,
                            ];

                            break 'setup_args (field, annotation)
                        }
                        x => todo!("add additional sources to call from {:?}", x)
                    }
                }
                _ => unreachable!("all calls should be via member access by this point")
            };

            // Rev is used because otherwise arguments are not loaded from left to right
            for (i, arg) in args.iter().enumerate().rev() {
                self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                self.bind_variable(format!("arg{i}"));
            }

            for i in 1..=args.len() { // 1..len for leaving space for object
                self.get_variable(format!("arg{}", i - 1));
                output.push(Bytecode::StoreArgument(i as u8));
            }

            let object = self.get_variable(var).expect("There should be method calling by this point");
            output.push(Bytecode::LoadLocal(object));
            output.push(Bytecode::StoreArgument(0));
            break (name, ty);
        };

        let name = name.segments.last().unwrap();

        if name.as_str() == "downcast" || name.as_str() == "downcast-contents" {
            assert_eq!(type_args.len(), 1, "Downcast only takes one type argument");
            let ty = match type_args.first().unwrap() {
                Type::Array(ty, _) => {
                    match ty.as_ref() {
                        Type::U8 | Type::I8 => Text::Borrowed("Array8"),
                        Type::U16 | Type::I16 => Text::Borrowed("Array16"),
                        Type::U32 | Type::I32 | Type::Char => Text::Borrowed("Array32"),
                        Type::U64 | Type::I64 => Text::Borrowed("Array64"),
                        Type::Str | Type::Function(_, _, _) | Type::Array(_, _) | Type::Void | Type::Tuple(_, _) | Type::TypeArg(_, _, _) => Text::Borrowed("ArrayObject"),
                        Type::F32 => Text::Borrowed("Arrayf32"),
                        Type::F64 => Text::Borrowed("Arrayf64"),
                        Type::Object(ty, _) => {
                            let path = self.add_path_if_needed(ty.to_string());
                            if self.classes.contains_key(&path) {
                                Text::Borrowed("Arrayobject")
                            } else {
                                match self.current_type_args.get(ty.as_str()).unwrap() {
                                    TypeTag::I8 | TypeTag::U8 => Text::Borrowed("Array8"),
                                    TypeTag::I16 | TypeTag::U16 => Text::Borrowed("Array16"),
                                    TypeTag::I32 | TypeTag::U32 => Text::Borrowed("Array32"),
                                    TypeTag::I64 | TypeTag::U64 => Text::Borrowed("Array64"),
                                    TypeTag::F32 => Text::Borrowed("Arrayf32"),
                                    TypeTag::F64 => Text::Borrowed("Arrayf64"),
                                    _ => Text::Borrowed("Arrayobject"),
                                }
                            }
                        }
                        Type::Native => unreachable!("Native shouldn't be a constructable type in Rowan"),
                    }
                }
                Type::Object(name, _) => name.clone(),
                _ => unreachable!("downcast can only take type arguments that are Objects or Arrays, not Tuples or primitives like integers and floats"),
            };

            let path = self.add_path_if_needed(ty.to_string());

            let class_symbol = partial_class.add_string(path.join("::"));

            output.push(Bytecode::LoadSymbol(class_symbol));
            output.push(Bytecode::StoreArgument(args.len() as u8));
        }

        if ty == *class_name {

            let mut method_name = class_name.clone();
            method_name.push(name.to_string());

            let name = method_name.join("::");

            let vtable = partial_class.get_vtable(&name).expect("add proper handling of missing vtable").clone();
            let method_entry = partial_class.get_method_entry(&name).expect("add proper handling of missing method");

            //println!("{}", partial_class.index_string_table(vtable.class_name));

            let class_name = partial_class.index_string_table(vtable.class_name);
            let class_name = class_name.to_string();
            let vtable_class_name = partial_class.add_string(class_name);

            let method_name = partial_class.index_string_table(method_entry.name);
            let method_name = method_name.to_string();
            let method_name = partial_class.add_string(method_name);

            output.push(Bytecode::InvokeVirt(vtable_class_name, method_name));
        }
        else if let Some(class) = self.classes.get(&ty) {
            //println!("{:#?}", class);
            let class_name_path = class.get_class_name();
            let mut field_path = self.add_path_if_needed(class_name_path.join("::"));
            field_path.push(name.to_string());
            let vtable = class.get_vtable(field_path.join("::")).expect("add proper handling of missing vtable");
            let method_entry = class.get_method_entry(field_path.join("::")).expect("add proper handling of missing method");

            //println!("{}", class.index_string_table(vtable.class_name));

            let class_name = class.index_string_table(vtable.class_name);
            let vtable_class_name = partial_class.add_string(class_name);

            let method_name = class.index_string_table(method_entry.name);
            let method_name = partial_class.add_string(method_name);


            output.push(Bytecode::InvokeVirt(vtable_class_name, method_name));
        } else {
            panic!("Classes are in a bad order of compiling")
        }
        Ok(())
    }

    fn compile_closure_expression<'a>(
        &mut self,
        class_name: &Vec<String>,
        partial_class: &mut PartialClass,
        closure_name: String,
        params: &[ClosureParameter<'a>],
        return_type: &Type,
        body: &Vec<Statement<'a>>,
        captures: &[(Text<'a>, Type<'a>)],
        span: Span,
        output: &mut Vec<Bytecode>,
    ) -> Result<(), CompilerError> {
        let path = class_name[0..(class_name.len() - 1)].join("::");
        let closure_number = *self.closures_under_path.entry(path.clone())
            .and_modify(|c| *c += 1)
            .or_insert(0);

        /*let mut class_name = class_name[0..(class_name.len() - 1)].to_vec();
        class_name.push(format!("Closure{closure_number}"));*/
        let class_name = format!("Closure{closure_number}");

        /*partial_class.set_name(&class_name.join("::"));*/

        let mut method_params = Vec::with_capacity(params.len() + 1);
        method_params.push(Parameter::This(false, Span::new(0, 0)));
        for param in params {
            let ClosureParameter { parameter } = param;
            method_params.push(parameter.clone());
        }

        let body = if captures.is_empty() {
            let mut new_body = Vec::new();
            for (capture, ty) in captures {
                let field = PathName::new(vec![capture.clone()], Span::new(0,0));
                let value = Expression::MemberAccess {
                    object: Box::new(Expression::This(Span::new(0,0))),
                    field,
                    span: Span::new(0,0),
                    annotation: ty.clone(),
                };

                new_body.push(Statement::Let {
                    bindings: Pattern::Variable(capture.clone(), false, Span::new(0, 0)),
                    ty: ty.clone(),
                    value,
                    span,
                })
            }
            new_body.append(&mut body.clone());
            new_body
        } else {
            body.clone()
        };


        let call_method = Method {
            name: Text::Owned(format!("std::closure::{}::call", closure_name.clone())),
            is_native: false,
            annotations: vec![Annotation::new(Text::Borrowed("Override"), Vec::new(), Span::new(0, 0))],
            visibility: Visibility::Public,
            type_params: Vec::new(),
            parameters: method_params,
            return_type: return_type.clone(),
            body,
            span,
        };

        let mut methods = vec![call_method];

        let members = if captures.is_empty() {
            Vec::new()
        } else {
            // If there are captures, we create a static method to initialize the closure
            // We also give the members to the closure
            let mut body = Vec::new();
            body.push(Statement::Let {
                // using a name that will not conflict due to its invalid nature
                bindings: Pattern::Variable(Text::Borrowed("11037"), false, Span::new(0, 0)),
                ty: Type::Object(Text::Owned(closure_name.clone()), Span::new(0, 0)),
                value: Expression::New(Type::Object(Text::Owned(class_name.clone()), Span::new(0,0)), None, Span::new(0,0)),
                span: Span::new(0, 0),
            });

            let mut members = Vec::with_capacity(captures.len());
            let mut params = Vec::with_capacity(captures.len());
            for (capture, ty) in captures {
                params.push(Parameter::Pattern {
                    name: Pattern::Variable(capture.clone(), false, Span::new(0,0)),
                    ty: ty.clone(),
                    span: Span::new(0, 0),
                });

                body.push(Statement::Assignment {
                    target: Expression::MemberAccess {
                        object: Box::new(Expression::This(Span::new(0,0))),
                        field: PathName::new(vec![capture.clone()], Span::new(0,0)),
                        span: Span::new(0,0),
                        annotation: ty.clone(),
                    },
                    value: Expression::Variable(capture.clone(), ty.clone() ,Span::new(0,0)),
                    span: Span::new(0, 0),
                });

                let member = ir::Member {
                    visibility: Visibility::Public,
                    name: capture.clone(),
                    ty: ty.clone(),
                    span: Span::new(0, 0),
                };
                members.push(member);
            }

            body.push(Statement::Expression(
                Expression::Return(
                    Some(Box::new(
                        Expression::Variable(
                            Text::Borrowed("11037"),
                            Type::Object(Text::Owned(class_name.clone()),
                                         Span::new(0,0)
                            ), Span::new(0,0)))
                    ),
                    Span::new(0, 0),
                ),
                Span::new(0, 0),
            ));
            let create_method = Method {
                name: Text::Borrowed("create"),
                is_native: false,
                annotations: vec![],
                visibility: Visibility::Public,
                type_params: Vec::new(),
                parameters: params,
                return_type: Type::Object(Text::Owned(class_name.clone()), Span::new(0,0)),
                body,
                span,
            };

            methods.push(create_method);

            members
        };



        let class = Class {
            name: Text::Owned(format!("Closure{closure_number}")),
            parent: Some(ParentDec {
                name: Text::Owned(closure_name.clone()),
                type_args: Vec::new(),
                type_params: Vec::new(),
                span: Span::new(0, 0),
            }),
            members,
            methods,
            static_members: vec![],
            type_params: vec![],
            span,
        };

        self.compile_class(class)?;

        let path = self.add_path_if_needed(format!("Closure{closure_number}"));

        if captures.is_empty() {
            let index = partial_class.add_string(path.join("::"));
            output.push(
                Bytecode::NewObject(index)
            );
        } else {
            let mut path = path;
            let class_index = partial_class.add_string(path.join("::"));
            path.push(String::from("create"));
            let method_index = partial_class.add_string(path.join("::"));
            output.push(
                Bytecode::InvokeStatic(class_index, method_index)
            );
        }


        Ok(())
    }
}
