//! This module defines the structure of a class file
//! This file format is used to store class definitions, including their members, signals, and bytecode.
//! The file is structured in a way that allows for easy parsing and manipulation of class data.
//!
//! Here is the structure of the class file as binary data using a vaguely Rust-like format:
//! ```rust
//! type StringIndex = u64;
//! type BytecodeIndex = u64;
//! type SignatureIndex = u64;
//! type TypeTag = u8;
//!
//! struct ClassFile {
//!     magic: u8,
//!     major_version: u8,
//!     minor_version: u8,
//!     name: StringIndex,
//!     parents_size: u8,
//!     padding: [u8; 3],
//!     parents: [StringIndex; parents_size],
//!     vtables_size: u64,
//!     vtables: [VTable; vtables_size],
//!     members_size: u64,
//!     members: [Member; members_size],
//!     signals_size: u64,
//!     signals: [Signal; signals_size],
//!     bytecode_table_size: u64,
//!     bytecode_table: [BytecodeEntry; bytecode_table_size],
//!     string_table_size: u64,
//!     string_table: [StringEntry; string_table_size],
//!     signature_table_size: u64,
//!     signature_table: [SignatureEntry; signature_table_size]
//! }
//!
//! VTable {
//!     vtable_size: u64,
//!     functions: [VTableEntry; vtable_size],
//! }
//!
//! VTableEntry {
//!     class_name: StringIndex,
//!     sub_class_name: StringIndex,
//!     name: StringIndex,
//!     responds_to: StringIndex,
//!     signature: SignatureIndex,
//!     bytecode: BytecodeIndex,
//! }
//!
//! Member {
//!     name: StringIndex,
//!     type_tag: TypeTag,
//! }
//!
//! Signal {
//!     name: StringIndex,
//!     is_static: bool,
//!     signature: SignatureIndex,
//! }
//!
//! BytecodeEntry {
//!     code_size: u64,
//!     code: [u8; code_size],
//! }
//!
//! StringEntry {
//!     length: u64,
//!     value: [u8; length],
//! }
//! ```
//!
use crate::TypeTag;




/// Index into the string table
type StringIndex = u64;
/// Index into the bytecode table
type BytecodeIndex = u64;
/// Index into the signature table
type SignatureIndex = u64;

#[derive(PartialEq, Debug)]
pub struct ClassFile<'a> {
    /// Magic number to identify the file
    pub magic: u8,
    /// Major, minor, and patch version numbers
    pub major_version: u8,
    /// Major, minor, and patch version numbers
    pub minor_version: u8,
    /// Major, minor, and patch version numbers
    pub patch_version: u8,
    /// Class name
    pub name: StringIndex,
    /// Parent class names
    pub parents: Vec<StringIndex>,
    /// Virtual tables
    pub vtables: Vec<VTable<'a>>,
    /// Members and their types
    pub members: Vec<Member>,
    /// Signals and their types
    pub signals: Vec<Signal>,
    /// Where the bytecode is stored
    /// This table is 1 indexed to allow for methods to be empty
    pub(crate) bytecode_table: Vec<BytecodeEntry<'a>>,
    /// String table
    /// This table is 1 indexed to allow for StringIndices 0 value to mean "null"
    pub(crate) string_table: Vec<StringEntry<'a>>,
    /// Signature table
    /// This holds the signatures of methods
    pub signature_table: Vec<SignatureEntry>,
}

