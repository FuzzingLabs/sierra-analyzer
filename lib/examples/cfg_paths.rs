use sierra_analyzer_lib::sierra_program::SierraProgram;

fn main() {
    let content = include_str!("../../examples/sierra/fib_match.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the Sierra program
    let use_color = false;
    decompiler.decompile(use_color);

    // Print the number of paths in the first function
    // It should be 10 in examples::fib_match::fib function
    decompiler.functions[0].create_cfg();
    println!(
        "Number of possible paths : {:#?}",
        decompiler.functions[0].cfg.as_ref().unwrap().paths().len()
    );
}
