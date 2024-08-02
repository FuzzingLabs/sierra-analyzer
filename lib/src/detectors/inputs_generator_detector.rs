use z3::{Config, Context};

use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};
use crate::sym_exec::sym_exec::SymbolicExecution;

#[derive(Debug)]
pub struct InputsGeneratorDetector;

impl InputsGeneratorDetector {
    /// Creates a new `InputsGeneratorDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for InputsGeneratorDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "inputs"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Inputs generator"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Generate inputs for a sierra function"
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Returns all the functions names
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let result = String::new();

        for function in &mut decompiler.functions {
            // Filter arguments whose type is `felt252`
            let felt252_arguments: Vec<&(String, String)> = function
                .arguments
                .iter()
                .filter(|(_, arg_type)| arg_type == "felt252")
                .collect();

            // Skip the function if there are no `felt252` arguments
            if felt252_arguments.is_empty() {
                continue;
            }

            // Generate the function CFG
            function.create_cfg();

            let function_paths = function.cfg.as_ref().unwrap().paths();
            for _path in &function_paths {
                // Create a new symbolic execution engine for the function
                let cfg = Config::new();
                let context = Context::new(&cfg);
                let _sym_exec = SymbolicExecution::new(&context);

                // TODO: Solve the constraints and get the function parameters values
                todo!();
            }
        }

        result
    }
}
