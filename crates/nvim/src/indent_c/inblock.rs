//! Indenting a line inside an unclosed `{}` block.
//!
//! Two halves.  This file settles the *scope*: where the `{` is, how indented
//! the line holding it is, and what the line being indented is looking for --
//! 'cinoptions' `>` (`level`), `e`/`^` (where an imaginary brace sits), `{`
//! and `}` (`open_extra`/`close_extra`), `:`/`=` (`case`), `g`/`h`
//! (`scopedecl`), `b` (`case_break`), `N`/`E` (the C++ block openers).
//!
//! Then [`BlockScan::run`] walks *backwards* from the cursor to that `{}`,
//! and [`lookfor`](super::lookfor) decides what each line it passes means.
//! The state of that walk is [`BlockScan`], and every field of it is
//! something one of those decisions needs.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

/// What the backwards scan should do after looking at one line.
#[derive(PartialEq, Eq)]
pub(crate) enum Step {
    /// Upstream's `continue`: keep walking up.
    Again,
    /// Upstream's `break`: `amount` is the answer.
    Done,
}

/// The backwards scan inside a `{}` block, and everything it carries.
pub(crate) struct BlockScan<'a> {
    /// The line being indented.
    pub line: &'a Line,
    /// The line the enclosing `{` is on -- the scan stops there.
    pub ourscope: linenr_T,
    /// Where that `{` is, which `LOOKFOR_COMMA` and the paren test compare
    /// against.
    pub brace: pos_T,
    /// Where the `{` sat on its line: [`BRACE_IN_COL0`], [`BRACE_AT_START`]
    /// or [`BRACE_AT_END`].
    pub start_brace: c_int,

    /// The answer being built.
    pub amount: c_int,
    /// The answer the scope alone implies, restored when the scan runs out.
    pub scope_amount: c_int,
    /// The indent of the line last looked at, `MAXCOL` for "none yet".
    pub cur_amount: c_int,
    /// The base for a continuation line, when one was computed.
    pub cont_amount: c_int,
    /// 'cinoptions' `+`, which the C++ base-class arm zeroes mid-scan.
    pub ind_continuation: c_int,
    /// How much `b_ind_open_extra` the scope's amount already includes, so
    /// the C++ namespace arm can take it back off.
    pub added_to_amount: c_int,

    /// What the scan is still searching for: one of the `LOOKFOR_*` states.
    pub lookfor: c_int,
    /// How many `while (...)` ends are still waiting for their `do`.
    pub whilelevel: c_int,
    /// 'cinoptions' `b`: a `break` may line up with its `case`.
    pub lookfor_break: bool,
    /// The scope opened at column 0, so a `namespace` may be above it.
    pub lookfor_cpp_namespace: bool,
    /// The line a raw string seen during the walk starts on; a line that
    /// *is* one must not become `LOOKFOR_UNTERM`.
    pub raw_string_start: linenr_T,
    /// `cin_is_cpp_baseclass`'s answer, cached across the walk.
    pub cache: cpp_baseclass_cache_T,
    /// The line being indented is a Javascript `key:` -- checked once, on the
    /// first line the scan reaches.
    pub js_cur_has_key: bool,
}

