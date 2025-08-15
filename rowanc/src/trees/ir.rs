use either::Either;
use crate::trees::{BinaryOperator, PathName, Text, Type, UnaryOperator};
use crate::trees::{Span, Visibility};



#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct File<'a> {
    pub path: PathName<'a>,
    pub content: Vec<TopLevelStatement<'a>>,
}

impl File<'_> {
    pub fn new<'a>(path: PathName<'a>, content: Vec<TopLevelStatement<'a>>) -> File<'a> {
        File { path, content }
    }

    pub fn get_imports(&self) -> Vec<&PathName> {
        self.content.iter().filter_map(|stmt| {
            if let TopLevelStatement::Import(import) = stmt {
                Some(&import.path)
            } else {
                None
            }
        }).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum TopLevelStatement<'a> {
    Import(Import<'a>),
    Class(Class<'a>),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Import<'a> {
    pub path: PathName<'a>,
    pub span: Span,
}

impl Import<'_> {
    pub fn new<'a>(path: PathName<'a>, span: Span) -> Import<'a> {
        Import { path, span }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Class<'a> {
    pub name: Text<'a>,
    pub parent: Option<ParentDec<'a>>,
    pub members: Vec<Member<'a>>,
    pub methods: Vec<Method<'a>>,
    pub static_members: Vec<StaticMember<'a>>,
    pub type_params: Vec<TypeParameter<'a>>,
    pub span: Span,
}

impl Class<'_> {
    pub fn new<'a>(
        name: Text<'a>,
        parent: Option<ParentDec<'a>>,
        members: Vec<Member<'a>>,
        methods: Vec<Method<'a>>,
        static_members: Vec<StaticMember<'a>>,
        type_params: Vec<TypeParameter<'a>>,
        span: Span
    ) -> Class<'a> {
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
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct ParentDec<'a> {
    pub name: Text<'a>,
    pub type_args: Vec<Type<'a>>,
    pub type_params: Vec<TypeParameter<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum ClassMember<'a> {
    Member(Member<'a>),
    Method(Method<'a>),
    StaticMember(StaticMember<'a>),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Member<'a> {
    pub visibility: Visibility,
    pub name: Text<'a>,
    pub ty: Type<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Method<'a> {
    pub name: Text<'a>,
    pub is_native: bool,
    pub annotations: Vec<Annotation<'a>>,
    pub visibility: Visibility,
    pub type_params: Vec<TypeParameter<'a>>,
    pub parameters: Vec<Parameter<'a>>,
    pub return_type: Type<'a>,
    pub body: Vec<Statement<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Annotation<'a> {
    pub name: Text<'a>,
    pub parameters: Vec<Text<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Parameter<'a> {
    This(bool, Span),
    Pattern {
        name: Pattern<'a>,
        ty: Type<'a>,
        span: Span,
    }
}

impl Parameter<'_> {
    pub fn new_this(mutable: bool, span: Span) -> Parameter<'static> {
        Parameter::This(mutable, span)
    }

    pub fn new_pattern<'a>(pattern: Pattern<'a>, ty: Type<'a>, span: Span) -> Parameter<'a> {
        Parameter::Pattern {
            name: pattern,
            ty,
            span,
        }

    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct StaticMember<'a> {
    pub visibility: Visibility,
    pub is_const: bool,
    pub name: Text<'a>,
    pub ty: Type<'a>,
    pub value: Option<Expression<'a>>,
    pub span: Span,
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct TypeParameter<'a> {
    pub name: Text<'a>,
    pub constraints: Vec<Constraint<'a>>,
    pub span: Span,
}

impl TypeParameter<'_> {
    pub fn new<'a>(name: Text<'a>, constraints: Vec<Constraint<'a>>, span: Span) -> TypeParameter<'a> {
        TypeParameter {
            name,
            constraints,
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Constraint<'a> {
    Extends(Vec<Type<'a>>, Span),
    //TODO: add trait bounds
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Statement<'a> {
    Expression(Expression<'a>, Span),
    Let {
        bindings: Pattern<'a>,
        ty: Type<'a>,
        value: Expression<'a>,
        span: Span,
    },
    Const {
        bindings: Pattern<'a>,
        ty: Type<'a>,
        value: Expression<'a>,
        span: Span,
    },
    Assignment {
        target: Expression<'a>,
        value: Expression<'a>,
        span: Span,
    },
    While {
        label: Option<Text<'a>>,
        test: Expression<'a>,
        body: Vec<Statement<'a>>,
        span: Span,
    },
    For {
        label: Option<Text<'a>>,
        bindings: Pattern<'a>,
        bindings_type: Type<'a>,
        iterable: Expression<'a>,
        span: Span,
    },
    With {
        expr: Expression<'a>,
        bindings: Pattern<'a>,
        bindings_type: Type<'a>,
        block: Vec<Statement<'a>>,
        span: Span,
    }
}

impl Statement<'_> {
    pub fn new_let<'a>(bindings: Pattern<'a>, ty: Type<'a>, value: Expression<'a>, span: Span) -> Statement<'a> {
        Statement::Let {
            bindings,
            ty,
            value,
            span,
        }
    }

    pub fn new_const<'a>(bindings: Pattern<'a>, ty: Type<'a>, value: Expression<'a>, span: Span) -> Statement<'a> {
        Statement::Const {
            bindings,
            ty,
            value,
            span,
        }
    }

    pub fn new_assignment<'a>(target: Expression<'a>, value: Expression<'a>, span: Span) -> Statement<'a> {
        Statement::Assignment {
            target,
            value,
            span
        }
    }

    pub fn new_while<'a>(
        label: Option<Text<'a>>,
        test: Expression<'a>,
        body: Vec<Statement<'a>>,
        span: Span,
    ) -> Statement<'a> {
        Statement::While {
            label,
            test,
            body,
            span
        }
    }

    pub fn new_for<'a>(
        label: Option<Text<'a>>,
        bindings: Pattern<'a>,
        bindings_type: Type<'a>,
        iterable: Expression<'a>,
        span: Span,
    ) -> Statement<'a> {
        Statement::For {
            label,
            bindings,
            bindings_type,
            iterable,
            span
        }
    }

    pub fn new_with<'a>(
        expr: Expression<'a>,
        bindings: Pattern<'a>,
        bindings_type: Type<'a>,
        block: Vec<Statement<'a>>,
        span: Span,
    ) -> Statement<'a> {
        Statement::With {
            expr,
            bindings,
            bindings_type,
            block,
            span
        }
    }

}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Pattern<'a> {
    Variable(Text<'a>, bool, Span),
    Tuple(Vec<Pattern<'a>>, Span),
    Constant(Constant<'a>),
    WildCard(Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Constant<'a> {
    Integer(Text<'a>, Type<'a>, Span),
    Float(Text<'a>, Type<'a>, Span),
    String(Text<'a>, Span),
    Character(Text<'a>, Span),
    Bool(bool, Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Expression<'a> {
    Variable(Text<'a>, Type<'a>, Span),
    Literal(Literal<'a>),
    This(Span),
    Call {
        name: Box<Expression<'a>>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span,
        annotation: Type<'a>,
    },
    StaticCall {
        name: PathName<'a>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span,
        annotation: Type<'a>,
    },
    MemberAccess {
        object: Box<Expression<'a>>,
        field: PathName<'a>,
        span: Span,
        annotation: Type<'a>,
    },
    ClassAccess {
        class_name: PathName<'a>,
        span: Span,
    },
    Closure {
        params: Vec<ClosureParameter<'a>>,
        return_type: Option<Type<'a>>,
        body: Vec<Statement<'a>>,
        span: Span,
    },
    Parenthesized(Box<Expression<'a>>, Span),
    IfExpression(IfExpression<'a>, Span),
    MatchExpression(MatchExpression<'a>, Span),
    UnaryOperation {
        operator: UnaryOperator,
        operand: Box<Expression<'a>>,
        span: Span,
    },
    BinaryOperation {
        operator: BinaryOperator,
        left: Box<Expression<'a>>,
        right: Box<Expression<'a>>,
        span: Span,
    },
    Return(Option<Box<Expression<'a>>>, Span),
    New(Type<'a>, Option<Box<Expression<'a>>>, Span),
    Loop {
        label: Option<Text<'a>>,
        body: Vec<Statement<'a>>,
        span: Span,
    },
    Continue(Option<Text<'a>>, Span),
    Break(Option<Text<'a>>, Option<Box<Expression<'a>>>, Span),
    As {
        source: Box<Expression<'a>>,
        typ: Type<'a>,
        span: Span,
    },
    Into {
        source: Box<Expression<'a>>,
        typ: Type<'a>,
        span: Span,
    },
}

impl Expression<'_> {
    pub fn new_call<'a>(
        name: Box<Expression<'a>>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        annotation: Type<'a>,
        span: Span
    ) -> Expression<'a> {
        Expression::Call {
            name,
            type_args,
            args,
            span,
            annotation,
        }
    }

    pub fn new_static_call<'a>(
        name: PathName<'a>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        annotation: Type<'a>,
        span: Span
    ) -> Expression<'a> {
        Expression::StaticCall {
            name,
            type_args,
            args,
            span,
            annotation,
        }
    }

    pub fn new_member_access<'a>(
        object: Box<Expression<'a>>, 
        field: PathName<'a>,
        annotation: Type<'a>,
        span: Span
    ) -> Expression<'a> {
        Expression::MemberAccess {
            object,
            field,
            span,
            annotation,
        }
    }

    pub fn new_closure<'a>(
        params: Vec<ClosureParameter<'a>>,
        return_type: Option<Type<'a>>,
        body: Vec<Statement<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::Closure {
            params,
            return_type,
            body,
            span
        }
    }

    pub fn new_unary_operation<'a>(
        operator: UnaryOperator,
        operand: Box<Expression<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::UnaryOperation {
            operator,
            operand,
            span
        }
    }

    pub fn new_binary_operation<'a>(
        operator: BinaryOperator,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::BinaryOperation {
            operator,
            left: lhs,
            right: rhs,
            span
        }
    }

    pub fn new_loop<'a>(
        label: Option<Text<'a>>,
        body: Vec<Statement<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::Loop {
            label,
            body,
            span,
        }
    }

    pub fn new_as_expression<'a>(
        source: Box<Expression<'a>>,
        typ: Type<'a>,
        span: Span,
    ) -> Expression<'a> {
        Expression::As {
            source,
            typ,
            span
        }
    }

    pub fn new_into_expression<'a>(
        source: Box<Expression<'a>>,
        typ: Type<'a>,
        span: Span,
    ) -> Expression<'a> {
        Expression::Into {
            source,
            typ,
            span
        }
    }

    pub fn get_type(&self) -> Either<Type, ()> {
        match self {
            Expression::As {typ, ..} => Either::Left(typ.clone()),
            Expression::Into {typ, ..} => Either::Left(typ.clone()),
            Expression::Literal(Literal::Constant(Constant::Bool(_, _))) => {
                Either::Left(Type::U8)
            }
            Expression::Literal(Literal::Constant(Constant::Character(_, _))) => {
                Either::Left(Type::U32)
            }
            Expression::Literal(Literal::Constant(Constant::Integer(_, ty, _))) => {
                Either::Left(ty.clone())
            }
            Expression::Literal(Literal::Constant(Constant::Float(_, ty, _))) => {
                Either::Left(ty.clone())
            }
            Expression::Literal(Literal::Constant(Constant::String(_, _))) => {
                Either::Left(Type::Str)
            }
            Expression::Literal(Literal::Void(_)) => {
                Either::Left(Type::Void)
            }
            Expression::Literal(Literal::Tuple(_, ty, _)) => {
                Either::Left(ty.clone())
            }
            Expression::Literal(Literal::Array(_, ty, _)) => {
                Either::Left(ty.clone())
            }
            Expression::Variable(_, ty, _) => {
                Either::Left(ty.clone())
            }
            Expression::BinaryOperation { operator: BinaryOperator::Add, left,  .. } |
            Expression::BinaryOperation { operator: BinaryOperator::Sub, left,  .. } |
            Expression::BinaryOperation { operator: BinaryOperator::Mul, left,  .. } |
            Expression::BinaryOperation { operator: BinaryOperator::Div, left,  .. } |
            Expression::BinaryOperation { operator: BinaryOperator::Mod, left,  .. } => {
                left.get_type()
            }
            Expression::Call {annotation, ..} => {
                Either::Left(annotation.clone())
            }
            Expression::This(_) => Either::Right(()),
            Expression::MemberAccess {
                annotation, ..
            } => {
                Either::Left(annotation.clone())
            }
            Expression::Parenthesized(expr, _) => expr.get_type(),
            Expression::StaticCall { annotation, ..} => {
                Either::Left(annotation.clone())
            }
            x => todo!("Expression::get_type {:?}", x),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Literal<'a> {
    Constant(Constant<'a>),
    Void(Span),
    Tuple(Vec<Expression<'a>>, Type<'a>, Span),
    Array(Vec<Expression<'a>>, Type<'a>, Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct ClosureParameter<'a> {
    parameter: Parameter<'a>,
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct IfExpression<'a> {
    pub condition: Box<Expression<'a>>,
    pub then_branch: Vec<Statement<'a>>,
    pub else_branch: Option<Either<Box<IfExpression<'a>>, Vec<Statement<'a>>>>,
    pub start: usize,
    pub end: usize,
}

impl IfExpression<'_> {
    pub fn new<'a>(
        condition: Box<Expression<'a>>,
        then_branch: Vec<Statement<'a>>,
        else_branch: Option<Either<Box<IfExpression<'a>>, Vec<Statement<'a>>>>,
        start: usize,
        end: usize,
    ) -> IfExpression<'a> {
        IfExpression { condition, then_branch, else_branch, start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct MatchExpression<'a> {
    pub value: Box<Expression<'a>>,
    pub arms: Vec<MatchArm<'a>>,
    pub start: usize,
    pub end: usize,
}

impl MatchExpression<'_> {
    pub fn new<'a>(
        value: Box<Expression<'a>>,
        arms: Vec<MatchArm<'a>>,
        start: usize,
        end: usize,
    ) -> MatchExpression<'a> {
        MatchExpression { value, arms, start, end }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct MatchArm<'a> {
    pub pattern: Pattern<'a>,
    pub value: Either<Expression<'a>, Vec<Statement<'a>>>,
    pub start: usize,
    pub end: usize,
}

impl MatchArm<'_> {
    pub fn new<'a>(pattern: Pattern<'a>, value: Either<Expression<'a>, Vec<Statement<'a>>>, start: usize, end: usize) -> MatchArm<'a> {
        MatchArm { pattern, value, start, end }
    }
}
