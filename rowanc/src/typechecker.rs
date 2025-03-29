use std::{borrow::BorrowMut, collections::HashMap};

use crate::ast::{Span, Type};

fn create_stdlib<'a>() -> HashMap<String, HashMap<String, ClassAttribute>> {
    let mut info = HashMap::new();
    let mut object_attributes = HashMap::new();
    object_attributes.insert("tick".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::F64], Box::new(TypeCheckerType::Void))));

    info.insert("Object".to_string(), object_attributes);
    
    info
}


#[derive(Debug)]
pub enum TypeCheckerError {

}

#[derive(Debug, PartialEq, PartialOrd)]
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
    class_information: HashMap<String, HashMap<String, ClassAttribute>>,
    scopes: Vec<Frame>,
}


impl TypeChecker {
    pub fn new() -> TypeChecker {
        TypeChecker {
            class_information: create_stdlib(),
            scopes: Vec::new(),
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

    pub fn check<'a>(&mut self, files: Vec<crate::ast::File<'a>>) -> Result<Vec<crate::ast::File<'a>>, TypeCheckerError> {
        self.check_files(files)
    }

    fn check_files<'a>(&mut self, mut files: Vec<crate::ast::File<'a>>) -> Result<Vec<crate::ast::File<'a>>, TypeCheckerError> {

        for file in files.iter_mut() {
            self.check_file(file)?;
        }
        

        Ok(files)
    }

    fn check_file<'a>(&mut self, file: &mut crate::ast::File<'a>) -> Result<(), TypeCheckerError> {
        for content in file.content.iter_mut() {
            match content {
                crate::ast::TopLevelStatement::Class(class) => {
                    self.check_class(class)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_class<'a>(&mut self, class: &mut crate::ast::Class<'a>) -> Result<(), TypeCheckerError> {
        let crate::ast::Class { name, members, methods, .. } = class;
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
                        argument_types.push(TypeCheckerType::Object(class_name.to_string()));
                    }
                    crate::ast::Parameter::Pattern { ty, .. } => {
                        argument_types.push(TypeCheckerType::from(ty.clone()));
                    }
                }
            }
            let ty = TypeCheckerType::Function(argument_types, Box::new(TypeCheckerType::from(return_type.clone())));
            class_attributes.insert(name.to_string(), ClassAttribute::Method(ty));
        }

        self.class_information.insert(class_name.to_string(), class_attributes);

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

        self.push_scope();

        self.check_body(TypeCheckerType::from(return_type.clone()), body)?;
        
        self.pop_scope();
        self.pop_scope();
        Ok(())
    }

    fn bind_pattern(&mut self, pattern: &crate::ast::Pattern, ty: &Type) {
        use crate::ast::Pattern;
        match (pattern, ty) {
            (Pattern::Variable(name, _,_), ty) => {
                self.insert_var(name, TypeCheckerType::from(ty))
            }
            (Pattern::Tuple(names, _), Type::Tuple(tys, _)) => {
                for (name, ty) in names.iter().zip(tys.iter()) {
                    self.bind_pattern(name, ty);
                }
            }
            _ => {}
        }
    }

    fn check_body<'a>(&mut self, return_type: TypeCheckerType, body: &mut Vec<crate::ast::Statement<'a>>) -> Result<(), TypeCheckerError> {

        for statement in body {
            self.check_statement(&return_type, statement)?;
        }
        
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

            }
        }

        Ok(())
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
                    if *var_ty == TypeCheckerType::from(ty) {
                        *annotation = Some(ty.clone());
                    } else {
                        todo!("report type mismatch");
                    }
                }
            }
            (Type::Tuple(tys, _), Expression::Literal(Literal::Tuple(exprs, _))) => {
                for (ty, expr) in tys.iter().zip(exprs.iter_mut()) {
                    self.annotate_expr(ty, expr)?;
                }
            }
            (ty, Expression::BinaryOperation {
                operator: BinaryOperator::Add, left, right, .. }) => {
                self.annotate_expr(ty, left.as_mut())?;
                self.annotate_expr(ty, right.as_mut())?;
            }
            (ty, Expression::Parenthesized(expr, _)) => {
                self.annotate_expr(ty, expr.as_mut())?;
            }
            _ => {}
        }
        Ok(())
    }
}

