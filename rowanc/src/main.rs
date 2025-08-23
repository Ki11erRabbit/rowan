extern crate core;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use clap::Parser;
use petgraph::graph::UnGraph;
use crate::backend::pre_compilation;
use crate::trees::ast::TopLevelStatement;

pub mod backend;
pub mod parser;
pub mod typechecker;
mod native;
mod trees;

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
        // Skip non-Rowan source files since we can't recognize them
        let entry_path = entry.path();
        let extension = entry_path.extension().unwrap();
        if extension != "rowan" {
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
        // Skip non-Rowan source files since we can't recognize them
        let entry_path = entry.path();
        let extension = entry_path.extension().unwrap();
        if extension != "rowan" {
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
    let mut index = HashMap::new();
    for (name, path, contents) in files.iter() {
        println!("{name} {path:?}");
        let file = parser::parse(&name, &contents).unwrap();
        index.insert(path.join("::"), class_files.len());
        class_files.push((path, file, contents));
    }

    let mut edges = Vec::new();
    for (i, (_, file, _)) in class_files.iter().enumerate() {
        let imports = file.content.iter()
            .filter_map(|x| {
                match x {
                    TopLevelStatement::Import(import) => {
                        let mut import = import.path.segments.clone();
                        import.pop();

                        Some(import.join("::"))
                    }
                    _ => None
                }
            }).collect::<Vec<_>>();

        for import in imports {
            //println!("{import}");
            edges.push((i as u32, *index.get(&import).expect("import not found") as u32));
        }
    }
    let mut class_files = class_files.into_iter().map(Some).collect::<Vec<_>>();
    drop(index);

    // Use Strongly Connected Components algo to figure out the propper order of compiling
    let graph = UnGraph::<u32, ()>::from_edges(edges);

    let sccs = petgraph::algo::kosaraju_scc(&graph);
    let mut seen_indicies = HashSet::new();
    let mut new_class_files = Vec::new();
    for scc in sccs {
        for index in scc.into_iter().rev() {
            if seen_indicies.contains(&index.index()) {
                continue;
            }
            let class_file = class_files[index.index()].take().unwrap();
            new_class_files.push(class_file);
            seen_indicies.insert(index.index());
        }
    }

    // Somehow the above process removes some files for some unknown reason
    for class_file in class_files {
        if class_file.is_some() {
            new_class_files.push(class_file.unwrap());
        }
    }

    // Ensure that all stdlib files get compiled first
    new_class_files.sort_by(|(ap, _, _), (bp, _ ,_)| {
        if ap[0].as_str() == "std" && bp[0].as_str() != "std" {
            Ordering::Less
        } else if ap[0].as_str() != "std" && bp[0].as_str() == "std" {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    let class_files = new_class_files;



    class_files.iter().for_each(|(path, file, _)| {
        println!("path: {:?}", path);
        //println!("file: {:#?}", file);
    });

    let class_files = class_files.into_iter().map(|(_, file, _)| file).collect();

    let mut typechecker = typechecker::TypeChecker::new();
    let class_files = typechecker.check(class_files).unwrap();
    
    let class_files = class_files.into_iter()
        .map(pre_compilation::ir_pass)
        .collect::<Result<Vec<_>, ()>>().unwrap();

    let mut compiler = backend::Compiler::new();
    compiler.compile_files(class_files).unwrap();

}
