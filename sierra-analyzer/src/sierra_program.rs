use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::ProgramParser;

/// A struct that represents a Sierra program
#[allow(dead_code)]
pub struct SierraProgram {
    /// The parsed Sierra program
    program: Program,
}

impl SierraProgram {
    /// Creates a new `SierraProgram` instance by parsing the given Sierra code
    pub fn new(content: String) -> Self {
        let program = ProgramParser::new().parse(&content).unwrap();
        SierraProgram { program }
    }
}
