use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Parser;
use serde_json;
use tokio;

use cairo_lang_starknet_classes::contract_class::ContractClass;
use sierra_analyzer_lib::decompiler::decompiler::Decompiler;
use sierra_analyzer_lib::detectors::detector::DetectorType;
use sierra_analyzer_lib::detectors::get_detectors;
use sierra_analyzer_lib::graph::graph::save_svg_graph_to_file;
use sierra_analyzer_lib::provider::NetworkConfig;
use sierra_analyzer_lib::provider::RpcClient;
use sierra_analyzer_lib::sierra_program::SierraProgram;

/// Decompile a Sierra program
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sierra program file
    #[clap(short = 'f', long)]
    sierra_file: Option<PathBuf>,

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

    /// Remote contract class address
    #[clap(long, default_value = "")]
    remote: String,

    /// Network type (Mainnet & Sepolia are supported)
    #[clap(long, default_value = "mainnet")]
    network: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Ensure either remote or Sierra file is provided
    if args.remote.is_empty() && args.sierra_file.is_none() {
        eprintln!("Error: Either remote or Sierra file must be provided");
        return;
    }

    // Load the Sierra program
    let program = match load_program(&args).await {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error loading program: {}", e);
            return;
        }
    };

    // Determine if colored output is needed
    let colored_output = !args.no_color ^ (args.cfg | args.callgraph);
    let mut decompiler = program.decompiler(args.verbose);
    let decompiled_code = decompiler.decompile(colored_output);

    // Filter functions if a specific function name is given
    if let Some(ref function_name) = args.function {
        decompiler.filter_functions(function_name);
    }

    // Determine the file stem based on the remote address or the Sierra file
    let file_stem = get_file_stem(&args);

    // Handle different output options
    // CFG
    if args.cfg {
        handle_cfg(&args, &mut decompiler, &file_stem);
    }
    // Callgraph
    else if args.callgraph {
        handle_callgraph(&args, &mut decompiler, &file_stem);
    }
    // Detectors
    else if args.detectors {
        handle_detectors(&mut decompiler);
    }
    // Decompiler (default)
    else {
        println!("{}", decompiled_code);
    }
}

/// Load the Sierra program from either a remote source or a local file
async fn load_program(args: &Args) -> Result<SierraProgram, String> {
    if !args.remote.is_empty() {
        load_remote_program(args).await
    } else {
        load_local_program(args)
    }
}

/// Load the Sierra program from a remote source
async fn load_remote_program(args: &Args) -> Result<SierraProgram, String> {
    let client = match args.network.as_str() {
        "mainnet" => RpcClient::new(NetworkConfig::MAINNET_API_URL),
        "sepolia" => RpcClient::new(NetworkConfig::SEPOLIA_API_URL),
        _ => {
            return Err(format!(
                "Error: Unsupported network type '{}'",
                args.network
            ))
        }
    };

    match client.get_class(&args.remote).await {
        Ok(response) => {
            let content = response.to_json();
            let program_string = serde_json::from_str::<ContractClass>(&content)
                .ok()
                .and_then(|prog| prog.extract_sierra_program().ok())
                .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());
            Ok(SierraProgram::new(program_string))
        }
        Err(e) => Err(format!("Error calling RPC: {}", e)),
    }
}

/// Load the Sierra program from a local file
fn load_local_program(args: &Args) -> Result<SierraProgram, String> {
    let sierra_file = args.sierra_file.as_ref().unwrap();
    let mut file = File::open(sierra_file).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let program_string = serde_json::from_str::<ContractClass>(&content)
        .ok()
        .and_then(|prog| prog.extract_sierra_program().ok())
        .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());
    Ok(SierraProgram::new(program_string))
}

/// Get the file stem based on the remote address or the Sierra file
fn get_file_stem(args: &Args) -> String {
    if !args.remote.is_empty() {
        args.remote.clone()
    } else {
        args.sierra_file
            .as_ref()
            .unwrap()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

/// Handle the generation and saving of the CFG (Control Flow Graph)
fn handle_cfg(args: &Args, decompiler: &mut Decompiler, file_stem: &str) {
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
}

/// Handle the generation and saving of the Call Graph
fn handle_callgraph(args: &Args, decompiler: &mut Decompiler, file_stem: &str) {
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
}

/// Handle the running of detectors and printing their results
fn handle_detectors(decompiler: &mut Decompiler) {
    let mut detectors = get_detectors();
    let mut output = String::new();

    // Run all the detectors except those of type TESTING
    for detector in detectors.iter_mut() {
        // Skip TESTING detectors
        if detector.detector_type() == DetectorType::TESTING {
            continue;
        }

        let result = detector.detect(decompiler);
        if !result.trim().is_empty() {
            // Each detector output is formatted like
            //
            // [Detector category] Detector name
            //      - detector content
            //      - ...
            output.push_str(&format!(
                "[{}] {}\n{}\n\n",
                detector.detector_type().as_str(),
                detector.name(),
                result
                    .lines()
                    .map(|line| format!("\t- {}", line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
        }
    }

    // Print the detectors result
    println!("{}", output.trim());
}
