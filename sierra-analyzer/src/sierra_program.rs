use cairo_lang_sierra::program::GenericArg;
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::ProgramParser;

/// A struct that represents a Sierra program
#[derive(Debug)]
#[allow(dead_code)]
pub struct SierraProgram {
    /// The parsed Sierra program
    program: Program,
}

impl SierraProgram {
    // Creates a new `SierraProgram` instance by parsing the given Sierra code
    pub fn new(content: String) -> Self {
        let program = match ProgramParser::new().parse(&content) {
            Ok(program) => program,
            Err(err) => {
                panic!("Error parsing Sierra code: {}", err);
            }
        };
        SierraProgram { program }
    }

    // Decompiles the Sierra program and returns a string representation of the decompiled code
    pub fn decompile(&self) -> String {
        let mut output = String::new();

        let types = self.decompile_types();
        output += &types;

        output
    }

    /// Decompiles the type declarations in the Sierra program
    fn decompile_types(&self) -> String {
        let mut decompiled_types = String::new();

        // Get the type declarations from the program
        let type_declarations = &self.program.type_declarations;

        // Iterate over each type declaration and decompile it
        for (i, type_declaration) in type_declarations.iter().enumerate() {
            // Get the debug name of the type's ID
            let id = type_declaration.id.debug_name.as_ref().unwrap();
            // Get the long ID of the type, which consists of the generic ID and any generic arguments
            let long_id = &type_declaration.long_id;
            let generic_id = long_id.generic_id.to_string();

            // Extract the debug names of any generic arguments
            let debug_name: Vec<_> = long_id
                .generic_args
                .iter()
                .filter_map(|arg| match arg {
                    GenericArg::UserType(aya) => Some(aya.debug_name.as_ref().unwrap().clone()),
                    GenericArg::Type(aya) => Some(aya.debug_name.as_ref().unwrap().clone()),
                    _ => None,
                })
                .collect();

            // Construct a string representation of the long ID
            let long_id = format!("{}<{}>", generic_id, debug_name.join(", "));

            // Retrieve the declared type information for the type, if it exists
            let declared_type_info_str =
                if let Some(declared_type_info) = &type_declaration.declared_type_info {
                    let storable = declared_type_info.storable.to_string();
                    let droppable = declared_type_info.droppable.to_string();
                    let duplicatable = declared_type_info.duplicatable.to_string();
                    let zero_sized = declared_type_info.zero_sized.to_string();
                    format!(
                        "[storable: {}, drop: {}, dup: {}, zero_sized: {}]",
                        storable, droppable, duplicatable, zero_sized
                    )
                } else {
                    String::new()
                };

            // Append the decompiled type declaration to the output string
            if declared_type_info_str.is_empty() {
                decompiled_types += &format!("type {} = {};", id, long_id);
            } else {
                decompiled_types +=
                    &format!("type {} = {} {};", id, long_id, declared_type_info_str);
            }

            // Add a newline character between lines, but not at the end of the string
            if i < type_declarations.len() - 1 {
                decompiled_types.push('\n');
            }
        }

        decompiled_types
    }
}