/// The indent for a line inside the `{}` block opened at `brace`.
///
/// # Safety
/// Moves the cursor; may unlock the current line.
pub(crate) unsafe fn indent_in_block(line: &Line, brace: pos_T) -> c_int {
    let ourscope = brace.lnum;
    // SAFETY: on the main thread with a current buffer; `ml_get` hands back a
    // NUL-terminated line and reports a bad line number itself.
    let start = unsafe { ml_get(ourscope) };

    // How indented is the block in general?  If the brace was at the
    // start of its line, use that; otherwise take the line's own indent
    // and add the "imaginary indent" below.
    // SAFETY: `start` is that NUL-terminated line, so `skipwhite` stops
    // inside it and the byte it stops on is readable.
    let brace_at_line_start = unsafe { *skipwhite(start) as u8 == b'{' };
    let (mut amount, start_brace) = if brace_at_line_start {
        // SAFETY: the same line; a NUL-terminated string has a first byte.
        let at_col0 = unsafe { *start as u8 == b'{' };
        (
            // SAFETY: `brace` came from a paren search over this buffer.
            unsafe { line_vcol(brace.lnum, brace.col) },
            if at_col0 {
                BRACE_IN_COL0
            } else {
                BRACE_AT_START
            },
        )
    } else {
        // The opening brace may have been on a continuation line; find
        // the start of *that*, by matching the rightmost paren.
        cur_win().w_cursor.lnum = ourscope;
        let mut lnum = ourscope;
        // SAFETY: `start` is a NUL-terminated line and the cursor sits on
        // `ourscope`, a line of the current buffer.  The chain stays whole:
        // `find_match_paren` searches from where `find_last_paren` put the
        // cursor, so it may only run once that returned true.
        if unsafe { find_last_paren(start, b'(', b')') }
            && let Some(trypos) = unsafe { find_match_paren(cur_buf().b_ind_maxparen) }
        {
            lnum = trypos.lnum;
        }

        // It could have been something like
        //         case 1: if (asdf &&
        //                      ldfd) {
        //                  }
        let mut l = ::core::ptr::null::<c_char>();
        let js_or_keep_case = cur_buf().b_ind_js != 0 || cur_buf().b_ind_keep_case_label != 0;
        // SAFETY: the cursor is on a line of the current buffer, so
        // `get_cursor_line_ptr` hands back a NUL-terminated one for
        // `skipwhite` and `cin_iscase` to walk.  Kept behind the option
        // test, which is what decides whether the line is read at all.
        let amount =
            if js_or_keep_case && unsafe { cin_iscase(skipwhite(get_cursor_line_ptr()), false) } {
                // SAFETY: the cursor is still on a line of the current buffer.
                unsafe { get_indent() }
            } else if cur_buf().b_ind_js != 0 {
                // SAFETY: `lnum` is a line of the current buffer -- either
                // `ourscope` or the line a paren match reported.
                unsafe { get_indent_lnum(lnum) }
            } else {
                // SAFETY: the same line number, and `l` is a local out-parameter.
                unsafe { skip_label(lnum, &mut l) }
            };
        (amount, BRACE_AT_END)
    };

    // For Javascript, check whether the line starts with "key:".
    // SAFETY: `line.theline` is a NUL-terminated copy of the cursor's line,
    // alive for the whole call.  The `&&` keeps the call behind the option.
    let js_cur_has_key = cur_buf().b_ind_js != 0 && unsafe { cin_has_js_key(line.theline) };

    // A closing brace is where we want to be already; some people want it
    // lined up with something other than the open brace.
    // SAFETY: `line.theline` is still valid.
    if unsafe { line.starts_with(b'}') } {
        return amount + cur_buf().b_ind_close_extra;
    }

    // An "else" wants its "if", a "while" its "do".
    // SAFETY: `line.theline` is a NUL-terminated line and `cur_curpos.lnum`
    // is the cursor's own line of the current buffer.
    let lookfor = if unsafe { cin_iselse(line.theline) } {
        LOOKFOR_IF
    } else if unsafe { cin_iswhileofdo(line.theline, line.cur_curpos.lnum) } {
        LOOKFOR_DO
    } else {
        LOOKFOR_INITIAL
    };
    if lookfor != LOOKFOR_INITIAL {
        cur_win().w_cursor.lnum = line.cur_curpos.lnum;
        // SAFETY: the cursor is on a line of the current buffer, and
        // `ourscope` is a line of it too -- where the search stops.
        if unsafe { find_match(lookfor, ourscope) } {
            // SAFETY: a successful match left the cursor on a line of it.
            return unsafe { get_indent() };
        }
    }

    // Not an "else" or a "while-of-do" (or the match failed).  Set the
    // amount for the case where the search below finds nothing.
    let mut added_to_amount = 0;
    let mut lookfor_cpp_namespace = false;
    if start_brace == BRACE_IN_COL0 {
        // A brace *really* at the left margin: use the imaginary
        // location of one, and look further back for a `namespace`.
        amount = cur_buf().b_ind_open_left_imag;
        lookfor_cpp_namespace = true;
    } else if start_brace == BRACE_AT_END {
        amount += cur_buf().b_ind_open_imag;
        // SAFETY: the cursor is on a line of the current buffer, which
        // `skipwhite` walks no further than its NUL.
        let l = unsafe { skipwhite(get_cursor_line_ptr()) };
        // SAFETY: `l` points into that same NUL-terminated line.
        if unsafe { cin_is_cpp_namespace(l) } {
            amount += cur_buf().b_ind_cpp_namespace;
        } else if unsafe { cin_is_cpp_extern_c(l) } {
            amount += cur_buf().b_ind_cpp_extern_c;
        }
    } else {
        // Compensate for adding `b_ind_open_extra` later.
        amount = (amount - cur_buf().b_ind_open_extra).max(0);
    }

    // What kind of line is being indented decides what to search for.
    let mut lookfor_break = false;
    // SAFETY: `line.theline` is a NUL-terminated copy of the cursor's line.
    let lookfor = if unsafe { cin_iscase(line.theline, false) } {
        amount += cur_buf().b_ind_case;
        LOOKFOR_CASE // a switch() label: find a previous one
    } else if unsafe { cin_isscopedecl(line.theline) } {
        amount += cur_buf().b_ind_scopedecl;
        LOOKFOR_SCOPEDECL // private:, ...: the class declaration
    } else {
        // SAFETY: the same; the `&&` keeps the call behind the option test.
        if cur_buf().b_ind_case_break != 0 && unsafe { cin_isbreak(line.theline) } {
            lookfor_break = true;
        }
        amount += cur_buf().b_ind_level;
        LOOKFOR_INITIAL
    };

    let scan = BlockScan {
        line,
        ourscope,
        brace,
        start_brace,
        amount,
        scope_amount: amount,
        cur_amount: MAXCOL,
        cont_amount: 0,
        // A copy: the C++ base-class arm sets it to zero mid-scan.
        ind_continuation: cur_buf().b_ind_continuation,
        added_to_amount,
        lookfor,
        whilelevel: 0,
        lookfor_break,
        lookfor_cpp_namespace,
        raw_string_start: 0,
        cache: cpp_baseclass_cache_T {
            found: 0,
            lpos: lpos_T {
                lnum: MAXLNUM as linenr_T,
                col: 0,
            },
        },
        js_cur_has_key,
    };
    // SAFETY: the scan moves the cursor over lines of the current buffer,
    // which is exactly what this function's own contract promises.
    unsafe { scan.run() }
}

impl BlockScan<'_> {
    /// Walk back from the cursor to `ourscope`, looking for something to line
    /// up with.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn run(mut self) -> c_int {
        cur_win().w_cursor = self.line.cur_curpos;
        loop {
            cur_win().w_cursor.lnum -= 1;
            cur_win().w_cursor.col = 0;

            // Back at the start of our scope: line up with it.
            if cur_win().w_cursor.lnum <= self.ourscope {
                // SAFETY: this function's own contract -- the cursor is ours
                // to move and the line it is on may be unlocked.
                if unsafe { self.at_scope_start() } == Step::Done {
                    break;
                }
                continue;
            }
            // SAFETY: the same.
            if unsafe { self.step() } == Step::Done {
                break;
            }
        }
        self.amount
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
