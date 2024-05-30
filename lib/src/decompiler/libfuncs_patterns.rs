use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Those libfuncs id patterns are blacklisted from the regular decompiler output (not the verbose)
    /// to make it more readable
    ///
    /// We use lazy_static for performances issues

    // Variable drop
    pub static ref DROP_REGEX: Regex = Regex::new(r"drop(<.*>)?").unwrap();

    // Store temporary variable
    pub static ref STORE_TEMP_REGEX: Regex = Regex::new(r"store_temp(<.*>)?").unwrap();

    /// These are libfuncs id patterns whose representation in the decompiler output can be improved

    // User defined function call
    pub static ref FUNCTION_CALL_REGEX: Regex = Regex::new(r"function_call<(.*)>").unwrap();

    // Arithmetic operations
    pub static ref ADDITION_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_add").unwrap();
    pub static ref SUBSTRACTION_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_sub").unwrap();
    pub static ref MULTIPLICATION_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)(_overflowing)?_mul").unwrap();

    // Variable duplication
    pub static ref DUP_REGEX: Regex = Regex::new(r"dup(<.*>)?").unwrap();

    // Variable renaming
    pub static ref VARIABLE_ASSIGNMENT_REGEX: Vec<Regex> = vec![
        Regex::new(r"rename<.+>").unwrap(),
        Regex::new(r"store_temp<.+>").unwrap()
    ];

    // Check if an integer is 0
    pub static ref IS_ZERO_REGEX: Regex = Regex::new(r"(felt|u)_?(8|16|32|64|128|252)_is_zero").unwrap();

    // Consts declarations
    pub static ref CONST_REGEXES: Vec<Regex> = vec![
        Regex::new(r"const_as_immediate<Const<.+, (?P<const>-?[0-9]+)>>").unwrap(),
        Regex::new(r"storage_base_address_const<(?P<const>-?[0-9]+)>").unwrap(),
        Regex::new(r"(felt|u)_?(8|16|32|64|128|252)_const<(?P<const>-?[0-9]+)>").unwrap(),
    ];

    // User defined function
    pub static ref USER_DEFINED_FUNCTION_REGEX: Regex = Regex::new(r"(function_call|(\[[0-9]+\]))(::)?<user@(?P<function_id>.+)>").unwrap();

    // Array declarations & mutations
    pub static ref NEW_ARRAY_REGEX: Regex = Regex::new(r"array_new<(?P<array_type>.+)>").unwrap();
    pub static ref ARRAY_APPEND_REGEX: Regex = Regex::new(r"array_append<(.+)>").unwrap();

    // Regex of a type ID
    // Used to match and replace them in remote contracts
    pub static ref TYPE_ID_REGEX: Regex = Regex::new(r"(?<type_id>\[[0-9]+\])").unwrap();
}
