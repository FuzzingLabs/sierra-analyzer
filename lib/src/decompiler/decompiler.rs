use colored::*;

use cairo_lang_sierra::extensions::core::CoreLibfunc;
use cairo_lang_sierra::extensions::core::CoreType;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenericArg;
use cairo_lang_sierra::program::LibfuncDeclaration;
use cairo_lang_sierra::program::StatementIdx;
use cairo_lang_sierra::program::TypeDeclaration;
use cairo_lang_sierra::program_registry::ProgramRegistry;

use crate::config::GraphConfig;
use crate::decompiler::cfg::BasicBlock;
use crate::decompiler::cfg::EdgeType;
use crate::decompiler::function::Function;
use crate::decompiler::function::SierraStatement;
use crate::decompiler::libfuncs_patterns::IS_ZERO_REGEX;
use crate::decompiler::utils::decode_user_defined_type_id;
use crate::decompiler::utils::replace_types_id;
use crate::graph::callgraph::process_callgraph;
use crate::parse_element_name;
use crate::parse_element_name_with_fallback;
use crate::sierra_program::SierraProgram;

/// A struct that represents a decompiler for a Sierra program
pub struct Decompiler<'a> {
    /// A reference to the Sierra program to decompile
    pub sierra_program: &'a SierraProgram,
    /// Program functions
    pub functions: Vec<Function<'a>>,
    /// Program registry
    registry: &'a ProgramRegistry<CoreType, CoreLibfunc>,
    /// Current indentation
    indentation: u32,
    /// Already printed basic blocks (to avoid printing two times the same BB)
    printed_blocks: Vec<BasicBlock>,
    /// The function we are currently working on
    current_function: Option<Function<'a>>,
    /// Names of all declared types (in order)
    pub declared_types_names: Vec<String>,
    /// Names of all declared libfuncs (in order)
    pub declared_libfuncs_names: Vec<String>,
    /// Enable / disable the verbose output
    /// Some statements are not included in the regular output to improve the readability
    verbose: bool,
}

impl<'a> Decompiler<'a> {
    pub fn new(sierra_program: &'a SierraProgram, verbose: bool) -> Self {
        Decompiler {
            sierra_program,
            functions: Vec::new(),
            registry: sierra_program.registry(),
            indentation: 1,
            printed_blocks: Vec::new(),
            current_function: None,
            declared_types_names: Vec::new(),
            declared_libfuncs_names: Vec::new(),
            verbose,
        }
    }

    /// Returns a reference to the program registry
    pub fn registry(&self) -> &ProgramRegistry<CoreType, CoreLibfunc> {
        &self.registry
    }

    /// Decompiles the Sierra Program and return the string output
    /// Output can be colored or not
    pub fn decompile(&mut self, use_color: bool) -> String {
        // Disable/enable color output
        colored::control::set_override(use_color);

        // Decompile types and libfuncs
        let types = self.decompile_types();
        let libfuncs = self.decompile_libfuncs();

        // Load statements into their corresponding functions
        self.set_functions_offsets();
        self.decompile_functions_prototypes();
        self.add_statements_to_functions();

        // Decompile the functions
        let functions = self.decompile_functions();

        // Format the output string
        let mut output = String::new();
        if self.verbose {
            output.push_str(&types);
            output.push_str("\n\n");
            output.push_str(&libfuncs);
            output.push_str("\n\n");
        }
        output.push_str(&functions);
        output
    }

