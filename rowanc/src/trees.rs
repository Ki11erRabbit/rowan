pub mod ast;
pub mod ir;

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

#[derive(Debug, Clone, Hash, PartialOrd)]
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
    Boolean,
    Native,
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

impl PartialEq for Type<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Void, Type::Void) => true,
            (Type::U8, Type::U8) => true,
            (Type::U16, Type::U16) => true,
            (Type::U32, Type::U32) => true,
            (Type::U64, Type::U64) => true,
            (Type::I8, Type::I8) => true,
            (Type::I16, Type::I16) => true,
            (Type::I32, Type::I32) => true,
            (Type::I64, Type::I64) => true,
            (Type::F32, Type::F32) => true,
            (Type::F64, Type::F64) => true,
            (Type::Char, Type::Char) => true,
            (Type::Str, Type::Str) => true,
            (Type::Native, Type::Native) => false,
            (Type::Object(l_txt, ..), Type::Object(r_txt, ..)) => {
                l_txt.as_str() == r_txt.as_str()
            }
            (Type::Array(l_array, ..), Type::Array(r_array, ..)) => {
                **l_array == **r_array
            }
            (Type::Function(l_args, l_return_ty, _), Type::Function(r_args, r_return_ty, _)) => {
                *l_args == *r_args && *l_return_ty == *r_return_ty
            }
            (Type::Tuple(l_ty, ..), Type::Tuple(r_ty, ..)) => {
                **l_ty == **r_ty
            }
            (Type::TypeArg(l_base, l_args, ..), Type::TypeArg(r_base, r_args, ..)) => {
                *l_base == *r_base && *l_args == *r_args
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub struct Annotation<'a> {
    pub name: Text<'a>,
    pub parameters: Vec<Text<'a>>,
    pub span: Span,
}

impl<'a> Annotation<'a> {
    pub fn new(name: Text<'a>, parameters: Vec<Text<'a>>, span: Span) -> Self {
        Annotation { name, parameters, span }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, PartialOrd)]
pub enum Constraint<'a> {
    Extends(Vec<Type<'a>>, Span),
    //TODO: add trait bounds
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