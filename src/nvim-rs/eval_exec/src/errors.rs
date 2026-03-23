//! Error handling for VimL evaluation.
//!
//! This module provides structured error types for VimL evaluation errors,
//! integrating with Neovim's message system. Migrated from error handling
//! in `src/nvim/eval.c`.
//!
//! ## Error Categories
//!
//! VimL errors fall into several categories:
//! - Type errors: Wrong type for operation (E701, E728, etc.)
//! - Index errors: Out-of-bounds access (E684, E685, etc.)
//! - Syntax errors: Invalid expression syntax
//! - Runtime errors: Division by zero, etc.
//! - Function errors: Wrong argument count, invalid args (E117-E119)
//!
//! ## Integration with Neovim
//!
//! Errors are reported through the C `emsg()` and `semsg()` functions.
//! The `:silent` command suppresses error messages.

#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int};

// =============================================================================
// Error Codes
// =============================================================================

/// VimL evaluation error codes.
///
/// These match the E-number error codes used in Neovim.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvalErrorCode {
    // Type errors
    /// E701: Invalid type for len()
    E701InvalidTypeForLen = 701,
    /// E702: Sort compare function returned an invalid value
    E702SortCompareInvalid = 702,
    /// E703: Using a Funcref as a Number
    E703FuncrefAsNumber = 703,
    /// E704: Funcref variable name must start with a capital
    E704FuncrefCapital = 704,
    /// E705: Variable name conflicts with existing function
    E705VarFuncConflict = 705,
    /// E706: Variable type mismatch for: %s
    E706TypeMismatch = 706,
    /// E707: Function name conflicts with existing variable
    E707FuncVarConflict = 707,
    /// E708: [:] must come last
    E708SliceMustComeLast = 708,
    /// E709: [:] requires a List or Blob value
    E709SliceRequiresListBlob = 709,
    /// E710: List value has more items than target
    E710ListTooMany = 710,
    /// E711: List value has not enough items
    E711ListTooFew = 711,
    /// E712: Argument of %s must be a List or Dictionary
    E712ArgMustBeListDict = 712,
    /// E713: Cannot use empty key for Dictionary
    E713EmptyDictKey = 713,
    /// E714: List required
    E714ListRequired = 714,
    /// E715: Dictionary required
    E715DictRequired = 715,
    /// E716: Key not present in Dictionary: %s
    E716KeyNotPresent = 716,
    /// E717: Dictionary entry already exists
    E717DictEntryExists = 717,
    /// E718: Funcref required
    E718FuncrefRequired = 718,
    /// E719: Cannot use [:] with a Dictionary
    E719SliceOnDict = 719,
    /// E720: Missing colon in Dictionary
    E720MissingColon = 720,
    /// E721: Duplicate key in Dictionary
    E721DuplicateKey = 721,
    /// E722: Trailing comma not allowed
    E722TrailingComma = 722,
    /// E723: Missing end of Dictionary
    E723MissingEndDict = 723,
    /// E724: Variable nested too deep for displaying
    E724NestedTooDeep = 724,
    /// E725: Calling dict function without Dictionary
    E725DictFuncNoDict = 725,
    /// E726: Stride is zero
    E726StrideZero = 726,
    /// E727: Start past end
    E727StartPastEnd = 727,
    /// E728: Using a Dictionary as a Number
    E728DictAsNumber = 728,
    /// E729: Using a Funcref as a String
    E729FuncrefAsString = 729,
    /// E730: Using a List as a String
    E730ListAsString = 730,
    /// E731: Using a Dictionary as a String
    E731DictAsString = 731,
    /// E734: Wrong variable type for %s
    E734WrongVarType = 734,
    /// E735: Can only compare Dictionary with Dictionary
    E735CompareDictDict = 735,
    /// E736: Can only compare List with List
    E736CompareListList = 736,
    /// E737: Key already exists: %s
    E737KeyExists = 737,
    /// E738: Can only compare Funcref with Funcref
    E738CompareFuncFunc = 738,

    // Index errors
    /// E684: List index out of range: %d
    E684ListIndexOOR = 684,
    /// E685: Internal error: %s
    E685Internal = 685,
    /// E686: Argument of %s must be a List
    E686ArgMustBeList = 686,
    /// E687: Less targets than List items
    E687FewerTargets = 687,
    /// E688: More targets than List items
    E688MoreTargets = 688,
    /// E689: Can only index a List, Dictionary or Blob
    E689CannotIndex = 689,
    /// E690: Missing "in" after :for
    E690MissingIn = 690,

    // Function errors
    /// E117: Unknown function: %s
    E117UnknownFunc = 117,
    /// E118: Too many arguments for function: %s
    E118TooManyArgs = 118,
    /// E119: Not enough arguments for function: %s
    E119NotEnoughArgs = 119,
    /// E120: Using <SID> not in a script context: %s
    E120SidNotInScript = 120,
    /// E121: Undefined variable: %s
    E121UndefinedVar = 121,
    /// E122: Function %s already exists
    E122FuncExists = 122,
    /// E123: Undefined function: %s
    E123UndefinedFunc = 123,
    /// E124: Missing '('
    E124MissingParen = 124,
    /// E125: Illegal argument: %s
    E125IllegalArg = 125,
    /// E126: Missing :endfunction
    E126MissingEndfunction = 126,
    /// E127: Cannot redefine function %s
    E127CannotRedefine = 127,
    /// E128: Function name must start with a capital
    E128FuncCapital = 128,
    /// E129: Function name required
    E129FuncNameReq = 129,
    /// E130: Unknown function: %s
    E130UnknownFunc = 130,
    /// E131: Cannot delete function %s
    E131CannotDelete = 131,
    /// E132: Function call depth is higher than 'maxfuncdepth'
    E132FuncDepth = 132,
    /// E133: :return not inside a function
    E133ReturnNotInFunc = 133,

    // Syntax/expression errors
    /// E15: Invalid expression: %s
    E15InvalidExpr = 15,
    /// E109: Missing ':' after '?'
    E109MissingColon = 109,
    /// E110: Missing ')'
    E110MissingParen = 110,
    /// E111: Missing ']'
    E111MissingBracket = 111,
    /// E112: Option name missing: %s
    E112OptionMissing = 112,
    /// E113: Unknown option: %s
    E113UnknownOption = 113,
    /// E114: Missing quote: %s
    E114MissingQuote = 114,
    /// E115: Missing quote: %s
    E115MissingQuote2 = 115,
    /// E116: Invalid arguments for function: %s
    E116InvalidFuncArgs = 116,

    // Runtime errors
    /// E806: Using a Float as a String
    E806FloatAsString = 806,
    /// E808: Number or Float required
    E808NumberFloatReq = 808,

    // Blob errors
    /// E974: Using a Blob as a Number
    E974BlobAsNumber = 974,
    /// E975: Using a Blob as a String
    E975BlobAsString = 975,
    /// E976: Using a Blob as a Float
    E976BlobAsFloat = 976,
    /// E977: Can only compare Blob with Blob
    E977CompareBlobBlob = 977,

    // Lock errors
    /// E741: Value is locked
    E741ValueLocked = 741,
    /// E742: Cannot change value
    E742CannotChange = 742,

    // Other
    /// Unknown error
    Unknown = 0,
}