    /// Decompiles the type declarations
    fn decompile_types(&mut self) -> String {
        self.sierra_program
            .program()
            .type_declarations
            .iter()
            .map(|type_declaration| self.decompile_type(type_declaration))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Decompiles the libfunc declarations
    fn decompile_libfuncs(&mut self) -> String {
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
                // User defined types
                GenericArg::UserType(t) => {
                    // Use debug name
                    if let Some(name) = &t.debug_name {
                        format!("ut@{}", name)
                    }
                    // use ID
                    else {
                        // We first format as ut@[<type_id] it and then decode the user-defined types ID part in it if needed
                        if !self.verbose {
                            decode_user_defined_type_id(format!(
                                "ut@[{}]",
                                t.id.clone().to_string()
                            ))
                        }
                        // Don't decode the user-defined types IDs in verbose mode
                        else {
                            format!("ut@[{}]", t.id.clone().to_string())
                        }
                    }
                }
                // Builtin type
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
    fn decompile_type(&mut self, type_declaration: &TypeDeclaration) -> String {
        // Get the debug name of the type's ID
        let id = format!(
            "{}",
            type_declaration
                .id
                .debug_name
                .as_ref()
                .unwrap_or(&"".into())
        );

        // Get the long ID of the type
        let long_id = &type_declaration.long_id;
        let generic_id = long_id.generic_id.to_string();

        // Parse generic arguments
        let arguments = self.parse_arguments(&long_id.generic_args);

        // Construct a string representation of the long ID
        let long_id_repr = if !arguments.is_empty() {
            format!("{}<{}>", generic_id, arguments)
        } else {
            generic_id.clone()
        };

        // Conditionally format id and long_id_repr
        let (id_colored, long_id_repr_colored) = if id.is_empty() {
            (id.yellow(), long_id_repr.yellow().to_string())
        } else {
            (id.white(), long_id_repr.clone())
        };

        // Retrieve declared type information
        let _declared_type_info_str = type_declaration.declared_type_info.as_ref().map_or_else(
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

        // Construct the type definition string
        // If the id is not empty, format the type definition with the id and optionally the long ID representation
        let type_definition = if !id.is_empty() {
            let id_string = id.clone().to_string();
            self.declared_types_names.push(id_string.clone());
            format!(
                "type {}{}",
                id.yellow(),
                if long_id_repr_colored != id_colored.to_string() {
                    format!(" ({})", long_id_repr_colored)
                } else {
                    "".to_string()
                }
            )
        }
        // If the id is empty, format the type definition with only the long ID representation
        else {
            let long_id_repr_string = long_id_repr.clone().to_string();
            self.declared_types_names.push(long_id_repr_string.clone());
            format!("type {}{}", long_id_repr_colored, "")
        };

        type_definition
    }

    /// Decompiles an individual libfunc declaration
    fn decompile_libfunc(&mut self, libfunc_declaration: &LibfuncDeclaration) -> String {
        // Get the debug name of the libfunc's ID
        let id = format!(
            "{}",
            libfunc_declaration
                .id
                .debug_name
                .as_ref()
                .unwrap_or(&"".into())
        );

        // Get the long ID of the libfunc
        let long_id = &libfunc_declaration.long_id;

        // Parse kgeneric arguments
        let _arguments = self.parse_arguments(&libfunc_declaration.long_id.generic_args);

        // Construct the libfunc definition string
        let libfunc_definition = if id.is_empty() {
            long_id.to_string() // Use long_id if id is empty
        } else {
            id.to_string()
        };

        self.declared_libfuncs_names
            .push(libfunc_definition.clone()); // Push non-colored version to declared_libfuncs_names

        format!("libfunc {}", libfunc_definition.blue())
    }

    /// Decompiles the functions prototypes
    pub fn decompile_functions_prototypes(&mut self) -> String {
        let prototypes_and_arguments: Vec<(String, Vec<(String, String)>)> = self
            .sierra_program
            .program()
            .funcs
            .iter()
            .map(|function_prototype| self.decompile_function_prototype(function_prototype))
            .collect();

        // Set prototypes and arguments for corresponding Function structs
        for ((prototype, arguments), function) in prototypes_and_arguments
            .iter()
            .zip(self.functions.iter_mut())
        {
            function.set_prototype(prototype.clone());
            function.set_arguments(arguments.clone());
        }

        prototypes_and_arguments
            .iter()
            .map(|(prototype, _)| prototype.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Decompiles a function prototype and returns both the formatted prototype & the arguments
    fn decompile_function_prototype(
        &self,
        function_declaration: &GenFunction<StatementIdx>,
    ) -> (String, Vec<(String, String)>) {
        // Parse the function name
        let id = format!("{}", parse_element_name!(function_declaration.id)).bold();

        // Get the function signature, which consists of the parameter types and return types
        let signature = &function_declaration.signature;
        let param_types: Vec<String> = signature
            .param_types
            .iter()
            .map(|param_type| {
                // We use `parse_element_name_with_fallback` and not `parse_element_name` because
                // we try to match the type id with its corresponding name if it's a remote contract
                parse_element_name_with_fallback!(param_type, self.declared_types_names)
            })
            .collect();

        // Create a list of strings representing the function parameters
        // with each string formatted as "<param_name>: <param_type>"
        let param_strings: Vec<String> = param_types
            .iter()
            .zip(function_declaration.params.iter())
            .map(|(param_type, param)| {
                let param_name_string = if let Some(debug_name) = &param.id.debug_name {
                    debug_name.to_string()
                } else {
                    format!("v{}", param.id.id)
                };
                let param_name = param_name_string.purple(); // Color param_name in purple
                let param_type_colored = param_type.yellow(); // Color param_type in yellow
                format!("{}: {}", param_name, param_type_colored)
            })
            .collect();

        // Collect arguments as a vector of tuples
        let arguments: Vec<(String, String)> = param_types
            .iter()
            .zip(function_declaration.params.iter())
            .map(|(param_type, param)| {
                let param_name_string = if let Some(debug_name) = &param.id.debug_name {
                    debug_name.to_string()
                } else {
                    format!("v{}", param.id.id)
                };
                (param_name_string, param_type.clone())
            })
            .collect();

        // Join the parameter strings into a single string, separated by commas
        let param_str = format!("{}", param_strings.join(", "));

        // Create a list of strings representing the function return types
        let ret_types: Vec<String> = signature
            .ret_types
            .iter()
            .map(|ret_type| {
                let ret_type_string = if let Some(debug_name) = &ret_type.debug_name {
                    debug_name.to_string()
                } else {
                    // Replace id with the corresponding type name
                    format!("[{}]", self.declared_types_names[ret_type.id as usize])
                };
                let ret_type_colored = ret_type_string.purple(); // Color ret_type_string in purple
                ret_type_colored.to_string()
            })
            .collect();

        // Join the return type strings into a single string, separated by commas
        let ret_types_str = format!("{}", ret_types.join(", "));

        // Construct the function declaration string
        let prototype = format!("func {} ({}) -> ({})", id, param_str, ret_types_str);

        (prototype, arguments)
    }

    /// Sets the start and end offsets for each function in the Sierra program
    /// They are then used to assign the statements their functions
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

    /// Adds the corresponding statements each function using their offsets
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
                    // Function statements based on their offsets
                    if offset >= start_offset && offset <= end_offset {
                        Some(SierraStatement::new(statement.clone(), offset))
                    }
                    // Other statements
                    else {
                        None
                    }
                })
                .collect();

            function.set_statements(statements_with_offsets);
        }
    }

    /// Decompiles all the functions
    pub fn decompile_functions(&mut self) -> String {
        // Clone functions to avoid borrowing conflicts
        let mut functions_clone = self.functions.clone();

        // Initialize a CFG for each function
        for function in &mut functions_clone {
            function.create_cfg();
        }

        let function_decompilations: Vec<String> = functions_clone
            .iter()
            .enumerate()
            .map(|(index, function)| {
                // Set the current function
                self.current_function = Some(function.clone());

                // Extract function prototype
                let prototype = function
                    .prototype
                    .as_ref()
                    .expect("Function prototype not set");

                let body = if let Some(cfg) = &function.cfg {
                    cfg.basic_blocks
                        .iter()
                        .map(|block| {
                            self.indentation = 1; // Reset indentation after processing each block
                            self.basic_block_recursive(block)
                        })
                        .collect::<String>()
                } else {
                    String::new()
                };

                // Define bold braces for function body enclosure
                let bold_brace_open = "{".bold();
                let bold_brace_close = "}".bold();

                // Combine prototype and body into a formatted string
                let purple_comment = format!("// Function {}", index + 1).purple();
                format!(
                    "{}\n{} {}\n{}{}", // Added bold braces around the function body
                    purple_comment, prototype, bold_brace_open, body, bold_brace_close
                )
            })
            .collect();

        // Join all function decompilations into a single string
        function_decompilations.join("\n\n")
    }

    /// Recursively decompile basic blocks
    fn basic_block_recursive(&mut self, block: &BasicBlock) -> String {
        let mut basic_blocks_str = String::new();

        // Define bold braces once for use in formatting
        let bold_brace_open = "{".bold();
        let bold_brace_close = "}".bold();

        // Add the root basic block
        basic_blocks_str += &self.basic_block_to_string(block);

        // Add the edges
        for edge in &block.edges {
            // If branch
            if edge.edge_type == EdgeType::ConditionalTrue {
                // Indent the if block
                self.indentation += 1;

                if let Some(edge_basic_block) = self
                    .current_function
                    .as_ref()
                    .unwrap()
                    .cfg
                    .clone()
                    .unwrap()
                    .basic_blocks
                    .iter()
                    .find(|b| edge.destination == b.start_offset)
                {
                    basic_blocks_str += &self.basic_block_recursive(edge_basic_block);
                }
            }
            // Else branch
            else if edge.edge_type == EdgeType::ConditionalFalse {
                if let Some(edge_basic_block) = self
                    .current_function
                    .as_ref()
                    .unwrap()
                    .cfg
                    .clone()
                    .unwrap()
                    .basic_blocks
                    .iter()
                    .find(|b| edge.destination == b.start_offset)
                {
                    if !self.printed_blocks.contains(edge_basic_block) {
                        // End of if block
                        self.indentation -= 1;

                        let magenta_else = "else".magenta();
                        basic_blocks_str += &format!(
                            "{}{} {} {}{}\n",
                            "\t".repeat(self.indentation as usize),
                            bold_brace_close,
                            magenta_else,
                            bold_brace_open,
                            "\t".repeat(self.indentation as usize)
                        );

                        // Indent the else block
                        self.indentation += 1;

                        basic_blocks_str += &self.basic_block_recursive(edge_basic_block);
                    }
                }

                // End of else block
                self.indentation -= 1;

                if !basic_blocks_str.is_empty() {
                    basic_blocks_str += &format!(
                        "{}{}\n",
                        "\t".repeat(self.indentation as usize),
                        bold_brace_close
                    );
                }
            }
        }

        basic_blocks_str
    }

    /// Converts a Sierra BasicBlock object to a string
    fn basic_block_to_string(&mut self, block: &BasicBlock) -> String {
        // Check if the block has already been printed
        if self.printed_blocks.contains(block) {
            return String::new(); // Return an empty string if already printed
        }

        // Add the block to the list of printed blocks
        self.printed_blocks.push(block.clone());

        // Initialize the basic block string
        let mut decompiled_basic_block = String::new();
        let indentation = "\t".repeat(self.indentation as usize);

        // Append each statement to the string block
        for statement in &block.statements {
            // If condition
            if let Some(conditional_branch) =
                // We pass it the declared libfunc names to allow the method to reconstruct function calls
                // For remote contracts
                statement.as_conditional_branch(self.declared_libfuncs_names.clone())
            {
                if block.edges.len() == 2 {
                    let function_name = &conditional_branch.function;
                    let function_arguments = conditional_branch.parameters.join(", ");
                    decompiled_basic_block += &self.format_if_statement(
                        function_name,
                        function_arguments,
                        self.indentation as usize,
                    );
                }
            }
            // Unconditional jump
            else if let Some(_unconditional_branch) =
                // We pass it the declared libfunc names to allow the method to reconstruct function calls
                // For remote contracts
                statement.as_conditional_branch(self.declared_libfuncs_names.clone())
            {
                // Handle unconditional branch logic
                todo!()
            }
            // Default case
            else {
                // Add the formatted statements to the block
                // Some statements are only included in the verbose output
                //
                // We pass it the declared libfunc names & types names to allow the method
                // to reconstruct function calls & used types for remote contracts
                if let Some(formatted_statement) = statement.formatted_statement(
                    self.verbose,
                    self.declared_libfuncs_names.clone(),
                    self.declared_types_names.clone(),
                ) {
                    decompiled_basic_block += &format!("{}{}\n", indentation, formatted_statement);
                }
            }
        }

        decompiled_basic_block
    }

    /// Formats an `if` statement
    fn format_if_statement(
        &self,
        function_name: &str,
        function_arguments: String,
        indentation: usize,
    ) -> String {
        let magenta_if = "if".magenta();
        let bold_brace_open = "{".bold();
        let indentation_str = "\t".repeat(indentation);

        // Check if the function name matches the IS_ZERO_REGEX
        if IS_ZERO_REGEX.is_match(function_name) && !self.verbose {
            let argument = function_arguments.trim();
            return format!(
                "{}{} ({argument} == 0) {}{}\n",
                indentation_str,
                magenta_if,
                bold_brace_open,
                "\t".repeat(indentation + 1)
            );
        }

        format!(
            "{}{} ({}({}) == 0) {}{}\n",
            indentation_str,
            magenta_if,
            // Recover the type from type_id if it's a remote contract
            replace_types_id(&self.declared_types_names, function_name).blue(),
            function_arguments,
            bold_brace_open,
            "\t".repeat(indentation + 1) // Adjust for nested content indentation
        )
    }

    /// Filters the functions stored in the decompiler, retaining only the one that match
    /// the given function name
    pub fn filter_functions(&mut self, function_name: &str) {
        // Retain only those functions whose prototype contains the specified function name
        self.functions.retain(|function| {
            if let Some(proto) = &function.prototype {
                proto.contains(function_name)
            } else {
                false
            }
        });
    }

    /// Generate a callgraph representation in DOT Format
    #[inline]
    pub fn generate_callgraph(&mut self) -> String {
        process_callgraph(&self.functions)
    }

    /// Generates a control flow graph representation (CFG) in DOT format
    pub fn generate_cfg(&mut self) -> String {
        let mut dot = String::from("digraph {\n");

        // Global graph configuration
        dot.push_str(&format!(
            "\tgraph [fontname=\"{}\" fontsize={} layout={} newrank={} overlap={}];\n",
            GraphConfig::CFG_GRAPH_ATTR_FONTNAME,
            GraphConfig::CFG_GRAPH_ATTR_FONTSIZE,
            GraphConfig::CFG_GRAPH_ATTR_LAYOUT,
            GraphConfig::CFG_GRAPH_ATTR_NEWRANK,
            GraphConfig::CFG_GRAPH_ATTR_OVERLAP,
        ));
        // Global node configuration
        dot.push_str(&format!("\tnode [color=\"{}\" fillcolor=\"{}\" fontname=\"{}\" margin={} shape=\"{}\" style=\"{}\"];\n",
            GraphConfig::CFG_NODE_ATTR_COLOR,
            GraphConfig::CFG_NODE_ATTR_FILLCOLOR,
            GraphConfig::CFG_NODE_ATTR_FONTNAME,
            GraphConfig::CFG_NODE_ATTR_MARGIN,
            GraphConfig::CFG_NODE_ATTR_SHAPE,
            GraphConfig::CFG_NODE_ATTR_STYLE,
        ));
        // Global edge configuration
        dot.push_str(&format!("\tedge [arrowsize={} fontname=\"{}\" labeldistance={} labelfontcolor=\"{}\" penwidth={}];\n",
            GraphConfig::CFG_EDGE_ATTR_ARROWSIZE,
            GraphConfig::CFG_EDGE_ATTR_FONTNAME,
            GraphConfig::CFG_EDGE_ATTR_LABELDISTANCE,
            GraphConfig::CFG_EDGE_ATTR_LABELFONTCOLOR,
            GraphConfig::CFG_EDGE_ATTR_PENWIDTH,
        ));

        // Add a CFG representation for each function
        for function in &mut self.functions {
            function.create_cfg();
            if let Some(cfg) = &function.cfg {
                // Generate function subgraph
                let subgraph = cfg.generate_dot_graph();
                dot += &subgraph;
            }
        }

        // Add the closing curly braces to the DOT graph representation
        dot.push_str("}\n");

        dot
    }
}
