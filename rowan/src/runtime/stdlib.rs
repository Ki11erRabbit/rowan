use super::{class::TypeTag, Reference};


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
    pub methods: Vec<VMMethod>
}

impl VMVTable {
    pub fn new(class: &'static str, methods: Vec<VMMethod>) -> Self {
        VMVTable {
            class,
            methods
        }
    }
}

pub struct VMMethod {
    pub name: &'static str,
    pub fn_pointer: *const u8,
    pub signature: Vec<TypeTag>,
}

impl VMMethod {
    pub fn new(name: &'static str, fn_pointer: *const u8, signature: Vec<TypeTag>) -> Self {
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
    pub arguments: Vec<TypeTag>
}

impl VMSignal {
    pub fn new(name: &'static str, arguments: Vec<TypeTag>) -> Self {
        VMSignal {
            name,
            arguments
        }
    }
}



pub fn generate_object_class() -> VMClass {
    let vtable = VMVTable::new(
        "Object",
        vec![
            VMMethod::new(
                "object/tick",
                object_tick as *const u8,
                vec![TypeTag::Void, TypeTag::Object, TypeTag::F64]
                ),
            VMMethod::new(
                "object/ready",
                object_ready as *const u8,
                vec![TypeTag::Void, TypeTag::Object]
                ),
            VMMethod::new(
                "object/upcast",
                object_upcast as *const u8,
                vec![TypeTag::Object, TypeTag::Object]
                ),
            VMMethod::new(
                "object/get-child",
                object_get_child as *const u8,
                vec![TypeTag::Object, TypeTag::Object, TypeTag::U64]
                ),
            VMMethod::new(
                "object/remove-child",
                object_remove_child as *const u8,
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
