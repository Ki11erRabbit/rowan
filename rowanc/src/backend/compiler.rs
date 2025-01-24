use std::collections::HashMap;
use crate::backend::compiler_utils::{Frame, ClassInfo};


pub struct Compiler {
    scopes: Vec<Frame>,
    classes: HashMap<String, ClassInfo>
}


