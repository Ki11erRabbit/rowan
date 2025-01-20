pub mod bytecode;
pub mod classfile;



/// Represents a type tag for a member or parameter
/// This represents all the primitive types
#[derive(PartialEq, Debug)]
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
            _ => 12
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
            _ => TypeTag::Object
        }
    }
}
