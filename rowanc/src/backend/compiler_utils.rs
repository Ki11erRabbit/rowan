use std::collections::HashMap;

use rowan_shared::{classfile::{BytecodeEntry, ClassFile, Member, Signal, SignatureEntry, SignatureIndex, StringEntry, StringIndex, VTable, VTableEntry}, TypeTag};

use crate::ast::Method;



pub type VarLocation = u8;
pub struct Frame {
    bindings: HashMap<String, VarLocation>,
    current_location: u8,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            bindings: HashMap::new(),
            current_location: 0
        }
    }

    pub fn new_with_location(location: u8) -> Self {
        Frame {
            bindings: HashMap::new(),
            current_location: location,
        }
    }

    pub fn add_binding(&mut self, name: impl AsRef<str>) -> u8 {
        let output = self.current_location;
        self.bindings.insert(String::from(name.as_ref()), self.current_location);
        self.current_location += 1;
        output
    }

    pub fn set_binding(&mut self, name: impl AsRef<str>, location: u8) {
        self.bindings.insert(String::from(name.as_ref()), location);
    }

    pub fn is_bound(&self, name: impl AsRef<str>) -> bool {
        self.bindings.contains_key(name.as_ref())
    }

    pub fn get_binding(&self, name: impl AsRef<str>) -> Option<VarLocation> {
        self.bindings.get(name.as_ref()).map(|l| *l)
    }

    pub fn get_location(&self) -> u8 {
        self.current_location
    }
}

#[derive(Debug)]
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
    method_to_class: HashMap<String, String>,
    dont_print: bool,
}

impl PartialClass {

    pub fn index_string_table(&self, index: StringIndex) -> &str {
        std::str::from_utf8(&self.string_table[(index - 1) as usize].value).unwrap()
    }

    pub fn index_bytecode_table(&self, index: StringIndex) -> &BytecodeEntry {
        &self.bytecode_table[(index - 1) as usize]
    }

    pub fn get_method_entry(&self, method_name: impl AsRef<str>) -> Option<VTableEntry> {
        println!("{}", method_name.as_ref());
        let class_name = self.method_to_class.get(method_name.as_ref())?;
        let vtable_index = self.class_to_vtable.get(class_name)?;
        let vtable_indices = self.method_to_function.get(method_name.as_ref())?;

        for (vtable, method) in vtable_indices {
            if vtable_index == vtable {
                return Some(self.vtables[*vtable_index].functions[*method])
            }
        }
        None
    }

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
            method_to_class: HashMap::new(),
            dont_print: false,
        }
    }

    pub fn make_not_printable(&mut self) {
        self.dont_print = true
    }

    pub fn create_class_file(self) -> Option<ClassFile> {
        if self.dont_print {
            return None;
        }
        Some(ClassFile::new_from_parts(
            self.name,
            self.parents,
            self.vtables,
            self.members,
            self.signals,
            self.bytecode_table,
            self.string_table,
            self.signature_table))
    }

    pub fn set_name(&mut self, name: &str) {
        self.string_table.push(StringEntry::new(name));
        self.name = self.string_table.len() as u64;
    }

    pub fn add_parent(&mut self, name: &str) {
        self.string_table.push(StringEntry::new(name));
        self.parents.push(self.string_table.len() as u64);
    }

    pub fn add_vtable(
        &mut self,
        class_name: impl AsRef<str>,
        mut vtable: VTable,
        class_names: Vec<impl AsRef<str>>,
        sub_class_names: Vec<impl AsRef<str>>,
        names: Vec<impl AsRef<str>>,
        responds_to: Vec<impl AsRef<str>>,
        signatures: Vec<SignatureEntry>,

    ) {

        for (i, function) in vtable.functions.iter_mut().enumerate() {
            self.string_table.push(StringEntry::new(class_names[i].as_ref()));
            function.class_name = self.string_table.len() as u64;
            self.string_table.push(StringEntry::new(sub_class_names[i].as_ref()));
            function.sub_class_name = self.string_table.len() as u64;
            self.string_table.push(StringEntry::new(names[i].as_ref()));
            function.name = self.string_table.len() as u64;
            if responds_to[i].as_ref() != "" {
                self.string_table.push(StringEntry::new(responds_to[i].as_ref()));
                function.responds_to = self.string_table.len() as u64;
            }
            function.signature = self.signature_table.len() as u64;
            self.signature_table.push(signatures[i].clone());

            self.method_to_class.insert(String::from(names[i].as_ref()), String::from(class_name.as_ref()));
            self.method_to_function.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push((self.vtables.len(), i)))
                .or_insert(vec![(self.vtables.len(), i)]);
        }
        self.class_to_vtable.insert(class_name.as_ref().to_string(), self.vtables.len());
        self.vtables.push(vtable);
    }

    pub fn add_member<S: AsRef<str>>(&mut self, mut member: Member, name: S) {
        self.string_table.push(StringEntry::new(name.as_ref()));
        member.name = self.string_table.len() as u64;

        self.members.push(member);
    }

    pub fn add_signal<S: AsRef<str>>(&mut self, mut signal: Signal, name: S, sig: SignatureEntry) {
        self.string_table.push(StringEntry::new(name.as_ref()));
        signal.name = self.string_table.len() as u64;

        signal.signature = self.signature_table.len() as u64;
        self.signature_table.push(sig);
    }

    pub fn attach_bytecode<B: AsRef<[u8]>>(
        &mut self,
        class_name: impl AsRef<str>,
        method_name: impl AsRef<str>,
        code: B,
    ) {
        let vtable_index = self.class_to_vtable.get(class_name.as_ref()).unwrap();
        let mut method_index = None;
        for (vtable, method) in self.method_to_function.get(method_name.as_ref()).unwrap() {
            if vtable_index == vtable {
                method_index = Some(*method);
                break;
            }
        }

        let method_index = method_index.unwrap();

        self.bytecode_table.push(BytecodeEntry::new(code.as_ref()));
        let bytecode_index = self.bytecode_table.len();

        self.vtables[*vtable_index].functions[method_index].bytecode = bytecode_index as u64;
    }

    pub fn get_vtable<S: AsRef<str>>(&self, class_name: S) -> (
        VTable,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<String>,
        Vec<SignatureEntry>) {

        let vtable_index = self.class_to_vtable.get(class_name.as_ref()).unwrap();

        let mut vtable = self.vtables[*vtable_index].clone();

        let mut class_names = Vec::new();
        let mut sub_class_names = Vec::new();
        let mut names = Vec::new();
        let mut responds_to = Vec::new();
        let mut signatures = Vec::new();

        for function in vtable.functions.iter_mut() {
            class_names.push(String::from(self.index_string_table(function.class_name)));
            sub_class_names.push(String::from(self.index_string_table(function.sub_class_name)));
            names.push(String::from(self.index_string_table(function.name)));
            if function.responds_to != 0 {
                responds_to.push(String::from(self.index_string_table(function.responds_to)));
            } else {
                responds_to.push(String::from(""));
            }
            signatures.push(self.signature_table[function.signature as usize].clone());
            
            function.bytecode = 0;
            function.class_name = 0;
            function.sub_class_name = 0;
            function.name = 0;
            function.responds_to = 0;
            function.signature = 0;
            
            
        }
        
        (vtable, class_names, sub_class_names, names, responds_to, signatures)
    }

    pub fn add_string<S: AsRef<str>>(&mut self, string: S) -> u64 {
        self.string_table.push(StringEntry::new(string.as_ref()));
        self.string_table.len() as u64
            
    }
}
