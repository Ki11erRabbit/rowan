use std::{collections::HashMap, sync::{LazyLock, RwLock}};

use class::{Class, MemberInfo, SignalInfo};
use linker::TableEntry;
use rowan_shared::classfile::{BytecodeEntry, BytecodeIndex, ClassFile, Member, Signal, SignatureIndex, VTableEntry};
use stdlib::{VMClass, VMMember, VMMethod, VMSignal, VMVTable};
use tables::{class_table::ClassTable, object_table::ObjectTable, string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue, VTable, VTables}};


mod tables;
pub mod class;
pub mod object;
pub mod stdlib;
pub mod linker;

pub type Symbol = usize;

pub type Reference = u64;

pub type Index = usize;

pub type VTableIndex = usize;


static VTABLES: LazyLock<RwLock<VTables>> = LazyLock::new(|| {
    let table = VTables::new();
    RwLock::new(table)
});

static CLASS_TABLE: LazyLock<RwLock<ClassTable>> = LazyLock::new(|| {
    let table = ClassTable::new();
    RwLock::new(table)
});

static STRING_TABLE: LazyLock<RwLock<StringTable>> = LazyLock::new(|| {
    let table = StringTable::new();
    RwLock::new(table)
});

static SYMBOL_TABLE: LazyLock<RwLock<SymbolTable>> = LazyLock::new(|| {
    let table = SymbolTable::new();
    RwLock::new(table)
});

static OBJECT_TABLE: LazyLock<RwLock<ObjectTable>> = LazyLock::new(|| {
    let table = ObjectTable::new();
    RwLock::new(table)
});



pub struct Context {}

impl Context {
    pub const fn new() -> Self {
        Context {}
    }

    pub fn link_classes(
        &self,
        classes: Vec<ClassFile>,
        pre_class_table: &mut Vec<TableEntry<Class>>,
        // The first hashmap is the class symbol which the vtable comes from.
        // The second hashmap is the class that has a custom version of the vtable
        // For example, two matching symbols means that that is the vtable of that particular class
        vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Option<Symbol>, Vec<rowan_shared::TypeTag>, Vec<u8>, FunctionValue)>>>,
        string_map: &mut HashMap<String, Symbol>,
        class_map: &mut HashMap<String, Symbol>
    ) {
        let Ok(mut string_table) = STRING_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut symbol_table) = SYMBOL_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut vtable_tables) = VTABLES.write() else {
            panic!("Lock poisoned");
        };

        linker::link_class_files(
            classes,
            &mut symbol_table,
            pre_class_table,
            &mut string_table,
            &mut vtable_tables,
            vtables_map,
            string_map,
            class_map,
            ).unwrap();

        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
    }


    pub fn link_vm_classes(
        &self,
        classes: Vec<VMClass>,
        pre_class_table: &mut Vec<TableEntry<Class>>,
        // The first hashmap is the class symbol which the vtable comes from.
        // The second hashmap is the class that has a custom version of the vtable
        // For example, two matching symbols means that that is the vtable of that particular class
        vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Option<Symbol>, Vec<rowan_shared::TypeTag>, Vec<u8>, FunctionValue)>>>,
        string_map: &mut HashMap<String, Symbol>,
        class_map: &mut HashMap<String, Symbol>
    ) {

    }
}
