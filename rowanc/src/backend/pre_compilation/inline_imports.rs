use std::collections::HashMap;

pub struct InlineImports {
    pub imports: HashMap<String, String>,
}


impl InlineImports {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }
}