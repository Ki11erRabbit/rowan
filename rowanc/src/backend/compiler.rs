use std::{collections::HashMap, io::Write, path::{Path, PathBuf}};
use either::Either;
use rowan_shared::{bytecode::compiled::Bytecode, classfile::{Member, Signal, SignatureEntry, VTable, VTableEntry}, TypeTag};

use crate::{ast::{BinaryOperator, Class, Constant, Expression, File, Literal, Method, Parameter, Pattern, Statement, TopLevelStatement, Type, UnaryOperator, Text}, backend::compiler_utils::Frame};
use crate::ast::IfExpression;
use super::compiler_utils::{PartialClass, PartialClassError};



fn create_stdlib() -> HashMap<String, PartialClass> {
    let mut classes = HashMap::new();

    let mut object = PartialClass::new();
    object.set_name("Object");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let names = vec![
        "tick",
        "ready",
        "upcast",
        "get-child",
        "remove-child",
    ];
    let responds_to = vec![
        "",
        "",
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::F64]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::Object]),
        SignatureEntry::new(vec![TypeTag::Object, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    object.add_vtable("Object", vtable, &names, &responds_to, &signatures);
    object.make_not_printable();
    classes.insert(String::from("Object"), object);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::F64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    
    printer.add_vtable("Printer", vtable, &names, &responds_to, &signatures);
    printer.make_not_printable();
    classes.insert(String::from("Printer"), printer);

    
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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    string.add_vtable("String", vtable, &names, &responds_to, &signatures);
    string.make_not_printable();
    classes.insert(String::from("String"), string);

    
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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array8", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("Array8"), array);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array16", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("Array16"), array);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array32", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("Array32"), array);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array64", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("Array64"), array);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Arrayf32", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("Arrayf32"), array);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Arrayf64", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("Arrayf64"), array);

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
    let responds_to = vec![
        "",
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("ArrayObject", vtable, &names, &responds_to, &signatures);
    array.make_not_printable();
    classes.insert(String::from("ArrayObject"), array);
    
    classes
}

#[derive(Debug)]
pub enum CompilerError {
    UnboundIdentifer(String, usize, usize),
    MalformedCharacter(String, usize, usize),
    
}


pub struct Compiler {
    scopes: Vec<Frame>,
    classes: HashMap<String, PartialClass>,
    current_block: u64,
}


impl Compiler {

