use std::fs;
use std::fs::File;
use std::io::Read;

use clap::Parser;
use serde_json;
use std::path::PathBuf;

use cairo_lang_starknet_classes::contract_class::ContractClass;
use sierra_analyzer_lib::graph::graph::save_svg_graph_to_file;
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
    #[clap(long, default_value = "false")]
    cfg: bool,

    /// Output directory for the CFG file
    #[clap(long, default_value = "./")]
    cfg_output: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Read input file
    let mut file = File::open(&args.sierra_file).expect("Failed to open file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");

    // Deserialize JSON into a ContractClass, or use the content directly if that fails
    let program_string = serde_json::from_str::<ContractClass>(&content)
        .ok()
        .and_then(|prog| prog.extract_sierra_program().ok())
        .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());
    let program = sierra_program::SierraProgram::new(program_string);

    let mut decompiler = program.decompiler();
    let decompiled_code = decompiler.decompile(!args.no_color);

    if args.cfg {
        // Determine the full path for the output file
        let file_stem = args
            .sierra_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let svg_filename = format!("{}_cfg.svg", file_stem);
        let full_path = args.cfg_output.join(svg_filename);

        // Create the output directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&args.cfg_output) {
            eprintln!(
                "Failed to create directory '{}': {}",
                args.cfg_output.display(),
                e
            );
            return;
        }

        // Generate CFG and save to SVG
        let cfg_graph = decompiler.generate_cfg();
        save_svg_graph_to_file(full_path.to_str().unwrap(), cfg_graph)
            .expect("Failed to save CFG to SVG");
    } else {
        println!("{}", decompiled_code);
    }
}
