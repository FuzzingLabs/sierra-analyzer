use cairo_lang_sierra::program::BranchTarget;
use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program::StatementIdx;

/// Enum representing different types of CFG edges
#[allow(dead_code)]
#[derive(Debug)]
pub enum EdgeType {
    Unconditional,
    ConditionalTrue,
    ConditionalFalse,
    Fallthrough,
}

/// Struct representing a control flow graph (CFG) edge
#[allow(dead_code)]
#[derive(Debug)]
pub struct Edge {
    source: usize,
    destination: usize,
    edge_type: EdgeType,
}

impl Edge {
    /// Creates a new `Edge` instance
    #[allow(dead_code)]
    pub fn new(source: usize, destination: usize, edge_type: EdgeType) -> Self {
        Self {
            source,
            destination,
            edge_type,
        }
    }
}

/// Struct representing a Sierra Control-Flow Graph basic block
#[allow(dead_code)]
#[derive(Debug)]
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
pub struct SierraConditionalBranch<'a> {
    // Inherit SierraStatement's fields
    statement: SierraStatement,
    // Function reference
    function: &'a Function<'a>,
    // TODO: Create a Variable object
    parameters: Vec<String>,
    // Edges offsets
    edge_1_offset: u32,
    edge_2_offset: Option<u32>,
    // Fallthrough conditional branch
    fallthrough: bool,
}

impl<'a> SierraConditionalBranch<'a> {
    /// Creates a new `SierraConditionalBranch` instance
    #[allow(dead_code)]
    pub fn new(
        statement: SierraStatement,
        function: &'a Function<'a>,
        // TODO: Create a Variable object
        parameters: Vec<String>,
        edge_1_offset: u32,
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
}

impl SierraStatement {
    /// Creates a new `SierraStatement` instance with an offset
    pub fn new(statement: cairo_lang_sierra::program::Statement, offset: u32) -> Self {
        Self { statement, offset }
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
        let cfg = ControlFlowGraph::new(self.statements.clone(), self.start_offset.unwrap());
        cfg.get_basic_blocks_delimitations();
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
