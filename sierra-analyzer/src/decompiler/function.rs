use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::Statement;
use cairo_lang_sierra::program::StatementIdx;

/// A struct representing a control flow graph (CFG) for a function
pub struct ControlFlowGraph {}

impl ControlFlowGraph {
    /// Creates a new `ControlFlowGraph` instance
    pub fn new() -> Self {
        Self {}
    }
}

/// A struct representing a function in a Sierra program
#[allow(dead_code)]
pub struct Function<'a> {
    /// The function's `GenFunction` representation
    function: &'a GenFunction<StatementIdx>,
    /// A vector of `Statement`s representing the function's body
    statements: Vec<Statement>,
    /// A `ControlFlowGraph` representing the function's CFG
    cfg: ControlFlowGraph,
}

impl<'a> Function<'a> {
    /// Creates a new `Function` instance
    pub fn new(function: &'a GenFunction<StatementIdx>) -> Self {
        Self {
            function,
            statements: Vec::new(),
            cfg: ControlFlowGraph::new(),
        }
    }

    /// Adds a `Statement` to the function's body
    #[allow(dead_code)]
    pub fn push_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}