    pub fn new() -> Compiler {

        
        Compiler {
            scopes: Vec::new(),
            classes: create_stdlib(),
            current_block: 0,
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

    /// files should be sorted in a way that means that means we don't need to do each file incrementally
    pub fn compile_files(mut self, files: Vec<File>) -> Result<(), CompilerError> {

        for file in files {
            let File { content, .. } = file;

            for statement in content {
                let TopLevelStatement::Class(class) = statement else {
                    unreachable!("Non classes should have been removed by this point");
                };

                self.compile_class(class)?;
            }
        }

        for (path, file) in self.classes.into_iter() {
            if let Some(file) = file.create_class_file() {
                let path = format!("output/{}.class", path);
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
            parents,
            members,
            methods,
            signals,
            ..
        } = class;

        let mut partial_class = PartialClass::new();
        partial_class.set_name(&name);

        let parent_vtables = parents.iter().map(|parent_name| {
            let partial_class = self.classes.get(&parent_name.name.clone().to_string()).expect("Order of files is wrong");
            let vtables = partial_class.get_vtables(&parent_name.name);
            vtables.into_iter().map(|(table, names, responds_to, signatures)| {
                let class_name = partial_class.index_string_table(table.class_name);
                let source_class = if table.sub_class_name == 0 {
                    None
                } else {
                    Some(partial_class.index_string_table(table.sub_class_name))
                };
                (class_name, source_class, table, names, responds_to, signatures)
            }).collect::<Vec<_>>()
            
        });
        parents.iter().for_each(|parent| {
            partial_class.add_parent(&parent.name);
        });

        let (vtable, names, responds_to, signatures) = self.construct_vtable(&name, &methods, &mut partial_class)?;

        if vtable.functions.len() != 0 {
            partial_class.add_vtable(&name, vtable, &names, &responds_to, &signatures);
        } else {
            drop(vtable);
            drop(names);
            drop(responds_to);
            drop(signatures);
        }

        if parents.len() == 0 {
            let object_class = self.classes.get("Object").expect("Object not added to known classes");

            let vtables = object_class.get_vtables("Object");
            let (vtable, names, responds_to, signatures) = &vtables[0];

            partial_class.add_vtable("Object", vtable.clone(), names, responds_to, signatures);
            partial_class.add_parent("Object");
        }
        
        for vtables in parent_vtables {
            for (class_name, _source_class, vtable, names, responds_to, signatures) in vtables {
                partial_class.add_vtable(class_name, vtable.clone(), &names, &responds_to, &signatures);
            }
        }

        members.into_iter().map(|member| {
            (member.name, Member::new(self.convert_type(&member.ty)))
        }).for_each(|(name, member)| {
            partial_class.add_member(member, name);
        });

        signals.into_iter().map(|sig| {
            let parameters = sig.parameters.into_iter().map(|ty| {
                self.convert_type(&ty)
            });
            (sig.name, Signal::new(sig.is_static), parameters)
        }).for_each(|(name, signal, param)| {
            let mut signature = vec![TypeTag::Void];
            signature.extend(param);
            partial_class.add_signal(signal, name, SignatureEntry::new(signature));
        });

        self.compile_methods(&name, &mut partial_class, methods)?;
        
        self.classes.insert(name.to_string(), partial_class);
        
        Ok(())
    }

    fn construct_vtable(&self, class_name: &str, methods: &Vec<Method>, class: &mut PartialClass) -> Result<(
        VTable,
        Vec<String>,
        Vec<String>,
        Vec<SignatureEntry>), CompilerError> {


        let mut entries = Vec::new();
        let mut names = Vec::new();
        let mut responds_to = Vec::new();
        let mut signatures = Vec::new();

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
                if annotation.name == "RespondsTo" {
                    responds_to.push(annotation.parameters[0].to_string());
                } else {
                    responds_to.push(String::from(""));
                }
            }

            names.push(name.to_string());

            entries.push(VTableEntry::default());

            let mut signature = Vec::new();
            signature.push(self.convert_type(return_type));

            parameters.iter().for_each(|param| {
                match param {
                    Parameter::This(_, _) => {
                        signature.push(TypeTag::Object);
                    }
                    Parameter::Pattern { ty, .. } => {
                        signature.push(self.convert_type(ty));
                    }
                }

            });
            signatures.push(SignatureEntry::new(signature));

        }
        let vtable = VTable::new(entries);


        Ok((vtable, names, responds_to, signatures))
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
            Type::Object(_, _) => TypeTag::Object,
            Type::TypeArg(_, _, _) => TypeTag::Object,
            Type::Tuple(_, _) => TypeTag::Object,
            Type::Function(_, _, _) => TypeTag::Object,
        }

    }


