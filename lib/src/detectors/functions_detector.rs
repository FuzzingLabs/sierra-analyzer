use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::function::FunctionType;
use crate::detectors::detector::{Detector, DetectorType};

#[derive(Debug)]
pub struct FunctionsDetector;

impl FunctionsDetector {
    /// Creates a new `FunctionsDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for FunctionsDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "functions"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Functions names"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Returns the user-defined functions names."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Returns all the functions names
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();

        // We extract the functions names from the prototypes
        decompiler.decompile_functions_prototypes();
        let total_functions = decompiler.functions.len();
        for (index, function) in decompiler.functions.iter().enumerate() {
            if let Some(prototype) = &function.prototype {
                // Remove the "func " prefix and then split at the first space
                let stripped_prototype = &prototype[5..];
                if let Some(first_space_index) = stripped_prototype.find(' ') {
                    let function_name = &stripped_prototype[..first_space_index];

                    // Put the function type in the output
                    let function_type = match function.function_type {
                        Some(FunctionType::External) => "External",
                        Some(FunctionType::View) => "View",
                        Some(FunctionType::Private) => "Private",
                        Some(FunctionType::Constructor) => "Constructor",
                        Some(FunctionType::Event) => "Event",
                        Some(FunctionType::Storage) => "Storage",
                        Some(FunctionType::Wrapper) => "Wrapper",
                        Some(FunctionType::Core) => "Core",
                        Some(FunctionType::AbiCallContract) => "AbiCallContract",
                        Some(FunctionType::AbiLibraryCall) => "AbiLibraryCall",
                        Some(FunctionType::L1Handler) => "L1Handler",
                        Some(FunctionType::Loop) => "Loop",
                        None => "",
                    };

                    result += &format!("{} : {}", function_type, function_name);
                }
                // Add a newline if it's not the last function
                if index < total_functions - 1 {
                    result += "\n";
                }

                // Append the ANSI reset sequence
                result += &"\x1b[0m".to_string();
            }
        }
        result
    }
}
