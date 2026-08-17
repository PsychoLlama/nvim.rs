//! `[[=a=]]`: the equivalence class each base character expands to.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::compile::regmbc;
use crate::regexp::equi_class;

/// Produce the bytes for the equivalence class `c` belongs to.
///
/// A character in no class stands for itself. The table lives in the shared
/// [`equi_class`] module; upstream kept a copy of it here and another in the
/// NFA compiler.
pub(crate) fn reg_equi_class(c: c_int) {
    match equi_class::backtracking_class_of(c) {
        Some(class) => {
            for member in class.backtracking_members() {
                regmbc(member);
            }
        }
        None => regmbc(c),
    }
}
