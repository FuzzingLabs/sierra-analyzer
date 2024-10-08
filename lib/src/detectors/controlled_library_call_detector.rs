use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::extensions::lib_func::ParamSignature;
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::GenStatement;

use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};
use crate::parse_element_name;

#[derive(Debug)]
pub struct ControlledLibraryCallDetector;

impl ControlledLibraryCallDetector {
    /// Creates a new `ControlledLibraryCallDetector` instance
    pub fn new() -> Self {
        Self
    }
}

fn check_user_controlled(
    _formal_params: &[ParamSignature],
    _actual_params: Vec<VarId>,
    _function_name: &str,
) {
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
    ///
    /// WIP
    ///
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let result = String::new();

        for function in decompiler.functions.iter() {
            for statement in function.statements.clone() {
                if let GenStatement::Invocation(statement) = statement.statement {
                    let libfunc = decompiler
                        .registry()
                        .get_libfunc(&statement.libfunc_id)
                        .expect("Library function not found in the registry");

                    if let CoreConcreteLibfunc::FunctionCall(abi_function) = libfunc {
                        check_user_controlled(
                            &abi_function.signature.param_signatures,
                            statement.args.clone(),
                            parse_element_name!(function.function.id.clone()).as_str(),
                        );
                    }
                }
            }
        }

        result
    }
}
