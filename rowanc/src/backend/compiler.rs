use std::collections::HashMap;




pub struct Compiler {
    scopes: Vec<Frame>,
    class_info: HashMap<String, ClassInfo>,

}



struct Frame {
    HashMap<String, VarLocation>,
}

enum VarLocation {
    LocalVar(u8),
    TopOfStack(usize),
}


struct ClassInfo {
    member_position: HashMap<String, u64>,
}
