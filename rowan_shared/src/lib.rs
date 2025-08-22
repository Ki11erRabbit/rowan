pub mod bytecode;
pub mod classfile;
mod interfacefile;
mod interfaceimplfile;

/// Represents a type tag for a member or parameter
/// This represents all the primitive types
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TypeTag {
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
    Str,
    Object,
    Native,
}

impl TypeTag {
    fn as_byte(&self) -> u8 {
        match self {
            TypeTag::Void => 0,
            TypeTag::U8 => 1,
            TypeTag::U16 => 2,
            TypeTag::U32 => 3,
            TypeTag::U64 => 4,
            TypeTag::I8 => 5,
            TypeTag::I16 => 6,
            TypeTag::I32 => 7,
            TypeTag::I64 => 8,
            TypeTag::F32 => 9,
            TypeTag::F64 => 10,
            TypeTag::Str => 11,
            TypeTag::Object => 12,
            TypeTag::Native => 13,
        }
    }
}

impl From<u8> for TypeTag {
    fn from(value: u8) -> Self {
        match value {
            0 => TypeTag::Void,
            1 => TypeTag::U8,
            2 => TypeTag::U16,
            3 => TypeTag::U32,
            4 => TypeTag::U64,
            5 => TypeTag::I8,
            6 => TypeTag::I16,
            7 => TypeTag::I32,
            8 => TypeTag::I64,
            9 => TypeTag::F32,
            10 => TypeTag::F64,
            11 => TypeTag::Str,
            12 => TypeTag::Object,
            13 => TypeTag::Native,
            _ => unreachable!("unknown type"),
        }
    }
}

pub enum FileType {
    Class,
    Interface,
    InterfaceImpl,
}

pub fn identify_file(binary: &[u8]) -> FileType {
    assert!(binary.len() >= 2, "Binary is too short for a class file");
    match binary[0] {
        0 => FileType::Class,
        1 => FileType::Interface,
        2 => FileType::InterfaceImpl,
        _ => unreachable!(),
    }
}