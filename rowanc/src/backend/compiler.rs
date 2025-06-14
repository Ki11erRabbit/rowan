use std::{collections::HashMap, io::Write, path::{Path, PathBuf}};
use either::Either;
use rowan_shared::{bytecode::compiled::Bytecode, classfile::{Member, SignatureEntry, VTable, VTableEntry}, TypeTag};
use rowan_shared::classfile::SignatureIndex;
use crate::{ast::{BinaryOperator, Class, Constant, Expression, File, Literal, Method, Parameter, Pattern, Statement, TopLevelStatement, Type, UnaryOperator, Text}, backend::compiler_utils::Frame};
use crate::ast::IfExpression;
use super::compiler_utils::{PartialClass, PartialClassError};



fn create_stdlib() -> HashMap<String, PartialClass> {
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
    object.add_vtable("Object", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::F64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Object]),
    ];
    let vtable = VTable::new(functions);
    
    printer.add_vtable("Printer", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    string.add_vtable("String", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array8", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array16", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array32", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Array64", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Arrayf32", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("Arrayf64", vtable, &names, &signatures);
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
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::Str]),
        SignatureEntry::new(vec![TypeTag::Void]),
        SignatureEntry::new(vec![TypeTag::U64]),
    ];
    let vtable = VTable::new(functions);
    
    array.add_vtable("ArrayObject", vtable, &names, &signatures);
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

    /// files should be sorted in a way that means we don't need to do each file incrementally
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
            ..
        } = class;

        let mut partial_class = PartialClass::new();
        partial_class.set_name(&name);

        let parent_vtables = parents.iter().map(|parent_name| {
            let partial_class = self.classes.get(&parent_name.name.clone().to_string()).expect("Order of files is wrong");
            let vtables = partial_class.get_vtables(&parent_name.name);
            vtables.into_iter().map(|(table, names, signatures)| {
                let class_name = partial_class.index_string_table(table.class_name);
                let source_class = if table.sub_class_name == 0 {
                    None
                } else {
                    Some(partial_class.index_string_table(table.sub_class_name))
                };
                (class_name, source_class, table, names, signatures)
            }).collect::<Vec<_>>()
            
        });
        parents.iter().for_each(|parent| {
            partial_class.add_parent(&parent.name);
        });

        let (vtable, names, signatures, static_method_map, static_signatures) = self.construct_vtable(&name, &methods)?;

        partial_class.add_signatures(static_signatures);
        
        partial_class.set_static_method_to_sig(static_method_map);
        
        if vtable.functions.len() != 0 {
            partial_class.add_vtable(&name, vtable, &names, &signatures);
        } else {
            drop(vtable);
            drop(names);
            drop(signatures);
        }

        if parents.len() == 0 {
            let object_class = self.classes.get("Object").expect("Object not added to known classes");

            let vtables = object_class.get_vtables("Object");
            let (vtable, names, signatures) = &vtables[0];

            partial_class.add_vtable("Object", vtable.clone(), names, signatures);
            partial_class.add_parent("Object");
        }
        
        for vtables in parent_vtables {
            for (class_name, _source_class, vtable, names, signatures) in vtables {
                partial_class.add_vtable(class_name, vtable.clone(), &names, &signatures);
            }
        }

        members.into_iter().map(|member| {
            (member.name, Member::new(self.convert_type(&member.ty)))
        }).for_each(|(name, member)| {
            partial_class.add_member(member, name);
        });

        self.compile_methods(&name, &mut partial_class, methods)?;
        
        self.classes.insert(name.to_string(), partial_class);
        
        Ok(())
    }

    fn construct_vtable(&self, class_name: &str, methods: &Vec<Method>) -> Result<(
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
                static_method_to_signature.insert(name.to_string(), signature_index as SignatureIndex);
                static_signatures.push(SignatureEntry::new(signature));
            } else {
                names.push(name.to_string());
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
            let mut is_static = true;
            for parameter in parameters {
                let Parameter::Pattern { name, .. } = parameter else {
                    self.bind_variable("self");
                    is_static = false;
                    continue;
                };
                self.bind_patterns(&name);
            }
            
            
            let bytecode = self.compile_method_body(class_name, partial_class, body)?;

            let bytecode = bytecode.into_iter().flat_map(|code| {
                code.into_binary()
            }).collect::<Vec<_>>();

            if is_static {
                partial_class.add_static_method(name, bytecode);
            } else {
                //println!("{}", name);
                let vtable = partial_class.get_vtable(&name).unwrap();
                let method_class_name = String::from(partial_class.index_string_table(vtable.class_name));
                //println!("{}", method_class_name);

                partial_class.attach_bytecode(method_class_name, name, bytecode).expect("Handle partial class error");
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

    fn compile_method_body(&mut self, class_name: &str, partial_class: &mut PartialClass, body: Vec<Statement>) -> Result<Vec<Bytecode>, CompilerError> {
        let mut output = Vec::new();
        //output.push(Bytecode::StartBlock(block));
        //output.push(Bytecode::Goto(1));
        self.current_block = 0;
        self.compile_block(class_name, partial_class, &body, &mut output)?;
        //println!("Bytecode output: {:#?}", output);

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
        class_name: &str,
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
            Expression::Call { name, type_args, args, .. } => {
                let (name, ty, var) = match name.as_ref() {
                    Expression::MemberAccess { object, field, span, .. } => {
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

                if name.as_str() == "downcast" || name.as_str() == "downcast-contents" {
                    assert_eq!(type_args.len(), 1, "Downcast only takes one type argument");
                    let ty = match type_args.first().unwrap() {
                        Type::Array(ty, _) => {
                            match ty.as_ref() {
                                Type::U8 | Type::I8 => Text::Borrowed("Array8"),
                                Type::U16 | Type::I16 => Text::Borrowed("Array16"),
                                Type::U32 | Type::I32 | Type::Char => Text::Borrowed("Array32"),
                                Type::U64 | Type::I64 => Text::Borrowed("Array64"),
                                Type::Object(_, _) | Type::Str | Type::Function(_, _, _) | Type::Array(_, _) | Type::Void | Type::Tuple(_, _) | Type::TypeArg(_, _, _) => Text::Borrowed("ArrayObject"),
                                Type::F32 => Text::Borrowed("Arrayf32"),
                                Type::F64 => Text::Borrowed("Arrayf64"),
                            }
                        }
                        Type::Object(name, _) => name.clone(),
                        _ => unreachable!("downcast can only take type arguments that are Objects or Arrays, not Tuples or primitives like integers and floats"),
                    };

                    let class_symbol = partial_class.add_string(ty.as_str());

                    output.push(Bytecode::LoadSymbol(class_symbol));
                    output.push(Bytecode::StoreArgument(argument_pos));
                }

                if let Some(class) = self.classes.get(&ty.to_string()) {
                    //println!("{:#?}", class);
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
                    let method_name = partial_class.add_string(method_name);


                    output.push(Bytecode::InvokeVirt(vtable_class_name, source_class, method_name));
                } else if ty.as_str() == class_name {
                    let vtable = partial_class.get_vtable(name).expect("add proper handling of missing vtable").clone();
                    let method_entry = partial_class.get_method_entry(name).expect("add proper handling of missing method");

                    //println!("{}", partial_class.index_string_table(vtable.class_name));

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
                    let method_name = partial_class.add_string(method_name);


                    output.push(Bytecode::InvokeVirt(vtable_class_name, source_class, method_name));
                } else {
                    panic!("Classes are in a bad order of compiling")
                }
            }
            Expression::StaticCall { name, type_args, args, .. } => {
                let mut argument_pos: u8 = 0;
                for arg in args {
                    self.compile_expression(class_name, partial_class, arg, output, lhs)?;
                    output.push(Bytecode::StoreArgument(argument_pos));
                    argument_pos += 1;
                }

                let method_name = name.segments.last().unwrap();
                let method_class = name.segments.iter().rev().skip(1).next().unwrap();
                let method_name = partial_class.add_string(method_name);
                let method_class = partial_class.add_string(method_class);
                
                output.push(Bytecode::InvokeStatic(method_class, method_name));
            }
            Expression::MemberAccess {
                ..
            } => {
                self.compile_member_get(class_name, partial_class, expr, output)?;
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
                self.increment_block();
                self.compile_block(class_name, partial_class, then_branch, output)?;
                self.increment_block();
                output.push(Bytecode::Goto(1));
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
                output.push(Bytecode::Goto(1));
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
                output.push(Bytecode::Goto((escape_block - then_block) as i64));
                output.extend(temp_output);
            }
        }
        Ok(())
    }

    fn compile_member_get<'a>(
        &mut self,
        class_name: &str,
        partial_class: &mut PartialClass,
        expr: &'a Expression<'a>,
        output: &mut Vec<Bytecode>
    ) -> Result<(), CompilerError> {
        let Expression::MemberAccess {
            object, field, ..
        } = expr else {
            unreachable!("We have already checked for expr being a MemberAccess");
        };

        self.compile_expression(class_name, partial_class, object.as_ref(), output, false)?;

        let Some(annotation) = object.get_type() else {
            unreachable!("Expression should be annotated by this point");
        };

        let name = match annotation  {
            Either::Left(Type::Object(name, _)) => name,
            Either::Right(()) => {
                Text::Borrowed(class_name)
            }
            _ => todo!("report error about method output not being an object"),
        };

        let class = match self.classes.get(name.as_str()) {
            Some(class) => class,
            _ => partial_class,
        };
        let (class_name, parent_name) = if class.contains_field(field.to_string().as_str()) {
            (class.get_class_name(), "")
        } else {
            let Some((name, parent)) = class.find_class_with_field(&self.classes, field.to_string().as_str()) else {
                todo!("report error about being unable to find field")
            };

            (name, parent)
        };
        let (offset, type_tag) = if parent_name != "" {
            let class = self.classes.get(parent_name).unwrap();
            class.get_member_offset(field.to_string().as_str())
        } else {
            class.get_member_offset(field.to_string().as_str())
        };
        let class_name = class_name.to_string();
        let parent_name = parent_name.to_string();

        let class_name = partial_class.add_string(class_name);
        let parent_name = partial_class.add_string(parent_name);

        output.push(Bytecode::GetField(class_name, parent_name, offset, type_tag));

        Ok(())
    }

    fn compile_member_set<'a>(
        &mut self,
        class_name: &str,
        partial_class: &mut PartialClass,
        expr: &'a Expression<'a>,
        output: &mut Vec<Bytecode>
    ) -> Result<Option<Box<dyn Fn(&mut Vec<Bytecode>) + 'a>>, CompilerError> {
        let Expression::MemberAccess {
            object, field, ..
        } = expr else {
            unreachable!("We have already checked for expr being a MemberAccess");
        };

        self.compile_expression(class_name, partial_class, object.as_ref(), output, false)?;

        let Some(annotation) = object.get_type() else {
            unreachable!("Expression should be annotated by this point");
        };

        let name = match annotation  {
            Either::Left(Type::Object(name, _)) => name,
            Either::Right(()) => {
                Text::Borrowed(class_name)
            }
            _ => todo!("report error about method output not being an object"),
        };

        let class = self.classes.get(name.as_str()).unwrap_or(partial_class);
        let (class_name, parent_name) = if class.contains_field(field.to_string().as_str()) {
            (class.get_class_name(), "")
        } else {
            let Some((name, parent)) = class.find_class_with_field(&self.classes, field.to_string().as_str()) else {
                todo!("report error about being unable to find field")
            };

            (name, parent)
        };
        let (offset, type_tag) = if parent_name != "" {
            let class = self.classes.get(parent_name).unwrap();
            class.get_member_offset(field.to_string().as_str())
        } else {
            class.get_member_offset(field.to_string().as_str())
        };
        let class_name = class_name.to_string();
        let parent_name = parent_name.to_string();

        let class_name = partial_class.add_string(class_name);
        let parent_name = partial_class.add_string(parent_name);



        Ok(Some(Box::new(move |output: &mut Vec<Bytecode>| {
            output.push(Bytecode::SetField(class_name, parent_name, offset, type_tag));
        })))
    }
}
