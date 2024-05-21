pub mod detector;
pub mod prototypes_detector;
pub mod strings_detector;

use crate::detectors::detector::Detector;
use crate::detectors::prototypes_detector::PrototypesDetector;
use crate::detectors::strings_detector::StringsDetector;

/// Returns a vector of all the instantiated detectors
pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(PrototypesDetector::new()),
        Box::new(StringsDetector::new()),
    ]
}
