## Tutorial : Create a new detector

#### Step 1 : Create the detector

For example, if you want to create a new detector `new_detector`, the first step is to create a new file `new_detector.rs` in [/lib/src/detectors](https://github.com/FuzzingLabs/sierra-analyzer/tree/master/lib/src/detectors) where you can set up the detector id, name, description and type. Each also detector has its own `detect` method that takes a `Decompiler` as input to which you can apply static analysis rules. 

```rs
use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};

#[derive(Debug)]
pub struct NewDetector;

impl NewDetector {
    /// Creates a new `NewDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for NewDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "new"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "New detector"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Returns a test message"
    }

    /// Returns the type of the detector
    /// You can choose between 3 different types : INFORMATIONAL, SECURITY & TESTING
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Run the detector
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let result = format!(
            "New detector",
        );

        result
    }
}
```

#### Step 2 : Add it to the detector list

Then you can edit the [/lib/src/detectors/mod.rs](https://github.com/FuzzingLabs/sierra-analyzer/blob/master/lib/src/detectors/mod.rs) file and add the newly created detector : 


```diff 
pub mod controlled_library_call_detector;
pub mod detector;
pub mod felt_overflow_detector;
pub mod functions_detector;
pub mod statistics_detector;
pub mod strings_detector;
pub mod tests_generator_detector;
+ pub mod new_detector;

use crate::detectors::controlled_library_call_detector::ControlledLibraryCallDetector;
use crate::detectors::detector::Detector;
use crate::detectors::felt_overflow_detector::FeltOverflowDetector;
use crate::detectors::functions_detector::FunctionsDetector;
use crate::detectors::statistics_detector::StatisticsDetector;
use crate::detectors::strings_detector::StringsDetector;
use crate::detectors::tests_generator_detector::TestsGeneratorDetector;
+use crate::detectors::new_detector::NewDetector;

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
        TestsGeneratorDetector,
        ControlledLibraryCallDetector,
        FeltOverflowDetector,
+        NewDetector
    )
}
```

#### More examples of detectors

You can find more examples of detectors [here](https://github.com/FuzzingLabs/sierra-analyzer/tree/master/lib/src/detectors). This directory contains simple detectors such as the [statistics detector](https://github.com/FuzzingLabs/sierra-analyzer/blob/master/lib/src/detectors/statistics_detector.rs) or more complex ones like the [felt overflow detector](https://github.com/FuzzingLabs/sierra-analyzer/blob/master/lib/src/detectors/felt_overflow_detector.rs).