impl EvalErrorCode {
    /// Get error number.
    pub const fn number(self) -> i32 {
        self as i32
    }

    /// Create from error number.
    pub const fn from_number(n: i32) -> Self {
        match n {
            701 => Self::E701InvalidTypeForLen,
            702 => Self::E702SortCompareInvalid,
            703 => Self::E703FuncrefAsNumber,
            704 => Self::E704FuncrefCapital,
            705 => Self::E705VarFuncConflict,
            706 => Self::E706TypeMismatch,
            707 => Self::E707FuncVarConflict,
            708 => Self::E708SliceMustComeLast,
            709 => Self::E709SliceRequiresListBlob,
            710 => Self::E710ListTooMany,
            711 => Self::E711ListTooFew,
            712 => Self::E712ArgMustBeListDict,
            713 => Self::E713EmptyDictKey,
            714 => Self::E714ListRequired,
            715 => Self::E715DictRequired,
            716 => Self::E716KeyNotPresent,
            717 => Self::E717DictEntryExists,
            718 => Self::E718FuncrefRequired,
            719 => Self::E719SliceOnDict,
            720 => Self::E720MissingColon,
            721 => Self::E721DuplicateKey,
            722 => Self::E722TrailingComma,
            723 => Self::E723MissingEndDict,
            724 => Self::E724NestedTooDeep,
            725 => Self::E725DictFuncNoDict,
            726 => Self::E726StrideZero,
            727 => Self::E727StartPastEnd,
            728 => Self::E728DictAsNumber,
            729 => Self::E729FuncrefAsString,
            730 => Self::E730ListAsString,
            731 => Self::E731DictAsString,
            734 => Self::E734WrongVarType,
            735 => Self::E735CompareDictDict,
            736 => Self::E736CompareListList,
            737 => Self::E737KeyExists,
            738 => Self::E738CompareFuncFunc,
            684 => Self::E684ListIndexOOR,
            685 => Self::E685Internal,
            686 => Self::E686ArgMustBeList,
            687 => Self::E687FewerTargets,
            688 => Self::E688MoreTargets,
            689 => Self::E689CannotIndex,
            690 => Self::E690MissingIn,
            117 => Self::E117UnknownFunc,
            118 => Self::E118TooManyArgs,
            119 => Self::E119NotEnoughArgs,
            120 => Self::E120SidNotInScript,
            121 => Self::E121UndefinedVar,
            122 => Self::E122FuncExists,
            123 => Self::E123UndefinedFunc,
            124 => Self::E124MissingParen,
            125 => Self::E125IllegalArg,
            126 => Self::E126MissingEndfunction,
            127 => Self::E127CannotRedefine,
            128 => Self::E128FuncCapital,
            129 => Self::E129FuncNameReq,
            130 => Self::E130UnknownFunc,
            131 => Self::E131CannotDelete,
            132 => Self::E132FuncDepth,
            133 => Self::E133ReturnNotInFunc,
            15 => Self::E15InvalidExpr,
            109 => Self::E109MissingColon,
            110 => Self::E110MissingParen,
            111 => Self::E111MissingBracket,
            112 => Self::E112OptionMissing,
            113 => Self::E113UnknownOption,
            114 => Self::E114MissingQuote,
            115 => Self::E115MissingQuote2,
            116 => Self::E116InvalidFuncArgs,
            806 => Self::E806FloatAsString,
            808 => Self::E808NumberFloatReq,
            974 => Self::E974BlobAsNumber,
            975 => Self::E975BlobAsString,
            976 => Self::E976BlobAsFloat,
            977 => Self::E977CompareBlobBlob,
            741 => Self::E741ValueLocked,
            742 => Self::E742CannotChange,
            _ => Self::Unknown,
        }
    }

