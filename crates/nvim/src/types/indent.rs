#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

/// One of the three engines that answer "what indent does the cursor line
/// want": 'indentexpr', the Lisp indenter and the C indenter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Indenter {
    /// 'indentexpr' (`get_expr_indent`).
    Expr,
    /// 'lisp' (`get_lisp_indent`).
    Lisp,
    /// 'cindent' (`get_c_indent`).
    C,
}
