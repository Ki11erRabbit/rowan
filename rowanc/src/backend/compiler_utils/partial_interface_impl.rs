use std::collections::HashMap;
use std::io::BufRead;
use rowan_shared::classfile::{BytecodeEntry, BytecodeIndex, SignatureEntry, SignatureIndex, StringEntry, StringIndex, VTable, VTableEntry};
use rowan_shared::interfaceimplfile::InterfaceImplFile;

#[derive(Debug)]
pub struct PartialInterfaceImpl {
    interface_name: StringIndex,
    implementer_name: StringIndex,
    /// Virtual tables
    vtable: VTable,
    /// Where the bytecode is stored
    /// This table is 1 indexed to allow for methods to be empty
    bytecode_table: Vec<BytecodeEntry>,
    /// String table
    /// This table is 1 indexed to allow for StringIndices 0 value to mean "null"
    string_table: Vec<StringEntry>,
    string_to_index: HashMap<String, StringIndex>,
    /// Signature table
    /// This holds the signatures of methods
    signature_table: Vec<SignatureEntry>,
    /// Maps method names to a vec of indices
    /// The first index is the index into the vtable table
    /// The second index is the index in the vtable itself
    method_to_function: HashMap<String, usize>,
}

impl PartialInterfaceImpl {
    pub fn new() -> PartialInterfaceImpl {
        PartialInterfaceImpl {
            interface_name: 0,
            implementer_name: 0,
            vtable: VTable::empty(),
            bytecode_table: Vec::new(),
            string_table: Vec::new(),
            string_to_index: HashMap::new(),
            signature_table: Vec::new(),
            method_to_function: HashMap::new(),
        }
    }


    pub fn index_string_table(&self, index: StringIndex) -> &str {
        std::str::from_utf8(&self.string_table[(index - 1) as usize].value).unwrap()
    }

    pub fn index_bytecode_table(&self, index: StringIndex) -> &BytecodeEntry {
        &self.bytecode_table[(index - 1) as usize]
    }

    pub fn get_vtable(&self) -> &VTable {
        &self.vtable
    }

    pub fn get_method_entry(&self, method_name: impl AsRef<str>) -> Option<VTableEntry> {
        let index = self.method_to_function.get(method_name.as_ref())?;
        self.vtable.functions.get(*index).cloned()
    }

    pub fn create_interface_file(self) -> InterfaceImplFile {
        InterfaceImplFile::new_from_parts(
            self.interface_name,
            self.implementer_name,
            self.vtable,
            self.bytecode_table,
            self.string_table,
            self.signature_table,
        )
    }

    pub fn add_signatures(&mut self, sigs: Vec<SignatureEntry>) {
        self.signature_table.extend(sigs);
    }

    pub fn set_interface_name(&mut self, name: &str) {
        self.interface_name = self.add_string(name);
        self.vtable.sub_class_name = self.interface_name;
    }

    pub fn set_implementer_name(&mut self, name: &str) {
        self.implementer_name = self.add_string(name);
        self.vtable.class_name = self.implementer_name;
    }

    pub fn get_interface_name<'a>(&'a self) -> Vec<String> {
        let index = self.interface_name;
        self.index_string_table(index).split("::")
            .map(String::from)
            .collect()
    }
    
    pub fn get_implementer_name<'a>(&'a self) -> Vec<String> {
        let index = self.implementer_name;
        self.index_string_table(index).split("::")
            .map(String::from)
            .collect()
    }

    pub fn add_string<S: AsRef<str>>(&mut self, string: S) -> u64 {
        if string.as_ref() == "U64" {
            panic!("bad class string");
        }
        if let Some(index) = self.string_to_index.get(string.as_ref()) {
            return *index;
        }
        self.string_table.push(StringEntry::new(string.as_ref()));
        let out =self.string_table.len() as u64;

        self.string_to_index.insert(String::from(string.as_ref()), out);
        out
    }

    pub fn add_functions(
        &mut self,
        names: &[impl AsRef<str>],
        signatures: &[SignatureEntry],
    ) {
        for (i, sig) in signatures.iter().enumerate() {
            let name = self.add_string(names[i].as_ref());
            let index = self.signature_table.len() as SignatureIndex;
            self.signature_table.push(sig.clone());
            self.vtable.functions.push(VTableEntry {
                name,
                signature: index,
                bytecode: 0,
            })
        }
    }

    pub fn attach_bytecode<B: AsRef<[u8]>>(
        &mut self,
        method_name: impl AsRef<str>,
        bytecode: B
    ) {
        let index = self.method_to_function.get(method_name.as_ref()).unwrap();
        self.bytecode_table.push(BytecodeEntry::new(bytecode.as_ref()));
        let bytecode_index = self.bytecode_table.len() as BytecodeIndex;
        self.vtable.functions[*index].bytecode = bytecode_index;
    }

}