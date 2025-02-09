use std::collections::HashMap;

use crate::ast::{Span, Type};

fn create_stdlib<'a>() -> HashMap<String, HashMap<String, ClassAttribute<'a>>> {
    let mut info = HashMap::new();
    let mut object_attributes = HashMap::new();
    object_attributes.insert("tick".to_string(), ClassAttribute::Method(Type::Function(vec![Type::F64], Box::new(Type::Void), Span::new(0, 0))));

    info.insert("Object".to_string(), object_attributes);
    
    info
}


#[derive(Debug)]
pub enum TypeCheckerError {

}

pub enum ClassAttribute<'a> {
    Member(Type<'a>),
    Method(Type<'a>),
}

pub struct TypeChecker<'a> {
    class_information: HashMap<String, HashMap<String, ClassAttribute<'a>>>,
}

impl TypeChecker<'_> {
    pub fn new<'a>() -> TypeChecker<'a> {
        TypeChecker {
            class_information: create_stdlib(),
        }
    }
}
