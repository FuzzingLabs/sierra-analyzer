use std::fs;
use std::fs::File;
use std::io::Read;

use clap::Parser;
use serde_json;
use std::path::PathBuf;

use cairo_lang_starknet_classes::contract_class::ContractClass;
use sierra_analyzer_lib::detectors::get_detectors;
use sierra_analyzer_lib::graph::graph::save_svg_graph_to_file;
use sierra_analyzer_lib::sierra_program;

/// Decompile a Sierra program
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sierra program file
    sierra_file: PathBuf,

    /// Function name to only get one function for CFG & Callgraph
    #[clap(long)]
    function: Option<String>,

    /// Do not use colored output
    #[clap(short, long, default_value_t = false)]
    no_color: bool,

    /// Generate a CFG (Control Flow Graph) instead of normal output
    #[clap(long, default_value_t = false)]
    cfg: bool,

    /// Output directory for the CFG file
    #[clap(long, default_value = "./output_cfg")]
    cfg_output: PathBuf,

    /// Generate a Call Graph instead of normal output
    #[clap(long, default_value_t = false)]
    callgraph: bool,

    /// Output directory for the Call Graph file
    #[clap(long, default_value = "./output_callgraph")]
    callgraph_output: PathBuf,

    /// Enable verbose decompiler output
    #[clap(short, long, default_value_t = false)]
    verbose: bool,

    /// Run the detectors
    #[clap(short, long)]
    detectors: bool,
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

    // Color output by default and if CFG or Callgraph is not enabled to avoid bugs in the SVG output
    let colored_output = !args.no_color ^ (args.cfg | args.callgraph);

    let mut decompiler = program.decompiler(args.verbose);

    let decompiled_code = decompiler.decompile(colored_output);

    // Filter functions if a specific function name is given
    if let Some(ref function_name) = args.function {
        decompiler.filter_functions(function_name);
    }

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
    } else if args.callgraph {
        // Determine the full path for the output file
        let file_stem = args
            .sierra_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let svg_filename = format!("{}_callgraph.svg", file_stem);
        let full_path = args.callgraph_output.join(svg_filename);

        // Create the output directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&args.callgraph_output) {
            eprintln!(
                "Failed to create directory '{}': {}",
                args.callgraph_output.display(),
                e
            );
            return;
        }

        // Generate Callgraph and save to SVG
        let callgraph_graph = decompiler.generate_callgraph();
        save_svg_graph_to_file(full_path.to_str().unwrap(), callgraph_graph)
            .expect("Failed to save Callgraph to SVG");
    } else if args.detectors {
        let mut detectors = get_detectors();
        for detector in &mut detectors {
            let detector_name = detector.name();
            let detector_result = detector.detect(&mut decompiler);

            // Trim and indent the result with one tab
            let indented_result: String = detector_result
                .lines()
                .map(|line| format!("\t{}", line))
                .collect::<Vec<String>>()
                .join("\n")
                .trim_end()
                .to_string();

            // Check if the result is empty before printing
            if !indented_result.is_empty() {
                let output = format!("{}:\n{}", detector_name, indented_result);
                println!("{}", output);
            }
        }
    } else {
        println!("{}", decompiled_code);
    }
}
