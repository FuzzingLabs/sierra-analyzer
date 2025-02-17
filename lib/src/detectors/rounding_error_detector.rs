use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::libfuncs_patterns::SAFE_MATH_REGEXES;
use crate::detectors::detector::{Detector, DetectorType};
use crate::parse_element_name_with_fallback;

use cairo_lang_sierra::program::GenStatement;

#[derive(Debug)]
pub struct RoundingErrorDetector;

impl RoundingErrorDetector {
    /// Creates a new `RoundingErrorDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for RoundingErrorDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "rounding"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Rounding error detector"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detect potential rounding errors."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::SECURITY
    }

    /// Detect potential rounding errors.
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();

        // We extract the functions names from the prototypes
        decompiler.decompile_functions_prototypes();
        for function in decompiler.functions.iter() {
            for statement in function.statements.clone() {
                if let GenStatement::Invocation(invocation) = &statement.statement {
                    // Parse the libfunc name used in the statement
                    let libfunc_name = parse_element_name_with_fallback!(
                        invocation.libfunc_id,
                        decompiler.declared_libfuncs_names
                    );

                    // Check if the libfunc_name matches any regex in SAFE_MATH_REGEXES
                    if SAFE_MATH_REGEXES
                        .iter()
                        .any(|regex| regex.is_match(&libfunc_name))
                    {
                        // Add the current function name to the result
                        if !result.is_empty() {
                            result.push_str(", ");
                        }

                        result.push_str(&format!(
                            "{} function could be vulnerable to a rounding error",
                            function.function.id
                        ));

                        // Move to the next function
                        break;
                    }
                }
            }
        }
        result
    }
}
