//! What each line the backwards scan passes means.
//!
//! [`BlockScan`]'s `lookfor` is the state: what the scan is still searching
//! for.  It starts as `LOOKFOR_INITIAL` (or `LOOKFOR_CASE`/`LOOKFOR_SCOPEDECL`
//! when the line being indented is a label), and each line it walks past
//! either answers the question, refines it, or is skipped.
//!
//! | state | what the scan is looking for |
//! | --- | --- |
//! | `LOOKFOR_INITIAL` | anything at all -- nothing has been decided |
//! | `LOOKFOR_TERM` | a *terminated* statement, to line up with |
//! | `LOOKFOR_UNTERM` | the start of the unterminated statement above |
//! | `LOOKFOR_ENUM_OR_INIT` | whether a run of `,` lines is a declaration or an initialiser |
//! | `LOOKFOR_CASE` / `LOOKFOR_SCOPEDECL` | a previous `case`/`private:` label |
//! | `LOOKFOR_NOBREAK` / `LOOKFOR_ANY` | as above, past a lone `break` ('cinoptions' `b`) |
//! | `LOOKFOR_CPP_BASECLASS` | a base-class list before an opening brace |
//! | `LOOKFOR_JS_KEY` / `LOOKFOR_COMMA` | the Javascript object-literal shapes |
//!
//! Each method here answers [`Step`]: `Again` is upstream's `continue`,
//! `Done` its `break`, at which point `amount` is the answer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::{Buf, Win};
use core::ffi::c_char;

