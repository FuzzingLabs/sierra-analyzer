use colored::*;
use num_bigint::BigInt;

use cairo_lang_sierra::program::BranchTarget;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program::StatementIdx;

use crate::decompiler::cfg::ControlFlowGraph;
use crate::decompiler::cfg::SierraConditionalBranch;
use crate::decompiler::libfuncs_patterns::{
    ADDITION_REGEX, ARRAY_APPEND_REGEX, CONST_REGEXES, DROP_REGEX, DUP_REGEX, FUNCTION_CALL_REGEX,
    MULTIPLICATION_REGEX, NEW_ARRAY_REGEX, STORE_TEMP_REGEX, SUBSTRACTION_REGEX,
    VARIABLE_ASSIGNMENT_REGEX,
};
use crate::decompiler::utils::decode_hex_bigint;
use crate::decompiler::utils::replace_types_id;
use crate::extract_parameters;
use crate::parse_element_name;
use crate::parse_element_name_with_fallback;

/// A struct representing a statement
#[derive(Debug, Clone)]
pub struct SierraStatement {
    /// Statement extracted from the parsed program
    pub statement: cairo_lang_sierra::program::Statement,
    /// We store the statement offset in the struct because it not present in cairo_lang_sierra::program::Statement
    pub offset: u32,
    /// A statement is considered a "conditional branch" if it has branching behavior
    pub is_conditional_branch: bool,
}

impl SierraStatement {
    /// Creates a new `SierraStatement` instance
    pub fn new(statement: cairo_lang_sierra::program::Statement, offset: u32) -> Self {
        // Check if it is a conditional branch
        let is_conditional_branch = if let GenStatement::Invocation(invocation) = &statement {
            invocation
                .branches
                .iter()
                .any(|branch| matches!(branch.target, BranchTarget::Statement(_)))
        } else {
            false
        };

        Self {
            statement,
            offset,
            is_conditional_branch,
        }
    }

    /// Formats the statement as a string
    /// We try to format them in a way that is as similar as possible to the Cairo syntax
    pub fn formatted_statement(
        &self,
        verbose: bool,
        declared_libfuncs_names: Vec<String>,
        declared_types_names: Vec<String>,
    ) -> Option<String> {
        match &self.statement {
            // Return statements
            GenStatement::Return(vars) => {
                let mut formatted = "return".red().to_string();
                formatted.push_str(" (");
                for (index, var) in vars.iter().enumerate() {
                    if index > 0 {
                        formatted.push_str(", ");
                    }
                    formatted.push_str(&format!("v{}", var.id));
                }
                formatted.push_str(")");
                Some(formatted)
            }
            // Invocation statements
            GenStatement::Invocation(invocation) => {
                // Try to get the debug name of the libfunc_id
                // We use `parse_element_name_with_fallback` and not `parse_element_name` because
                // we try to match the libfunc id with it's corresponding name if it's a remote contract
                let libfunc_id = parse_element_name_with_fallback!(
                    invocation.libfunc_id,
                    declared_libfuncs_names
                );

                if !Self::is_function_allowed(&libfunc_id, verbose) {
                    return None; // Skip formatting if function is not allowed
                }
                let libfunc_id_str = libfunc_id.blue();

                let parameters = extract_parameters!(invocation.args);
                let assigned_variables = extract_parameters!(&invocation
                    .branches
                    .first()
                    .map(|branch| &branch.results)
                    .unwrap_or(&vec![]));
                let assigned_variables_str = if !assigned_variables.is_empty() {
                    assigned_variables.join(", ")
                } else {
                    String::new()
                };

                if STORE_TEMP_REGEX.is_match(&libfunc_id)
                    && assigned_variables_str == parameters.join(", ")
                    // Print the redundant store_temp in the verbose output
                    && !verbose
                {
                    return None; // Do not format if it's a redundant store_temp
                }

                Some(Self::invocation_formatting(
                    &assigned_variables_str,
                    &libfunc_id_str,
                    &parameters,
                    &verbose,
                    &declared_types_names,
                ))
            }
        }
    }

    /// Checks if the given function name is allowed to be included in the formatted statement
    fn is_function_allowed(function_name: &str, verbose: bool) -> bool {
        // We allow every function in the verbose output
        if verbose {
            return true;
        }

        match function_name {
            "branch_align"
            | "disable_ap_tracking"
            | "enable_ap_tracking"
            | "finalize_locals"
            | "revoke_ap_tracking"
            | "get_builtin_costs" => false,
            _ => {
                // Check blacklisted functions patterns
                if DROP_REGEX.is_match(function_name) {
                    false
                } else {
                    true
                }
            }
        }
    }

