use cairo_lang_sierra::program::BranchTarget;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program::StatementIdx;

/// Enum representing different types of CFG edges
#[derive(Debug, Clone)]
pub enum EdgeType {
    Unconditional,
    ConditionalTrue,
    ConditionalFalse,
    Fallthrough,
}

/// Struct representing a control flow graph (CFG) edge
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Edge {
    source: u32,
    destination: u32,
    edge_type: EdgeType,
}

impl Edge {
    /// Creates a new `Edge` instance
    #[allow(dead_code)]
    pub fn new(source: u32, destination: u32, edge_type: EdgeType) -> Self {
        Self {
            source,
            destination,
            edge_type,
        }
    }
}

/// Struct representing a Sierra Control-Flow Graph basic block
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BasicBlock {
    /// Basic block delimitations
    start_statement: SierraStatement,
    start_offset: u32,
    end_offset: Option<u32>,
    /// Name of the basic block
    name: String,
    /// Instructions (statements) in the basic block
    statements: Vec<SierraStatement>,
    /// Edges of the basic block
    edges: Vec<Edge>,
}

#[allow(dead_code)]
impl BasicBlock {
    /// Creates a new `BasicBlock` instance
    pub fn new(start_statement: SierraStatement) -> Self {
        let start_offset = start_statement.offset;
        let name = format!("bb_{}", start_offset);
        BasicBlock {
            start_statement,
            start_offset,
            end_offset: None,
            name,
            statements: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Returns the name of the basic block
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a statement to the basic block
    pub fn add_statement(&mut self, statement: SierraStatement) {
        self.statements.push(statement);
    }

    /// Adds an edge to the basic block
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Sets the end offset of the basic block
    pub fn set_end_offset(&mut self, end_offset: u32) {
        self.end_offset = Some(end_offset);
    }
}

/// Struct representing a Sierra conditional branch
#[allow(dead_code)]
#[derive(Debug)]
pub struct SierraConditionalBranch {
    // Inherit SierraStatement's fields
    statement: SierraStatement,
    // Function name
    function: String,
    // TODO: Create a Variable object
    parameters: Vec<String>,
    // Edges offsets
    edge_1_offset: Option<u32>,
    edge_2_offset: Option<u32>,
    // Fallthrough conditional branch
    fallthrough: bool,
}

impl SierraConditionalBranch {
    /// Creates a new `SierraConditionalBranch` instance
    #[allow(dead_code)]
    pub fn new(
        statement: SierraStatement,
        function: String,
        // TODO: Create a Variable object
        parameters: Vec<String>,
        edge_1_offset: Option<u32>,
        edge_2_offset: Option<u32>,
        fallthrough: bool,
    ) -> Self {
        let mut edge_2_offset = edge_2_offset;
        if fallthrough && edge_2_offset.is_none() {
            edge_2_offset = Some(statement.offset);
        }

        SierraConditionalBranch {
            statement,
            function,
            parameters,
            edge_1_offset,
            edge_2_offset,
            fallthrough,
        }
    }
}

/// A struct representing a statement in a Sierra program with an offset
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SierraStatement {
    statement: cairo_lang_sierra::program::Statement,
    offset: u32,
    is_conditional_branch: bool,
}

impl SierraStatement {
    /// Creates a new `SierraStatement` instance with an offset
    pub fn new(statement: cairo_lang_sierra::program::Statement, offset: u32) -> Self {
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

    /// Returns a reference to this statement as a conditional branch if it is one
    fn as_conditional_branch(&self) -> Option<SierraConditionalBranch> {
        if self.is_conditional_branch {
            if let GenStatement::Invocation(invocation) = &self.statement {
                // Statement
                let statement = self.statement.clone();

                // Function name
                let libfunc_id_str = if let Some(debug_name) = &invocation.libfunc_id.debug_name {
                    debug_name.to_string()
                } else {
                    invocation.libfunc_id.id.to_string()
                };

                // Parameters
                let parameters: Vec<String> = invocation
                    .args
                    .iter()
                    .map(|var_id| {
                        if let Some(debug_name) = &var_id.debug_name {
                            debug_name.clone().into()
                        } else {
                            format!("v{}", var_id.id)
                        }
                    })
                    .collect();

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

/// A struct representing a control flow graph (CFG) for a function
#[allow(dead_code)]
#[derive(Debug)]
pub struct ControlFlowGraph {
    statements: Vec<SierraStatement>,
    start_offset: u32,
    basic_blocks: Vec<BasicBlock>,
}

impl<'a> ControlFlowGraph {
    /// Creates a new `ControlFlowGraph` instance
    pub fn new(statements: Vec<SierraStatement>, start_offset: u32) -> Self {
        Self {
            statements,
            start_offset,
            basic_blocks: Vec::new(),
        }
    }

    /// Gets the start and end offsets of basic blocks within the function's control flow graph
    pub fn get_basic_blocks_delimitations(&self) -> (Vec<u32>, Vec<u32>) {
        // Initialize vectors to store the start and end offsets of basic blocks
        let mut basic_blocks_starts = vec![self.start_offset];
        let mut basic_blocks_ends = vec![];

        // Iterate over each statement in the function
        for statement in &self.statements {
            // Match the type of statement
            match &statement.statement {
                // If it's a return statement, add its offset to the list of basic block ends
                GenStatement::Return(_) => {
                    basic_blocks_ends.push(statement.offset);
                }
                // If it's an invocation statement
                GenStatement::Invocation(invocation) => {
                    // Iterate over each branch target of the invocation
                    for target in &invocation.branches {
                        // Match the branch target type
                        match &target.target {
                            // If it's a statement target
                            BranchTarget::Statement(statement_idx) => {
                                // Add the offset of the statement after the invocation as the start of a new basic block
                                basic_blocks_starts.push(statement.offset + 1);
                                // Add the offset of the targeted statement as the start of a new basic block
                                basic_blocks_starts.push(statement_idx.0.try_into().unwrap());
                            }
                            // Ignore other types of branch targets
                            _ => {}
                        }
                    }
                }
            }
        }

        // Return the vectors containing the start and end offsets of basic blocks
        (basic_blocks_starts, basic_blocks_ends)
    }

    /// Generates the CFG basic blocks
    pub fn generate_basic_blocks(&mut self) {
        // Retrieve basic blocks delimitations
        let (basic_blocks_starts, basic_blocks_ends) = self.get_basic_blocks_delimitations();

        // Initialize variables for tracking the current basic block
        let mut new_basic_block = true;
        let mut current_basic_block = BasicBlock::new(self.statements[0].clone());

        // Iterate through each statement
        for i in 0..self.statements.len() {
            let statement = &self.statements[i];

            // Check if the current statement marks the beginning of a new basic block
            if basic_blocks_starts.contains(&statement.offset) {
                // If it's the beginning of a new basic block, push the previous one to the list
                if !new_basic_block {
                    self.basic_blocks.push(current_basic_block.clone());
                }
                // Create a new basic block
                current_basic_block = BasicBlock::new(statement.clone());
                new_basic_block = false;
            }

            // Add the current statement to the current basic block
            current_basic_block.statements.push(statement.clone());

            // Check if the current statement marks the end of the current basic block
            if basic_blocks_ends.contains(&statement.offset) {
                new_basic_block = true;
            }

            // Handle conditional branches
            if let Some(conditional_branch) = statement.as_conditional_branch() {
                if let Some(edge_2_offset) = conditional_branch.edge_2_offset {
                    // Conditional branch with 2 edges (JNZ)
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: conditional_branch.edge_1_offset.unwrap(),
                        edge_type: EdgeType::ConditionalTrue,
                    });
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: edge_2_offset + 1,
                        edge_type: EdgeType::ConditionalFalse,
                    });
                } else if let Some(edge_1_offset) = conditional_branch.edge_1_offset {
                    // Conditional jump with 1 edge (JUMP)
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: edge_1_offset,
                        edge_type: EdgeType::Unconditional,
                    });
                }
            }
            // Check for fallthrough edges
            else if i < (self.statements.len() - 1) {
                if basic_blocks_starts.contains(&(self.statements[i + 1].offset))
                    && !matches!(statement.statement, GenStatement::Return(_))
                {
                    // Fallthrough edge
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: statement.offset + 1,
                        edge_type: EdgeType::Fallthrough,
                    });
                }
            }
        }

        // Push the last basic block to the list
        self.basic_blocks.push(current_basic_block);
    }
}

/// A struct representing a function in a Sierra program
#[allow(dead_code)]
#[derive(Debug)]
pub struct Function<'a> {
    /// The function's `GenFunction` representation
    function: &'a GenFunction<StatementIdx>,
    // Function start offset
    pub start_offset: Option<u32>,
    // Function end offset
    pub end_offset: Option<u32>,
    /// A vector of `SierraStatement` instances representing the function's body with offsets
    statements: Vec<SierraStatement>,
    /// A `ControlFlowGraph` representing the function's CFG
    cfg: Option<ControlFlowGraph>,
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

    /// Returns a reference to the statements in the function's body
    #[allow(dead_code)]
    pub fn statements(&self) -> &Vec<SierraStatement> {
        &self.statements
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

    /// Returns the statements in the function's body as a string
    pub fn statements_as_string(&self) -> String {
        self.statements
            .iter()
            .map(|stmt| stmt.statement.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Sets the prototype of the function
    pub fn set_prototype(&mut self, prototype: String) {
        self.prototype = Some(prototype);
    }
}
