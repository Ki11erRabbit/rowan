use std::collections::{HashMap, HashSet};
use rowan_shared::bytecode::linked::Bytecode;
use crate::context::WrappedReference;
use crate::runtime::object::Object;
use super::{Reference, Symbol, VTableIndex};

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

    pub fn tag(self) -> u8 {
        use TypeTag::*;
        match self {
            U8 => 0,
            U16 => 1,
            U32 => 2,
            U64 => 3,
            I8 => 4,
            I16 => 5,
            I32 => 6,
            I64 => 7,
            F32 => 8,
            F64 => 9,
            Object => 10,
            Str => 11,
            Void => 12,
            _ => panic!("native types are not ffi safe"),
        }
    }

    pub fn from_tag(tag: u8) -> TypeTag {
        match tag {
            0 => TypeTag::U8,
            1 => TypeTag::U16,
            2 => TypeTag::U32,
            3 => TypeTag::U64,
            4 => TypeTag::I8,
            5 => TypeTag::I16,
            6 => TypeTag::I32,
            7 => TypeTag::I64,
            8 => TypeTag::F32,
            9 => TypeTag::F64,
            10 => TypeTag::Object,
            11 => TypeTag::Str,
            12 => TypeTag::Void,
            _ => panic!("invalid tag"),
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
    pub parent: Symbol,
    pub vtables: HashMap<Symbol, VTableIndex>,
    pub members: Vec<MemberInfo>,
    pub static_methods: VTableIndex,
    pub class_members: Vec<ClassMember>,
    pub init_function: Option<Box<[Bytecode]>>,
    pub drop_function: Option<extern "C" fn(&mut Object)>,
}

impl Class {
    pub fn new(
        name: Symbol,
        parent: Symbol,
        vtables: HashMap<Symbol, VTableIndex>,
        members: Vec<MemberInfo>,
        static_methods: VTableIndex,
        class_members: Vec<ClassMember>,
        init_function: Option<Box<[Bytecode]>>,
        drop_function: Option<extern "C" fn(&mut Object)>,

    ) -> Self {
        Class {
            name,
            parent,
            vtables,
            members,
            static_methods,
            class_members,
            init_function,
            drop_function,
        }
    }
    
    pub fn get_member_size(&self) -> usize {
        let mut out = 0;
        for member in &self.members {
            let size = member.get_size();
            let mut padding = std::mem::size_of::<usize>();
            while padding < size {
                padding += std::mem::size_of::<usize>();
            }
            out += size + (padding - size);  // we are padding the struct so that it is compatible with C
        }
        out + 8 //padding of 8 bytes to make valgrind happy
    }

    pub fn get_vtable(&self, sym: &Symbol) -> Option<VTableIndex> {
        self.vtables.get(sym).map(|index| *index)
    }

    pub fn get_member(&self, index: usize) -> Option<&ClassMember> {
        self.class_members.get(index)
    }
    pub fn get_member_mut(&mut self, index: usize) -> Option<&mut ClassMember> {
        self.class_members.get_mut(index)
    }

    pub fn get_drop(&self) -> Option<extern "C" fn(&mut Object)> {
        self.drop_function
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
    
    pub fn collect_members(&self, live_memory: &mut HashSet<Reference>) {
        for member in &self.class_members {
            match member {
                ClassMember { data: ClassMemberData::Object(object), ..} => {
                    live_memory.insert(*object);
                }
                _ => {}
            }
        }
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

    pub fn get_size_and_padding(&self) -> usize {
        let size = self.ty.size();
        let mut padding = std::mem::size_of::<usize>();
        while padding < size {
            padding += std::mem::size_of::<usize>();
        }
        size + (std::mem::size_of::<usize>() - size)
    }

    pub fn has_native_type(&self) -> bool {
        match self.ty {
            TypeTag::Sized(_) => true,
            _ => false
        }
    }
}

