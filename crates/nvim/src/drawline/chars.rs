//! Deciding which character goes in the next screen cell.
//!
//! This is the half of [`Cells`](super::Cells)'s pass that answers
//! [`Cells::cell_char`](super::Cells) — the other half, in
//! [`cells`](super::cells), decides what *attribute* it takes and where it
//! goes.
//!
//! There are four sources, tried in order:
//!
//! 1. a run already in progress — [`WinLineVars::extra_todo`] cells of either one
//!    repeated character ([`WinLineVars::extra_fill`]) or a string
//!    ([`WinLineVars::extra_text`]). Tabs, `<xx>` escapes, `'listchars'`
//!    replacements, the fold fill and inline virtual text all become one of
//!    these;
//! 2. a blank, while diff or virtual-line filler is being drawn;
//! 3. nothing at all, for a closed fold whose text is already placed;
//! 4. the buffer line itself, which is where the work is: the character is
//!    read, then syntax, decorations, spelling, `:terminal` attributes,
//!    `'linebreak'`, `'listchars'`, the non-printable forms and finally
//!    concealment each get a look at it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

impl Cells {
    /// Put the next character in [`Cells::cell_char`].
    ///
    /// # Safety
    /// `wp` must be live and `f` must hold the caller's frame.
    pub(super) unsafe fn next_char(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
    ) {
        // SAFETY: the caller's window and frame.
        unsafe {
            if wlv.extra_todo > 0 {
                self.char_from_extra(wlv, wp);
            } else if wlv.filler_todo > 0 {
                // Wait with reading text until the filler lines are done, but
                // still give the cell something to be.
                self.char_code = ' ' as ::core::ffi::c_int;
                self.cell_char = schar_from_ascii(b' ');
            } else if self.has_foldtext || (self.has_fold && wlv.col >= self.view_width) {
                // The fold text is already placed; skip the buffer line.
                self.cell_char = NUL as schar_T;
            } else {
                self.char_from_buffer(wlv, wp, f);
            }
        }
    }

