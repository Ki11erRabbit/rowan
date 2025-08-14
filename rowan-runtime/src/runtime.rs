use std::{collections::HashMap, sync::LazyLock};
use std::sync::{Arc, RwLock};

use class::Class;

use cranelift::prelude::Signature;
use jit::{JITCompiler, JITController};
use linker::TableEntry;
use object::Object;
use rowan_shared::classfile::ClassFile;
use core::VMClass;
use tables::{class_table::ClassTable, object_table::ObjectTable, string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{FunctionValue, VTables}};
use std::borrow::{BorrowMut};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32};
use fxhash::FxHashMap;
use rowan_shared::bytecode::linked::Bytecode;
use crate::context::{BytecodeContext, MethodName, WrappedReference};
use crate::fake_lock::FakeLock;
use crate::runtime::class::{ClassMember, ClassMemberData};

mod tables;
pub mod class;
pub mod object;
pub mod core;
pub mod linker;
pub mod jit;
pub mod garbage_collection;
pub use tables::FunctionDetails;
use crate::runtime::core::StringBuffer;
use crate::runtime::tables::native_object_table::NativeObjectTable;

pub type Symbol = usize;

pub type Reference = *mut Object;




pub type Index = usize;

pub type VTableIndex = usize;


pub static DO_GARBAGE_COLLECTION: LazyLock<Arc<RwLock<()>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(()))
});


pub static THREAD_COUNT: LazyLock<FakeLock<AtomicU32>> = LazyLock::new(|| {
    FakeLock::new(AtomicU32::new(1))
});

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

static CLASS_MAPPER: LazyLock<RwLock<HashMap<String, Symbol>>> = LazyLock::new(|| {
    let map = HashMap::new();
    RwLock::new(map)
});

static LIBRARY_TABLE: LazyLock<RwLock<NativeObjectTable>> = LazyLock::new(|| {
    let table = NativeObjectTable::new();
    RwLock::new(table)
});

static STRING_MAP: LazyLock<RwLock<HashMap<String, Symbol>>> = LazyLock::new(|| {
    let map = HashMap::new();
    RwLock::new(map)
});

pub trait StaticMemberAccess<T>: Sized + Default {
    fn make_self() -> Self {
        Default::default()
    }
    fn get_static_member(&self, class: Symbol, index: usize) -> Option<T>;
    fn set_static_member(&self, class: Symbol, index: usize, value: T) -> Option<()>;
}


