use sierra_analyzer_lib::provider::NetworkConfig;
use sierra_analyzer_lib::provider::RpcClient;
use sierra_analyzer_lib::sierra_program::SierraProgram;

use cairo_lang_starknet_classes::contract_class::ContractClass;

use tokio;

#[tokio::main]
async fn main() {
    let client = RpcClient::new(NetworkConfig::MAINNET_API_URL);
    let contract_class = "0x01c0bb51e2ce73dc007601a1e7725453627254016c28f118251a71bbb0507fcb";
    match client.get_class(contract_class).await {
        Ok(response) => {
            // Convert RpcClient response to JSON content
            let content = response.to_json();

            // Deserialize JSON into a ContractClass
            let program_string = serde_json::from_str::<ContractClass>(&content)
                .ok()
                .and_then(|prog| prog.extract_sierra_program().ok())
                .map_or_else(|| content.clone(), |prog_sierra| prog_sierra.to_string());
            let program = SierraProgram::new(program_string);

            // Don't use the verbose output
            let verbose_output = false;

            // Decompile the Sierra program
            let mut decompiler = program.decompiler(verbose_output);

            // Print the decompiled program with use_color=true parameter
            // You can disable colored output by passing use_color=false
            let use_color = true;
            println!("{}", decompiler.decompile(use_color));
        }
        Err(e) => {
            eprintln!("Error calling RPC: {}", e);
        }
    }
}