    /// Take one cell from the run in [`WinLineVars::extra_todo`].
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn char_from_extra(&mut self, wlv: &mut WinLineVars, wp: *mut win_T) {
        // SAFETY: the caller's window; `extra_text` is NUL-terminated whenever
        // `extra_fill` and `extra_last` are not set.
        unsafe {
            if wlv.extra_fill != NUL as schar_T
                || (wlv.extra_todo == 1 && wlv.extra_last != NUL as schar_T)
            {
                // One character repeated, with an optional different last one.
                self.cell_char = if wlv.extra_todo == 1 && wlv.extra_last != NUL as schar_T {
                    wlv.extra_last
                } else {
                    wlv.extra_fill
                };
                self.char_code = schar_get_first_codepoint(self.cell_char);
                wlv.extra_todo -= 1;
            } else {
                debug_assert!(!wlv.extra_text.is_null());
                self.char_len = utfc_ptr2len(wlv.extra_text);
                self.cell_char = utfc_ptr2schar(wlv.extra_text, &raw mut self.char_code);
                // `char_len` is 0 at the end-of-line NUL.
                if self.char_len > wlv.extra_todo || self.char_len == 0 {
                    self.char_len = 1;
                }

                if wlv.col >= self.view_width - 1 && schar_cells(self.cell_char) == 2 {
                    // A double-width character with one column left: show a
                    // `>` and leave the pointer where it is, so the character
                    // itself starts the next row.
                    self.char_code = '>' as ::core::ffi::c_int;
                    self.char_len = 1;
                    self.cell_char = schar_from_ascii(b'>');
                    self.overflow_attr = win_hl_attr(wp, HLF_AT);
                    if wlv.cursorline_attr != 0 {
                        self.overflow_attr = if wlv.line_attr_lowprio != 0 {
                            hl_combine_attr(wlv.cursorline_attr, self.overflow_attr)
                        } else {
                            hl_combine_attr(self.overflow_attr, wlv.cursorline_attr)
                        };
                    }
                } else {
                    wlv.extra_todo -= self.char_len;
                    wlv.extra_text = wlv.extra_text.offset(self.char_len as isize);
                }

                if wlv.filler_todo <= 0 && wlv.skip_cells > 0 && self.char_len > 1 {
                    // A double-width character half scrolled off the left
                    // edge: show a `<` for the half that is on screen.
                    if wlv.extra_todo > 0 {
                        self.extra_todo_next = wlv.extra_todo;
                        self.extra_attr_next = wlv.extra_attr;
                    }
                    wlv.extra_todo = 1;
                    wlv.extra_fill = schar_from_ascii(MB_FILLER_CHAR);
                    wlv.extra_last = NUL as schar_T;
                    self.cell_char = schar_from_ascii(b' ');
                    self.char_code = ' ' as ::core::ffi::c_int;
                    self.char_len = 1;
                    wlv.n_attr += 1;
                    wlv.extra_attr = win_hl_attr(wp, HLF_AT);
                }
            }

            if wlv.extra_todo > 0 {
                return;
            }
            if self.extra_todo_next <= 0 {
                // The run is spent: put back the attributes the inline virtual
                // text displaced.
                if self.search_attr == 0 {
                    self.search_attr = self.saved_search_attr;
                    self.saved_search_attr = 0;
                }
                if self.area_attr == 0 && *self.ptr as ::core::ffi::c_int != NUL {
                    self.area_attr = self.saved_area_attr;
                    self.saved_area_attr = 0;
                }
                if self.decor_attr == 0 {
                    self.decor_attr = self.saved_decor_attr;
                    self.saved_decor_attr = 0;
                }
                if wlv.extra_is_virt_text {
                    // `extra_attr` applies at this position but no further.
                    wlv.reset_extra_attr = true;
                    self.extra_attr_next = -1;
                }
                wlv.extra_is_virt_text = false;
            } else {
                // A `<` filler interrupted a longer run; resume it.
                debug_assert!(wlv.extra_fill != NUL as schar_T || wlv.extra_last != NUL as schar_T);
                debug_assert!(!wlv.extra_text.is_null());
                wlv.extra_fill = NUL as schar_T;
                wlv.extra_last = NUL as schar_T;
                wlv.extra_todo = self.extra_todo_next;
                self.extra_todo_next = 0;
                // `extra_attr` applies at this position; `extra_attr_next`
                // after it.
                wlv.reset_extra_attr = true;
                debug_assert!(self.extra_attr_next >= 0);
            }
        }
    }

