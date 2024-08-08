use std::collections::HashSet;
use std::str::FromStr;

use z3::{
    ast::{Ast, Bool, Int},
    Config, Context, SatResult, Solver,
};

use cairo_lang_sierra::program::GenStatement;

use crate::decompiler::function::{Function, SierraStatement};
use crate::decompiler::libfuncs_patterns::{
    ADDITION_REGEX, CONST_REGEXES, DUP_REGEX, IS_ZERO_REGEX, MULTIPLICATION_REGEX,
    SUBSTRACTION_REGEX,
};
use crate::{extract_parameters, parse_element_name_with_fallback};

/// Generates test cases for a single function
pub fn generate_test_cases_for_function(
    function: &mut Function,
    declared_libfuncs_names: Vec<String>,
) -> String {
    let mut result = String::new();
    let mut unique_results = HashSet::new();

    let felt252_arguments: Vec<(String, String)> = function
        .arguments
        .iter()
        .filter(|(_, arg_type)| arg_type == "felt252")
        .map(|(arg_name, arg_type)| (arg_name.clone(), arg_type.clone()))
        .collect();

    // Skip the function if there are no felt252 arguments
    if felt252_arguments.is_empty() {
        return result;
    }

    // Generate the function CFG
    function.create_cfg();

    let function_paths = function.cfg.as_ref().unwrap().paths();

    for path in &function_paths {
        // Create a new symbolic execution engine for the function
        let cfg = Config::new();
        let context = Context::new(&cfg);

        // Create a solver
        let solver = Solver::new(&context);

        // Create Z3 variables for each felt252 argument
        let z3_variables: Vec<Int> = felt252_arguments
            .iter()
            .map(|(arg_name, _)| Int::new_const(&context, &**arg_name))
            .collect();

        // Convert Sierra statements to z3 constraints
        for basic_block in path {
            for statement in &basic_block.statements {
                // Convert SierraStatement to a Z3 constraint and add to solver
                if let Some(constraint) = sierra_statement_to_constraint(
                    &statement,
                    &context,
                    declared_libfuncs_names.clone(),
                ) {
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
                    let values_str = format!("{:?}\n", values);
                    if unique_results.insert(values_str.clone()) {
                        result.push_str(&values_str);
                    }
                }
            }
            SatResult::Unsat | SatResult::Unknown => {
                let non_solvable_str = "non solvable\n".to_string();
                if unique_results.insert(non_solvable_str.clone()) {
                    result.push_str(&non_solvable_str);
                }
            }
        }
    }

    result.trim_end().to_string()
}

/// A struct that represents a symbolic execution solver
#[derive(Debug)]
pub struct SymbolicExecution<'a> {
    pub solver: Solver<'a>,
}

/// Converts a SierraStatement to a Z3 constraint, or returns None if not applicable
pub fn sierra_statement_to_constraint<'ctx>(
    statement: &SierraStatement,
    context: &'ctx Context,
    declared_libfuncs_names: Vec<String>,
) -> Option<Bool<'ctx>> {
    let inner_statement = &statement.statement;
    match inner_statement {
        GenStatement::Invocation(invocation) => {
            // Extract libfunc name, parameters & assigned variables
            let libfunc_id_str =
                parse_element_name_with_fallback!(invocation.libfunc_id, declared_libfuncs_names);
            let parameters = extract_parameters!(invocation.args);
            let assigned_variables = extract_parameters!(&invocation
                .branches
                .first()
                .map(|branch| &branch.results)
                .unwrap_or(&vec![]));

            // Handling variables duplications
            if DUP_REGEX.is_match(&libfunc_id_str) {
                let first_var_z3 = Int::new_const(context, assigned_variables[0].clone());
                let second_var_z3 = Int::new_const(context, assigned_variables[1].clone());
                return Some(second_var_z3._eq(&first_var_z3).into());
            }

            // Handling constant assignments
            for regex in CONST_REGEXES.iter() {
                if let Some(captures) = regex.captures(&libfunc_id_str) {
                    if let Some(const_value) = captures.name("const") {
                        // Convert string to a u64 to avoid overflow
                        let const_value_str = const_value.as_str();
                        if let Ok(const_value_u64) = u64::from_str(const_value_str) {
                            if !assigned_variables.is_empty() {
                                let assigned_var_z3 =
                                    Int::new_const(context, &*assigned_variables[0]);
                                let const_value_z3 = Int::from_u64(context, const_value_u64);
                                return Some(assigned_var_z3._eq(&const_value_z3).into());
                            }
                        }
                    }
                }
            }

            // Handle conditions
            if IS_ZERO_REGEX.is_match(&libfunc_id_str) {
                let operand = Int::new_const(context, parameters[0].clone());
                let constraint = operand._eq(&Int::from_i64(context, 0));
                return Some(constraint);
            }

            // Handling arithmetic operations
            let operator = if ADDITION_REGEX.is_match(&libfunc_id_str) {
                "+"
            } else if SUBSTRACTION_REGEX.is_match(&libfunc_id_str) {
                "-"
            } else if MULTIPLICATION_REGEX.is_match(&libfunc_id_str) {
                "*"
            } else {
                return None;
            };

            let assigned_variable = Int::new_const(context, assigned_variables[0].clone());
            let first_operand = Int::new_const(context, parameters[0].clone());
            let second_operand = Int::new_const(context, parameters[1].clone());

            let constraint = match operator {
                "+" => assigned_variable._eq(&(first_operand + second_operand)),
                "-" => assigned_variable._eq(&(first_operand - second_operand)),
                "*" => assigned_variable._eq(&(first_operand * second_operand)),
                _ => return None,
            };

            return Some(constraint);
        }
        _ => {}
    }
    None
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
