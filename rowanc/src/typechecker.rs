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

pub struct Frame<'a> {
    variables: HashMap<String, crate::ast::Type<'a>>,
}

pub struct TypeChecker<'a> {
    class_information: HashMap<String, HashMap<String, ClassAttribute<'a>>>,
    scopes: Vec<Frame<'a>>,
}


impl TypeChecker<'_> {
    pub fn new<'a>() -> TypeChecker<'a> {
        TypeChecker {
            class_information: create_stdlib(),
            scopes: Vec::new(),
        }
    }

    pub fn check<'a>(&mut self, files: Vec<crate::ast::File<'a>>) -> Result<Vec<crate::ast::File<'a>>, TypeCheckerError> {
        self.check_files(files)
    }

    fn check_files<'a>(&mut self, mut files: Vec<crate::ast::File<'a>>) -> Result<Vec<crate::ast::File<'a>>, TypeCheckerError> {

        for file in files.iter_mut() {
            self.check_file(file)?;
        }
        

        Ok(files)
    }

    fn check_file<'a>(&mut self, file: &mut crate::ast::File<'a>) -> Result<(), TypeCheckerError> {
        for content in file.content.iter_mut() {
            match content {
                crate::ast::TopLevelStatement::Class(class) => {
                    self.check_class(class)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_class<'a>(&mut self, class: &mut crate::ast::Class<'a>) -> Result<(), TypeCheckerError> {
        let crate::ast::Class { name, members, methods, .. } = class;

        let mut class_attributes = HashMap::new();
        for member in members.iter() {
            let crate::ast::Member { name, ty, .. } = member;
            class_attributes.insert(name.to_string(), ClassAttribute::Member(ty.clone()));
        }

        for method in methods.iter() {
        }

        Ok(())
    }
}

