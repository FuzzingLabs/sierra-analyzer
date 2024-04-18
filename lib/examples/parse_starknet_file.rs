use std::fs;
use std::path::Path;

use cairo_lang_starknet_classes::contract_class::ContractClass;

use sierra_analyzer_lib::sierra_program::SierraProgram;

fn main() {
    // Get the directory of the current file
    let current_file_dir = match std::env::current_dir() {
        Ok(mut dir) => {
            dir.push(Path::new(file!()).parent().unwrap());
            dir
        }
        Err(err) => {
            println!("Error getting current directory: {}", err);
            return;
        }
    };

    // Construct the file path relative to the current file's directory
    let file_path = current_file_dir.join("../../examples/starknet/erc20.contract_class.json");

    // Read the file content
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => {
            println!("Error reading file: {}", err);
            return;
        }
    };

    // Deserialize the JSON content into a ContractClass
    let prog: ContractClass =
        serde_json::from_str(&content).expect("Error deserializing JSON contract class");

    // Convert the ContractClass into a sierra program
    let prog_sierra = prog.extract_sierra_program().unwrap();

    // Convert Sierra program to a string
    let prog_sierra_string = format!("{}", prog_sierra.to_string());

    // Init a new SierraProgram with the deserialized sierra file content
    let program = SierraProgram::new(prog_sierra_string);

    // Decompile the Sierra program
    let mut decompiler = program.decompiler();

    // Print the decompiled program with use_color=true parameter
    // You can disable colored output by passing use_color=false
    let use_color = true;
    println!("{}", decompiler.decompile(use_color));
}