impl ClassFile<'_> {
    pub fn new<'input>(binary: &'input [u8]) -> ClassFile<'input> {
        let mut index = 0;
        let magic = binary[0];
        let major_version = binary[1];
        let minor_version = binary[2];
        let patch_version = binary[3];
        index += 4;
        let name = u64::from_le_bytes([
            binary[4], binary[5], binary[6], binary[7],
            binary[8], binary[9], binary[10], binary[11]
        ]);
        index += 8;
        let parents_size = binary[12];
        index += 1;

        index += 3; // Weird padding of 3 bytes
        
        let mut parents = Vec::new();
        for _ in 0..parents_size {
            let parent = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            parents.push(parent);
            index += std::mem::size_of::<StringIndex>();
        }
        let vtables_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += std::mem::size_of::<u64>();
        let mut vtables = Vec::new();
        for _ in 0..vtables_size {
            let vtable_size = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += std::mem::size_of::<u64>();
            let functions = unsafe {
                std::slice::from_raw_parts(
                    binary.as_ptr().add(index) as *const VTableEntry,
                    vtable_size as usize
                )
            };
            vtables.push(VTable {
                functions
            });
            index += vtable_size as usize * std::mem::size_of::<VTableEntry>();
        }

        let members_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += std::mem::size_of::<u64>();

        let mut members = Vec::new();
        for _ in 0..members_size {
            let name = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += std::mem::size_of::<StringIndex>();
            let type_tag = unsafe {
                std::ptr::read(binary.as_ptr().add(index) as *const u8)
            };
            let tag = type_tag.into();
            members.push(Member {
                name,
                type_tag: tag
            });
            index += std::mem::size_of::<u8>();
        }

        
        let signals_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += 8;

        let mut signals = Vec::new();
        for _ in 0..signals_size {
            let name = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += std::mem::size_of::<StringIndex>();
            let is_static = unsafe {
                std::ptr::read(binary.as_ptr().add(index) as *const u8)
            } != 0;
            index += std::mem::size_of::<u8>();
            let signature = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += std::mem::size_of::<SignatureIndex>();
            signals.push(Signal {
                name,
                is_static,
                signature
            });
        }

        let mut bytecode_table = Vec::new();
        let bytecode_table_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += 8;

        for _ in 0..bytecode_table_size {
            let code_size = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += 8;
            let code = unsafe {
                std::slice::from_raw_parts(
                    binary.as_ptr().add(index),
                    code_size as usize
                )
            };
            bytecode_table.push(BytecodeEntry {
                code
            });
            index += code_size as usize;
        }

        let string_table_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += std::mem::size_of::<u64>();

        let mut string_table = Vec::new();
        for _ in 0..string_table_size {
            let length = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += 8;
            let value = unsafe {
                std::str::from_utf8_unchecked(
                    std::slice::from_raw_parts(
                        binary.as_ptr().add(index),
                        length as usize
                    )
                )
            };
            string_table.push(StringEntry {
                value
            });
            index += length as usize;
        }

        let signature_table_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += std::mem::size_of::<u64>();

        let mut signature_table = Vec::new();
        for _ in 0..signature_table_size {
            let length = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += std::mem::size_of::<u64>();
            let mut types = Vec::new();
            for _ in 0..length {
                let type_tag = unsafe {
                    std::ptr::read(binary.as_ptr().add(index) as *const u8)
                };
                let tag = type_tag.into();
                types.push(tag);
                index += std::mem::size_of::<u8>();
            }
            signature_table.push(SignatureEntry {
                types
            });
        }

        ClassFile {
            magic,
            major_version,
            minor_version,
            patch_version,
            name,
            parents,
            vtables,
            members,
            signals,
            bytecode_table,
            string_table,
            signature_table
        }
        
    }

    pub fn index_string_table(&self, index: StringIndex) -> &str {
        &self.string_table[(index - 1) as usize].value
    }

    pub fn index_bytecode_table(&self, index: StringIndex) -> &BytecodeEntry {
        &self.bytecode_table[(index - 1) as usize]
    }


    #[inline]
    pub fn as_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        binary.push(self.magic);
        binary.push(self.major_version);
        binary.push(self.minor_version);
        binary.push(self.patch_version);
        binary.extend_from_slice(&self.name.to_le_bytes());
        binary.push(self.parents.len() as u8);

        binary.extend_from_slice(&[0, 0, 0]); // Weird padding of 3 bytes

        binary.extend_from_slice(&self.parents.iter().flat_map(|&p| p.to_le_bytes()).collect::<Vec<u8>>());
        binary.extend_from_slice(&self.vtables.len().to_le_bytes());
        for vtable in &self.vtables {
            binary.extend_from_slice(&(vtable.functions.len() as u64).to_le_bytes());
            for function in vtable.functions {
                binary.extend_from_slice(&function.class_name.to_le_bytes());
                binary.extend_from_slice(&function.sub_class_name.to_le_bytes());
                binary.extend_from_slice(&function.name.to_le_bytes());
                binary.extend_from_slice(&function.responds_to.to_le_bytes());
                binary.extend_from_slice(&function.signature.to_le_bytes());
                binary.extend_from_slice(&function.bytecode.to_le_bytes());
            }
        }
        binary.extend_from_slice(&(self.members.len() as u64).to_le_bytes());
        for member in self.members.iter() {
            binary.extend_from_slice(&member.name.to_le_bytes());
            binary.push(member.type_tag.as_byte());
        }
        binary.extend_from_slice(&(self.signals.len() as u64).to_le_bytes());
        for signal in self.signals.iter() {
            binary.extend_from_slice(&signal.name.to_le_bytes());
            binary.push(signal.is_static as u8);
            binary.extend_from_slice(&signal.signature.to_le_bytes());
        }
        binary.extend_from_slice(&(self.bytecode_table.len() as u64).to_le_bytes());
        for bytecode in &self.bytecode_table {
            binary.extend_from_slice(&(bytecode.code.len() as u64).to_le_bytes());
            binary.extend_from_slice(bytecode.code);
        }
        binary.extend_from_slice(&(self.string_table.len() as u64).to_le_bytes());
        for string in &self.string_table {
            binary.extend_from_slice(&(string.value.len() as u64).to_le_bytes());
            binary.extend_from_slice(string.value.as_bytes());
        }
        binary.extend_from_slice(&(self.signature_table.len() as u64).to_le_bytes());
        for signature in &self.signature_table {
            binary.extend_from_slice(&(signature.types.len() as u64).to_le_bytes());
            for type_tag in &signature.types {
                binary.push(type_tag.as_byte());
            }
        }

        binary
    }
}