    pub fn compile_methods(&mut self, class_name: &str, partial_class: &mut PartialClass, methods: Vec<Method>) -> Result<(), CompilerError> {

        for method in methods {
            let Method {
                name,
                parameters,
                body,
                ..
            } = method;

            self.push_scope();

            for parameter in parameters {
                let Parameter::Pattern { name, .. } = parameter else {
                    self.bind_variable("self");
                    continue;
                };
                self.bind_patterns(&name);
            }
            
            
            let bytecode = self.compile_method_body(class_name, partial_class, body)?;

            let bytecode = bytecode.into_iter().flat_map(|code| {
                code.into_binary()
            }).collect::<Vec<_>>();

            //println!("{}", name);
            let vtable = partial_class.get_vtable(&name).unwrap();
            let method_class_name = String::from(partial_class.index_string_table(vtable.class_name));
            //println!("{}", method_class_name);

            partial_class.attach_bytecode(method_class_name, name, bytecode).expect("Handle partial class error");

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

    fn compile_method_body(&mut self, class_name: &str, partial_class: &mut PartialClass, body: Vec<Statement>) -> Result<Vec<Bytecode>, CompilerError> {
        let mut output = Vec::new();
        let block = self.current_block;
        output.push(Bytecode::StartBlock(block));
        output.push(Bytecode::Goto(1));
        self.compile_block(class_name, partial_class, &body, &mut output)?;

        Ok(output)
    }

    fn compile_block(
        &mut self,
        class_name: &str,
        partial_class: &mut PartialClass,
        body: &Vec<Statement>,
        output: &mut Vec<Bytecode>
    ) -> Result<(), CompilerError> {

        self.push_scope();
        self.increment_block();
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
        class_name: &str,
        partial_class: &mut PartialClass, 
        expr: &'a Expression,
        output: &mut Vec<Bytecode>,
        lhs : bool,
    ) -> Result<Option<Box<dyn Fn(&mut Vec<Bytecode>) + 'a>>, CompilerError> {
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
                                    _ => unreachable!("integer literal")
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
                    (Some(lhs), BinaryOperator::Add, Some(rhs)) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::AddInt)
                    }
                    (Some(lhs), BinaryOperator::Sub, Some(rhs)) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::SubInt)
                    }
                    (Some(lhs), BinaryOperator::Mul, Some(rhs)) if lhs.is_integer() && rhs.is_integer() => {
                        output.push(Bytecode::MulInt)
                    }
                    (Some(lhs), BinaryOperator::Div, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::DivUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Div, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::DivSigned)
                    }
                    (Some(lhs), BinaryOperator::Mod, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::ModUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Mod, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::ModSigned)
                    }
                    (Some(lhs), BinaryOperator::Add, Some(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::AddFloat)
                    }
                    (Some(lhs), BinaryOperator::Sub, Some(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::SubFloat)
                    }
                    (Some(lhs), BinaryOperator::Mul, Some(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::MulFloat)
                    }
                    (Some(lhs), BinaryOperator::Div, Some(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::DivFloat)
                    }
                    (Some(lhs), BinaryOperator::Mod, Some(rhs)) if lhs.is_float() || rhs.is_float() => {
                        output.push(Bytecode::ModFloat)
                    }
                    (Some(lhs), BinaryOperator::Eq, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::EqualUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Eq, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::EqualSigned)
                    }
                    (Some(lhs), BinaryOperator::Ne, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::NotEqualUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Ne, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::NotEqualSigned)
                    }
                    (Some(lhs), BinaryOperator::Lt, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::LessUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Lt, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::LessSigned)
                    }
                    (Some(lhs), BinaryOperator::Le, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::LessOrEqualUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Le, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::LessOrEqualSigned)
                    }
                    (Some(lhs), BinaryOperator::Gt, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::GreaterUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Gt, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::GreaterSigned)
                    }
                    (Some(lhs), BinaryOperator::Ge, Some(rhs)) if lhs.is_unsigned() && rhs.is_unsigned() => {
                        output.push(Bytecode::GreaterOrEqualUnsigned)
                    }
                    (Some(lhs), BinaryOperator::Ge, Some(rhs)) if lhs.is_signed() && rhs.is_signed() => {
                        output.push(Bytecode::GreaterOrEqualSigned)
                    }
                    (Some(Type::U8), BinaryOperator::And, Some(Type::U8)) => {
                        output.push(Bytecode::And)
                    }
                    (Some(Type::U8), BinaryOperator::Or, Some(Type::U8)) => {
                        output.push(Bytecode::Or)
                    }
                    (Some(Type::Array(generic, _)), BinaryOperator::Index, _) => {
                        return Ok(Some(
                            Box::new(move |output: &mut Vec<Bytecode>| {
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
                                };
                                if lhs {
                                    output.push(Bytecode::ArraySet(type_tag));
                                } else {
                                    output.push(Bytecode::ArrayGet(type_tag));
                                }
                            })
                        ));

                    }
                    (l, x, r) => todo!("binary operator {:?} {:?} {:?}", l, x, r),
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
            Expression::Call { name, args, .. } => {
                let (name, ty, var) = match name.as_ref() {
                    Expression::MemberAccess { object, field, span } => {
                        match object.as_ref() {
                            Expression::Variable(var, Some(Type::Object(ty, _)), _) => {
                                (field, ty.clone(), var.clone())
                            }
                            Expression::Variable(var, Some(Type::Array(ty, _)), _) => {
                                let ty = match ty.as_ref() {
                                    Type::U8 | Type::I8 => Text::Borrowed("Array8"),
                                    Type::U16 | Type::I16 => Text::Borrowed("Array16"),
                                    Type::U32 | Type::I32 | Type::Char => Text::Borrowed("Array32"),
                                    Type::U64 | Type::I64 => Text::Borrowed("Array64"),
                                    Type::Object(_, _) | Type::Str | Type::Function(_, _, _) | Type::Array(_, _) | Type::Void | Type::Tuple(_, _) | Type::TypeArg(_, _, _) => Text::Borrowed("ArrayObject"),
                                    Type::F32 => Text::Borrowed("Arrayf32"),
                                    Type::F64 => Text::Borrowed("Arrayf64"),
                                };
                                (field, ty, var.clone())
                            }
                            Expression::This(_) => {
                                (field, Text::Borrowed(class_name), Text::Borrowed("self"))
                            }
                            x => todo!("add additional sources to call from {:?}", x)
                        }
                    }
                    _ => unreachable!("all calls should be via member access by this point")
                };

                let mut argument_pos: u8 = 0;
                
                let object = self.get_variable(var).expect("There should be method calling by this point");
                output.push(Bytecode::LoadLocal(object));
                output.push(Bytecode::StoreArgument(argument_pos));
                argument_pos += 1;

                for arg in args {
                    self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                    output.push(Bytecode::StoreArgument(argument_pos));
                    argument_pos += 1;
                }


                let name = name.segments.last().unwrap();
                let class = self.classes.get(&ty.to_string()).expect("Classes are in a bad order of compiling");
                //println!("{:#?}", class);
                let vtable = class.get_vtable(name).expect("add proper handling of missing vtable");
                let method_entry = class.get_method_entry(name).expect("add proper handling of missing method");

                println!("{}", class.index_string_table(vtable.class_name));

                let class_name = class.index_string_table(vtable.class_name);
                let vtable_class_name = partial_class.add_string(class_name);

                let source_class = if vtable.sub_class_name == 0 {
                    0
                } else {
                    let class_name = class.index_string_table(vtable.sub_class_name);
                    partial_class.add_string(class_name)
                };

                let method_name = class.index_string_table(method_entry.name);
                let method_name = partial_class.add_string(method_name);
                
                
                output.push(Bytecode::InvokeVirt(vtable_class_name, source_class, method_name));
                
            }
            Expression::New(ty, arr_size, _) => {
                let name = match ty {
                    Type::Object(name, _) => name,
                    _ => todo!("handle array new")
                };

                let string_ref = partial_class.add_string(name);

                output.push(Bytecode::NewObject(string_ref));
            }
            Expression::IfExpression(if_expr, _) => {

                self.compile_if_expression(class_name, partial_class, if_expr, output, lhs)?;
            }
            _ => todo!("add remaining expressions")
        }
        Ok(None)
    }

    fn compile_if_expression(
        &mut self,
        class_name: &str,
        partial_class: &mut PartialClass,
        expr: &IfExpression,
        output: &mut Vec<Bytecode>,
        lhs: bool,
    ) -> Result<(), CompilerError> {
        match expr {
            IfExpression { condition, then_branch, else_branch: None, ..} => {
                self.compile_expression(class_name, partial_class, condition.as_ref(), output, lhs)?;
                output.push(Bytecode::If(1, 2));
                self.compile_block(class_name, partial_class, then_branch, output)?;
                output.push(Bytecode::Goto(1));
                self.increment_block();
                let block = self.current_block;
                output.push(Bytecode::StartBlock(block));
            }
            IfExpression { condition, then_branch, else_branch: Some(Either::Right(else_branch)), ..} => {
                self.compile_expression(class_name, partial_class, condition.as_ref(), output, lhs)?;
                output.push(Bytecode::If(1, 2));
                self.compile_block(class_name, partial_class, then_branch, output)?;
                output.push(Bytecode::Goto(2));
                self.compile_block(class_name, partial_class, else_branch, output)?;
                output.push(Bytecode::Goto(1));
                let block = self.current_block;
                output.push(Bytecode::StartBlock(block));
            }
            IfExpression { condition, then_branch, else_branch: Some(Either::Left(else_branch)), ..} => {
                self.compile_expression(class_name, partial_class, condition.as_ref(), output, lhs)?;
                output.push(Bytecode::If(1, 2));
                self.compile_block(class_name, partial_class, then_branch, output)?;
                let mut temp_output = Vec::new();
                let then_block = self.current_block;
                self.compile_if_expression(class_name, partial_class, else_branch.as_ref(), &mut temp_output, lhs)?;
                let escape_block = self.current_block;
                output.push(Bytecode::Goto((escape_block - then_block) as i64));
                output.extend(temp_output);
            }
        }
        Ok(())
    }
}
