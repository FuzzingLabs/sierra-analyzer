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

    /// Generate a CFG graph instead of normal output
    #[clap(short, long, default_value = "false")]
    cfg: bool,
}

fn main() {
    let args = Args::parse();

    // Read input file
    let mut file = File::open(&args.sierra_file).expect("Failed to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");

    // Try to deserialize the JSON content into a ContractClass, and extract Sierra program
    let program_string = serde_json::from_str::<ContractClass>(&content)
        .ok()
        .and_then(|prog| prog.extract_sierra_program().ok())
        .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string()); // Use original content if deserialization fails

    let program = sierra_program::SierraProgram::new(program_string);

    // Determine if output should be colored
    let colored_output = !args.no_color;

    // Decompile normally
    let mut decompiler = program.decompiler();
    let decompiled_code = decompiler.decompile(colored_output);

    if args.cfg {
        // If the cfg flag is true, generate and print the CFG dot graph
        let cfg_graph = decompiler.generate_cfg(); // This method should exist in SierraProgram to generate CFG
        println!("{}", cfg_graph);
    } else {
        println!("{}", decompiled_code);
    }
}
