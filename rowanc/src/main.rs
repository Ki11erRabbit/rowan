use std::cmp::Ordering;
use std::path::Path;
use clap::Parser;
use crate::ast::TopLevelStatement;

pub mod backend;
pub mod parser;
pub mod ast;
pub mod typechecker;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg()]
    pub directory: String,

    #[arg(short, long)]
    pub stdlib_path: String,
}

fn explore_directories<P: AsRef<Path>>(path: P, files: &mut Vec<(String, Vec<String>, String)>) {
    let mut dirs_to_explore = Vec::new();
    let dir_path = path.as_ref().to_path_buf();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            dirs_to_explore.push(entry.path());
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap();
        let name = entry.file_name().to_str().unwrap().to_string();
        let path = dir_path.join(name.replace(".rowan", "")).into_iter()
            .map(|p| p.to_str().unwrap().to_string())
        .collect::<Vec<String>>();


        files.push((name, path, content));
    }
    for dir in dirs_to_explore {
        explore_directories(dir, files);
    }
}

fn explore_directories_start<P: AsRef<Path>>(path: P, files: &mut Vec<(String, Vec<String>, String)>) {
    let mut dirs_to_explore = Vec::new();
    let dir_path = path.as_ref().to_path_buf();
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            dirs_to_explore.push(entry.path());
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).unwrap();
        let name = entry.file_name().to_str().unwrap().to_string();
        let path = vec![name.replace(".rowan", "")];


        files.push((name, path, content));
    }
    for dir in dirs_to_explore {
        explore_directories(dir, files);
    }
}

fn main() {

    let args = Args::parse();

    let mut files = Vec::new();

    explore_directories(&args.stdlib_path, &mut files);

    explore_directories_start(&args.directory, &mut files);



    let mut class_files = Vec::new();
    for (name, path, contents) in files.iter() {
        println!("{name}");
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

    class_files.iter().for_each(|(path, _, _)| {
        println!("path: {:?}", path);
    });

    let class_files = class_files.into_iter().map(|(_, file, _)| file).collect();

    let mut typechecker = typechecker::TypeChecker::new();
    let class_files = typechecker.check(class_files).unwrap();

    let mut compiler = backend::Compiler::new();
    compiler.compile_files(class_files).unwrap();

}
