use serde_json;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use cairo_lang_starknet_classes::contract_class::ContractClass;

use sierra_analyzer_lib::sierra_program;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <sierra_file>", args[0]);
        return;
    }

    // Read input file
    let path = Path::new(&args[1]);
    let mut file = File::open(&path).expect("Failed to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");

    // Decompile a serialized Sierra program
    // Deserialize the JSON content into a ContractClass
    if let Ok(prog) = serde_json::from_str::<ContractClass>(&content) {
        let prog_sierra = prog.extract_sierra_program().unwrap();

        // Convert Sierra program to a string
        let prog_sierra_string = format!("{}", prog_sierra.to_string());
        let program = sierra_program::SierraProgram::new(prog_sierra_string);

        // Decompile
        let mut decompiler = program.decompiler();
        println!("{}", decompiler.decompile());
    } 
    // Decompile a Sierra program
    else {
        let program = sierra_program::SierraProgram::new(content);

        // Decompile
        let mut decompiler = program.decompiler();
        println!("{}", decompiler.decompile());
    }
}
