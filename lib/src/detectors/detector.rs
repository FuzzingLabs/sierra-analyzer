use crate::decompiler::decompiler::Decompiler;

/// Possible types of a detector
pub enum DetectorType {
    INFORMATIONAL,
    SECURITY,
}

/// Detector marker trait
pub trait Detector {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detector_type(&self) -> DetectorType;
    fn detect(&mut self, decompiler: &mut Decompiler) -> String;
}
