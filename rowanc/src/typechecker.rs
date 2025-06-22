use std::{borrow::BorrowMut, collections::HashMap};
use std::cmp::Ordering;
use std::collections::HashSet;
use either::Either;
use itertools::Itertools;
use crate::ast::{BinaryOperator, Expression, Literal, Span, Text, Type};

fn create_stdlib<'a>() -> HashMap<Vec<String>, (Vec<String>, HashMap<String, ClassAttribute>)> {
    let mut info = HashMap::new();
    let mut object_attributes = HashMap::new();
    object_attributes.insert("tick".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::F64], Box::new(TypeCheckerType::Void))));
    object_attributes.insert("ready".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![], Box::new(TypeCheckerType::Void))));
    object_attributes.insert("downcast".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![], Box::new(TypeCheckerType::Object(String::from("Object"))))));
    object_attributes.insert("get-child".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::U64], Box::new(TypeCheckerType::Object(String::from("Object"))))));
    object_attributes.insert("remove-child".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::Object(String::from("Object"))], Box::new(TypeCheckerType::Void))));
    object_attributes.insert("add-child".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::Object(String::from("Object"))], Box::new(TypeCheckerType::Void))));
    object_attributes.insert("add-child-at".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::Object(String::from("Object")), TypeCheckerType::U64], Box::new(TypeCheckerType::Void))));
    object_attributes.insert("child-died".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::U64, TypeCheckerType::Object(String::from("Object"))], Box::new(TypeCheckerType::Void))));

    info.insert(vec!["Object".to_string()], (Vec::new(), object_attributes));

    let mut printer_attributes = HashMap::new();
    printer_attributes.insert("println-int".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::U64], Box::new(TypeCheckerType::Void))));

    info.insert(vec!["Printer".to_string()], (vec![String::from("Object")], printer_attributes));

    let mut array_attributes = HashMap::new();
    array_attributes.insert("len".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![], Box::new(TypeCheckerType::U64))));
    array_attributes.insert("downcast-contents".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![], Box::new(TypeCheckerType::Object(String::from(""))))));

    info.insert(vec!["Array".to_string()], (vec![String::from("Object")], array_attributes));
    
    info
}


#[derive(Debug)]
pub enum TypeCheckerError {
    UnableToDeduceType {
        start: usize,
        end: usize,
    }
}

#[derive(Debug, Clone)]
pub enum TypeCheckerType {
    Void,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Char,
    Str,
    Array(Box<TypeCheckerType>),
    Object(String),
    TypeArg(Box<TypeCheckerType>, Vec<TypeCheckerType>),
    Function(Vec<TypeCheckerType>, Box<TypeCheckerType>),
    Tuple(Vec<TypeCheckerType>),
}

impl<'a> From<Type<'a>> for TypeCheckerType {
    fn from(ty: Type<'a>) -> TypeCheckerType {
        match ty {
            Type::Void => TypeCheckerType::Void,
            Type::U8 => TypeCheckerType::U8,
            Type::U16 => TypeCheckerType::U16,
            Type::U32 => TypeCheckerType::U32,
            Type::U64 => TypeCheckerType::U64,
            Type::I8 => TypeCheckerType::I8,
            Type::I16 => TypeCheckerType::I16,
            Type::I32 => TypeCheckerType::I32,
            Type::I64 => TypeCheckerType::I64,
            Type::F32 => TypeCheckerType::F32,
            Type::F64 => TypeCheckerType::F64,
            Type::Char => TypeCheckerType::Char,
            Type::Str => TypeCheckerType::Str,
            Type::Array(ty, _) => TypeCheckerType::Array(Box::new(TypeCheckerType::from(*ty))),
            Type::Object(name, _) => TypeCheckerType::Object(name.to_string()),
            Type::TypeArg(name, constraint, _) => TypeCheckerType::TypeArg(
                Box::new(TypeCheckerType::from(*name)),
                constraint.into_iter().map(TypeCheckerType::from).collect()),
            Type::Function(args, ret, _) => TypeCheckerType::Function(
                args.into_iter().map(TypeCheckerType::from).collect(),
                Box::new(TypeCheckerType::from(*ret))
            ),
            Type::Tuple(tys, _) => TypeCheckerType::Tuple(tys.into_iter().map(TypeCheckerType::from).collect())
        }
    }
}

impl<'a> From<&Type<'a>> for TypeCheckerType {
    fn from(ty: &Type<'a>) -> TypeCheckerType {
        TypeCheckerType::from(ty.clone())
    }
}

impl<'a> From<&mut Type<'a>> for TypeCheckerType {
    fn from(ty: &mut Type<'a>) -> TypeCheckerType {
        TypeCheckerType::from(ty.clone())
    }
}

impl<'a> Into<Type<'a>> for TypeCheckerType {
    fn into(self: TypeCheckerType) -> Type<'a> {
        match self {
            TypeCheckerType::Void => Type::Void,
            TypeCheckerType::U8 => Type::U8,
            TypeCheckerType::U16 => Type::U16,
            TypeCheckerType::U32 => Type::U32,
            TypeCheckerType::U64 => Type::U64,
            TypeCheckerType::I8 => Type::I8,
            TypeCheckerType::I16 => Type::I16,
            TypeCheckerType::I32 => Type::I32,
            TypeCheckerType::I64 => Type::I64,
            TypeCheckerType::F32 => Type::F32,
            TypeCheckerType::F64 => Type::F64,
            TypeCheckerType::Char => Type::Char,
            TypeCheckerType::Str => Type::Str,
            TypeCheckerType::Array(ty) => Type::Array(Box::new((*ty).into()), crate::ast::Span::new(0, 0)),
            TypeCheckerType::Object(name) => Type::Object(crate::ast::Text::Owned(name), crate::ast::Span::new(0, 0)),
            TypeCheckerType::TypeArg(name, constraint) => Type::TypeArg(
                Box::new((*name).into()),
                constraint.into_iter().map(|x| x.into()).collect(), crate::ast::Span::new(0, 0)),
            TypeCheckerType::Function(args, ret) => Type::Function(
                args.into_iter().map(|x| x.into()).collect(),
                Box::new((*ret).into()), crate::ast::Span::new(0, 0)
            ),
            TypeCheckerType::Tuple(tys) => Type::Tuple(tys.into_iter().map(|x| x.into()).collect(), crate::ast::Span::new(0, 0))
        }
    }
}

