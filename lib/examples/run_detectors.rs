use sierra_analyzer_lib::detectors::get_detectors;
use sierra_analyzer_lib::sierra_program::SierraProgram;

fn main() {
    let content = include_str!("../../examples/sierra/hello_starknet.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = true;
    decompiler.decompile(use_color);

    // Get the detectors list
    let detectors = get_detectors();

    println!("{:#?}", detectors);
}
