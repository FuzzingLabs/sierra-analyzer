use crate::decompiler::decompiler::Decompiler;
use std::fmt::Debug;

/// Possible types of a detector
#[derive(Debug)]
pub enum DetectorType {
    INFORMATIONAL,
    SECURITY,
}

/// Detector marker trait
pub trait Detector: Debug {
    // The id of a detector is used to select it using a command-line argument
    // e.g. the id of the detector with the name "Protoypes detector" is "prototypes"
    fn id(&self) -> &'static str;
    // Name of the detector
    fn name(&self) -> &'static str;
    // Description of the detector
    fn description(&self) -> &'static str;
    // A detector can be either a security detector or an informational detector
    fn detector_type(&self) -> DetectorType;
    // Run the detector on the
    fn detect(&mut self, decompiler: &mut Decompiler) -> String;
}
