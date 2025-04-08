use std::{borrow::BorrowMut, collections::HashMap};

use either::Either;

use crate::ast::{BinaryOperator, Literal, Span, Type};

fn create_stdlib<'a>() -> HashMap<String, HashMap<String, ClassAttribute>> {
    let mut info = HashMap::new();
    let mut object_attributes = HashMap::new();
    object_attributes.insert("tick".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::F64], Box::new(TypeCheckerType::Void))));

    info.insert("Object".to_string(), object_attributes);

    let mut printer_attributes = HashMap::new();
    printer_attributes.insert("println-int".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![TypeCheckerType::U64], Box::new(TypeCheckerType::Void))));

    info.insert("Printer".to_string(), printer_attributes);

    let mut array_attributes = HashMap::new();
    array_attributes.insert("len".to_string(), ClassAttribute::Method(TypeCheckerType::Function(vec![], Box::new(TypeCheckerType::U64))));

    info.insert("Array".to_string(), array_attributes);
    
    info
}


#[derive(Debug)]
pub enum TypeCheckerError {
    UnableToDeduceType {
        start: usize,
        end: usize,
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

#[derive(Debug, Clone, PartialEq)]
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

    fn get_attribute<S: AsRef<str>>(&self, class: S, attribute: S) -> Option<&ClassAttribute> {
        self.class_information.get(class.as_ref()).and_then(|attributes| {
            let out = attributes.get(attribute.as_ref());
            out
        })
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


        self.check_body(&TypeCheckerType::from(return_type.clone()), body)?;
        
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
                if *return_type != TypeCheckerType::Void {
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
                        println!("annotating right side");
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
                let method = self.get_type(name)?;

                for (i, arg) in args.iter_mut().enumerate() {
                    // check each argument in the call
                    self.check_expr(return_type, arg)?;
                    let arg_ty = self.get_type(arg)?;
                    match &method {
                        Type::Function(arg_types, return_type, _) => {
                            if i < arg_types.len() {
                                let expected_ty = &arg_types[i];
                                if arg_ty != *expected_ty {
                                    todo!("report type mismatch for argument {} in method call", i);
                                }
                            } else {
                                todo!("report too many arguments in method call");
                            }
                        
                        }
                        _ => unreachable!("expected method to be a function type but got {:?}", method),
                    }
                }
                
            }
            Expression::MemberAccess { object, field, .. } => {
                self.check_expr(return_type, object)?;
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
            _ => {}
        }

        Ok(())
    }

    fn check_if_expr<'a>(&mut self, return_type: &TypeCheckerType, expr: &mut crate::ast::IfExpression<'a>) -> Result<(), TypeCheckerError> {
        use crate::ast::IfExpression;
        let IfExpression { condition, then_branch, else_branch, .. } = expr;

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
                    Ok(ty.into())
                } else {
                    todo!("report unbound variable {}", name);
                }
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
            Expression::New(object, _, _) => {
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
            Expression::MemberAccess { object, field, .. } => {
                match object.as_mut() {
                    Expression::Variable(name,ty, _) => {
                        let var_ty = self.lookup_var(name) // lookup the type of the variable
                            .ok_or_else(|| {
                                todo!("report unbound variable in member access")
                            })?;
                        *ty = Some(var_ty.into()); // annotate the type of the variable
                        match var_ty {
                            TypeCheckerType::Object(name) => {
                                match self.get_attribute(name.to_string(), field.to_string()) {
                                    Some(ClassAttribute::Member(ty)) => {
                                        Ok(ty.clone().into())
                                    }
                                    Some(ClassAttribute::Method(ty)) => {
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
                                        match self.get_attribute(name.to_string(), field.to_string()) {
                                            Some(ClassAttribute::Member(ty)) => {
                                                Ok(ty.clone().into())
                                            }
                                            Some(ClassAttribute::Method(ty)) => {
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
                                match self.get_attribute(String::from("Array"), field.to_string()) {
                                    Some(ClassAttribute::Member(ty)) => {
                                        Ok(ty.clone().into())
                                    }
                                    Some(ClassAttribute::Method(ty)) => {
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
                    _ => todo!("report member access on non-variable expression"),
                }
            }
            Expression::BinaryOperation { operator: BinaryOperator::Add, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Sub, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Mul, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Div, left, right, .. }
            | Expression::BinaryOperation { operator: BinaryOperator::Mod, left, right, .. } => {
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
            (ty, Expression::BinaryOperation {
                operator: BinaryOperator::Add, left, right, .. }) => {
                self.annotate_expr(ty, left.as_mut())?;
                self.annotate_expr(ty, right.as_mut())?;
            }
            (ty, Expression::Parenthesized(expr, _)) => {
                self.annotate_expr(ty, expr.as_mut())?;
            }
            (ty, Expression::Call { name, annotation, ..}) => {
                let access_ty = self.get_type(name.as_mut())?;

                match access_ty {
                    Type::Function(_, ret_ty, _) => {
                        if *ty != *ret_ty {
                            todo!("report type mismatch")
                        }
                    }
                    _ => todo!("report not a function")
                }
                *annotation = Some(ty.clone());
            }
            _ => {}
        }
        Ok(())
    }
}

