use std::collections::HashMap;

use super::{Context, Reference, Symbol, VTableIndex};

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
    Sized(usize),
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
            Sized(x) => x,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ClassMember {
    pub name: Symbol,
    pub data: ClassMemberData,
}
#[derive(Copy, Clone, Debug)]
pub enum ClassMemberData {
    Byte(u8),
    Short(u16),
    Int(u32),
    Long(u64),
    Float(f32),
    Double(f64),
    Object(Reference)
}

#[derive(Debug)]
pub struct Class {
    pub name: Symbol,
    pub parents: Vec<Symbol>,
    pub vtables: HashMap<(Symbol, Option<Symbol>), VTableIndex>,
    pub members: Vec<MemberInfo>,
    pub static_methods: VTableIndex,
    pub class_members: Vec<ClassMember>,
    pub init_function: fn(&mut Context),
}

impl Class {
    pub fn new(
        name: Symbol,
        parents: Vec<Symbol>,
        vtables: HashMap<(Symbol, Option<Symbol>), VTableIndex>,
        members: Vec<MemberInfo>,
        static_methods: VTableIndex,
        class_members: Vec<ClassMember>,
        init_function: fn(&mut Context),
    ) -> Self {
        Class {
            name,
            parents,
            vtables,
            members,
            static_methods,
            class_members,
            init_function,
        }
    }
    
    pub fn get_member_size(&self) -> usize {
        let out = self.members.iter().map(|member| member.get_size()).sum();
        out
    }

    pub fn get_vtable(&self, sym: &(Symbol, Option<Symbol>)) -> Option<VTableIndex> {
        self.vtables.get(sym).map(|index| *index)
    }

    pub fn get_member(&self, index: usize) -> Option<&ClassMember> {
        self.class_members.get(index)
    }
    pub fn get_member_mut(&mut self, index: usize) -> Option<&mut ClassMember> {
        self.class_members.get_mut(index)
    }

    pub fn get_object_member_indices(&self) -> Vec<usize> {
        let mut output = Vec::new();
        for (i, info) in self.members.iter().enumerate() {
            match info.ty {
                TypeTag::Object => output.push(i),
                _ => {}
            }
        }
        output
    }
}

unsafe impl Send for Class {}
unsafe impl Sync for Class {}

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

