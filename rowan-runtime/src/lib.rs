use std::{collections::HashMap, io::Read};
use std::path::PathBuf;
use rowan_shared::classfile::ClassFile;
use runtime::{core, Runtime};
use crate::context::BytecodeContext;
use crate::runtime::garbage_collection::{GarbageCollection, GC_SENDER};
use crate::runtime::jit::{set_jit_sender, JITController};

mod runtime;
mod fake_lock;
mod external;
mod context;

/// The start function for calling the main method in Rowan.
/// This function will parse commandline arguments from a Rust Context so don't call it from anywhere else.
/// It will initialize the state of the Rowan runtime by configuring the VM, linking core, and user classes, and start the garbage collector.
/// After that, it will call the main method.
#[unsafe(no_mangle)]
pub extern "C" fn rowan_main() {
    env_logger::init();
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return
    }

    let (classes, paths): (Vec<ClassFile>, Vec<PathBuf>) = args[1..].iter().map(|f| {
        //println!("{}", f);
        let mut file = std::fs::File::open(f).unwrap();
        let mut output = Vec::new();
        file.read_to_end(&mut output).unwrap();
        (ClassFile::new(&output), PathBuf::from(f))
    }).unzip();

    let paths = paths.into_iter()
        .map(|mut f| {
            f.pop();
            f
        })
        .collect::<Vec<PathBuf>>();

    let vm_classes = vec![
        core::generate_object_class(),
        core::generate_array8_class(),
        core::generate_array16_class(),
        core::generate_array32_class(),
        core::generate_array64_class(),
        core::generate_arrayobject_class(),
        core::generate_arrayf32_class(),
        core::generate_arrayf64_class(),
        core::generate_printer_class(),
        core::generate_exception_class(),
        core::generate_backtrace_class(),
        core::generate_string_class(),
        core::generate_index_out_of_bounds_class(),
        core::generate_null_pointer_class(),
    ];


    let mut pre_class_table = Vec::new();
    let mut vtables_map = HashMap::new();

    Runtime::link_vm_classes(vm_classes, &mut pre_class_table, &mut vtables_map);


    let (main_symbol, main_method_symbol) = Runtime::link_classes(classes, paths, &mut pre_class_table, &mut vtables_map);

    //println!("String Map: {string_map:#?}");

    Runtime::finish_linking_classes(pre_class_table);

    let (jit_sender, jit_receiver) = std::sync::mpsc::channel();

    set_jit_sender(jit_sender);

    std::thread::Builder::new().name("JIT".to_owned())
        .spawn(move || {
            JITController::jit_thread(jit_receiver);
        }).expect("Thread 'new' panicked at 'Garbage Collection'");


    //println!("String Map: {string_map:#?}");
    let mut gc = GarbageCollection::new();
    std::thread::Builder::new().name("Garbage Collection".to_owned())
        .spawn(move || {
            gc.main_loop()
        }).expect("Thread 'new' panicked at 'Garbage Collection'");

    let sender = GC_SENDER.read().clone().unwrap();
    let mut context = BytecodeContext::new(sender);

    //println!("main_symbol: {}, main_method_symbol: {}", main_symbol, main_method_symbol);
    context.call_main(main_symbol, main_method_symbol);

    /*let method = context.get_static_method(main_symbol, main_method_symbol);

    let method = unsafe { std::mem::transmute::<_, fn(&mut Runtime, u64)>(method) };

    method(&mut context, 0);
    if !context.current_exception.borrow().is_null() {
        let exception = context.get_exception();
        let exception = unsafe { exception.as_ref().unwrap() };
        let base_exception_ref = exception.parent_objects[0];
        let exception = unsafe { base_exception_ref.as_ref().unwrap() };
        let message = unsafe { exception.get::<Reference>(0) };
        let message = unsafe { message.as_ref().unwrap() };
        let message_slice = unsafe { std::slice::from_raw_parts(message.get::<*const u8>(16), message.get(0)) };
        let message_str = std::str::from_utf8(message_slice).unwrap();
        println!("{message_str}");
        exception_print_stack_trace(&mut context, base_exception_ref);
        std::process::exit(1);
    }*/
}