    /// Read the next character out of the buffer line and work out what it
    /// looks like on screen.
    ///
    /// # Safety
    /// `wp` must be live and `f` must hold the caller's frame.
    pub(super) unsafe fn char_from_buffer(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
    ) {
        // SAFETY: the caller's window and frame; `ptr` walks a NUL-terminated
        // buffer line.
        unsafe {
            let mut prev_ptr: *mut ::core::ffi::c_char = self.ptr;

            // First byte of the next character.
            let mut c0 = *self.ptr as uint8_t as ::core::ffi::c_int;
            if c0 == NUL {
                // No more cells to skip.
                wlv.skip_cells = 0;
            }

            self.char_len = utfc_ptr2len(self.ptr);
            self.cell_char = utfc_ptr2schar(self.ptr, &raw mut self.char_code);

            // Overlong-encoded ASCII, or ASCII with a composing character, is
            // displayed normally — except a NUL.
            if self.char_len > 1 && self.char_code < 0x80 {
                c0 = self.char_code;
            }

            if (self.char_len == 1 && c0 >= 0x80)
                || (self.char_len >= 1 && self.char_code == 0)
                || (self.char_len > 1 && !vim_isprintc(self.char_code))
            {
                // An illegal UTF-8 byte shows as `<xx>`; an unprintable
                // character as `?` or its fullwidth form.
                transchar_hex(wlv.escape_buf.as_mut_ptr(), self.char_code);
                if (*wp).w_onebuf_opt.wo_rl != 0 {
                    rl_mirror_ascii(wlv.escape_buf.as_mut_ptr(), ::core::ptr::null_mut());
                }
                wlv.extra_text = wlv.escape_buf.as_mut_ptr();
                let mut p: *const ::core::ffi::c_char = wlv.extra_text;
                self.char_code = mb_ptr2char_adv(&raw mut p);
                wlv.extra_text = p as *mut ::core::ffi::c_char;
                self.cell_char = schar_from_char(self.char_code);
                wlv.extra_todo = strlen(wlv.extra_text) as ::core::ffi::c_int;
                wlv.extra_fill = NUL as schar_T;
                wlv.extra_last = NUL as schar_T;
                if self.area_attr == 0 && self.search_attr == 0 {
                    wlv.n_attr = wlv.extra_todo + 1;
                    wlv.extra_attr = win_hl_attr(wp, HLF_8);
                    self.attr_before_run = wlv.char_attr;
                }
            } else if self.char_len == 0 {
                // At the NUL that ends the line.
                self.char_len = 1;
            }

            if wlv.col >= self.view_width - 1 && schar_cells(self.cell_char) == 2 {
                // A double-width character with one column left: show a `>`
                // and put the pointer back, so the character is drawn at the
                // start of the next row.
                self.cell_char = schar_from_ascii(b'>');
                self.char_code = '>' as ::core::ffi::c_int;
                self.char_len = 1;
                self.overflow_attr = win_hl_attr(wp, HLF_AT);
                self.ptr = self.ptr.offset(-1);
                self.did_decrement_ptr = true;
            } else if *self.ptr as ::core::ffi::c_int != NUL {
                self.ptr = self.ptr.offset(self.char_len as isize - 1);
            }

            if wlv.skip_cells > 0 && self.char_len > 1 && wlv.extra_todo == 0 {
                // A double-width character half scrolled off the left edge
                // shows a `<`. Not for unprintable characters, which took the
                // branch above.
                wlv.extra_todo = 1;
                wlv.extra_fill = schar_from_ascii(MB_FILLER_CHAR);
                wlv.extra_last = NUL as schar_T;
                self.cell_char = schar_from_ascii(b' ');
                self.char_code = ' ' as ::core::ffi::c_int;
                self.char_len = 1;
                if self.area_attr == 0 && self.search_attr == 0 {
                    wlv.n_attr = wlv.extra_todo + 1;
                    wlv.extra_attr = win_hl_attr(wp, HLF_AT);
                    self.attr_before_run = wlv.char_attr;
                }
            }
            self.ptr = self.ptr.offset(1);

            self.decor_attr = 0;
            if self.extra_check {
                self.slow_path(wlv, wp, f, &mut prev_ptr, c0);
            }

            if !vim_isprintc(self.char_code) {
                self.unprintable(wlv, wp);
            }

            self.conceal(wlv, wp);

            if wlv.skip_cells > 0 && self.did_decrement_ptr {
                // The `>` is not being shown, so put the pointer back or the
                // loop would not move on.
                self.ptr = self.ptr.offset(1);
            }
        }
    }

    /// Everything a character may need beyond being read: syntax and
    /// decoration attributes, spelling, `:terminal` colours, `'linebreak'` and
    /// the `'listchars'` space replacements.
    ///
    /// Reached only when the setup half decided the line needs it, which is
    /// what keeps a plain line's inner loop short.
    ///
    /// # Safety
    /// `wp` must be live, `f` must hold the caller's frame and `prev_ptr` must
    /// point into the line.
    pub(super) unsafe fn slow_path(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
        prev_ptr: &mut *mut ::core::ffi::c_char,
        c0: ::core::ffi::c_int,
    ) {
        // SAFETY: the caller's window, frame and line pointers.
        unsafe {
            let no_plain_buffer =
                (*(*wp).w_s).b_p_spo_flags & kOptSpoFlagNoplainbuffer as uint32_t != 0;
            let mut can_spell = !no_plain_buffer;

            // Not at the start of the line only because a double-width
            // character did not fit: then there is nothing to ask about yet.
            let at = self.byte_col();
            if self.has_syntax && at > 0 {
                let prev_at = prev_ptr.offset_from(self.line);
                self.syntax_attr(wlv, wp, f, at, &mut can_spell);
                // The syntax walk may have re-fetched the line.
                *prev_ptr = self.line.offset(prev_at);
            }

            if self.has_decor && at > 0 {
                // Extmarks take precedence over syntax.
                self.decor_attr = hl_combine_attr(self.decor_attr, self.extmark_attr);
                self.decor_conceal = wlv.decor.conceal;
                // The decoration only speaks when it has an opinion.
                can_spell = wlv.decor.spell.unwrap_or(can_spell);
            }

            self.attr_base = hl_combine_attr(self.fold_attr, self.decor_attr);
            wlv.char_attr = hl_combine_attr(self.attr_base, self.attr_pri);

            self.spell_at(wlv, wp, f, *prev_ptr, can_spell);
            if self.spell_attr != 0 {
                self.attr_base = hl_combine_attr(self.attr_base, self.spell_attr);
                wlv.char_attr = hl_combine_attr(self.attr_base, self.attr_pri);
            }

            if !(*(*wp).w_buffer).terminal.is_null() {
                let term_attr = if wlv.vcol < TERM_ATTRS_MAX as ::core::ffi::c_int {
                    *f.term_attrs.offset(wlv.vcol as isize)
                } else {
                    0
                };
                wlv.char_attr = hl_combine_attr(term_attr, wlv.char_attr);
            }

            self.linebreak(wlv, wp, c0);
            self.listchars(wlv, wp, *prev_ptr);
        }
    }

