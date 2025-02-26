use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::function::FunctionType;
use crate::detectors::detector::{Detector, DetectorType};

#[derive(Debug)]
pub struct L1HandlerDetector;

impl L1HandlerDetector {
    /// Creates a new `L1HandlerDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for L1HandlerDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "l1_handler"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "L1 Handler"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detects the L1 handler functions."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Returns all l1 hanler functions
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();

        // We extract the functions names from the prototypes
        decompiler.decompile_functions_prototypes();

        for function in decompiler.functions.clone() {
            // Detect L1 Handler functions
            if let Some(function_type) = function.function_type {
                if matches!(function_type, FunctionType::L1Handler) {
                    if let Some(debug_name) = &function.function.id.debug_name {
                        result += &debug_name.to_string();
                        result += "\n";
                    }
                }
            }
        }

        result
    }
}
