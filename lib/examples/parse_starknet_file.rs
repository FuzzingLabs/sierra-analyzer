use cairo_lang_starknet_classes::contract_class::ContractClass;
use sierra_analyzer_lib::sierra_program::SierraProgram;

fn main() {
    // Read the JSON content from the file
    let content = include_str!("../../examples/starknet/erc20.contract_class.json").to_string();

    // Deserialize the JSON content into a ContractClass
    let contract_class: ContractClass = match serde_json::from_str(&content) {
        Ok(prog) => prog,
        Err(e) => {
            eprintln!("Error deserializing JSON contract class: {}", e);
            return;
        }
    };

    // Extract the Sierra program from the ContractClass
    let sierra_program = match contract_class.extract_sierra_program() {
        Ok(prog_sierra) => prog_sierra,
        Err(e) => {
            eprintln!("Error extracting Sierra program: {}", e);
            return;
        }
    };

    // Convert the Sierra program to a string
    let program_string = sierra_program.to_string();

    // Initialize a new SierraProgram with the deserialized Sierra program content
    let mut program = SierraProgram::new(program_string);

    // Set the program ABI
    let abi = contract_class.abi;
    program.set_abi(abi.unwrap());

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Print the decompiled program with use_color=true parameter
    // You can disable colored output by passing use_color=false
    let use_color = true;
    println!("{}", decompiler.decompile(use_color));
}
