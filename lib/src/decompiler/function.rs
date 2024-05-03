use colored::*;
use lazy_static::lazy_static;
use regex::Regex;

use cairo_lang_sierra::program::BranchTarget;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program::StatementIdx;

use crate::decompiler::cfg::ControlFlowGraph;
use crate::decompiler::cfg::SierraConditionalBranch;
use crate::extract_parameters;
use crate::parse_element_name;

lazy_static! {
    /// Those libfuncs id patterns are blacklisted from the regular decompiler output (not the verbose)
    /// to make it more readable
    ///
    /// We use lazy_static for performances issues

    // Variable drop
    static ref DROP_REGEX: Regex = Regex::new(r"drop(<.*>)?").unwrap();

    // Store temporary variable
    static ref STORE_TEMP_REGEX: Regex = Regex::new(r"store_temp(<.*>)?").unwrap();

    /// These are libfuncs id patterns whose representation in the decompiler output can be improved

    // User defined function call
    static ref FUNCTION_CALL_REGEX: Regex = Regex::new(r"function_call<(.*)>").unwrap();

    // Arithmetic operations
    static ref ADDITION_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_add").unwrap();
    static ref SUBSTRACTION_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_sub").unwrap();
    static ref MULTIPLICATION_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_mul").unwrap();

    // Variable duplication
    static ref DUP_REGEX: Regex = Regex::new(r"dup(<.*>)?").unwrap();

    // Consts declarations
    static ref CONST_REGEXES: Vec<Regex> = vec![
        Regex::new(r"const_as_immediate<Const<.+, (?P<const>[0-9]+)>>").unwrap(),
        Regex::new(r"storage_base_address_const<(?P<const>[0-9]+)>").unwrap(),
        Regex::new(r"(felt|u)_?(8|16|32|64|128|252)_const<(?P<const>[0-9]+)>").unwrap(),
    ];
}

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
    pub fn formatted_statement(&self, verbose: bool) -> Option<String> {
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
                let libfunc_id = parse_element_name!(invocation.libfunc_id);
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
    ) -> String {
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

        // Handling variables duplications
        // In the Sierra IR it it represented like : v1, v2 = dup<felt252>(v1)
        // But we can represent it as a variable assignment such as : v2 = v1
        if DUP_REGEX.is_match(libfunc_id_str) {
            if let Some((first_var, second_var)) = assigned_variables_str.split_once(", ") {
                return format!("{} = {}", second_var, first_var);
            }
        }

        // Handling const declarations
        for regex in CONST_REGEXES.iter() {
            if let Some(captures) = regex.captures(libfunc_id_str) {
                if let Some(const_value) = captures.name("const") {
                    let const_value_str = const_value.as_str();
                    return format!("{} = {}", assigned_variables_str, const_value_str);
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
    pub fn as_conditional_branch(&self) -> Option<SierraConditionalBranch> {
        if self.is_conditional_branch {
            if let GenStatement::Invocation(invocation) = &self.statement {
                // Statement
                let statement = self.statement.clone();

                // Function name
                let libfunc_id_str = parse_element_name!(invocation.libfunc_id);

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
    function: &'a GenFunction<StatementIdx>,
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