    /// Ask the syntax state machine for this character's attribute, and for
    /// whether it is in the `@Spell` cluster.
    ///
    /// # Safety
    /// `wp` must be live and `at`/`prev_at` byte indexes into the line.
    pub(super) unsafe fn syntax_attr(
        &mut self,
        wlv: &WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
        at: ::core::ffi::c_int,
        can_spell: &mut bool,
    ) {
        // SAFETY: the caller's window and byte indexes.
        unsafe {
            // An error inside the syntax patterns turns highlighting off for
            // the buffer rather than being reported per character.
            let save_did_emsg = did_emsg.get();
            did_emsg.set(0);

            self.decor_attr = get_syntax_attr(
                at - 1,
                if (*f.spv).spv_has_spell {
                    can_spell as *mut bool
                } else {
                    ::core::ptr::null_mut()
                },
                false,
            );

            if did_emsg.get() != 0 {
                (*(*wp).w_s).b_syn_error = true;
                self.has_syntax = false;
            } else {
                did_emsg.set(save_did_emsg);
            }
            if (*(*wp).w_s).b_syn_slow {
                self.has_syntax = false;
            }

            // A multi-line regexp may have invalidated the line.
            self.refetch_line(wp, wlv.lnum, at);

            // No concealing past the end of the line: it would interfere with
            // the line highlighting.
            self.syntax_flags = if self.cell_char == 0 {
                SynFlags::NONE
            } else {
                get_syntax_info(&raw mut self.syntax_seqnr)
            };
        }
    }

