use std::fmt;

use cairo_lang_sierra::program::GenericArg;
use cairo_lang_sierra::program::LibfuncDeclaration;
use cairo_lang_sierra::program::TypeDeclaration;

use crate::sierra_program::SierraProgram;

/// A struct that represents a decompiler for a Sierra program
pub struct Decompiler<'a> {
    /// A reference to the Sierra program to decompile
    sierra_program: &'a SierraProgram,
}

impl<'a> Decompiler<'a> {
    pub fn new(sierra_program: &'a SierraProgram) -> Self {
        Decompiler { sierra_program }
    }

    /// Decompiles the Sierra Program 
    pub fn decompile(&self) -> String {
        let types = self.decompile_types();
        let libfuncs = self.decompile_libfuncs();

        // Using format! macro to concatenate strings
        format!("{}\n\n{}", types, libfuncs)
    }

    /// Decompiles the type declarations
    fn decompile_types(&self) -> String {
        self.sierra_program
            .program()
            .type_declarations
            .iter()
            .map(|type_declaration| self.decompile_type(type_declaration))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Decompiles the libfunc declarations in the Sierra program
    fn decompile_libfuncs(&self) -> String {
        self.sierra_program
            .program()
            .libfunc_declarations
            .iter()
            .map(|libfunc_declaration| self.decompile_libfunc(libfunc_declaration))
            .collect::<Vec<String>>()
            .join("\n")
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

impl<'a> fmt::Display for Decompiler<'a> {
    /// Formats the decompiled Sierra program as a string.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.decompile())
    }
}
