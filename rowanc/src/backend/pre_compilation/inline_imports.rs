use std::collections::HashMap;
use either::Either;
use crate::trees::ir::{Class, Expression, File, IfExpression, Literal, Method, Parameter, Statement, TopLevelStatement, Trait, TraitImpl};
use crate::trees::{PathName, Text, Type};
use crate::trees::ir::ClosureParameter;

pub struct InlineImports {
    pub imports: HashMap<String, String>,
    pub current_path: String,
}


impl InlineImports {
    pub fn new() -> Self {

        let mut imports = HashMap::new();
        imports.insert("String".to_string(), "core::String".to_string());
        imports.insert("StringBuffer".to_string(), "core::StringBuffer".to_string());
        imports.insert("InternedString".to_string(), "core::InternedString".to_string());

        Self {
            imports,
            current_path: String::new(),
        }
    }

    pub fn inline_import<'inline>(&mut self, file: File<'inline>) -> File<'inline> {
        let File {
            path,
            content,
        } = file;
        self.current_path = path.segments.join("::");

        let mut new_content = Vec::new();

        for stmt in content {
            match stmt {
                TopLevelStatement::Import(import) => {
                    self.imports.insert(
                        import.path.segments.last().unwrap().to_string(),
                        import.path.segments.join("::")
                    );
                }
                x => new_content.push(x)
            }
        }
        let mut content = Vec::new();
        for stmt in new_content {
            match stmt {
                TopLevelStatement::Class(class) => {
                    let result = self.inline_class(class);
                    content.push(TopLevelStatement::Class(result));
                }
                TopLevelStatement::Trait(r#trait) => {
                    let result = self.inline_trait(r#trait);
                    content.push(TopLevelStatement::Trait(result));
                }
                TopLevelStatement::TraitImpl(r#impl) => {
                    let result = self.inline_impl(r#impl);
                    content.push(TopLevelStatement::TraitImpl(result));
                }
                _ => {}
            }
        }

        File {
            path,
            content,
        }
    }

    fn inline_class<'inline>(&mut self, class: Class<'inline>) -> Class<'inline> {
        let Class {
            name,
            parent,
            members,
            methods,
            static_members,
            type_params,
            span
        } = class;

        let parent = parent.map(|mut decl| {
            let path = self.imports.get(decl.name.as_str()).unwrap();
            decl.name = Text::Owned(path.to_string());
            decl
        });

        let mut new_methods = Vec::new();
        for method in methods {
            let result = self.inline_method(method);
            new_methods.push(result);
        }

        let methods = new_methods;

        Class {
            name,
            parent,
            members,
            methods,
            static_members,
            type_params,
            span,
        }
    }

    fn inline_trait<'inline>(&mut self, r#trait: Trait<'inline>) -> Trait<'inline> {
        let Trait {
            name,
            parents,
            methods,
            type_params,
            span
        } = r#trait;

        let mut new_parents = Vec::new();
        for parent in parents {
            let Type::Object(name, span) = parent else {
                unreachable!("we should have normalized all of parent types for traits")
            };
            let name = if let Some(path) = self.imports.get(name.as_str()) {
                path.clone()
            } else {
                format!("{}::{}", self.current_path, name)
            };
            let parent = Type::Object(Text::Owned(name), span);
            new_parents.push(parent);
        }
        let parents = new_parents;

        let mut new_methods = Vec::new();
        for method in methods {
            let result = self.inline_method(method);
            new_methods.push(result);
        }
        let methods = new_methods;

        Trait {
            name,
            parents,
            methods,
            type_params,
            span,
        }
    }

    fn inline_impl<'inline>(&mut self, r#impl: TraitImpl<'inline>) -> TraitImpl<'inline> {
        let TraitImpl {
            r#trait,
            implementer,
            methods,
            type_params,
            span
        } = r#impl;
        let trait_span = span;

        let Type::Object(name, span) = r#trait else {
            unreachable!("we should have normalized the trait name for impls")
        };

        let name = if let Some(path) = self.imports.get(name.as_str()) {
            path.clone()
        } else {
            format!("{}::{}", self.current_path, name)
        };

        let r#trait = Type::Object(Text::Owned(name), span);

        let Type::Object(name, span) = implementer else {
            unreachable!("we should have normalized the trait implementer for impls")
        };

        let name = if let Some(path) = self.imports.get(name.as_str()) {
            path.clone()
        } else {
            format!("{}::{}", self.current_path, name)
        };

        let implementer = Type::Object(Text::Owned(name), span);

        let mut new_methods = Vec::new();
        for method in methods {
            let result = self.inline_method(method);
            new_methods.push(result);
        }
        let methods = new_methods;

        let span = trait_span;
        TraitImpl {
            r#trait,
            implementer,
            methods,
            type_params,
            span,
        }
    }

    fn inline_method<'inline>(&mut self, method: Method<'inline>) -> Method<'inline> {
        let Method {
            name,
            is_native,
            annotations,
            visibility,
            type_params,
            parameters,
            return_type,
            body,
            span
        } = method;

        let mut new_params = Vec::new();
        for parameter in parameters {
            match parameter {
                Parameter::Pattern {
                    name,
                    ty,
                    span
                } => {
                    let ty = self.inline_type(ty);
                    new_params.push(Parameter::Pattern {
                        name,
                        ty,
                        span
                    });
                }
                x => new_params.push(x)
            }
        }
        let parameters = new_params;

        let return_type = self.inline_type(return_type);

        let body = self.inline_body(body);

        Method {
            name,
            is_native,
            annotations,
            visibility,
            type_params,
            parameters,
            return_type,
            body,
            span,
        }
    }

    fn inline_body<'inline>(&mut self, body: Vec<Statement<'inline>>) -> Vec<Statement<'inline>> {
        let mut new_body = Vec::new();

        for stmt in body {
            new_body.push(self.inline_statement(stmt));
        }

        new_body
    }

    fn inline_statement<'inline>(&mut self, statement: Statement<'inline>) -> Statement<'inline> {
        match statement {
            Statement::Let {
                bindings,
                ty,
                value,
                span
            } => {
                let ty = self.inline_type(ty);
                let value = self.inline_expression(value);

                Statement::Let {
                    bindings,
                    ty,
                    value,
                    span,
                }
            }
            Statement::Const {
                bindings,
                ty,
                value,
                span
            } => {
                let ty = self.inline_type(ty);
                let value = self.inline_expression(value);

                Statement::Const {
                    bindings,
                    ty,
                    value,
                    span,
                }
            }
            Statement::Assignment {
                target,
                value,
                span
            } => {
                let target = self.inline_expression(target);
                let value = self.inline_expression(value);

                Statement::Assignment {
                    target,
                    value,
                    span,
                }
            }
            Statement::Expression(expr, span) => {
                let expr = self.inline_expression(expr);
                Statement::Expression(expr, span)
            }
            Statement::While {
                label,
                test,
                body,
                span
            } => {
                let test = self.inline_expression(test);
                let body = self.inline_body(body);

                Statement::While {
                    label,
                    test,
                    body,
                    span
                }
            }
            _ => todo!("inline remaining statements")
        }
    }

    fn inline_expression<'inline>(&mut self, expr: Expression<'inline>) -> Expression<'inline> {
        match expr {
            Expression::Variable(name, annotation, span) => {
                Expression::Variable(name, self.inline_type(annotation), span)
            }
            Expression::Literal(Literal::Constant(c)) => {
                Expression::Literal(Literal::Constant(c))
            }
            Expression::Literal(Literal::Array(exprs, annotation, span)) => {
                let mut new_exprs = Vec::new();
                for expr in exprs {
                    new_exprs.push(self.inline_expression(expr));
                }
                let annotation = self.inline_type(annotation);
                Expression::Literal(Literal::Array(new_exprs, annotation, span))
            }
            Expression::Literal(Literal::Tuple(exprs, annotation, span)) => {
                let mut new_exprs = Vec::new();
                for expr in exprs {
                    new_exprs.push(self.inline_expression(expr));
                }
                let annotation = self.inline_type(annotation);
                Expression::Literal(Literal::Tuple(new_exprs, annotation, span))
            }
            Expression::This(span) => Expression::This(span),
            Expression::Call {
                name,
                type_args,
                args,
                span,
                annotation
            } => {
                let name = Box::new(self.inline_expression(*name));
                let mut new_args = Vec::new();
                for arg in args {
                    new_args.push(self.inline_expression(arg));
                }
                let args = new_args;
                let annotation = self.inline_type(annotation);

                Expression::Call {
                    name,
                    type_args,
                    args,
                    span,
                    annotation,
                }
            }
            Expression::StaticCall {
                name,
                type_args,
                args,
                span,
                annotation
            } => {
                let mut new_args = Vec::new();
                for arg in args {
                    new_args.push(self.inline_expression(arg));
                }
                let args = new_args;
                let annotation = self.inline_type(annotation);

                Expression::StaticCall {
                    name,
                    type_args,
                    args,
                    span,
                    annotation,
                }
            }
            Expression::MemberAccess {
                object,
                field,
                span,
                annotation
            } => {
                let object = Box::new(self.inline_expression(*object));
                let annotation = self.inline_type(annotation);

                Expression::MemberAccess {
                    object,
                    field,
                    span,
                    annotation,
                }
            }
            Expression::ClassAccess {
                class_name,
                span
            } => {
                let class_access_span = span;
                let PathName {
                    segments,
                    span
                } = class_name;

                let segments = if let Some(path) = self.imports.get(segments.first().unwrap().as_str()) {
                    path.split("::")
                        .map(String::from)
                        .map(Text::Owned)
                        .chain(segments.into_iter())
                        .collect::<Vec<_>>()
                } else {
                    segments
                };

                let class_name = PathName {
                    segments,
                    span,
                };

                let span = class_access_span;

                Expression::ClassAccess {
                    class_name,
                    span,
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                captures,
                processed_captures,
                span
            } => {
                let mut new_params = Vec::new();
                for param in params {
                    let ClosureParameter {
                        parameter,
                    } = param;
                    let Parameter::Pattern { name, ty, span } = parameter else {
                        unreachable!("`this` is not permitted in closures");
                    };
                    let ty = self.inline_type(ty);

                    let param = ClosureParameter {
                        parameter: Parameter::new_pattern(name, ty, span)
                    };
                    new_params.push(param);
                }
                let params = new_params;
                let return_type = self.inline_type(return_type);
                let body = self.inline_body(body);

                let mut new_captures = Vec::new();
                for (name, ty) in captures {
                    let ty = self.inline_type(ty);
                    new_captures.push((name, ty));
                }
                let captures = new_captures;

                Expression::Closure {
                    params,
                    return_type,
                    body,
                    captures,
                    processed_captures,
                    span,
                }
            }
            Expression::Parenthesized(value, span) => {
                let value = Box::new(self.inline_expression(*value));
                Expression::Parenthesized(value, span)
            }
            Expression::IfExpression(if_expr, span) => {
                let if_expr = self.inline_if_expr(if_expr);
                Expression::IfExpression(if_expr, span)
            }
            Expression::UnaryOperation {
                operator,
                operand,
                span
            } => {
                let operand = Box::new(self.inline_expression(*operand));

                Expression::UnaryOperation {
                    operator,
                    operand,
                    span,
                }
            }
            Expression::BinaryOperation {
                operator,
                left,
                right,
                span
            } => {
                let left = Box::new(self.inline_expression(*left));
                let right = Box::new(self.inline_expression(*right));

                Expression::BinaryOperation {
                    operator,
                    left,
                    right,
                    span,
                }
            }
            Expression::Return(expr, span) => {
                let expr = expr.map(|expr| {
                    Box::new(self.inline_expression(*expr))
                });
                Expression::Return(expr, span)
            }
            Expression::New(ty, array_size, span) => {
                let ty = self.inline_type(ty);
                let array_size = array_size.map(|array_size| {
                    Box::new(self.inline_expression(*array_size))
                });
                Expression::New(ty, array_size, span)
            }
            Expression::As {
                source,
                typ,
                span
            } => {
                let source = Box::new(self.inline_expression(*source));
                let typ = self.inline_type(typ);

                Expression::As { source, typ, span }
            }
            Expression::Into {
                source,
                typ,
                span
            } => {
                let source = Box::new(self.inline_expression(*source));
                let typ = self.inline_type(typ);
                Expression::Into { source, typ, span }
            }
            _ => todo!("inline imports for remaining expressions"),
        }
    }

    fn inline_if_expr<'inline>(&mut self, if_expr: IfExpression<'inline>) -> IfExpression<'inline> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
            start,
            end
        } = if_expr;

        let condition = Box::new(self.inline_expression(*condition));
        let then_branch = self.inline_body(then_branch);

        let else_branch = match else_branch {
            None => None,
            Some(Either::Left(if_expr)) => {
                Some(Either::Left(Box::new(self.inline_if_expr(*if_expr))))
            }
            Some(Either::Right(else_branch)) => {
                Some(Either::Right(self.inline_body(else_branch)))
            }
        };

        IfExpression {
            condition,
            then_branch,
            else_branch,
            start,
            end,
        }
    }

    fn inline_type<'inline>(&mut self, r#type: Type<'inline>) -> Type<'inline> {
        match r#type {
            Type::Object(name, span) => {
                let name = if let Some(path) = self.imports.get(name.as_str()) {
                    path.clone()
                } else {
                    format!("{}::{}", self.current_path, name)
                };
                Type::Object(Text::Owned(name), span)
            }
            Type::Array(inner, span) => {
                let ty = self.inline_type(*inner);
                Type::Array(Box::new(ty), span)
            }
            Type::Tuple(inner, span) => {
                let mut new_inner = Vec::new();
                for inner in inner {
                    new_inner.push(self.inline_type(inner));
                }
                Type::Tuple(new_inner, span)
            }
            Type::Existential(inner) => {
                Type::Existential(Box::new(self.inline_type(*inner)))
            }
            Type::Function(args, ret, span) => {
                let mut new_args = Vec::new();
                for arg in args {
                    new_args.push(self.inline_type(arg));
                }
                let ret = Box::new(self.inline_type(*ret));
                Type::Function(new_args, ret, span)
            }
            Type::TypeArg(..) => unreachable!("we should have normalized all of types"),
            x => x,
        }
    }
}