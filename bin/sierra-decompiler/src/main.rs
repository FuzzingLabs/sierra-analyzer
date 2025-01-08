use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

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
    #[clap(short = 'd', long)]
    detectors: bool,

    /// List of detector names to run
    #[clap(long, use_value_delimiter = true)]
    detector_names: Vec<String>,

    /// Remote contract class address
    #[clap(long, default_value = "")]
    remote: String,

    /// Network type (Mainnet & Sepolia are supported)
    #[clap(long, default_value = "mainnet")]
    network: String,

    /// Run sierra-analyzer in a repo that uses Scarb
    #[clap(long)]
    scarb: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Ensure either remote, Sierra file, or scarb is provided
    if args.remote.is_empty() && args.sierra_file.is_none() && !args.scarb {
        eprintln!("Error: Either remote, Sierra file, or --scarb flag must be provided");
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
        handle_detectors(&mut decompiler, args.detector_names);
    }
    // Decompiler (default)
    else {
        println!("{}", decompiled_code);
    }
}

/// Load the Sierra program from either a remote source, a local file, or scarb
async fn load_program(args: &Args) -> Result<SierraProgram, String> {
    if args.scarb {
        load_scarb_program().await
    } else if !args.remote.is_empty() {
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

    // Open the file
    let mut file = File::open(sierra_file).map_err(|e| format!("Failed to open file: {}", e))?;

    // Read the file content into a string
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Deserialize the JSON content into a ContractClass
    let contract_class: Result<ContractClass, _> = serde_json::from_str(&content);

    let program_string = match contract_class {
        Ok(ref prog) => {
            // Extract the Sierra program from the ContractClass
            match prog.extract_sierra_program() {
                Ok(prog_sierra) => prog_sierra.to_string(),
                Err(e) => {
                    eprintln!("Error extracting Sierra program: {}", e);
                    content.clone()
                }
            }
        }
        Err(ref _e) => content.clone(),
    };

    // Initialize a new SierraProgram with the deserialized Sierra program content
    let mut program = SierraProgram::new(program_string);

    // Set the program ABI if deserialization was successful
    if let Ok(ref contract_class) = contract_class {
        let abi = contract_class.abi.clone();
        program.set_abi(abi.unwrap());
    }

    Ok(program)
}

/// Load the Sierra program from the /target directory
async fn load_scarb_program() -> Result<SierraProgram, String> {
    let target_dir = Path::new("./target/dev/");

    // Read the directory contents
    let entries =
        fs::read_dir(target_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    // Find the file that ends with "contract_class.json"
    let contract_class_file = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| name.ends_with("contract_class.json"))
            {
                Some(path)
            } else {
                None
            }
        })
        .next();

    // Check if the file was found
    let contract_class_file = if let Some(file) = contract_class_file {
        file
    } else {
        eprintln!("You need to run scarb build before running the sierra-analyzer");
        exit(1);
    };

    // Open the file
    let mut file =
        File::open(&contract_class_file).map_err(|e| format!("Failed to open file: {}", e))?;

    // Read the file content into a string
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Deserialize the JSON content into a ContractClass
    let contract_class: Result<ContractClass, _> = serde_json::from_str(&content);

    let program_string = match contract_class {
        Ok(ref prog) => {
            // Extract the Sierra program from the ContractClass
            match prog.extract_sierra_program() {
                Ok(prog_sierra) => prog_sierra.to_string(),
                Err(e) => {
                    eprintln!("Error extracting Sierra program: {}", e);
                    content.clone()
                }
            }
        }
        Err(ref _e) => content.clone(),
    };

    // Initialize a new SierraProgram with the deserialized Sierra program content
    let mut program = SierraProgram::new(program_string);

    // Set the program ABI if deserialization was successful
    if let Ok(ref contract_class) = contract_class {
        let abi = contract_class.abi.clone();
        program.set_abi(abi.unwrap());
    }

    Ok(program)
}

/// Get the file stem based on the remote address or the Sierra file
fn get_file_stem(args: &Args) -> String {
    if !args.remote.is_empty() {
        args.remote.clone()
    } else if args.scarb {
        // TODO : modify with the program name
        "sierra_program".to_string()
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
fn handle_detectors(decompiler: &mut Decompiler, detector_names: Vec<String>) {
    let mut detectors = get_detectors();
    let mut output = String::new();

    // Run the specified detectors
    for detector in detectors.iter_mut() {
        // Skip TESTING detectors if no specific detector names are provided
        if detector_names.is_empty() && detector.detector_type() == DetectorType::TESTING {
            continue;
        }

        // Skip detectors not in the provided names if names are provided
        if !detector_names.is_empty() && !detector_names.contains(&detector.id().to_string()) {
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

    // Print the detectors result if not empty
    if !output.trim().is_empty() {
        println!("{}", output.trim());
    }
}
