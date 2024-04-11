use cairo_lang_sierra::program::GenFunction;
use cairo_lang_sierra::program::Statement;
use cairo_lang_sierra::program::StatementIdx;

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
    /// A vector of `Statement`s representing the function's body
    statements: Vec<Statement>,
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
    pub fn statements(&self) -> &Vec<Statement> {
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
    pub fn set_statements(&mut self, statements: Vec<Statement>) {
        self.statements = statements;
    }

    /// Returns the statements in the function's body as a string
    pub fn statements_as_string(&self) -> String {
        let mut statement_strings = Vec::new();

        for statement in &self.statements {
            let statement_string = statement.to_string();
            statement_strings.push(statement_string);
        }

        statement_strings.join("\n")
    }

    /// Sets the prototype of the function
    pub fn set_prototype(&mut self, prototype: String) {
        self.prototype = Some(prototype);
    }
}
