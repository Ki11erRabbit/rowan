use either::Either;
use rowan_shared::TypeTag;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Text<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl Text<'_> {
    pub fn as_str(&self) -> &str {
        match self {
            Text::Borrowed(s) => s,
            Text::Owned(s) => s,
        }
    }
}

impl std::ops::Deref for Text<'_> {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            Text::Borrowed(s) => s,
            Text::Owned(s) => s,
        }
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(s: &'a str) -> Text<'a> {
        Text::Borrowed(s)
    }
}

impl From<String> for Text<'_> {
    fn from(s: String) -> Text<'static> {
        Text::Owned(s)
    }
}

impl std::fmt::Display for Text<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Text::Borrowed(s) => write!(f, "{}", s),
            Text::Owned(s) => write!(f, "{}", s),
        }
    }
}

impl AsRef<str> for Text<'_> {
    fn as_ref(&self) -> &str {
        match self {
            Text::Borrowed(s) => s,
            Text::Owned(s) => s,
        }
    }
}

impl std::cmp::PartialEq<str> for Text<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}

impl std::cmp::PartialEq<Text<'_>> for str {
    fn eq(&self, other: &Text<'_>) -> bool {
        self == other.as_ref()
    }
}

impl std::cmp::PartialEq<&str> for Text<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}

impl std::cmp::PartialEq<Text<'_>> for &str {
    fn eq(&self, other: &Text<'_>) -> bool {
        *self == other.as_ref()
    }
}

impl std::cmp::PartialEq<String> for Text<'_> {
    fn eq(&self, other: &String) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl std::cmp::PartialEq<Text<'_>> for String {
    fn eq(&self, other: &Text<'_>) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl std::borrow::Borrow<str> for Text<'_> {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}




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
pub enum Visibility {
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
    pub segments: Vec<Text<'a>>,
    pub span: Span,
}

impl PathName<'_> {
    pub fn new<'a>(segments: Vec<Text<'a>>, span: Span) -> PathName<'a> {
        PathName { segments, span }
    }
}

impl std::fmt::Display for PathName<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", segment)?;
        }
        Ok(())
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
    pub parents: Vec<ParentDec<'a>>,
    pub members: Vec<Member<'a>>,
    pub methods: Vec<Method<'a>>,
    pub type_params: Vec<TypeParameter<'a>>,
    pub span: Span,
}

impl Class<'_> {
    pub fn new<'a>(
        name: Text<'a>,
        parents: Vec<ParentDec<'a>>,
        members: Vec<Member<'a>>,
        methods: Vec<Method<'a>>,
        type_params: Vec<TypeParameter<'a>>,
        span: Span
    ) -> Class<'a> {
        Class {
            name,
            parents,
            members,
            methods,
            type_params,
            span
        }
    }
}


