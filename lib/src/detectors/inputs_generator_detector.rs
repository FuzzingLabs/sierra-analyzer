use z3::{ast::Bool, ast::Int, Config, Context, SatResult, Solver};

use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::function::SierraStatement;
use crate::detectors::detector::{Detector, DetectorType};

/// Converts a SierraStatement to a Z3 constraint, or returns None if not applicable
fn sierra_statement_to_constraint<'ctx>(_statement: &SierraStatement) -> Option<Bool<'ctx>> {
    // For now, always return None
    None
}

#[derive(Debug)]
pub struct InputsGeneratorDetector;

impl InputsGeneratorDetector {
    /// Creates a new InputsGeneratorDetector instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for InputsGeneratorDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "inputs"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Inputs generator"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Generate inputs for a sierra function"
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Returns all the functions names
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();

        for function in &mut decompiler.functions {
            let felt252_arguments: Vec<(String, String)> = function
                .arguments
                .iter()
                .filter(|(_, arg_type)| arg_type == "felt252")
                .map(|(arg_name, arg_type)| (arg_name.clone(), arg_type.clone()))
                .collect();

            // Skip the function if there are no felt252 arguments
            if felt252_arguments.is_empty() {
                continue;
            }

            // Generate the function CFG
            function.create_cfg();

            let function_paths = function.cfg.as_ref().unwrap().paths();
            for path in &function_paths {
                // Create a new symbolic execution engine for the function
                let cfg = Config::new();
                let context = Context::new(&cfg);

                // Create Z3 variables for each felt252 argument
                let z3_variables: Vec<Int> = felt252_arguments
                    .iter()
                    .map(|(arg_name, _)| Int::new_const(&context, &**arg_name))
                    .collect();

                // Create a solver
                let solver = Solver::new(&context);

                // Convert Sierra statements to z3 constraints
                for basic_block in path {
                    for statement in &basic_block.statements {
                        // Convert SierraStatement to a Z3 constraint and add to solver
                        if let Some(constraint) = sierra_statement_to_constraint(&statement) {
                            solver.assert(&constraint);
                        }
                    }
                }

                // Check if the constraints are satisfiable
                match solver.check() {
                    SatResult::Sat => {
                        // If solvable, add the argument names and values to the result
                        if let Some(model) = solver.get_model() {
                            let values: Vec<String> = felt252_arguments
                                .iter()
                                .zip(z3_variables.iter())
                                .map(|((arg_name, _), var)| {
                                    format!(
                                        "{}: {}",
                                        arg_name,
                                        model.eval(var, true).unwrap().to_string()
                                    )
                                })
                                .collect();
                            result.push_str(&format!("{:?}\n", values));
                        }
                    }
                    SatResult::Unsat | SatResult::Unknown => {
                        // If not solvable, add "non solvable" to the result
                        result.push_str("non solvable\n");
                    }
                }
            }
        }

        result
    }
}