    /// Check if this is a type error.
    pub const fn is_type_error(self) -> bool {
        matches!(
            self,
            Self::E701InvalidTypeForLen
                | Self::E703FuncrefAsNumber
                | Self::E706TypeMismatch
                | Self::E714ListRequired
                | Self::E715DictRequired
                | Self::E718FuncrefRequired
                | Self::E728DictAsNumber
                | Self::E729FuncrefAsString
                | Self::E730ListAsString
                | Self::E731DictAsString
                | Self::E734WrongVarType
                | Self::E806FloatAsString
                | Self::E808NumberFloatReq
                | Self::E974BlobAsNumber
                | Self::E975BlobAsString
                | Self::E976BlobAsFloat
        )
    }

    /// Check if this is a comparison error.
    pub const fn is_compare_error(self) -> bool {
        matches!(
            self,
            Self::E735CompareDictDict
                | Self::E736CompareListList
                | Self::E738CompareFuncFunc
                | Self::E977CompareBlobBlob
        )
    }

    /// Check if this is a function error.
    pub const fn is_function_error(self) -> bool {
        matches!(
            self,
            Self::E117UnknownFunc
                | Self::E118TooManyArgs
                | Self::E119NotEnoughArgs
                | Self::E122FuncExists
                | Self::E123UndefinedFunc
                | Self::E127CannotRedefine
                | Self::E131CannotDelete
                | Self::E132FuncDepth
        )
    }

    /// Check if this is a lock error.
    pub const fn is_lock_error(self) -> bool {
        matches!(self, Self::E741ValueLocked | Self::E742CannotChange)
    }
}