    /// Formats an invocation statement
    fn invocation_formatting(
        assigned_variables_str: &str,
        libfunc_id_str: &str,
        parameters: &[String],
        verbose: &bool,
        declared_types_names: &Vec<String>,
    ) -> String {
        // Replace types id in libfuncs names by their types names equivalents in remote contracts
        let binding = replace_types_id(declared_types_names, &libfunc_id_str);
        let libfunc_id_str = binding.as_str();

        // Join parameters for general use
        let parameters_str = parameters.join(", ");

        // Handling user-defined function calls
        if let Some(caps) = FUNCTION_CALL_REGEX.captures(libfunc_id_str) {
            if let Some(inner_func) = caps.get(1) {
                let formatted_func = inner_func.as_str();
                if !assigned_variables_str.is_empty() {
                    return format!(
                        "{} = {}({})",
                        assigned_variables_str,
                        formatted_func.blue(),
                        parameters_str
                    );
                } else {
                    return format!("{}({})", formatted_func.blue(), parameters_str);
                }
            }
        }

        if *verbose {
            // If verbose is true, return the invocation as is
            if assigned_variables_str.is_empty() {
                return format!("{}({})", libfunc_id_str.blue(), parameters_str);
            } else {
                return format!(
                    "{} = {}({})",
                    assigned_variables_str,
                    libfunc_id_str.blue(),
                    parameters_str
                );
            }
        }

        // Handling variables duplications
        // In the Sierra IR it it represented like : v1, v2 = dup<felt252>(v1)
        // But we can represent it as a variable assignment such as : v2 = v1
        if DUP_REGEX.is_match(libfunc_id_str) {
            if let Some((first_var, second_var)) = assigned_variables_str.split_once(", ") {
                return format!("{} = {}", second_var, first_var);
            }
        }

        // Handling variables assignments
        if VARIABLE_ASSIGNMENT_REGEX
            .iter()
            .any(|regex| regex.is_match(libfunc_id_str))
        {
            if let Some(old_var) = parameters.first().cloned() {
                let assigned_variable = assigned_variables_str.to_string();
                return format!("{} = {}", assigned_variable, old_var);
            }
        }

        // Handling array declarations
        // <variable> = Array<<array type>>::new()
        if let Some(captures) = NEW_ARRAY_REGEX.captures(libfunc_id_str) {
            if let Some(array_type) = captures.get(1) {
                let formatted_array_type = array_type.as_str();

                let final_array_type = formatted_array_type;

                // Return the formatted array declaration string
                return format!(
                    "{} = {}<{}>::{}()",
                    assigned_variables_str,
                    "Array".blue(),
                    final_array_type,
                    "new".blue()
                );
            }
        }

        // Handling array append operations
        // <variable> = <array>.append(<variable>)
        if ARRAY_APPEND_REGEX.is_match(libfunc_id_str) {
            let array_name = parameters[0].clone();
            let appent_value_name = parameters[1].clone();
            return format!(
                "{} = {}.{}({})",
                assigned_variables_str,
                array_name,
                "append".blue(),
                appent_value_name
            );
        }

        // Handling const declarations
        for regex in CONST_REGEXES.iter() {
            if let Some(captures) = regex.captures(libfunc_id_str) {
                if let Some(const_value) = captures.name("const") {
                    // Convert string to a BigInt in order to decode it
                    let const_value_str = const_value.as_str();
                    let const_value_bigint =
                        BigInt::parse_bytes(const_value_str.as_bytes(), 10).unwrap();

                    // If the const integer can be decoded to a valid string, use the string as a comment
                    if let Some(decoded_string) = decode_hex_bigint(&const_value_bigint) {
                        let string_comment = format!(r#"// "{}""#, decoded_string).green();
                        return format!(
                            "{} = {} {}",
                            assigned_variables_str, const_value_str, string_comment
                        );
                    }
                    // If the string can not be decoded as a valid string
                    else {
                        return format!("{} = {}", assigned_variables_str, const_value_str);
                    }
                }
            }
        }

        // Handling arithmetic operations
        let operator = if ADDITION_REGEX.is_match(libfunc_id_str) {
            "+"
        } else if SUBSTRACTION_REGEX.is_match(libfunc_id_str) {
            "-"
        } else if MULTIPLICATION_REGEX.is_match(libfunc_id_str) {
            "*"
        } else {
            // Return default formatting if no special formatting is applicable
            return if !assigned_variables_str.is_empty() {
                format!(
                    "{} = {}({})",
                    assigned_variables_str,
                    libfunc_id_str.blue(),
                    parameters_str
                )
            } else {
                format!("{}({})", libfunc_id_str.blue(), parameters_str)
            };
        };

        // Format arithmetic operations more explicitly
        format!(
            "{} = {}",
            assigned_variables_str,
            parameters
                .iter()
                .map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join(&format!(" {} ", operator))
        )
    }

    /// Return the raw statement, as in the original sierra file
    /// Used in the CFG
    #[inline]
    pub fn raw_statement(&self) -> String {
        self.statement.to_string()
    }

    /// Returns a reference to this statement as a conditional branch if it is one
    pub fn as_conditional_branch(
        &self,
        declared_libfuncs_names: Vec<String>,
    ) -> Option<SierraConditionalBranch> {
        if self.is_conditional_branch {
            if let GenStatement::Invocation(invocation) = &self.statement {
                // Statement
                let statement = self.statement.clone();

                // Function name
                let libfunc_id_str = invocation
                    .libfunc_id
                    .debug_name
                    .as_ref()
                    .map(|name| name.to_string())
                    // If the debug name is not present, try to get the name from declared_libfuncs_names
                    .or_else(|| {
                        declared_libfuncs_names
                            .get(invocation.libfunc_id.id as usize)
                            .map(|name| name.to_string())
                            // If neither the debug name nor the name from declared_libfuncs_names is present,
                            // format the id as a string
                            .or_else(|| Some(format!("[{}]", invocation.libfunc_id.id)))
                    })
                    .unwrap();

                // Parameters
                let parameters = extract_parameters!(invocation.args);

                // Fallthrough
                let fallthrough = invocation
                    .branches
                    .iter()
                    .any(|branch| matches!(branch.target, BranchTarget::Fallthrough));

                // Initialize edge offsets
                let mut edge_1_offset: Option<u32> = None;
                let mut edge_2_offset: Option<u32> = None;

                // Handle fallthrough case
                if fallthrough {
                    if let Some(statement_idx) = invocation.branches.iter().find_map(|branch| {
                        if let BranchTarget::Statement(statement_idx) = &branch.target {
                            Some(statement_idx.0 as u32)
                        } else {
                            None
                        }
                    }) {
                        edge_1_offset = Some(statement_idx);
                    }
                } else {
                    // Handle non-fallthrough case
                    if let Some(first_branch) = invocation.branches.iter().next() {
                        if let BranchTarget::Statement(statement_idx) = &first_branch.target {
                            edge_1_offset = Some(statement_idx.0 as u32);
                        }
                    }

                    // Set edge_2_offset if there are two branches pointing to a statement_idx
                    if let Some(second_statement_idx) = invocation
                        .branches
                        .iter()
                        .filter_map(|branch| {
                            if let BranchTarget::Statement(statement_idx) = &branch.target {
                                Some(statement_idx.0 as u32)
                            } else {
                                None
                            }
                        })
                        .nth(1)
                    {
                        edge_2_offset = Some(second_statement_idx);
                    }
                }

                // Create and return SierraConditionalBranch instance
                return Some(SierraConditionalBranch::new(
                    SierraStatement::new(statement, self.offset),
                    libfunc_id_str,
                    parameters,
                    edge_1_offset,
                    edge_2_offset,
                    fallthrough,
                ));
            }
        }

        None
    }
}

/// A struct representing a function in a Sierra program
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Function<'a> {
    /// The function's `GenFunction` representation
    pub function: &'a GenFunction<StatementIdx>,
    // Function start offset
    pub start_offset: Option<u32>,
    // Function end offset
    pub end_offset: Option<u32>,
    /// A vector of `SierraStatement` instances representing the function's body with offsets
    pub statements: Vec<SierraStatement>,
    /// A `ControlFlowGraph` representing the function's CFG
    pub cfg: Option<ControlFlowGraph>,
    /// The prototype of the function
    pub prototype: Option<String>,
}

