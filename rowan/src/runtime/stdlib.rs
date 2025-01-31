use super::Reference;
use rowan_shared::TypeTag;


/// This represents a class in the Virtual Machine.
pub struct VMClass {
    pub name: &'static str,
    pub parents: Vec<&'static str>,
    pub vtables: Vec<VMVTable>,
    pub members: Vec<VMMember>,
    pub signals: Vec<VMSignal>,
}

impl VMClass {
    pub fn new(
        name: &'static str,
        parents: Vec<&'static str>,
        vtables: Vec<VMVTable>,
        members: Vec<VMMember>,
        signals: Vec<VMSignal>,
    ) -> Self {
        VMClass {
            name,
            parents,
            vtables,
            members,
            signals,
        }
    }
}


pub struct VMVTable {
    pub class: &'static str,
    pub source_class: Option<&'static str>,
    pub methods: Vec<VMMethod>
}

impl VMVTable {
    pub fn new(class: &'static str, source_class: Option<&'static str>, methods: Vec<VMMethod>) -> Self {
        VMVTable {
            class,
            source_class,
            methods
        }
    }
}

pub struct VMMethod {
    pub name: &'static str,
    pub fn_pointer: *const (),
    pub signature: Vec<TypeTag>,
}

impl VMMethod {
    pub fn new(name: &'static str, fn_pointer: *const (), signature: Vec<TypeTag>) -> Self {
        VMMethod {
            name,
            fn_pointer,
            signature
        }
    }
}

pub struct VMMember {
    pub name: &'static str,
    pub ty: TypeTag,
}

impl VMMember {
    pub fn new(name: &'static str, ty: TypeTag) -> Self {
        VMMember {
            name,
            ty
        }
    }
}

pub struct VMSignal {
    pub name: &'static str,
    pub is_static: bool,
    pub arguments: Vec<TypeTag>
}

impl VMSignal {
    pub fn new(name: &'static str, is_static: bool, arguments: Vec<TypeTag>) -> Self {
        VMSignal {
            name,
            is_static,
            arguments
        }
    }
}



pub fn generate_object_class() -> VMClass {
    let vtable = VMVTable::new(
        "Object",
        None,
        vec![
            VMMethod::new(
                "tick",
                object_tick as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::F64]
                ),
            VMMethod::new(
                "ready",
                object_ready as *const (),
                vec![TypeTag::Void, TypeTag::Object]
                ),
            VMMethod::new(
                "upcast",
                object_upcast as *const (),
                vec![TypeTag::Object, TypeTag::Object]
                ),
            VMMethod::new(
                "get-child",
                object_get_child as *const (),
                vec![TypeTag::Object, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "remove-child",
                object_remove_child as *const (),
                vec![TypeTag::Object, TypeTag::Object, TypeTag::Object]
                ),
        ]
    );

    VMClass::new("Object", Vec::new(), vec![vtable], Vec::new(), Vec::new())
}


extern "C" fn object_tick(_: Reference, _: f64) {
    
}

extern "C" fn object_ready(_: Reference) {
    
}

// Possibly change this to take an additional parameter which is a class index
extern "C" fn object_upcast(this: Reference) -> Reference {
    this 
}


extern "C" fn object_get_child(this: Reference, nth: u64) -> Reference {
    todo!("get a context and look up the nth child of that object")
}

extern "C" fn object_remove_child(this: Reference, reference: Reference) -> Reference {
    todo!("get a context and find the child of that object that matches reference")
}

pub fn generate_printer_class() -> VMClass {
    let vtable = VMVTable::new(
        "Printer",
        None,
        vec![
            VMMethod::new(
                "println-int",
                printer_println_int as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "println-float",
                printer_println_float as *const (),
                vec![TypeTag::Void, TypeTag::Object, TypeTag::F64]
                ),
        ]
    );

    VMClass::new("Printer", vec!["Object"], vec![vtable], Vec::new(), Vec::new())
}


extern "C" fn printer_println_int(_: Reference, int: u64) {
    println!("{}", int);
}

extern "C" fn printer_println_float(_: Reference, float: f64) {
    println!("{}", float);
}
