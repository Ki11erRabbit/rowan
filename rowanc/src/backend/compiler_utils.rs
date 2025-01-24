use std::collections::HashMap;

use rowan_shared::{classfile::{BytecodeEntry, ClassFile, Member, Signal, SignatureEntry, SignatureIndex, StringEntry, StringIndex, VTable, VTableEntry}, TypeTag};


pub type VarLocation = u8;
pub struct Frame {
    bindings: HashMap<String, VarLocation>,
    current_location: u8,
}



pub struct ClassInfo {
    class: ClassFile,
    method_names: HashMap<String, Vec<(String, String)>>,
    method_position: HashMap<(String, String), (usize, usize)>,
    member_position: HashMap<String, usize>,
}

impl ClassInfo {
    pub fn new(class: ClassFile) -> ClassInfo {
        let mut method_names = HashMap::new();
        let mut method_position = HashMap::new();
        let mut member_position = HashMap::new();

        for (i, table) in class.vtables.iter().enumerate() {
            for (j, entry) in table.functions.iter().enumerate() {
                let VTableEntry {
                    sub_class_name,
                    name,
                    ..
                } = entry;

                let name = class.index_string_table(*name).to_string();
                let sub_class_name = class.index_string_table(*sub_class_name).to_string();
                method_names.entry(name.clone())
                    .and_modify(|vec: &mut Vec<(String, String)>| vec.push((name.clone(), sub_class_name.clone())))
                    .or_insert(vec![(name.clone(), sub_class_name.clone())]);
                method_position.insert((name, sub_class_name), (i, j));
            }
        }

        for (i, member) in class.members.iter().enumerate() {
            let Member {
                name,
                ..
            } = member;
            let name = class.index_string_table(*name).to_string();
            member_position.insert(name, i);
        }

        ClassInfo {
            class,
            method_names,
            method_position,
            member_position
        }
    }

    pub fn get_method_table(&self, method_name: &str) -> Option<&Vec<(String, String)>> {
        self.method_names.get(method_name)
    }

    pub fn get_method_positions(&self, name: &(String, String)) -> Option<&(usize, usize)> {
        self.method_position.get(name)
    }

    pub fn get_method(&self, (vtable, entry): &(usize, usize)) -> Option<&VTableEntry> {
        if *vtable >= self.class.vtables.len() {
            return None;
        }
        if *entry >= self.class.vtables[*vtable].functions.len() {
            return None;
        }

        Some(&self.class.vtables[*vtable].functions[*entry])
    }

    pub fn get_parents(&self) -> Vec<String> {
        let mut output = Vec::new();
        for parent_index in self.class.parents.iter() {
            output.push(String::from(self.class.index_string_table(*parent_index)))
        }
        output
    }
}

pub struct PartialClass {
    name: StringIndex,
    /// Parent class names
    parents: Vec<StringIndex>,
    /// Virtual tables
    vtables: Vec<VTable>,
    /// Members and their types
    members: Vec<Member>,
    /// Signals and their types
    signals: Vec<Signal>,
    /// Where the bytecode is stored
    /// This table is 1 indexed to allow for methods to be empty
    bytecode_table: Vec<BytecodeEntry>,
    /// String table
    /// This table is 1 indexed to allow for StringIndices 0 value to mean "null"
    string_table: Vec<StringEntry>,
    /// Signature table
    /// This holds the signatures of methods
    signature_table: Vec<SignatureEntry>,
    class_to_vtable: HashMap<String, usize>,
    method_to_function: HashMap<String, Vec<(usize, usize)>>,
}

impl PartialClass {
    pub fn new() -> PartialClass {
        PartialClass {
            name: 0,
            parents: Vec::new(),
            vtables: Vec::new(),
            members: Vec::new(),
            signals: Vec::new(),
            bytecode_table: Vec::new(),
            string_table: Vec::new(),
            signature_table: Vec::new(),
            class_to_vtable: HashMap::new(),
            method_to_function: HashMap::new(),
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.string_table.push(StringEntry::new(name));
        self.name = self.string_table.len() as u64;
    }

    pub fn add_parent(&mut self, name: &str) {
        self.string_table.push(StringEntry::new(name));
        self.parents.push(self.string_table.len() as u64);
    }

    pub fn add_vtable<S: AsRef<str>>(
        &mut self,
        class_name: S,
        mut vtable: VTable,
        class_names: Vec<S>,
        sub_class_names: Vec<S>,
        names: Vec<S>,
        responds_to: Vec<S>,
        signatures: Vec<SignatureEntry>,

    ) {

        for (i, function) in vtable.functions.iter_mut().enumerate() {
            self.string_table.push(StringEntry::new(class_names[i].as_ref()));
            function.class_name = self.string_table.len() as u64;
            self.string_table.push(StringEntry::new(sub_class_names[i].as_ref()));
            function.sub_class_name = self.string_table.len() as u64;
        }

        

        self.class_to_vtable.insert(class_name.as_ref().to_string(), self.vtables.len());
        self.vtables.push(vtable);
        
    }
}
