use std::{collections::HashMap, sync::{LazyLock, RwLock}};
use std::sync::Arc;

use class::{Class, MemberInfo, SignalInfo};
use cranelift_jit::{JITModule, JITBuilder};
use cranelift::{codegen, prelude::Signature};
use cranelift::prelude::Configurable;
use jit::{JITCompiler, JITController};
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

static JIT_CONTROLLER: LazyLock<RwLock<JITController>> = LazyLock::new(|| {
    let jit_controller = JITController::default();
    RwLock::new(jit_controller)
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
        vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Option<Symbol>, Vec<rowan_shared::TypeTag>, Vec<u8>, Arc<RwLock<FunctionValue>>)>>>,
        string_map: &mut HashMap<String, Symbol>,
        class_map: &mut HashMap<String, Symbol>
    ) -> (Symbol, Symbol) {
        let Ok(mut string_table) = STRING_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut symbol_table) = SYMBOL_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut vtable_tables) = VTABLES.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
            panic!("Lock poisoned");
        };

        linker::link_class_files(
            classes,
            &mut jit_controller,
            &mut symbol_table,
            pre_class_table,
            &mut string_table,
            &mut vtable_tables,
            vtables_map,
            string_map,
            class_map,
            ).unwrap()
    }


    pub fn link_vm_classes(
        &self,
        classes: Vec<VMClass>,
        pre_class_table: &mut Vec<TableEntry<Class>>,
        // The first hashmap is the class symbol which the vtable comes from.
        // The second hashmap is the class that has a custom version of the vtable
        // For example, two matching symbols means that that is the vtable of that particular class
        vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Option<Symbol>, Vec<rowan_shared::TypeTag>, Vec<u8>, Arc<RwLock<FunctionValue>>)>>>,
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
        let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
            panic!("Lock poisoned");
        };

        linker::link_vm_classes(
            classes,
            &mut jit_controller,
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

        let value = function.value.read().expect("Lock poisoned");
        match &*value {
            FunctionValue::Builtin(ptr, _) => *ptr,
            FunctionValue::Compiled(ptr, _) => *ptr,
            _ => {
                drop(value);
                let mut compiler = self.create_jit_compiler();
                let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
                    panic!("Lock poisoned");
                };
                jit_controller.create_test_function();
                
                compiler.compile(&function, &mut jit_controller.module).unwrap();

                let value = function.value.read().expect("Lock poisoned");
                match &*value {
                    FunctionValue::Compiled(ptr, _) => *ptr,
                    _ => panic!("Function wasn't compiled")
                }
            }
        }

    }

    pub fn create_jit_compiler(&self) -> JITCompiler {
        let Ok(jit_controller) = JIT_CONTROLLER.write() else {
            panic!("Lock poisoned");
        };
        let context = jit_controller.new_context();

        JITCompiler::new(context)
    }

    pub fn get_method_signature(&self, class_symbol: Symbol, method_name: Symbol) -> Signature {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(object_class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };

        let class = &class_table[object_class_index];
        let vtable_index = class.get_vtable(&(class_symbol, None));
        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name);
        let value = function.value.read().expect("Lock poisoned");
        match &*value {
            FunctionValue::Builtin(_, signature) => signature.clone(),
            FunctionValue::Compiled(_, signature) => signature.clone(),
            FunctionValue::Bytecode(_, _, signature) => signature.clone(),
            _ => panic!("Method not compiled yet"),
        }
    }

    pub fn new_object(&self, class_symbol: Symbol) -> Reference {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(class_ref) = symbol_table[class_symbol] else {
            println!("{:?}", symbol_table[class_symbol]);
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_ref];
        let parent_objects = &class.parents;

        let mut parents = Vec::new();
        
        for parent in parent_objects.iter() {
            parents.push(self.new_object(*parent));
        }

        let data_size = class.get_member_size();

        let object = Object::new(class_symbol, parents.into_boxed_slice(), data_size);
        
        let Ok(mut object_table) = OBJECT_TABLE.write() else {
            panic!("Lock poisoned");
        };

        let reference = object_table.add(object);
        
        reference
    }

    pub fn get_class_name(&self, class_symbol: Symbol) -> String {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class symbol wasn't a class");
        }; 
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];
        
        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::StringRef(class_name) = symbol_table[class.name] else {
            panic!("class name wasn't a string");
        }; 
        String::from(&string_table[class_name])
    }

    pub fn get_class(&self, class_symbol: Symbol) -> *const Class {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class symbol wasn't a class");
        }; 
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        &class_table[class_index]
    }

    pub fn get_string(&self, string_symbol: Symbol) -> &'static str {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::StringRef(string_index) = symbol_table[string_symbol] else {
            panic!("string symbol wasn't a string");
        }; 
        string_table.get_string(string_index)
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

pub extern "C" fn new_object(class_symbol: u64) -> u64 {
    let context = Context::new();
    let class_symbol = class_symbol as Symbol;
    let object = context.new_object(class_symbol);
    object as usize as u64
}
