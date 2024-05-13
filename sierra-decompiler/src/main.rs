use cairo_lang_starknet_classes::contract_class::ContractClass;

use sierra_analyzer_lib::graph::graph::save_svg_graph_to_file;
use sierra_analyzer_lib::provider::NetworkConfig;
use sierra_analyzer_lib::provider::RpcClient;
use sierra_analyzer_lib::sierra_program;
use sierra_analyzer_lib::sierra_program::SierraProgram;

use clap::Parser;
use serde_json;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tokio;

/// Decompile a Sierra program
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Sierra program file
    #[clap(short = 'f', long)]
    sierra_file: Option<PathBuf>,

    /// Do not use colored output
    #[clap(short, long, default_value = "false")]
    no_color: bool,

    /// Generate a CFG graph instead of normal output
    #[clap(long, default_value = "false")]
    cfg: bool,

    /// Output directory for the CFG file
    #[clap(long, default_value = "./output_cfg")]
    cfg_output: PathBuf,

    /// Enable verbose decompiler output
    #[clap(short, long, default_value = "false")]
    verbose: bool,

    /// Remote contract class address
    #[clap(long, default_value = "")]
    remote: String,

    /// Network type (Mainnet & Sepolia are supported)
    #[clap(long, default_value = "mainnet")]
    network: String,
}

fn main() {
    let args = Args::parse();

    if args.remote.is_empty() && args.sierra_file.is_none() {
        eprintln!("Error: Either remote or Sierra file must be provided");
        return;
    }

    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // Define program and sierra_file before the if statement
        let program: SierraProgram;
        let mut sierra_file: Option<PathBuf> = None;

        // Analyze a contract deployed on Starknet
        if !args.remote.is_empty() {
            // Define the client based on the network parameter
            let client = match args.network.as_str() {
                "mainnet" => RpcClient::new(NetworkConfig::MAINNET_API_URL),
                "sepolia" => RpcClient::new(NetworkConfig::SEPOLIA_API_URL),
                _ => {
                    eprintln!("Error: Unsupported network type '{}'", args.network);
                    return;
                }
            };

            // Fetch contract class from the RPC Node
            match client.get_class(&args.remote).await {
                Ok(response) => {
                    // Convert RpcClient response to JSON content
                    let content = response.to_json();

                    // Deserialize JSON into a ContractClass
                    let program_string = serde_json::from_str::<ContractClass>(&content)
                        .ok()
                        .and_then(|prog| prog.extract_sierra_program().ok())
                        .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());
                    program = SierraProgram::new(program_string);
                }
                Err(e) => {
                    eprintln!("Error calling RPC: {}", e);
                    // Stop the program if there is an error in the RPC response
                    return;
                }
            }
        }
        // Analyze a local file
        else {
            sierra_file = args.sierra_file;
            let mut file = File::open(sierra_file.as_ref().unwrap()).expect("Failed to open file");
            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("Failed to read file");

            // Deserialize JSON into a ContractClass, or use the content directly if that fails
            let program_string = serde_json::from_str::<ContractClass>(&content)
                .ok()
                .and_then(|prog| prog.extract_sierra_program().ok())
                .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());
            program = sierra_program::SierraProgram::new(program_string);
        }

        // Color output by default and if CFG is not enabled to avoid bugs in the SVG output
        let colored_output = !args.no_color ^ args.cfg;

        // Now you can use program and sierra_file outside the if and else blocks
        let mut decompiler = program.decompiler(args.verbose);
        let decompiled_code = decompiler.decompile(colored_output);

        if args.cfg {
            // Determine the full path for the output file
            let file_stem = sierra_file
                .as_ref()
                .unwrap()
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
    });
}
