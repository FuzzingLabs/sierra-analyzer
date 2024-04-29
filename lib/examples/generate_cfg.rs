use sierra_analyzer_lib::sierra_program::SierraProgram;

fn main() {
    let content = include_str!("../../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Decompile the Sierra programs
    let mut decompiler = program.decompiler(false);
    decompiler.decompile(false);

    // Generate & print the dot graph
    let cfg = decompiler.generate_cfg();
    println!("{}", cfg)
}
