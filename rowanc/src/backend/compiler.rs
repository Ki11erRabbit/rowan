use std::collections::HashMap;
use rowan_shared::{classfile::{SignatureEntry, VTable, VTableEntry}, TypeTag};

use crate::{ast::{Class, File, Method, Parameter, TopLevelStatement, Type}, backend::compiler_utils::Frame};

use super::compiler_utils::PartialClass;


pub enum CompilerError {

}


pub struct Compiler {
    scopes: Vec<Frame>,
    classes: HashMap<String, PartialClass>
}


impl Compiler {

    pub fn new() -> Compiler {
        Compiler {
            scopes: Vec::new(),
            classes: HashMap::new(),
        }
    }

    /// files should be sorted in a way that means that means we don't need to do each file incrementally
    pub fn compile_files(&mut self, files: Vec<File>) -> Result<(), CompilerError> {

        for file in files {
            let File { content, .. } = file;

            for statement in content {
                let TopLevelStatement::Class(class) = statement else {
                    unreachable!("Non classes should have been removed by this point");
                };

                self.compile_class(class)?;
            }
        }


        Ok(())
    }

    fn compile_class(&mut self, class: Class) -> Result<(), CompilerError> {
        let Class {
            name,
            parents,
            members,
            methods,
            signals,
            span,
            ..
        } = class;

        let mut partial_class = PartialClass::new();
        partial_class.set_name(name);

        let parent_vtables = parents.iter().map(|parent_name| {
            let partial_class = self.classes.get(parent_name.name).expect("Order of files is wrong");

            (parent_name.name, partial_class.get_vtable(parent_name.name))

        }).collect::<Vec<_>>();
        parents.iter().for_each(|parent| {
            partial_class.add_parent(parent.name);
        });

        let (vtable, class_names, sub_class_names, names, responds_to, signatures) = self.construct_vtable(name, &methods)?;

        partial_class.add_vtable(name, vtable, class_names, sub_class_names, names, responds_to, signatures);

        for (class_name, (vtable, class_names, sub_class_names, names, responds_to, signatures)) in parent_vtables {
            let class_names = class_names.into_iter().map(|s| s.as_str()).collect::<Vec<_>>();
            let sub_class_names = sub_class_names.into_iter().map(|s| s.as_str()).collect::<Vec<_>>();
            let names = names.into_iter().map(|s| s.as_str()).collect::<Vec<_>>();
            let responds_to = responds_to.into_iter().map(|s| s.as_str()).collect::<Vec<_>>();


            partial_class.add_vtable(class_name, vtable, class_names, sub_class_names, names, responds_to, signatures);
        }

        Ok(())
    }

    fn construct_vtable<'a>(&'a self, class_name: &str, methods: &'a Vec<Method>) -> Result<(
        VTable,
        Vec<&'a str>,
        Vec<&'a str>,
        Vec<&'a str>,
        Vec<&'a str>,
        Vec<SignatureEntry>), CompilerError> {


        let mut entries = Vec::new();
        let mut class_names = Vec::new();
        let mut sub_class_names = Vec::new();
        let mut names = Vec::new();
        let mut responds_to = Vec::new();
        let mut signatures = Vec::new();

        'methods: for method in methods.iter() {
            let Method {
                name,
                annotations,
                parameters,
                return_type,
                span: _span,
                ..
            } = method;

            for annotation in annotations.iter() {
                if annotation.name == "Override" {
                    continue 'methods;
                }
                if annotation.name == "RespondsTo" {
                    responds_to.push(&annotation.parameters[0]);
                } else {
                    responds_to.push("");
                }
            }

            names.push(name);
            class_names.push(class_name);
            sub_class_names.push(class_name);

            entries.push(VTableEntry::default());

            let mut signature = Vec::new();
            signature.push(self.convert_type(return_type));

            parameters.iter().for_each(|param| {
                match param {
                    Parameter::This(_, _) => {
                        signature.push(TypeTag::Object);
                    }
                    Parameter::Pattern { ty, .. } => {
                        signature.push(self.convert_type(ty));
                    }
                }

            });
            signatures.push(SignatureEntry::new(signature));

        }
        let vtable = VTable::new(entries);


        Ok((vtable, class_names, sub_class_names, names, responds_to, signatures))
    }

    fn convert_type(&self, ty: &Type) -> TypeTag {
        match ty {
            Type::Void => TypeTag::Void,
            Type::U8 => TypeTag::U8,
            Type::U16 => TypeTag::U16,
            Type::U32 => TypeTag::U32,
            Type::U64 => TypeTag::U64,
            Type::I8 => TypeTag::I8,
            Type::I16 => TypeTag::I16,
            Type::I32 => TypeTag::I32,
            Type::I64 => TypeTag::I64,
            Type::Char => TypeTag::U32,
            Type::Str => TypeTag::Str,
            Type::F32 => TypeTag::F32,
            Type::F64 => TypeTag::F64,
            Type::Array(_, _) => TypeTag::Object,
            Type::Object(_, _) => TypeTag::Object,
            Type::TypeArg(_, _, _) => TypeTag::Object,
            Type::Tuple(_, _) => TypeTag::Object,
            Type::Function(_, _, _) => TypeTag::Object,
        }

    }
}
