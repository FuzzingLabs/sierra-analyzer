use std::collections::HashMap;

/// A struct that represents a symbolic execution solver
#[derive(Debug, Clone)]
pub struct SymbolicExecution {
    /// A list of variables assignations (variable -> value)
    pub variables_assignations: HashMap<String, String>,
    /// A list of constraints
    pub constraints: Vec<String>,
}

impl SymbolicExecution {
    /// Creates a new instance of `SymbolicExecution`
    pub fn new() -> Self {
        SymbolicExecution {
            variables_assignations: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    /// Adds a variable_assignation to the symbolic execution solver
    pub fn add_variable_assignation(&mut self, variable: String, value: String) {
        self.variables_assignations.insert(variable, value);
    }

    /// Adds a constraint to the symbolic execution solver
    pub fn add_constraint(&mut self, constraint: String) {
        self.constraints.push(constraint);
    }
}