// =============================================================================
// Error Result Type
// =============================================================================

/// Result type for VimL evaluation operations.
pub type EvalOpResult<T> = Result<T, EvalError>;

/// A VimL evaluation error.
#[derive(Debug, Clone)]
pub struct EvalError {
    /// Error code
    pub code: EvalErrorCode,
    /// Error message (may contain format placeholders)
    pub message: String,
    /// Optional context (variable name, function name, etc.)
    pub context: Option<String>,
}

impl EvalError {
    /// Create a new error.
    pub fn new(code: EvalErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
        }
    }

    /// Create an error with context.
    pub fn with_context(
        code: EvalErrorCode,
        message: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            context: Some(context.into()),
        }
    }

    /// Create a type error.
    pub fn type_error(expected: &str, got: &str) -> Self {
        Self::with_context(
            EvalErrorCode::E706TypeMismatch,
            format!("expected {expected}, got {got}"),
            expected,
        )
    }

    /// Create an unknown variable error.
    pub fn undefined_variable(name: &str) -> Self {
        Self::with_context(
            EvalErrorCode::E121UndefinedVar,
            format!("Undefined variable: {name}"),
            name,
        )
    }

    /// Create an unknown function error.
    pub fn unknown_function(name: &str) -> Self {
        Self::with_context(
            EvalErrorCode::E117UnknownFunc,
            format!("Unknown function: {name}"),
            name,
        )
    }

    /// Create a list index out of range error.
    pub fn list_index_out_of_range(index: i64) -> Self {
        Self::with_context(
            EvalErrorCode::E684ListIndexOOR,
            format!("List index out of range: {index}"),
            index.to_string(),
        )
    }

    /// Create a value locked error.
    pub fn value_locked(name: &str) -> Self {
        Self::with_context(
            EvalErrorCode::E741ValueLocked,
            format!("Value is locked: {name}"),
            name,
        )
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{}: {}", self.code.number(), self.message)
    }
}

impl std::error::Error for EvalError {}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    /// Show error message.
    fn emsg(msg: *const c_char) -> c_int;

    /// Show error message with format.
    fn semsg(fmt: *const c_char, ...) -> c_int;

    /// Show warning message.
    fn msg(msg: *const c_char) -> c_int;

    /// Check if errors should be suppressed.
    static mut emsg_silent: c_int;

    /// Get did_emsg counter.
    fn did_emsg_get() -> c_int;

    /// Check if we're aborting due to error.
    fn aborting() -> c_int;

    /// Update force_abort flag.
    fn update_force_abort();

    /// Clear error message.
    fn emsg_clear();
}

// =============================================================================
// Error Reporting
// =============================================================================

/// Check if error messages are being suppressed (e.g., during `:silent`).
pub fn is_emsg_silent() -> bool {
    unsafe { emsg_silent != 0 }
}

/// Check if an error has occurred (did_emsg counter > 0).
pub fn has_emsg() -> bool {
    unsafe { did_emsg_get() != 0 }
}

/// Check if we should abort execution due to an error.
pub fn is_aborting() -> bool {
    unsafe { aborting() != 0 }
}

/// Report a simple error message.
///
/// # Safety
/// - Must be called when it's safe to report errors (not during cleanup, etc.)
pub unsafe fn report_error(msg: &str) -> c_int {
    let c_msg = std::ffi::CString::new(msg).unwrap_or_default();
    emsg(c_msg.as_ptr())
}

/// Report an EvalError to Neovim.
///
/// # Safety
/// - Must be called when it's safe to report errors
pub unsafe fn report_eval_error(err: &EvalError) -> c_int {
    let msg = err.to_string();
    let c_msg = std::ffi::CString::new(msg).unwrap_or_default();
    emsg(c_msg.as_ptr())
}

// =============================================================================
// Error State
// =============================================================================

/// Saved error state for try/catch blocks.
#[derive(Debug, Clone, Copy)]
pub struct ErrorState {
    /// did_emsg value when state was saved
    pub did_emsg: c_int,
    /// Whether errors were being suppressed
    pub emsg_silent: bool,
}

impl ErrorState {
    /// Save current error state.
    pub fn save() -> Self {
        Self {
            did_emsg: unsafe { did_emsg_get() },
            emsg_silent: is_emsg_silent(),
        }
    }

