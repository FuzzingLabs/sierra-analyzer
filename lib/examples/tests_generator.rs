use sierra_analyzer_lib::sierra_program::SierraProgram;
use sierra_analyzer_lib::sym_exec::sym_exec::generate_test_cases_for_function;

fn main() {
    // Read file content
    let content = include_str!("../../examples/sierra/symbolic_execution_test.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program
    let use_color = false;
    decompiler.decompile(use_color);

    // Generate test cases
    let mut functions = decompiler.functions;
    let test_cases = generate_test_cases_for_function(
        &mut functions[0],
        decompiler.declared_libfuncs_names.clone(),
    );
    println!("{}", test_cases);
}
