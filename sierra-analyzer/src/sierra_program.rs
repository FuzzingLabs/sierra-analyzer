use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::ProgramParser;

use crate::decompiler::decompiler::Decompiler;

/// A struct that represents a Sierra program
#[allow(dead_code)]
pub struct SierraProgram {
    /// The parsed Sierra program
    program: Program,
}

impl SierraProgram {
    /// Creates a new `SierraProgram` instance by parsing the given Sierra code
    pub fn new(content: String) -> Self {
        let program = match ProgramParser::new().parse(&content) {
            Ok(program) => program,
            Err(err) => {
                panic!("Error parsing Sierra code: {}", err);
            }
        };

        SierraProgram { program }
    }

    /// Returns a reference to the parsed Sierra program
    pub fn program(&self) -> &Program {
        &self.program
    }

    /// Decompiles the Sierra program and returns a `Decompiler` instance
    pub fn decompiler(&self) -> Decompiler {
        Decompiler::new(self)
    }
}
