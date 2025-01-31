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
    let vm_classes = vec![stdlib::generate_object_class(), stdlib::generate_printer_class()];

    let mut class_map = HashMap::new();
    let mut string_map = HashMap::new();

    let mut pre_class_table = Vec::new();
    let mut vtables_map = HashMap::new(); 

    context.link_vm_classes(vm_classes, &mut pre_class_table, &mut vtables_map, &mut string_map, &mut class_map);

    let class = ClassFile::new(&output);

    let classes = vec![class];

    context.link_classes(classes, &mut pre_class_table, &mut vtables_map, &mut string_map, &mut class_map);
    

}
