use std::{collections::HashMap, io::Write, path::PathBuf};
use std::cmp::Ordering;
use either::Either;
use itertools::Itertools;
use rowan_shared::{bytecode::compiled::Bytecode, classfile::{Member, SignatureEntry, VTable, VTableEntry}, TypeTag};
use rowan_shared::classfile::SignatureIndex;
use crate::{ast, ast::{BinaryOperator, Class, Constant, Expression, File, Literal, Method, Parameter, Pattern, Statement, TopLevelStatement, Type, UnaryOperator, Text}, backend::compiler_utils::Frame};
use crate::ast::{IfExpression, ParentDec, PathName};
use super::compiler_utils::{PartialClass, StaticMember};



fn create_stdlib() -> HashMap<Vec<String>, PartialClass> {
    let mut classes = HashMap::new();

    let mut object = PartialClass::new();
    object.set_name("Object");
    let functions = vec![
        VTableEntry::default(),
    ];
    let names = vec![
        "downcast",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    object.add_vtable(&vec![String::from("Object")], vtable, &names, &signatures, &vec![false]);
    object.make_not_printable();
    classes.insert(vec![String::from("Object")], object);

    let mut printer = PartialClass::new();
    printer.set_name("Printer");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "println-int",
        "println-float",
        "println",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::F64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    
    printer.add_vtable(&vec![String::from("Printer")], vtable, &names, &signatures, &vec![false, false, false]);
    printer.make_not_printable();
    classes.insert(vec![String::from("Printer")], printer);

    
    let mut string = PartialClass::new();
    string.set_name("String");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    string.add_vtable(&vec![String::from("String")], vtable, &names, &signatures, &vec![false, false, false]);
    string.make_not_printable();
    classes.insert(vec![String::from("String")], string);

    
    let mut array = PartialClass::new();
    array.set_name("Array8");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Array8")], vtable, &names, &signatures, &vec![false, false, false]);
    array.make_not_printable();
    classes.insert(vec![String::from("Array8")], array);

    let mut array = PartialClass::new();
    array.set_name("Array16");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Array16")], vtable, &names, &signatures, &vec![false, false, false]);
    array.make_not_printable();
    classes.insert(vec![String::from("Array16")], array);

    let mut array = PartialClass::new();
    array.set_name("Array32");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Array32")], vtable, &names, &signatures, &vec![false, false, false]);
    array.make_not_printable();
    classes.insert(vec![String::from("Array32")], array);

    let mut array = PartialClass::new();
    array.set_name("Array64");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Array64")], vtable, &names, &signatures, &vec![false, false, false]);
    array.make_not_printable();
    classes.insert(vec![String::from("Array64")], array);

    let mut array = PartialClass::new();
    array.set_name("Arrayf32");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Arrayf32")], vtable, &names, &signatures, &vec![false, false, false]);
    array.make_not_printable();
    classes.insert(vec![String::from("Arrayf32")], array);

    let mut array = PartialClass::new();
    array.set_name("Arrayf64");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Arrayf64")], vtable, &names, &signatures, &vec![false, false, false]);
    array.make_not_printable();
    classes.insert(vec![String::from("Arrayf64")], array);

    let mut array = PartialClass::new();
    array.set_name("ArrayObject");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "load-str",
        "init",
        "len",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable(&vec![String::from("Arrayobject")], vtable, &names, &signatures, &vec![false, false, false]);
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
            parents,
            members,
            methods,
            static_members,
            ..
        } = class;

        let class_name = self.add_path_if_needed(name.to_string());

        if type_params.is_empty() {
            let new_parents = self.create_new_parents(&parents);
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

                let new_parents = self.create_new_parents(&parents);

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

    fn create_new_parents<'a>(&mut self, parents: &'a Vec<ParentDec<'a>>) -> Vec<ParentDec<'a>> {
        parents.into_iter().map(|parent| {
            let mut string = parent.name.to_string();
            for type_arg in parent.type_args.iter() {
                let str_value = match type_arg {
                    Type::I8 | Type::U8 => "8",
                    Type::I16 | Type::I16 => "16",
                    Type::I32 | Type::I32 => "32",
                    Type::I64 | Type::I64 => "64",
                    Type::F32 => "f32",
                    Type::F64 => "f64",
                    Type::Object(name, _) => {
                        if self.current_type_args.contains_key(name.as_str()) {
                            match type_arg {
                                Type::I8 | Type::U8 => "8",
                                Type::I16 | Type::I16 => "16",
                                Type::I32 | Type::I32 => "32",
                                Type::I64 | Type::I64 => "64",
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
        }).collect()
    }

    fn compile_class_inner(
        &mut self,
        name: &Vec<String>,
        parents: &Vec<ParentDec>,
        methods: &Vec<Method>,
        members: &Vec<ast::Member>,
        static_members: &Vec<ast::StaticMember>,
    ) -> Result<(), CompilerError> {
        let mut partial_class = PartialClass::new();
        let path_name = name.join("::");
        partial_class.set_name(&path_name);
        let class_name = name;

        let parent_vtables = parents.iter().map(|parent_name| {
            let path = self.add_path_if_needed(parent_name.name.clone().to_string());
            let partial_class = self.classes.get(&path).expect("Order of files is wrong");
            let vtables = partial_class.get_vtables(&path);
            vtables.into_iter().map(|(table, names, signatures)| {
                let mut is_natives= Vec::new();
                let class_name = partial_class.index_string_table(table.class_name).split("::")
                    .map(|name| {
                        is_natives.push(false);
                        name.to_string()
                    }).collect::<Vec<String>>();
                let source_class = if table.sub_class_name == 0 {
                    None
                } else {
                    Some(partial_class.index_string_table(table.sub_class_name))
                };
                (class_name, source_class, table, names, signatures, is_natives)
            }).collect::<Vec<_>>()

        });
        parents.iter().for_each(|parent| {
            let path = self.add_path_if_needed(parent.name.clone().to_string()).join("::");
            partial_class.add_parent(&path);
        });

        let (
            vtable,
            names,
            signatures,
            static_method_map,
            static_signatures,
            is_natives
        ) = self.construct_vtable(&name, &methods)?;

        let names = names.into_iter()
            .map(|name| {
                let mut class_name = class_name.clone();
                class_name.push(name);
                class_name.join("::")
            })
            .collect::<Vec<_>>();

        partial_class.add_signatures(static_signatures);

        partial_class.set_static_method_to_sig(static_method_map);

        if vtable.functions.len() != 0 {
            partial_class.add_vtable(&name, vtable, &names, &signatures, &is_natives);
        } else {
            drop(vtable);
            drop(names);
            drop(signatures);
        }

        if parents.len() == 0 {
            let object_class = self.classes.get(&vec!["Object".to_string()]).expect("Object not added to known classes");

            let vtables = object_class.get_vtables(&[String::from("Object")]);
            let (vtable, names, signatures) = &vtables[0];
            let names = names.iter()
                .map(|n| format!("core::Object::{n}"))
                .collect::<Vec<String>>();

            partial_class.add_vtable(&vec![String::from("core::Object")], vtable.clone(), &names, signatures, &vec![false]);
            partial_class.add_parent("core::Object");
        }

        for vtables in parent_vtables {
            for (class_name, _source_class, vtable, names, signatures, is_natives) in vtables {
                let names = names.into_iter()
                    .map(|n| self.add_path_if_needed(n).join("::"))
                    .collect::<Vec<String>>();
                partial_class.add_vtable(&class_name, vtable.clone(), &names, &signatures, &is_natives);
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
        Vec<bool>,
    ), CompilerError> {


        let mut entries = Vec::new();
        let mut names = Vec::new();
        let mut signatures = Vec::new();
        let mut static_signatures = Vec::new();
        let mut static_method_to_signature = HashMap::new();
        let mut is_natives = Vec::new();

        'methods: for method in methods.iter() {
            let Method {
                name,
                annotations,
                parameters,
                return_type,
                span: _span,
                is_native,
                ..
            } = method;
            is_natives.push(*is_native);

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
                let mut class_name = class_name.clone();
                class_name.push(name.to_string());
                let name = class_name.join("::");
                names.push(name);
                entries.push(VTableEntry::default());
                signatures.push(SignatureEntry::new(signature));
            }
        }
        let vtable = VTable::new(entries);

        Ok((vtable, names, signatures, static_method_to_signature, static_signatures, is_natives))
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
                if !*is_native {
                    partial_class.add_static_method(name, bytecode);
                }
            } else {
                //println!("{}", name);
                let vtable = partial_class.get_vtable(&name).unwrap();
                let method_class_name = partial_class.index_string_table(vtable.class_name).split("::")
                    .map(|name| name.to_string())
                    .collect::<Vec<String>>();
                //println!("{}", method_class_name);

                let mut method_name = method_class_name.clone();
                method_name.push(name.to_string());
                let method_name = method_name.join("::");

                if !*is_native {
                    partial_class.attach_bytecode(&method_class_name, method_name, bytecode).expect("Handle partial class error");
                }
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
                Statement::Expression(expr, span) => {
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
                                    Some(Type::F32) => {
                                        let value = value.parse::<f32>().expect("malformed f32");
                                        output.push(Bytecode::LoadF32(value));
                                    }
                                    Some(Type::F64) => {
                                        let value = value.parse::<f64>().expect("malformed f64");
                                        output.push(Bytecode::LoadF64(value));
                                    }
                                    _ => unreachable!("float literal"),
                                }
                            }
                            Constant::Integer(value, ty, _) => {
                                match ty {
                                    Some(Type::F32) => {
                                        let value = value.parse::<f32>().expect("malformed f32");
                                        output.push(Bytecode::LoadF32(value));
                                    }
                                    Some(Type::F64) => {
                                        let value = value.parse::<f64>().expect("malformed f64");
                                        output.push(Bytecode::LoadF64(value));
                                    }
                                    Some(Type::U8) => {
                                        let value = value.parse::<u8>().expect("malformed u8");
                                        output.push(Bytecode::LoadU8(value));
                                    }
                                    Some(Type::U16) => {
                                        let value = value.parse::<u16>().expect("malformed u16");
                                        output.push(Bytecode::LoadU16(value));
                                    }
                                    Some(Type::U32) => {
                                        let value = value.parse::<u32>().expect("malformed u32");
                                        output.push(Bytecode::LoadU32(value));
                                    }
                                    Some(Type::U64) => {
                                        let value = value.parse::<u64>().expect("malformed u64");
                                        output.push(Bytecode::LoadU64(value));
                                    }
                                    Some(Type::I8) => {
                                        let value = value.parse::<i8>().expect("malformed i8");
                                        output.push(Bytecode::LoadI8(value));
                                    }
                                    Some(Type::I16) => {
                                        let value = value.parse::<i16>().expect("malformed i16");
                                        output.push(Bytecode::LoadI16(value));
                                    }
                                    Some(Type::I32) => {
                                        let value = value.parse::<i32>().expect("malformed i32");
                                        output.push(Bytecode::LoadI32(value));
                                    }
                                    Some(Type::I64) => {
                                        let value = value.parse::<i64>().expect("malformed i64");
                                        output.push(Bytecode::LoadI64(value));
                                    }
                                    None => {
                                        let value = value.parse::<i32>().expect("malformed i32");
                                        output.push(Bytecode::LoadI32(value));
                                    }
                                    x => unreachable!("integer literal {:?}", x)
                                }
                            }
                        }
                    }
                    Literal::Array(exprs, ty, _) => {
                        let type_tag = match ty {
                            Some(Type::U8) => TypeTag::U8,
                            Some(Type::I8) => TypeTag::I8,
                            Some(Type::U16) => TypeTag::U16,
                            Some(Type::I16) => TypeTag::I16,
                            Some(Type::U32) => TypeTag::U32,
                            Some(Type::I32) => TypeTag::I32,
                            Some(Type::U64) => TypeTag::U64,
                            Some(Type::F32) => TypeTag::F32,
                            Some(Type::F64) => TypeTag::F64,
                            Some(Type::I64) => TypeTag::I64,
                            Some(Type::Array(_, _)) => TypeTag::Object,
                            Some(Type::Tuple(_, _)) => TypeTag::Object,
                            Some(Type::Char) => TypeTag::U32,
                            Some(Type::Str) => TypeTag::Str,
                            Some(Type::Object(_, _)) => TypeTag::Object,
                            Some(Type::Void) => TypeTag::Void,
                            Some(Type::TypeArg(_ ,_, _)) => TypeTag::Object,
                            Some(Type::Function(_, _, _)) => TypeTag::Object,
                            Some(Type::Native) => unreachable!("Native should not ever occur in here"),
                            None => todo!("handle case where we don't know what the type is for the array")
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
                    (Some(Either::Left(lhs)), BinaryOperator::Add, Some(Either::Left(rhs))) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::AddInt)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Sub, Some(Either::Left(rhs))) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::SubInt)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Mul, Some(Either::Left(rhs))) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::MulInt)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Div, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::DivUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Div, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::DivSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Mod, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::ModUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Mod, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::ModSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Add, Some(Either::Left(rhs))) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::AddFloat)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Sub, Some(Either::Left(rhs))) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::SubFloat)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Mul, Some(Either::Left(rhs))) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::MulFloat)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Div, Some(Either::Left(rhs))) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::DivFloat)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Mod, Some(Either::Left(rhs))) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::ModFloat)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Eq, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::EqualUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Eq, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::EqualSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Ne, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::NotEqualUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Ne, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::NotEqualSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Lt, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::LessUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Lt, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::LessSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Le, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::LessOrEqualUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Le, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::LessOrEqualSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Gt, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::GreaterUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Gt, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::GreaterSigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Ge, Some(Either::Left(rhs))) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::GreaterOrEqualUnsigned)
                    }
                    (Some(Either::Left(lhs)), BinaryOperator::Ge, Some(Either::Left(rhs))) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::GreaterOrEqualSigned)
                    }
                    (Some(Either::Left(Type::U8)), BinaryOperator::And, Some(Either::Left(Type::U8))) => {
                        output.push(Bytecode::And)
                    }
                    (Some(Either::Left(Type::U8)), BinaryOperator::Or, Some(Either::Left(Type::U8))) => {
                        output.push(Bytecode::Or)
                    }
                    (Some(Either::Left(Type::Array(generic, _))), BinaryOperator::Index, _) => {
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
            Expression::StaticCall { name, type_args, args, annotation, .. } => {

                for (i, arg) in args.iter().enumerate() {
                    self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                    self.bind_variable(format!("arg{i}"));
                }

                for i in 0..args.len() {
                    self.get_variable(format!("arg{}", i));
                    output.push(Bytecode::StoreArgument(i as u8));
                }

                let method_name = name.segments.last().unwrap();
                let method_class = name.segments.iter().rev().skip(1).next().unwrap();
                let method_class = match annotation {
                    Some(Type::TypeArg(_, args, _)) => {
                        let mut string = method_class.to_string();
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
                            string.push_str(name_mod);
                        }
                        string
                    }
                    _ => {
                        method_class.to_string()
                    }
                };

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

                    let name = self.add_path_if_needed(name.to_string());

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
                    class_name.segments[..class_name.segments.len()].iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                };
                let path = partial_class.add_string(class_name.join("::"));
                let class = self.classes.get(&class_name).unwrap_or(partial_class);
                let (member_index, member_type) = class.get_static_member_offset(field.segments.last().unwrap().as_str());

                output.push(Bytecode::GetStaticMember(path, member_index, member_type));
                return Ok(());
            }
            _ => {}
        }

        self.compile_expression(class_name, partial_class, object.as_ref(), output, false)?;

        let Some(annotation) = object.get_type() else {
            unreachable!("Expression should be annotated by this point");
        };

        let name = match annotation  {
            Either::Left(Type::Object(name, _)) => self.add_path_if_needed(name.to_string()),
            Either::Right(()) => {
                class_name.clone()
            }
            _ => todo!("report error about method output not being an object"),
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

        let Some(annotation) = object.get_type() else {
            unreachable!("Expression should be annotated by this point");
        };

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
                        Expression::Variable(var, Some(Type::Object(ty, _)), _) => {
                            let path = self.add_path_if_needed(ty.to_string());
                            (field, path, var.clone())
                        }
                        Expression::Variable(var, Some(Type::Array(ty, _)), _) => {
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
                        Expression::Variable(var, Some(Type::TypeArg(obj, args, _)), _) => {
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

                            let annotation = match annotation.as_ref() {
                                Some(Type::Object(name, _)) => {
                                    name.clone()
                                }
                                Some(Type::Array(ty, _)) => {
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
                                Some(Type::TypeArg(obj, args, _)) => {
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
                        x => todo!("add additional sources to call from {:?}", x)
                    }
                }
                _ => unreachable!("all calls should be via member access by this point")
            };

            for (i, arg) in args.iter().enumerate() {
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
            let vtable = partial_class.get_vtable(name).expect("add proper handling of missing vtable").clone();
            let method_entry = partial_class.get_method_entry(name).expect("add proper handling of missing method");

            //println!("{}", partial_class.index_string_table(vtable.class_name));
            let mut path = class_name.clone();

            let class_name = partial_class.index_string_table(vtable.class_name);
            let class_name = class_name.to_string();
            let vtable_class_name = partial_class.add_string(class_name);

            let source_class = if vtable.sub_class_name == 0 {
                0
            } else {
                let class_name = partial_class.index_string_table(vtable.sub_class_name);
                let class_name = class_name.to_string();
                partial_class.add_string(class_name)
            };

            let method_name = partial_class.index_string_table(method_entry.name);
            let method_name = method_name.to_string();
            path.push(method_name);
            let method_name = partial_class.add_string(path.join("::"));

            output.push(Bytecode::InvokeVirt(vtable_class_name, source_class, method_name));
        }
        else if let Some(class) = self.classes.get(&ty) {
            //println!("{:#?}", class);
            let mut class_name_path = class.get_class_name();
            //println!("class name: {class_name}");
            let vtable = class.get_vtable(name).expect("add proper handling of missing vtable");
            let method_entry = class.get_method_entry(name).expect("add proper handling of missing method");

            //println!("{}", class.index_string_table(vtable.class_name));

            let class_name = class.index_string_table(vtable.class_name);
            let vtable_class_name = partial_class.add_string(class_name);

            let source_class = if vtable.sub_class_name == 0 {
                0
            } else {
                let class_name = class.index_string_table(vtable.sub_class_name);
                partial_class.add_string(class_name)
            };

            let method_name = class.index_string_table(method_entry.name);
            class_name_path.push(method_name.to_string());
            let method_name = partial_class.add_string(class_name_path.join("::"));


            output.push(Bytecode::InvokeVirt(vtable_class_name, source_class, method_name));
        } else {
            panic!("Classes are in a bad order of compiling")
        }
        Ok(())
    }
}
