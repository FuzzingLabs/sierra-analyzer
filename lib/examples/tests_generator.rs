use sierra_analyzer_lib::sierra_program::SierraProgram;
use sierra_analyzer_lib::sym_exec::sym_exec::generate_test_cases_for_function;

fn main() {
    // Read the content of the Sierra program file
    // The file `symbolic_execution_test.sierra` contains a function `symbolic::symbolic::symbolic_execution_test`
    // that takes 4 parameters (v0, v1, v2, and v3) as input.
    // To pass all the conditions in the function, the values of the parameters should be:
    // v0: 102, v1: 117, v2: 122, v3: 122 ("f", "u", "z", "z").
    let content = include_str!("../../examples/sierra/symbolic_execution_test.sierra").to_string();

    // Initialize a new SierraProgram with the content of the .sierra file
    let program = SierraProgram::new(content);

    // Disable verbose output for the decompiler
    let verbose_output = false;

    // Create a decompiler instance for the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the Sierra program
    let use_color = false;
    decompiler.decompile(use_color);

    // Retrieve the decompiled functions
    let mut functions = decompiler.functions;

    // Generate test cases for the `symbolic::symbolic::symbolic_execution_test` function
    // This should return the input values that maximize code coverage:
    // ["v0: 102", "v1: 117", "v2: 122", "v3: 122"]
    let test_cases = generate_test_cases_for_function(
        &mut functions[0],
        decompiler.declared_libfuncs_names.clone(),
    );

    // Print the generated test cases
    println!("{}", test_cases);
}
