use std::fs;
use std::path::Path;

use sierra_analyzer_lib::sierra_program::SierraProgram;

#[test]
fn test_decompiler_output() {
    // Get the directory of the current file
    let current_file_dir = match std::env::current_dir() {
        Ok(mut dir) => {
            dir.push(Path::new(file!()).parent().unwrap());
            dir
        }
        Err(_) => return,
    };

    // Construct the file path relative to the current file's directory
    let file_path = current_file_dir.join("../../examples/sierra/fib.sierra");

    // Read the file content
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(_) => return,
    };

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Decompile the Sierra program
    let mut decompiler = program.decompiler();

    // Decompile the sierra program with a colorless output
    let use_color = false;
    let decompiler_output = decompiler.decompile(use_color);

    let expected_output = r#"
type RangeCheck = RangeCheck<> [storable: true, drop: false, dup: false, zero_sized: false]
type core::panics::Panic = Struct<ut@core::panics::Panic> [storable: true, drop: true, dup: true, zero_sized: true]
type Array<felt252> = Array<felt252> [storable: true, drop: true, dup: false, zero_sized: false]
type Tuple<core::panics::Panic, Array<felt252>> = Struct<ut@Tuple, core::panics::Panic, Array<felt252>> [storable: true, drop: true, dup: false, zero_sized: false]
type Const<felt252, 375233589013918064796019> = Const<felt252, 375233589013918064796019> [storable: false, drop: false, dup: false, zero_sized: false]
type Const<felt252, 1> = Const<felt252, 1> [storable: false, drop: false, dup: false, zero_sized: false]
type felt252 = felt252<> [storable: true, drop: true, dup: true, zero_sized: false]
type Tuple<felt252> = Struct<ut@Tuple, felt252> [storable: true, drop: true, dup: true, zero_sized: false]
type core::panics::PanicResult::<(core::felt252)> = Enum<ut@core::panics::PanicResult::<(core::felt252)>, Tuple<felt252>, Tuple<core::panics::Panic, Array<felt252>>> [storable: true, drop: true, dup: false, zero_sized: false]
type NonZero<felt252> = NonZero<felt252> [storable: true, drop: true, dup: true, zero_sized: false]
type GasBuiltin = GasBuiltin<> [storable: true, drop: false, dup: false, zero_sized: false]

libfunc disable_ap_tracking = disable_ap_tracking<>
libfunc withdraw_gas = withdraw_gas<>
libfunc branch_align = branch_align<>
libfunc dup<felt252> = dup<felt252>
libfunc store_temp<RangeCheck> = store_temp<RangeCheck>
libfunc felt252_is_zero = felt252_is_zero<>
libfunc drop<felt252> = drop<felt252>
libfunc struct_construct<Tuple<felt252>> = struct_construct<Tuple<felt252>>
libfunc enum_init<core::panics::PanicResult::<(core::felt252)>, 0> = enum_init<core::panics::PanicResult::<(core::felt252)>, 0>
libfunc store_temp<GasBuiltin> = store_temp<GasBuiltin>
libfunc store_temp<core::panics::PanicResult::<(core::felt252)>> = store_temp<core::panics::PanicResult::<(core::felt252)>>
libfunc drop<NonZero<felt252>> = drop<NonZero<felt252>>
libfunc felt252_add = felt252_add<>
libfunc const_as_immediate<Const<felt252, 1>> = const_as_immediate<Const<felt252, 1>>
libfunc felt252_sub = felt252_sub<>
libfunc store_temp<felt252> = store_temp<felt252>
libfunc function_call<user@examples::fib::fib> = function_call<>
libfunc array_new<felt252> = array_new<felt252>
libfunc const_as_immediate<Const<felt252, 375233589013918064796019>> = const_as_immediate<Const<felt252, 375233589013918064796019>>
libfunc array_append<felt252> = array_append<felt252>
libfunc struct_construct<core::panics::Panic> = struct_construct<core::panics::Panic>
libfunc struct_construct<Tuple<core::panics::Panic, Array<felt252>>> = struct_construct<Tuple<core::panics::Panic, Array<felt252>>>
libfunc enum_init<core::panics::PanicResult::<(core::felt252)>, 1> = enum_init<core::panics::PanicResult::<(core::felt252)>, 1>

// Function 1
func examples::fib::fib (v0: RangeCheck, v1: GasBuiltin, v2: felt252, v3: felt252, v4: felt252) -> (RangeCheck, GasBuiltin, core::panics::PanicResult::<(core::felt252)>) {
    disable_ap_tracking()
    if (withdraw_gas(v0, v1) == 0) {
        branch_align()
        drop<felt252>(v3)
        drop<felt252>(v4)
        drop<felt252>(v2)
        v20 = array_new<felt252>()
        v21 = const_as_immediate<Const<felt252, 375233589013918064796019>>()
        v21 = store_temp<felt252>(v21)
        v22 = array_append<felt252>(v20, v21)
        v23 = struct_construct<core::panics::Panic>()
        v24 = struct_construct<Tuple<core::panics::Panic, Array<felt252>>>(v23, v22)
        v25 = enum_init<core::panics::PanicResult::<(core::felt252)>, 1>(v24)
        v7 = store_temp<RangeCheck>(v7)
        v8 = store_temp<GasBuiltin>(v8)
        v25 = store_temp<core::panics::PanicResult::<(core::felt252)>>(v25)
        return (v7, v8, v25)
    } else {
        branch_align()
        v4, v9 = dup<felt252>(v4)
        v5 = store_temp<RangeCheck>(v5)
        if (felt252_is_zero(v9) == 0) {
            branch_align()
            drop<NonZero<felt252>>(v10)
            v3, v13 = dup<felt252>(v3)
            v14 = felt252_add(v2, v13)
            v15 = const_as_immediate<Const<felt252, 1>>()
            v16 = felt252_sub(v4, v15)
            v5 = store_temp<RangeCheck>(v5)
            v6 = store_temp<GasBuiltin>(v6)
            v3 = store_temp<felt252>(v3)
            v14 = store_temp<felt252>(v14)
            v16 = store_temp<felt252>(v16)
            v17, v18, v19 = function_call<user@examples::fib::fib>(v5, v6, v3, v14, v16)
            return (v17, v18, v19)
        } else {
            branch_align()
            drop<felt252>(v4)
            drop<felt252>(v3)
            v11 = struct_construct<Tuple<felt252>>(v2)
            v12 = enum_init<core::panics::PanicResult::<(core::felt252)>, 0>(v11)
            v5 = store_temp<RangeCheck>(v5)
            v6 = store_temp<GasBuiltin>(v6)
            v12 = store_temp<core::panics::PanicResult::<(core::felt252)>>(v12)
            return (v5, v6, v12)
        }
    }
}
    "#;
    assert_eq!(decompiler_output, expected_output);
}
