use colored::*;

use cairo_lang_sierra::program::BranchTarget;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program::StatementIdx;

use crate::decompiler::cfg::ControlFlowGraph;
use crate::decompiler::cfg::SierraConditionalBranch;
use crate::extract_parameters;
use crate::parse_libfunc_name;

/// A struct representing a statement in a Sierra program with an offset
#[derive(Debug, Clone)]
pub struct SierraStatement {
    pub statement: cairo_lang_sierra::program::Statement,
    pub offset: u32,
    pub is_conditional_branch: bool,
}

impl SierraStatement {
    /// Creates a new `SierraStatement` instance with an offset
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
    pub fn formatted_statement(&self) -> String {
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
                formatted
            }
            // Function calls & variables assignments
            GenStatement::Invocation(invocation) => {
                // Function name in blue
                let libfunc_id_str = parse_libfunc_name!(invocation.libfunc_id).blue();

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
                    format!(
                        "{} = {}({})",
                        assigned_variables_str, libfunc_id_str, parameters_str
                    )
                } else {
                    format!("{}({})", libfunc_id_str, parameters_str)
                }
            }
        }
    }

    /// Returns a reference to this statement as a conditional branch if it is one
    pub fn as_conditional_branch(&self) -> Option<SierraConditionalBranch> {
        if self.is_conditional_branch {
            if let GenStatement::Invocation(invocation) = &self.statement {
                // Statement
                let statement = self.statement.clone();

                // Function name
                let libfunc_id_str = parse_libfunc_name!(invocation.libfunc_id);

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
#[allow(dead_code)]
#[derive(Debug, Clone)]
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
        let mut cfg = ControlFlowGraph::new(self.statements.clone(), self.start_offset.unwrap());

        // Generate the CFG basic blocks
        cfg.generate_basic_blocks();

        // Assign the control flow graph to the function's CFG field
        self.cfg = Some(cfg);
    }

    /// Sets the start offset of the function
    pub fn set_start_offset(&mut self, start_offset: u32) {
        self.start_offset = Some(start_offset);
    }

    /// Sets the end offset of the function
    pub fn set_end_offset(&mut self, end_offset: u32) {
        self.end_offset = Some(end_offset);
    }

    /// Sets the statements for the function's body
    pub fn set_statements(&mut self, statements: Vec<SierraStatement>) {
        self.statements = statements;
    }

    /// Sets the prototype of the function
    pub fn set_prototype(&mut self, prototype: String) {
        self.prototype = Some(prototype);
    }
}