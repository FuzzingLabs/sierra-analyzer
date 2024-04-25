use sierra_analyzer_lib::sierra_program::SierraProgram;

#[test]
fn test_dogtgraph_cfg_output() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Decompile the Sierra program
    let mut decompiler = program.decompiler();

    // Decompile the sierra program with a colorless output
    let use_color = false;
    decompiler.decompile(use_color);

    // Generate CFG dotgraph
    let cfg_dotgraph = decompiler.generate_cfg();

    // Expected dotgraph
    let expected_output = "type felt252\ntype Const<felt252, 1>\ntype NonZero<felt252>\n\nlibfunc disable_ap_tracking\nlibfunc dup<felt252>\nlibfunc felt252_is_zero\nlibfunc branch_align\nlibfunc drop<felt252>\nlibfunc store_temp<felt252>\nlibfunc drop<NonZero<felt252>>\nlibfunc felt252_add\nlibfunc const_as_immediate<Const<felt252, 1>>\nlibfunc felt252_sub\nlibfunc function_call<user@examples::fib::fib>\n\n// Function 1\nfunc examples::fib::fib (v0: felt252, v1: felt252, v2: felt252) -> (felt252) {\n\tdisable_ap_tracking()\n\tv2, v3 = dup<felt252>(v2)\n\tif (felt252_is_zero(v3) == 0) {\n\t\tdrop<NonZero<felt252>>(v4)\n\t\tv1, v5 = dup<felt252>(v1)\n\t\tv6 = felt252_add(v0, v5)\n\t\tv7 = const_as_immediate<Const<felt252, 1>>()\n\t\tv8 = felt252_sub(v2, v7)\n\t\tv1 = store_temp<felt252>(v1)\n\t\tv6 = store_temp<felt252>(v6)\n\t\tv8 = store_temp<felt252>(v8)\n\t\tv9 = function_call<user@examples::fib::fib>(v1, v6, v8)\n\t\treturn (v9)\n\t} else {\n\t\tdrop<felt252>(v1)\n\t\tdrop<felt252>(v2)\n\t\tv0 = store_temp<felt252>(v0)\n\t\treturn (v0)\n\t}\n}";

    assert_eq!(cfg_dotgraph, expected_output);
}
