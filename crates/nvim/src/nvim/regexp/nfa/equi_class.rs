//! `[[=a=]]`: the equivalence class each base character expands to, emitted
//! straight into the postfix form.

use core::ffi::c_int;

use super::compile::realloc_post_list;
use crate::src::nvim::regexp::equi_class;
use crate::src::nvim::regexp::{NFA_CONCAT, post_end, post_ptr};

/// Append one item to the postfix list, growing it if it is full.
///
/// The transpiled form of upstream's `EMIT` macro, which is open-coded at
/// every use.
fn post_emit(c: c_int) {
    // SAFETY: `post_ptr` stays within `post_start..post_end` by construction,
    // and `realloc_post_list` re-anchors all three after growing the list.
    unsafe {
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let slot = post_ptr.get();
        post_ptr.set(slot.add(1));
        *slot = c;
    }
}

/// Emit the equivalence class `c` belongs to as postfix items.
///
/// Each member is concatenated onto the collection being built, so the class
/// reads as `a NFA_CONCAT b NFA_CONCAT …`. A character in no class stands for
/// itself. The table lives in the shared [`equi_class`] module; upstream kept
/// a copy of it here and another in the backtracking compiler.
pub(crate) fn nfa_emit_equi_class(c: c_int) {
    let mut emit2 = |c| {
        post_emit(c);
        post_emit(NFA_CONCAT as c_int);
    };
    match equi_class::nfa_class_of(c) {
        Some(class) => {
            for member in class.nfa_members() {
                emit2(member);
            }
        }
        None => emit2(c),
    }
}
