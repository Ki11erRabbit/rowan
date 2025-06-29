use std::{collections::HashMap, io::Read};

use rowan_shared::classfile::ClassFile;
use runtime::{core, Context};
use crate::runtime::Reference;
use crate::runtime::core::exception_print_stack_trace;
use crate::runtime::garbage_collection::{GarbageCollection, GC_SENDER};

mod runtime;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return
    }

    let classes = args[1..].iter().map(|f| {
        //println!("{}", f);
        let mut file = std::fs::File::open(f).unwrap();
        let mut output = Vec::new();
        file.read_to_end(&mut output).unwrap();
        ClassFile::new(&output)
    }).collect::<Vec<_>>();

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

    let mut string_map = HashMap::new();

    let mut pre_class_table = Vec::new();
    let mut vtables_map = HashMap::new(); 

    Context::link_vm_classes(vm_classes, &mut pre_class_table, &mut vtables_map, &mut string_map);


    let (main_symbol, main_method_symbol) = Context::link_classes(classes, &mut pre_class_table, &mut vtables_map, &mut string_map);

    Context::finish_linking_classes(pre_class_table);
    //println!("String Map: {string_map:#?}");
    let mut gc = GarbageCollection::new();
    std::thread::Builder::new().name("Garbage Collection".to_owned())
        .spawn(move || {
            gc.main_loop()
        }).expect("Thread 'new' panicked at 'Garbage Collection'");

    let sender = unsafe { GC_SENDER.clone().unwrap() };
    let mut context = Context::new(sender);
    
    //println!("main_symbol: {}, main_method_symbol: {}", main_symbol, main_method_symbol);
    
    let method = context.get_static_method(main_symbol, main_method_symbol);

    let method = unsafe { std::mem::transmute::<_, fn(&mut Context, u64)>(method) };
    
    method(&mut context, 0);
    if *context.current_exception.borrow() != 0 {
        let exception = context.get_exception();
        let exception = context.get_object(exception).unwrap();
        let exception = unsafe { exception.as_ref().unwrap() };
        let base_exception_ref = exception.parent_objects[0];
        let exception = context.get_object(base_exception_ref).unwrap();
        let exception = unsafe { exception.as_ref().unwrap() };
        let message = unsafe { exception.get::<Reference>(0) };
        let message = context.get_object(message).unwrap();
        let message = unsafe { message.as_ref().unwrap() };
        let message_slice = unsafe { std::slice::from_raw_parts(message.get::<*const u8>(16), message.get(0)) };
        let message_str = std::str::from_utf8(message_slice).unwrap();
        println!("{message_str}");
        exception_print_stack_trace(&mut context, base_exception_ref);
        std::process::exit(1);
    }
}
