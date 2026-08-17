//! `[[=a=]]`: the equivalence class each base character expands to, emitted
//! straight into the postfix form.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::postfix;
use crate::regexp::equi_class;

/// Emit the equivalence class `c` belongs to as postfix items.
///
/// Each member is concatenated onto the collection being built, so the class
/// reads as `a NFA_CONCAT b NFA_CONCAT …`. A character in no class stands for
/// itself. The table lives in the shared [`equi_class`] module; upstream kept
/// a copy of it here and another in the backtracking compiler.
pub(crate) fn nfa_emit_equi_class(c: c_int) {
    match equi_class::nfa_class_of(c) {
        Some(class) => {
            for member in class.nfa_members() {
                postfix::emit_concat(member);
            }
        }
        None => postfix::emit_concat(c),
    }
}
