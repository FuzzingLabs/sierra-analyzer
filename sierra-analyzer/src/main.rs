use sierra_analyzer_lib::sierra_program;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <sierra_file>", args[0]);
        return;
    }

    let path = Path::new(&args[1]);
    let mut file = File::open(&path).expect("Failed to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");

    let program = sierra_program::SierraProgram::new(content);
    let mut decompiler = program.decompiler();

    println!("{}", decompiler.decompile());
}
