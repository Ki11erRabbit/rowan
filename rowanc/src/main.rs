extern crate core;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use ariadne::Source;
use clap::Parser;
use petgraph::graph::UnGraph;
use crate::backend::pre_compilation;
use crate::trees::ast::{File, TopLevelStatement};

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
    pub stdlib_path: Option<String>,
}

fn explore_directories<P: AsRef<Path>>(path: P, files: &mut Vec<(String, Vec<String>, String, String)>) {
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

        let path_string = path.join("/");

        files.push((name, path, path_string, content));
    }
    for dir in dirs_to_explore {
        explore_directories(dir, files);
    }
}

fn explore_directories_start<P: AsRef<Path>>(path: P, files: &mut Vec<(String, Vec<String>, String, String)>) {
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

        let path_string = path.join("/");

        files.push((name, path, path_string, content));
    }
    for dir in dirs_to_explore {
        explore_directories(dir, files);
    }
}

fn main() {

    let args = Args::parse();

    let mut files = Vec::new();

    if let Some(stdlib_path) = args.stdlib_path {
        explore_directories(&stdlib_path, &mut files);
    }

    explore_directories_start(&args.directory, &mut files);



    let mut class_files = Vec::new();
    let mut index = HashMap::new();
    for (name, path, string_path, contents) in files.iter() {
        println!("{name} {path:?}");
        let file = parser::parse(&name, string_path, &contents);
        index.insert(path.join("::"), class_files.len());
        class_files.push((path, string_path, file, contents));
    }

    let class_files = if class_files.iter().any(|(_, _, file, _)| {
        file.is_err()
    }) {
        let errors = class_files.into_iter()
            .filter_map(|(path, string_path, file, contents)| {
                if file.is_err() {
                    Some((path, string_path, file.unwrap_err(), contents))
                }
                else {
                    None
                }
            }).collect::<Vec<_>>();
        for (_, string_path, error, contents) in errors {
            let contents_ptr = contents.as_ptr();
            let contents_len = contents.len();
            let contents_slice = unsafe { std::slice::from_raw_parts(contents_ptr, contents_len) };
            let contents_str = std::str::from_utf8(contents_slice).unwrap();

            error.finish()
                .eprint(((string_path.as_str(), Source::from(contents_str))))
                .expect("TODO: panic message");
            eprintln!();
        }
        std::process::exit(1);
    } else {
        class_files.into_iter()
            .map(|(path, string_path, file, contents)| {
                (path, string_path, file.unwrap(), contents)
            }).collect::<Vec<_>>()
    };

    class_files.iter().for_each(|(path, _, file, _)| {
        println!("path: {:?}", path);
        //println!("file: {:#?}", file);
    });

    let class_files = class_files.into_iter()
        .map(|(_, string_path, file, source)| (string_path.clone(), file, source))
        .collect();

    let mut typechecker = typechecker::TypeChecker::new();
    let class_files = typechecker.check(class_files).unwrap();
    let class_files: Vec<File> = class_files.into_iter()
        .map(|(_, file, _)| file)
        .collect();
    
    let class_files = class_files.into_iter()
        .map(pre_compilation::ir_pass1)
        .collect::<Result<Vec<_>, ()>>().unwrap();

    let class_files = pre_compilation::ir_pass2(class_files);
    let class_files = class_files.into_iter()
        .map(pre_compilation::ir_pass3)
        .collect::<Vec<_>>();

    let mut compiler = backend::Compiler::new();
    compiler.compile_files(class_files).unwrap();

}