impl StaticMemberAccess<u8> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<u8> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Byte(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: u8) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Byte(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

impl StaticMemberAccess<u16> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<u16> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Short(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: u16) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Short(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

impl StaticMemberAccess<u32> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<u32> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Int(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: u32) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Int(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

impl StaticMemberAccess<u64> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<u64> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Long(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: u64) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Long(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

impl StaticMemberAccess<Reference> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<Reference> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Object(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: Reference) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Object(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

impl StaticMemberAccess<f32> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<f32> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Float(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: f32) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Float(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

impl StaticMemberAccess<f64> for RuntimeHelper {
    fn get_static_member(&self, class_symbol: Symbol, index: usize) -> Option<f64> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_index];

        match class.get_member(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Double(v), .. } ) => Some(*v),
            _ => None
        }
    }

    fn set_static_member(&self, class_symbol: Symbol, index: usize, value: f64) -> Option<()> {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let class = &mut class_table[class_index];

        match class.get_member_mut(index as usize) {
            Some(ClassMember {  data: ClassMemberData::Double(v), .. } ) => {
                *v = value;
                Some(())
            },
            _ => None
        }
    }
}

pub trait MakeObject<N> {
    fn make_self() -> Self;
    fn new_object(&self, class_name: N) -> Reference;
}

#[derive(Default)]
pub struct RuntimeHelper;

impl MakeObject<Symbol> for RuntimeHelper {

    #[inline]
    fn make_self() -> Self {
        RuntimeHelper
    }

    #[inline]
    fn new_object(&self, class_symbol: Symbol) -> Reference {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(class_ref) = symbol_table[class_symbol] else {
            panic!("class wasn't a class {class_symbol}");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_ref];
        let parent_object = if class.parent != 0 {
            Runtime::new_object(class.parent)
        } else {
            std::ptr::null_mut()
        };

        let data_size = class.get_member_size();
        let object = Object::new(class_symbol, parent_object, data_size, class.drop_function);

        let Ok(mut object_table) = OBJECT_TABLE.write() else {
            panic!("Lock poisoned");
        };

        let reference = object_table.add(object);

        reference
    }
}

impl MakeObject<&str> for RuntimeHelper {

    #[inline]
    fn make_self() -> Self {
        RuntimeHelper
    }

    #[inline]
    fn new_object(&self, class_name: &str) -> Reference {
        let Ok(class_map) = CLASS_MAPPER.read() else {
            panic!("Lock poisoned");
        };
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class_symbol = class_map[class_name];

        let SymbolEntry::ClassRef(class_ref) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let class = &class_table[class_ref];
        let parent_object = Runtime::new_object(class.parent);

        let data_size = class.get_member_size();

        let object = Object::new(class_symbol, parent_object, data_size, class.drop_function);

        let Ok(mut object_table) = OBJECT_TABLE.write() else {
            panic!("Lock poisoned");
        };

        let reference = object_table.add(object);

        reference
    }
}



pub struct Runtime {}

impl Runtime {
    pub extern "C" fn should_unwind(context: &mut BytecodeContext) -> u8 {
        if !context.is_current_exception_set() {
            return 0;
        }
        /*let exception = *context.current_exception.borrow();
        let exception = unsafe { exception.as_ref().unwrap() };
        let parent_exception = exception.parent_objects[0];
        exception_fill_in_stack_trace(context, parent_exception);
        context.pop_backtrace();
        let Some(last) = context.function_backtrace.last() else {
            return 0;
        };
        if let Some(symbols) = context.registered_exceptions.get(&last.get_method_name()) {
            for symbol in symbols {
                if *symbol == exception.class {
                    return 0;
                }
                let parent_exception = unsafe { parent_exception.as_ref().unwrap() };
                if *symbol == parent_exception.class {
                    return 0;
                }
            }
        }*/
        1
    }

    pub fn link_classes(
        classes: Vec<ClassFile>,
        class_locations: Vec<PathBuf>,
        pre_class_table: &mut Vec<TableEntry<Class>>,
        // The first hashmap is the class symbol which the vtable comes from.
        // The second hashmap is the class that has a custom version of the vtable
        // For example, two matching symbols means that that is the vtable of that particular class
        vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Vec<rowan_shared::TypeTag>, linker::MethodLocation, Box<[Bytecode]>, FunctionValue, Signature)>>>,
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
        let Ok(mut class_map) = CLASS_MAPPER.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut library_table) = LIBRARY_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut string_map) = STRING_MAP.write() else {
            panic!("Lock poisoned");
        };

        let out = linker::link_class_files(
            classes,
            class_locations,
            &mut jit_controller,
            &mut symbol_table,
            pre_class_table,
            &mut string_table,
            &mut vtable_tables,
            vtables_map,
            &mut string_map,
            class_map.borrow_mut(),
            &mut library_table,
        ).unwrap();

        //println!("class_map: {:#?}", &class_map);

        out
    }


    pub fn link_vm_classes(
        classes: Vec<VMClass>,
        pre_class_table: &mut Vec<TableEntry<Class>>,
        // The first hashmap is the class symbol which the vtable comes from.
        // The second hashmap is the class that has a custom version of the vtable
        // For example, two matching symbols means that that is the vtable of that particular class
        vtables_map: &mut HashMap<Symbol, HashMap<Symbol, Vec<(Symbol, Vec<rowan_shared::TypeTag>, linker::MethodLocation, Box<[Bytecode]>, FunctionValue, Signature)>>>,
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
        let Ok(mut class_map) = CLASS_MAPPER.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut string_map) = STRING_MAP.write() else {
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
            &mut string_map,
            class_map.borrow_mut(),
        );
    }

    pub fn finish_linking_classes(
        pre_class_table: Vec<TableEntry<Class>>,
    ) {
        let Ok(mut class_table) = CLASS_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let mut init_functions = Vec::new();
        for class in pre_class_table {
            match class {
                TableEntry::Hole => {
                    panic!("missing class");
                }
                TableEntry::Entry(class) => {
                    let init_function = if let Some(init_function) = &class.init_function {
                        Some(init_function.as_ref() as *const [Bytecode])
                    } else {
                        None
                    };
                    init_functions.push(init_function);
                    class_table.insert_class(class);
                }

            }
        }
        drop(class_table);
        let (sender, _) = std::sync::mpsc::channel();
        let mut context = BytecodeContext::new(sender);
        let block_positions = Box::new(FxHashMap::default());
        let block_positions = &*block_positions as *const FxHashMap<_, _>;
        let block_positions = unsafe { block_positions.as_ref().unwrap() };
        for function in init_functions {
            if let Some(function) = function {
                //let bytecode = unsafe { std::mem::transmute::<'static, _>(function.as_ref())};
                let function = unsafe {
                    function.as_ref().unwrap()
                };
                context.run_bytecode(function, block_positions);
            }
        }
    }

    pub fn get_virtual_method_details(
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        method_name: Symbol,
    ) -> FunctionDetails {
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
        let vtable_index = if let Some(index) = class.get_vtable(&class_symbol) {
            index
        } else {
            panic!("unable to find vtable");
        };

        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name).expect("unable to find function");

        function.create_details(MethodName::VirtualMethod {
            object_class_symbol,
            class_symbol,
            method_name,
        })
    }

    pub fn get_static_method_details(
        class_symbol: Symbol,
        method_name: Symbol,
    ) -> FunctionDetails {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(class_table) = CLASS_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };

        let class = &class_table[class_index];

        let vtable_index = class.static_methods;
        let Ok(vtables_table) = VTABLES.read() else {
            unreachable!("Lock poisoned");
        };
        drop(class_table);

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name).expect("unable to get function");

        function.create_details(MethodName::StaticMethod {
            class_symbol,
            method_name,
        })
    }

    pub fn jit_virtual_method(
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        method_name: Symbol,
    ) {
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
        let vtable_index = if let Some(index) = class.get_vtable(&class_symbol) {
            index
        } else {
            panic!("unable to find vtable");
        };

        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name).unwrap();

        let value = function.value.lock().unwrap();
        match &*value {
            FunctionValue::Bytecode(_) => {
                drop(value);
                let mut compiler = Runtime::create_jit_compiler();
                let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
                    panic!("Lock poisoned");
                };

                compiler.compile(function, &mut jit_controller.module).unwrap();
            }
            _ => {}
        };
    }

    pub fn jit_static_method(
        class_symbol: Symbol,
        method_name: Symbol,
    ) {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(class_table) = CLASS_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };

        let class = &class_table[class_index];

        let vtable_index = class.static_methods;
        let Ok(vtables_table) = VTABLES.read() else {
            unreachable!("Lock poisoned");
        };
        drop(class_table);

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name).unwrap();

        let value = function.value.lock().unwrap();
        match &*value {
            FunctionValue::Bytecode(_) => {
                drop(value);
                let mut compiler = Runtime::create_jit_compiler();
                let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
                    unreachable!("Lock poisoned");
                };

                match compiler.compile(function, &mut jit_controller.module) {
                    Ok(_) => {}
                    Err(e) => panic!("Compilation error:\n{}", e)
                }
            }
            _ => {}
        };
    }

    pub fn create_jit_compiler() -> JITCompiler {
        let Ok(jit_controller) = JIT_CONTROLLER.write() else {
            panic!("Lock poisoned");
        };
        let context = jit_controller.new_context();

        JITCompiler::new(context)
    }

    pub fn get_virtual_method_signature(class_symbol: Symbol, method_name: Symbol) -> (Signature, bool) {
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
        let vtable_index = class.get_vtable(&class_symbol).unwrap();
        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        /*let SymbolEntry::StringRef(index) = symbol_table[method_name] else {
            panic!("Method wasn't a string");
        };

        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
        };
        println!("method_name: {}", &string_table[index]);*/

        let function = vtable.get_function(method_name).unwrap();
        let is_object = match function.return_type {
            crate::runtime::class::TypeTag::Object => true,
            _ => false,
        };
        (function.signature.clone(), is_object)

    }

    pub fn get_static_method_signature(
        class_symbol: Symbol, 
        method_name: Symbol
    ) -> (Signature, bool) {
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
        let vtable_index = class.static_methods;
        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name).unwrap();
        let is_object = match function.return_type {
            crate::runtime::class::TypeTag::Object => true,
            _ => false,
        };
        (function.signature.clone(), is_object)
    }

    pub fn new_object<N>(class_name: N) -> Reference
    where
        RuntimeHelper: MakeObject<N> {
        let creator = <RuntimeHelper as MakeObject<N>>::make_self();
        creator.new_object(class_name)
    }

    pub fn get_class_symbol(class_name: &str) -> Symbol {
        let Ok(class_map) = CLASS_MAPPER.read() else {
            panic!("Lock poisoned");
        };
        class_map[class_name]
    }

    pub fn get_class_name(class_symbol: Symbol) -> String {
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

    pub fn get_class(class_symbol: Symbol) -> *const Class {
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

    pub fn get_string(string_symbol: Symbol) -> &'static str {
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

    pub fn intern_string(string_buffer: *mut StringBuffer) -> Symbol {
        let string_buffer = unsafe { string_buffer.as_ref().unwrap() };
        let pointer = string_buffer.buffer;
        let length = string_buffer.length;
        let slice = unsafe {
            std::slice::from_raw_parts(pointer, length as usize)
        };
        let string = std::str::from_utf8(slice).unwrap();



        let Ok(string_map) = STRING_MAP.read() else {
            panic!("Lock poisoned");
        };

        if let Some(symbol) = string_map.get(string) {
            return *symbol;
        }


        let Ok(mut symbol_table) = SYMBOL_TABLE.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut string_table) = STRING_TABLE.write() else {
            panic!("Lock poisoned");
        };


        let index = string_table.add_string(string);

        symbol_table.add_string(index)
    }




    pub fn dereference_stack_pointer(
        backtrace_stack_pointer_instruction_pointer: &[(MethodName, usize, usize)],
        references: &mut HashSet<WrappedReference>
    ) {

        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(class_table) = CLASS_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(vtables_table) = VTABLES.read() else {
            unreachable!("Lock poisoned");
        };


        for (name, sp, ip) in backtrace_stack_pointer_instruction_pointer {
            //println!("\tderef: {:?}", name);
            //println!("\tsp: {:x}, RIP: {:x}", sp, ip);
            match name {
                MethodName::StaticMethod {
                    class_symbol,
                    method_name,
                } => {
                    let SymbolEntry::ClassRef(class_index) = symbol_table[*class_symbol] else {
                        panic!("class wasn't a class");
                    };
                    let class = &class_table[class_index];
                    let vtable_index = class.static_methods;
                    let vtable = &vtables_table[vtable_index];
                    let function = vtable.get_function(*method_name).expect("unable to get function");
                    let value = function.value.lock().unwrap();
                    let FunctionValue::Compiled(_, map) = &*value else {
                        unreachable!("we are trying to access the stack of a non-compiled function");
                    };
                    if let Some(offsets) = map.get(ip) {
                        for offset in offsets {
                            let pointer = (*sp + *offset as usize) as *mut Reference;
                            //println!("dereferencing: {:x?}", pointer);
                            unsafe {
                                references.insert(WrappedReference(*pointer));
                            }
                        }
                    }
                }
                MethodName::VirtualMethod {
                    object_class_symbol,
                    class_symbol,
                    method_name
                } => {
                    let SymbolEntry::ClassRef(object_class_index) = symbol_table[*object_class_symbol] else {
                        panic!("class wasn't a class");
                    };

                    let class = &class_table[object_class_index];
                    let vtable_index = if let Some(index) = class.get_vtable(&class_symbol) {
                        index
                    } else {
                        panic!("unable to find vtable");
                    };

                    let Ok(vtables_table) = VTABLES.read() else {
                        panic!("Lock poisoned");
                    };

                    let vtable = &vtables_table[vtable_index];
                    let function = vtable.get_function(*method_name).expect("unable to find function");

                    let value = &*function.value.lock().unwrap();
                    let FunctionValue::Compiled(_, map) = value else {
                        let Ok(string_table) = STRING_TABLE.read() else {
                            unreachable!("we are trying to access the stack of a non-compiled function");
                        };
                        let SymbolEntry::StringRef(index) = symbol_table[*method_name] else {
                            panic!("class wasn't a string");
                        };
                        let string = &string_table[index];
                        unreachable!("we are trying to access the stack of a non-compiled function: {string}");
                    };
                    if let Some(offsets) = map.get(ip) {
                        for offset in offsets {
                            let pointer = (*sp + *offset as usize) as *mut Reference;
                            unsafe {
                                references.insert(WrappedReference(*pointer));
                            }
                        }
                    }
                }
            }
        }
    }


    pub fn gc_explore_object(reference: Reference, live_objects: &mut HashSet<Reference>) {
        Object::garbage_collect(reference, live_objects);
    }

    pub fn gc_collect_garbage(live_objects: &HashSet<Reference>) {
        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };
        let Ok(mut object_table) = OBJECT_TABLE.write() else {
            panic!("Lock poisoned");
        };

        let mut objects_to_delete = Vec::new();

        for (i, _) in object_table.iter().enumerate() {
            if !live_objects.contains(&(i as Reference)) {
                objects_to_delete.push(i as Reference);
            }
        }

        //println!("Survived: {live_objects:?}");

        for reference in objects_to_delete {
            object_table.free(reference, &symbol_table, &class_table);
        }
    }

    pub fn check_and_do_garbage_collection(ctx: &mut BytecodeContext) {
        ctx.check_and_do_garbage_collection();
    }


    pub fn get_virtual_method_name(class: &str, method_name: &str) -> Option<(Symbol, Symbol)> {
        let Ok(class_map) = CLASS_MAPPER.write() else {
            panic!("Lock poisoned");
        };
        let Ok(string_map) = STRING_MAP.write() else {
            panic!("Lock poisoned");
        };

        let class_symbol = *class_map.get(class)?;
        let method_name = *string_map.get(method_name)?;

        Some((class_symbol, method_name))
    }

    pub fn get_static_method_name(class: &str, method_name: &str) -> Option<(Symbol, Symbol)> {
        let Ok(class_map) = CLASS_MAPPER.write() else {
            panic!("Lock poisoned");
        };
        let Ok(string_map) = STRING_MAP.write() else {
            panic!("Lock poisoned");
        };

        let class_symbol = *class_map.get(class)?;
        let method_name = *string_map.get(method_name)?;

        Some((class_symbol, method_name))
    }

    pub fn get_static_member<T>(_ctx: &mut BytecodeContext, class: Symbol, index: u64) -> T
    where
    RuntimeHelper: StaticMemberAccess<T> {
        let helper = <RuntimeHelper as StaticMemberAccess<T>>::make_self();
        helper.get_static_member(class, index as usize).expect("todo: throw exception")
    }

    pub fn set_static_member<T>(_ctx: &mut BytecodeContext, class: Symbol, index: u64, value: T)
    where
        RuntimeHelper: StaticMemberAccess<T> {
        let helper = <RuntimeHelper as StaticMemberAccess<T>>::make_self();
        helper.set_static_member(class, index as usize, value).expect("todo: throw exception");
    }
}


