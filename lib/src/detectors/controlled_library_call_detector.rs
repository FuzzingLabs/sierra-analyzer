use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};

#[derive(Debug)]
pub struct ControlledLibraryCallDetector;

impl ControlledLibraryCallDetector {
    /// Creates a new `ControlledLibraryCallDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for ControlledLibraryCallDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "controlled_library_call"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Controlled library call"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detect library calls with a user controlled class hash."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::SECURITY
    }

    /// Detect library calls with a user controlled class hash
    fn detect(&mut self, _decompiler: &mut Decompiler) -> String {
        let result = String::new();
        result
    }
}
