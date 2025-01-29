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
    let vm_classes = [stdlib::generate_object_class(), stdlib::generate_printer_class()];

    let mut class_map = HashMap::new();
    let mut string_map = HashMap::new();
    
    for class in vm_classes {
        context.link_vm_class(class, &mut string_map, &mut class_map);
    }

    let class = ClassFile::new(&output);

    context.link_class(class, &mut string_map, &mut class_map);
    

}
