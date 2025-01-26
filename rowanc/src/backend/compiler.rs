use std::{collections::HashMap, io::Write, path::{Path, PathBuf}};
use rowan_shared::{bytecode::compiled::Bytecode, classfile::{Member, Signal, SignatureEntry, VTable, VTableEntry}, TypeTag};

use crate::{ast::{BinaryOperator, Class, Constant, Expression, File, Literal, Method, Parameter, Pattern, Statement, TopLevelStatement, Type, UnaryOperator}, backend::compiler_utils::Frame};

use super::compiler_utils::PartialClass;



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
    let vtable = VTable::new(functions);
    let class_names = vec![
        "Object",
        "Object",
        "Object",
        "Object",
        "Object",
        ];
    let sub_class_names = vec![
        "Object",
        "Object",
        "Object",
        "Object",
        "Object",
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
    
    object.add_vtable("Object",vtable, class_names, sub_class_names, names, responds_to, signatures);
    object.make_not_printable();
    classes.insert(String::from("Object"), object);

    let mut printer = PartialClass::new();
    printer.set_name("Printer");
    let functions = vec![
        VTableEntry::default(),
        VTableEntry::default(),
    ];
    let vtable = VTable::new(functions);
    let class_names = vec![
        "Printer",
        "Printer",
        ];
    let sub_class_names = vec![
        "Printer",
        "Printer",
        ];
    let names = vec![
        "println-int",
        "println-float",
    ];
    let responds_to = vec![
        "",
        "",
    ];
    let signatures = vec![
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::U64]),
        SignatureEntry::new(vec![TypeTag::Void, TypeTag::F64]),
    ];
    
    printer.add_vtable("Printer",vtable, class_names, sub_class_names, names, responds_to, signatures);
    printer.make_not_printable();
    classes.insert(String::from("Printer"), printer);


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
        partial_class.set_name(name);

        let parent_vtables = parents.iter().map(|parent_name| {
            let partial_class = self.classes.get(parent_name.name).expect("Order of files is wrong");

            (parent_name.name, partial_class.get_vtable(parent_name.name))

        }).collect::<Vec<_>>();
        parents.iter().for_each(|parent| {
            partial_class.add_parent(parent.name);
        });

        let (vtable, class_names, sub_class_names, names, responds_to, signatures) = self.construct_vtable(name.to_string(), &methods)?;

        partial_class.add_vtable(name, vtable, class_names, sub_class_names, names, responds_to, signatures);

        if parents.len() == 0 {
            let object_class = self.classes.get("Object").expect("Object not added to known classes");

            let (vtable, class_names, sub_class_names, names, responds_to, signatures) = object_class.get_vtable("Object");

            partial_class.add_vtable("Object", vtable, class_names, sub_class_names, names, responds_to, signatures);
        }
        
        for (class_name, (vtable, class_names, sub_class_names, names, responds_to, signatures)) in parent_vtables {
            partial_class.add_vtable(class_name, vtable, class_names, sub_class_names, names, responds_to, signatures);
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

        self.compile_methods(name, &mut partial_class, methods)?;
        
        self.classes.insert(name.to_string(), partial_class);
        
        Ok(())
    }

    fn construct_vtable(&self, class_name: String, methods: &Vec<Method>) -> Result<(
        VTable,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<SignatureEntry>), CompilerError> {


        let mut entries = Vec::new();
        let mut class_names = Vec::new();
        let mut sub_class_names = Vec::new();
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
            class_names.push(class_name.to_string());
            sub_class_names.push(class_name.to_string());

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


        Ok((vtable, class_names, sub_class_names, names, responds_to, signatures))
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

            let method_entry = partial_class.get_method_entry(name).unwrap();
            let method_class_name = String::from(partial_class.index_string_table(method_entry.sub_class_name));

            partial_class.attach_bytecode(method_class_name, name, bytecode);

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

        self.compile_block(class_name, partial_class, body, &mut output)?;

        Ok(output)
    }

    fn compile_block(
        &mut self,
        class_name: &str,
        partial_class: &mut PartialClass,
        body: Vec<Statement>,
        output: &mut Vec<Bytecode>
    ) -> Result<(), CompilerError> {

        self.push_scope();
        let block = self.current_block;
        self.increment_block();
        output.push(Bytecode::StartBlock(block));
        
        for statement in body {
            match statement {
                Statement::Expression(expr, span) => {
                    self.compile_expression(class_name, partial_class, &expr, output)?;
                }
                Statement::Let { bindings, value, .. } => {
                    self.compile_expression(class_name, partial_class, &value, output)?;
                    match bindings {
                        Pattern::Variable(var, _, _) => {
                            let index = self.bind_variable(var);
                            output.push(Bytecode::StoreLocal(index));

                        }
                        _ => todo!("let bindings"),
                    }
                }
                _ => unimplemented!(),
            }
        }

        self.pop_scope();

        Ok(())
    }

    fn compile_expression(
        &mut self,
        class_name: &str,
        partial_class: &mut PartialClass, 
        expr: &Expression,
        output: &mut Vec<Bytecode>
    ) -> Result<(), CompilerError> {
        match expr {
            Expression::Variable(var, _, span) => {
                let index = self.get_variable(var)
                    .ok_or(CompilerError::UnboundIdentifer(String::from(*var), span.start, span.end))?;
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
                                let chr = match *char_str {
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
                                    x => return Err(CompilerError::MalformedCharacter(String::from(x), span.start, span.end)),
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
                    _ => todo!("all other literals")
                }
            }
            Expression::This(_) => {
                output.push(Bytecode::LoadLocal(0));
            }
            Expression::BinaryOperation { operator, left, right, span } => {
                self.compile_expression(class_name, partial_class, left.as_ref(), output)?;
                self.compile_expression(class_name, partial_class, right.as_ref(), output)?;

                match operator {
                    BinaryOperator::Add => {
                        output.push(Bytecode::Add)
                    }
                    BinaryOperator::Sub => {
                        output.push(Bytecode::Sub)
                    }
                    BinaryOperator::Mul => {
                        output.push(Bytecode::Mul)
                    }
                    BinaryOperator::Div => {
                        output.push(Bytecode::Div)
                    }
                    BinaryOperator::Mod => {
                        output.push(Bytecode::Mod)
                    }
                    BinaryOperator::Eq => {
                        output.push(Bytecode::Equal)
                    }
                    BinaryOperator::Ne => {
                        output.push(Bytecode::NotEqual)
                    }
                    BinaryOperator::Lt => {
                        output.push(Bytecode::Less)
                    }
                    BinaryOperator::Le => {
                        output.push(Bytecode::LessOrEqual)
                    }
                    BinaryOperator::Gt => {
                        output.push(Bytecode::Greater)
                    }
                    BinaryOperator::Ge => {
                        output.push(Bytecode::GreaterOrEqual)
                    }
                    BinaryOperator::And => {
                        output.push(Bytecode::And)
                    }
                    BinaryOperator::Or => {
                        output.push(Bytecode::Or)
                    }
                    _ => todo!("binary operator")
                }
                
            }
            Expression::UnaryOperation { operator, operand, span } => {
                self.compile_expression(class_name, partial_class, operand.as_ref(), output)?;

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
                self.compile_expression(class_name, partial_class, expr.as_ref(), output)?;
            }
            Expression::Call { name, type_args, args, span } => {
                let (name, ty, var) = match name.as_ref() {
                    Expression::MemberAccess { object, field, span } => {
                        match object.as_ref() {
                            Expression::Variable(var, Some(Type::Object(ty, _)), _) => {
                                (field, *ty, *var)
                            }
                            Expression::This(_) => {
                                (field, class_name, "self")
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

                for arg in args {
                    self.compile_expression(class_name, partial_class, arg, output)?;
                    output.push(Bytecode::StoreArgument(argument_pos));
                    argument_pos += 1;
                }


                let name = name.segments.last().unwrap();
                let class = self.classes.get(ty).expect("Classes are in a bad order of compiling");
                println!("{:#?}", class);
                let method_entry = class.get_method_entry(name).expect("add proper handling of missing method");

                output.push(Bytecode::InvokeVirt(method_entry.class_name, method_entry.sub_class_name, method_entry.name));
                
            }
            Expression::New(ty, arr_size, _) => {
                let name = match ty {
                    Type::Object(name, _) => name,
                    _ => todo!("handle array new")
                };

                let string_ref = partial_class.add_string(name);

                output.push(Bytecode::NewObject(string_ref));
            }
            _ => todo!("add remaining expressions")
        }
        Ok(())
    }
}
