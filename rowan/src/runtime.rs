use std::{collections::HashMap, sync::{LazyLock, RwLock}};

use class::{Class, MemberInfo, SignalInfo};
use linker::TableEntry;
use object::Object;
use rowan_shared::classfile::{BytecodeEntry, BytecodeIndex, ClassFile, Member, Signal, SignatureIndex, VTableEntry};
use stdlib::{VMClass, VMMember, VMMethod, VMSignal, VMVTable};
use tables::{class_table::ClassTable, object_table::ObjectTable, string_table::StringTable, symbol_table::{self, SymbolEntry, SymbolTable}, vtable::{Function, FunctionValue, VTable, VTables}};


mod tables;
pub mod class;
pub mod object;
pub mod stdlib;
pub mod linker;
pub mod jit;

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
        let Ok(mut string_table) = STRING_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut symbol_table) = SYMBOL_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut vtable_tables) = VTABLES.write() else {
            panic!("Lock poisoned");
        };

        linker::link_vm_classes(
            classes,
            &mut symbol_table,
            pre_class_table,
            &mut string_table,
            &mut vtable_tables,
            vtables_map,
            string_map,
            class_map,
            );
    }

    pub fn finish_linking_classes(
        &self,
        pre_class_table: Vec<TableEntry<Class>>
    ) {
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };

        for class in pre_class_table {
            match class {
                TableEntry::Hole => {
                    panic!("missing class");
                }
                TableEntry::Entry(class) => {
                    class_table.insert_class(class);
                }

            }
        }
    }

    pub fn get_object(&self, reference: Reference) -> *mut Object {
        let Ok(object_table) = OBJECT_TABLE.read() else {
            panic!("Lock poisoned");
        };
        object_table[reference]
    }

    pub fn get_method(
        &self,
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        source_class: Option<Symbol>,
        method_name: Symbol,
    ) -> *const () {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(object_class_index) = symbol_table[object_class_symbol] else {
            panic!("class wasn't a class");
        };

        let class = &class_table[object_class_index];
        let key = (class_symbol, source_class);

        let vtable_index = class.get_vtable(&key);
        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name);
        // TODO: either make sure that this is always compiled/builtin or add code to jit it on the fly
        match function.value {
            FunctionValue::Builtin(ptr) => ptr,
            FunctionValue::Compiled(ptr) => ptr,
            _ => panic!("Method not compiled yet"),
        }

    }
}



pub extern "C" fn get_virtual_function(object: Reference, class_symbol: u64, source_class: i64, method_name: u64) -> u64 {
    let context = Context::new();
    let object = context.get_object(object);
    let object = unsafe {object.as_mut().unwrap()};

    let object_class_symbol = object.class;
    let class_symbol = class_symbol as Symbol;
    let source_class = if source_class < 0 {
        None
    } else {
        Some(source_class as Symbol)
    };
    let method_name = method_name as Symbol;
    let method_ptr = context.get_method(object_class_symbol, class_symbol, source_class, method_name);

    method_ptr as usize as u64
}
