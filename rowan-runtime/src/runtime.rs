use std::{collections::HashMap, sync::LazyLock};
use std::cell::RefCell;
use std::sync::{Arc, RwLock, TryLockError};

use class::Class;

use cranelift::prelude::Signature;
use jit::{JITCompiler, JITController};
use linker::TableEntry;
use object::Object;
use rowan_shared::classfile::ClassFile;
use core::VMClass;
use tables::{class_table::ClassTable, object_table::ObjectTable, string_table::StringTable, symbol_table::{SymbolEntry, SymbolTable}, vtable::{FunctionValue, VTables}};
use std::borrow::{BorrowMut, Cow};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32};
use std::sync::mpsc::Sender;
use rowan_shared::bytecode::linked::Bytecode;
use crate::context::BytecodeContext;
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

use crate::runtime::core::{exception_fill_in_stack_trace, exception_print_stack_trace};
use crate::runtime::garbage_collection::GC_SENDER;
use crate::runtime::tables::native_object_table::NativeObjectTable;

pub type Symbol = usize;

pub type Reference = *mut Object;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct WrappedReference(Reference);

unsafe impl Send for WrappedReference {}
unsafe impl Sync for WrappedReference {}

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

trait MakeObject<N> {
    fn make_self() -> Self;
    fn new_object(&self, class_name: N) -> Reference;
}