impl<'a, 'b> Into<Type<'a>> for &'b TypeCheckerType {
    fn into(self: &'b TypeCheckerType) -> Type<'a> {
        (*self).clone().into()
    }
}

#[derive(Debug, Clone)]
pub enum ClassAttribute {
    Member(TypeCheckerType),
    Method(TypeCheckerType),
}

pub struct Frame {
    variables: HashMap<String, TypeCheckerType>,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            variables: HashMap::new()
        }
    }

    pub fn insert<S: AsRef<str>>(&mut self, var: S, ty: TypeCheckerType) {
        self.variables.insert(var.as_ref().to_string(), ty);
    }

    pub fn get<S: AsRef<str>>(&self, var: S) -> Option<&TypeCheckerType> {
        self.variables.get(var.as_ref())
    }
}

pub struct TypeChecker {
    class_information: HashMap<Vec<String>, (Vec<String>, HashMap<String, ClassAttribute>)>,
    scopes: Vec<Frame>,
    current_class: Vec<String>,
    active_paths: HashMap<String, Vec<String>>,
    active_module: Vec<String>,
}


impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            class_information: create_stdlib(),
            scopes: Vec::new(),
            current_class: Vec::new(),
            active_paths: HashMap::new(),
            active_module: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Frame::new())
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup_var<S: AsRef<str>>(&self, var: S) -> Option<&TypeCheckerType> {
        for frame in self.scopes.iter().rev() {
            let value = frame.get(var.as_ref());
            if value.is_some() {
                return value
            }
        }
        None
    }

    fn insert_var<S: AsRef<str>>(&mut self, var: S, ty: TypeCheckerType) {
        if let Some(frame) = self.scopes.last_mut() {
            frame.insert(var.as_ref(), ty)
        }
    }

    fn get_attribute<S: AsRef<str>>(&self, class: &[String], attribute: S) -> Option<&ClassAttribute> {
        if class.is_empty() {
            return None
        }
        self.class_information.get(class).and_then(|attributes| {
            let out = attributes.1.get(attribute.as_ref());
            out
        })
    }

    fn attach_module_if_needed(&self, class: String) -> Vec<String> {
        let path = self.active_paths.get(&class);
        if let Some(path) = path {
            let module = path.clone();
            module
        } else if self.class_information.get(&vec![class.clone()]).is_some() {
            return vec![class]
        } else {
            let mut module = self.active_module.clone();
            module.push(class);
            module
        }
    }
    
    fn compare_types(&self, left: &TypeCheckerType, right: &TypeCheckerType) -> bool {
        match (left, right) {
            (TypeCheckerType::Void, TypeCheckerType::Void) => true,
            (TypeCheckerType::U8, TypeCheckerType::U8) => true,
            (TypeCheckerType::U16, TypeCheckerType::U16) => true,
            (TypeCheckerType::U32, TypeCheckerType::U32) => true,
            (TypeCheckerType::U64, TypeCheckerType::U64) => true,
            (TypeCheckerType::I8, TypeCheckerType::I8) => true,
            (TypeCheckerType::I16, TypeCheckerType::I16) => true,
            (TypeCheckerType::I32, TypeCheckerType::I32) => true,
            (TypeCheckerType::I64, TypeCheckerType::I64) => true,
            (TypeCheckerType::F32, TypeCheckerType::F32) => true,
            (TypeCheckerType::F64, TypeCheckerType::F64) => true,
            (TypeCheckerType::Char, TypeCheckerType::Char) => true,
            (TypeCheckerType::Str, TypeCheckerType::Str) => true,
            (TypeCheckerType::Array(ty1), TypeCheckerType::Array(ty2)) => {
                self.compare_types(ty1, ty2)
            }
            (TypeCheckerType::Object(name1), TypeCheckerType::Object(name2)) => {
                if name1 == name2 {
                    true
                } else {
                    self.compare_object(name1, name2)
                }
            }
            (
                TypeCheckerType::TypeArg(obj1, params1),
                TypeCheckerType::TypeArg(obj2, params2)
            ) => {
                if !self.compare_types(obj1, obj2) {
                    return false;
                }
                if params1.len() != params2.len() {
                    return false;
                }
                for (param1, param2) in params1.iter().zip(params2.iter()) {
                    if !self.compare_types(param1, param2) {
                        return false;
                    }
                }
                true
            }
            (TypeCheckerType::Object(name), _) => {
                let default = &Vec::new();
                let name = self.active_paths.get(name).unwrap_or(default);
                if self.class_information.contains_key(name) {
                    // Here the class exists so we know that the other type can't equal an object
                    false
                } else {
                    // Here the class doesn't exist. This means we are likely a generic type, and therefore always equal
                    true
                }
            }
            (_, TypeCheckerType::Object(name)) => {
                let default = &Vec::new();
                let name = self.active_paths.get(name).unwrap_or(default);
                if self.class_information.contains_key(name) {
                    // Here the class exists so we know that the other type can't equal an object
                    false
                } else {
                    // Here the class doesn't exist. This means we are likely a generic type, and therefore always equal
                    true
                }
            }
            _ => false,
        }
    }
    
    fn compare_object(&self, left: &str, right: &str) -> bool {
        if left == "object" || right == "object" {
            true
        } else if left == right {
            true
        } else {
            let right_path = self.active_paths.get(right).unwrap();
            for right_parents in self.class_information.get(right_path).unwrap().0.iter() {
                if self.compare_object(left, right_parents) {
                    return true;
                }
            }
            let left_path = self.active_paths.get(left).unwrap();
            for left_parent in self.class_information.get(left_path).unwrap().0.iter() {
                if self.compare_object(right, left_parent) {
                    return true;
                }
            }
            false
        }
    }

    pub fn check<'a>(&mut self, files: Vec<crate::ast::File<'a>>) -> Result<Vec<crate::ast::File<'a>>, TypeCheckerError> {
        self.check_files(files)
    }

    fn check_files<'a>(&mut self, mut files: Vec<crate::ast::File<'a>>) -> Result<Vec<crate::ast::File<'a>>, TypeCheckerError> {
        for file in files.iter_mut() {
            self.check_file(file)?;
            self.active_paths.clear();
        }
        Ok(files)
    }

    fn check_file<'a>(&mut self, file: &mut crate::ast::File<'a>) -> Result<(), TypeCheckerError> {
        let module: Vec<String> = file.path.segments.iter().map(ToString::to_string).collect();
        file.content.sort_by(|a, b| {
            match (a, b) {
                (crate::ast::TopLevelStatement::Class(_), crate::ast::TopLevelStatement::Import(_)) => {
                    Ordering::Greater
                }
                (crate::ast::TopLevelStatement::Import(_), crate::ast::TopLevelStatement::Class(_)) => {
                    Ordering::Less
                }
                _ => Ordering::Equal,
            }
        });
        self.active_module = module.clone();
        for content in file.content.iter_mut() {
            match content {
                crate::ast::TopLevelStatement::Class(class) => {
                    self.check_class(class, &module)?;
                }
                crate::ast::TopLevelStatement::Import(import) => {
                    let path_terminator = import.path.segments.last().unwrap().to_string();
                    let path = import.path.segments.iter().map(ToString::to_string).collect::<Vec<_>>();
                    self.active_paths.insert(path_terminator, path);
                }
            }
        }
        Ok(())
    }

    fn check_class<'a>(&mut self, class: &mut crate::ast::Class<'a>, module: &Vec<String>) -> Result<(), TypeCheckerError> {
        let crate::ast::Class {
            name,
            members,
            methods,
            parents,
            ..
        } = class;
        let class_name = name;
        let mut class_attributes = HashMap::new();
        for member in members.iter() {
            let crate::ast::Member { name, ty, .. } = member;
            class_attributes.insert(name.to_string(), ClassAttribute::Member(TypeCheckerType::from(ty.clone())));
        }

        for method in methods.iter() {
            let crate::ast::Method { name, parameters, return_type, .. } = method;
            let mut argument_types = Vec::new();
            for parameter in parameters {
                match parameter {
                    crate::ast::Parameter::This(_, _) => {
                        //argument_types.push(TypeCheckerType::Object(class_name.to_string()));
                    }
                    crate::ast::Parameter::Pattern { ty, .. } => {
                        argument_types.push(TypeCheckerType::from(ty.clone()));
                    }
                }
            }
            let ty = TypeCheckerType::Function(argument_types, Box::new(TypeCheckerType::from(return_type.clone())));
            class_attributes.insert(name.to_string(), ClassAttribute::Method(ty));
        }

        
        let parents = parents.iter().map(|dec| dec.name.to_string()).collect();
        let mut module = module.clone();
        module.push(class_name.to_string());

        self.class_information.insert(module.clone(), (parents, class_attributes));

        self.current_class = module;
        for method in methods.iter_mut() {
            self.check_method(method)?
        }

        Ok(())
    }

    fn check_method<'a>(&mut self, method: &mut crate::ast::Method<'a>) -> Result<(), TypeCheckerError> {
        let crate::ast::Method { parameters, return_type, body, .. } = method;
        self.push_scope();

        for parameter in parameters {
            match parameter {
                crate::ast::Parameter::This(_, _) => {}
                crate::ast::Parameter::Pattern { name, ty, .. } => {
                    self.bind_pattern(name, ty);
                }
            }
        }


        self.check_body(&TypeCheckerType::from(return_type.clone()), body)?;
        
        self.pop_scope();
        Ok(())
    }

    fn bind_pattern(&mut self, pattern: &crate::ast::Pattern, ty: &Type) {
        use crate::ast::Pattern;
        match (pattern, ty) {
            (Pattern::Variable(name, _,_), ty) => {
                let ty = TypeCheckerType::from(ty);
                self.insert_var(name, ty);
            }
            (Pattern::Tuple(names, _), Type::Tuple(tys, _)) => {
                for (name, ty) in names.iter().zip(tys.iter()) {
                    self.bind_pattern(name, ty);
                }
            }
            _ => {}
        }
    }

    fn check_body<'a>(&mut self, return_type: &TypeCheckerType, body: &mut Vec<crate::ast::Statement<'a>>) -> Result<(), TypeCheckerError> {

        self.push_scope();
        for statement in body {
            self.check_statement(&return_type, statement)?;
        }
        
        self.pop_scope();
        Ok(())
    }

    fn check_statement<'a>(&mut self, return_type: &TypeCheckerType, statement: &mut crate::ast::Statement<'a>) -> Result<(), TypeCheckerError> {
        use crate::ast::Statement;

        match statement {
            Statement::Const { bindings, ty, value, .. } => {
                self.annotate_expr(ty, value)?;
                self.bind_pattern(bindings, ty);
            }
            Statement::Let { bindings, ty, value, .. } => {
                self.annotate_expr(ty, value)?;
                self.bind_pattern(bindings, ty);
            }
            Statement::Assignment { target, value, .. } => {
                let lhs = self.get_type(target)?;
                self.annotate_expr(&lhs, value)?;
            }
            Statement::Expression(expr, _) => {
                self.check_expr(return_type, expr)?;
            }
            Statement::While { test, body, ..} => {
                self.check_expr(return_type, test)?;
                self.check_body(return_type, body)?;
            }
            _ => {}
        }

        Ok(())
    }

    fn check_expr<'a>(&mut self, return_type: &TypeCheckerType, expr: &mut crate::ast::Expression<'a>) -> Result<(), TypeCheckerError> {
        use crate::ast::{Expression, BinaryOperator, UnaryOperator};
        match expr {
            Expression::IfExpression(expr, _) => {
                // TODO: check if if expression return values are the same
                self.check_if_expr(return_type, expr)?;
            }
            Expression::Return(None, _) => {
                if !self.compare_types(return_type, &TypeCheckerType::Void) {
                    todo!("report type mismatch returning void when non-void")
                }
            }
            Expression::Return(Some(expr), _) => {
                self.annotate_expr(&return_type.into(), expr.as_mut())?;
            }
            Expression::BinaryOperation { operator: BinaryOperator::And, left, right, .. } => {
                let lhs = self.get_type(left)?;
                let rhs = self.get_type(right)?;

                if lhs != Type::U8 || rhs != Type::U8 {
                    todo!("report boolean operands aren't booleans")
                }
                if lhs != rhs {
                    todo!("report type mismatch");
                }
            }
            Expression::BinaryOperation { operator: BinaryOperator::Or, left, right, .. } => {
                let lhs = self.get_type(left)?;
                let rhs = self.get_type(right)?;

                if lhs != Type::U8 || rhs != Type::U8 {
                    todo!("report boolean operands aren't booleans")
                }
                if lhs != rhs {
                    todo!("report type mismatch");
                }
            }
            Expression::BinaryOperation { operator: BinaryOperator::And, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Or, left, right, .. }=> {
                // TODO: add conversion when traits are added
                // TODO: make it so that types get upgraded if they are compatable

                let lhs = match self.get_type(left) {
                    Ok(ty) => Some(ty),
                    Err(TypeCheckerError::UnableToDeduceType {..}) => None,
                };
                let rhs = match self.get_type(right) {
                    Ok(ty) => Some(ty),
                    Err(TypeCheckerError::UnableToDeduceType {..}) => None,
                };

                let (lhs, rhs) = match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => (lhs, rhs),
                    (Some(lhs), None) => {
                        self.annotate_expr(&lhs, right.as_mut())?;
                        (lhs.clone(), lhs)
                    }
                    (None, Some(rhs)) => {
                        self.annotate_expr(&rhs, left.as_mut())?;
                        (rhs.clone(), rhs)
                    }
                    _ => todo!("report missing type information"),
                };

                if lhs != rhs && (lhs != Type::U8 || rhs != Type::U8) {
                    todo!("report type mismatch for logical and or logical or {:?} {:?}", lhs, rhs);
                }
            }
            Expression::BinaryOperation { operator: BinaryOperator::Index, left, right, .. } => {
                // TODO: add conversion when traits are added

                let lhs = self.get_type(left)?;
                match lhs {
                    Type::TypeArg(obj, _,_) => {
                        match obj.as_ref() {
                            Type::Object(arr,_) if arr.as_str() == "Array" => {}
                            _ => todo!("add support for non-array objects with indexing"),
                        }
                    }
                    Type::Array(_, _) => {}
                    _ => todo!("add support for non-array objects with indexing"),
                }
                self.annotate_expr(&Type::U64, right.as_mut())?;
                let rhs = self.get_type(right)?;
                match rhs {
                    Type::U64 => {}
                    _ => todo!("add support for non-array objects with indexing anything other than u64")
                }
            }
            Expression::BinaryOperation { operator, left, right, .. } => {
                // TODO: add conversion when traits are added
                // TODO: make it so that types get upgraded if they are compatable


                let lhs = match self.get_type(left) {
                    Ok(ty) => Some(ty),
                    Err(TypeCheckerError::UnableToDeduceType {..}) => None,
                };
                let rhs = match self.get_type(right) {
                    Ok(ty) => Some(ty),
                    Err(TypeCheckerError::UnableToDeduceType {..}) => None,
                };

                let (lhs, rhs) = match (lhs, rhs) {
                    (Some(lhs), Some(rhs)) => (lhs, rhs),
                    (Some(lhs), None) => {
                        self.annotate_expr(&lhs, right.as_mut())?;
                        (lhs.clone(), lhs)
                    }
                    (None, Some(rhs)) => {
                        self.annotate_expr(&rhs, left.as_mut())?;
                        (rhs.clone(), rhs)
                    }
                    _ => todo!("report missing type information"),
                };


                if lhs != rhs {
                    todo!("report type mismatch {:?} {:?}", lhs, rhs);
                }
            }
            Expression::UnaryOperation { operator: UnaryOperator::Neg, operand, .. } => {
                let ty = self.get_type(operand)?;
                // TODO check if ty is a numeric type
            }
            Expression::UnaryOperation { operator: UnaryOperator::Not, operand, .. } => {
                let ty = self.get_type(operand)?;

                if ty != Type::U8 {
                    todo!("report boolean operands aren't booleans")
                }
            }
            Expression::Variable(name, annotation, _) => {
                if let Some(ty) = self.lookup_var(name) {
                    // annotate the expression with the type
                    *annotation = Some(ty.clone().into());
                } else {
                    todo!("report unbound variable");
                }
            }
            Expression::Call { name, type_args: _, args, .. } => {
                self.check_expr(return_type, name)?;
                //println!("{:?}", name);
                let method = self.get_type(name)?;

                for (i, arg) in args.iter_mut().enumerate() {
                    // check each argument in the call
                    self.check_expr(return_type, arg)?;
                    let arg_ty = self.get_type(arg)?;
                    match &method {
                        Type::Function(arg_types, ..) => {
                            if i < arg_types.len() {
                                let expected_ty = &arg_types[i];
                                //println!("left: {:?}\nright: {:?}", arg_ty, expected_ty);
                                if !self.compare_types(&TypeCheckerType::from(&arg_ty), &TypeCheckerType::from(expected_ty)) {
                                    todo!("report type mismatch for argument {} in method call {:?} ({:?}, {:?})", i, name, arg_ty, expected_ty);
                                }
                            } else {
                                todo!("report too many arguments in method call");
                            }
                        
                        }
                        _ => unreachable!("expected method to be a function type but got {:?}", method),
                    }
                }
            }
            Expression::StaticCall { name, type_args: _, args, .. } => {
                let class_name = if self.active_paths.contains_key(name.segments[0].as_str()) {
                    let mut active_path = self.active_paths.get(name.segments[0].as_str()).unwrap().clone();
                    active_path.extend(
                        name.segments[1..name.segments.len() - 1].iter()
                        .map(ToString::to_string)
                    );
                    active_path
                } else {
                    name.segments[..name.segments.len() - 1].iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                };
                let method_name = &name.segments[name.segments.len() - 1];

                let (_, attributes) = self.class_information.get(&class_name)
                    .expect("class missing or not loaded");

                let ClassAttribute::Method(method) = attributes.get(method_name.as_str())
                    .expect("method missing or not loaded") else {
                    todo!("report attribute not a method")
                };

                let method = method.clone();

                for (i, arg) in args.iter_mut().enumerate() {
                    // check each argument in the call
                    self.check_expr(return_type, arg)?;
                    let arg_ty = self.get_type(arg)?;
                    match &method {
                        TypeCheckerType::Function(arg_types, _) => {
                            if i < arg_types.len() {
                                let expected_ty = &arg_types[i];
                                //println!("left: {:?}\nright: {:?}", arg_ty, expected_ty);
                                if !self.compare_types(&TypeCheckerType::from(&arg_ty), expected_ty) {
                                    todo!("report type mismatch for argument {} in method call {:?} ({:?}, {:?})", i, name, arg_ty, expected_ty);
                                }
                            } else {
                                todo!("report too many arguments in method call");
                            }

                        }
                        _ => unreachable!("expected method to be a function type but got {:?}", method),
                    }
                }

            }
            Expression::MemberAccess { object, field, annotation, .. } => {
                self.check_expr(return_type, object)?;
                let ty = self.get_type(object.as_mut())?;
                let name = match ty {
                    Type::Object(name, _) => name,
                    Type::TypeArg(obj, _, _) => {
                        let Type::Object(name, _) = obj.as_ref() else {
                            unreachable!("type arg should always be an object")
                        };
                        name.clone()
                    }
                    _ => todo!("member access is incomplete"),
                };
                let class_name = name;
                let path = self.attach_module_if_needed(class_name.to_string());
                if path.len() == 0 {
                    todo!("report missing import");
                }

                let member_name = &field.segments[field.segments.len() - 1];

                let (_, attributes) = self.class_information.get(&path)
                    .expect("class missing or not loaded");


                let member = match attributes.get(member_name.as_str()) {
                    Some(ClassAttribute::Method(method)) => method,
                    Some(ClassAttribute::Member(member)) => member,
                    None => todo!("member access is incomplete"),
                };

                *annotation = Some(member.into());

            }
            Expression::Literal(Literal::Array(body, typ, _)) => {
                for body in body {
                    self.check_expr(return_type, body)?;
                    if let Some(typ) = typ {
                        if self.get_type(body)? != *typ {
                            todo!("report type mismatch in array body")
                        }
                    }
                }
            }
            Expression::Return(value, _) => {
                let result = value.as_mut().map(|mut value| {
                    self.annotate_expr(&return_type.into(), value.as_mut())?;
                    let ty = self.get_type(value.as_mut())?;
                    if <&TypeCheckerType as Into<Type>>::into(return_type) != ty {
                        todo!("report type mismatch in return value")
                    } else {
                        Ok(())
                    }
                });
                if let Some(result) = result {
                    _ = result?;
                }
            }
            Expression::New(_, arr_size, _) => {
                if let Some(arr_size) = arr_size {
                    let ty = self.get_type(arr_size.as_mut())?;

                }
            }
            _ => {}
        }

        Ok(())
    }

    fn check_if_expr<'a>(&mut self, return_type: &TypeCheckerType, expr: &mut crate::ast::IfExpression<'a>) -> Result<(), TypeCheckerError> {
        use crate::ast::IfExpression;
        let IfExpression { condition, then_branch, else_branch, .. } = expr;

        self.annotate_expr(&Type::U8, condition.as_mut())?;
        let condition_type = self.get_type(condition.as_mut())?;
        if condition_type != Type::U8 {
            todo!("report type mismatch if condition");
        }

        self.check_body(return_type, then_branch)?;

        match else_branch {
            Some(Either::Left(expr)) => {
                self.check_if_expr(return_type, expr)?;
            }
            Some(Either::Right(else_branch)) => {
                self.check_body(return_type, else_branch)?;
            }
            None => {}
        }

        Ok(())
    }

    fn get_type<'a>(&self, expr: &mut crate::ast::Expression<'a>) -> Result<Type<'a>, TypeCheckerError> {
        use crate::ast::{Expression, Literal, Constant};
        //println!("Expression: {:#?}", expr);
        match expr {
            Expression::Literal(Literal::Constant(Constant::Bool(_, _))) => Ok(Type::U8),
            Expression::Literal(Literal::Constant(Constant::Float(_, annotation, span))) => match annotation {
                Some(ty) => Ok(ty.clone()),
                None => Err(TypeCheckerError::UnableToDeduceType { start: span.start, end: span.end }),
            },
            Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, span))) => match annotation {
                Some(ty) => Ok(ty.clone()),
                None => Err(TypeCheckerError::UnableToDeduceType { start: span.start, end: span.end }),
            },
            Expression::Literal(Literal::Constant(Constant::Character(_, _))) => Ok(Type::Char),
            Expression::Variable(name, annotation, _) => {
                if let Some(ty) = self.lookup_var(&name) {
                    *annotation = Some(ty.into());
                    //println!("annotation: {:?}", annotation);
                    Ok(ty.into())
                } else {
                    todo!("report unbound variable {}", name);
                }
            }
            Expression::This(_) => {

                Ok(TypeCheckerType::Object(self.current_class.last().unwrap().clone()).into())
            }
            Expression::As { source, typ, .. } => {
                let _source_ty = self.get_type(source.as_mut())?;
                let target_ty = typ.clone(); // convert to Type
                // TODO: check if source_ty can be converted to target_ty
                Ok(target_ty)
            }
            Expression::Into { source, typ, .. } => {
                let _source_ty = self.get_type(source.as_mut())?;
                let target_ty = typ.clone(); // convert to Type
                // TODO: check if source_ty can be converted binary wise to target_ty
                Ok(target_ty)
            }
            Expression::New(object, arr_size, span) => {
                if let Some(arr_size) = arr_size {
                    self.get_type(arr_size.as_mut())?;
                    return Ok(Type::Array(Box::new(object.clone()), span.clone()))
                }
                Ok(object.clone())
            }
            Expression::Call { name, annotation, .. } => {
                if let Some(ty)= annotation {
                    Ok(ty.clone())
                } else {
                    let ty = self.get_type(name.as_mut())?;
                    let ty = match ty {
                        Type::Function(_, ret_type, _) => {
                            *ret_type.clone()
                        }
                        _ => unreachable!("something other than function")
                    };
                    *annotation = Some(ty.clone());
                    Ok(ty)
                }
            }
            Expression::StaticCall { name, annotation, .. } => {
                if let Some(ty)= annotation {
                    Ok(ty.clone())
                } else {
                    let class_name = if self.active_paths.contains_key(name.segments[0].as_str()) {
                        let mut active_path = self.active_paths.get(name.segments[0].as_str()).unwrap().clone();
                        active_path.extend(
                            name.segments[1..name.segments.len() - 1].iter()
                                .map(ToString::to_string)
                        );
                        active_path
                    } else {
                        name.segments[..name.segments.len() - 1].iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                    };


                    let attribute = self.get_attribute(
                        &class_name,
                        &name.segments[name.segments.len() - 1]
                    );
                    let Some(ClassAttribute::Method(ty)) = attribute else {
                        todo!("report missing attribute")
                    };
                    let ty = match ty {
                        TypeCheckerType::Function(_, ret_type) => {
                            *ret_type.clone()
                        }
                        _ => unreachable!("something other than function")
                    };
                    let ty = <TypeCheckerType as Into<Type>>::into(ty);
                    *annotation = Some(ty.clone());
                    Ok(ty)
                }
            }
            Expression::MemberAccess { .. } => {
                self.get_type_member_access(expr)
            }
            Expression::BinaryOperation { operator: BinaryOperator::Add, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Sub, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Mul, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Div, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Mod, left, right, .. } => {
                let lhs = self.get_type(left.as_mut());
                let rhs = self.get_type(right.as_mut());

                match (lhs, rhs) {
                    (Ok(ty), Err(_)) => {
                        self.annotate_expr(&ty, right.as_mut())?;
                    }
                    (Err(_), Ok(ty)) => {
                        self.annotate_expr(&ty, left.as_mut())?;
                    }
                    (Err(e), Err(_)) => {
                        return Err(e);
                    }
                    _ => {}
                }

                let lhs = self.get_type(left.as_mut())?;
                let rhs = self.get_type(right.as_mut())?;

                match (lhs, rhs) {
                    (Type::F32, _) | (_, Type::F32) => {
                        Ok(Type::F32)
                    }
                    (Type::F64, _) | (_, Type::F64) => {
                        Ok(Type::F64)
                    }
                    (lhs, rhs) => {
                        if lhs != rhs {
                            todo!("Report mismatch type")
                        }
                        Ok(lhs)
                    }
                }
            }
            Expression::BinaryOperation { operator: BinaryOperator::Eq, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Ne, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Lt, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Le, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Gt, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Ge, .. } => {
                Ok(Type::U8)
            }
            Expression::BinaryOperation { operator: BinaryOperator::And, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Or, .. }=> {
                Ok(Type::U8)
            }
            Expression::Literal(Literal::Array(_, ty, _)) => {
                if let Some(ty) = ty {
                    Ok(Type::Array(Box::new(ty.clone()), Span::new(0, 0)))
                } else {
                    todo!("report lack of array type")
                }
            }
            Expression::BinaryOperation { operator: BinaryOperator::Index, left, .. } => {
                match self.get_type(left.as_mut())? {
                    Type::Array(ty, _) => {
                        Ok(*ty.clone())
                    }
                    _ => {
                        todo!("add in trait support so we can index other things than just arrays")
                    }
                }
            }
            x => todo!("finish get_type: {:?}", x),
        }
    }

    fn get_type_member_access<'a>(&self, expr: &mut crate::ast::Expression<'a>) -> Result<Type<'a>, TypeCheckerError> {
        match expr {
            Expression::MemberAccess { object, field, annotation, .. } => {
                match object.as_mut() {
                    Expression::Variable(name,ty, _) => {
                        let var_ty = self.lookup_var(name) // lookup the type of the variable
                            .ok_or_else(|| {
                                todo!("report unbound variable in member access")
                            })?;
                        *ty = Some(var_ty.into()); // annotate the type of the variable
                        match var_ty {
                            TypeCheckerType::Object(name) => {
                                match self.get_attribute(&[name.to_string()], field.to_string()) {
                                    Some(ClassAttribute::Member(ty)) => {
                                        *annotation = Some(ty.into());
                                        Ok(ty.clone().into())
                                    }
                                    Some(ClassAttribute::Method(ty)) => {
                                        *annotation = Some(ty.into());
                                        Ok(ty.clone().into())
                                    }
                                    _ => {
                                        eprintln!("Failed to find attribute {} in class {}", field.to_string(), name);
                                        todo!("report unknown member access")
                                    }
                                }
                            }
                            TypeCheckerType::TypeArg(obj, args) => {
                                match obj.as_ref() {
                                    TypeCheckerType::Object(name) => {
                                        let path = self.attach_module_if_needed(name.to_string());

                                        match self.get_attribute(&path, field.to_string()) {
                                            Some(ClassAttribute::Member(ty)) => {
                                                *annotation = Some(ty.into());
                                                Ok(ty.clone().into())
                                            }
                                            Some(ClassAttribute::Method(ty)) => {
                                                *annotation = Some(ty.into());
                                                Ok(ty.clone().into())
                                            }
                                            _ => {
                                                eprintln!("Failed to find attribute {} in class {}", field.to_string(), name);
                                                todo!("report unknown member access")
                                            }
                                        }
                                    }
                                    _ => unreachable!("Only object types can have type parameters")
                                }
                            }
                            TypeCheckerType::Array(ty) => {
                                let path = self.attach_module_if_needed(String::from("Array"));

                                match self.get_attribute(&path, field.to_string()) {
                                    Some(ClassAttribute::Member(ty)) => {
                                        *annotation = Some(ty.into());
                                        Ok(ty.clone().into())
                                    }
                                    Some(ClassAttribute::Method(ty)) => {
                                        *annotation = Some(ty.into());
                                        Ok(ty.clone().into())
                                    }
                                    _ => {
                                        eprintln!("Failed to find attribute {} in class Array", field.to_string());
                                        todo!("report unknown member access")
                                    }
                                }
                            }
                            _ => todo!("report member access on non-object type"),
                        }
                    }
                    Expression::This(_) => {
                        match self.get_attribute(&self.current_class, field.to_string()) {
                            Some(ClassAttribute::Member(ty)) => {
                                *annotation = Some(ty.into());
                                Ok(ty.clone().into())
                            }
                            Some(ClassAttribute::Method(ty)) => {
                                *annotation = Some(ty.into());
                                Ok(ty.clone().into())
                            }
                            _ => {
                                // TODO: change this to look at base classes
                                match self.get_attribute(&[String::from("Object")], field.to_string()) {
                                    Some(ClassAttribute::Member(ty)) => {
                                        *annotation = Some(ty.into());
                                        Ok(ty.clone().into())
                                    }
                                    Some(ClassAttribute::Method(ty)) => {
                                        *annotation = Some(ty.into());
                                        Ok(ty.clone().into())
                                    }
                                    _ => {
                                        eprintln!("Failed to find attribute {} in class {:?}", field.to_string(), self.current_class);
                                        todo!("report unknown member access")
                                    }
                                }

                            }
                        }
                    }
                    Expression::MemberAccess {..} => {
                        let ty = self.get_type_member_access(object.as_mut())?;
                        let name = match ty {
                            Type::Object(name, _) => name,
                            Type::Array(ty, _) => {
                                Text::Borrowed("Array")
                            },
                            _ => unreachable!("Only object types can have type parameters"),
                        };
                        match self.get_attribute(&[name.to_string()], field.to_string()) {
                            Some(ClassAttribute::Member(ty)) => {
                                *annotation = Some(ty.into());
                                Ok(ty.clone().into())
                            }
                            Some(ClassAttribute::Method(ty)) => {
                                *annotation = Some(ty.into());
                                Ok(ty.clone().into())
                            }
                            _ => {
                                eprintln!("Failed to find attribute {} in class {}", field.to_string(), name.to_string());
                                todo!("report unknown member access")
                            }
                        }

                    },
                    x => todo!("report member access on non-variable expression: {:?}", x),
                }
            }
            x =>  self.get_type(x),
        }
    }
    

    fn annotate_expr<'a, E: BorrowMut<crate::ast::Expression<'a>>>(&self, ty: &Type<'a>, mut value: E) -> Result<(), TypeCheckerError> {
        use crate::ast::{Expression, Literal, Constant, BinaryOperator};
        match (ty, value.borrow_mut()) {
            (Type::U8, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::U8);
            }
            (Type::U16, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::U16);
            }
            (Type::U32, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::U32);
            }
            (Type::U64, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::U64);
            }
            (Type::I8, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::I8);
            }
            (Type::I16, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::I16);
            }
            (Type::I32, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::I32);
            }
            (Type::I64, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::I64);
            }
            (Type::F32, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::F32);
            }
            (Type::F64, Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _)))) => {
                *annotation = Some(Type::F64);
            }
            (Type::F32, Expression::Literal(Literal::Constant(Constant::Float(_, annotation, _)))) => {
                *annotation = Some(Type::F32);
            }
            (Type::F64, Expression::Literal(Literal::Constant(Constant::Float(_, annotation, _)))) => {
                *annotation = Some(Type::F64);
            }
            (ty, Expression::Variable(var, annotation, _)) => {
                if let Some(var_ty) = self.lookup_var(var) {
                    if self.compare_types(var_ty, &TypeCheckerType::from(ty)) {
                        *annotation = Some(ty.clone());
                    } else {
                        todo!("report type mismatch");
                    }
                }
            }
            (Type::Tuple(tys, _), Expression::Literal(Literal::Tuple(exprs, annotation, _))) => {
                let mut type_vec = Vec::new();
                for (ty, expr) in tys.iter().zip(exprs.iter_mut()) {
                    self.annotate_expr(ty, expr)?;
                    type_vec.push(ty.clone());
                }
                *annotation = Some(Type::Tuple(type_vec, Span::new(0, 0)));
            }
            (Type::Array(ty, _), Expression::Literal(Literal::Array(exprs, annotation, _))) => {
                for expr in exprs.iter_mut() {
                    self.annotate_expr(ty, expr)?;
                }
                *annotation = Some(*ty.clone());
            }
            (
                ty, Expression::BinaryOperation {
                operator: BinaryOperator::Add, left, right, .. }
            ) |
            (
                ty, Expression::BinaryOperation {
                operator: BinaryOperator::Sub, left, right, .. }
            ) |
            (
                ty, Expression::BinaryOperation {
                operator: BinaryOperator::Mul, left, right, .. }
            ) |
            (
                ty, Expression::BinaryOperation {
                operator: BinaryOperator::Div, left, right, .. }
            ) |
            (
                ty, Expression::BinaryOperation {
                operator: BinaryOperator::Mod, left, right, .. }
            ) => {
                self.annotate_expr(ty, left.as_mut())?;
                self.annotate_expr(ty, right.as_mut())?;
            }
            (_, Expression::BinaryOperation {
                operator: BinaryOperator::Index, left, right, .. }) => {
                let ty = self.get_type(left.as_mut())?;
                self.annotate_expr(&ty, left.as_mut())?;
                let ty = self.get_type(right.as_mut())?;
                self.annotate_expr(&ty, right.as_mut())?;
            }
            (ty, Expression::Parenthesized(expr, _)) => {
                self.annotate_expr(ty, expr.as_mut())?;
            }
            (ty, Expression::Call { name, annotation, span, ..}) => {
                let access_ty = self.get_type(name.as_mut())?;

                match access_ty {
                    Type::Function(_, ret_ty, _) => {
                        if !self.compare_types(&TypeCheckerType::from(ty), &TypeCheckerType::from(ret_ty.as_ref())) {
                            todo!("report type mismatch")
                        }
                    }
                    x => todo!("report not a function: {:?} spanning: {:?}", x, span)
                }
                *annotation = Some(ty.clone());
            }
            (ty, Expression::StaticCall { name, annotation, ..}) => {
                let path = if self.active_paths.contains_key(name.segments[0].as_str()) {
                    let mut path = self.active_paths.get(name.segments[0].as_str()).unwrap().clone();
                    path.extend(name.segments[1..name.segments.len() - 1].iter().map(ToString::to_string));
                    path
                } else {
                    name.segments[..name.segments.len() - 1].iter().map(ToString::to_string).collect()
                };

                let (_, attributes) = self.class_information.get(&path).unwrap();
                let ClassAttribute::Method(access_ty) = attributes.get(name.segments.last().unwrap().as_str()).unwrap() else {
                    unreachable!("report missing method");
                };

                match access_ty {
                    TypeCheckerType::Function(_, ret_ty) => {
                        if !self.compare_types(&TypeCheckerType::from(ty), ret_ty.as_ref()) {
                            todo!("report type mismatch")
                        }
                    }
                    _ => todo!("report not a function")
                }
                *annotation = Some(ty.clone());
            }
            (_, Expression::BinaryOperation { operator: BinaryOperator::Lt, left, right, .. })
            | (_, Expression::BinaryOperation { operator: BinaryOperator::Gt, left, right, .. })
            | (_, Expression::BinaryOperation { operator: BinaryOperator::Le, left, right, .. })
            | (_, Expression::BinaryOperation { operator: BinaryOperator::Ge, left, right, .. })
            | (_, Expression::BinaryOperation { operator: BinaryOperator::Eq, left, right, .. })
            | (_, Expression::BinaryOperation { operator: BinaryOperator::Ne, left, right, .. }) => {
                let lhs = self.get_type(left.as_mut())?;
                self.annotate_expr(&lhs, right.as_mut())?;
            }
            (ty, Expression::MemberAccess { annotation, ..}) => {
                *annotation = Some(ty.clone());
            }
            (_, Expression::New(_, arr_size, _)) => {
                if let Some(arr_size) = arr_size {
                    self.annotate_expr(&Type::U64, arr_size.as_mut())?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

