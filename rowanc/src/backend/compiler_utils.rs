use std::collections::HashMap;
use std::io::BufRead;
use rowan_shared::{classfile::{BytecodeEntry, ClassFile, Member, SignatureEntry, SignatureIndex, StringEntry, StringIndex, VTable, VTableEntry}, TypeTag};
use rowan_shared::classfile::{BytecodeIndex, StaticMethods};
use crate::backend::Compiler;
use crate::native::NativeAttributes;

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

/// Represents a member of a class
#[derive(PartialEq, Debug)]
pub struct StaticMember {
    pub name: StringIndex,
    pub is_const: bool,
    pub type_tag: TypeTag,
}

impl StaticMember {
    pub fn new(is_const: bool, type_tag: TypeTag) -> Self {
        StaticMember {
            name: 0,
            is_const,
            type_tag
        }
    }
}

impl Into<Member> for StaticMember {
    fn into(self) -> Member {
        Member {
            name: self.name,
            type_tag: self.type_tag,
        }
    }
}

#[derive(Debug)]
pub struct PartialClass {
    name: StringIndex,
    /// Parent class names
    parent: StringIndex,
    /// Virtual tables
    vtables: Vec<VTable>,
    /// Members and their types
    members: Vec<Member>,
    /// Static methods and their entry
    static_methods: Vec<VTableEntry>,
    /// Static members
    static_members: Vec<StaticMember>,
    /// Static member initialization function.
    /// This can be null
    static_init: BytecodeIndex,
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
    /// If there are more than one index, then that means that we have two different versions of the same vtable
    class_to_vtable: HashMap<Vec<String>, Vec<usize>>,
    /// This maps a vtable index to a string with the name of the class the vtable comes from.
    /// The origin class should always be a parent of the current object
    vtable_to_class: HashMap<usize, Vec<String>>,
    /// Maps method names to a vec of indices
    /// The first index is the index into the vtable table
    /// The second index is the index in the vtable itself
    method_to_function: HashMap<String, Vec<(usize, usize)>>,
    /// Maps method names to a vec of class names
    /// This is so that we can reference the class a method is coming from
    method_to_class: HashMap<String, Vec<Vec<String>>>,
    /// Maps static method names to a method signature
    static_method_to_signature: HashMap<String, SignatureIndex>,
    /// Names of functions that get the size of a native member
    native_member_sizes: Vec<String>,
    /// Names and types of native defined methods,
    native_functions: Vec<(String, Vec<TypeTag>, TypeTag)>,
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
        //println!("{}: {:#?}", method_name.as_ref(), self.method_to_class);
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
            parent: 0,
            vtables: Vec::new(),
            members: Vec::new(),
            static_methods: Vec::new(),
            static_members: Vec::new(),
            static_init: 0,
            bytecode_table: Vec::new(),
            string_table: Vec::new(),
            string_to_index: HashMap::new(),
            signature_table: Vec::new(),
            class_to_vtable: HashMap::new(),
            method_to_function: HashMap::new(),
            method_to_class: HashMap::new(),
            vtable_to_class: HashMap::new(),
            static_method_to_signature: HashMap::new(),
            native_member_sizes: Vec::new(),
            native_functions: Vec::new(),
            dont_print: false,
        }
    }

    pub fn make_not_printable(&mut self) {
        self.dont_print = true
    }

    pub fn create_class_file(self) -> Option<(ClassFile, NativeAttributes)> {
        if self.dont_print {
            return None;
        }
        let class_name = self.get_class_name().join("::");
        Some((ClassFile::new_from_parts(
            self.name,
            self.parent,
            self.vtables,
            self.members,
            StaticMethods::new(self.static_methods),
            self.static_members.into_iter().map(Into::<Member>::into).collect(),
            self.static_init,
            self.bytecode_table,
            self.string_table,
            self.signature_table),
            NativeAttributes::new(class_name, self.native_member_sizes, self.native_functions),
        ))
    }
    
    pub fn add_signatures(&mut self, sigs: Vec<SignatureEntry>) {
        self.signature_table.extend(sigs);
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = self.add_string(name);
    }

    pub fn set_static_method_to_sig(&mut self, map: HashMap<String, SignatureIndex>) {
        self.static_method_to_signature = map;
    }

    pub fn set_parent(&mut self, name: &str) {
        let index = self.add_string(name);
        self.parent = index;
    }

    pub fn add_vtable(
        &mut self,
        class_name: &Vec<String>,
        mut vtable: VTable,
        names: &Vec<impl AsRef<str>>,
        signatures: &Vec<SignatureEntry>,
    ) {
        //println!("add_vtable1 {} {}", self.index_string_table(self.name), class_name.as_ref());
        vtable.class_name = self.add_string(class_name.join("::"));
        for (i, function) in vtable.functions.iter_mut().enumerate() {
            function.name = self.add_string(names[i].as_ref());
            function.signature = self.signature_table.len() as u64;
            self.signature_table.push(signatures[i].clone());

            self.method_to_class.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push(class_name.clone()))
                .or_insert(vec![class_name.clone()]);

            self.method_to_function.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push((self.vtables.len(), i)))
                .or_insert(vec![(self.vtables.len(), i)]);
        }
        self.class_to_vtable.entry(class_name.clone())
            .and_modify(|v| v.push(self.vtables.len()))
            .or_insert(vec![self.vtables.len()]);
        self.vtable_to_class.insert(self.vtables.len(), class_name.clone());
        self.vtables.push(vtable);
    }

    pub fn add_static_methods(
        &mut self,
        class_name: &Vec<String>,
        mut static_methods: StaticMethods,
        names: &Vec<impl AsRef<str>>,
        signatures: &Vec<SignatureEntry>,
    ) {
        //println!("add_vtable1 {} {}", self.index_string_table(self.name), class_name.as_ref());

        for (i, function) in static_methods.functions.iter_mut().enumerate() {
            function.name = self.add_string(names[i].as_ref());
            function.signature = self.signature_table.len() as u64;
            self.signature_table.push(signatures[i].clone());

            self.method_to_class.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push(class_name.clone()))
                .or_insert(vec![class_name.clone()]);

            self.method_to_function.entry(String::from(names[i].as_ref()))
                .and_modify(|v| v.push((self.vtables.len(), i)))
                .or_insert(vec![(self.vtables.len(), i)]);
        }
        self.class_to_vtable.entry(class_name.clone())
            .and_modify(|v| v.push(self.vtables.len()))
            .or_insert(vec![self.vtables.len()]);
        self.static_methods.extend_from_slice(&static_methods.functions);
    }

    pub fn add_static_method<B: AsRef<[u8]>>(
        &mut self,
        method_name: impl AsRef<str>,
        code: B,
        is_native: bool,
    ) {
        let name_index = self.add_string(method_name.as_ref());
        let signature_index = self.static_method_to_signature.get(method_name.as_ref()).unwrap();

        if is_native {

            let signature: SignatureEntry = self.signature_table[*signature_index as usize].clone();

            self.native_functions.push((
                method_name.as_ref().to_string(),
                signature.types[1..].to_vec(),
                signature.types[0],
            ));

            self.static_methods.push(VTableEntry {
                name: name_index,
                signature: *signature_index,
                bytecode: -1,
            });

            return
        }

        self.bytecode_table.push(BytecodeEntry::new(code.as_ref()));
        let bytecode_index = self.bytecode_table.len() as BytecodeIndex;

        self.static_methods.push(VTableEntry {
            name: name_index,
            signature: *signature_index,
            bytecode: bytecode_index,
        })
    }

    pub fn add_member<S: AsRef<str>>(&mut self, mut member: Member, name: S) {
        member.name = self.add_string(name.as_ref());

        match member.type_tag {
            TypeTag::Native => {
                self.set_field_as_native(name.as_ref())
            }
            _ => {}
        }

        self.members.push(member);
    }

    pub fn add_static_member<S: AsRef<str>>(&mut self, mut member: StaticMember, member_name: S) {
        member.name = self.add_string(member_name);

        self.static_members.push(member);
    }

    pub fn attach_bytecode<B: AsRef<[u8]>>(
        &mut self,
        class_name: &[String],
        method_name: impl AsRef<str>,
        code: B,
        is_native: bool,
    ) -> PartialClassResult<()> {
        //println!("{}", class_name.as_ref());
        //println!("{:#?}", self);
        let vtable_indices = self.class_to_vtable.get(class_name).unwrap();
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

        let signature_index =  self.vtables[vtable_index].functions[method_index].signature;

        if is_native {

            let signature: SignatureEntry = self.signature_table[signature_index as usize].clone();

            self.native_functions.push((
                method_name.as_ref().to_string(),
                signature.types[1..].to_vec(),
                signature.types[0],
            ));

            self.vtables[vtable_index].functions[method_index].bytecode = -1;

            return Ok(());
        }

        self.bytecode_table.push(BytecodeEntry::new(code.as_ref()));
        let bytecode_index = self.bytecode_table.len();

        self.vtables[vtable_index].functions[method_index].bytecode = bytecode_index as u64 as BytecodeIndex;
        Ok(())
    }

    pub fn attach_static_init_bytecode<B: AsRef<[u8]>>(
        &mut self,
        code: B,
    ) -> PartialClassResult<()> {
        self.bytecode_table.push(BytecodeEntry::new(code.as_ref()));
        let bytecode_index = self.bytecode_table.len();

        self.static_init = bytecode_index as u64 as BytecodeIndex;
        Ok(())
    }

    pub fn get_vtables(&self, class_name: &[String]) -> Vec<(
        VTable,
        Vec<String>,
        Vec<SignatureEntry>)> {

        let vtable_indices = self.class_to_vtable.get(class_name).unwrap();

        let mut output = Vec::new();
        for vtable_index in vtable_indices {
            let mut vtable = self.vtables[*vtable_index].clone();

            let mut names = Vec::new();
            let mut signatures = Vec::new();

            for function in vtable.functions.iter_mut() {
                names.push(String::from(self.index_string_table(function.name)));
                signatures.push(self.signature_table[function.signature as usize].clone());

                function.bytecode = 0;
                function.name = 0;
                function.signature = 0;
            }

            output.push((vtable, names, signatures));
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
    
    pub fn get_class_name<'a>(&'a self) -> Vec<String> {
        let index = self.name;
        self.index_string_table(index).split("::")
            .map(String::from)
        .collect()
    }
    
    pub fn contains_field(&self, field: &str) -> bool{
        for member in self.members.iter() {
            if field == self.index_string_table(member.name) {
                return true;
            }
        }
        false
    }

    pub fn set_field_as_native(
        &mut self,
        field: &str,
    ) {
        self.native_member_sizes.push(format!("{}::get-size", field));
    }
    
    pub fn find_class_with_field<'a>(
        &'a self, 
        compiler: &'a Compiler,
        field: &str
    ) -> Option<(Vec<String>, Vec<String>)> {
        let parent = self.index_string_table(self.parent);
        let parent = parent.split("::").map(ToString::to_string).collect::<Vec<String>>();
        let parent_class = compiler.classes.get(&parent).unwrap();

        let Some(class_name) = parent_class.find_class_with_field_helper(compiler, field) else {
            return None;
        };
        Some((class_name, parent))
    }
    
    fn find_class_with_field_helper<'a>(
        &'a self,
        compiler: &'a Compiler,
        field: &str
    ) -> Option<Vec<String>> {
        if self.contains_field(field) {
            let class_name = self.index_string_table(self.name)
                .split("::")
                .map(ToString::to_string)
            .collect::<Vec<String>>();
            return Some(class_name)
        }

        let parent = self.index_string_table(self.parent);
        let parent = parent.split("::").map(ToString::to_string).collect::<Vec<String>>();
        let parent = compiler.classes.get(&parent).unwrap();

        let Some(class_name) = parent.find_class_with_field_helper(compiler, field) else {
            return None;
        };

        Some(class_name)
    }
    
    pub fn get_member_offset(&self, field: &str) -> (u64, TypeTag) {
        for (i, member) in self.members.iter().enumerate() {
            if field == self.index_string_table(member.name) {
                return (i as u64, member.type_tag);
            }
        }
        unreachable!("Can't find member {}", field)
    }

    pub fn get_static_member_offset(&self, field: &str) -> (u64, TypeTag) {
        for (i, member) in self.static_members.iter().enumerate() {
            if field == self.index_string_table(member.name) {
                return (i as u64, member.type_tag);
            }
        }
        unreachable!("Can't find static member {}", field)
    }
}
