use crate::runtime::{Symbol, VTableIndex};

pub struct Interface {
    pub name: Symbol,
    pub vtable: VTableIndex,
}

impl Interface {
    pub fn new(name: Symbol, vtable: VTableIndex) -> Self {
        Self { name, vtable }
    }
}

pub struct InterfaceImpl {
    pub name: Symbol,
    pub vtable: VTableIndex,
}

impl InterfaceImpl {
    pub fn new(name: Symbol, vtable: VTableIndex) -> Self {
        Self { name, vtable }
    }
}
