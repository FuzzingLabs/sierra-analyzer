use z3::{ast::Bool, Context, Solver};

/// A struct that represents a symbolic execution solver
#[derive(Debug)]
pub struct SymbolicExecution<'a> {
    pub solver: Solver<'a>,
}

impl<'a> SymbolicExecution<'a> {
    /// Creates a new instance of `SymbolicExecution`
    pub fn new(context: &'a Context) -> Self {
        let solver = Solver::new(context);

        SymbolicExecution { solver }
    }

    /// Loads constraints into the Z3 solver
    pub fn load_constraints(&mut self, constraints: Vec<&Bool<'a>>) {
        for constraint in constraints {
            self.solver.assert(constraint);
        }
    }

    /// Adds a single constraint into the Z3 solver
    pub fn add_constraint(&mut self, constraint: &Bool<'a>) {
        self.solver.assert(constraint);
    }

    /// Checks if the current set of constraints is satisfiable
    pub fn check(&self) -> z3::SatResult {
        self.solver.check()
    }
}
