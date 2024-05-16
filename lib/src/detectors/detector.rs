/// Possibles types of a detector
pub enum DetectorType {
    INFORMATIONAL,
    SECURITY,
}

/// Detector marker trait
pub trait Detector {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detector_type(&self) -> DetectorType;
    fn detect(&self) -> String;
}
