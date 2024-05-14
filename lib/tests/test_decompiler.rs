use sierra_analyzer_lib::sierra_program::SierraProgram;

#[test]
fn test_decompiler_output() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"// Function 1
func examples::fib::fib (v0: felt252, v1: felt252, v2: felt252) -> (felt252) {
	v3 = v2
	if (v3 == 0) {		
		v5 = v1
		v6 = v0 + v5
		v7 = 1
		v8 = v2 - v7
		v9 = user@examples::fib::fib(v1, v6, v8)
		return (v9)
	} else {	
		return (v0)
	}
}"#;
    assert_eq!(decompiler_output, expected_output);
}

#[test]
fn test_decompiler_verbose_output() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Use the verbose output
    let verbose_output = true;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"type felt252
type Const<felt252, 1>
type NonZero<felt252>

libfunc disable_ap_tracking
libfunc dup<felt252>
libfunc felt252_is_zero
libfunc branch_align
libfunc drop<felt252>
libfunc store_temp<felt252>
libfunc drop<NonZero<felt252>>
libfunc felt252_add
libfunc const_as_immediate<Const<felt252, 1>>
libfunc felt252_sub
libfunc function_call<user@examples::fib::fib>

// Function 1
func examples::fib::fib (v0: felt252, v1: felt252, v2: felt252) -> (felt252) {
	disable_ap_tracking()
	v2, v3 = dup<felt252>(v2)
	if (felt252_is_zero(v3) == 0) {		
		branch_align()
		drop<NonZero<felt252>>(v4)
		v1, v5 = dup<felt252>(v1)
		v6 = felt252_add(v0, v5)
		v7 = const_as_immediate<Const<felt252, 1>>()
		v8 = felt252_sub(v2, v7)
		v1 = store_temp<felt252>(v1)
		v6 = store_temp<felt252>(v6)
		v8 = store_temp<felt252>(v8)
		v9 = user@examples::fib::fib(v1, v6, v8)
		return (v9)
	} else {	
		branch_align()
		drop<felt252>(v1)
		drop<felt252>(v2)
		v0 = store_temp<felt252>(v0)
		return (v0)
	}
}"#;
    assert_eq!(decompiler_output, expected_output);
}

#[test]
fn test_decompiler_array_output() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib_gas.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't Use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"// Function 1
func examples::fib::fib (v0: RangeCheck, v1: GasBuiltin, v2: felt252, v3: felt252, v4: felt252) -> (RangeCheck, GasBuiltin, core::panics::PanicResult::<(core::felt252)>) {
	if (withdraw_gas(v0, v1) == 0) {		
		v20 = Array<felt252>::new()
		v21 = 375233589013918064796019 // "Out of gas"
		v22 = v20.append(v21)
		v23 = struct_construct<core::panics::Panic>()
		v24 = struct_construct<Tuple<core::panics::Panic, Array<felt252>>>(v23, v22)
		v25 = enum_init<core::panics::PanicResult::<(core::felt252)>, 1>(v24)
		return (v7, v8, v25)
	} else {	
		v9 = v4
		if (v9 == 0) {			
			v13 = v3
			v14 = v2 + v13
			v15 = 1
			v16 = v4 - v15
			v17, v18, v19 = user@examples::fib::fib(v5, v6, v3, v14, v16)
			return (v17, v18, v19)
		} else {		
			v11 = struct_construct<Tuple<felt252>>(v2)
			v12 = enum_init<core::panics::PanicResult::<(core::felt252)>, 0>(v11)
			return (v5, v6, v12)
		}
	}
}"#;
    assert_eq!(decompiler_output, expected_output);
}
