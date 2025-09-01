pub mod partial_class;
pub mod partial_interface;
pub mod partial_interface_impl;

use std::collections::HashMap;
use std::io::BufRead;
use rowan_shared::classfile::{StringIndex, VTable, VTableEntry};
use crate::backend::compiler_utils::partial_class::PartialClass;
use crate::backend::compiler_utils::partial_interface::PartialInterface;
use crate::backend::compiler_utils::partial_interface_impl::PartialInterfaceImpl;

pub struct ClassMap {
    class_table: Vec<PartialClass>,
    aliases: HashMap<Vec<String>, usize>,
}

impl ClassMap {
    pub fn new() -> Self {
        ClassMap {
            class_table: Vec::new(),
            aliases: HashMap::new(),
        }
    }
    
    pub fn get(&self, name: &Vec<String>) -> Option<&PartialClass> {
        let index = self.aliases.get(name)?;
        self.class_table.get(*index)
    }
    
    pub fn get_mut(&mut self, name: &Vec<String>) -> Option<&mut PartialClass> {
        let index = self.aliases.get(name)?;
        self.class_table.get_mut(*index)
    }
    
    pub fn insert(&mut self, name: Vec<String>, class: PartialClass) -> usize {
        let index = self.class_table.len();
        self.class_table.push(class);
        self.aliases.insert(name, index);
        index
    }
    
    pub fn add_alias(&mut self, name: Vec<String>, index: usize) {
        self.aliases.insert(name, index);
    }

    pub fn contains_key(&self, path: &Vec<String>) -> bool {
        self.aliases.contains_key(path)
    }

    pub fn display_classes(&self) {
        println!("Classmap {{");
        for key in self.aliases.keys() {
            println!("\t{}", key.join("::"))
        }
        println!("}}");
    }
}

impl IntoIterator for ClassMap {
    type Item = (Vec<String>, PartialClass);
    type IntoIter = std::collections::hash_map::IntoIter<Vec<String>, PartialClass>;
    fn into_iter(mut self) -> Self::IntoIter {
        let mut values_to_keys = HashMap::new();
        for (key, value) in self.aliases.iter() {
            values_to_keys.entry(*value)
                .and_modify(|v: &mut Vec<&Vec<String>>| v.push(key))
                .or_insert(vec![key]);
        }
        let mut classes = self.class_table.drain(..)
            .map(Some)
            .collect::<Vec<_>>();
        let mut output: HashMap<Vec<String>, PartialClass> = HashMap::new();
        for (key, values) in values_to_keys.into_iter() {
            let mut longest_value: Option<&Vec<String>> = None;
            for value in values.into_iter() {
                if let Some(the_longest_value) = longest_value {
                    if value.len() > the_longest_value.len() {
                        longest_value = Some(value);
                    }
                } else {
                    longest_value = Some(value);
                }
            }
            output.insert(longest_value.cloned().unwrap(), classes[key].take().unwrap());
        }
        output.into_iter()
    }
}

#[derive(Debug)]
pub enum PartialClassError {
    Ambiguity,
    ClassNotNotFound(String),
    VTableNotNotFound(String),
    MethodNotNotFound(String),
}

pub type PartialClassResult<T> = Result<T, PartialClassError>;

pub type VarLocation = u8;

#[derive(Clone)]
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



pub enum CurrentCompilationUnit<'a> {
    Class(&'a mut PartialClass),
    Interface(&'a mut PartialInterface),
    InterfaceImpl(&'a mut PartialInterfaceImpl),
}

impl<'a> CurrentCompilationUnit<'a> {
    pub fn get_class_name(&self) -> Vec<String> {
        match self {
            CurrentCompilationUnit::Class(class) => class.get_class_name(),
            CurrentCompilationUnit::Interface(interface) => interface.get_interface_name(),
            CurrentCompilationUnit::InterfaceImpl(r#impl) => r#impl.get_interface_name(),
        }
    }
    
    pub fn add_static_method<B: AsRef<[u8]>>(
        &mut self, 
        name: impl AsRef<str>,
        bytecode: B,
        is_native: bool,
        
    ) {
        match self {
            CurrentCompilationUnit::Class(class) => {
                class.add_static_method(name.as_ref(), bytecode.as_ref(), is_native);
            }
            CurrentCompilationUnit::Interface(_) | CurrentCompilationUnit::InterfaceImpl(_) => {
                unreachable!("Interfaces and InterfaceImpls do not support static methods")
            }
        }
    } 
    
    pub fn get_vtable(&self, method_name: impl AsRef<str>) -> Result<&VTable, PartialClassError> {
        match self {
            CurrentCompilationUnit::Class(class) => {
                class.get_vtable(method_name.as_ref())
            }
            CurrentCompilationUnit::Interface(interface) => {
                Ok(interface.get_vtable())
            }
            CurrentCompilationUnit::InterfaceImpl(r#impl) => {
                Ok(r#impl.get_vtable())
            }
        }
    }
    
    pub fn index_string_table(&self, index: StringIndex) -> &str {
        match self {
            CurrentCompilationUnit::Class(class) => {
                class.index_string_table(index)
            }
            CurrentCompilationUnit::Interface(interface) => {
                interface.index_string_table(index)
            }
            CurrentCompilationUnit::InterfaceImpl(r#impl) => {
                r#impl.index_string_table(index)
            }
        }
    }
    
    pub fn attach_bytecode(
        &mut self, 
        method_class_name: &Vec<String>,
        method_name: impl AsRef<str>,
        bytecode: impl AsRef<[u8]>,
        is_native: bool,
    ) -> PartialClassResult<()> {
        match self {
            CurrentCompilationUnit::Class(class) => {
                class.attach_bytecode(method_class_name, method_name, bytecode, is_native)
            }
            CurrentCompilationUnit::Interface(interface) => {
                Ok(interface.attach_bytecode(method_name.as_ref(), bytecode.as_ref()))
            }
            CurrentCompilationUnit::InterfaceImpl(r#impl) => {
                Ok(r#impl.attach_bytecode(method_name.as_ref(), bytecode.as_ref()))
            }
        }
    }
    
    pub fn add_string(&mut self, string: impl AsRef<str>) -> StringIndex {
        match self {
            CurrentCompilationUnit::Class(class) => {
                class.add_string(string)
            }
            CurrentCompilationUnit::Interface(interface) => {
                interface.add_string(string)
            }
            CurrentCompilationUnit::InterfaceImpl(r#impl) => {
                r#impl.add_string(string)
            }
        }
    }
    
    pub fn get_method_entry(&self, method_name: impl AsRef<str>) -> PartialClassResult<VTableEntry> {
        match self {
            CurrentCompilationUnit::Class(class) => {
                class.get_method_entry(method_name.as_ref())
            }
            CurrentCompilationUnit::Interface(interface) => {
                interface.get_method_entry(method_name.as_ref())
                    .ok_or(PartialClassError::MethodNotNotFound(method_name.as_ref().to_string()))
            }
            CurrentCompilationUnit::InterfaceImpl(r#impl) => {
                r#impl.get_method_entry(method_name.as_ref())
                    .ok_or(PartialClassError::MethodNotNotFound(method_name.as_ref().to_string()))
            }
        }
    }
}
