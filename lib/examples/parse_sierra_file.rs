use sierra_analyzer_lib::sierra_program::SierraProgram;

fn main() {
    let content = include_str!("../../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Print the decompiled program with use_color=true parameter
    // You can disable colored output by passing use_color=false
    let use_color = true;
    println!("{}", decompiler.decompile(use_color));
}
