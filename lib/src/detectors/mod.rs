pub mod detector;
pub mod functions_detector;
pub mod inputs_generator_detector;
pub mod statistics_detector;
pub mod strings_detector;

use crate::detectors::detector::Detector;
use crate::detectors::functions_detector::FunctionsDetector;
use crate::detectors::inputs_generator_detector::InputsGeneratorDetector;
use crate::detectors::statistics_detector::StatisticsDetector;
use crate::detectors::strings_detector::StringsDetector;

/// Macro to create a vector of detectors
macro_rules! create_detectors {
    ($($detector:ty),*) => {
        vec![
            $(
                Box::new(<$detector>::new()),
            )*
        ]
    };
}

/// Returns a vector of all the instantiated detectors
pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    create_detectors!(
        FunctionsDetector,
        StringsDetector,
        StatisticsDetector,
        InputsGeneratorDetector
    )
}
