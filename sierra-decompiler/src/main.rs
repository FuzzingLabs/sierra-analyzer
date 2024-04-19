use serde_json;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use cairo_lang_starknet_classes::contract_class::ContractClass;

use clap::Parser;
use sierra_analyzer_lib::sierra_program;

/// Decompile a Sierra program
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sierra program file
    sierra_file: PathBuf,

    /// Do not use colored output
    #[clap(short, long, default_value = "false")]
    no_color: bool,
}

fn main() {
    let args = Args::parse();

    // Read input file
    let mut file = File::open(&args.sierra_file).expect("Failed to open file");
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
        println!("{}", decompiler.decompile(!args.no_color));
    }
    // Decompile a Sierra program
    else {
        let program = sierra_program::SierraProgram::new(content);

        // Decompile
        let mut decompiler = program.decompiler();
        println!("{}", decompiler.decompile(!args.no_color));
    }
}
