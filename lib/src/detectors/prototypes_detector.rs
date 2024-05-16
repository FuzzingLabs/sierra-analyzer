use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};
use cairo_lang_sierra::program::GenStatement;
use num_bigint::BigInt;

pub struct PrototypesDetector<'a> {
    decompiler: &'a mut Decompiler<'a>,
}

impl<'a> PrototypesDetector<'a> {
    /// Creates a new `PrototypesDetector` instance
    pub fn new(decompiler: &'a mut Decompiler<'a>) -> Self {
        Self { decompiler }
    }
}

impl<'a> Detector for PrototypesDetector<'a> {
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
    fn detect(&mut self) -> String {
        let result = self.decompiler.decompile_functions_prototypes();
        result
    }
}
