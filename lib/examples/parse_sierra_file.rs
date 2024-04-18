use std::fs;
use std::path::Path;

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
    let file_path = current_file_dir.join("../../examples/sierra/fib.sierra");

    // Read the file content
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => {
            println!("Error reading file: {}", err);
            return;
        }
    };

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Decompile the Sierra program
    let mut decompiler = program.decompiler();

    // Print the decompiled program with use_color=true parameter
    // You can disable colored output by passing use_color=false
    let use_color = true;
    println!("{}", decompiler.decompile(use_color));
}
