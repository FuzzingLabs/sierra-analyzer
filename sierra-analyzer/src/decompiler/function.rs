use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::Statement;
use cairo_lang_sierra::program::StatementIdx;

/// A struct representing a control flow graph (CFG) for a function
pub struct ControlFlowGraph {}

impl ControlFlowGraph {}

/// A struct representing a function in a Sierra program
pub struct Function<'a> {
    /// The function's `GenFunction` representation
    function: GenFunction<&'a StatementIdx>,
    /// A vector of `Statement`s representing the function's body
    statements: Vec<Statement>,
    /// A `ControlFlowGraph` representing the function's CFG
    cfg: ControlFlowGraph,
}

impl<'a> Function<'a> {
    /// Creates a new `Function` instance
    pub fn new(function: GenFunction<&'a StatementIdx>, cfg: ControlFlowGraph) -> Self {
        Self {
            function,
            statements: Vec::new(),
            cfg,
        }
    }

    /// Adds a `Statement` to the function's body
    pub fn push_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }
}
