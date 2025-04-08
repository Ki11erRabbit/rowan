use std::{collections::HashMap, io::Read};

use rowan_shared::classfile::ClassFile;
use runtime::{stdlib, Context};

mod runtime;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return
    }

    let mut file = std::fs::File::open(&args[1]).unwrap();
    let mut output = Vec::new();
    file.read_to_end(&mut output).unwrap();

    let context = Context::new();
    let vm_classes = vec![
        stdlib::generate_object_class(),
        stdlib::generate_array_8_class(),
        stdlib::generate_array_16_class(),
        stdlib::generate_array_32_class(),
        stdlib::generate_array_64_class(),
        stdlib::generate_array_object_class(),
        stdlib::generate_array_f32_class(),
        stdlib::generate_array_f64_class(),
        stdlib::generate_printer_class()];

    let mut class_map = HashMap::new();
    let mut string_map = HashMap::new();

    let mut pre_class_table = Vec::new();
    let mut vtables_map = HashMap::new(); 

    context.link_vm_classes(vm_classes, &mut pre_class_table, &mut vtables_map, &mut string_map, &mut class_map);

    let class = ClassFile::new(&output);

    let classes = vec![class];

    let (main_symbol, ready_symbol) = context.link_classes(classes, &mut pre_class_table, &mut vtables_map, &mut string_map, &mut class_map);

    context.finish_linking_classes(pre_class_table);

    let main_object_ref = context.new_object(main_symbol);
    let main_object = context.get_object(main_object_ref);
    let main_object = unsafe { main_object.as_ref().unwrap() };

    /*println!("[Main] {}", context.get_class_name(12));

    let class = context.get_class(main_object.class);
    println!("[Main] Class: {:?}", unsafe {class.read()});*/

    let method = context.get_method(main_object.class, 1, None, ready_symbol);
    let method = unsafe { std::mem::transmute::<_, fn(u64)>(method) };
    method(main_object_ref);

}
