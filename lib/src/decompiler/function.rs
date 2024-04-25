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
    /// We use lazy_static for performances issues

    // Variable drop
    static ref DROP_REGEX: Regex = Regex::new(r"drop(<.*>)?").unwrap();
    // Store temporary variable
    static ref STORE_TEMP_REGEX: Regex = Regex::new(r"store_temp(<.*>)?").unwrap();
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
    pub fn formatted_statement(&self) -> Option<String> {
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
            // Function calls & variables assignments
            GenStatement::Invocation(invocation) => {
                let libfunc_id = parse_element_name!(invocation.libfunc_id);
                if !Self::is_function_allowed(&libfunc_id) {
                    return None; // Skip formatting if function is not allowed to simplify the decompiler output
                }
                let libfunc_id_str = libfunc_id.blue();

                // Function parameters
                let parameters = extract_parameters!(invocation.args);
                let parameters_str = parameters.join(", ");

                // Assigned variables
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

                // Format the string based on the presence of assigned variables
                if !assigned_variables.is_empty() {
                    Some(format!(
                        "{} = {}({})",
                        assigned_variables_str, libfunc_id_str, parameters_str
                    ))
                } else {
                    Some(format!("{}({})", libfunc_id_str, parameters_str))
                }
            }
        }
    }

    /// Checks if the given function name is allowed to be included in the formatted statement
    fn is_function_allowed(function_name: &str) -> bool {
        match function_name {
            "branch_align" | "disable_ap_tracking" => false,
            _ => {
                // Check blacklisted functions patterns
                if DROP_REGEX.is_match(function_name) || STORE_TEMP_REGEX.is_match(function_name) {
                    false
                } else {
                    true
                }
            }
        }
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
