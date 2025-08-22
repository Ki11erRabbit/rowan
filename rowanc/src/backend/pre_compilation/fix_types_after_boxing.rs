use std::collections::HashMap;
use either::Either;
use crate::trees::ir::{Class, Expression, File, IfExpression, Method, Parameter, Pattern, Statement, TopLevelStatement, Trait, TraitImpl};
use crate::trees::{PathName, Span, Text, Type};

pub struct Frame<'fix> {
    pub variables: HashMap<String, Type<'fix>>
}

impl<'fix> Frame<'fix> {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new()
        }
    }
}

/// This IR pass will do two things:
/// 1. Correct types that have been boxed but the annotations don't match
/// 2. Change boxed variables to access the value member of their box
pub struct FixTypesAfterBoxing<'fix> {
    frames: Vec<Frame<'fix>>,
}

impl<'fix> FixTypesAfterBoxing<'fix> {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }
    
    fn push_frame(&mut self) {
        self.frames.push(Frame::new());
    }
    
    fn pop_frame(&mut self) {
        self.frames.pop();
    }
    
    fn bind_variable(&mut self, name: &str, ty: Type<'fix>) {
        self.frames.last_mut().unwrap().variables.insert(name.to_string(), ty);
    }
    
    fn lookup_variable(&self, name: &str) -> Option<Type<'fix>> {
        for frame in self.frames.iter().rev() {
            if let Some(ty) = frame.variables.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
    
    pub fn fix_file(&mut self, file: File<'fix>) -> File<'fix> {
        let File {
            path,
            content,
        } = file;
        
        let mut new_content = Vec::new();
        for item in content {
            match item {
                TopLevelStatement::Import(import) => {
                    new_content.push(TopLevelStatement::Import(import));
                }
                TopLevelStatement::Class(class) => {
                    new_content.push(TopLevelStatement::Class(self.fix_class(class)));
                }
                TopLevelStatement::Trait(r#trait) => {
                    let Trait {
                        name, 
                        parents, 
                        methods, 
                        type_params,
                        span
                    } = r#trait;
                    let trait_name = name.clone();
                    
                    let mut new_methods = Vec::new();
                    for method in methods {
                        let Method {
                            name,
                            is_native,
                            annotations,
                            visibility,
                            type_params,
                            parameters,
                            return_type,
                            mut body,
                            span
                        } = method;
                        self.push_frame();
                        for param in &parameters {
                            match &param {
                                Parameter::Pattern { name, ty, .. } => {
                                    self.bind_vars(name, ty);
                                }
                                Parameter::This(..) => {
                                    self.bind_variable("this", Type::Existential(Box::new(Type::Object(trait_name.clone(), Span::new(0,0)))));
                                }
                            }
                        }

                        self.fix_body(&mut body);
                        self.pop_frame();

                        new_methods.push(Method {
                            name,
                            is_native,
                            annotations,
                            visibility,
                            type_params,
                            parameters,
                            return_type,
                            body,
                            span
                        });
                    }
                    let methods = new_methods;
                    
                    let r#trait = Trait {
                        name,
                        parents,
                        methods,
                        type_params,
                        span,
                    };
                    new_content.push(TopLevelStatement::Trait(r#trait));
                }
                TopLevelStatement::TraitImpl(r#impl) => {
                    let TraitImpl {
                        r#trait, 
                        implementer, 
                        methods, 
                        type_params, 
                        span
                    } = r#impl;

                    let mut new_methods = Vec::new();
                    for method in methods {
                        let Method {
                            name,
                            is_native,
                            annotations,
                            visibility,
                            type_params,
                            parameters,
                            return_type,
                            mut body,
                            span
                        } = method;
                        self.push_frame();
                        for param in &parameters {
                            match &param {
                                Parameter::Pattern { name, ty, .. } => {
                                    self.bind_vars(name, ty);
                                }
                                Parameter::This(..) => {
                                    self.bind_variable("this", Type::Existential(Box::new(implementer.clone())));
                                }
                            }
                        }

                        self.fix_body(&mut body);
                        self.pop_frame();

                        new_methods.push(Method {
                            name,
                            is_native,
                            annotations,
                            visibility,
                            type_params,
                            parameters,
                            return_type,
                            body,
                            span
                        });
                    }
                    let methods = new_methods;
                    
                    let r#impl = TraitImpl {
                        r#trait,
                        implementer,
                        methods,
                        type_params,
                        span,
                    };
                    new_content.push(TopLevelStatement::TraitImpl(r#impl));
                }
            }
        }
        let content = new_content;
        
        File {
            path,
            content,
        }
    }
    
    fn fix_class(&mut self, class: Class<'fix>) -> Class<'fix> {
        let Class {
            name, 
            parent, 
            members, 
            methods, 
            static_members, 
            type_params, 
            span
        } = class;
        
        let mut new_methods = Vec::new();
        for method in methods {
            let Method {
                name, 
                is_native, 
                annotations, 
                visibility, 
                type_params, 
                parameters, 
                return_type, 
                mut body, 
                span
            } = method;
            self.push_frame();
            for param in &parameters {
                match &param {
                    Parameter::Pattern { name, ty, .. } => {
                        self.bind_vars(name, ty);
                    }
                    Parameter::This(..) => {
                        self.bind_variable("this", Type::Object(name.clone(), Span::new(0,0)));
                    }
                }
            }
            
            self.fix_body(&mut body);
            self.pop_frame();
            
            new_methods.push(Method {
                name,
                is_native,
                annotations,
                visibility,
                type_params,
                parameters,
                return_type,
                body,
                span
            });
        }
        let methods = new_methods;
        
        Class {
            name,
            parent,
            members,
            methods,
            static_members,
            type_params,
            span
        }
    }
    
    fn fix_body(&mut self, body: &mut Vec<Statement<'fix>>) {
        println!();
        for statement in body {
            //println!("{:?}", statement);
            match statement {
                Statement::Expression(expr, ..) => {
                    self.fix_expr(expr);
                }
                Statement::Let { bindings, ty, value, .. } => {
                    self.fix_expr(value);
                    //println!("bindings: {:?}", bindings);
                    self.bind_vars(bindings, ty);
                }
                Statement::Const { bindings, ty, value, .. } => {
                    self.fix_expr(value);
                    self.bind_vars(bindings, ty);
                }
                Statement::Assignment { target, value, .. } => {
                    self.fix_expr(target);
                    self.fix_expr(value);
                }
                Statement::While { test, body, .. } => {
                    self.fix_expr(test);
                    self.fix_body(body);
                }
                _ => todo!("complete remaining statements in fix_types_after_boxing"),
            }
        }
    }
    
    fn fix_expr(&mut self, expr: &mut Expression<'fix>) {
        match expr {
            Expression::Variable(var, ty, span) => {
                let Some(var_ty) = self.lookup_variable(var.as_str()) else {
                    panic!("Variable {} not found", var);
                };
                if var.as_str() == "q" {
                    println!("variable: {:?} {:?}", var_ty, ty);
                }
                match ty {
                    Type::U8 if var_ty != *ty => {}
                    Type::I8 if var_ty != *ty => {}
                    Type::U16 if var_ty != *ty => {}
                    Type::I16 if var_ty != *ty => {}
                    Type::U32 if var_ty != *ty => {}
                    Type::I32 if var_ty != *ty => {}
                    Type::U64 if var_ty != *ty => {}
                    Type::I64 if var_ty != *ty => {}
                    Type::F32 if var_ty != *ty => {}
                    Type::F64 if var_ty != *ty => {}
                    _ => return,
                }
                println!("var: {:?}", var_ty);
                *expr = Expression::MemberAccess {
                    object: Box::new(Expression::Variable(var.clone(), var_ty.clone(), span.clone())),
                    field: PathName::new(vec![Text::Borrowed("value")], Span::new(0, 0)),
                    span: *span,
                    annotation: ty.clone(),
                }
            }
            Expression::Literal(..) => {}
            Expression::This(..) => {}
            Expression::MemberAccess { .. } => {}
            Expression::Call { args, .. } => {
                for arg in args {
                    self.fix_expr(arg);
                }
            }
            Expression::StaticCall { args, .. } => {
                for arg in args {
                    self.fix_expr(arg);
                }
            }
            Expression::Closure { params, body, ..} => {
                self.push_frame();
                for param in params {
                    match &param.parameter {
                        Parameter::Pattern { name, ty, .. } => {
                            self.bind_vars(name, ty);
                        }
                        _ => unreachable!("Can't have a this parameter in a closure")
                    }
                }
                
                self.fix_body(body);
                
                self.pop_frame();
            }
            Expression::Parenthesized(expr, ..) => {
                self.fix_expr(expr.as_mut());
            }
            Expression::UnaryOperation { operand, .. } => {
                self.fix_expr(operand.as_mut());
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.fix_expr(left.as_mut());
                self.fix_expr(right.as_mut());
            }
            Expression::Return(expr, ..) => {
                if let Some(expr) = expr {
                    self.fix_expr(expr.as_mut());
                }
            }
            Expression::New(_, array_size, _) => {
                if let Some(array_size) = array_size {
                    self.fix_expr(array_size.as_mut());
                }
            }
            Expression::IfExpression(if_expr, ..) => {
                self.fix_expr_if(if_expr);
            }
            e => todo!("remaining expression in fix_types_after_boxing {e:?}"),
        }
    }
    
    fn bind_vars(&mut self, bindings: &Pattern<'fix>, ty: &Type<'fix>) {
        match (bindings, ty) {
            (Pattern::Variable(var, ..), ty) => {
                self.bind_variable(var.as_str(), ty.clone());
            }
            _ => todo!("complete remaining pattern bindings in fix_types_after_boxing")
        }
    }
    
    fn fix_expr_if(&mut self, expr: &mut IfExpression<'fix>) {
        let IfExpression {
            condition, 
            then_branch, 
            else_branch, 
            ..
        } = expr;
        
        self.fix_expr(condition.as_mut());
        self.push_frame();
        self.fix_body(then_branch);
        self.pop_frame();
        
        match else_branch {
            Some(Either::Left(elif)) => {
                self.fix_expr_if(elif.as_mut());
            }
            Some(Either::Right(else_branch)) => {
                self.push_frame();
                self.fix_body(else_branch);
                self.pop_frame();
            }
            None => {}
        }
    }
}