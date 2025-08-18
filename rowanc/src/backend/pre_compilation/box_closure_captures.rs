use std::collections::{HashMap, HashSet};
use either::Either;
use crate::trees::ir::{Class, ClosureParameter, Expression, File, IfExpression, Method, Parameter, Pattern, Statement, TopLevelStatement};
use crate::trees::{PathName, Span, Text, Type};

pub struct BoxClosureCapture<> {}

impl<'boxing> BoxClosureCapture<> {

    pub fn new() -> Self {
        Self {}
    }

    pub fn box_closures(&mut self, file: File<'boxing>) -> File<'boxing> {
        let File {
            path,
            content,
        } = file;

        let mut new_content = Vec::new();

        for top_level_statement in content.into_iter() {
            match top_level_statement {
                TopLevelStatement::Import(import) => {
                    new_content.push(TopLevelStatement::Import(import));
                }
                TopLevelStatement::Class(class) => {
                    new_content.push(TopLevelStatement::Class(self.box_class(class)));
                }
            }
        }

        File { path, content: new_content }
    }

    fn box_class(&mut self, class: Class<'boxing>) -> Class<'boxing> {
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

        for method in methods.into_iter() {
            new_methods.push(self.box_method(method));
        }

        Class { name, parent, members, methods: new_methods, static_members, type_params, span }
    }

    fn box_method(&mut self, method: Method<'boxing>) -> Method<'boxing> {
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

        let new_body = self.box_body(body);

        Method {
            name,
            is_native,
            annotations,
            visibility,
            type_params,
            parameters,
            return_type,
            body: new_body,
            span,
        }
    }

    fn box_body(&mut self, mut body: Vec<Statement<'boxing>>) -> Vec<Statement<'boxing>> {
        let mut index = 0;
        while index < body.len() {
            let found_closure = match &mut body[index] {
                Statement::Let { value, .. } => {
                    self.find_closure(value)
                }
                Statement::Expression(expr, ..) => {
                    self.find_closure(expr)
                }
                Statement::While { test, body, .. } => {
                    self.find_closure(test) || self.find_closure_body(body)
                }
                Statement::Const { value, .. } => {
                    self.find_closure(value)
                }
                Statement::Assignment { target, value, .. } => {
                    self.find_closure(target) || self.find_closure(value)
                }
                _ => todo!("finding closure for remaining statements")
            };
            if found_closure {
                println!("found closure");
                loop {
                    let mut stmts: Vec<Statement> = self.box_primitives(index, &mut body);
                    let is_stmts_empty = stmts.is_empty();
                    index += stmts.len();
                    println!();
                    for stmt in &stmts {
                        println!("{:?}", stmt);
                    }
                    stmts.append(&mut body);
                    body = stmts;
                    if is_stmts_empty {
                        break;
                    }
                }
            }
            index += 1;
        }

        body
    }

    fn find_closure(&mut self, expr: &mut Expression<'boxing>) -> bool {
        match expr {
            Expression::Variable(..) => false,
            Expression::Literal(..) => false,
            Expression::This(..) => false,
            Expression::New(..) => false,
            Expression::MemberAccess { .. } => false,
            Expression::As { .. } => false,
            Expression::Into { .. } => false,
            Expression::ClassAccess { .. } => false,
            Expression::UnaryOperation { operand, .. } => {
                self.find_closure(operand.as_mut())
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.find_closure(left.as_mut()) || self.find_closure(right.as_mut())
            }
            Expression::Parenthesized(expr, _) => {
                self.find_closure(expr.as_mut())
            }
            Expression::Return(expr, _) => {
                if let Some(expr) = expr {
                    self.find_closure(expr.as_mut())
                } else {
                    false
                }
            }
            Expression::Call{ name, args, .. } => {
                self.find_closure(name.as_mut()) || args.iter_mut()
                    .any(|arg| self.find_closure(arg))
            }
            Expression::StaticCall { args, .. } => {
                args.iter_mut()
                    .any(|arg| self.find_closure(arg))
            }
            Expression::IfExpression(if_expr, ..) => {
                self.find_closure_if(if_expr)
            }
            Expression::Closure { body, .. } => {
                true || self.find_closure_body(body)
            }
            _ => todo!("finding closure for remaining expressions"),
        }
    }

    fn find_closure_if(&mut self, if_expr: &mut IfExpression<'boxing>) -> bool {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
            ..
        } = if_expr;
        let test_result = self.find_closure(condition.as_mut());
        let then_result = self.find_closure_body(then_branch);
        let else_result = match else_branch {
            Some(Either::Left(if_expr)) => self.find_closure_if(if_expr),
            Some(Either::Right(else_branch)) => self.find_closure_body(else_branch),
            None => false,
        };

        test_result || then_result || else_result || test_result
    }

    fn find_closure_body(&mut self, body: &mut Vec<Statement<'boxing>>) -> bool {
        for statement in body.iter_mut() {
            match statement {
                Statement::Let { value, .. } => {
                    let result = self.find_closure(value);
                    if result {
                        return true;
                    }
                }
                Statement::Const { value, .. } => {
                    let result = self.find_closure(value);
                    if result {
                        return true;
                    }
                }
                Statement::Assignment { target, value, .. } => {
                    let result = self.find_closure(target) || self.find_closure(value);
                    if result {
                        return true;
                    }
                }
                Statement::While { test, body, .. } => {
                    let result = self.find_closure(test) || self.find_closure_body(body);
                    if result {
                        return true;
                    }
                }
                Statement::Expression(expr, ..) => {
                    let result = self.find_closure(expr);
                    if result {
                        return true;
                    }
                }
                _ => todo!("finding closure for remaining statements"),
            }
        }
        false
    }

    /// TODO: fix this to handle nested closures
    fn box_primitives<'input>(&mut self, index: usize, stmts: &mut Vec<Statement<'boxing>>) -> Vec<Statement<'boxing>> {
        println!("boxing primitives");
        let mut prepend_statements = Vec::new();
        let mut indices = Vec::new();

        let mut found_captures = {
            let the_closure = self.get_closure(&mut stmts[index]);
            let Some(Expression::Closure {
                         params,
                         captures,
                         processed_captures,
                         body,
                         ..
                     }) = the_closure else {
                return Vec::new();
            };
            *processed_captures = true;
            let mut bound_vars = self.get_param_set(params);
            let mut found_captures = HashMap::new();

            for stmt in body.iter() {
                self.get_capture(stmt, &mut bound_vars, &mut found_captures);
            }
            println!("found captures: {:?}", found_captures);

            for (key, (_, ty) ) in &found_captures {
                captures.push((Text::Owned(key.clone()), ty.clone()));
            }


            for i in (0..index).rev() {
                match &stmts[i] {
                    Statement::Let { bindings, .. } => {
                        if self.is_bound(bindings, &found_captures) {
                            indices.push(i);
                        }
                    }
                    Statement::Const { bindings, .. } => {
                        if self.is_bound(bindings, &found_captures) {
                            indices.push(i);
                        }
                    }
                    _ => {}
                }
            }
            found_captures
        };

        println!("indices: {:?}", indices);
        for index in indices.into_iter().rev() {
            match &mut stmts[index] {
                Statement::Let { bindings, ty, value, .. } => {
                    self.modify_binding(bindings, ty, value, &mut found_captures);
                }
                Statement::Const { bindings, ty, value, .. } => {
                    self.modify_binding(bindings, ty, value, &mut found_captures);
                }
                _ => unreachable!("only be let or const"),
            }
        }

        for (var, (mutated, ty)) in found_captures {
            if !mutated {
                continue;
            }
            let string = match &ty {
                Type::U8 => Text::Borrowed("U8"),
                Type::U16 => Text::Borrowed("U16"),
                Type::U32 => Text::Borrowed("U32"),
                Type::U64 => Text::Borrowed("U64"),
                Type::I8 => Text::Borrowed("I8"),
                Type::I16 => Text::Borrowed("I16"),
                Type::I32 => Text::Borrowed("I32"),
                Type::I64 => Text::Borrowed("I64"),
                Type::F32 => Text::Borrowed("F32"),
                Type::F64 => Text::Borrowed("F64"),
                _ => unreachable!("unsupported type, can only be primitive"),
            };
            let bindings = Pattern::Variable(Text::Owned(var.clone()), false, Span::new(0, 0));
            let let_type = Type::Object(string.clone(), Span::new(0, 0));
            let value = Expression::StaticCall {
                name: PathName::new(vec![string.clone(), Text::Borrowed("create")], Span::new(0, 0)),
                type_args: vec![],
                args: vec![Expression::Variable(Text::Owned(var), ty, Span::new(0, 0))],
                annotation: Type::Object(string, Span::new(0,0)),
                span: Span::new(0, 0),
            };
            let statement = Statement::Let {
                bindings,
                ty: let_type,
                value,
                span: Span::new(0, 0),
            };
            prepend_statements.push(statement);
        }

        prepend_statements
    }

    fn get_capture(
        &self,
        stmt: &Statement<'boxing>,
        bound_vars: &mut HashSet<String>,
        captures: &mut HashMap<String, (bool, Type<'boxing>)>
    ) {
        match stmt {
            Statement::Let { bindings, value, .. } => {
                let Pattern::Variable(var, _, _) = bindings else {
                    todo!("support additional patterns")
                };
                bound_vars.insert(var.to_string());
                self.get_capture_expression(value, bound_vars, captures, false);
            }
            Statement::Const { bindings, value, .. } => {
                let Pattern::Variable(var, _, _) = bindings else {
                    todo!("support additional patterns")
                };
                bound_vars.insert(var.to_string());
                self.get_capture_expression(value, bound_vars, captures, false);
            }
            Statement::Assignment { target, value, .. } => {
                self.get_capture_expression(target, bound_vars, captures, true);
                self.get_capture_expression(value, bound_vars, captures, false);
            }
            Statement::While { test, body, .. } => {
                self.get_capture_expression(test, bound_vars, captures, false);
                for stmt in body {
                    self.get_capture(stmt, bound_vars, captures);
                }
            }
            Statement::Expression(expr, ..) => {
                self.get_capture_expression(expr, bound_vars, captures, false);
            }
            _ => todo!("remaining statements of get_capture"),
        }
    }

    fn get_capture_expression(
        &self,
        expr: &Expression<'boxing>,
        bound_vars: &mut HashSet<String>,
        captures: &mut HashMap<String, (bool, Type<'boxing>)>,
        mutating_context: bool,
    ) {
        match expr {
            Expression::Variable(name, ty, ..) => {
                if !bound_vars.contains(name.as_str()) {
                    captures.entry(name.to_string())
                        .or_insert((mutating_context, ty.clone()));
                }
            },
            Expression::This(..) => {
                captures.insert(String::from("self"), (mutating_context, Type::Object(Text::Borrowed(""), Span::new(0, 0))));
            }
            Expression::Call { name, args, .. } => {
                self.get_capture_expression(name.as_ref(), bound_vars, captures, true);
                for arg in args {
                    self.get_capture_expression(arg, bound_vars, captures, false);
                }
            }
            Expression::StaticCall { args, .. } => {
                for arg in args {
                    self.get_capture_expression(arg, bound_vars, captures, false);
                }
            }
            Expression::Parenthesized(expr, _) => {
                self.get_capture_expression(expr, bound_vars, captures, mutating_context);
            }
            Expression::UnaryOperation { operand, .. } => {
                self.get_capture_expression(operand.as_ref(), bound_vars, captures, false);
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.get_capture_expression(left.as_ref(), bound_vars, captures, false);
                self.get_capture_expression(right.as_ref(), bound_vars, captures, false);
            }
            Expression::Return(expr, _) => {
                if let Some(expr) = expr {
                    self.get_capture_expression(expr, bound_vars, captures, false);
                }
            }
            Expression::New(_, expr, _) => {
                if let Some(expr) = expr {
                    self.get_capture_expression(expr, bound_vars, captures, false);
                }
            }
            Expression::Closure {
                params,
                body,
                ..
            } => {
                let mut new_bindings = self.get_param_set(params);
                new_bindings.extend(bound_vars.iter().cloned());
                for stmt in body {
                    self.get_capture(stmt, &mut new_bindings, captures);
                }
            }
            Expression::IfExpression(if_expr,..) => {
                self.get_capture_expression_if(if_expr, bound_vars, captures);
            }
            Expression::Literal(..) => {}
            Expression::MemberAccess { object, .. } => {
                self.get_capture_expression(object.as_ref(), bound_vars, captures, false);
            }
            Expression::ClassAccess { .. } => {}
            _ => todo!("handle remaining expressions in get_capture_expression"),
        }
    }

    fn get_capture_expression_if(
        &self,
        expr: &IfExpression<'boxing>,
        bound_vars: &mut HashSet<String>,
        captures: &mut HashMap<String, (bool, Type<'boxing>)>
    ) {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
            ..
        } = expr;
        self.get_capture_expression(condition, bound_vars, captures, false);
        for stmt in then_branch {
            self.get_capture(stmt, bound_vars, captures);
        }

        match else_branch {
            Some(Either::Left(elif_branch)) => {
                self.get_capture_expression_if(elif_branch.as_ref(), bound_vars, captures)
            }
            Some(Either::Right(else_branch)) => {
                for stmt in else_branch {
                    self.get_capture(stmt, bound_vars, captures);
                }
            }
            None => {}
        }
    }

    fn modify_binding(
        &mut self,
        pattern: & Pattern<'boxing>,
        ty: &mut Type,
        value: &mut Expression<'boxing>,
        bound_vars: &mut HashMap<String, (bool, Type<'boxing>)>,
    ) {
        match (pattern, ty, value) {
            (Pattern::Variable(var, ..), ty, value) => {
                match ty {
                    Type::U8 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("U8"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("U8"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("U8"), Span::new(0, 0));
                    }
                    Type::U16 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("U16"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("U16"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("U16"), Span::new(0, 0));
                    }
                    Type::U32 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("U32"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("U32"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("U32"), Span::new(0, 0));
                    }
                    Type::U64 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("U64"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("U64"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("U64"), Span::new(0, 0));
                    }
                    Type::I8 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("I8"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("I8"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("I8"), Span::new(0, 0));
                    }
                    Type::I16 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("I16"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("I16"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("I16"), Span::new(0, 0));
                    }
                    Type::I32 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("I32"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("I32"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("I32"), Span::new(0, 0));
                    }
                    Type::I64 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("I64"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("I64"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("I64"), Span::new(0, 0));
                    }
                    Type::F32 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("F32"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("F32"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("F32"), Span::new(0, 0));
                    }
                    Type::F64 => {
                        let Some((mutated, _)) = bound_vars.remove(var.as_str()) else {
                            unreachable!("cannot find bound var")
                        };
                        if !mutated {
                            // We only need to box primitives if they are captured and mutated
                            return;
                        }
                        *value = Expression::StaticCall {
                            name: PathName::new(vec![
                                Text::Borrowed("F64"),
                                Text::Borrowed("create"),
                            ], Span::new(0, 0)),
                            type_args: vec![],
                            args: vec![value.clone()],
                            span: Span::new(0, 0),
                            annotation: Type::Object(Text::Borrowed("F64"), Span::new(0, 0)),
                        };
                        *ty = Type::Object(Text::Borrowed("F64"), Span::new(0, 0));
                    }
                    _ => {}
                }
            }
            (Pattern::Tuple(..), _, _) => todo!("handle tuple pattern, this might need adding a new binding after this one"),
            _ => {}
        }
    }

    fn get_param_set(&self, params: &[ClosureParameter<'boxing>]) -> HashSet<String> {
        let mut set = HashSet::new();
        fn bind_pattern<'boxing>(pattern: &Pattern<'boxing>, set: &mut HashSet<String>) {
            match pattern {
                Pattern::Variable(var, ..) => {
                    set.insert(var.to_string());
                }
                Pattern::Tuple(patterns, ..) => {
                    for pattern in patterns {
                        bind_pattern(pattern, set);
                    }
                }
                _ => {}
            }
        }
        for param in params {
            match &param.parameter {
                Parameter::Pattern { name, .. } => {
                    bind_pattern(name, &mut set);
                }
                Parameter::This(..) => {
                    unreachable!("invalid closure parameter")
                }
            }
        }
        set
    }


    fn get_closure<'input>(&mut self, stmt: &'input mut Statement<'boxing>) -> Option<&'input mut Expression<'boxing>> {
        match stmt {
            Statement::Let { value, .. } => {
                self.get_closure_expression(value)
            }
            Statement::Const { value, .. } => {
                self.get_closure_expression(value)
            }
            Statement::Assignment { target, value, .. } => {
                if let Some(target) = self.get_closure_expression(target) {
                    Some(target)
                } else if let Some(value) = self.get_closure_expression(value) {
                    Some(value)
                } else {
                    None
                }
            }
            Statement::While { test, body, .. } => {
                {
                    let result = self.get_closure_expression(test);
                    if result.is_some() {
                        return result;
                    }
                }
                for stmt in body.iter_mut() {
                    if let Some(expr) = self.get_closure(stmt) {
                        return Some(expr);
                    }
                }
                None
            }
            Statement::Expression(expr, ..) => {
                self.get_closure_expression(expr)
            }
            _ => todo!("getting closure for remaining statements"),
        }
    }

    fn get_closure_expression<'input>(&mut self, mut expr: &'input mut Expression<'boxing>) -> Option<&'input mut Expression<'boxing>> {
        if expr.is_closure() && !expr.processed_captures() {
            return Some(expr);
        }

        match expr {
            Expression::Closure { body, captures, .. } => {
                for stmt in body.iter_mut() {
                    if let Some(expr) = self.get_closure(stmt) {
                        return Some(expr);
                    }
                }
                None

            }
            Expression::Call { name, args, .. } => {
                let name_result = self.get_closure_expression(name.as_mut());
                if name_result.is_some() {
                    return name_result;
                }
                for arg in args {
                    let arg_result = self.get_closure_expression(arg);
                    if arg_result.is_some() {
                        return arg_result;
                    }
                }
                None
            }
            Expression::StaticCall { args, .. } => {
                for arg in args {
                    let arg_result = self.get_closure_expression(arg);
                    if arg_result.is_some() {
                        return arg_result;
                    }
                }
                None
            }
            Expression::Parenthesized(expr, ..) => {
                self.get_closure_expression(expr.as_mut())
            }
            Expression::IfExpression(if_expr, ..) => {
                self.get_closure_expression_if(if_expr)
            }
            Expression::UnaryOperation { operand, ..} => {
                self.get_closure_expression(operand.as_mut())
            }
            Expression::BinaryOperation { left, right, .. } => {
                let left_result = self.get_closure_expression(left.as_mut());
                if left_result.is_some() {
                    return left_result;
                }
                let right_result = self.get_closure_expression(right.as_mut());
                if right_result.is_some() {
                    return right_result;
                }
                None
            }
            Expression::Return(Some(expr), ..) => {
                self.get_closure_expression(expr.as_mut())
            }
            Expression::Return(None, ..) => {
                None
            }
            Expression::Variable(..) => None,
            Expression::New(..) => None,
            Expression::MemberAccess { .. } => None,
            Expression::Literal(..) => None,
            x => todo!("getting closure for remaining expression: {x:?}"),
        }
    }

    fn get_closure_expression_if<'input>(&mut self, expr: &'input mut IfExpression<'boxing>) -> Option<&'input mut Expression<'boxing>> {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
            ..
        } = expr;

        let condition_result = self.get_closure_expression(condition.as_mut());
        if condition_result.is_some() {
            return condition_result;
        }
        for stmt in then_branch.iter_mut() {
            if let Some(expr) = self.get_closure(stmt) {
                return Some(expr);
            }
        }
        match else_branch {
            Some(Either::Left(elif_branch)) => {
                self.get_closure_expression_if(elif_branch.as_mut())
            }
            Some(Either::Right(else_branch)) => {
                for stmt in else_branch.iter_mut() {
                    if let Some(expr) = self.get_closure(stmt) {
                        return Some(expr);
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn is_bound(&self, pattern: &Pattern<'boxing>, captures: &HashMap<String, (bool, Type)>) -> bool {
        match pattern {
            Pattern::Variable(name, ..) => {
                captures.contains_key(name.as_str())
            }
            Pattern::Tuple(pattern, ..) => {
                for pattern in pattern.iter() {
                    if self.is_bound(pattern, captures) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}