    /// Spell-check the word starting at this character, unless the checker has
    /// already answered for it.
    ///
    /// The answer covers a whole word, so this runs once per word rather than
    /// once per cell: [`Cells::word_end`] is where the last answer reached and
    /// [`Cells::cur_checked_col`] is how far a word carried over from the
    /// previous buffer line already reached.
    ///
    /// # Safety
    /// `wp` must be live, `f` must hold the caller's frame and `prev_ptr` must
    /// point into the line.
    pub(super) unsafe fn spell_at(
        &mut self,
        wlv: &WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
        prev_ptr: *mut ::core::ffi::c_char,
        can_spell: bool,
    ) {
        // SAFETY: the caller's window, frame and line pointers.
        unsafe {
            let mut at = self.byte_col();
            if !(*f.spv).spv_has_spell || at < self.word_end || at <= self.cur_checked_col {
                return;
            }
            self.spell_attr = 0;
            // No capital column at the end of the line, or when only white
            // space follows.
            if self.cell_char == 0
                || *skipwhite(prev_ptr) as ::core::ffi::c_int == NUL
                || !can_spell
            {
                return;
            }

            let mut spell_hlf: hlf_T = HLF_COUNT;
            at -= self.char_len - 1;

            // Use the look-ahead buffer where it reaches: it carries the start
            // of the next line, so a word at the end of this one is checked
            // against what follows it.
            let prev_at = prev_ptr.offset_from(self.line);
            let p = if prev_at - self.nextlinecol as isize >= 0 {
                f.nextline.offset(prev_at - self.nextlinecol as isize)
            } else {
                prev_ptr
            };

            (*f.spv).spv_cap_col -= prev_at as ::core::ffi::c_int;
            let tmplen = spell_check(
                wp,
                p,
                &raw mut spell_hlf,
                &raw mut (*f.spv).spv_cap_col,
                (*f.spv).spv_unchanged,
            );
            debug_assert!(tmplen <= ::core::ffi::c_int::MAX as size_t);
            let len = tmplen as ::core::ffi::c_int;
            self.word_end = at + len;

            // In Insert mode do not highlight a word the cursor is touching.
            if spell_hlf != HLF_COUNT
                && State.get() & MODE_INSERT != 0
                && (*wp).w_cursor.lnum == wlv.lnum
                && (*wp).w_cursor.col >= prev_at as colnr_T
                && (*wp).w_cursor.col < self.word_end
            {
                spell_hlf = HLF_COUNT;
                spell_redraw_lnum.set(wlv.lnum);
            }

            if spell_hlf == HLF_COUNT
                && p != prev_ptr
                && p.offset_from(f.nextline) + len as isize > self.nextline_idx as isize
            {
                // The good word continues at the start of the next line.
                (*f.spv).spv_checked_lnum = wlv.lnum + 1;
                (*f.spv).spv_checked_col = (p.offset_from(f.nextline) + len as isize
                    - self.nextline_idx as isize)
                    as ::core::ffi::c_int;
            }

            if spell_hlf != HLF_COUNT {
                self.spell_attr = default_hl_attr(spell_hlf as usize);
            }

            if (*f.spv).spv_cap_col > 0 {
                if p != prev_ptr
                    && p.offset_from(f.nextline) + (*f.spv).spv_cap_col as isize
                        >= self.nextline_idx as isize
                {
                    // The word on the next line must start with a capital.
                    (*f.spv).spv_capcol_lnum = wlv.lnum + 1;
                    (*f.spv).spv_cap_col = (p.offset_from(f.nextline)
                        + (*f.spv).spv_cap_col as isize
                        - self.nextline_idx as isize)
                        as ::core::ffi::c_int;
                } else {
                    // Turn it back into a column of this line.
                    (*f.spv).spv_cap_col += prev_at as ::core::ffi::c_int;
                }
            }
        }
    }

    /// `'linebreak'`: when this character is the last blank before a word that
    /// will not fit, pad out to the end of the row so the word starts on the
    /// next one.
    ///
    /// # Safety
    /// `wp` must be a live window.
    pub(super) unsafe fn linebreak(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        c0: ::core::ffi::c_int,
    ) {
        // SAFETY: the caller's window and the loop's line pointers.
        unsafe {
            if (*wp).w_onebuf_opt.wo_lbr == 0 {
                return;
            }
            // Do not break a line that starts with blanks followed by a long
            // word: the break would land at the very beginning, which is not
            // what anyone means by 'linebreak'. So arm it only once a
            // character outside 'breakat' has been seen.
            if !wlv.linebreak_armed
                && self.cell_char != NUL as schar_T
                && !vim_isbreak(*self.ptr as uint8_t as ::core::ffi::c_int)
            {
                wlv.linebreak_armed = true;
            }
            // The last blank before a word: this is where the break goes.
            if !(c0 == self.char_code
                && self.char_code < 128
                && wlv.linebreak_armed
                && vim_isbreak(self.char_code)
                && !vim_isbreak(*self.ptr as uint8_t as ::core::ffi::c_int))
            {
                return;
            }
            let mb_off = utf_head_off(self.line, self.ptr.offset(-1));
            let p = self.ptr.offset(-(mb_off as isize + 1));

            let mut csarg = CharsizeArg::default();
            // `lnum` 0: virtual text is not to be counted here.
            let cstype = init_charsize_arg(&mut csarg, Win::new(wp), 0, self.line);
            wlv.extra_todo =
                win_charsize(cstype, wlv.vcol, p, utf_ptr2char_info(p).value, &mut csarg).width - 1;

            if self.on_last_col && self.char_code != TAB {
                // Search and match highlighting do not continue over the line
                // break — but a Tab's highlight covers its whole width.
                self.search_attr = 0;
            }
            if self.char_code == TAB && wlv.extra_todo + wlv.col > self.view_width {
                wlv.extra_todo = tabstop_padding(
                    wlv.vcol,
                    (*(*wp).w_buffer).b_p_ts,
                    (*(*wp).w_buffer).b_p_vts_array,
                ) - 1;
            }
            wlv.extra_fill = schar_from_ascii(if mb_off > 0 { MB_FILLER_CHAR } else { b' ' });
            wlv.extra_last = NUL as schar_T;
            if self.char_code < 128 && ascii_iswhite(self.char_code) {
                if self.char_code == TAB {
                    // See "Tab alignment" in `unprintable`.
                    wlv.fix_for_boguscols();
                }
                if (*wp).w_onebuf_opt.wo_list == 0 {
                    self.char_code = ' ' as ::core::ffi::c_int;
                    self.cell_char = schar_from_ascii(b' ');
                }
            }
        }
    }

