use std::{collections::HashMap, io::Read};

use rowan_shared::classfile::ClassFile;
use runtime::{stdlib, Context};
use crate::runtime::Runtime;

mod runtime;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return
    }

    let mut file = std::fs::File::open(&args[1]).unwrap();
    let mut output = Vec::new();
    file.read_to_end(&mut output).unwrap();

    let vm_classes = vec![
        stdlib::generate_object_class(),
        stdlib::generate_array8_class(),
        stdlib::generate_array16_class(),
        stdlib::generate_array32_class(),
        stdlib::generate_array64_class(),
        stdlib::generate_arrayobject_class(),
        stdlib::generate_arrayf32_class(),
        stdlib::generate_arrayf64_class(),
        stdlib::generate_printer_class(),
        stdlib::generate_exception_class(),
        stdlib::generate_backtrace_class(),
        stdlib::generate_string_class(),
        stdlib::generate_index_out_of_bounds_class(),
        stdlib::generate_null_pointer_class(),
    ];

    let mut string_map = HashMap::new();

    let mut pre_class_table = Vec::new();
    let mut vtables_map = HashMap::new(); 

    Context::link_vm_classes(vm_classes, &mut pre_class_table, &mut vtables_map, &mut string_map);

    let class = ClassFile::new(&output);

    let classes = vec![class];

    let (main_symbol, ready_symbol) = Context::link_classes(classes, &mut pre_class_table, &mut vtables_map, &mut string_map);
    
    Context::finish_linking_classes(pre_class_table);

    let mut runtime = Runtime::new(1);
    runtime.spawn_thread();
    
    runtime.main_loop(main_symbol);

    /*let main_object_ref = Context::new_object(main_symbol);
    let Some(main_object) = context.get_object(main_object_ref) else {
        unreachable!("should have succeeded");
    };
    let main_object = unsafe { main_object.as_ref().unwrap() };

    println!("[Main] {}", context.get_class_name(12));

    let class = context.get_class(main_object.class);
    println!("[Main] Class: {:?}", unsafe {class.read()});

    // println!("{ready_symbol}");
    let method = context.get_method(main_object.class, 1, None, ready_symbol);
    let method = unsafe { std::mem::transmute::<_, fn(&mut Context, u64)>(method) };
    method(&mut context, main_object_ref);*/
}