impl<'a> Function<'a> {
    /// Creates a new `Function` instance
    pub fn new(function: &'a GenFunction<StatementIdx>) -> Self {
        Self {
            function,
            statements: Vec::new(),
            start_offset: None,
            end_offset: None,
            cfg: None,
            prototype: None,
        }
    }

    /// Initializes the control flow graph (CFG) for the function
    pub fn create_cfg(&mut self) {
        // Create a new control flow graph instance
        let mut cfg = ControlFlowGraph::new(
            parse_element_name!(self.function.id.clone()),
            self.statements.clone(),
        );

        // Generate the CFG basic blocks
        cfg.generate_basic_blocks();

        // Assign the control flow graph to the function's CFG field
        self.cfg = Some(cfg);
    }

    /// Sets the start offset of the function
    #[inline]
    pub fn set_start_offset(&mut self, start_offset: u32) {
        self.start_offset = Some(start_offset);
    }

    /// Sets the end offset of the function
    #[inline]
    pub fn set_end_offset(&mut self, end_offset: u32) {
        self.end_offset = Some(end_offset);
    }

    /// Sets the statements for the function's body
    #[inline]
    pub fn set_statements(&mut self, statements: Vec<SierraStatement>) {
        self.statements = statements;
    }

    /// Sets the prototype of the function
    #[inline]
    pub fn set_prototype(&mut self, prototype: String) {
        self.prototype = Some(prototype);
    }
}
