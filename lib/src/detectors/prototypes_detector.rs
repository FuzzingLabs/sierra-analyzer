use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};

#[derive(Debug)]
pub struct PrototypesDetector;

impl PrototypesDetector {
    /// Creates a new `PrototypesDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for PrototypesDetector {
    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Functions Prototypes"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Returns the functions prototypes."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Returns all the functions prototypes
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let result = decompiler.decompile_functions_prototypes();
        result
    }
}
