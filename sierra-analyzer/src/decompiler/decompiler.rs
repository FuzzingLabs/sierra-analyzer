use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenericArg;
use cairo_lang_sierra::program::LibfuncDeclaration;
use cairo_lang_sierra::program::StatementIdx;
use cairo_lang_sierra::program::TypeDeclaration;

use crate::decompiler::function::Function;
use crate::decompiler::function::SierraStatement;
use crate::sierra_program::SierraProgram;

/// A struct that represents a decompiler for a Sierra program
pub struct Decompiler<'a> {
    /// A reference to the Sierra program to decompile
    sierra_program: &'a SierraProgram,
    /// Program functions
    pub functions: Vec<Function<'a>>,
}

impl<'a> Decompiler<'a> {
    pub fn new(sierra_program: &'a SierraProgram) -> Self {
        Decompiler {
            sierra_program,
            functions: Vec::new(),
        }
    }

    /// Decompiles the Sierra Program
    pub fn decompile(&mut self) -> String {
        let types = self.decompile_types();
        let libfuncs = self.decompile_libfuncs();

        // Load statements into their corresponding functions
        self.set_functions_offsets();
        let functions_prototypes = self.decompile_functions_prototypes();
        self.add_statements_to_functions();
        // Decompile the functions
        let functions = self.decompile_functions();

        // Using format! macro to concatenate strings
        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            types, libfuncs, functions, functions_prototypes
        )
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

    /// Decompiles the libfunc declarations
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

    /// Decompiles the functions prototypes
    fn decompile_functions_prototypes(&mut self) -> String {
        let prototypes: Vec<String> = self
            .sierra_program
            .program()
            .funcs
            .iter()
            .map(|function_prototype| self.decompile_function_prototype(function_prototype))
            .collect();

        // Set prototypes for corresponding Function structs
        for (prototype, function) in prototypes.iter().zip(self.functions.iter_mut()) {
            function.set_prototype(prototype.clone());
        }

        prototypes.join("\n")
    }

    /// Decompiles a single function prototype
    fn decompile_function_prototype(
        &self,
        function_declaration: &GenFunction<StatementIdx>,
    ) -> String {
        // Get the debug name of the function's ID
        let id = function_declaration
            .id
            .debug_name
            .as_ref()
            .expect("Function ID missing debug name");

        // Get the function signature, which consists of the parameter types and return types
        let signature = &function_declaration.signature;
        let param_types: Vec<String> = signature
            .param_types
            .iter()
            .map(|param_type| {
                param_type
                    .debug_name
                    .as_ref()
                    .unwrap_or(&format!("[{}]", param_type.id).into())
                    .to_string()
            })
            .collect();

        // Create a list of strings representing the function parameters,
        // with each string formatted as "<param_name>: <param_type>"
        let param_strings: Vec<String> = param_types
            .iter()
            .zip(function_declaration.params.iter())
            .map(|(param_type, param)| {
                format!(
                    "{}: {}",
                    param
                        .id
                        .debug_name
                        .as_ref()
                        .unwrap_or(&format!("v{}", param.id.id).into()),
                    param_type
                )
            })
            .collect();

        // Join the parameter strings into a single string, separated by commas
        let param_str = format!("{}", param_strings.join(", "));

        // Create a list of strings representing the function return types
        let ret_types: Vec<String> = signature
            .ret_types
            .iter()
            .map(|ret_type| {
                ret_type
                    .debug_name
                    .as_ref()
                    .unwrap_or(&format!("[{}]", ret_type.id).into())
                    .to_string()
            })
            .collect();

        // Join the return type strings into a single string, separated by commas
        let ret_types_str = format!("{}", ret_types.join(", "));

        // Construct the function declaration string
        format!("func {}({}) -> ({})", id, param_str, ret_types_str)
    }

    /// Sets the start and end offsets for each function in the Sierra program
    fn set_functions_offsets(&mut self) {
        let num_functions = self.sierra_program.program().funcs.len();

        for (i, function_declaration) in self.sierra_program.program().funcs.iter().enumerate() {
            let mut function = Function::new(function_declaration);
            function.set_start_offset(function_declaration.entry_point.0.try_into().unwrap());

            // Set the end offset of the current function to the start offset of the next function minus one
            if i < num_functions - 1 {
                let next_function_declaration = &self.sierra_program.program().funcs[i + 1];
                let next_start_offset: u32 =
                    next_function_declaration.entry_point.0.try_into().unwrap();
                function.set_end_offset(next_start_offset - 1);
            }

            self.functions.push(function);
        }

        // Set the end offset of the last function to the total number of statements
        if let Some(last_function) = self.functions.last_mut() {
            let total_statements = self.sierra_program.program().statements.len() as u16;
            last_function.set_end_offset(total_statements.into());
        }
    }

    /// Adds the corresponding statements and offsets to each function in the Sierra program
    fn add_statements_to_functions(&mut self) {
        for function in &mut self.functions {
            let start_offset = function.start_offset.unwrap();
            let end_offset = function.end_offset.unwrap();

            // Filter statements based on offset range and map them with their offsets
            let statements_with_offsets: Vec<SierraStatement> = self
                .sierra_program
                .program()
                .statements
                .iter()
                .enumerate()
                .filter_map(|(idx, statement)| {
                    let offset = idx as u32;
                    if offset >= start_offset && offset < end_offset {
                        Some(SierraStatement::new(statement.clone(), offset))
                    } else {
                        None
                    }
                })
                .collect();

            function.set_statements(statements_with_offsets);
        }
    }

    /// Decompiles the functions
    pub fn decompile_functions(&mut self) -> String {
        // Initialize CFG for each function
        for function in &mut self.functions {
            function.create_cfg();
        }

        // Collect function decompilations
        let function_decompilations: Vec<String> = {
            let functions = &self.functions;
            functions
                .iter()
                .map(|function| {
                    let prototype = function
                        .prototype
                        .as_ref()
                        .expect("Function prototype not set");
                    let body = function.statements_as_string();
                    let indented_body = body
                        .lines()
                        .map(|line| format!("    {}", line))
                        .collect::<Vec<String>>()
                        .join("\n");
                    format!("{} {{\n{}\n}}", prototype, indented_body)
                })
                .collect()
        };

        // Return function decompilations joined by newlines
        function_decompilations.join("\n\n")
    }
}