    /// `'list'`: replace a space, a non-breaking space or a run of spaces with
    /// the `'listchars'` character that stands for it.
    ///
    /// # Safety
    /// `wp` must be live and `prev_ptr` point into the line.
    pub(super) unsafe fn listchars(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        prev_ptr: *mut ::core::ffi::c_char,
    ) {
        // SAFETY: the caller's window and line pointers.
        unsafe {
            if (*wp).w_onebuf_opt.wo_list != 0 {
                self.in_multispace = self.char_code == ' ' as ::core::ffi::c_int
                    && (*self.ptr as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                        || (prev_ptr > self.line
                            && *prev_ptr.offset(-1) as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int));
                if !self.in_multispace {
                    self.multispace_pos = 0;
                }
            }

            let lcs = &(*wp).w_p_lcs_chars;
            let at = self.ptr.offset_from(self.line);

            // 'list': change 160 to "nbsp" and a space to "space" — but not
            // when a composing character follows, which `char_len` reveals.
            if (*wp).w_onebuf_opt.wo_list != 0
                && ((((self.char_code == 160 && self.char_len == 2)
                    || (self.char_code == 0x202f && self.char_len == 3))
                    && lcs.nbsp != 0)
                    || (self.char_code == ' ' as ::core::ffi::c_int
                        && self.char_len == 1
                        && (lcs.space != 0 || (self.in_multispace && !lcs.multispace.is_null()))
                        && at >= self.leadcol as isize
                        && at <= self.trailcol as isize))
            {
                if self.in_multispace && !lcs.multispace.is_null() {
                    self.cell_char = *lcs.multispace.offset(self.multispace_pos as isize);
                    self.multispace_pos += 1;
                    if *lcs.multispace.offset(self.multispace_pos as isize) == NUL as schar_T {
                        self.multispace_pos = 0;
                    }
                } else {
                    self.cell_char = if self.char_code == ' ' as ::core::ffi::c_int {
                        lcs.space
                    } else {
                        lcs.nbsp
                    };
                }
                wlv.n_attr = 1;
                wlv.extra_attr = win_hl_attr(wp, HLF_0);
                self.attr_before_run = wlv.char_attr;
                self.char_code = schar_get_first_codepoint(self.cell_char);
            }

            // Leading and trailing whitespace get their own characters, and
            // this one is honoured with 'nolist' too.
            if self.char_code == ' ' as ::core::ffi::c_int
                && self.char_len == 1
                && ((self.trailcol != MAXCOL as colnr_T && at > self.trailcol as isize)
                    || (self.leadcol != 0 && at < self.leadcol as isize))
            {
                if self.leadcol != 0
                    && self.in_multispace
                    && at < self.leadcol as isize
                    && !lcs.leadmultispace.is_null()
                {
                    self.cell_char = *lcs.leadmultispace.offset(self.multispace_pos as isize);
                    self.multispace_pos += 1;
                    if *lcs.leadmultispace.offset(self.multispace_pos as isize) == NUL as schar_T {
                        self.multispace_pos = 0;
                    }
                } else if at > self.trailcol as isize && lcs.trail != 0 {
                    self.cell_char = lcs.trail;
                } else if at < self.leadcol as isize && lcs.lead != 0 {
                    self.cell_char = lcs.lead;
                } else if self.leadcol != 0 && lcs.space != 0 {
                    self.cell_char = lcs.space;
                }
                wlv.n_attr = 1;
                wlv.extra_attr = win_hl_attr(wp, HLF_0);
                self.attr_before_run = wlv.char_attr;
                self.char_code = schar_get_first_codepoint(self.cell_char);
            }
        }
    }
}
