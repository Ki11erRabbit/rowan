use std::collections::HashMap;
use either::Either;
use itertools::Itertools;
use crate::trees::ir::{Class, Expression, File, IfExpression, Import, Literal, Method, Parameter, ParentDec, Statement, TopLevelStatement, Trait, TraitImpl};
use crate::trees::{PathName, Span, Text, Type};

pub struct SpecializeGenerics {
    imports_to_change: HashMap<String, Vec<String>>,
    current_type_argument: HashMap<String, Type<'static>>,
    functions_to_change: HashMap<Vec<String>, Vec<(String, Type<'static>)>>,
}

impl SpecializeGenerics {
    pub fn new() -> SpecializeGenerics {
        Self {
            imports_to_change: HashMap::new(),
            current_type_argument: HashMap::new(),
            functions_to_change: HashMap::new(),
        }
    }

    fn clear_type_arguments(&mut self) {
        self.current_type_argument.clear();
    }

    pub fn specialize_generics<'special>(&mut self, files: Vec<File<'special>>) -> Vec<File<'special>> {
        self.load_definitions(&files);

        let mut output_files: Vec<File> = Vec::new();

        for file in files {
            let result = self.specialize_code(file);
            output_files.push(result);
        }

        output_files
    }

    fn load_definitions<'special>(&mut self, files: &Vec<File<'special>>) {
        for file in files {
            let File {
                path,
                content
            } = file;
            let path = path.segments.iter().map(ToString::to_string).collect();
            for stmt in content {
                self.load_definition(&path, stmt);
            }
        }
    }

    fn load_definition(&mut self, path: &Vec<String>, stmt: &TopLevelStatement) {
        match stmt {
            TopLevelStatement::Class(class) => {
                self.load_class(path, class);
            }
            TopLevelStatement::Trait(r#trait) => {
                self.load_trait(path, r#trait);
            }
            TopLevelStatement::TraitImpl(r#impl) => {
                self.load_impl(path, r#impl);
            }
            _ => {}
        }
    }

    /// TODO: go through methods and fill in functions_to_change
    fn load_class(&mut self, path: &Vec<String>, class: &Class) {
        let Class {
            name,
            type_params,
            ..
        } = class;

        if type_params.is_empty() {
            return
        }

        let permutations = vec![
            Type::I8,
            Type::I16,
            Type::I32,
            Type::I64,
            Type::F32,
            Type::F64,
            Type::Object(Text::Borrowed(""), Span::new(0,0)),
        ].into_iter().permutations(type_params.len()).collect::<Vec<_>>();

        for permutation in permutations {
            let mut modifier_string = String::new();
            for ty in permutation {
                match ty {
                    Type::I8 => modifier_string.push_str("8"),
                    Type::I16 => modifier_string.push_str("16"),
                    Type::I32 => modifier_string.push_str("32"),
                    Type::I64 => modifier_string.push_str("64"),
                    Type::F32 => modifier_string.push_str("f32"),
                    Type::F64 => modifier_string.push_str("f64"),
                    Type::Object(..) => modifier_string.push_str("object"),
                    _ => unreachable!("bizzare posible type"),
                }
            }
            let mut import = path.clone();
            import.push(name.to_string());
            let import = import.join("::");

            let mut new_path = path.clone();
            new_path.push(format!("{name}{modifier_string}"));
            let new_path = new_path.join("::");

            self.imports_to_change.entry(import)
                .and_modify(|imports| imports.push(new_path.clone()))
                .or_insert(vec![new_path]);
        }
    }

    fn load_trait(&mut self, path: &Vec<String>, r#trait: &Trait) {
        let Trait {
            name,
            type_params,
            ..
        } = r#trait;

        if type_params.is_empty() {
            return
        }

        let permutations = vec![
            Type::I8,
            Type::I16,
            Type::I32,
            Type::I64,
            Type::F32,
            Type::F64,
            Type::Object(Text::Borrowed(""), Span::new(0,0)),
        ].into_iter().permutations(type_params.len()).collect::<Vec<_>>();

        for permutation in permutations {
            let mut modifier_string = String::new();
            for ty in permutation {
                match ty {
                    Type::I8 => modifier_string.push_str("8"),
                    Type::I16 => modifier_string.push_str("16"),
                    Type::I32 => modifier_string.push_str("32"),
                    Type::I64 => modifier_string.push_str("64"),
                    Type::F32 => modifier_string.push_str("f32"),
                    Type::F64 => modifier_string.push_str("f64"),
                    Type::Object(..) => modifier_string.push_str("object"),
                    _ => unreachable!("bizzare posible type"),
                }
            }
            let mut import = path.clone();
            import.push(name.to_string());
            let import = import.join("::");

            let mut new_path = path.clone();
            new_path.push(format!("{name}{modifier_string}"));
            let new_path = new_path.join("::");

            self.imports_to_change.entry(import)
                .and_modify(|imports| imports.push(new_path.clone()))
                .or_insert(vec![new_path]);
        }
    }

    fn load_impl(&mut self, _: &Vec<String>, _: &TraitImpl) {
    }

    fn specialize_code<'special>(&mut self, file: File<'special>) -> File<'special> {
        let File {
            path,
            content,
        } = file;

        let file_path = path.segments.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let mut new_content = Vec::with_capacity(content.len());
        for stmt in content {
            match stmt {
                TopLevelStatement::Import(import) => {
                    let Import {
                        path,
                        span,
                    } = import;

                    let Some(modified_imports) = self.imports_to_change.get(&path.segments.join("::")) else {
                        new_content.push(TopLevelStatement::Import(Import::new(path, span)));
                        continue
                    };
                    let modified_imports = modified_imports.clone();
                    modified_imports.iter().for_each(|import| {
                        let new_path = import.split("::")
                            .map(|s| Text::Owned(s.to_string()))
                            .collect::<Vec<_>>();
                        let path = PathName::new(new_path, path.span);
                        new_content.push(TopLevelStatement::Import(Import::new(path, span)));
                    });
                }
                TopLevelStatement::Class(class) => {
                    let results = self.specialize_class(&file_path, class);
                    new_content.extend(results.into_iter().map(TopLevelStatement::Class));
                }
                TopLevelStatement::Trait(r#trait) => {
                    let results = self.specialize_trait(&file_path, r#trait);
                    new_content.extend(results.into_iter().map(TopLevelStatement::Trait))
                }
                TopLevelStatement::TraitImpl(r#impl) => {
                    let results = self.specialize_impl(&file_path, r#impl);
                    new_content.extend(results.into_iter().map(TopLevelStatement::TraitImpl))
                }
            }
        }

        File {
            path,
            content: new_content,
        }
    }

    fn specialize_class<'special>(&mut self, path: &Vec<String>, class: Class<'special>) -> Vec<Class<'special>> {
        let type_parameters = &class.type_params;

        if type_parameters.is_empty() {
            return vec![class];
        }

        let typ_param_names = type_parameters.iter()
            .map(|t| t.name.to_string())
            .collect::<Vec<_>>();

        let permutations = vec![
            Type::I8,
            Type::I16,
            Type::I32,
            Type::I64,
            Type::F32,
            Type::F64,
            Type::Object(Text::Borrowed(""), Span::new(0,0)),
        ].into_iter().permutations(type_parameters.len()).collect::<Vec<_>>();

        let mut output = Vec::new();
        for permutation in permutations {
            let mut modifier_string = String::new();
            for (name, ty) in typ_param_names.iter().zip(permutation.into_iter()) {
                match ty {
                    Type::I8 => {
                        self.current_type_argument.insert(name.to_string(), Type::I8);
                        modifier_string.push_str("8");
                    },
                    Type::I16 => {
                        self.current_type_argument.insert(name.to_string(), Type::I16);
                        modifier_string.push_str("16");
                    },
                    Type::I32 => {
                        self.current_type_argument.insert(name.to_string(), Type::I32);
                        modifier_string.push_str("32");
                    },
                    Type::I64 => {
                        self.current_type_argument.insert(name.to_string(), Type::I64);
                        modifier_string.push_str("64");
                    },
                    Type::F32 => {
                        self.current_type_argument.insert(name.to_string(), Type::F32);
                        modifier_string.push_str("f32");
                    },
                    Type::F64 => {
                        self.current_type_argument.insert(name.to_string(), Type::F64);
                        modifier_string.push_str("f64");
                    },
                    Type::Object(..) => {
                        self.current_type_argument.insert(name.to_string(), Type::Object(Text::Borrowed(""), Span::new(0,0)));
                        modifier_string.push_str("object");
                    },
                    _ => unreachable!("bizzare posible type"),
                }
            }
            output.push(self.specialize_class_inner(path, &class, modifier_string));
            self.clear_type_arguments();
        }
        output
    }

    fn specialize_class_inner<'special>(&mut self, path: &Vec<String>, class: &Class<'special>, name_mod: String) -> Class<'special> {
        let Class {
            name,
            parent,
            mut members,
            methods,
            mut static_members,
            span,
            ..
        } = class.clone();


        let name = Text::Owned(format!("{name}{name_mod}"));
        let parent = parent.map(|p| {
            let ParentDec {
                name,
                type_args,
                type_params,
                span
            } = p;

            let mut modifier_string = String::new();
            for arg in type_args {
                match arg {
                    Type::U8 | Type::I8 => {
                        modifier_string.push_str("8");
                    }
                    Type::U16 | Type::I16 => {
                        modifier_string.push_str("16");
                    }
                    Type::U32 | Type::I32 => {
                        modifier_string.push_str("32");
                    }
                    Type::U64 | Type::I64 => {
                        modifier_string.push_str("64");
                    }
                    Type::F32 => {
                        modifier_string.push_str("f32");
                    }
                    Type::F64 => {
                        modifier_string.push_str("f64");
                    }
                    Type::Object(value, ..) => {
                        if let Some(ty) = self.current_type_argument.get(value.as_str()) {
                            match ty {
                                Type::I8 => {
                                    modifier_string.push_str("8");
                                }
                                Type::I16 => {
                                    modifier_string.push_str("16");
                                }
                                Type::I32 => {
                                    modifier_string.push_str("32");
                                }
                                Type::I64 => {
                                    modifier_string.push_str("64");
                                }
                                Type::F32 => {
                                    modifier_string.push_str("f32");
                                }
                                Type::F64 => {
                                    modifier_string.push_str("f64");
                                }
                                Type::Object(..) => {
                                    modifier_string.push_str("object");
                                }
                                _ => unreachable!("bizzare posible type"),
                            }
                        } else {
                            modifier_string.push_str("object");
                        }
                    }
                    _ => unreachable!("bizzare posible type"),
                }
            }

            let name = Text::Owned(format!("{name}{modifier_string}"));

            ParentDec {
                name,
                type_args: Vec::new(),
                type_params: Vec::new(),
                span
            }

        });

        for member in &mut members {
            match &member.ty {
                Type::Object(name, _) => {
                    if let Some(ty) = self.current_type_argument.get(name.as_str()) {
                        member.ty = ty.clone();
                    }
                }
                _ => {}
            }
        }

        let mut new_methods = Vec::new();
        for method in methods {
            let mut methods = self.specialize_method(path, method);
            new_methods.append(&mut methods);
        }

        let methods = new_methods;

        for static_member in &mut static_members {
            self.specialize_type(&mut static_member.ty);
            static_member.value.as_mut().map(|v| self.specialize_expression(path, v));
        }

        Class {
            name,
            parent,
            members,
            methods,
            static_members,
            type_params: Vec::new(),
            span
        }
    }

    fn specialize_trait<'special>(&mut self, path: &Vec<String>, r#trait: Trait<'special>) -> Vec<Trait<'special>> {
        let type_parameters = &r#trait.type_params;

        if type_parameters.is_empty() {
            return vec![r#trait];
        }

        let typ_param_names = type_parameters.iter()
            .map(|t| t.name.to_string())
            .collect::<Vec<_>>();

        let permutations = vec![
            Type::I8,
            Type::I16,
            Type::I32,
            Type::I64,
            Type::F32,
            Type::F64,
            Type::Object(Text::Borrowed(""), Span::new(0,0)),
        ].into_iter().permutations(type_parameters.len()).collect::<Vec<_>>();

        let mut output = Vec::new();
        for permutation in permutations {
            let mut modifier_string = String::new();
            for (name, ty) in typ_param_names.iter().zip(permutation.into_iter()) {
                match ty {
                    Type::I8 => {
                        self.current_type_argument.insert(name.to_string(), Type::I8);
                        modifier_string.push_str("8");
                    },
                    Type::I16 => {
                        self.current_type_argument.insert(name.to_string(), Type::I16);
                        modifier_string.push_str("16");
                    },
                    Type::I32 => {
                        self.current_type_argument.insert(name.to_string(), Type::I32);
                        modifier_string.push_str("32");
                    },
                    Type::I64 => {
                        self.current_type_argument.insert(name.to_string(), Type::I64);
                        modifier_string.push_str("64");
                    },
                    Type::F32 => {
                        self.current_type_argument.insert(name.to_string(), Type::F32);
                        modifier_string.push_str("f32");
                    },
                    Type::F64 => {
                        self.current_type_argument.insert(name.to_string(), Type::F64);
                        modifier_string.push_str("f64");
                    },
                    Type::Object(..) => {
                        self.current_type_argument.insert(name.to_string(), Type::Object(Text::Borrowed(""), Span::new(0,0)));
                        modifier_string.push_str("object");
                    },
                    _ => unreachable!("bizzare posible type"),
                }
            }

            output.push(self.specialize_trait_inner(path, &r#trait, modifier_string));
            self.clear_type_arguments();
        }
        output
    }

    fn specialize_trait_inner<'special>(&mut self, path: &Vec<String>, r#trait: &Trait<'special>, modifier: String) -> Trait<'special> {
        let Trait {
            name,
            mut parents,
            methods,
            span,
            ..
        } = r#trait.clone();

        let name = Text::Owned(format!("{name}{modifier}"));
        for parent in &mut parents {
            match parent {
                Type::Object(..) => {}
                Type::TypeArg(name,args,span) => {
                    let Type::Object(name, _) = name.as_ref() else {
                        unreachable!("only object is allowed here")
                    };
                    let mut modifier_string = String::new();
                    for arg in args {
                        match arg {
                            Type::U8 | Type::I8 => {
                                modifier_string.push_str("8");
                            }
                            Type::U16 | Type::I16 => {
                                modifier_string.push_str("16");
                            }
                            Type::U32 | Type::I32 => {
                                modifier_string.push_str("32");
                            }
                            Type::U64 | Type::I64 => {
                                modifier_string.push_str("64");
                            }
                            Type::F32 => {
                                modifier_string.push_str("f32");
                            }
                            Type::F64 => {
                                modifier_string.push_str("f64");
                            }
                            Type::Object(name, ..) => {
                                if let Some(ty) = self.current_type_argument.get(name.as_str()) {
                                    match ty {
                                        Type::I8 => {
                                            modifier_string.push_str("8");
                                        },
                                        Type::I16 => {
                                            modifier_string.push_str("16");
                                        },
                                        Type::I32 => {
                                            modifier_string.push_str("32");
                                        },
                                        Type::I64 => {
                                            modifier_string.push_str("64");
                                        },
                                        Type::F32 => {
                                            modifier_string.push_str("f32");
                                        },
                                        Type::F64 => {
                                            modifier_string.push_str("f64");
                                        },
                                        Type::Object(..) => {
                                            modifier_string.push_str("object");
                                        },
                                        _ => unreachable!("bizzare posible type"),
                                    }
                                } else {
                                    modifier_string.push_str("object");
                                }
                            }
                            _ => modifier_string.push_str("object"),
                        }
                    }
                    let name = Text::Owned(format!("{name}{modifier_string}"));
                    *parent = Type::Object(name, *span);
                }
                _ => unreachable!("bizzare posible type"),
            }
        }


        let mut new_methods = Vec::new();
        for method in methods {
            let mut methods = self.specialize_method(path, method);
            new_methods.append(&mut methods);
        }

        let methods = new_methods;

        Trait {
            name,
            parents,
            methods,
            span,
            type_params: Vec::new(),
        }

    }

    fn specialize_impl<'special>(&mut self, path: &Vec<String>, r#impl: TraitImpl<'special>) -> Vec<TraitImpl<'special>> {
        let type_parameters = &r#impl.type_params;

        if type_parameters.is_empty() {
            return vec![r#impl];
        }

        let typ_param_names = type_parameters.iter()
            .map(|t| t.name.to_string())
            .collect::<Vec<_>>();

        let permutations = vec![
            Type::I8,
            Type::I16,
            Type::I32,
            Type::I64,
            Type::F32,
            Type::F64,
            Type::Object(Text::Borrowed(""), Span::new(0,0)),
        ].into_iter().permutations(type_parameters.len()).collect::<Vec<_>>();

        let mut output = Vec::new();
        for permutation in permutations {
            let mut modifier_string = String::new();
            for (name, ty) in typ_param_names.iter().zip(permutation.into_iter()) {
                match ty {
                    Type::I8 => {
                        self.current_type_argument.insert(name.to_string(), Type::I8);
                    },
                    Type::I16 => {
                        self.current_type_argument.insert(name.to_string(), Type::I16);
                    },
                    Type::I32 => {
                        self.current_type_argument.insert(name.to_string(), Type::I32);
                    },
                    Type::I64 => {
                        self.current_type_argument.insert(name.to_string(), Type::I64);
                    },
                    Type::F32 => {
                        self.current_type_argument.insert(name.to_string(), Type::F32);
                    },
                    Type::F64 => {
                        self.current_type_argument.insert(name.to_string(), Type::F64);
                    },
                    Type::Object(..) => {
                        self.current_type_argument.insert(name.to_string(), Type::Object(Text::Borrowed(""), Span::new(0,0)));
                    },
                    _ => unreachable!("bizzare posible type"),
                }
            }

            output.push(self.specialize_impl_inner(path, &r#impl));
            self.clear_type_arguments();
        }
        output
    }

    fn specialize_impl_inner<'special>(&mut self, path: &Vec<String>, r#impl: &TraitImpl<'special>) -> TraitImpl<'special> {
        let TraitImpl {
            mut r#trait,
            mut implementer,
            methods,
            span,
            ..
        } = r#impl.clone();

        match &mut r#trait {
            Type::Object(..) => {}
            Type::TypeArg(name, args, span) => {
                let Type::Object(name, _) = name.as_ref() else {
                    unreachable!("can only be object type")
                };
                let mut modifier_string = String::new();
                for arg in args {
                    match arg {
                        Type::U8 | Type::I8 => {
                            modifier_string.push_str("8");
                        }
                        Type::U16 | Type::I16 => {
                            modifier_string.push_str("16");
                        }
                        Type::U32 | Type::I32 => {
                            modifier_string.push_str("32");
                        }
                        Type::U64 | Type::I64 => {
                            modifier_string.push_str("64");
                        }
                        Type::F32 => {
                            modifier_string.push_str("f32");
                        }
                        Type::F64 => {
                            modifier_string.push_str("f64");
                        }
                        Type::Object(name, ..) => {
                            if let Some(ty) = self.current_type_argument.get(name.as_str()) {
                                match arg {
                                    Type::I8 => {
                                        modifier_string.push_str("8");
                                    }
                                    Type::I16 => {
                                        modifier_string.push_str("16");
                                    }
                                    Type::I32 => {
                                        modifier_string.push_str("32");
                                    }
                                    Type::I64 => {
                                        modifier_string.push_str("64");
                                    }
                                    Type::F32 => {
                                        modifier_string.push_str("f32");
                                    }
                                    Type::F64 => {
                                        modifier_string.push_str("f64");
                                    }
                                    Type::Object(name, ..) => {
                                        modifier_string.push_str("object");
                                    }
                                    _ => modifier_string.push_str("object"),
                                }
                            } else {
                                modifier_string.push_str("object");
                            }
                        }
                        _ => modifier_string.push_str("object"),
                    }
                }
                let name = Text::Owned(format!("{name}{modifier_string}"));
                r#trait = Type::Object(name, *span);
            }
            _ => unreachable!("bizzare posible type"),
        }

        match &mut implementer {
            Type::Object(..) => {}
            Type::TypeArg(name, args, span) => {
                let Type::Object(name, _) = name.as_ref() else {
                    unreachable!("can only be object type")
                };
                let mut modifier_string = String::new();
                for arg in args {
                    match arg {
                        Type::U8 | Type::I8 => {
                            modifier_string.push_str("8");
                        }
                        Type::U16 | Type::I16 => {
                            modifier_string.push_str("16");
                        }
                        Type::U32 | Type::I32 => {
                            modifier_string.push_str("32");
                        }
                        Type::U64 | Type::I64 => {
                            modifier_string.push_str("64");
                        }
                        Type::F32 => {
                            modifier_string.push_str("f32");
                        }
                        Type::F64 => {
                            modifier_string.push_str("f64");
                        }
                        Type::Object(name, ..) => {
                            if let Some(ty) = self.current_type_argument.get(name.as_str()) {
                                match arg {
                                    Type::I8 => {
                                        modifier_string.push_str("8");
                                    }
                                    Type::I16 => {
                                        modifier_string.push_str("16");
                                    }
                                    Type::I32 => {
                                        modifier_string.push_str("32");
                                    }
                                    Type::I64 => {
                                        modifier_string.push_str("64");
                                    }
                                    Type::F32 => {
                                        modifier_string.push_str("f32");
                                    }
                                    Type::F64 => {
                                        modifier_string.push_str("f64");
                                    }
                                    Type::Object(name, ..) => {
                                        modifier_string.push_str("object");
                                    }
                                    _ => modifier_string.push_str("object"),
                                }
                            } else {
                                modifier_string.push_str("object");
                            }
                        }
                        _ => modifier_string.push_str("object"),
                    }
                }
                let name = Text::Owned(format!("{name}{modifier_string}"));
                implementer = Type::Object(name, *span);
            }
            _ => unreachable!("bizzare posible type"),
        }

        let mut new_methods = Vec::new();
        for method in methods {
            let mut methods = self.specialize_method(path, method);
            new_methods.append(&mut methods);
        }
        let methods = new_methods;

        TraitImpl {
            r#trait,
            implementer,
            methods,
            type_params: Vec::new(),
            span,
        }
    }


    /// TODO: add in expanding of methods if they have their own type parameters
    fn specialize_method<'special>(&mut self, path: &Vec<String>, method: Method<'special>) -> Vec<Method<'special>> {
        let Method {
            name,
            is_native,
            annotations,
            visibility,
            type_params,
            mut parameters,
            mut return_type,
            mut body,
            span
        } = method;

        match &return_type {
            Type::Object(name, _) => {
                if let Some(ty) = self.current_type_argument.get(name.as_str()) {
                    return_type = ty.clone();
                }
            }
            _ => {}
        }

        for parameter in &mut parameters {
            match parameter {
                Parameter::Pattern {
                    ty, ..
                } => {
                    self.specialize_type(ty);
                }
                _ => {}
            }
        }

        self.specialize_body(path, &mut body);

        vec![
            Method {
                name,
                is_native,
                annotations,
                visibility,
                type_params,
                parameters,
                return_type,
                body,
                span
            }
        ]

    }

    fn specialize_body<'special>(&mut self, path: &Vec<String>, body: &mut Vec<Statement<'special>>) {
        for stmt in body {
            self.specialize_statement(path, stmt);
        }
    }

    fn specialize_statement<'special>(&mut self, path: &Vec<String>, stmt: &mut Statement<'special>) {
        match stmt {
            Statement::Let {
                ty,
                value,
                ..
            } => {
                self.specialize_type(ty);
                self.specialize_expression(path, value);
            }
            Statement::Const {
                ty,
                value,
                ..
            } => {
                self.specialize_type(ty);
                self.specialize_expression(path, value);
            }
            Statement::Expression(expr, _) => {
                self.specialize_expression(path, expr);
            }
            Statement::Assignment {
                target,
                value,
                ..
            } => {
                self.specialize_expression(path, target);
                self.specialize_expression(path, value);
            }
            Statement::While {
                test,
                body,
                ..
            } => {
                self.specialize_expression(path, test);
                self.specialize_body(path, body);
            }
            x => todo!("specialize statement: {:?}", x),
        }
    }

    fn specialize_expression<'special>(&mut self, path: &Vec<String>, expr: &mut Expression<'special>) {
        match expr {
            Expression::Variable(_, ty, _) => {
                self.specialize_type(ty);
            }
            Expression::Literal(Literal::Array(exprs, ty, _)) => {
                for expr in exprs {
                    self.specialize_expression(path, expr);
                }
                self.specialize_type(ty);
            }
            Expression::Literal(Literal::Tuple(exprs, ty, _)) => {
                for expr in exprs {
                    self.specialize_expression(path, expr);
                }
                self.specialize_type(ty);
            }
            Expression::Call {
                args,
                annotation,
                ..
            } => {
                // TODO: look at type args and functions that have generic parameters
                for arg in args {
                    self.specialize_expression(path, arg);
                }
                self.specialize_type(annotation);
            }
            Expression::StaticCall {
                args,
                annotation,
                ..
            } => {
                for arg in args {
                    self.specialize_expression(path, arg);
                }
                self.specialize_type(annotation);
            }
            Expression::MemberAccess {
                object,
                annotation,
                ..
            } => {
                self.specialize_expression(path, object.as_mut());
                self.specialize_type(annotation);
            }
            Expression::Closure {
                params,
                return_type,
                captures,
                ..
            } => {
                for param in params {
                    match &mut param.parameter {
                        Parameter::Pattern { ty, .. } => {
                            self.specialize_type(ty);
                        }
                        _ => {}
                    }
                }
                self.specialize_type(return_type);
                for (_, ty) in captures {
                    self.specialize_type(ty);
                }
            }
            Expression::Parenthesized(expr, _) => {
                self.specialize_expression(path, expr.as_mut());
            }
            Expression::IfExpression(if_expr, ..) => {
                self.specialize_if(path, if_expr);
            }
            Expression::UnaryOperation {
                operand,
                ..
            } => {
                self.specialize_expression(path, operand.as_mut());
            }
            Expression::BinaryOperation {
                left,
                right,
                ..
            } => {
                self.specialize_expression(path, left.as_mut());
                self.specialize_expression(path, right.as_mut());
            }
            Expression::Return(ret, _) => {
                ret.as_mut().map(|expr| self.specialize_expression(path, expr.as_mut()));
            }
            Expression::New(ty, expr, _) => {
                self.specialize_type(ty);
                expr.as_mut().map(|expr| self.specialize_expression(path, expr.as_mut()));
            }
            Expression::As {
                source,
                typ,
                ..
            } => {
                self.specialize_type(typ);
                self.specialize_expression(path, source.as_mut());
            }
            x => todo!("complete specializing generics for: {x:?}"),
        }
    }

    fn specialize_if<'special>(&mut self, path: &Vec<String>, expr: &mut IfExpression<'special>) {
        let IfExpression {
            condition,
            then_branch,
            else_branch,
            ..
        } = expr;

        self.specialize_expression(path, condition);
        self.specialize_body(path, then_branch);
        match else_branch {
            Some(Either::Left(elif)) => {
                self.specialize_if(path, elif.as_mut());
            }
            Some(Either::Right(else_branch)) => {
                self.specialize_body(path, else_branch);
            }
            None => {}
        }
    }

    fn specialize_type(&mut self, ty: &mut Type) {
        match ty {
            Type::Object(name, _) => {
                if let Some(replacement) = self.current_type_argument.get(name.as_str())  {
                    *ty = replacement.clone();
                }
            }
            Type::Array(ty, ..) => {
                self.specialize_type(ty.as_mut());
            }
            Type::TypeArg(_, args, ..) => {
                for arg in args {
                    self.specialize_type(arg);
                }
            }
            Type::Function(args, ret, ..) => {
                for arg in args {
                    self.specialize_type(arg);
                }
                self.specialize_type(ret.as_mut());
            }
            Type::Tuple(values, ..) => {
                for value in values {
                    self.specialize_type(value);
                }
            }
            _ => {}
        }
    }
}