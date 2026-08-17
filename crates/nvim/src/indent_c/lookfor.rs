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
use ::core::ffi::c_char;

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
    ///
    /// # Safety
    /// Writes the cursor.
    unsafe fn resume_at(&self, lnum: linenr_T) {
        unsafe {
            (*curwin.get()).w_cursor.lnum = lnum + 1;
            (*curwin.get()).w_cursor.col = 0;
        }
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
        unsafe {
            if self.lookfor == LOOKFOR_ENUM_OR_INIT {
                return self.at_scope_start_enum_or_init();
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
                    self.amount += (*curbuf.get()).b_ind_open_extra;
                    self.added_to_amount = (*curbuf.get()).b_ind_open_extra;
                }
            }

            if self.lookfor_cpp_namespace {
                let lnum = (*curwin.get()).w_cursor.lnum;
                if lnum == self.ourscope {
                    return Step::Again;
                }
                if lnum == 0 || lnum < self.ourscope - FIND_NAMESPACE_LIM {
                    return Step::Done;
                }

                let trypos = ind_find_start_CORS(None);
                if !trypos.is_null() {
                    self.resume_at((*trypos).lnum);
                    return Step::Again;
                }

                let mut l = get_cursor_line_ptr().cast_const();
                if cin_ispreproc_cont(&mut l, &mut (*curwin.get()).w_cursor.lnum, &mut self.amount)
                {
                    return Step::Again;
                }

                // Finally, the actual check for "namespace".
                if cin_is_cpp_namespace(l) {
                    self.amount += (*curbuf.get()).b_ind_cpp_namespace - self.added_to_amount;
                    return Step::Done;
                }
                if cin_is_cpp_extern_c(l) {
                    self.amount += (*curbuf.get()).b_ind_cpp_extern_c - self.added_to_amount;
                    return Step::Done;
                }
                if cin_nocode(l) {
                    return Step::Again;
                }
            }
            Step::Done
        }
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
        unsafe {
            let lnum = (*curwin.get()).w_cursor.lnum;
            if lnum == 0 || lnum < self.ourscope - (*curbuf.get()).b_ind_maxparen {
                // Nothing found (abusing `b_ind_maxparen` as the limit):
                // assume a terminated line, i.e. a variable initialisation.
                if self.cont_amount > 0 {
                    self.amount = self.cont_amount;
                } else if (*curbuf.get()).b_ind_js == 0 {
                    self.amount += self.ind_continuation;
                }
                return Step::Done;
            }

            let trypos = ind_find_start_CORS(None);
            if !trypos.is_null() {
                self.resume_at((*trypos).lnum);
                return Step::Again;
            }

            let mut l = get_cursor_line_ptr().cast_const();
            if cin_ispreproc_cont(&mut l, &mut (*curwin.get()).w_cursor.lnum, &mut self.amount)
                || cin_nocode(l)
            {
                return Step::Again;
            }

            let terminated = cin_isterminated(l, false, true);

            // At top level and looking like a function declaration: done, it
            // is a variable declaration.
            if self.start_brace != BRACE_IN_COL0
                || !cin_isfuncdecl(Some(&mut l), (*curwin.get()).w_cursor.lnum, 0)
            {
                // Terminated with another ',': a continued initialisation, so
                // no extra indent.
                // TODO(vim): does not work if a function declaration is split
                // over several lines -- `cin_isfuncdecl` says no then.
                if terminated == b',' {
                    return Step::Done;
                }
                // An enum declaration or an assignment: done.
                if terminated != b';' && cin_isinit() {
                    return Step::Done;
                }
                if terminated == 0 || terminated == b'{' {
                    return Step::Again;
                }
            }

            if terminated != b';' {
                // Skip parens and braces: position on the rightmost paren so
                // that matching it takes us to the start of the line.
                let mut trypos = ::core::ptr::null_mut::<pos_T>();
                if find_last_paren(l, b'(', b')') {
                    trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                }
                if trypos.is_null() && find_last_paren(l, b'{', b'}') {
                    trypos = find_start_brace();
                }
                if !trypos.is_null() {
                    self.resume_at((*trypos).lnum);
                    return Step::Again;
                }
            }

            // A variable declaration, so add indentation:
            //     int a,
            //        b;
            self.continuation();
            Step::Done
        }
    }

    /// One line of the backwards scan, above `ourscope`.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    pub(crate) unsafe fn step(&mut self) -> Step {
        unsafe {
            // In a comment or raw string now: skip to the start of it.
            let trypos = ind_find_start_CORS(Some(&mut self.raw_string_start));
            if !trypos.is_null() {
                self.resume_at((*trypos).lnum);
                return Step::Again;
            }

            let mut l = get_cursor_line_ptr().cast_const();

            // A switch() label or a C++ scope declaration may be what we line
            // up relative to.
            let iscase = cin_iscase(l, false);
            if iscase || cin_isscopedecl(l) {
                return self.on_label(iscase);
            }

            // Looking for a switch() label or scope declaration: ignore other
            // lines and skip `{}` blocks whole.
            if self.lookfor == LOOKFOR_CASE || self.lookfor == LOOKFOR_SCOPEDECL {
                if find_last_paren(l, b'{', b'}') {
                    let trypos = find_start_brace();
                    if !trypos.is_null() {
                        self.resume_at((*trypos).lnum);
                    }
                }
                return Step::Again;
            }

            // Ignore jump labels with nothing after them.
            if (*curbuf.get()).b_ind_js == 0 && cin_islabel() {
                let after = after_label(get_cursor_line_ptr());
                if after.is_null() || cin_nocode(after) {
                    return Step::Again;
                }
            }

            // Ignore #defines, comments and empty lines.  (Get the line
            // again: `cin_islabel` may have unlocked it.)
            l = get_cursor_line_ptr();
            if cin_ispreproc_cont(&mut l, &mut (*curwin.get()).w_cursor.lnum, &mut self.amount)
                || cin_nocode(l)
            {
                return Step::Again;
            }

            // The start of a C++ base-class declaration or constructor
            // initialisation?
            let mut is_baseclass = false;
            if self.lookfor != LOOKFOR_TERM && (*curbuf.get()).b_ind_cpp_baseclass > 0 {
                is_baseclass = cin_is_cpp_baseclass(&mut self.cache);
                l = get_cursor_line_ptr();
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
                    self.amount = get_baseclass_amount(self.cache.lpos.col);
                }
                return Step::Done;
            }
            if self.lookfor == LOOKFOR_CPP_BASECLASS {
                // Only interested in whether there is a base-class
                // declaration or initialisation before the opening brace.
                return if cin_isterminated(l, true, false) != 0 {
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
            let terminated = cin_isterminated(l, false, true);

            if self.js_cur_has_key {
                self.js_cur_has_key = false; // only check the first line
                if (*curbuf.get()).b_ind_js != 0 && terminated == b',' {
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
            if self.lookfor == LOOKFOR_JS_KEY && cin_has_js_key(l) {
                self.amount = get_indent();
                return Step::Done;
            }
            if self.lookfor == LOOKFOR_COMMA {
                if self.brace.lnum >= (*curwin.get()).w_cursor.lnum {
                    return Step::Done;
                }
                if terminated == b',' {
                    // The line below is the one that starts a (possibly
                    // broken) line ending in a comma.
                    return Step::Done;
                }
                self.amount = get_indent();
                if (*curwin.get()).w_cursor.lnum - 1 == self.ourscope {
                    // The line above starts the scope, so this line is the
                    // one that starts the comma-terminated line.
                    return Step::Done;
                }
            }

            if terminated == 0 || (self.lookfor != LOOKFOR_UNTERM && terminated == b',') {
                self.on_unterminated(l, terminated)
            } else if cin_iswhileofdo_end(terminated) {
                self.on_while_of_do_end()
            } else {
                self.on_terminated()
            }
        }
    }

    /// The line is a `case`/`default` label or a scope declaration.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_label(&mut self, iscase: bool) -> Step {
        unsafe {
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
                let trypos = find_start_brace();
                if trypos.is_null() || (*trypos).lnum == self.ourscope {
                    self.amount = get_indent();
                    return Step::Done;
                }
                return Step::Again;
            }

            let n = get_indent_nolabel((*curwin.get()).w_cursor.lnum);

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
                let l = after_label(get_cursor_line_ptr());
                if !l.is_null() && cin_is_cinword(l) {
                    self.amount += if self.line.starts_with(b'{') {
                        (*curbuf.get()).b_ind_open_extra
                    } else {
                        (*curbuf.get()).b_ind_level + (*curbuf.get()).b_ind_no_brace
                    };
                }
                return Step::Done;
            }

            // Try to get the indent of a statement before the label.  If
            // nothing is found, line up relative to the label.
            //      break;              <- may line up with this line
            //   case xx:
            // ->   y = 1;
            self.scope_amount = get_indent()
                + if iscase {
                    (*curbuf.get()).b_ind_case_code
                } else {
                    (*curbuf.get()).b_ind_scopedecl_code
                };
            self.lookfor = if (*curbuf.get()).b_ind_case_break != 0 {
                LOOKFOR_NOBREAK
            } else {
                LOOKFOR_ANY
            };
            Step::Again
        }
    }

    /// The line is after a `while (cond);` -- ignore everything until the
    /// matching `do`.
    ///
    /// # Safety
    /// Reads the cursor; may unlock the current line.
    unsafe fn on_while_of_do_end(&mut self) -> Step {
        unsafe {
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
                self.amount = get_indent();
                if self.line.starts_with(b'{') {
                    self.amount += (*curbuf.get()).b_ind_open_extra;
                }
            }
            self.whilelevel += 1;
            Step::Again
        }
    }

    /// The line is a terminated "normal" statement.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_terminated(&mut self) -> Step {
        unsafe {
            // Skip a lone `break` before a switch label: it may be lined up
            // with the label ('cinoptions' `b`).
            if self.lookfor == LOOKFOR_NOBREAK && cin_isbreak(skipwhite(get_cursor_line_ptr())) {
                self.lookfor = LOOKFOR_ANY;
                return Step::Again;
            }

            // Handle a "do {" line.
            if self.whilelevel > 0 && cin_isdo(cin_skipcomment(get_cursor_line_ptr())) {
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
            self.walk_back_over_terminated()
        }
    }

    /// Upstream's `term_again`: step from a terminated line onto whatever
    /// encloses it, repeating while that is another block's end.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn walk_back_over_terminated(&mut self) -> Step {
        unsafe {
            loop {
                // Position on the rightmost paren so that matching it takes
                // us to the start of the line.  Helps for:
                //     func(asdr,
                //              asdfasdf);
                //     here;
                let mut l = get_cursor_line_ptr().cast_const();
                if find_last_paren(l, b'(', b')') {
                    let trypos = find_match_paren((*curbuf.get()).b_ind_maxparen);
                    if !trypos.is_null() {
                        // Check whether we are on a case label now; that is
                        // handled above.
                        //         case xx:  if ( asdf &&
                        //                          asdf)
                        (*curwin.get()).w_cursor = *trypos;
                        l = get_cursor_line_ptr();
                        if cin_iscase(l, false) || cin_isscopedecl(l) {
                            // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                            self.resume_at((*curwin.get()).w_cursor.lnum);
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
                let iscase = (*curbuf.get()).b_ind_keep_case_label != 0 && cin_iscase(l, false);

                // The indent of the current line, ignoring any jump label.
                self.amount = skip_label((*curwin.get()).w_cursor.lnum, &mut l);
                if self.line.starts_with(b'{') {
                    self.amount += (*curbuf.get()).b_ind_open_extra;
                }
                // See the remark above: only add `b_ind_open_extra` when the
                // line does not itself start with a '{'.
                l = skipwhite(l);
                if *l as u8 == b'{' {
                    self.amount -= (*curbuf.get()).b_ind_open_extra;
                }
                self.lookfor = if iscase { LOOKFOR_ANY } else { LOOKFOR_TERM };

                // A terminated line starting with "else" needs the scope of
                // *that* else, so skip to the matching "if".  With
                // `whilelevel != 0` keep looking for a "do {" instead.
                if self.lookfor == LOOKFOR_TERM
                    && *l as u8 != b'}'
                    && cin_iselse(l)
                    && self.whilelevel == 0
                {
                    let trypos = find_start_brace();
                    if trypos.is_null() || !find_match(LOOKFOR_IF, (*trypos).lnum) {
                        return Step::Done;
                    }
                    return Step::Again;
                }

                // At the end of a block: skip to the start of that block.
                l = get_cursor_line_ptr();
                if find_last_paren(l, b'{', b'}') {
                    let trypos = find_start_brace();
                    if !trypos.is_null() {
                        (*curwin.get()).w_cursor = *trypos;
                        // If not "else {", check for terminated again; but
                        // skip the block for "} else {".
                        l = cin_skipcomment(get_cursor_line_ptr());
                        if *l as u8 == b'}' || !cin_iselse(l) {
                            continue; // term_again
                        }
                        // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                        self.resume_at((*curwin.get()).w_cursor.lnum);
                    }
                }
                return Step::Again;
            }
        }
    }

    /// The line is *not* terminated (or ends in a `,` that does not count).
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn on_unterminated(&mut self, mut l: *const c_char, terminated: u8) -> Step {
        unsafe {
            // `l` holds code -- `cin_nocode` was false -- so `strlen` is at
            // least 1 and upstream's `l[strlen(l) - 1]` is in bounds.
            let last = strlen(l).checked_sub(1).map_or(0, |i| *l.add(i) as u8);
            if self.lookfor != LOOKFOR_ENUM_OR_INIT && (*skipwhite(l) as u8 == b'[' || last == b'[')
            {
                self.amount += self.ind_continuation;
            }

            // In the middle of a paren thing: go back to the line that starts
            // it, to get the right prevailing indent --
            //     if ( foo &&
            //              bar )
            // Position on the rightmost paren so that matching it takes us to
            // the start of the line, and ignore a match before the block.
            find_last_paren(l, b'(', b')');
            let mut trypos = find_match_paren(corr_ind_maxparen(&self.line.cur_curpos));
            if !trypos.is_null()
                && ((*trypos).lnum < self.brace.lnum
                    || ((*trypos).lnum == self.brace.lnum && (*trypos).col < self.brace.col))
            {
                trypos = ::core::ptr::null_mut::<pos_T>();
            }
            l = get_cursor_line_ptr();

            // Looking for a ',' means matching braces count too.
            if trypos.is_null() && terminated == b',' {
                if find_last_paren(l, b'{', b'}') {
                    trypos = find_start_brace();
                }
                l = get_cursor_line_ptr();
            }

            if !trypos.is_null() {
                // Check whether we are on a case label now; that is handled
                // above.
                //     case xx:  if ( asdf &&
                //                        asdf)
                (*curwin.get()).w_cursor = *trypos;
                l = get_cursor_line_ptr();
                if cin_iscase(l, false) || cin_isscopedecl(l) {
                    // Upstream's `w_cursor.lnum++; col = 0;`: re-read this line.
                    self.resume_at((*curwin.get()).w_cursor.lnum);
                    return Step::Again;
                }
            }

            // Skip over continuation lines to find the one to take the indent
            // from --
            //     char *usethis = "bla\
            //               bla",
            //          here;
            if terminated == b',' {
                while (*curwin.get()).w_cursor.lnum > 1 {
                    let above = ml_get((*curwin.get()).w_cursor.lnum - 1);
                    if *above == 0 || *above.add(strlen(above) - 1) as u8 != b'\\' {
                        break;
                    }
                    (*curwin.get()).w_cursor.lnum -= 1;
                    (*curwin.get()).w_cursor.col = 0;
                }
                l = get_cursor_line_ptr();
            }

            // The indent and the text of the current line, ignoring any jump
            // label.
            self.cur_amount = if (*curbuf.get()).b_ind_js != 0 {
                get_indent()
            } else {
                skip_label((*curwin.get()).w_cursor.lnum, &mut l)
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
                if *skipwhite(l) as u8 != b'{' {
                    self.amount += (*curbuf.get()).b_ind_open_extra;
                }
                if (*curbuf.get()).b_ind_cpp_baseclass != 0 && (*curbuf.get()).b_ind_js == 0 {
                    // Have to look back for a cpp base-class declaration or
                    // initialisation.
                    self.lookfor = LOOKFOR_CPP_BASECLASS;
                    return Step::Again;
                }
                return Step::Done;
            }

            // After an "if", "while", etc.  Also allow "   } else".
            if cin_is_cinword(l) || cin_iselse(skipwhite(l)) {
                self.after_cinword()
            } else {
                self.after_plain_unterminated(terminated)
            }
        }
    }

    /// The unterminated line above is an `if`/`while`/`for`/`else`.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn after_cinword(&mut self) -> Step {
        unsafe {
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
                self.amount += (*curbuf.get()).b_ind_open_extra;
            }
            if self.lookfor != LOOKFOR_TERM {
                self.amount += (*curbuf.get()).b_ind_level + (*curbuf.get()).b_ind_no_brace;
                return Step::Done;
            }

            // Expecting the `while ()` after a `do`: line up with the
            // `while()`.
            //     do
            //            x = 1;
            // ->  here
            let l = skipwhite(get_cursor_line_ptr()).cast_const();
            if cin_isdo(l) {
                if self.whilelevel == 0 {
                    return Step::Done;
                }
                self.whilelevel -= 1;
            }

            // Searching for a terminated line: do not use the one between the
            // "if" and the matching "else"; use the scope of *this* "else".
            // With `whilelevel != 0` keep looking for a "do {".
            if cin_iselse(l) && self.whilelevel == 0 {
                // For "} else", find the opening brace of the enclosing
                // scope, not the one from "if () {".
                if *l as u8 == b'}' {
                    (*curwin.get()).w_cursor.col =
                        l.offset_from(get_cursor_line_ptr()) as colnr_T + 1;
                }
                let trypos = find_start_brace();
                if trypos.is_null() || !find_match(LOOKFOR_IF, (*trypos).lnum) {
                    return Step::Done;
                }
            }
            Step::Again
        }
    }

    /// The unterminated line above is an ordinary statement.
    ///
    /// # Safety
    /// Moves the cursor; may unlock the current line.
    unsafe fn after_plain_unterminated(&mut self, terminated: u8) -> Step {
        unsafe {
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
                    if (*curbuf.get()).b_ind_cpp_baseclass == 0 {
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
            let l = get_cursor_line_ptr().cast_const();
            self.amount = self.cur_amount;

            let n = strlen(l);
            if (*curbuf.get()).b_ind_js != 0
                && terminated == b','
                && (*skipwhite(l) as u8 == b']' || (n >= 2 && *l.add(n - 2) as u8 == b']'))
            {
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
                if (*curbuf.get()).b_ind_js == 0 {
                    self.lookfor = LOOKFOR_ENUM_OR_INIT;
                    self.cont_amount = cin_first_id_amount();
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
                if cin_iscomment(skipwhite(l)) {
                    return Step::Done;
                }
                self.lookfor = LOOKFOR_COMMA;
                let trypos = find_match_char(b'[', (*curbuf.get()).b_ind_maxparen);
                if !trypos.is_null() {
                    if (*trypos).lnum == (*curwin.get()).w_cursor.lnum - 1 {
                        // The current line is the first inside [], so line up
                        // with it.
                        return Step::Done;
                    }
                    self.ourscope = (*trypos).lnum;
                }
                return Step::Again;
            }

            if self.lookfor == LOOKFOR_INITIAL && *l != 0 && *l.add(n - 1) as u8 == b'\\' {
                self.cont_amount = cin_get_equal_amount((*curwin.get()).w_cursor.lnum);
            }
            if self.lookfor != LOOKFOR_TERM
                && self.lookfor != LOOKFOR_JS_KEY
                && self.lookfor != LOOKFOR_COMMA
                && self.raw_string_start != (*curwin.get()).w_cursor.lnum
            {
                self.lookfor = LOOKFOR_UNTERM;
            }
            Step::Again
        }
    }
}
