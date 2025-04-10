use std::collections::HashMap;

use rowan_shared::{classfile::{BytecodeEntry, ClassFile, Member, Signal, SignatureEntry, SignatureIndex, StringEntry, StringIndex, VTable, VTableEntry}, TypeTag};

use crate::ast::Method;

#[derive(Debug)]
pub enum PartialClassError {
    Ambiguity,
    ClassNotNotFound(String),
    VTableNotNotFound(String),
    MethodNotNotFound(String),
}

pub type PartialClassResult<T> = Result<T, PartialClassError>;

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
    string_to_index: HashMap<String, StringIndex>,
    /// Signature table
    /// This holds the signatures of methods
    signature_table: Vec<SignatureEntry>,
    /// This maps class names to a vtable.
    /// If there are more than one indices, then that means that we have two different versions of the same vtable
    class_to_vtable: HashMap<String, Vec<usize>>,
    /// This maps a vtable index to a string with the name of the class the vtable comes from.
    /// The origin class should always be a parent of the current object
    vtable_to_class: HashMap<usize, String>,
    /// Maps method names to a vec of indices
    /// The first index is the index into the vtable table
    /// The second index is the index in the vtable itself
    method_to_function: HashMap<String, Vec<(usize, usize)>>,
    /// Maps method names to a vec of class names
    /// This is so that we can reference the class a method is coming from
    method_to_class: HashMap<String, Vec<String>>,
    /// A flag to mark the class as one to not emit a class file for
    dont_print: bool,
}

impl PartialClass {

    pub fn index_string_table(&self, index: StringIndex) -> &str {
        //println!("{}", index);
        std::str::from_utf8(&self.string_table[(index - 1) as usize].value).unwrap()
    }

    pub fn index_bytecode_table(&self, index: StringIndex) -> &BytecodeEntry {
        &self.bytecode_table[(index - 1) as usize]
    }

    pub fn get_vtable(&self, method_name: impl AsRef<str>) -> Result<&VTable, PartialClassError> {
        let class_names = self.method_to_class.get(method_name.as_ref()).ok_or(PartialClassError::ClassNotNotFound(method_name.as_ref().to_string()))?;
        if class_names.len() > 1 {
            return Err(PartialClassError::Ambiguity);
        }
        let class_name = &class_names[0];

        let vtable_indices = self.class_to_vtable.get(class_name).ok_or(PartialClassError::VTableNotNotFound(method_name.as_ref().to_string()))?;
        if vtable_indices.len() > 1 {
            return Err(PartialClassError::Ambiguity);
        }
        let vtable_index = vtable_indices[0];
        let vtable_indices = self.method_to_function.get(method_name.as_ref()).ok_or(PartialClassError::MethodNotNotFound(method_name.as_ref().to_string()))?;

        for (vtable, _) in vtable_indices {
            if vtable_index == *vtable {
                return Ok(&self.vtables[vtable_index])
            }
        }
        Err(PartialClassError::VTableNotNotFound(method_name.as_ref().to_string()))
    }

    pub fn get_method_entry(&self, method_name: impl AsRef<str>) -> Result<VTableEntry, PartialClassError> {
        let vtable = self.get_vtable(method_name.as_ref())?;

        let vtable_indices = self.method_to_function.get(method_name.as_ref()).ok_or(PartialClassError::MethodNotNotFound(method_name.as_ref().to_string()))?;

        if vtable_indices.len() > 1 {
            return Err(PartialClassError::Ambiguity);
        }
        let (_, function_index) = vtable_indices[0];
        Ok(vtable.functions[function_index])
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
            string_to_index: HashMap::new(),
            signature_table: Vec::new(),
            class_to_vtable: HashMap::new(),
            method_to_function: HashMap::new(),
            method_to_class: HashMap::new(),
            vtable_to_class: HashMap::new(),
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
        self.name = self.add_string(name);
    }

    pub fn add_parent(&mut self, name: &str) {
        let index = self.add_string(name);
        self.parents.push(index);
    }

