pub mod detector;
pub mod functions_detector;
pub mod statistics_detector;
pub mod strings_detector;

use crate::detectors::detector::Detector;
use crate::detectors::functions_detector::FunctionsDetector;
use crate::detectors::statistics_detector::StatisticsDetector;
use crate::detectors::strings_detector::StringsDetector;

/// Returns a vector of all the instantiated detectors
pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    vec![
        Box::new(FunctionsDetector::new()),
        Box::new(StringsDetector::new()),
        Box::new(StatisticsDetector::new()),
    ]
}
