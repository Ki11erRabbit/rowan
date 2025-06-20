use std::collections::HashMap;

use super::{Context, Symbol, VTableIndex};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
    Void,
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
            Void => 1,
        }
    }
}

#[derive(Debug)]
pub struct Class {
    pub name: Symbol,
    pub parents: Vec<Symbol>,
    pub vtables: HashMap<(Symbol, Option<Symbol>), VTableIndex>,
    pub members: Vec<MemberInfo>,
    pub static_methods: VTableIndex,
}

impl Class {
    pub fn new(
        name: Symbol,
        parents: Vec<Symbol>,
        vtables: HashMap<(Symbol, Option<Symbol>), VTableIndex>,
        members: Vec<MemberInfo>,
        static_methods: VTableIndex
    ) -> Self {
        Class {
            name,
            parents,
            vtables,
            members,
            static_methods
        }
    }
    
    pub fn get_member_size(&self) -> usize {
        println!("\nsymbol: {}", self.name);
        let out = self.members.iter().map(|member| member.get_size()).sum();
        println!("member size: {}", out);
        out
    }

    pub fn get_vtable(&self, sym: &(Symbol, Option<Symbol>)) -> VTableIndex {
        *self.vtables.get(sym).unwrap()
    }
}


#[derive(Debug)]
pub struct MemberInfo {
    name: Symbol,
    ty: TypeTag,
}

impl MemberInfo {
    pub fn new(symbol: Symbol, ty: TypeTag) -> Self {
        MemberInfo {
            name: symbol,
            ty
        }
    }

    pub fn get_size(&self) -> usize {
        self.ty.size()
    }
}

