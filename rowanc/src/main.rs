use std::cmp::Ordering;
use crate::ast::TopLevelStatement;

pub mod backend;
pub mod parser;
pub mod ast;
pub mod typechecker;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return
    }

    let location = &args[1];
    let mut files = Vec::new();
    
    for file in std::fs::read_dir(location).unwrap() {
        let entry = file.unwrap();

        let contents = std::fs::read_to_string(entry.path()).unwrap();

        let name_path = entry.path().to_str().unwrap().to_string();
        let path = name_path.split("/").skip(1).map(|x|{
            if x.contains(".rowan") {
                x.replace(".rowan", "").to_string()
            } else {
                x.to_string()
            }
        }).collect::<Vec<String>>();

        files.push((name_path, path, contents));
    }



    let mut class_files = Vec::new();
    for (name, path, contents) in files.iter() {
        let file = parser::parse(&name, &contents).unwrap();
        class_files.push((path, file, contents));
    }

    class_files.sort_by(|(pa, a, _), (path, _, _)| {
        //println!("{:?}\n{:?}", pa, path);
        let a_imports = a.content.iter()
            .filter_map(|x| {
                match x {
                    TopLevelStatement::Import(import) => {
                        Some(&import.path.segments)
                    }
                    _ => None
                }
            }).collect::<Vec<_>>();

        for import in &a_imports {
            let mut matching_parts = 0;
            for (import, part) in import.iter().zip(path.iter()) {
                if import.as_str() == part.as_str() {
                    matching_parts += 1;
                } else {
                    break;
                }
            }
            if matching_parts > 0 {
                return Ordering::Greater
            }
        }
        return Ordering::Less
    });

    /*class_files.iter().for_each(|(path, _, _)| {
        println!("{:?}", path);
    });*/

    let class_files = class_files.into_iter().map(|(_, file, _)| file).collect();

    let mut typechecker = typechecker::TypeChecker::new();
    let class_files = typechecker.check(class_files).unwrap();

    let mut compiler = backend::Compiler::new();
    compiler.compile_files(class_files).unwrap();

}
