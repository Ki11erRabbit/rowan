use either::Either;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span {
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Visiblity {
    Public,
    Protected,
    Private,
}


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
pub struct PathName<'a> {
    pub segments: Vec<&'a str>,
    pub span: Span,
}

impl PathName<'_> {
    pub fn new<'a>(segments: Vec<&'a str>, span: Span) -> PathName<'a> {
        PathName { segments, span }
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
    pub name: &'a str,
    pub parents: Vec<ParentDec<'a>>,
    pub members: Vec<Member<'a>>,
    pub methods: Vec<Method<'a>>,
    pub signals: Vec<Signal<'a>>,
    pub type_params: Vec<TypeParameter<'a>>,
    pub span: Span,
}

impl Class<'_> {
    pub fn new<'a>(
        name: &'a str,
        parents: Vec<ParentDec<'a>>,
        members: Vec<Member<'a>>,
        methods: Vec<Method<'a>>,
        signals: Vec<Signal<'a>>,
        type_params: Vec<TypeParameter<'a>>,
        span: Span
    ) -> Class<'a> {
        Class {
            name,
            parents,
            members,
            methods,
            signals,
            type_params,
            span
        }
    }
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct ParentDec<'a> {
    pub name: &'a str,
    pub type_params: Vec<Type<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum ClassMember<'a> {
    Member(Member<'a>),
    Method(Method<'a>),
    Signal(Signal<'a>),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Member<'a> {
    pub visiblity: Visiblity,
    pub name: &'a str,
    pub ty: Type<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Method<'a> {
    pub name: &'a str,
    pub annotations: Vec<Annotation<'a>>,
    pub visiblity: Visiblity,
    pub type_params: Vec<TypeParameter<'a>>,
    pub parameters: Vec<Parameter<'a>>,
    pub return_type: Type<'a>,
    pub body: Vec<Statement<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Annotation<'a> {
    pub name: &'a str,
    pub parameters: Vec<&'a str>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Parameter<'a> {
    pub name: &'a str,
    pub ty: Type<'a>,
    pub span: Span,
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Type<'a> {
    Void,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Char,
    Str,
    Array(Box<Type<'a>>, Span),
    Object(&'a str, Span),
    TypeArg(Box<Type<'a>>, Vec<Type<'a>>, Span),
    Function(Vec<Type<'a>>, Box<Type<'a>>, Span),
    Tuple(Vec<Type<'a>>, Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct TypeParameter<'a> {
    pub name: &'a str,
    pub constraints: Vec<Constraint<'a>>,
    pub span: Span,
}

impl TypeParameter<'_> {
    pub fn new<'a>(name: &'a str, constraints: Vec<Constraint<'a>>, span: Span) -> TypeParameter<'a> {
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
pub struct Signal<'a> {
    pub name: &'a str,
    pub is_static: bool,
    pub parameters: Vec<Type<'a>>,
    pub span: Span,
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
        label: Option<&'a str>,
        test: Expression<'a>,
        body: Vec<Statement<'a>>,
        span: Span,
    },
    For {
        label: Option<&'a str>,
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
        label: Option<&'a str>,
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
        label: Option<&'a str>,
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
    Variable(&'a str, Span),
    Tuple(Vec<Pattern<'a>>, Span),
    Constant(Constant<'a>),
    WildCard(Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Constant<'a> {
    Integer(&'a str, Span),
    Float(&'a str, Span),
    String(&'a str, Span),
    Character(&'a str, Span),
    Bool(bool, Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Expression<'a> {
    Variable(&'a str, Span),
    Literal(Literal<'a>),
    This(Span),
    Call {
        name: Box<Expression<'a>>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span,
    },
    MemberAccess {
        object: Box<Expression<'a>>,
        field: PathName<'a>,
        span: Span,
    },
    Closure {
        params: Vec<ClosureParameter<'a>>,
        return_type: Option<Type<'a>>,
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
    Connect {
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: &'a str,
        span: Span,
    },
    Disconnect {
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: &'a str,
        span: Span,
    },
    Loop {
        label: Option<&'a str>,
        body: Vec<Statement<'a>>,
        span: Span,
    },
    Continue(Option<&'a str>, Span),
    Break(Option<&'a str>, Option<Box<Expression<'a>>>, Span),
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
    Emit(&'a str, Span),
}

impl Expression<'_> {
    pub fn new_call<'a>(
        name: Box<Expression<'a>>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::Call {
            name,
            type_args,
            args,
            span
        }
    }

    pub fn new_member_access<'a>(object: Box<Expression<'a>>, field: PathName<'a>, span: Span) -> Expression<'a> {
        Expression::MemberAccess {
            object,
            field,
            span
        }
    }

    pub fn new_closure_expression<'a>(
        params: Vec<ClosureParameter<'a>>,
        return_type: Option<Type<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::Closure {
            params,
            return_type,
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

    pub fn new_connect_expression<'a>(
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: &'a str,
        span: Span,
    ) -> Expression<'a> {
        Expression::Connect {
            source,
            destination,
            signal_name,
            span,
        }
    }

    pub fn new_disconnect_expression<'a>(
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: &'a str,
        span: Span,
    ) -> Expression<'a> {
        Expression::Disconnect {
            source,
            destination,
            signal_name,
            span,
        }
    }

    pub fn new_loop_expression<'a>(
        label: Option<&'a str>,
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
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Literal<'a> {
    Constant(Constant<'a>),
    Void(Span),
    Tuple(Vec<Expression<'a>>, Span),
    Array(Vec<Expression<'a>>, Span),
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum ClosureParameter<'a> {
    Typed(Parameter<'a>),
    Untyped(&'a str, Span),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UnaryOperator {
    Neg,
    Not,
    Try,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Concat,
    Index,
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