pub extern "C" fn call_virtual_function(context: &mut BytecodeContext, class_symbol: u64, method_name: u64) {
    let class_symbol = class_symbol as Symbol;
    let method_name = method_name as Symbol;
    context.invoke_virtual_extern(class_symbol, method_name, None);
}

pub extern "C" fn new_object(class_symbol: u64) -> u64 {
    let class_symbol = class_symbol as Symbol;
    let object = Runtime::new_object(class_symbol);
    object as usize as u64
}

pub extern "C" fn call_static_function(context: &mut BytecodeContext, class_symbol: u64, method_name: u64) {
    let class_symbol = class_symbol as Symbol;
    let method_name = method_name as Symbol;
    context.invoke_static_extern(class_symbol, method_name, None);
}

pub extern "C" fn get_static_member8(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> u8 {
    Runtime::get_static_member::<u8>(context, class_symbol as Symbol, member_index)
}

pub extern "C" fn get_static_member16(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> u16 {
    Runtime::get_static_member::<u16>(context, class_symbol as Symbol, member_index)
}

pub extern "C" fn get_static_member32(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> u32 {
    Runtime::get_static_member::<u32>(context, class_symbol as Symbol, member_index)
}

pub extern "C" fn get_static_member64(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> u64 {
    Runtime::get_static_member::<u64>(context, class_symbol as Symbol, member_index)
}

pub extern "C" fn get_static_memberf32(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> f32 {
    Runtime::get_static_member::<f32>(context, class_symbol as Symbol, member_index)
}

pub extern "C" fn get_static_memberf64(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> f64 {
    Runtime::get_static_member::<f64>(context, class_symbol as Symbol, member_index)
}

pub extern "C" fn get_static_memberobject(context: &mut BytecodeContext, class_symbol: u64, member_index: u64) -> u64 {
    Runtime::get_static_member::<Reference>(context, class_symbol as Symbol, member_index) as usize as u64
}

pub extern "C" fn set_static_member8(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: u8) {
    Runtime::set_static_member::<u8>(context, class_symbol as Symbol, member_index, value)
}

pub extern "C" fn set_static_member16(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: u16) {
    Runtime::set_static_member::<u16>(context, class_symbol as Symbol, member_index, value)
}

pub extern "C" fn set_static_member32(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: u32) {
    Runtime::set_static_member::<u32>(context, class_symbol as Symbol, member_index, value)
}

pub extern "C" fn set_static_member64(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: u64) {
    Runtime::set_static_member::<u64>(context, class_symbol as Symbol, member_index, value)
}

pub extern "C" fn set_static_memberf32(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: f32) {
    Runtime::set_static_member::<f32>(context, class_symbol as Symbol, member_index, value)
}

pub extern "C" fn set_static_memberf64(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: f64) {
    Runtime::set_static_member::<f64>(context, class_symbol as Symbol, member_index, value)
}

pub extern "C" fn set_static_memberobject(context: &mut BytecodeContext, class_symbol: u64, member_index: u64, value: u64) {
    Runtime::set_static_member::<Reference>(context, class_symbol as Symbol, member_index, value as usize as Reference)
}