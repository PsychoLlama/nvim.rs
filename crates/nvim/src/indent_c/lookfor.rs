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
#![allow(unsafe_code)]

use super::*;
use crate::winlayer::{Buf, Win};

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
    fn resume_at(&self, lnum: LineNr) {
        Win::current().w_cursor.lnum = lnum + 1;
        Win::current().w_cursor.col = 0;
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
            if self.line.starts_with(b'{') {
                self.amount += Buf::current().b_ind_open_extra;
                self.added_to_amount = Buf::current().b_ind_open_extra;
            }
        }

        if self.lookfor_cpp_namespace {
            let lnum = Win::current().w_cursor.lnum;
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

            if preproc_start(&mut Win::current().w_cursor.lnum, &mut self.amount) {
                return Step::Again;
            }

            // Finally, the actual check for "namespace".
            let cursor_lnum = Win::current().w_cursor.lnum;
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            // SAFETY: both read the current buffer's 'iskeyword'.
            if unsafe { opens_namespace(l, 0) } {
                self.amount += Buf::current().b_ind_cpp_namespace - self.added_to_amount;
                return Step::Done;
            }
            // SAFETY: the same.
            if unsafe { opens_extern_c(l, 0) } {
                self.amount += Buf::current().b_ind_cpp_extern_c - self.added_to_amount;
                return Step::Done;
            }
            if only_comment_left(l, 0) {
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
        let lnum = Win::current().w_cursor.lnum;
        if lnum == 0 || lnum < self.ourscope - Buf::current().b_ind_maxparen {
            // Nothing found (abusing `b_ind_maxparen` as the limit):
            // assume a terminated line, i.e. a variable initialisation.
            if self.cont_amount > 0 {
                self.amount = self.cont_amount;
            } else if Buf::current().b_ind_js == 0 {
                self.amount += self.ind_continuation;
            }
            return Step::Done;
        }

        // SAFETY: on the main thread, with a current buffer.
        if let Some(trypos) = unsafe { ind_find_start_comment_or_raw_string(None) } {
            self.resume_at(trypos.lnum);
            return Step::Again;
        }

        // The chain stays whole: `preproc_start` may move the cursor's line
        // number on before the text is read.
        let skipped = preproc_start(&mut Win::current().w_cursor.lnum, &mut self.amount) || {
            let cursor_lnum = Win::current().w_cursor.lnum;
            only_comment_left(Lines::current().line(cursor_lnum), 0)
        };
        if skipped {
            return Step::Again;
        }

        let cursor_lnum = Win::current().w_cursor.lnum;
        let terminated = terminator(Lines::current().line(cursor_lnum), 0, false, true);

        // At top level and looking like a function declaration: done, it
        // is a variable declaration.
        // SAFETY: the search moves the cursor over lines of the buffer.
        let is_var_decl = self.start_brace != BRACE_IN_COL0
            || !unsafe { is_func_decl(Win::current().w_cursor.lnum, 0) };
        if is_var_decl {
            // Terminated with another ',': a continued initialisation, so
            // no extra indent.
            // TODO(vim): does not work if a function declaration is split
            // over several lines -- `is_func_decl` says no then.
            if terminated == b',' {
                return Step::Done;
            }
            // An enum declaration or an assignment: done.
            // SAFETY: reads the cursor's line of the current buffer.
            if terminated != b';' && unsafe { is_enum_or_init() } {
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
            let cursor_lnum = Win::current().w_cursor.lnum;
            // The borrow ends with the statement; the match runs only when a
            // paren was found, as upstream has it.
            if find_last_paren(Lines::current().line(cursor_lnum), b'(', b')') {
                // SAFETY: moves the cursor inside the current buffer.
                trypos = unsafe { find_match_paren(Buf::current().b_ind_maxparen) };
            }
            let cursor_lnum = Win::current().w_cursor.lnum;
            if trypos.is_none() && find_last_paren(Lines::current().line(cursor_lnum), b'{', b'}') {
                // SAFETY: the same, for the brace pair.
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

        // A switch() label or a C++ scope declaration may be what we line
        // up relative to.
        let cursor_lnum = Win::current().w_cursor.lnum;
        let (iscase, isscopedecl) = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            // SAFETY: `is_scope_decl` reads the buffer's 'cinscopedecls'.
            (is_case_label(l, 0, false), unsafe { is_scope_decl(l, 0) })
        };
        if iscase || isscopedecl {
            // SAFETY: the caller's promise, handed straight on.
            return unsafe { self.on_label(iscase) };
        }

        // Looking for a switch() label or scope declaration: ignore other
        // lines and skip `{}` blocks whole.
        if self.lookfor == LOOKFOR_CASE || self.lookfor == LOOKFOR_SCOPEDECL {
            // The borrow ends with the statement; the hunt runs only when a
            // brace was found.
            let has_brace = find_last_paren(Lines::current().line(cursor_lnum), b'{', b'}');
            // SAFETY: moves the cursor inside the current buffer.
            if has_brace && let Some(trypos) = unsafe { find_start_brace() } {
                self.resume_at(trypos.lnum);
            }
            return Step::Again;
        }

        // Ignore jump labels with nothing after them.
        // SAFETY: reads the cursor's line of the current buffer.
        if Buf::current().b_ind_js == 0 && unsafe { is_jump_label() } {
            // `is_jump_label` may have unlocked the line, so it is read again.
            let cursor_lnum = Win::current().w_cursor.lnum;
            let mut lines = Lines::current();
            let text = lines.line(cursor_lnum);
            let nothing_after =
                after_label(text).is_none_or(|after| only_comment_left(text, after));
            if nothing_after {
                return Step::Again;
            }
        }

        // Ignore #defines, comments and empty lines.  The chain stays whole:
        // `preproc_start` may move the cursor's line number on before the
        // text is read.
        let skipped = preproc_start(&mut Win::current().w_cursor.lnum, &mut self.amount) || {
            let cursor_lnum = Win::current().w_cursor.lnum;
            only_comment_left(Lines::current().line(cursor_lnum), 0)
        };
        if skipped {
            return Step::Again;
        }

        // The start of a C++ base-class declaration or constructor
        // initialisation?
        let mut is_baseclass = false;
        if self.lookfor != LOOKFOR_TERM && Buf::current().b_ind_cpp_baseclass > 0 {
            // SAFETY: on the main thread with a current buffer.
            is_baseclass = unsafe { in_baseclass_list(&mut self.cache) };
        }
        if is_baseclass {
            if self.lookfor == LOOKFOR_UNTERM {
                self.continuation();
            } else if self.line.starts_with(b'{') {
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
            // declaration or initialisation before the opening brace.  The
            // check above may have unlocked the line, so it is read again.
            let cursor_lnum = Win::current().w_cursor.lnum;
            return if terminator(Lines::current().line(cursor_lnum), 0, true, false) != 0 {
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
        let cursor_lnum = Win::current().w_cursor.lnum;
        let terminated = terminator(Lines::current().line(cursor_lnum), 0, false, true);

        if self.js_cur_has_key {
            self.js_cur_has_key = false; // only check the first line
            if Buf::current().b_ind_js != 0 && terminated == b',' {
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
        if self.lookfor == LOOKFOR_JS_KEY && has_js_key(Lines::current().line(cursor_lnum), 0) {
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = get_indent();
            return Step::Done;
        }
        if self.lookfor == LOOKFOR_COMMA {
            if self.brace.lnum >= Win::current().w_cursor.lnum {
                return Step::Done;
            }
            if terminated == b',' {
                // The line below is the one that starts a (possibly
                // broken) line ending in a comma.
                return Step::Done;
            }
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = get_indent();
            if Win::current().w_cursor.lnum - 1 == self.ourscope {
                // The line above starts the scope, so this line is the
                // one that starts the comma-terminated line.
                return Step::Done;
            }
        }

        // SAFETY: the caller's promise, handed on.
        if terminated == 0 || (self.lookfor != LOOKFOR_UNTERM && terminated == b',') {
            unsafe { self.on_unterminated(terminated) }
        } else if unsafe { ends_a_do_while(terminated) } {
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
                self.amount = get_indent();
                return Step::Done;
            }
            return Step::Again;
        }

        // SAFETY: the line number is the cursor's own.
        let n = unsafe { get_indent_nolabel(Win::current().w_cursor.lnum) };

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
            let cursor_lnum = Win::current().w_cursor.lnum;
            let mut lines = Lines::current();
            let text = lines.line(cursor_lnum);
            // SAFETY: `starts_with_cinword` reads the buffer's 'cinwords'.
            let cinword = after_label(text)
                .is_some_and(|after| unsafe { starts_with_cinword(&text[after..]) });
            if cinword {
                self.amount += if self.line.starts_with(b'{') {
                    Buf::current().b_ind_open_extra
                } else {
                    Buf::current().b_ind_level + Buf::current().b_ind_no_brace
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
        self.scope_amount = get_indent()
            + if iscase {
                Buf::current().b_ind_case_code
            } else {
                Buf::current().b_ind_scopedecl_code
            };
        self.lookfor = if Buf::current().b_ind_case_break != 0 {
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
            self.amount = get_indent();
            if self.line.starts_with(b'{') {
                self.amount += Buf::current().b_ind_open_extra;
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
        let cursor_lnum = Win::current().w_cursor.lnum;
        let (isbreak, isdo) = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            (
                // Skip a lone `break` before a switch label: it may be lined
                // up with the label ('cinoptions' `b`).
                self.lookfor == LOOKFOR_NOBREAK && is_break(l, skip::white(l)),
                // Handle a "do {" line.
                self.whilelevel > 0 && is_do(l, code_at(l, 0)),
            )
        };
        if isbreak {
            self.lookfor = LOOKFOR_ANY;
            return Step::Again;
        }
        if isdo {
            // SAFETY: reads the cursor's line of the current buffer.
            self.amount = get_indent();
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
            // The borrow ends with the statement; the match runs only when a
            // paren was found, as upstream has it.
            let cursor_lnum = Win::current().w_cursor.lnum;
            if find_last_paren(Lines::current().line(cursor_lnum), b'(', b')') {
                // SAFETY: moves the cursor inside the current buffer.
                let trypos = unsafe { find_match_paren(Buf::current().b_ind_maxparen) };
                if let Some(trypos) = trypos {
                    // Check whether we are on a case label now; that is
                    // handled above.
                    //         case xx:  if ( asdf &&
                    //                          asdf)
                    Win::current().w_cursor = trypos;
                    let labelled = {
                        let mut lines = Lines::current();
                        let l = lines.line(trypos.lnum);
                        // SAFETY: reads the buffer's 'cinscopedecls'.
                        is_case_label(l, 0, false) || unsafe { is_scope_decl(l, 0) }
                    };
                    if labelled {
                        // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                        self.resume_at(Win::current().w_cursor.lnum);
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
            let cursor_lnum = Win::current().w_cursor.lnum;
            let iscase = Buf::current().b_ind_keep_case_label != 0
                && is_case_label(Lines::current().line(cursor_lnum), 0, false);

            // The indent of the current line, ignoring any jump label.
            // SAFETY: the line number is the cursor's own.
            let (amount, at) = unsafe { skip_label(cursor_lnum) };
            self.amount = amount;
            if self.line.starts_with(b'{') {
                self.amount += Buf::current().b_ind_open_extra;
            }
            // See the remark above: only add `b_ind_open_extra` when the
            // line does not itself start with a '{'.
            let (opens_brace, is_else_line) = {
                let mut lines = Lines::current();
                let l = lines.line(cursor_lnum);
                let at = at + skip::white(&l[at.min(l.len())..]);
                (
                    byte_at(l, at) == b'{',
                    byte_at(l, at) != b'}' && is_else(l, at),
                )
            };
            if opens_brace {
                self.amount -= Buf::current().b_ind_open_extra;
            }
            self.lookfor = if iscase { LOOKFOR_ANY } else { LOOKFOR_TERM };

            // A terminated line starting with "else" needs the scope of
            // *that* else, so skip to the matching "if".  With
            // `whilelevel != 0` keep looking for a "do {" instead.
            if self.lookfor == LOOKFOR_TERM && is_else_line && self.whilelevel == 0 {
                // SAFETY: both move the cursor inside the current buffer.
                let unmatched = unsafe {
                    find_start_brace().is_none_or(|pos| !find_match(LOOKFOR_IF, pos.lnum))
                };
                if unmatched {
                    return Step::Done;
                }
                return Step::Again;
            }

            // At the end of a block: skip to the start of that block.  The
            // borrow ends with the statement; the hunt runs only when a
            // brace was found.
            let cursor_lnum = Win::current().w_cursor.lnum;
            let has_brace = find_last_paren(Lines::current().line(cursor_lnum), b'{', b'}');
            // SAFETY: moves the cursor inside the current buffer.
            if has_brace && let Some(trypos) = unsafe { find_start_brace() } {
                Win::current().w_cursor = trypos;
                // If not "else {", check for terminated again; but
                // skip the block for "} else {".
                let term_again = {
                    let mut lines = Lines::current();
                    let l = lines.line(trypos.lnum);
                    let at = code_at(l, 0);
                    byte_at(l, at) == b'}' || !is_else(l, at)
                };
                if term_again {
                    continue;
                }
                // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                self.resume_at(Win::current().w_cursor.lnum);
            }
            return Step::Again;
        }
    }

    /// The line is *not* terminated (or ends in a `,` that does not count).
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_unterminated(&mut self, terminated: u8) -> Step {
        let cursor_lnum = Win::current().w_cursor.lnum;
        let opens_bracket = self.lookfor != LOOKFOR_ENUM_OR_INIT && {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            // The line holds code -- `only_comment_left` was false -- so
            // upstream's `l[strlen(l) - 1]` is in bounds.
            byte_at(l, skip::white(l)) == b'[' || l.last() == Some(&b'[')
        };
        if opens_bracket {
            self.amount += self.ind_continuation;
        }

        // In the middle of a paren thing: go back to the line that starts
        // it, to get the right prevailing indent --
        //     if ( foo &&
        //              bar )
        // Position on the rightmost paren so that matching it takes us to
        // the start of the line, and ignore a match before the block.  The
        // borrow ends with the statement.
        find_last_paren(Lines::current().line(cursor_lnum), b'(', b')');
        // SAFETY: moves the cursor in the buffer; the limit is derived from
        // `self.line`'s position.
        let mut trypos = unsafe { find_match_paren(corr_ind_maxparen(&self.line.cur_curpos)) };
        if let Some(pos) = trypos
            && (pos.lnum < self.brace.lnum
                || (pos.lnum == self.brace.lnum && pos.col < self.brace.col))
        {
            trypos = None;
        }

        // Looking for a ',' means matching braces count too.
        if trypos.is_none() && terminated == b',' {
            let cursor_lnum = Win::current().w_cursor.lnum;
            // The hunt runs only when a brace was found.
            if find_last_paren(Lines::current().line(cursor_lnum), b'{', b'}') {
                // SAFETY: moves the cursor inside the current buffer.
                trypos = unsafe { find_start_brace() };
            }
        }

        if let Some(trypos) = trypos {
            // Check whether we are on a case label now; that is handled
            // above.
            //     case xx:  if ( asdf &&
            //                        asdf)
            Win::current().w_cursor = trypos;
            let labelled = {
                let mut lines = Lines::current();
                let l = lines.line(trypos.lnum);
                // SAFETY: reads the buffer's 'cinscopedecls'.
                is_case_label(l, 0, false) || unsafe { is_scope_decl(l, 0) }
            };
            if labelled {
                // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                self.resume_at(Win::current().w_cursor.lnum);
                return Step::Again;
            }
        }

        // Skip over continuation lines to find the one to take the indent
        // from --
        //     char *usethis = "bla\
        //               bla",
        //          here;
        if terminated == b',' {
            while Win::current().w_cursor.lnum > 1 {
                // `lnum - 1` is at least 1, so it is a line of the buffer.
                let above = Win::current().w_cursor.lnum - 1;
                if !ends_in_backslash(Lines::current().line(above)) {
                    break;
                }
                Win::current().w_cursor.lnum -= 1;
                Win::current().w_cursor.col = 0;
            }
        }

        // The indent and the offset of the current line's text, ignoring
        // any jump label.
        let cursor_lnum = Win::current().w_cursor.lnum;
        let at = if Buf::current().b_ind_js != 0 {
            self.cur_amount = get_indent();
            0
        } else {
            // SAFETY: the line number is the cursor's own.
            let (amount, label_end) = unsafe { skip_label(cursor_lnum) };
            self.cur_amount = amount;
            label_end
        };

        // Just above the line being indented and it starts with a '{':
        // line up with this line.
        //          while (not)
        // ->       {
        //          }
        if terminated != b',' && self.lookfor != LOOKFOR_TERM && self.line.starts_with(b'{') {
            self.amount = self.cur_amount;
            // Only add `b_ind_open_extra` when the line does not itself
            // start with a '{', which must have a match on the same line
            // (the same scope).  Probably:
            //        { 1, 2 },
            // ->     { 3, 4 }
            let opens_brace = {
                let mut lines = Lines::current();
                let l = lines.line(cursor_lnum);
                byte_at(l, at + skip::white(&l[at.min(l.len())..])) == b'{'
            };
            if !opens_brace {
                self.amount += Buf::current().b_ind_open_extra;
            }
            if Buf::current().b_ind_cpp_baseclass != 0 && Buf::current().b_ind_js == 0 {
                // Have to look back for a cpp base-class declaration or
                // initialisation.
                self.lookfor = LOOKFOR_CPP_BASECLASS;
                return Step::Again;
            }
            return Step::Done;
        }

        // After an "if", "while", etc.  Also allow "   } else".
        let cinword = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            // SAFETY: `starts_with_cinword` reads the buffer's 'cinwords'.
            (unsafe { starts_with_cinword(&l[at.min(l.len())..]) })
                || is_else(l, at + skip::white(&l[at.min(l.len())..]))
        };
        // SAFETY: the caller's promise, handed on to both arms.
        if cinword {
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
        if self.line.starts_with(b'{') {
            self.amount += Buf::current().b_ind_open_extra;
        }
        if self.lookfor != LOOKFOR_TERM {
            self.amount += Buf::current().b_ind_level + Buf::current().b_ind_no_brace;
            return Step::Done;
        }

        // Expecting the `while ()` after a `do`: line up with the
        // `while()`.
        //     do
        //            x = 1;
        // ->  here
        let cursor_lnum = Win::current().w_cursor.lnum;
        let (isdo, is_else_line, at) = {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            let at = skip::white(l);
            (is_do(l, at), is_else(l, at) && byte_at(l, at) == b'}', at)
        };
        if isdo {
            if self.whilelevel == 0 {
                return Step::Done;
            }
            self.whilelevel -= 1;
        }

        // Searching for a terminated line: do not use the one between the
        // "if" and the matching "else"; use the scope of *this* "else".
        // With `whilelevel != 0` keep looking for a "do {".
        let iselse = {
            let mut lines = Lines::current();
            is_else(lines.line(cursor_lnum), at)
        };
        if iselse && self.whilelevel == 0 {
            // For "} else", find the opening brace of the enclosing
            // scope, not the one from "if () {".
            if is_else_line {
                Win::current().w_cursor.col = at as ColNr + 1;
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
                if Buf::current().b_ind_cpp_baseclass == 0 {
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
        let cursor_lnum = Win::current().w_cursor.lnum;
        self.amount = self.cur_amount;

        let ends_in_bracket = Buf::current().b_ind_js != 0 && terminated == b',' && {
            let mut lines = Lines::current();
            let l = lines.line(cursor_lnum);
            // Upstream reads `l[strlen(l) - 2]`, the byte before the last.
            byte_at(l, skip::white(l)) == b']'
                || l.len().checked_sub(2).is_some_and(|i| l[i] == b']')
        };
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
            if Buf::current().b_ind_js == 0 {
                self.lookfor = LOOKFOR_ENUM_OR_INIT;
                // SAFETY: reads the cursor's line of the current buffer.
                self.cont_amount = unsafe { first_id_amount() };
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
            let commented = {
                let mut lines = Lines::current();
                let l = lines.line(cursor_lnum);
                starts_comment(l, skip::white(l))
            };
            if commented {
                return Step::Done;
            }
            self.lookfor = LOOKFOR_COMMA;
            // SAFETY: moves the cursor inside the current buffer.
            if let Some(trypos) = unsafe { find_match_char(b'[', Buf::current().b_ind_maxparen) } {
                if trypos.lnum == Win::current().w_cursor.lnum - 1 {
                    // The current line is the first inside [], so line up
                    // with it.
                    return Step::Done;
                }
                self.ourscope = trypos.lnum;
            }
            return Step::Again;
        }

        if self.lookfor == LOOKFOR_INITIAL && ends_in_backslash(Lines::current().line(cursor_lnum))
        {
            // SAFETY: the line number is the cursor's own.
            self.cont_amount = unsafe { equal_amount(Win::current().w_cursor.lnum) };
        }
        if self.lookfor != LOOKFOR_TERM
            && self.lookfor != LOOKFOR_JS_KEY
            && self.lookfor != LOOKFOR_COMMA
            && self.raw_string_start != Win::current().w_cursor.lnum
        {
            self.lookfor = LOOKFOR_UNTERM;
        }
        Step::Again
    }
}
