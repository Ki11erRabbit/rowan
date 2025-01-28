use std::collections::HashMap;

use super::{Symbol, VTableIndex};

#[derive(Copy, Clone)]
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

impl TypeTag {
    pub fn size(self) -> usize {
        use TypeTag::*;
        match self {
            U8 => 1,
            U16 => 2,
            U32 => 4,
            U64 => 8,
            I8 => 1,
            I16 => 2,
            I32 => 4,
            I64 => 8,
            F32 => 4,
            F64 => 8,
            Object => 8,
            Str => 8,
        }
    }
}


pub struct Class {
    name: Symbol,
    parents: Vec<Symbol>,
    vtables: HashMap<(Symbol, Symbol), VTableIndex>,
    members: Vec<MemberInfo>,
    signals: Vec<SignalInfo>,
}

impl Class {
    pub fn get_member_size(&self) -> usize {
        self.members.iter().map(|member| member.get_size()).sum()
    }
}


pub struct MemberInfo {
    name: Symbol,
    ty: TypeTag,
}

impl MemberInfo {
    pub fn get_size(&self) -> usize {
        self.ty.size()
    }
}


pub struct SignalInfo {
    name: Symbol,
    is_static: bool,
    arguments: Vec<TypeTag>,
}
