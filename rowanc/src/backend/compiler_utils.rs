pub mod partial_class;
mod partial_interface;

use std::collections::HashMap;
use std::io::BufRead;
use crate::backend::compiler_utils::partial_class::PartialClass;

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

