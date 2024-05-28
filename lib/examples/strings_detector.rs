use sierra_analyzer_lib::detectors::detector::Detector;
use sierra_analyzer_lib::detectors::strings_detector::StringsDetector;
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

    // Init the strings detector
    let mut detector = StringsDetector::new();

    // Print the detected strings
    println!("{}", detector.detect(&mut decompiler));
}
