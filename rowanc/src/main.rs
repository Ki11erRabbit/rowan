pub mod backend;
pub mod parser;
pub mod ast;

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

        files.push((entry.path().to_str().unwrap().to_string(), contents));
    }

    let mut class_files = Vec::new();
    for (name, contents) in files.iter() {
        let file = parser::parse(name, &contents).unwrap();
        class_files.push(file);
    }

    let mut compiler = backend::Compiler::new();
    compiler.compile_files(class_files).unwrap();

}
