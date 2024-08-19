use std::collections::HashSet;
use std::str::FromStr;

use z3::{
    ast::{Ast, Bool, Int},
    Config, Context, SatResult, Solver,
};

use cairo_lang_sierra::program::GenStatement;
use cairo_lang_sierra::program::Invocation;

use crate::decompiler::cfg::BasicBlock;
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

    // Extract felt252 arguments from the function
    let felt252_arguments = extract_felt252_arguments(function);

    // Skip the function if there are no felt252 arguments
    if felt252_arguments.is_empty() {
        return result;
    }

    // Generate the function CFG (Control Flow Graph)
    function.create_cfg();
    let function_paths = function.cfg.as_ref().unwrap().paths();

    for path in &function_paths {
        result.push_str(&process_function_path(
            path,
            &felt252_arguments,
            &declared_libfuncs_names,
            &mut unique_results,
        ));
    }

    result.trim_end().to_string()
}

/// Processes a function path (A sequence of basic blocks)
fn process_function_path<'ctx>(
    path: &[&BasicBlock],
    felt252_arguments: &[(String, String)],
    declared_libfuncs_names: &[String],
    unique_results: &mut HashSet<String>,
) -> String {
    let cfg = Config::new();
    let context = Context::new(&cfg);
    let z3_variables = create_z3_variables(&context, felt252_arguments);
    let mut symbolic_execution = SymbolicExecution::new(&context);

    let mut zero_constraints = Vec::new();
    let mut other_constraints = Vec::new();
    let mut result = String::new();

    for basic_block in path {
        process_basic_block(
            basic_block,
            &context,
            declared_libfuncs_names,
            &mut symbolic_execution,
            &mut zero_constraints,
            &mut other_constraints,
        );

        // Generate test cases for `variable == 0` conditions
        result.push_str(&generate_cases(
            &symbolic_execution,
            felt252_arguments,
            &z3_variables,
            unique_results,
        ));

        // Generate test cases for `variable != 0` conditions
        result.push_str(&generate_non_zero_cases(
            &context,
            &zero_constraints,
            &other_constraints,
            felt252_arguments,
            &z3_variables,
            unique_results,
        ));
    }

    result
}

/// Processes a single basic block
fn process_basic_block<'ctx>(
    basic_block: &BasicBlock,
    context: &'ctx Context,
    declared_libfuncs_names: &[String],
    symbolic_execution: &mut SymbolicExecution<'ctx>,
    zero_constraints: &mut Vec<(Int<'ctx>, Bool<'ctx>)>,
    other_constraints: &mut Vec<Bool<'ctx>>,
) {
    for statement in &basic_block.statements {
        if let Some(constraint) =
            sierra_statement_to_constraint(&statement, context, declared_libfuncs_names.to_vec())
        {
            symbolic_execution.add_constraint(&constraint);

            // Classify constraints in 2 categories : zero constraints & other constraints
            classify_constraint(
                &statement,
                context,
                declared_libfuncs_names,
                zero_constraints,
                other_constraints,
            );
        }
    }
}

/// Extracts `felt252` arguments from the function
fn extract_felt252_arguments(function: &Function) -> Vec<(String, String)> {
    function
        .arguments
        .iter()
        .filter(|(_, arg_type)| arg_type == "felt252")
        .cloned()
        .collect()
}

/// Creates Z3 variables for the felt252 arguments
fn create_z3_variables<'ctx>(
    context: &'ctx Context,
    felt252_arguments: &[(String, String)],
) -> Vec<Int<'ctx>> {
    felt252_arguments
        .iter()
        .map(|(arg_name, _)| Int::new_const(context, &**arg_name))
        .collect()
}

/// Classifies constraints into zero constraints and other constraints
fn classify_constraint<'ctx>(
    statement: &SierraStatement,
    context: &'ctx Context,
    declared_libfuncs_names: &[String],
    zero_constraints: &mut Vec<(Int<'ctx>, Bool<'ctx>)>,
    other_constraints: &mut Vec<Bool<'ctx>>,
) {
    if let GenStatement::Invocation(invocation) = &statement.statement {
        let libfunc_id_str =
            parse_element_name_with_fallback!(invocation.libfunc_id, declared_libfuncs_names);

        if IS_ZERO_REGEX.is_match(&libfunc_id_str) {
            let operand_name = format!("v{}", invocation.args[0].id.to_string());
            let operand = Int::new_const(context, operand_name.clone());
            zero_constraints.push((operand, Bool::from_bool(context, true)));
        } else {
            other_constraints.push(Bool::from_bool(context, true));
        }
    }
}

/// Generates test cases based on the symbolic execution results
fn generate_cases(
    symbolic_execution: &SymbolicExecution,
    felt252_arguments: &[(String, String)],
    z3_variables: &[Int],
    unique_results: &mut HashSet<String>,
) -> String {
    let mut result = String::new();
    if symbolic_execution.check() == SatResult::Sat {
        if let Some(model) = symbolic_execution.solver.get_model() {
            let values = format_model(felt252_arguments, z3_variables, &model);
            if unique_results.insert(values.clone()) {
                result.push_str(&format!("{}\n", values));
            }
        }
    }
    result
}

/// Generates test cases for non-zero constraints
fn generate_non_zero_cases(
    context: &Context,
    zero_constraints: &[(Int, Bool)],
    other_constraints: &[Bool],
    felt252_arguments: &[(String, String)],
    z3_variables: &[Int],
    unique_results: &mut HashSet<String>,
) -> String {
    let mut result = String::new();
    for (operand, _) in zero_constraints {
        let non_zero_solver = Solver::new(context);

        for constraint in other_constraints {
            non_zero_solver.assert(constraint);
        }

        non_zero_solver.assert(&operand._eq(&Int::from_i64(context, 0)).not());

        if non_zero_solver.check() == SatResult::Sat {
            if let Some(model) = non_zero_solver.get_model() {
                let values = format_model(felt252_arguments, z3_variables, &model);
                if unique_results.insert(values.clone()) {
                    result.push_str(&format!("{}\n", values));
                }
            }
        }
    }
    result
}

