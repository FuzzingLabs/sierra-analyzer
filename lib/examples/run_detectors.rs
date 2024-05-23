use sierra_analyzer_lib::detectors::get_detectors;
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

    // Get the detectors list
    let mut detectors = get_detectors();

    // Init the output
    let mut output = String::new();

    // Run all the detectors
    for detector in detectors.iter_mut() {
        let result = detector.detect(&mut decompiler);
        if !result.trim().is_empty() {
            // Each detector output is formatted like
            //
            // [Detector category] Detector name
            //      - detector content
            //      - ...
            output.push_str(&format!(
                "[{}] {}\n{}\n\n",
                detector.detector_type().as_str(),
                detector.name(),
                result
                    .lines()
                    .map(|line| format!("\t- {}", line))
                    .collect::<Vec<String>>()
                    .join("\n")
            ));
        }
    }

    // Print the detectors result
    println!("{}", output.trim());
}
