/// Macro to parse the debug name from a libfunc or a function ID,
/// using the debug_name if present or falling back to the ID field
#[macro_export]
macro_rules! parse_element_name {
    ($libfunc_id:expr) => {
        if let Some(debug_name) = &$libfunc_id.debug_name {
            debug_name.to_string()
        } else {
            $libfunc_id.id.to_string()
        }
    };
}

/// Macro to extract parameters from the args field of a GenInvocation object.
/// It converts each parameter into a String, using the debug_name if available,
/// otherwise using the ID field
#[macro_export]
macro_rules! extract_parameters {
    ($args:expr) => {
        $args
            .iter()
            .map(|var_id| {
                if let Some(debug_name) = &var_id.debug_name {
                    // If debug_name exists, use it as parameter
                    debug_name.clone().into()
                } else {
                    // If debug_name is None, use id field as parameter
                    format!("v{}", var_id.id)
                }
            })
            .collect::<Vec<String>>()
    };
}