impl BlockScan<'_> {
    /// A continuation line's amount: the remembered base, or one more level.
    fn continuation(&mut self) {
        if self.cont_amount > 0 {
            self.amount = self.cont_amount;
        } else {
            self.amount += self.ind_continuation;
        }
    }

    /// Put the cursor one line *below* `lnum`, so that the scan's next
    /// decrement lands on `lnum` itself.
    fn resume_at(&self, lnum: linenr_T) {
        cur_win().w_cursor.lnum = lnum + 1;
        cur_win().w_cursor.col = 0;
    }

    /// The scan reached the line the enclosing `{` is on.
    ///
    /// Two states keep going past it: `LOOKFOR_ENUM_OR_INIT`, which has to
    /// decide whether the run of `,`-terminated lines was a declaration (add
    /// a continuation) or an initialiser (do not), and the C++ namespace
    /// hunt, which looks a further [`FIND_NAMESPACE_LIM`] lines back.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    pub(crate) unsafe fn at_scope_start(&mut self) -> Step {
        if self.lookfor == LOOKFOR_ENUM_OR_INIT {
            // SAFETY: the caller's promise, handed straight on.
            return unsafe { self.at_scope_start_enum_or_init() };
        }
        if self.lookfor == LOOKFOR_UNTERM {
            self.continuation();
            return Step::Done;
        }

        if self.lookfor != LOOKFOR_TERM
            && self.lookfor != LOOKFOR_CPP_BASECLASS
            && self.lookfor != LOOKFOR_COMMA
        {
            self.amount = self.scope_amount;
            // SAFETY: `self.line` keeps its copy of the text alive.
            if unsafe { self.line.starts_with(b'{') } {
                self.amount += cur_buf().b_ind_open_extra;
                self.added_to_amount = cur_buf().b_ind_open_extra;
            }
        }

        if self.lookfor_cpp_namespace {
            let lnum = cur_win().w_cursor.lnum;
            if lnum == self.ourscope {
                return Step::Again;
            }
            if lnum == 0 || lnum < self.ourscope - FIND_NAMESPACE_LIM {
                return Step::Done;
            }

            // SAFETY: on the main thread, with a current buffer.
            if let Some(trypos) = unsafe { ind_find_start_comment_or_raw_string(None) } {
                self.resume_at(trypos.lnum);
                return Step::Again;
            }

            // SAFETY: the cursor is on a line of the current buffer.
            let mut l = unsafe { get_cursor_line_ptr() }.cast_const();
            // SAFETY: `l` is that line, NUL-terminated; the `w_cursor.lnum`
            // borrow is sound -- `cin_ispreproc_cont` never reads `curwin`.
            let is_preproc = unsafe {
                cin_ispreproc_cont(&mut l, &mut cur_win().w_cursor.lnum, &mut self.amount)
            };
            if is_preproc {
                return Step::Again;
            }

            // Finally, the actual check for "namespace".
            // SAFETY: `l` is a NUL-terminated line.
            if unsafe { cin_is_cpp_namespace(l) } {
                self.amount += cur_buf().b_ind_cpp_namespace - self.added_to_amount;
                return Step::Done;
            }
            // SAFETY: the same.
            if unsafe { cin_is_cpp_extern_c(l) } {
                self.amount += cur_buf().b_ind_cpp_extern_c - self.added_to_amount;
                return Step::Done;
            }
            // SAFETY: the same.
            if unsafe { cin_nocode(l) } {
                return Step::Again;
            }
        }
        Step::Done
    }

    /// [`at_scope_start`](Self::at_scope_start) for `LOOKFOR_ENUM_OR_INIT`.
    ///
    /// ```text
    /// int x,
    ///     here;   <-- a declaration: add a continuation
    /// enum { a,
    ///        here  <-- an initialiser: do not
    /// ```
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn at_scope_start_enum_or_init(&mut self) -> Step {
        let lnum = cur_win().w_cursor.lnum;
        if lnum == 0 || lnum < self.ourscope - cur_buf().b_ind_maxparen {
            // Nothing found (abusing `b_ind_maxparen` as the limit):
            // assume a terminated line, i.e. a variable initialisation.
            if self.cont_amount > 0 {
                self.amount = self.cont_amount;
            } else if cur_buf().b_ind_js == 0 {
                self.amount += self.ind_continuation;
            }
            return Step::Done;
        }

        // SAFETY: on the main thread, with a current buffer.
        if let Some(trypos) = unsafe { ind_find_start_comment_or_raw_string(None) } {
            self.resume_at(trypos.lnum);
            return Step::Again;
        }

        // SAFETY: the cursor is on a line of the current buffer.
        let mut l = unsafe { get_cursor_line_ptr() }.cast_const();
        // SAFETY: `l` is that line, NUL-terminated; the `w_cursor.lnum` borrow
        // is sound -- `cin_ispreproc_cont` never reads `curwin`.  The chain
        // stays whole: it may move `l` on before `cin_nocode` reads it.
        let skipped = unsafe {
            cin_ispreproc_cont(&mut l, &mut cur_win().w_cursor.lnum, &mut self.amount)
                || cin_nocode(l)
        };
        if skipped {
            return Step::Again;
        }

        // SAFETY: `l` is a NUL-terminated line.
        let terminated = unsafe { cin_isterminated(l, false, true) };

        // At top level and looking like a function declaration: done, it
        // is a variable declaration.
        // SAFETY: `l` is the cursor's NUL-terminated line.  The chain stays
        // whole: `cin_isfuncdecl` moves `l` on, so it must not run early.
        let is_var_decl = unsafe {
            self.start_brace != BRACE_IN_COL0
                || !cin_isfuncdecl(Some(&mut l), cur_win().w_cursor.lnum, 0)
        };
        if is_var_decl {
            // Terminated with another ',': a continued initialisation, so
            // no extra indent.
            // TODO(vim): does not work if a function declaration is split
            // over several lines -- `cin_isfuncdecl` says no then.
            if terminated == b',' {
                return Step::Done;
            }
            // An enum declaration or an assignment: done.
            // SAFETY: reads the cursor's line of the current buffer.
            if terminated != b';' && unsafe { cin_isinit() } {
                return Step::Done;
            }
            if terminated == 0 || terminated == b'{' {
                return Step::Again;
            }
        }

        if terminated != b';' {
            // Skip parens and braces: position on the rightmost paren so
            // that matching it takes us to the start of the line.
            let mut trypos = None;
            // SAFETY: `l` is the cursor's line; both move the cursor inside the
            // buffer, and the match runs only when a paren was found, as upstream.
            if unsafe { find_last_paren(l, b'(', b')') } {
                trypos = unsafe { find_match_paren(cur_buf().b_ind_maxparen) };
            }
            // SAFETY: the same, for the brace pair.
            if trypos.is_none() && unsafe { find_last_paren(l, b'{', b'}') } {
                trypos = unsafe { find_start_brace() };
            }
            if let Some(trypos) = trypos {
                self.resume_at(trypos.lnum);
                return Step::Again;
            }
        }

        // A variable declaration, so add indentation:
        //     int a,
        //        b;
        self.continuation();
        Step::Done
    }

    /// One line of the backwards scan, above `ourscope`.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    pub(crate) unsafe fn step(&mut self) -> Step {
        // In a comment or raw string now: skip to the start of it.
        // SAFETY: on the main thread with a current buffer.
        let trypos =
            unsafe { ind_find_start_comment_or_raw_string(Some(&mut self.raw_string_start)) };
        if let Some(trypos) = trypos {
            self.resume_at(trypos.lnum);
            return Step::Again;
        }

        // SAFETY: the cursor is on a line of the current buffer.
        let mut l = unsafe { get_cursor_line_ptr() }.cast_const();

        // A switch() label or a C++ scope declaration may be what we line
        // up relative to.
        // SAFETY: `l` is a NUL-terminated line.
        let iscase = unsafe { cin_iscase(l, false) };
        // SAFETY: the same.
        if iscase || unsafe { cin_isscopedecl(l) } {
            // SAFETY: the caller's promise, handed straight on.
            return unsafe { self.on_label(iscase) };
        }

        // Looking for a switch() label or scope declaration: ignore other
        // lines and skip `{}` blocks whole.
        if self.lookfor == LOOKFOR_CASE || self.lookfor == LOOKFOR_SCOPEDECL {
            // SAFETY: `l` is the cursor's line; both move the cursor inside
            // the buffer, and the hunt runs only when a brace was found.
            if unsafe { find_last_paren(l, b'{', b'}') }
                && let Some(trypos) = unsafe { find_start_brace() }
            {
                self.resume_at(trypos.lnum);
            }
            return Step::Again;
        }

        // Ignore jump labels with nothing after them.
        // SAFETY: reads the cursor's line of the current buffer.
        if cur_buf().b_ind_js == 0 && unsafe { cin_islabel() } {
            // SAFETY: the cursor's line is NUL-terminated.
            let after = unsafe { after_label(get_cursor_line_ptr()) };
            // SAFETY: `after` points into that line, once it is not NULL.
            if after.is_null() || unsafe { cin_nocode(after) } {
                return Step::Again;
            }
        }

        // Ignore #defines, comments and empty lines.  (Get the line
        // again: `cin_islabel` may have unlocked it.)
        // SAFETY: the cursor is on a line of the current buffer.
        l = unsafe { get_cursor_line_ptr() };
        // SAFETY: `l` is that line, NUL-terminated; the `w_cursor.lnum` borrow
        // is sound -- `cin_ispreproc_cont` never reads `curwin`.  The chain
        // stays whole: it may move `l` on before `cin_nocode` reads it.
        let skipped = unsafe {
            cin_ispreproc_cont(&mut l, &mut cur_win().w_cursor.lnum, &mut self.amount)
                || cin_nocode(l)
        };
        if skipped {
            return Step::Again;
        }

        // The start of a C++ base-class declaration or constructor
        // initialisation?
        let mut is_baseclass = false;
        if self.lookfor != LOOKFOR_TERM && cur_buf().b_ind_cpp_baseclass > 0 {
            // SAFETY: on the main thread with a current buffer.
            is_baseclass = unsafe { cin_is_cpp_baseclass(&mut self.cache) };
            // SAFETY: the same; the check above may have unlocked the line.
            l = unsafe { get_cursor_line_ptr() };
        }
        if is_baseclass {
            if self.lookfor == LOOKFOR_UNTERM {
                self.continuation();
                // SAFETY: `self.line` keeps its copy of the text alive.
            } else if unsafe { self.line.starts_with(b'{') } {
                // Need to find the start of the declaration.
                self.lookfor = LOOKFOR_UNTERM;
                self.ind_continuation = 0;
                return Step::Again;
            } else {
                // SAFETY: the column came out of `self.cache`.
                self.amount = unsafe { get_baseclass_amount(self.cache.lpos.col) };
            }
            return Step::Done;
        }
        if self.lookfor == LOOKFOR_CPP_BASECLASS {
            // Only interested in whether there is a base-class
            // declaration or initialisation before the opening brace.
            // SAFETY: `l` is a NUL-terminated line.
            return if unsafe { cin_isterminated(l, true, false) } != 0 {
                Step::Done
            } else {
                Step::Again
            };
        }

        // What happens next depends on the line being terminated.  A ','
        // only terminates if there is another unterminated statement
        // behind it:
        //   123,
        //   sizeof
        //      here
        // SAFETY: `l` is a NUL-terminated line.
        let terminated = unsafe { cin_isterminated(l, false, true) };

        if self.js_cur_has_key {
            self.js_cur_has_key = false; // only check the first line
            if cur_buf().b_ind_js != 0 && terminated == b',' {
                // Inside a Javascript object:
                //   key: something,  <- align with this
                //   key: something
                // or:
                //   key: something +  <- align with this
                //       something,
                //   key: something
                self.lookfor = LOOKFOR_JS_KEY;
            }
        }
        // SAFETY: `l` is a NUL-terminated line.
        if self.lookfor == LOOKFOR_JS_KEY && unsafe { cin_has_js_key(l) } {
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = unsafe { get_indent() };
            return Step::Done;
        }
        if self.lookfor == LOOKFOR_COMMA {
            if self.brace.lnum >= cur_win().w_cursor.lnum {
                return Step::Done;
            }
            if terminated == b',' {
                // The line below is the one that starts a (possibly
                // broken) line ending in a comma.
                return Step::Done;
            }
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = unsafe { get_indent() };
            if cur_win().w_cursor.lnum - 1 == self.ourscope {
                // The line above starts the scope, so this line is the
                // one that starts the comma-terminated line.
                return Step::Done;
            }
        }

        // SAFETY: the caller's promise, handed on; `l` is NUL-terminated.
        if terminated == 0 || (self.lookfor != LOOKFOR_UNTERM && terminated == b',') {
            unsafe { self.on_unterminated(l, terminated) }
        } else if unsafe { cin_iswhileofdo_end(terminated) } {
            unsafe { self.on_while_of_do_end() }
        } else {
            unsafe { self.on_terminated() }
        }
    }

    /// The line is a `case`/`default` label or a scope declaration.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_label(&mut self, iscase: bool) -> Step {
        // Only a cpp base class is still of interest.
        if self.lookfor == LOOKFOR_CPP_BASECLASS {
            return Step::Done;
        }
        // Looking for a "do", labels are not interesting.
        if self.whilelevel > 0 {
            return Step::Again;
        }

        //  case xx:
        //      c = 99 +        <- this indent plus continuation
        // ->          here;
        if self.lookfor == LOOKFOR_UNTERM || self.lookfor == LOOKFOR_ENUM_OR_INIT {
            self.continuation();
            return Step::Done;
        }

        // case xx: <- line up with this case
        //     x = 333;
        // case yy:
        if (iscase && self.lookfor == LOOKFOR_CASE)
            || (iscase && self.lookfor_break)
            || (!iscase && self.lookfor == LOOKFOR_SCOPEDECL)
        {
            // Check that this label is not for another switch().
            // SAFETY: moves the cursor inside the current buffer.
            let brace = unsafe { find_start_brace() };
            if brace.is_none_or(|trypos| trypos.lnum == self.ourscope) {
                // SAFETY: reads the cursor's line of the current buffer.
                self.amount = unsafe { get_indent() };
                return Step::Done;
            }
            return Step::Again;
        }

        // SAFETY: the line number is the cursor's own.
        let n = unsafe { get_indent_nolabel(cur_win().w_cursor.lnum) };

        //   case xx: if (cond)         <- line up with this if
        //                y = y + 1;
        // ->         s = 99;
        //
        //   case xx:
        //       if (cond)          <- line up with this line
        //           y = y + 1;
        // ->    s = 99;
        if self.lookfor == LOOKFOR_TERM {
            if n != 0 {
                self.amount = n;
            }
            if !self.lookfor_break {
                return Step::Done;
            }
        }

        //   case xx: x = x + 1;        <- line up with this x
        // ->         y = y + 1;
        //
        //   case xx: if (cond)         <- line up with this if
        // ->              y = y + 1;
        if n != 0 {
            self.amount = n;
            // SAFETY: the cursor's line is NUL-terminated.
            let l = unsafe { after_label(get_cursor_line_ptr()) };
            // SAFETY: `l` points into that line, once it is not NULL.
            if !l.is_null() && unsafe { cin_is_cinword(l) } {
                // SAFETY: `self.line` keeps its copy of the text alive.
                self.amount += if unsafe { self.line.starts_with(b'{') } {
                    cur_buf().b_ind_open_extra
                } else {
                    cur_buf().b_ind_level + cur_buf().b_ind_no_brace
                };
            }
            return Step::Done;
        }

        // Try to get the indent of a statement before the label.  If
        // nothing is found, line up relative to the label.
        //      break;              <- may line up with this line
        //   case xx:
        // ->   y = 1;
        // SAFETY: reads the cursor's line of the current buffer.
        self.scope_amount = unsafe { get_indent() }
            + if iscase {
                cur_buf().b_ind_case_code
            } else {
                cur_buf().b_ind_scopedecl_code
            };
        self.lookfor = if cur_buf().b_ind_case_break != 0 {
            LOOKFOR_NOBREAK
        } else {
            LOOKFOR_ANY
        };
        Step::Again
    }

    /// The line is after a `while (cond);` -- ignore everything until the
    /// matching `do`.
    ///
    /// # Safety
    /// Reads the cursor; may unlock the current line.
    unsafe fn on_while_of_do_end(&mut self) -> Step {
        // An unterminated line after a `while ();` lines up with the last
        // one:
        //      while (cond);
        //      100 +               <- line up with this one
        // ->           here;
        if self.lookfor == LOOKFOR_UNTERM || self.lookfor == LOOKFOR_ENUM_OR_INIT {
            self.continuation();
            return Step::Done;
        }
        if self.whilelevel == 0 {
            self.lookfor = LOOKFOR_TERM;
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = unsafe { get_indent() };
            // SAFETY: `self.line` keeps its copy of the text alive.
            if unsafe { self.line.starts_with(b'{') } {
                self.amount += cur_buf().b_ind_open_extra;
            }
        }
        self.whilelevel += 1;
        Step::Again
    }

    /// The line is a terminated "normal" statement.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_terminated(&mut self) -> Step {
        // Skip a lone `break` before a switch label: it may be lined up
        // with the label ('cinoptions' `b`).
        // SAFETY: the cursor's line is NUL-terminated; `skipwhite` stays in it.
        let isbreak = self.lookfor == LOOKFOR_NOBREAK
            && unsafe { cin_isbreak(skipwhite(get_cursor_line_ptr())) };
        if isbreak {
            self.lookfor = LOOKFOR_ANY;
            return Step::Again;
        }

        // Handle a "do {" line.
        // SAFETY: the same, with `cin_skipcomment`, which also stays inside.
        let isdo =
            self.whilelevel > 0 && unsafe { cin_isdo(cin_skipcomment(get_cursor_line_ptr())) };
        if isdo {
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = unsafe { get_indent() };
            self.whilelevel -= 1;
            return Step::Again;
        }

        // A terminated line above an unterminated one: add the amount for
        // a continuation line.
        //   x = 1;
        //   y = foo +
        // ->       here;
        if self.lookfor == LOOKFOR_UNTERM || self.lookfor == LOOKFOR_ENUM_OR_INIT {
            self.continuation();
            return Step::Done;
        }

        // A terminated line above a terminated one, or an "if" line: use
        // the amount of the line below us.
        //   x = 1;                         x = 1;
        //   if (asdf)                  y = 2;
        //       while (asdf)         ->here;
        //          here;
        // ->foo;
        if self.lookfor == LOOKFOR_TERM {
            if !self.lookfor_break && self.whilelevel == 0 {
                return Step::Done;
            }
            return Step::Again;
        }

        // The first line above the one being indented is terminated.  To
        // know what to do, look further back for another terminated line.
        // SAFETY: the caller's promise, handed straight on.
        unsafe { self.walk_back_over_terminated() }
    }

    /// Upstream's `term_again`: step from a terminated line onto whatever
    /// encloses it, repeating while that is another block's end.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn walk_back_over_terminated(&mut self) -> Step {
        loop {
            // Position on the rightmost paren so that matching it takes
            // us to the start of the line.  Helps for:
            //     func(asdr,
            //              asdfasdf);
            //     here;
            // SAFETY: the cursor is on a line of the current buffer.
            let mut l = unsafe { get_cursor_line_ptr() }.cast_const();
            // SAFETY: `l` is that line; both move the cursor inside the buffer,
            // and the match runs only when a paren was found, as upstream has it.
            if unsafe { find_last_paren(l, b'(', b')') } {
                let trypos = unsafe { find_match_paren(cur_buf().b_ind_maxparen) };
                if let Some(trypos) = trypos {
                    // Check whether we are on a case label now; that is
                    // handled above.
                    //         case xx:  if ( asdf &&
                    //                          asdf)
                    cur_win().w_cursor = trypos;
                    // SAFETY: the cursor is on a line of the current buffer.
                    l = unsafe { get_cursor_line_ptr() };
                    // SAFETY: `l` is that line, NUL-terminated.
                    if unsafe { cin_iscase(l, false) || cin_isscopedecl(l) } {
                        // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                        self.resume_at(cur_win().w_cursor.lnum);
                        return Step::Again;
                    }
                }
            }

            // When aligning with the case statement, do not align with a
            // statement after it.
            //  case 1: {   <-- do not use this { position
            //        stat;
            //  }
            //  case 2:
            //        stat;
            // }
            // SAFETY: `l` is a NUL-terminated line.
            let iscase = cur_buf().b_ind_keep_case_label != 0 && unsafe { cin_iscase(l, false) };

            // The indent of the current line, ignoring any jump label.
            // SAFETY: the line number is the cursor's own and `l` is that line.
            self.amount = unsafe { skip_label(cur_win().w_cursor.lnum, &mut l) };
            // SAFETY: `self.line` keeps its copy of the text alive.
            if unsafe { self.line.starts_with(b'{') } {
                self.amount += cur_buf().b_ind_open_extra;
            }
            // See the remark above: only add `b_ind_open_extra` when the
            // line does not itself start with a '{'.
            // SAFETY: `l` is NUL-terminated, so `skipwhite` stays in it and the
            // dereference behind it is in bounds.
            l = unsafe { skipwhite(l) };
            if unsafe { *l } as u8 == b'{' {
                self.amount -= cur_buf().b_ind_open_extra;
            }
            self.lookfor = if iscase { LOOKFOR_ANY } else { LOOKFOR_TERM };

            // A terminated line starting with "else" needs the scope of
            // *that* else, so skip to the matching "if".  With
            // `whilelevel != 0` keep looking for a "do {" instead.
            // SAFETY: `l` is NUL-terminated; the chain keeps upstream's order.
            let is_else = self.lookfor == LOOKFOR_TERM
                && unsafe { *l as u8 != b'}' && cin_iselse(l) }
                && self.whilelevel == 0;
            if is_else {
                // SAFETY: both move the cursor inside the current buffer.
                let unmatched = unsafe {
                    find_start_brace().is_none_or(|pos| !find_match(LOOKFOR_IF, pos.lnum))
                };
                if unmatched {
                    return Step::Done;
                }
                return Step::Again;
            }

            // At the end of a block: skip to the start of that block.
            // SAFETY: the cursor is on a line of the current buffer.
            l = unsafe { get_cursor_line_ptr() };
            // SAFETY: `l` is that line; both move the cursor inside the
            // buffer, and the hunt runs only when a brace was found.
            if unsafe { find_last_paren(l, b'{', b'}') }
                && let Some(trypos) = unsafe { find_start_brace() }
            {
                cur_win().w_cursor = trypos;
                // If not "else {", check for terminated again; but
                // skip the block for "} else {".
                // SAFETY: the cursor's line is NUL-terminated.
                l = unsafe { cin_skipcomment(get_cursor_line_ptr()) };
                // SAFETY: `l` points into that line.
                if unsafe { *l as u8 == b'}' || !cin_iselse(l) } {
                    continue; // term_again
                }
                // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                self.resume_at(cur_win().w_cursor.lnum);
            }
            return Step::Again;
        }
    }

    /// The line is *not* terminated (or ends in a `,` that does not count).
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_unterminated(&mut self, mut l: *const c_char, terminated: u8) -> Step {
        // `l` holds code -- `cin_nocode` was false -- so `strlen` is at
        // least 1 and upstream's `l[strlen(l) - 1]` is in bounds.
        // SAFETY: `l` is NUL-terminated, so `strlen`'s index is in bounds.
        let last = unsafe { strlen(l).checked_sub(1).map_or(0, |i| *l.add(i) as u8) };
        // SAFETY: `l` is NUL-terminated, so `skipwhite` stays in it.
        let opens_bracket = self.lookfor != LOOKFOR_ENUM_OR_INIT
            && (unsafe { *skipwhite(l) } as u8 == b'[' || last == b'[');
        if opens_bracket {
            self.amount += self.ind_continuation;
        }

        // In the middle of a paren thing: go back to the line that starts
        // it, to get the right prevailing indent --
        //     if ( foo &&
        //              bar )
        // Position on the rightmost paren so that matching it takes us to
        // the start of the line, and ignore a match before the block.
        // SAFETY: `l` is the cursor's line; it moves the cursor in the buffer.
        unsafe { find_last_paren(l, b'(', b')') };
        // SAFETY: the same; the limit is derived from `self.line`'s position.
        let mut trypos = unsafe { find_match_paren(corr_ind_maxparen(&self.line.cur_curpos)) };
        if let Some(pos) = trypos
            && (pos.lnum < self.brace.lnum
                || (pos.lnum == self.brace.lnum && pos.col < self.brace.col))
        {
            trypos = None;
        }
        // SAFETY: the cursor is on a line of the current buffer.
        l = unsafe { get_cursor_line_ptr() };

        // Looking for a ',' means matching braces count too.
        if trypos.is_none() && terminated == b',' {
            // SAFETY: `l` is that line; both move the cursor inside the
            // buffer, and the hunt runs only when a brace was found.
            if unsafe { find_last_paren(l, b'{', b'}') } {
                trypos = unsafe { find_start_brace() };
            }
            // SAFETY: the cursor is on a line of the current buffer.
            l = unsafe { get_cursor_line_ptr() };
        }

        if let Some(trypos) = trypos {
            // Check whether we are on a case label now; that is handled
            // above.
            //     case xx:  if ( asdf &&
            //                        asdf)
            cur_win().w_cursor = trypos;
            // SAFETY: the cursor is on a line of the current buffer.
            l = unsafe { get_cursor_line_ptr() };
            // SAFETY: `l` is that line, NUL-terminated.
            if unsafe { cin_iscase(l, false) || cin_isscopedecl(l) } {
                // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                self.resume_at(cur_win().w_cursor.lnum);
                return Step::Again;
            }
        }

        // Skip over continuation lines to find the one to take the indent
        // from --
        //     char *usethis = "bla\
        //               bla",
        //          here;
        if terminated == b',' {
            while cur_win().w_cursor.lnum > 1 {
                // SAFETY: `lnum - 1` is at least 1, so it is a line of the buffer.
                let above = unsafe { ml_get(cur_win().w_cursor.lnum - 1) };
                // SAFETY: `ml_get` hands back a NUL-terminated line.
                if !unsafe { cin_ends_in_backslash(above) } {
                    break;
                }
                cur_win().w_cursor.lnum -= 1;
                cur_win().w_cursor.col = 0;
            }
            // SAFETY: the cursor is on a line of the current buffer.
            l = unsafe { get_cursor_line_ptr() };
        }

        // The indent and the text of the current line, ignoring any jump
        // label.
        // SAFETY: the line number is the cursor's own and `l` is that line.
        self.cur_amount = if cur_buf().b_ind_js != 0 {
            unsafe { get_indent() }
        } else {
            unsafe { skip_label(cur_win().w_cursor.lnum, &mut l) }
        };

        // Just above the line being indented and it starts with a '{':
        // line up with this line.
        //          while (not)
        // ->       {
        //          }
        // SAFETY: `self.line` keeps its copy of the text alive.
        if terminated != b','
            && self.lookfor != LOOKFOR_TERM
            && unsafe { self.line.starts_with(b'{') }
        {
            self.amount = self.cur_amount;
            // Only add `b_ind_open_extra` when the line does not itself
            // start with a '{', which must have a match on the same line
            // (the same scope).  Probably:
            //        { 1, 2 },
            // ->     { 3, 4 }
            // SAFETY: `l` is NUL-terminated, so `skipwhite` stays in it.
            if unsafe { *skipwhite(l) } as u8 != b'{' {
                self.amount += cur_buf().b_ind_open_extra;
            }
            if cur_buf().b_ind_cpp_baseclass != 0 && cur_buf().b_ind_js == 0 {
                // Have to look back for a cpp base-class declaration or
                // initialisation.
                self.lookfor = LOOKFOR_CPP_BASECLASS;
                return Step::Again;
            }
            return Step::Done;
        }

        // After an "if", "while", etc.  Also allow "   } else".
        // SAFETY: `l` is NUL-terminated; the promise is handed on to both arms.
        if unsafe { cin_is_cinword(l) || cin_iselse(skipwhite(l)) } {
            unsafe { self.after_cinword() }
        } else {
            unsafe { self.after_plain_unterminated(terminated) }
        }
    }

    /// The unterminated line above is an `if`/`while`/`for`/`else`.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn after_cinword(&mut self) -> Step {
        // An unterminated line after an `if ()` lines up with the last
        // one:
        //   if (cond)
        //             100 +
        // ->              here;
        if self.lookfor == LOOKFOR_UNTERM || self.lookfor == LOOKFOR_ENUM_OR_INIT {
            self.continuation();
            return Step::Done;
        }

        // Just above the line being indented: finished.
        //            while (not)
        // ->             here;
        // Otherwise this indent is usable once the line before it is
        // terminated:
        //        yyy;
        //        if (stat)
        //            while (not)
        //                xxx;
        // ->     here;
        self.amount = self.cur_amount;
        // SAFETY: `self.line` keeps its copy of the text alive.
        if unsafe { self.line.starts_with(b'{') } {
            self.amount += cur_buf().b_ind_open_extra;
        }
        if self.lookfor != LOOKFOR_TERM {
            self.amount += cur_buf().b_ind_level + cur_buf().b_ind_no_brace;
            return Step::Done;
        }

        // Expecting the `while ()` after a `do`: line up with the
        // `while()`.
        //     do
        //            x = 1;
        // ->  here
        // SAFETY: the cursor's line is NUL-terminated.
        let l = unsafe { skipwhite(get_cursor_line_ptr()) }.cast_const();
        // SAFETY: `l` points into that line.
        if unsafe { cin_isdo(l) } {
            if self.whilelevel == 0 {
                return Step::Done;
            }
            self.whilelevel -= 1;
        }

        // Searching for a terminated line: do not use the one between the
        // "if" and the matching "else"; use the scope of *this* "else".
        // With `whilelevel != 0` keep looking for a "do {".
        // SAFETY: `l` points into the cursor's NUL-terminated line.
        if unsafe { cin_iselse(l) } && self.whilelevel == 0 {
            // For "} else", find the opening brace of the enclosing
            // scope, not the one from "if () {".
            // SAFETY: the read is in bounds, and `l` came out of `skipwhite` on
            // the very line `get_cursor_line_ptr` hands back: one allocation.
            if unsafe { *l } as u8 == b'}' {
                cur_win().w_cursor.col =
                    unsafe { l.offset_from(get_cursor_line_ptr()) } as colnr_T + 1;
            }
            // SAFETY: both move the cursor inside the current buffer.
            let unmatched =
                unsafe { find_start_brace().is_none_or(|pos| !find_match(LOOKFOR_IF, pos.lnum)) };
            if unmatched {
                return Step::Done;
            }
        }
        Step::Again
    }

    /// The unterminated line above is an ordinary statement.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn after_plain_unterminated(&mut self, terminated: u8) -> Step {
        // Two unterminated lines in a row: line up with the last one.
        //   c = 99 +
        //            100 +
        // ->         here;
        if self.lookfor == LOOKFOR_UNTERM {
            // A line ending in a comma gets extra indent.
            if terminated == b',' {
                self.amount += self.ind_continuation;
            }
            return Step::Done;
        }

        if self.lookfor == LOOKFOR_ENUM_OR_INIT {
            // Two lines ending in ',': line up with the lowest -- but
            // check for a cpp base-class declaration first, if this is an
            // opening brace or we are only looking for enums.
            if terminated == b',' {
                if cur_buf().b_ind_cpp_baseclass == 0 {
                    return Step::Done;
                }
                self.lookfor = LOOKFOR_CPP_BASECLASS;
                return Step::Again;
            }
            // Ignore unterminated lines in between, but reduce indent.
            self.amount = self.amount.min(self.cur_amount);
            return Step::Again;
        }

        // The first unterminated line in a row: this line may be what to
        // line up with, so remember its indent.
        //          100 +
        // ->       here;
        // SAFETY: the cursor is on a line of the current buffer.
        let l = unsafe { get_cursor_line_ptr() }.cast_const();
        self.amount = self.cur_amount;

        // SAFETY: `l` is that line, NUL-terminated.
        let n = unsafe { strlen(l) };
        // SAFETY: `l` is NUL-terminated; the `n >= 2` test in front of
        // `l.add(n - 2)` is its bounds proof, so the chain stays whole.
        let ends_in_bracket = cur_buf().b_ind_js != 0
            && terminated == b','
            && unsafe { *skipwhite(l) as u8 == b']' || (n >= 2 && *l.add(n - 2) as u8 == b']') };
        if ends_in_bracket {
            return Step::Done;
        }

        // If the previous line ends in ',', decide whether we are in an
        // initialisation or an enum --
        //     struct xxx =
        //     {
        //          sizeof a,
        //          124 };
        // -- or in an ordinary continuation line.  Only when no other
        // statement has been found yet.
        if self.lookfor == LOOKFOR_INITIAL && terminated == b',' {
            if cur_buf().b_ind_js == 0 {
                self.lookfor = LOOKFOR_ENUM_OR_INIT;
                // SAFETY: reads the cursor's line of the current buffer.
                self.cont_amount = unsafe { cin_first_id_amount() };
                return Step::Again;
            }
            // Javascript: search for a line ending in a comma and line up
            // with the line below it (which may be this one).
            //     some = [
            //         1,     <- line up here
            //         2,
            //     some = [
            //         3 +    <- line up here
            //           4 *
            //            5,
            //         6,
            // SAFETY: `l` is NUL-terminated, so `skipwhite` stays in it.
            if unsafe { cin_iscomment(skipwhite(l)) } {
                return Step::Done;
            }
            self.lookfor = LOOKFOR_COMMA;
            // SAFETY: moves the cursor inside the current buffer.
            if let Some(trypos) = unsafe { find_match_char(b'[', cur_buf().b_ind_maxparen) } {
                if trypos.lnum == cur_win().w_cursor.lnum - 1 {
                    // The current line is the first inside [], so line up
                    // with it.
                    return Step::Done;
                }
                self.ourscope = trypos.lnum;
            }
            return Step::Again;
        }

        // SAFETY: `l` is a NUL-terminated line.
        if self.lookfor == LOOKFOR_INITIAL && unsafe { cin_ends_in_backslash(l) } {
            // SAFETY: the line number is the cursor's own.
            self.cont_amount = unsafe { cin_get_equal_amount(cur_win().w_cursor.lnum) };
        }
        if self.lookfor != LOOKFOR_TERM
            && self.lookfor != LOOKFOR_JS_KEY
            && self.lookfor != LOOKFOR_COMMA
            && self.raw_string_start != cur_win().w_cursor.lnum
        {
            self.lookfor = LOOKFOR_UNTERM;
        }
        Step::Again
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