impl<'input> From<&'input [u8]> for ClassFile<'input> {
    fn from(binary: &'input [u8]) -> Self {
        ClassFile::new(binary)
    }
}

impl Into<Vec<u8>> for ClassFile<'_> {
    fn into(self) -> Vec<u8> {
        self.as_binary()
    }
}


/// Represents a member of a class
#[derive(PartialEq, Debug)]
pub struct Member {
    name: StringIndex,
    type_tag: TypeTag,
}

/// Represents a virtual table for a class
#[derive(PartialEq, Debug)]
pub struct VTable<'a> {
    functions: &'a [VTableEntry],
}

#[derive(PartialEq, Debug)]
#[repr(C)]
pub struct VTableEntry {
    /// The name of the class to start looking for the function
    class_name: StringIndex,
    /// The name of the subclass the method is defined in
    sub_class_name: StringIndex,
    /// The name of the function
    name: StringIndex,
    /// The name of the signal this method responds to
    responds_to: StringIndex,
    /// The signature of the function
    signature: SignatureIndex,
    /// The index of the bytecode for this function
    bytecode: BytecodeIndex,
}

/// Represents a bytecode entry
/// This is a slice of bytes
#[derive(PartialEq, Debug)]
pub struct BytecodeEntry<'a> {
    code: &'a [u8], 
}

/// Represents a signal in a class
/// This is a signal that can be emitted by a class
/// A static signal is a signal that is broadcasted to all objects that are connected to the class staticly
#[derive(PartialEq, Debug)]
pub struct Signal {
    /// The name of the signal
    name: StringIndex,
    /// Whether the signal is static or not
    is_static: bool,
    /// The signature of the signal
    /// A signal always has a return type of void
    signature: SignatureIndex,
}

/// Represents a string entry in the string table
#[derive(PartialEq, Debug)]
pub struct StringEntry<'a> {
    value: &'a str
}

/// Represents a signature entry in the signature table
#[derive(PartialEq, Debug)]
pub struct SignatureEntry {
    /// The types of the parameters in the signature
    /// The return type is always the first type in the vector
    types: Vec<TypeTag> 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_into_binary_and_back() {
        let parents = vec![1, 2];
        let vtables = vec![
            VTable {
                functions: &[
                    VTableEntry {
                        class_name: 1,
                        sub_class_name: 2,
                        name: 1,
                        responds_to: 2,
                        signature: 3,
                        bytecode: 4
                    }
                ]
            }
        ];

        let members = vec![
            Member {
                name: 1,
                type_tag: TypeTag::U8
            }
        ];

        let signals = vec![
            Signal {
                name: 1,
                is_static: true,
                signature: 2
            }
        ];

        let bytecode_table = vec![
            BytecodeEntry {
                code: &[0, 1, 2, 3]
            }
        ];

        let string_table = vec![
            StringEntry {
                value: "Hello"
            }
        ];

        let signature_table = vec![
            SignatureEntry {
                types: vec![TypeTag::U8, TypeTag::I32]
            }
        ];


        let class_file = ClassFile {
            magic: 0,
            major_version: 1,
            minor_version: 2,
            patch_version: 3,
            name: 1,
            parents,
            vtables,
            members,
            signals,
            bytecode_table,
            string_table,
            signature_table
        };

        let binary = class_file.as_binary();

        let class_file2 = ClassFile::new(&binary);

        assert_eq!(class_file, class_file2);
        
    }
}
