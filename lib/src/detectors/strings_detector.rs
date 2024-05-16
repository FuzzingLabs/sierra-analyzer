use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::libfuncs_patterns::CONST_REGEXES;
use crate::decompiler::utils::decode_hex_bigint;
use crate::detectors::detector::{Detector, DetectorType};
use crate::parse_element_name;
use cairo_lang_sierra::program::GenStatement;
use num_bigint::BigInt;

pub struct StringsDetector<'a> {
    decompiler: &'a mut Decompiler<'a>,
}

impl<'a> StringsDetector<'a> {
    /// Creates a new `StringsDetector` instance
    pub fn new(decompiler: &'a mut Decompiler<'a>) -> Self {
        Self { decompiler }
    }
}

impl<'a> Detector for StringsDetector<'a> {
    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Strings"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detects strings in the decompiled Sierra code."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Detects strings in the decompiled Sierra code and returns them as a single string
    fn detect(&mut self) -> String {
        // A vector to store the extracted strings
        let mut extracted_strings: Vec<String> = vec![];

        // Iterate over all the program statements
        for function in &self.decompiler.functions {
            for statement in &function.statements {
                let statement = &statement.statement;
                match statement {
                    GenStatement::Invocation(invocation) => {
                        let libfunc_id_str = parse_element_name!(invocation.libfunc_id); // Parse the ID of the invoked library function

                        // Iterate over the CONST_REGEXES and check if the input string matches
                        for regex in CONST_REGEXES.iter() {
                            if let Some(captures) = regex.captures(&libfunc_id_str) {
                                if let Some(const_value) = captures.name("const") {
                                    // Convert string to a BigInt in order to decode it
                                    let const_value_str = const_value.as_str();
                                    let const_value_bigint =
                                        BigInt::parse_bytes(const_value_str.as_bytes(), 10)
                                            .unwrap();

                                    // If the const integer can be decoded to a valid string, use the string as a comment
                                    if let Some(decoded_string) =
                                        decode_hex_bigint(&const_value_bigint)
                                    {
                                        // Add the decoded string to the vector
                                        extracted_strings.push(decoded_string);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Convert the extracted strings to a single string, separated by newline characters
        let result = extracted_strings.join("\n");

        result
    }
}
