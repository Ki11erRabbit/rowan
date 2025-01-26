use std::collections::HashMap;

use super::{Symbol, VTableIndex};


pub enum TypeTag {
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
    Object, // u64
    Str,
}


pub struct Class {
    name: Symbol,
    parents: Vec<Symbol>,
    vtables: HashMap<(Symbol, Symbol), VTableIndex>,
    members: Vec<MemberInfo>,
    signals: Vec<SignalInfo>,
}


pub struct MemberInfo {
    name: Symbol,
    ty: TypeTag,
}


pub struct SignalInfo {
    name: Symbol,
    is_static: bool,
    arguments: Vec<TypeTag>,
}
