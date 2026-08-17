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
    unsafe {
        let ourscope = brace.lnum;
        let start = ml_get(ourscope);

        // How indented is the block in general?  If the brace was at the
        // start of its line, use that; otherwise take the line's own indent
        // and add the "imaginary indent" below.
        let (mut amount, start_brace) = if *skipwhite(start) as u8 == b'{' {
            let at_col0 = *start as u8 == b'{';
            (
                line_vcol(brace.lnum, brace.col),
                if at_col0 {
                    BRACE_IN_COL0
                } else {
                    BRACE_AT_START
                },
            )
        } else {
            // The opening brace may have been on a continuation line; find
            // the start of *that*, by matching the rightmost paren.
            (*curwin.get()).w_cursor.lnum = ourscope;
            let mut lnum = ourscope;
            if find_last_paren(start, b'(', b')') {
                let trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                if !trypos.is_null() {
                    lnum = (*trypos).lnum;
                }
            }

            // It could have been something like
            //         case 1: if (asdf &&
            //                      ldfd) {
            //                  }
            let mut l = ::core::ptr::null::<c_char>();
            let amount = if ((*curbuf.get()).b_ind_js != 0
                || (*curbuf.get()).b_ind_keep_case_label != 0)
                && cin_iscase(skipwhite(get_cursor_line_ptr()), false)
            {
                get_indent()
            } else if (*curbuf.get()).b_ind_js != 0 {
                get_indent_lnum(lnum)
            } else {
                skip_label(lnum, &mut l)
            };
            (amount, BRACE_AT_END)
        };

        // For Javascript, check whether the line starts with "key:".
        let js_cur_has_key = (*curbuf.get()).b_ind_js != 0 && cin_has_js_key(line.theline);

        // A closing brace is where we want to be already; some people want it
        // lined up with something other than the open brace.
        if line.starts_with(b'}') {
            return amount + (*curbuf.get()).b_ind_close_extra;
        }

        // An "else" wants its "if", a "while" its "do".
        let lookfor = if cin_iselse(line.theline) {
            LOOKFOR_IF
        } else if cin_iswhileofdo(line.theline, line.cur_curpos.lnum) {
            LOOKFOR_DO
        } else {
            LOOKFOR_INITIAL
        };
        if lookfor != LOOKFOR_INITIAL {
            (*curwin.get()).w_cursor.lnum = line.cur_curpos.lnum;
            if find_match(lookfor, ourscope) {
                return get_indent();
            }
        }

        // Not an "else" or a "while-of-do" (or the match failed).  Set the
        // amount for the case where the search below finds nothing.
        let mut added_to_amount = 0;
        let mut lookfor_cpp_namespace = false;
        if start_brace == BRACE_IN_COL0 {
            // A brace *really* at the left margin: use the imaginary
            // location of one, and look further back for a `namespace`.
            amount = (*curbuf.get()).b_ind_open_left_imag;
            lookfor_cpp_namespace = true;
        } else if start_brace == BRACE_AT_END {
            amount += (*curbuf.get()).b_ind_open_imag;
            let l = skipwhite(get_cursor_line_ptr());
            if cin_is_cpp_namespace(l) {
                amount += (*curbuf.get()).b_ind_cpp_namespace;
            } else if cin_is_cpp_extern_c(l) {
                amount += (*curbuf.get()).b_ind_cpp_extern_c;
            }
        } else {
            // Compensate for adding `b_ind_open_extra` later.
            amount = (amount - (*curbuf.get()).b_ind_open_extra).max(0);
        }

        // What kind of line is being indented decides what to search for.
        let mut lookfor_break = false;
        let lookfor = if cin_iscase(line.theline, false) {
            amount += (*curbuf.get()).b_ind_case;
            LOOKFOR_CASE // a switch() label: find a previous one
        } else if cin_isscopedecl(line.theline) {
            amount += (*curbuf.get()).b_ind_scopedecl;
            LOOKFOR_SCOPEDECL // private:, ...: the class declaration
        } else {
            if (*curbuf.get()).b_ind_case_break != 0 && cin_isbreak(line.theline) {
                lookfor_break = true;
            }
            amount += (*curbuf.get()).b_ind_level;
            LOOKFOR_INITIAL
        };

        BlockScan {
            line,
            ourscope,
            brace,
            start_brace,
            amount,
            scope_amount: amount,
            cur_amount: MAXCOL,
            cont_amount: 0,
            // A copy: the C++ base-class arm sets it to zero mid-scan.
            ind_continuation: (*curbuf.get()).b_ind_continuation,
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
        }
        .run()
    }
}

impl BlockScan<'_> {
    /// Walk back from the cursor to `ourscope`, looking for something to line
    /// up with.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn run(mut self) -> c_int {
        unsafe {
            (*curwin.get()).w_cursor = self.line.cur_curpos;
            loop {
                (*curwin.get()).w_cursor.lnum -= 1;
                (*curwin.get()).w_cursor.col = 0;

                // Back at the start of our scope: line up with it.
                if (*curwin.get()).w_cursor.lnum <= self.ourscope {
                    if self.at_scope_start() == Step::Done {
                        break;
                    }
                    continue;
                }
                if self.step() == Step::Done {
                    break;
                }
            }
            self.amount
        }
    }
}