#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct ParentDec<'a> {
    pub name: Text<'a>,
    pub type_params: Vec<TypeParameter<'a>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum ClassMember<'a> {
    Member(Member<'a>),
    Method(Method<'a>),
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
    F32,
    F64,
    Char,
    Str,
    Array(Box<Type<'a>>, Span),
    Object(Text<'a>, Span),
    TypeArg(Box<Type<'a>>, Vec<Type<'a>>, Span),
    Function(Vec<Type<'a>>, Box<Type<'a>>, Span),
    Tuple(Vec<Type<'a>>, Span),
}

impl Type<'_> {
    pub fn is_integer(&self) -> bool {
        match self {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => true,
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Type::F32 | Type::F64 => true,
            _ => false,
        }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => true,
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Type::I8 | Type::I16 | Type::I32 | Type::I64 => true,
            _ => false,
        }
    }

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
pub struct Signal<'a> {
    pub name: Text<'a>,
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
    Integer(Text<'a>, Option<Type<'a>>, Span),
    Float(Text<'a>, Option<Type<'a>>, Span),
    String(Text<'a>, Span),
    Character(Text<'a>, Span),
    Bool(bool, Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Expression<'a> {
    Variable(Text<'a>, Option<Type<'a>>, Span),
    Literal(Literal<'a>),
    This(Span),
    Call {
        name: Box<Expression<'a>>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span,
        annotation: Option<Type<'a>>,
    },
    StaticCall {
        name: PathName<'a>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span,
        annotation: Option<Type<'a>>,
    },
    MemberAccess {
        object: Box<Expression<'a>>,
        field: PathName<'a>,
        span: Span,
        annotation: Option<Type<'a>>,
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
    Connect {
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: Text<'a>,
        span: Span,
    },
    Disconnect {
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: Text<'a>,
        span: Span,
    },
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
    Emit(Text<'a>, Vec<Expression<'a>>,Span),
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
            span,
            annotation: None,
        }
    }

    pub fn new_static_call<'a>(
        name: PathName<'a>,
        type_args: Vec<Type<'a>>,
        args: Vec<Expression<'a>>,
        span: Span
    ) -> Expression<'a> {
        Expression::StaticCall {
            name,
            type_args,
            args,
            span,
            annotation: None,
        }
    }

    pub fn new_member_access<'a>(object: Box<Expression<'a>>, field: PathName<'a>, span: Span) -> Expression<'a> {
        Expression::MemberAccess {
            object,
            field,
            span,
            annotation: None,
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

    pub fn new_connect_expression<'a>(
        source: Box<Expression<'a>>,
        destination: Box<Expression<'a>>,
        signal_name: Text<'a>,
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
        signal_name: Text<'a>,
        span: Span,
    ) -> Expression<'a> {
        Expression::Disconnect {
            source,
            destination,
            signal_name,
            span,
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

    pub fn get_type(&self) -> Option<Either<Type, ()>> {
        match self {
            Expression::As {typ, ..} => Some(Either::Left(typ.clone())),
            Expression::Into {typ, ..} => Some(Either::Left(typ.clone())),
            Expression::Literal(Literal::Constant(Constant::Bool(_, _))) => {
                Some(Either::Left(Type::U8))
            }
            Expression::Literal(Literal::Constant(Constant::Character(_, _))) => {
                Some(Either::Left(Type::U32))
            }
            Expression::Literal(Literal::Constant(Constant::Integer(_, ty, _))) => {
                ty.clone().map(|t| Either::Left(t))
            }
            Expression::Literal(Literal::Constant(Constant::Float(_, ty, _))) => {
                ty.clone().map(|t| Either::Left(t))
            }
            Expression::Literal(Literal::Constant(Constant::String(_, _))) => {
                Some(Either::Left(Type::Str))
            }
            Expression::Literal(Literal::Void(_)) => {
                Some(Either::Left(Type::Void))
            }
            Expression::Literal(Literal::Tuple(_, ty, _)) => {
                ty.clone().map(|t| Either::Left(t))
            }
            Expression::Literal(Literal::Array(_, ty, _)) => {
                ty.clone().map(|t| Either::Left(t))
            }
            Expression::Variable(_, ty, _) => {
                ty.clone().map(|t| Either::Left(t))
            }
            Expression::BinaryOperation { operator: BinaryOperator::Add, left,  .. } => {
                left.get_type()
            }
            Expression::Call {annotation, ..} => {
                annotation.clone().map(|t| Either::Left(t))
            }
            Expression::This(_) => Some(Either::Right(())),
            Expression::MemberAccess {
                annotation, ..
            } => {
                annotation.clone().map(|t| Either::Left(t))
            }
            x => todo!("Expression::get_type {:?}", x),
        }
    }

    /*pub fn set_type(&mut self, ty: Type) {
        match self {
            Expression::Literal(Literal::Constant(Constant::Bool(_, _))) => {}
            Expression::Literal(Literal::Constant(Constant::Character(_, _))) => {}
            Expression::Literal(Literal::Constant(Constant::Integer(_, annotation, _))) => {
                *annotation = Some(ty);
            }
            Expression::Literal(Literal::Constant(Constant::Float(_, annotation, _))) => {
                *annotation = Some(ty);
            }
            Expression::Literal(Literal::Constant(Constant::String(_, _))) => {}
            Expression::Literal(Literal::Void(_)) => {}
            Expression::BinaryOperation { .. } => {}
            Expression::Variable(_, annotation, _) => {
                *annotation = Some(ty);
            }
            Expression::Literal(Literal::Tuple(_, annotation, _)) => {
                *annotation = Some(ty);
            }
            Expression::Literal(Literal::Array(_, annotation, _)) => {
                *annotation = Some(ty);
            }
            x => todo!("setting type for expression {:?}", x),
        }
    }*/
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Literal<'a> {
    Constant(Constant<'a>),
    Void(Span),
    Tuple(Vec<Expression<'a>>, Option<Type<'a>>, Span),
    Array(Vec<Expression<'a>>, Option<Type<'a>>, Span),
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum ClosureParameter<'a> {
    Typed(Parameter<'a>),
    Untyped(Pattern<'a>, Span),
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
