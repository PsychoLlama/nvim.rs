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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

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
    pub ourscope: LineNr,
    /// Where that `{` is, which `LOOKFOR_COMMA` and the paren test compare
    /// against.
    pub brace: Pos,
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
    pub raw_string_start: LineNr,
    /// `in_baseclass_list`'s answer, cached across the walk.
    pub cache: CppBaseclassCache,
    /// The line being indented is a Javascript `key:` -- checked once, on the
    /// first line the scan reaches.
    pub js_cur_has_key: bool,
}

/// The indent for a line inside the `{}` block opened at `brace`.
pub(crate) fn indent_in_block(line: &Line, brace: Pos) -> c_int {
    let ourscope = brace.lnum;

    // How indented is the block in general?  If the brace was at the
    // start of its line, use that; otherwise take the line's own indent
    // and add the "imaginary indent" below.
    let (brace_at_line_start, brace_at_col0) = {
        let mut lines = Lines::current();
        let start = lines.line(ourscope);
        (
            byte_at(start, skip::white(start)) == b'{',
            start.first() == Some(&b'{'),
        )
    };
    let (mut amount, start_brace) = if brace_at_line_start {
        (
            line_vcol(brace.lnum, brace.col),
            if brace_at_col0 {
                BRACE_IN_COL0
            } else {
                BRACE_AT_START
            },
        )
    } else {
        // The opening brace may have been on a continuation line; find
        // the start of *that*, by matching the rightmost paren.
        Win::current().w_cursor.lnum = ourscope;
        let mut lnum = ourscope;
        // The borrow ends with the statement: the match search reads other
        // lines, and only runs once `find_last_paren` found one.
        let has_paren = find_last_paren(Lines::current().line(ourscope), b'(', b')');
        if has_paren && let Some(trypos) = find_match_paren(Buf::current().b_ind_maxparen) {
            lnum = trypos.lnum;
        }

        // It could have been something like
        //         case 1: if (asdf &&
        //                      ldfd) {
        //                  }
        let js_or_keep_case =
            Buf::current().b_ind_js != 0 || Buf::current().b_ind_keep_case_label != 0;
        let starts_case = js_or_keep_case && {
            let cursor_lnum = Win::current().w_cursor.lnum;
            let mut lines = Lines::current();
            let text = lines.line(cursor_lnum);
            is_case_label(text, skip::white(text), false)
        };
        let amount = if starts_case {
            // SAFETY: the cursor is still on a line of the current buffer.
            get_indent()
        } else if Buf::current().b_ind_js != 0 {
            // SAFETY: `lnum` is a line of the current buffer -- either
            // `ourscope` or the line a paren match reported.
            get_indent_lnum(lnum)
        } else {
            skip_label(lnum).0
        };
        (amount, BRACE_AT_END)
    };

    // For Javascript, check whether the line starts with "key:".
    // The `&&` keeps the call behind the option.
    let js_cur_has_key = Buf::current().b_ind_js != 0 && has_js_key(line.theline(), 0);

    // A closing brace is where we want to be already; some people want it
    // lined up with something other than the open brace.
    if line.starts_with(b'}') {
        return amount + Buf::current().b_ind_close_extra;
    }

    // An "else" wants its "if", a "while" its "do".
    let lookfor = if is_else(line.theline(), 0) {
        LOOKFOR_IF
    } else if starts_while(line.theline(), 0) && while_closes_do(line.cur_curpos.lnum) {
        LOOKFOR_DO
    } else {
        LOOKFOR_INITIAL
    };
    if lookfor != LOOKFOR_INITIAL {
        Win::current().w_cursor.lnum = line.cur_curpos.lnum;
        if find_match(lookfor, ourscope) {
            // SAFETY: a successful match left the cursor on a line of it.
            return get_indent();
        }
    }

    // Not an "else" or a "while-of-do" (or the match failed).  Set the
    // amount for the case where the search below finds nothing.
    let added_to_amount = 0;
    let mut lookfor_cpp_namespace = false;
    if start_brace == BRACE_IN_COL0 {
        // A brace *really* at the left margin: use the imaginary
        // location of one, and look further back for a `namespace`.
        amount = Buf::current().b_ind_open_left_imag;
        lookfor_cpp_namespace = true;
    } else if start_brace == BRACE_AT_END {
        amount += Buf::current().b_ind_open_imag;
        let cursor_lnum = Win::current().w_cursor.lnum;
        let mut lines = Lines::current();
        let text = lines.line(cursor_lnum);
        let at = skip::white(text);
        if opens_namespace(text, at) {
            amount += Buf::current().b_ind_cpp_namespace;
        } else if opens_extern_c(text, at) {
            amount += Buf::current().b_ind_cpp_extern_c;
        }
    } else {
        // Compensate for adding `b_ind_open_extra` later.
        amount = (amount - Buf::current().b_ind_open_extra).max(0);
    }

    // What kind of line is being indented decides what to search for.
    let mut lookfor_break = false;
    let lookfor = if is_case_label(line.theline(), 0, false) {
        amount += Buf::current().b_ind_case;
        LOOKFOR_CASE // a switch() label: find a previous one
    } else if is_scope_decl(line.theline(), 0) {
        amount += Buf::current().b_ind_scopedecl;
        LOOKFOR_SCOPEDECL // private:, ...: the class declaration
    } else {
        // The `&&` keeps the call behind the option test.
        if Buf::current().b_ind_case_break != 0 && is_break(line.theline(), 0) {
            lookfor_break = true;
        }
        amount += Buf::current().b_ind_level;
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
        ind_continuation: Buf::current().b_ind_continuation,
        added_to_amount,
        lookfor,
        whilelevel: 0,
        lookfor_break,
        lookfor_cpp_namespace,
        raw_string_start: 0,
        cache: CppBaseclassCache {
            found: 0,
            lpos: LPos {
                lnum: MAXLNUM,
                col: 0,
            },
        },
        js_cur_has_key,
    };
    scan.run()
}

impl BlockScan<'_> {
    /// Walk back from the cursor to `ourscope`, looking for something to line
    /// up with.
    fn run(mut self) -> c_int {
        Win::current().w_cursor = self.line.cur_curpos;
        loop {
            Win::current().w_cursor.lnum -= 1;
            Win::current().w_cursor.col = 0;

            // Back at the start of our scope: line up with it.
            if Win::current().w_cursor.lnum <= self.ourscope {
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
