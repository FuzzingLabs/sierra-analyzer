use sierra_analyzer_lib::detectors::detector::Detector;
use sierra_analyzer_lib::detectors::prototypes_detector::PrototypesDetector;
use sierra_analyzer_lib::detectors::statistics_detector::StatisticsDetector;
use sierra_analyzer_lib::detectors::strings_detector::StringsDetector;
use sierra_analyzer_lib::sierra_program::SierraProgram;

#[test]
fn test_string_detector() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib_array.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the strings detector
    let mut detector = StringsDetector::new();

    // Detected strings
    let detected_strings = detector.detect(&mut decompiler);

    let expected_output = r#"Index out of bounds
u32_sub Overflow"#;

    assert_eq!(detected_strings, expected_output);
}

#[test]
fn test_prototypes_detector() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib_array.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the prototypes detector
    let mut detector = PrototypesDetector::new();

    // functions prototypes
    let detected_prototypes = detector.detect(&mut decompiler);

    let expected_output = r#"func examples::fib_array::fib (v0: RangeCheck, v1: u32) -> (RangeCheck, core::panics::PanicResult::<((core::array::Array::<core::felt252>, core::felt252, core::integer::u32))>)
func examples::fib_array::fib_inner (v0: RangeCheck, v1: u32, v2: Array<felt252>) -> (RangeCheck, core::panics::PanicResult::<(core::array::Array::<core::felt252>, ())>)"#;

    assert_eq!(detected_prototypes, expected_output);
}

#[test]
fn test_statistics_detector() {
    // Read file content
    let content = include_str!("../../examples/sierra/fib_array.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);
    let use_color = false;
    decompiler.decompile(use_color);

    // Init the prototypes detector
    let mut detector = StatisticsDetector::new();

    // functions prototypes
    let statistics = detector.detect(&mut decompiler);

    let expected_output = r#"Libfuncs: 42
Types: 19
Functions: 2"#;

    assert_eq!(statistics, expected_output);
}