/// Formats the model to a string representation
fn format_model(
    felt252_arguments: &[(String, String)],
    z3_variables: &[Int],
    model: &z3::Model,
) -> String {
    felt252_arguments
        .iter()
        .zip(z3_variables.iter())
        .map(|((arg_name, _), var)| {
            format!(
                "{}: {}",
                arg_name,
                model.eval(var, true).unwrap().to_string()
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Converts a Sierra statement to a Z3 constraint
pub fn sierra_statement_to_constraint<'ctx>(
    statement: &SierraStatement,
    context: &'ctx Context,
    declared_libfuncs_names: Vec<String>,
) -> Option<Bool<'ctx>> {
    match &statement.statement {
        GenStatement::Invocation(invocation) => {
            let libfunc_id_str = parse_libfunc_name(invocation, &declared_libfuncs_names);

            // Create an equality constraint when encoutering a variable duplication
            handle_duplication(context, &libfunc_id_str, invocation)
                .or_else(|| handle_constant_assignment(context, &libfunc_id_str, invocation))
                .or_else(|| handle_zero_check(context, &libfunc_id_str, invocation))
                .or_else(|| handle_arithmetic_operations(context, &libfunc_id_str, invocation))
        }
        _ => None,
    }
}

/// Extracts the libfunc name from the invocation
fn parse_libfunc_name(invocation: &Invocation, declared_libfuncs_names: &[String]) -> String {
    parse_element_name_with_fallback!(invocation.libfunc_id, declared_libfuncs_names)
}

/// Handles variable duplication cases by creating an equality constraint
fn handle_duplication<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    invocation: &Invocation,
) -> Option<Bool<'ctx>> {
    if DUP_REGEX.is_match(libfunc_id_str) {
        let assigned_variables = extract_parameters!(&invocation
            .branches
            .first()
            .map(|branch| &branch.results)
            .unwrap_or(&vec![]));

        let first_var_z3 = Int::new_const(context, assigned_variables[0].clone());
        let second_var_z3 = Int::new_const(context, assigned_variables[1].clone());
        return Some(second_var_z3._eq(&first_var_z3).into());
    }
    None
}

/// Handles constant assignment cases by creating a constraint that checks equality with a constant value
fn handle_constant_assignment<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    invocation: &Invocation,
) -> Option<Bool<'ctx>> {
    let assigned_variables = extract_parameters!(&invocation
        .branches
        .first()
        .map(|branch| &branch.results)
        .unwrap_or(&vec![]));

    CONST_REGEXES.iter().find_map(|regex| {
        regex.captures(libfunc_id_str).and_then(|captures| {
            captures.name("const").and_then(|const_value| {
                u64::from_str(const_value.as_str())
                    .ok()
                    .map(|const_value_u64| {
                        let assigned_var_z3 = Int::new_const(context, &*assigned_variables[0]);
                        let const_value_z3 = Int::from_u64(context, const_value_u64);
                        assigned_var_z3._eq(&const_value_z3).into()
                    })
            })
        })
    })
}

/// Handles zero-check cases by creating a constraint that checks equality with zero
fn handle_zero_check<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    invocation: &Invocation,
) -> Option<Bool<'ctx>> {
    if IS_ZERO_REGEX.is_match(libfunc_id_str) {
        let parameters = extract_parameters!(invocation.args);
        let operand = Int::new_const(context, parameters[0].clone());
        let constraint = operand._eq(&Int::from_i64(context, 0));
        return Some(constraint);
    }
    None
}

/// Handles arithmetic operations by creating constraints for the respective operation
fn handle_arithmetic_operations<'ctx>(
    context: &'ctx Context,
    libfunc_id_str: &str,
    invocation: &Invocation,
) -> Option<Bool<'ctx>> {
    let operator = if ADDITION_REGEX.is_match(libfunc_id_str) {
        "+"
    } else if SUBSTRACTION_REGEX.is_match(libfunc_id_str) {
        "-"
    } else if MULTIPLICATION_REGEX.is_match(libfunc_id_str) {
        "*"
    } else {
        return None;
    };

    let parameters = extract_parameters!(invocation.args);
    let assigned_variables = extract_parameters!(&invocation
        .branches
        .first()
        .map(|branch| &branch.results)
        .unwrap_or(&vec![]));

    let assigned_variable = Int::new_const(context, assigned_variables[0].clone());
    let first_operand = Int::new_const(context, parameters[0].clone());
    let second_operand = Int::new_const(context, parameters[1].clone());

    let constraint = match operator {
        "+" => assigned_variable._eq(&(first_operand + second_operand)),
        "-" => assigned_variable._eq(&(first_operand - second_operand)),
        "*" => assigned_variable._eq(&(first_operand * second_operand)),
        _ => return None,
    };

    Some(constraint)
}

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

    /// Adds a single constraint into the Z3 solver
    pub fn add_constraint(&mut self, constraint: &Bool<'a>) {
        self.solver.assert(constraint);
    }

    /// Loads multiple constraints into the Z3 solver
    pub fn load_constraints(&mut self, constraints: Vec<&Bool<'a>>) {
        for constraint in constraints {
            self.add_constraint(constraint);
        }
    }

    /// Checks if the current set of constraints is satisfiable
    pub fn check(&self) -> z3::SatResult {
        self.solver.check()
    }
}