    /// Check if an error occurred since this state was saved.
    pub fn has_new_error(&self) -> bool {
        unsafe { did_emsg_get() > self.did_emsg }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if error messages are suppressed.
#[no_mangle]
pub extern "C" fn rs_eval_emsg_silent() -> c_int {
    c_int::from(is_emsg_silent())
}

/// Check if an error has occurred.
#[no_mangle]
pub extern "C" fn rs_eval_has_emsg() -> c_int {
    c_int::from(has_emsg())
}

/// Check if we should abort.
#[no_mangle]
pub extern "C" fn rs_eval_is_aborting() -> c_int {
    c_int::from(is_aborting())
}

/// Get error code number from enum.
#[no_mangle]
pub extern "C" fn rs_eval_error_number(code: EvalErrorCode) -> c_int {
    code.number()
}

/// Check if error code is a type error.
#[no_mangle]
pub extern "C" fn rs_eval_is_type_error(code: EvalErrorCode) -> c_int {
    c_int::from(code.is_type_error())
}

/// Check if error code is a function error.
#[no_mangle]
pub extern "C" fn rs_eval_is_function_error(code: EvalErrorCode) -> c_int {
    c_int::from(code.is_function_error())
}

/// Check if error code is a lock error.
#[no_mangle]
pub extern "C" fn rs_eval_is_lock_error(code: EvalErrorCode) -> c_int {
    c_int::from(code.is_lock_error())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_number() {
        assert_eq!(EvalErrorCode::E117UnknownFunc.number(), 117);
        assert_eq!(EvalErrorCode::E684ListIndexOOR.number(), 684);
        assert_eq!(EvalErrorCode::E741ValueLocked.number(), 741);
    }

    #[test]
    fn test_error_code_from_number() {
        assert_eq!(
            EvalErrorCode::from_number(117),
            EvalErrorCode::E117UnknownFunc
        );
        assert_eq!(
            EvalErrorCode::from_number(684),
            EvalErrorCode::E684ListIndexOOR
        );
        assert_eq!(EvalErrorCode::from_number(9999), EvalErrorCode::Unknown);
    }

    #[test]
    fn test_error_categories() {
        assert!(EvalErrorCode::E703FuncrefAsNumber.is_type_error());
        assert!(EvalErrorCode::E728DictAsNumber.is_type_error());
        assert!(!EvalErrorCode::E117UnknownFunc.is_type_error());

        assert!(EvalErrorCode::E735CompareDictDict.is_compare_error());
        assert!(!EvalErrorCode::E117UnknownFunc.is_compare_error());

        assert!(EvalErrorCode::E117UnknownFunc.is_function_error());
        assert!(EvalErrorCode::E118TooManyArgs.is_function_error());
        assert!(!EvalErrorCode::E728DictAsNumber.is_function_error());

        assert!(EvalErrorCode::E741ValueLocked.is_lock_error());
        assert!(!EvalErrorCode::E117UnknownFunc.is_lock_error());
    }

    #[test]
    fn test_eval_error_display() {
        let err = EvalError::unknown_function("foo");
        let s = err.to_string();
        assert!(s.contains("E117"));
        assert!(s.contains("foo"));
    }

    #[test]
    fn test_eval_error_constructors() {
        let err = EvalError::type_error("Number", "String");
        assert_eq!(err.code, EvalErrorCode::E706TypeMismatch);
        assert!(err.message.contains("Number"));
        assert!(err.message.contains("String"));

        let err = EvalError::undefined_variable("myvar");
        assert_eq!(err.code, EvalErrorCode::E121UndefinedVar);
        assert_eq!(err.context, Some("myvar".to_string()));

        let err = EvalError::list_index_out_of_range(10);
        assert_eq!(err.code, EvalErrorCode::E684ListIndexOOR);
        assert!(err.message.contains("10"));

        let err = EvalError::value_locked("v:count");
        assert_eq!(err.code, EvalErrorCode::E741ValueLocked);
    }

    #[test]
    fn test_error_state() {
        // Can only test the structure, not the C function calls
        let state = ErrorState {
            did_emsg: 0,
            emsg_silent: false,
        };
        assert!(!state.emsg_silent);
    }
}
