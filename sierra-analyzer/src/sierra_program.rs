use cairo_lang_sierra::program::GenericArg;
use cairo_lang_sierra::program::LibfuncDeclaration;
use cairo_lang_sierra::program::Program;
use cairo_lang_sierra::program::TypeDeclaration;
use cairo_lang_sierra::ProgramParser;

/// A struct that represents a Sierra program
#[derive(Debug)]
#[allow(dead_code)]
pub struct SierraProgram {
    /// The parsed Sierra program
    program: Program,
}

impl SierraProgram {
    /// Creates a new `SierraProgram` instance by parsing the given Sierra code
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
        let types = self.decompile_types();
        let libfuncs = self.decompile_libfuncs();

        // Using format! macro to concatenate strings
        format!("{}\n\n{}", types, libfuncs)
    }

    /// Parses generic arguments for both type & libfunc declarations
    fn parse_arguments(&self, generic_args: &[GenericArg]) -> String {
        generic_args
            .iter()
            .map(|arg| match arg {
                GenericArg::UserType(t) => {
                    // Use debug name
                    if let Some(name) = &t.debug_name {
                        format!("ut@{}", name)
                    }
                    // use ID
                    else {
                        format!("ut@[{}]", t.id)
                    }
                }
                GenericArg::Type(t) => t
                    .debug_name
                    .as_ref()
                    .map_or_else(String::new, |s| s.clone().into()),
                GenericArg::Value(t) => t.to_string(),
                _ => String::new(),
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    /// Decompiles the type declarations in the Sierra program
    fn decompile_types(&self) -> String {
        self.program
            .type_declarations
            .iter()
            .map(|type_declaration| self.decompile_type(type_declaration))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Decompiles a single type declaration
    fn decompile_type(&self, type_declaration: &TypeDeclaration) -> String {
        // Get the debug name of the type's ID
        let id = type_declaration
            .id
            .debug_name
            .as_ref()
            .expect("Type ID missing debug name");
        // Get the long ID of the type, which consists of the generic ID and any generic arguments
        let long_id = &type_declaration.long_id;
        let generic_id = long_id.generic_id.to_string();

        // Parse generic arguments
        let arguments = self.parse_arguments(&long_id.generic_args);

        // Construct a string representation of the long ID
        let long_id = format!("{}<{}>", generic_id, arguments);

        // Retrieve the declared type information for the type, if it exists
        let declared_type_info_str = type_declaration.declared_type_info.as_ref().map_or_else(
            String::new,
            |declared_type_info| {
                let storable = declared_type_info.storable.to_string();
                let droppable = declared_type_info.droppable.to_string();
                let duplicatable = declared_type_info.duplicatable.to_string();
                let zero_sized = declared_type_info.zero_sized.to_string();
                format!(
                    "[storable: {}, drop: {}, dup: {}, zero_sized: {}]",
                    storable, droppable, duplicatable, zero_sized
                )
            },
        );

        // Construct the type declaration string
        if declared_type_info_str.is_empty() {
            format!("type {} = {};", id, long_id)
        } else {
            format!("type {} = {} {};", id, long_id, declared_type_info_str)
        }
    }

    /// Decompiles the libfunc declarations in the Sierra program
    fn decompile_libfuncs(&self) -> String {
        self.program
            .libfunc_declarations
            .iter()
            .map(|libfunc_declaration| self.decompile_libfunc(libfunc_declaration))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Decompiles a single libfunc declaration
    fn decompile_libfunc(&self, libfunc_declaration: &LibfuncDeclaration) -> String {
        // Get the debug name of the libfunc's ID
        let id = libfunc_declaration.id.debug_name.as_ref().unwrap();
        // Get the long ID of the libfunc, which consists of the generic ID and any generic arguments
        let long_id = &libfunc_declaration.long_id;
        let generic_id = long_id.generic_id.to_string();

        // Parse generic arguments
        let arguments = self.parse_arguments(&long_id.generic_args);

        // Construct a string representation of the long ID
        let long_id = format!("{}<{}>", generic_id, arguments);

        format!("libfunc {} = {};", id, long_id)
    }
}
