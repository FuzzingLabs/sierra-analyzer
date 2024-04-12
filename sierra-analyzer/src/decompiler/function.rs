use cairo_lang_sierra::program::GenFunction;
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

/// A struct representing a statement in a Sierra program with an offset
#[allow(dead_code)]
#[derive(Debug)]
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
#[derive(Debug)]
pub struct ControlFlowGraph {}

impl ControlFlowGraph {
    /// Creates a new `ControlFlowGraph` instance
    pub fn new() -> Self {
        Self {}
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
    cfg: ControlFlowGraph,
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
            cfg: ControlFlowGraph::new(),
            prototype: None,
        }
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
