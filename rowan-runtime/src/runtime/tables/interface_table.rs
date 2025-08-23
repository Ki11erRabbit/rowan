use crate::runtime::interface::{Interface, InterfaceImpl};

pub struct InterfaceTable {
    interfaces: Vec<Interface>,
}

impl InterfaceTable {
    pub fn new() -> Self {
        InterfaceTable {
            interfaces: Vec::new(),
        }
    }
    
    pub fn insert_interface(&mut self, interface: Interface) -> usize {
        let index = self.interfaces.len();
        self.interfaces.push(interface);
        index
    }
}

impl std::ops::Index<usize> for InterfaceTable {
    type Output = Interface;
    fn index(&self, index: usize) -> &Self::Output {
        &self.interfaces[index]
    }
}

impl std::ops::IndexMut<usize> for InterfaceTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.interfaces[index]
    }
}