struct RuntimeHelper;

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
        let parent_objects = &class.parents;

        let mut parents = Vec::new();

        for parent in parent_objects.iter() {
            parents.push(Runtime::new_object(*parent));
        }

        let data_size = class.get_member_size();
        let object = Object::new(class_symbol, parents.into_boxed_slice(), data_size, class.drop_function);

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
        let Ok(mut class_map) = CLASS_MAPPER.read() else {
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
        let parent_objects = &class.parents;

        let mut parents = Vec::new();

        for parent in parent_objects.iter() {
            parents.push(Runtime::new_object(*parent));
        }

        let data_size = class.get_member_size();

        let object = Object::new(class_symbol, parents.into_boxed_slice(), data_size, class.drop_function);

        let Ok(mut object_table) = OBJECT_TABLE.write() else {
            panic!("Lock poisoned");
        };

        let reference = object_table.add(object);

        reference
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MethodName {
    StaticMethod {
        class_symbol: Symbol,
        method_name: Symbol,
    },
    VirtualMethod {
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        source_class: Option<Symbol>,
        method_name: Symbol,
    }
}

impl MethodName {
    pub fn get_method_name(&self) -> Symbol {
        match self {
            MethodName::StaticMethod { method_name, .. } => *method_name,
            MethodName::VirtualMethod { method_name, .. } => *method_name,
        }
    }
}

pub struct Runtime {
    /// The reference to the current exception
    /// If the reference is non-zero then we should unwind until we hit a registered exception
    pub current_exception: RefCell<Reference>,
    /// The backtrace of function names.
    /// This gets appended as functions get called, popped as functions return
    function_backtrace: Vec<MethodName>,
    /// A map between function_backtraces and all currently registered exceptions
    registered_exceptions: HashMap<Symbol, Vec<Symbol>>,
    static_memo: HashMap<(Symbol, Symbol), *const ()>,
    virtual_memo: HashMap<(Symbol, Symbol, Option<Symbol>, Symbol), *const ()>,
    sender: Sender<HashSet<WrappedReference>>,
}

impl Runtime {
    pub fn new(sender: Sender<HashSet<WrappedReference>>) -> Self {
        Runtime {
            current_exception: RefCell::new(std::ptr::null_mut()),
            function_backtrace: Vec::new(),
            registered_exceptions: HashMap::new(),
            static_memo: HashMap::new(),
            virtual_memo: HashMap::new(),
            sender,
        }
    }
    
    pub fn set_exception(&self, exception: Reference) {
        *self.current_exception.borrow_mut() = exception;
    }

    pub fn get_exception(&self) -> Reference {
        *self.current_exception.borrow()
    }

    pub fn push_backtrace(&mut self, method_name: MethodName) {
        self.function_backtrace.push(method_name);
    }

    pub fn pop_backtrace(&mut self) -> Option<MethodName> {
        self.function_backtrace.pop()
    }

    pub fn get_current_method(&mut self) -> Reference {
        let string_ref = Self::new_object("String");

        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(string_table) = STRING_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let method_name = self.function_backtrace[self.function_backtrace.len() - 1].get_method_name();

        let SymbolEntry::StringRef(method_name_index) = symbol_table[method_name] else {
            panic!("method wasn't a string");
        };

        let name = &string_table[method_name_index];

        core::string_from_str(string_ref, name);

        string_ref
    }

    pub extern "C" fn should_unwind(context: &mut Self) -> u8 {
        if context.current_exception.borrow().is_null() {
            return 0;
        }
        let exception = *context.current_exception.borrow();
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
        }
        1
    }

    pub extern "C" fn normal_return(ctx: &mut Self) {
        let _out = ctx.pop_backtrace();
        //println!("Normal Return: {out:?}");
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
        let mut jit_compiler = Runtime::create_jit_compiler();
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
            &mut jit_compiler,
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
                    init_functions.push(class.init_function);
                    class_table.insert_class(class);
                }

            }
        }
        drop(class_table);
        /*for function in init_functions {
            //let (sender, _) = std::sync::mpsc::channel();
            let mut context = BytecodeContext::new();
            function(&mut context);
            /*if !context.current_exception.borrow().is_null() {
                println!("Failed to initialize class static members");
                let exception = context.get_exception();
                let exception = unsafe { exception.as_ref().unwrap() };
                let base_exception_ref = exception.parent_objects[0];
                let message = unsafe { exception.get::<Reference>(0) };
                let message = unsafe { message.as_ref().unwrap() };
                let message_slice = unsafe { std::slice::from_raw_parts(message.get::<*const u8>(16), message.get(0)) };
                let message_str = std::str::from_utf8(message_slice).unwrap();
                println!("{message_str}");
                exception_print_stack_trace(&mut context, base_exception_ref);
                std::process::exit(1);
            }*/
        }*/
    }

    pub fn get_virtual_method_details(
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        source_class: Option<Symbol>,
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
        let SymbolEntry::StringRef(method_name_index) = symbol_table[method_name] else {
            panic!("method wasn't a string");
        };

        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let name = &string_table[method_name_index];

        let class = &class_table[object_class_index];
        let vtable_index = if source_class.is_some() {
            let key = (class_symbol, None);
            if let Some(index) = class.get_vtable(&key) {
                index
            } else if let Some(index) = class.get_vtable(&(class_symbol, source_class)) {
                index
            } else {
                panic!("unable to find vtable");
            }
        } else {
            if let Some(index) = class.get_vtable(&(class_symbol, source_class)) {
                index
            } else {
                panic!("unable to find vtable");
            }
        };

        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &vtables_table[vtable_index];
        let function = vtable.get_function(method_name).expect("unable to find function");

        function.create_details()
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

        function.create_details()
    }

    /*pub fn get_method(
        &mut self,
        object_class_symbol: Symbol,
        class_symbol: Symbol,
        source_class: Option<Symbol>,
        method_name: Symbol,
    ) -> *const () {
        self.push_backtrace(MethodName::VirtualMethod {
            object_class_symbol,
            class_symbol,
            source_class,
            method_name,
        });

        if let Some(method) = self.virtual_memo.get(&(object_class_symbol, class_symbol, source_class, method_name)) {
            return *method;
        }

        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            panic!("Lock poisoned");
        };


        let Ok(class_table) = CLASS_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(object_class_index) = symbol_table[object_class_symbol] else {
            panic!("class wasn't a class");
        };
        let SymbolEntry::StringRef(method_name_index) = symbol_table[method_name] else {
            panic!("method wasn't a string");
        };

        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let name = &string_table[method_name_index];

        let class = &class_table[object_class_index];
        let vtable_index = if source_class.is_some() {
            let key = (class_symbol, None);
            if let Some(index) = class.get_vtable(&key) {
                index
            } else if let Some(index) = class.get_vtable(&(class_symbol, source_class)) {
                index
            } else {
                panic!("unable to find vtable");
            }
        } else {
            if let Some(index) = class.get_vtable(&(class_symbol, source_class)) {
                index
            } else {
                panic!("unable to find vtable");
            }
        };

        let Ok(vtables_table) = VTABLES.read() else {
            panic!("Lock poisoned");
        };

        let vtable = &mut vtables_table[vtable_index];
        let function = vtable.get_function_mut(method_name);

        let value = &function.value;
        let value = match &*value {
            FunctionValue::Builtin(ptr) => *ptr,
            FunctionValue::Compiled(ptr, _) => *ptr,
            FunctionValue::Native(ptr) => *ptr,
            _ => {
                let mut compiler = Runtime::create_jit_compiler();
                let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
                    panic!("Lock poisoned");
                };
                
                compiler.compile(&function, &mut jit_controller.module, name).unwrap();

                match &*value {
                    FunctionValue::Compiled(ptr,_ ) => *ptr,
                    _ => panic!("Function wasn't compiled")
                }
            }
        };

        self.virtual_memo.insert((object_class_symbol, object_class_symbol, source_class, method_name), value);
        value
    }

    pub fn get_static_method(
        &mut self,
        class_symbol: Symbol,
        method_name: Symbol,
    ) -> *const () {
        self.push_backtrace(MethodName::StaticMethod {
            class_symbol,
            method_name,
        });

        if let Some(method) = self.static_memo.get(&(class_symbol, method_name)) {
            return *method;
        }

        let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(class_table) = CLASS_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol] else {
            panic!("class wasn't a class");
        };
        let SymbolEntry::StringRef(method_name_index) = symbol_table[method_name] else {
            panic!("method wasn't a string");
        };

        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
        };

        let name = &string_table[method_name_index];


        let class = &class_table[class_index];

        let vtable_index = class.static_methods;
        let Ok(vtables_table) = VTABLES.read() else {
            unreachable!("Lock poisoned");
        };
        drop(class_table);

        let vtable = &vtables_table[vtable_index];
        let mut function = vtable.get_function_mut(method_name);

        let value = &function.value;
        let value = match &*value {
            FunctionValue::Builtin(ptr) => *ptr,
            FunctionValue::Native(ptr) => *ptr,
            FunctionValue::Compiled(ptr, _) => *ptr,
            _ => {
                let mut compiler = Runtime::create_jit_compiler();
                let Ok(mut jit_controller) = JIT_CONTROLLER.write() else {
                    unreachable!("Lock poisoned");
                };

                match compiler.compile(&function, &mut jit_controller.module, name) {
                    Ok(_) => {}
                    Err(e) => panic!("Compilation error:\n{}", e)
                }

                match &*value {
                    FunctionValue::Compiled(ptr, _) => *ptr,
                    _ => panic!("Function wasn't compiled")
                }
            }
        };

        self.static_memo.insert((class_symbol, method_name), value);
        value
    }*/

    pub fn create_jit_compiler() -> JITCompiler {
        let Ok(jit_controller) = JIT_CONTROLLER.write() else {
            panic!("Lock poisoned");
        };
        let context = jit_controller.new_context();

        JITCompiler::new(context)
    }

    pub fn get_method_signature(class_symbol: Symbol, method_name: Symbol) -> (Signature, bool) {
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
        let vtable_index = class.get_vtable(&(class_symbol, None)).unwrap();
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
        let creator = RuntimeHelper::make_self();
        creator.new_object(class_name)
    }

    pub fn get_class_symbol(class_name: &str) -> Symbol {
        let Ok(mut class_map) = CLASS_MAPPER.read() else {
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


    pub fn check_and_do_garbage_collection(&mut self) {
        match DO_GARBAGE_COLLECTION.try_read() {
            Ok(_) => return,
            Err(TryLockError::WouldBlock) => {

            }
            Err(TryLockError::Poisoned(_)) => {
                panic!("Lock poisoned");
            }
        }
        //println!("Garbage collection");
        
        let mut info = Vec::new();

        //println!("backtrace len {}", self.function_backtrace.len());
        let mut backtrace_iter = self.function_backtrace.iter().rev();

        rowan_unwind::backtrace(|frame| {
            if frame.is_jitted() {
                let sp = frame.sp();
                let ip = frame.ip();
                //println!("RSP: {:x}, RIP: {:x}", sp, ip);
                let Some(backtrace) = backtrace_iter.next() else {
                    return false;
                };

                info.push((*backtrace, sp as usize, ip as usize));
            }

            true
        });

        let live_objects = self.dereference_stack_pointer(&info);
        self.sender.send(live_objects).unwrap();
        loop {
            match DO_GARBAGE_COLLECTION.try_read() {
                Ok(_) => return,
                Err(TryLockError::WouldBlock) => {

                }
                Err(TryLockError::Poisoned(_)) => {
                    panic!("Lock poisoned");
                }
            }
        }
        //println!("live_objects: {:x?}", live_objects);
    }

    fn dereference_stack_pointer(
        &self,
        backtrace_stack_pointer_instruction_pointer: &[(MethodName, usize, usize)]
    ) -> HashSet<WrappedReference> {
        let mut live_objects = HashSet::new();

        /*let Ok(symbol_table) = SYMBOL_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(class_table) = CLASS_TABLE.read() else {
            unreachable!("Lock poisoned");
        };

        let Ok(vtables_table) = VTABLES.read() else {
            unreachable!("Lock poisoned");
        };
        let Ok(string_table) = STRING_TABLE.read() else {
            panic!("Lock poisoned");
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
                    let value = &function.value;
                    let FunctionValue::Compiled(_, map) = &*value else {
                        unreachable!("we are trying to access the stack of a non-compiled function");
                    };
                    if let Some(offsets) = map.get(ip) {
                        for offset in offsets {
                            let pointer = (*sp + *offset as usize) as *mut Reference;
                            //println!("dereferencing: {:x?}", pointer);
                            unsafe {
                                live_objects.insert(*pointer);
                            }
                        }
                    }
                }
                MethodName::VirtualMethod {
                    object_class_symbol,
                    class_symbol,
                    source_class,
                    method_name
                } => {
                    let SymbolEntry::ClassRef(object_class_index) = symbol_table[*object_class_symbol] else {
                        panic!("class wasn't a class");
                    };

                    let class = &class_table[object_class_index];
                    let vtable_index = if source_class.is_some() {
                        let key = (*class_symbol, None);
                        if let Some(index) = class.get_vtable(&key) {
                            index
                        } else if let Some(index) = class.get_vtable(&(*class_symbol, *source_class)) {
                            index
                        } else {
                            panic!("unable to find vtable");
                        }
                    } else {
                        if let Some(index) = class.get_vtable(&(*class_symbol, *source_class)) {
                            index
                        } else {
                            panic!("unable to find vtable");
                        }
                    };

                    let Ok(vtables_table) = VTABLES.read() else {
                        panic!("Lock poisoned");
                    };

                    let vtable = &vtables_table[vtable_index];
                    let function = vtable.get_function(*method_name).expect("unable to find function");

                    let value = function.value.read().unwrap();
                    let FunctionValue::Compiled(_, map) = &*value else {
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
                                live_objects.insert(*pointer);
                            }
                        }
                    }
                }
            }
        }*/

        live_objects.into_iter().map(WrappedReference).collect::<HashSet<_>>()
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

    /*pub fn get_virtual_method(&mut self, object: Reference, class: &str, source_class: Option<&str>, method_name: &str) -> *const () {
        self.check_and_do_garbage_collection();

        let Ok(mut class_map) = CLASS_MAPPER.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut string_map) = STRING_MAP.write() else {
            panic!("Lock poisoned");
        };

        let class_symbol = *class_map.get(class).expect("unable to find class symbol");
        let source_class = source_class.map(|c| *class_map.get(c).expect("unable to find class symbol"));
        let method_name = *string_map.get(method_name).expect("unable to find method name");
        drop(class_map);
        drop(string_map);

        let object_class = unsafe {
            let object = object.as_ref().unwrap();
            object.class
        };

        self.get_method(object_class, class_symbol, source_class, method_name)
    }

    pub fn get_static_function(&mut self, class: &str, method_name: &str) -> *const () {
        self.check_and_do_garbage_collection();

        let Ok(mut class_map) = CLASS_MAPPER.write() else {
            panic!("Lock poisoned");
        };
        let Ok(mut string_map) = STRING_MAP.write() else {
            panic!("Lock poisoned");
        };

        let class_symbol = *class_map.get(class).expect("unable to find class symbol");
        let method_name = *string_map.get(method_name).expect("unable to find method name");
        drop(class_map);
        drop(string_map);

        self.get_static_method(class_symbol, method_name)
    }*/
}


pub extern "C" fn get_virtual_function(context: &mut Runtime, object: Reference, class_symbol: u64, source_class: i64, method_name: u64) -> u64 {
    context.check_and_do_garbage_collection();

    let object = unsafe {object.as_mut().unwrap()};

    let object_class_symbol = object.class;
    let class_symbol = class_symbol as Symbol;
    let source_class = if source_class < 0 {
        None
    } else {
        Some(source_class as Symbol)
    };
    let method_name = method_name as Symbol;
    /*let method_ptr = context.get_method(object_class_symbol, class_symbol, source_class, method_name);

    method_ptr as usize as u64*/
    0
}

pub extern "C" fn new_object(class_symbol: u64) -> u64 {
    let class_symbol = class_symbol as Symbol;
    let object = Runtime::new_object(class_symbol);
    object as usize as u64
}

pub extern "C" fn get_static_function(context: &mut Runtime, class_symbol: u64, method_name: u64) -> u64 {
    context.check_and_do_garbage_collection();

    let class_symbol = class_symbol as Symbol;
    let method_name = method_name as Symbol;
    /*let method_ptr = context.get_static_method(class_symbol, method_name);

    method_ptr as usize as u64*/
    0
}

pub extern "C" fn get_static_member8(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> u8 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Byte(v), .. } ) => *v,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn get_static_member16(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> u16 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Short(v), .. } ) => *v,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn get_static_member32(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> u32 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Int(v), .. } ) => *v,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn get_static_member64(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> u64 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class {class_symbol}");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Long(v), .. } ) => *v,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn get_static_memberf32(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> f32 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Float(v), .. } ) => *v,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn get_static_memberf64(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> f64 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Double(v), .. } ) => *v,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn get_static_memberobject(_context: &mut Runtime, class_symbol: u64, member_index: u64) -> u64 {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(class_table) = CLASS_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let class = &class_table[class_index];

    match class.get_member(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Object(v), .. } ) => *v as usize as u64,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_member8(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: u8) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    match class.get_member_mut(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Byte(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_member16(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: u16) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    match class.get_member_mut(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Short(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_member32(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: u32) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    match class.get_member_mut(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Int(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_member64(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: u64) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    match class.get_member_mut(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Long(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_memberf32(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: f32) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    match class.get_member_mut(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Float(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_memberf64(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: f64) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    match class.get_member_mut (member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Double(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}

pub extern "C" fn set_static_memberobject(_context: &mut Runtime, class_symbol: u64, member_index: u64, value: u64) {
    let Ok(symbol_table) = SYMBOL_TABLE.read() else {
        panic!("Lock poisoned");
    };
    let SymbolEntry::ClassRef(class_index) = symbol_table[class_symbol as Symbol] else {
        panic!("class wasn't a class");
    };
    let Ok(mut class_table) = CLASS_TABLE.write() else {
        panic!("Lock poisoned");
    };
    let class = &mut class_table[class_index];

    let value = value as usize as Reference;

    match class.get_member_mut(member_index as usize) {
        Some(ClassMember {  data: ClassMemberData::Object(v), .. } ) => *v = value,
        _ => todo!("Throw an exception")
    }
}