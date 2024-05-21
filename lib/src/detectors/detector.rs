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
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detector_type(&self) -> DetectorType;
    fn detect(&mut self, decompiler: &mut Decompiler) -> String;
}