    pub fn add_vtable(
        &mut self,
        class_name: impl AsRef<str>,
        mut vtable: VTable,
        names: &Vec<impl AsRef<str>>,
        responds_to: &Vec<impl AsRef<str>>,
        signatures: &Vec<SignatureEntry>,

    ) {
        //println!("add_vtable1 {} {}", self.index_string_table(self.name), class_name.as_ref());
        vtable.class_name = self.add_string(class_name.as_ref());
        for (i, function) in vtable.functions.iter_mut().enumerate() {
            function.name = self.add_string(names[i].as_ref());
            if responds_to.len() > 0 && responds_to[i].as_ref() != "" {
                function.responds_to = self.add_string(responds_to[i].as_ref());
            }
            function.signature = self.signature_table.len() as u64;
            self.signature_table.push(signatures[i].clone());

            self.method_to_class.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push(String::from(class_name.as_ref())))
                .or_insert(vec![String::from(class_name.as_ref())]);
            self.method_to_function.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push((self.vtables.len(), i)))
                .or_insert(vec![(self.vtables.len(), i)]);
        }
        self.class_to_vtable.entry(class_name.as_ref().to_string())
            .and_modify(|v| v.push(self.vtables.len()))
            .or_insert(vec![self.vtables.len()]);
        self.vtable_to_class.insert(self.vtables.len(), class_name.as_ref().to_string());
        self.vtables.push(vtable);
    }

    pub fn add_member<S: AsRef<str>>(&mut self, mut member: Member, name: S) {
        member.name = self.add_string(name);

        self.members.push(member);
    }

    pub fn add_signal<S: AsRef<str>>(&mut self, mut signal: Signal, name: S, sig: SignatureEntry) {
        signal.name = self.add_string(name);

        signal.signature = self.signature_table.len() as u64;
        self.signature_table.push(sig);
    }

    pub fn attach_bytecode<B: AsRef<[u8]>>(
        &mut self,
        class_name: impl AsRef<str>,
        method_name: impl AsRef<str>,
        code: B,
    ) -> PartialClassResult<()> {
        //println!("{}", class_name.as_ref());
        //println!("{:#?}", self);
        let vtable_indices = self.class_to_vtable.get(class_name.as_ref()).unwrap();
        if vtable_indices.len() > 1 {
            return Err(PartialClassError::Ambiguity);
        }
        let vtable_index = vtable_indices[0];
        let mut method_index = None;
        for (vtable, method) in self.method_to_function.get(method_name.as_ref()).unwrap() {
            //println!("{:?}", (vtable, method));
            if vtable_index == *vtable {
                method_index = Some(*method);
                break;
            }
        }
        //println!("{:#?}", self);
        let method_index = method_index.unwrap();

        self.bytecode_table.push(BytecodeEntry::new(code.as_ref()));
        let bytecode_index = self.bytecode_table.len();

        self.vtables[vtable_index].functions[method_index].bytecode = bytecode_index as u64;
        Ok(())
    }

    pub fn get_vtables<S: AsRef<str>>(&self, class_name: S) -> Vec<(
        VTable,
        Vec<String>,
        Vec<String>,
        Vec<SignatureEntry>)> {

        let vtable_indices = self.class_to_vtable.get(class_name.as_ref()).unwrap();

        let mut output = Vec::new();
        for vtable_index in vtable_indices {
            let mut vtable = self.vtables[*vtable_index].clone();

            let mut names = Vec::new();
            let mut responds_to = Vec::new();
            let mut signatures = Vec::new();

            for function in vtable.functions.iter_mut() {
                names.push(String::from(self.index_string_table(function.name)));
                if function.responds_to != 0 {
                    responds_to.push(String::from(self.index_string_table(function.responds_to)));
                } else {
                    responds_to.push(String::from(""));
                }
                signatures.push(self.signature_table[function.signature as usize].clone());

                function.bytecode = 0;
                function.name = 0;
                function.responds_to = 0;
                function.signature = 0;
            }

            output.push((vtable, names, responds_to, signatures));
        }
        output
    }

    pub fn add_string<S: AsRef<str>>(&mut self, string: S) -> u64 {
        if let Some(index) = self.string_to_index.get(string.as_ref()) {
            return *index;
        }
        self.string_table.push(StringEntry::new(string.as_ref()));
        let out =self.string_table.len() as u64;

        self.string_to_index.insert(String::from(string.as_ref()), out);
        out
    }
}
