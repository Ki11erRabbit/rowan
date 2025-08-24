use crate::classfile::{BytecodeEntry, BytecodeIndex, SignatureEntry, StringEntry, StringIndex, VTable, VTableEntry};
use crate::RowanClassFileUtils;

#[derive(PartialEq, Debug)]
pub struct InterfaceFile {
    /// Magic number to identify the file
    pub magic: u8,
    /// Type of Class File, (Class, Interface, or InterfaceImpl)
    pub r#type: u8,
    /// Major, minor, and patch version numbers
    pub major_version: u8,
    /// Major, minor, and patch version numbers
    pub minor_version: u8,
    /// Major, minor, and patch version numbers
    pub patch_version: u8,
    /// Interface name
    pub name: StringIndex,
    /// Virtual tables
    pub vtable: VTable,
    /// Where the bytecode is stored
    /// This table is 1 indexed to allow for methods to be empty
    pub(crate) bytecode_table: Vec<BytecodeEntry>,
    /// String table
    /// This table is 1 indexed to allow for StringIndices 0 value to mean "null"
    pub(crate) string_table: Vec<StringEntry>,
    /// Signature table
    /// This holds the signatures of methods
    pub signature_table: Vec<SignatureEntry>,
}

impl InterfaceFile {

    pub fn index_string_table(&self, index: StringIndex) -> &str {
        assert_ne!(index, 0, "string index should not be zero");
        std::str::from_utf8(&self.string_table[(index - 1) as usize].value).unwrap()
    }

    pub fn index_bytecode_table(&self, index: BytecodeIndex) -> &BytecodeEntry {
        assert_ne!(index, 0, "bytecode index should not be zero");
        &self.bytecode_table[(index - 1) as usize]
    }
    pub fn new_from_parts(
        name: StringIndex, 
        vtable: VTable, 
        bytecode_table: Vec<BytecodeEntry>,
        string_table: Vec<StringEntry>,
        signature_table: Vec<SignatureEntry>,
    ) -> Self {
        InterfaceFile {
            magic: 0,
            r#type: 1,
            major_version: 0,
            minor_version: 0,
            patch_version: 0,
            name,
            vtable,
            bytecode_table,
            string_table,
            signature_table,
        }
    }
    
    pub fn new(binary: &[u8]) -> Self {
        let mut index = 0;
        let magic = binary[0];
        // asserting that we are indeed a Interface file and not an Class or InterfaceImpl
        assert_eq!(binary[1], 1);
        let major_version = binary[2];
        let minor_version = binary[3];
        let patch_version = binary[4];
        index += 5;
        let name = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += size_of::<u64>();
        
        index += 3; // padding of 3 bytes

        let class_name = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += size_of::<u64>();
        let sub_class_name = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += size_of::<u64>();
        let vtable_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += size_of::<u64>();
        let functions = unsafe {
            std::slice::from_raw_parts(
                binary.as_ptr().add(index) as *const VTableEntry,
                vtable_size as usize
            )
        };
        let vtable = VTable {
            class_name,
            sub_class_name,
            functions: functions.to_vec()
        };
        index += vtable_size as usize * size_of::<VTableEntry>();

        let mut bytecode_table = Vec::new();
        let bytecode_table_size = u64::from_le_bytes([
            binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
            binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
        ]);
        index += size_of::<u64>();

        for _ in 0..bytecode_table_size {
            let code_size = u64::from_le_bytes([
                binary[index], binary[index + 1], binary[index + 2], binary[index + 3],
                binary[index + 4], binary[index + 5], binary[index + 6], binary[index + 7]
            ]);
            index += size_of::<u64>();
            assert!(code_size < binary.len() as u64, "Code size is too too large");
            let code = unsafe {
                std::slice::from_raw_parts(
                    binary.as_ptr().add(index),
                    code_size as usize
                )
            };
            bytecode_table.push(BytecodeEntry {
                code: code.to_vec()
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
                std::slice::from_raw_parts(
                    binary.as_ptr().add(index),
                    length as usize
                )
            };
            string_table.push(StringEntry {
                value: value.to_vec()
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
        
        InterfaceFile {
            magic,
            r#type: 1,
            major_version,
            minor_version,
            patch_version,
            name,
            vtable,
            bytecode_table,
            string_table,
            signature_table
        }
    }
    
    #[inline]
    pub fn as_binary(&self) -> Vec<u8> {
        let mut binary = Vec::new();
        binary.push(self.magic);
        binary.push(self.r#type);
        binary.push(self.major_version);
        binary.push(self.minor_version);
        binary.push(self.patch_version);
        binary.extend_from_slice(&self.name.to_le_bytes());
        
        binary.extend_from_slice(&[0u8; 3]); // padding of 3 bytes

        binary.extend_from_slice(&self.vtable.class_name.to_le_bytes());
        binary.extend_from_slice(&self.vtable.sub_class_name.to_le_bytes());
        binary.extend_from_slice(&(self.vtable.functions.len() as u64).to_le_bytes());
        for function in &self.vtable.functions {
            binary.extend_from_slice(&function.name.to_le_bytes());
            binary.extend_from_slice(&function.signature.to_le_bytes());
            binary.extend_from_slice(&function.bytecode.to_le_bytes());
        }
        binary.extend_from_slice(&(self.bytecode_table.len() as u64).to_le_bytes());
        for bytecode in &self.bytecode_table {
            binary.extend_from_slice(&(bytecode.code.len() as u64).to_le_bytes());
            binary.extend_from_slice(&bytecode.code);
        }
        binary.extend_from_slice(&(self.string_table.len() as u64).to_le_bytes());
        for string in &self.string_table {
            binary.extend_from_slice(&(string.value.len() as u64).to_le_bytes());
            binary.extend_from_slice(&string.value);
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

impl From<&[u8]> for InterfaceFile {
    fn from(binary: &[u8]) -> Self {
        InterfaceFile::new(binary)
    }
}

impl Into<Vec<u8>> for InterfaceFile {
    fn into(self) -> Vec<u8> {
        self.as_binary()
    }
}

impl RowanClassFileUtils for InterfaceFile {
    fn index_string_table(&self, index: StringIndex) -> &str {
        self.index_string_table(index)
    }
    fn index_bytecode_table(&self, index: BytecodeIndex) -> &BytecodeEntry {
        self.index_bytecode_table(index)
    }
}