use either::Either;
use crate::trees::*;

pub struct IRConverter {}

impl<'convert> IRConverter {
    pub fn new() -> Self {
        IRConverter {}
    }

    pub fn convert(&mut self, file: ast::File<'convert>) -> Result<ir::File<'convert>, ()> {
        let ast::File {
            path,
            content
        } = file;
        let content = self.convert_toplevel_statements(content)?;

        Ok(ir::File::new(path, content))
    }

    fn convert_toplevel_statements(
        &mut self,
        statements: Vec<ast::TopLevelStatement<'convert>>
    ) -> Result<Vec<ir::TopLevelStatement<'convert>>, ()> {
        let mut output = Vec::new();
        for statement in statements {
            let statement = self.convert_toplevel_statement(statement)?;
            output.push(statement);
        }
        Ok(output)
    }

    fn convert_toplevel_statement(&mut self, statement: ast::TopLevelStatement<'convert>) -> Result<ir::TopLevelStatement<'convert>, ()> {
        let result = match statement {
            ast::TopLevelStatement::Import(ast::Import{ path, span }) => {
                ir::TopLevelStatement::Import(ir::Import::new(path, span))
            }
            ast::TopLevelStatement::Class(class) => {
                ir::TopLevelStatement::Class(self.convert_class(class)?)
            }
        };

        Ok(result)
    }

    fn convert_class(&mut self, class: ast::Class<'convert>) -> Result<ir::Class<'convert>, ()> {
        let ast::Class {
            name,
            parent,
            members,
            methods,
            static_members,
            type_params,
            span
        } = class;

        let parent = if let Some(p) = parent {
            Some(self.convert_parent_dec(p)?)
        } else {
            None
        };
        let members = members.into_iter()
            .map(|m| self.convert_member(m))
            .collect::<Result<Vec<_>, _>>()?;
        let methods = methods.into_iter()
            .map(|m| self.convert_method(m))
            .collect::<Result<Vec<_>, _>>()?;
        let static_members = static_members.into_iter()
            .map(|sm| self.convert_static_member(sm))
            .collect::<Result<Vec<_>, _>>()?;

        let type_params = type_params.into_iter()
            .map(|tp| self.convert_type_param(tp))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ir::Class {
            name,
            parent,
            members,
            methods,
            static_members,
            type_params,
            span
        })
    }

    fn convert_parent_dec(&mut self, parent_dec: ast::ParentDec<'convert>) -> Result<ir::ParentDec<'convert>, ()> {
        let ast::ParentDec {
            name, type_args, type_params, span
        } = parent_dec;

        let type_params = type_params.into_iter()
            .map(|tp| self.convert_type_param(tp))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ir::ParentDec {
            name,
            type_args,
            type_params,
            span,
        })
    }

    fn convert_member(&mut self, member: ast::Member<'convert>) -> Result<ir::Member<'convert>, ()> {
        let ast::Member {
            visibility, name, ty, span
        } = member;

        Ok(ir::Member {
            visibility,
            name,
            ty,
            span,
        })
    }

    fn convert_static_member(&mut self, member: ast::StaticMember<'convert>) -> Result<ir::StaticMember<'convert>, ()> {
        let ast::StaticMember {
            visibility, is_const, name, ty, value, span
        } = member;

        let value = if let Some(v) = value {
            Some(self.convert_expression(v)?)
        } else {
            None
        };

        Ok(ir::StaticMember {
            visibility,
            is_const,
            name,
            ty,
            value,
            span,
        })
    }

    fn convert_type_param(&mut self, typ_param: ast::TypeParameter<'convert>) -> Result<ir::TypeParameter<'convert>, ()> {
        let ast::TypeParameter {
            name, constraints, span
        } = typ_param;

        Ok(ir::TypeParameter {
            name,
            constraints,
            span,
        })
    }

    fn convert_method(&mut self, method: ast::Method<'convert>) -> Result<ir::Method<'convert>, ()> {
        let ast::Method {
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

        let type_params = type_params.into_iter()
            .map(|tp| self.convert_type_param(tp))
            .collect::<Result<Vec<_>, _>>()?;

        let parameters = parameters.into_iter()
            .map(|p| self.convert_parameter(p))
            .collect::<Result<Vec<_>, _>>()?;

        let body = body.into_iter()
            .map(|stmt| self.convert_statement(stmt))
            .collect::<Result<Vec<_>, ()>>()?;

        Ok(ir::Method {
            name,
            is_native,
            annotations,
            visibility,
            type_params,
            parameters,
            return_type,
            body,
            span
        })
    }

    fn convert_parameter(&mut self, param: ast::Parameter<'convert>) -> Result<ir::Parameter<'convert>, ()> {
        match param {
            ast::Parameter::This(mutable, span) => {
                Ok(ir::Parameter::This(mutable, span))
            }
            ast::Parameter::Pattern {
                name, ty, span
            } => {
                let name = self.convert_pattern(name)?;

                Ok(ir::Parameter::Pattern {
                    name,
                    ty,
                    span,
                })
            }
        }
    }

    fn convert_pattern(&mut self, pattern: ast::Pattern<'convert>) -> Result<ir::Pattern<'convert>, ()> {
        match pattern {
            ast::Pattern::Variable(Text::Borrowed("_"), mutable, span) => {
                Ok(ir::Pattern::WildCard(span))
            }
            ast::Pattern::Variable(name, mutable, span) => {
                Ok(ir::Pattern::Variable(name, mutable, span))
            }
            ast::Pattern::Tuple(patterns, span) => {
                let patterns = patterns.into_iter()
                    .map(|pat| self.convert_pattern(pat))
                    .collect::<Result<Vec<_>, ()>>()?;
                Ok(ir::Pattern::Tuple(patterns, span))
            }
            ast::Pattern::Constant(constant) => {
                Ok(ir::Pattern::Constant(self.convert_constant(constant)?))
            }
            ast::Pattern::WildCard(span) => {
                Ok(ir::Pattern::WildCard(span))
            }
        }
    }

    fn convert_constant(&mut self, constant: ast::Constant<'convert>) -> Result<ir::Constant<'convert>, ()> {
        match constant {
            ast::Constant::Integer(value, ty, span) => {
                Ok(ir::Constant::Integer(value, ty.expect("TODO: handle missing type"), span))
            }
            ast::Constant::Float(value, ty, span) => {
                Ok(ir::Constant::Float(value, ty.expect("TODO: handle missing type"), span))
            }
            ast::Constant::String(value, span) => {
                Ok(ir::Constant::String(value, span))
            }
            ast::Constant::Character(value, span) => {
                Ok(ir::Constant::Character(value, span))
            }
            ast::Constant::Bool(value, span) => {
                Ok(ir::Constant::Bool(value, span))
            }
        }
    }

    fn convert_statement(&mut self, stmt: ast::Statement<'convert>) -> Result<ir::Statement<'convert>, ()> {
        match stmt {
            ast::Statement::Expression(expr, span) => {
                Ok(ir::Statement::Expression(self.convert_expression(expr)?, span))
            }
            ast::Statement::Let {
                bindings,
                ty,
                value,
                span,
            } => {
                let bindings = self.convert_pattern(bindings)?;
                let value = self.convert_expression(value)?;
                Ok(ir::Statement::Let {
                    bindings,
                    ty,
                    value,
                    span,
                })
            }
            ast::Statement::Const {
                bindings,
                ty,
                value,
                span,
            } => {
                let bindings = self.convert_pattern(bindings)?;
                let value = self.convert_expression(value)?;
                Ok(ir::Statement::Const {
                    bindings,
                    ty,
                    value,
                    span,
                })
            }
            ast::Statement::Assignment {
                target,
                value,
                span
            } => {
                let target = self.convert_expression(target)?;
                let value = self.convert_expression(value)?;

                Ok(ir::Statement::Assignment {
                    target,
                    value,
                    span,
                })
            }
            ast::Statement::While {
                label,
                test,
                body,
                span
            } => {
                let test = self.convert_expression(test)?;
                let body = body.into_iter()
                    .map(|stmt| self.convert_statement(stmt))
                    .collect::<Result<Vec<_>, ()>>()?;

                Ok(ir::Statement::While {
                    label,
                    test,
                    body,
                    span
                })
            }
            stmt => todo!("IR conversion statement: {:?}", stmt),
        }
    }

    fn convert_expression(&mut self, expr: ast::Expression<'convert>) -> Result<ir::Expression<'convert>, ()> {
        match expr {
            ast::Expression::Variable(name, ty, span) => {
                Ok(ir::Expression::Variable(name.clone(),
                ty.expect(&format!("TODO: handle missing type of variable: {}", name)), span))
            }
            ast::Expression::Literal(literal) => {
                Ok(ir::Expression::Literal(self.convert_literal(literal)?))
            }
            ast::Expression::This(span) => {
                Ok(ir::Expression::This(span))
            }
            ast::Expression::Call {
                name,
                type_args,
                args,
                span,
                annotation
            } => {
                let name = Box::new(self.convert_expression(*name)?);
                let args = args.into_iter()
                    .map(|arg| self.convert_expression(arg))
                    .collect::<Result<Vec<_>, ()>>()?;

                Ok(ir::Expression::Call {
                    name,
                    type_args,
                    args,
                    span,
                    annotation: annotation.expect("TODO: handle missing type"),
                })
            }
            ast::Expression::StaticCall {
                name,
                type_args,
                args,
                span,
                annotation
            } => {
                let args = args.into_iter()
                    .map(|arg| self.convert_expression(arg))
                    .collect::<Result<Vec<_>, ()>>()?;

                Ok(ir::Expression::StaticCall {
                    name,
                    type_args,
                    args,
                    span,
                    annotation: annotation.expect("TODO: handle missing type"),
                })
            }
            ast::Expression::MemberAccess {
                object,
                field,
                span,
                annotation
            } => {
                let object = Box::new(self.convert_expression(*object)?);

                Ok(ir::Expression::MemberAccess {
                    object,
                    field,
                    span,
                    annotation: annotation.expect("TODO: handle missing type"),
                })
            }
            ast::Expression::ClassAccess {
                class_name,
                span,
            } => {
                Ok(ir::Expression::ClassAccess {
                    class_name,
                    span,
                })
            }
            ast::Expression::Closure {
                params,
                return_type,
                body,
                span
            } => {
                let params = params.into_iter()
                    .map(|param| self.convert_closure_param(param))
                    .collect::<Result<Vec<_>, ()>>()?;
                let return_type = return_type.expect("TODO: handle missing type");
                let body = body.into_iter()
                    .map(|stmt| self.convert_statement(stmt))
                    .collect::<Result<Vec<_>, ()>>()?;

                Ok(ir::Expression::new_closure(params, return_type, body, span))
            }
            ast::Expression::Parenthesized(expr, span) => {
                let expr = Box::new(self.convert_expression(*expr)?);

                Ok(ir::Expression::Parenthesized(expr, span))
            }
            ast::Expression::IfExpression(if_expr, span) => {
                Ok(ir::Expression::IfExpression(self.convert_if_expression(if_expr)?, span))
            }
            ast::Expression::UnaryOperation {
                operator,
                operand,
                span,
            } => {
                let operand = Box::new(self.convert_expression(*operand)?);

                Ok(ir::Expression::UnaryOperation {
                    operator,
                    operand,
                    span,
                })
            }
            ast::Expression::BinaryOperation {
                operator,
                left,
                right,
                span,
            } => {
                let left = Box::new(self.convert_expression(*left)?);
                let right = Box::new(self.convert_expression(*right)?);

                Ok(ir::Expression::BinaryOperation {
                    operator,
                    left,
                    right,
                    span,
                })
            }
            ast::Expression::Return(expr, span) => {
                let expr = if let Some(expr) = expr {
                    Some(Box::new(self.convert_expression(*expr)?))
                } else {
                    None
                };

                Ok(ir::Expression::Return(expr, span))
            }
            ast::Expression::New(ty, array_size, span) => {
                let array_size = if let Some(array_size) = array_size {
                    Some(Box::new(self.convert_expression(*array_size)?))
                } else {
                    None
                };
                Ok(ir::Expression::New(ty, array_size, span))
            }
            x => todo!("IR conversion expression: {:?}", x),
        }
    }

    fn convert_literal(&mut self, literal: ast::Literal<'convert>) -> Result<ir::Literal<'convert>, ()> {
        match literal {
            ast::Literal::Constant(constant) => {
                Ok(ir::Literal::Constant(self.convert_constant(constant)?))
            }
            ast::Literal::Void(span) => {
                Ok(ir::Literal::Void(span))
            }
            ast::Literal::Array(body, annotation, span) => {
                let body = body.into_iter()
                    .map(|expr| self.convert_expression(expr))
                    .collect::<Result<Vec<_>, ()>>()?;

                Ok(ir::Literal::Array(body, annotation.expect("TODO: handle missing type"), span))
            }
            _ => todo!("IR conversion literal tuple")
        }
    }

    fn convert_closure_param(&mut self, param: ast::ClosureParameter<'convert>) -> Result<ir::ClosureParameter<'convert>, ()> {
        match param {
            ast::ClosureParameter::Typed(param) => {
                Ok(ir::ClosureParameter {
                    parameter: self.convert_parameter(param)?,
                })
            }
            _ => todo!("report missing closure parameter type")
        }
    }

    fn convert_if_expression(&mut self, if_expr: ast::IfExpression<'convert>) -> Result<ir::IfExpression<'convert>, ()> {
        let ast::IfExpression {
            condition,
            then_branch,
            else_branch,
            start,
            end
        } = if_expr;
        let condition = Box::new(self.convert_expression(*condition)?);
        let then_branch = then_branch.into_iter()
            .map(|stmt| self.convert_statement(stmt))
            .collect::<Result<Vec<_>, ()>>()?;

        let else_branch = match else_branch {
            Some(Either::Left(elif_branch)) => {
                let elif_branch = Box::new(self.convert_if_expression(*elif_branch)?);
                Some(Either::Left(elif_branch))
            }
            Some(Either::Right(else_branch)) => {
                let else_branch = else_branch.into_iter()
                    .map(|stmt| self.convert_statement(stmt))
                    .collect::<Result<Vec<_>, ()>>()?;

                Some(Either::Right(else_branch))
            }
            None => None
        };

        Ok(ir::IfExpression {
            condition,
            then_branch,
            else_branch,
            start,
            end,
        })
    }
}