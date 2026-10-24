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

#![forbid(unsafe_code)]

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

impl Cells {
    /// Put the next character in [`Cells::cell_char`].
    pub(super) fn next_char(&mut self, wlv: &mut WinLineVars, window: Win, f: &mut LineFrame<'_>) {
        if wlv.extra_todo > 0 {
            self.char_from_extra(wlv, window);
        } else if wlv.filler_todo > 0 {
            // Wait with reading text until the filler lines are done, but
            // still give the cell something to be.
            self.char_code = ' ' as ::core::ffi::c_int;
            self.cell_char = schar_from_ascii(b' ');
        } else if self.has_foldtext || (self.has_fold && wlv.col >= self.view_width) {
            // The fold text is already placed; skip the buffer line.
            self.cell_char = NUL as ScreenChar;
        } else {
            self.char_from_buffer(wlv, window, f);
        }
    }

    /// Take one cell from the run in [`WinLineVars::extra_todo`].
    pub(super) fn char_from_extra(&mut self, wlv: &mut WinLineVars, window: Win) {
        if wlv.extra_fill != NUL as ScreenChar
            || (wlv.extra_todo == 1 && wlv.extra_last != NUL as ScreenChar)
        {
            // One character repeated, with an optional different last one.
            self.cell_char = if wlv.extra_todo == 1 && wlv.extra_last != NUL as ScreenChar {
                wlv.extra_last
            } else {
                wlv.extra_fill
            };
            self.char_code = schar_get_first_codepoint(self.cell_char);
            wlv.extra_todo -= 1;
        } else {
            (self.char_len, self.cell_char, self.char_code) = cluster_at(wlv.extra.rest());
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
                self.overflow_attr = win_hl_attr(window, HLF_AT);
                if wlv.cursorline_attr != 0 {
                    self.overflow_attr = if wlv.line_attr_lowprio != 0 {
                        hl_combine_attr(wlv.cursorline_attr, self.overflow_attr)
                    } else {
                        hl_combine_attr(self.overflow_attr, wlv.cursorline_attr)
                    };
                }
            } else {
                wlv.extra_todo -= self.char_len;
                wlv.extra
                    .advance(usize::try_from(self.char_len).unwrap_or(0));
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
                wlv.extra_last = NUL as ScreenChar;
                self.cell_char = schar_from_ascii(b' ');
                self.char_code = ' ' as ::core::ffi::c_int;
                self.char_len = 1;
                wlv.n_attr += 1;
                wlv.extra_attr = win_hl_attr(window, HLF_AT);
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
            if self.area_attr == 0 && self.byte() != b'\0' {
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
            debug_assert!(
                wlv.extra_fill != NUL as ScreenChar || wlv.extra_last != NUL as ScreenChar
            );
            wlv.extra_fill = NUL as ScreenChar;
            wlv.extra_last = NUL as ScreenChar;
            wlv.extra_todo = self.extra_todo_next;
            self.extra_todo_next = 0;
            // `extra_attr` applies at this position; `extra_attr_next`
            // after it.
            wlv.reset_extra_attr = true;
            debug_assert!(self.extra_attr_next >= 0);
        }
    }

    /// Read the next character out of the buffer line and work out what it
    /// looks like on screen.
    pub(super) fn char_from_buffer(
        &mut self,
        wlv: &mut WinLineVars,
        window: Win,
        f: &mut LineFrame<'_>,
    ) {
        let prev_at = self.pos;

        // First byte of the next character.
        let first = self.byte();
        let mut c0 = ::core::ffi::c_int::from(first);
        if c0 == NUL {
            // No more cells to skip.
            wlv.skip_cells = 0;
        }

        (self.char_len, self.cell_char, self.char_code) = cluster_at(self.rest());

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
            let mut hex: CharDisplay = [0; CHAR_DISPLAY_LEN];
            let len = transchar_hex_into(&mut hex, self.char_code);
            let mut text = hex.map(|c| c as u8);
            let text = &mut text[..len];
            if window.w_onebuf_opt.wo_rl != 0 {
                mirror_ascii(text);
            }
            // The first character is this cell; the rest is the run.
            self.char_code = c_char_info(text, 0).value;
            let first = c_cluster_len(text);
            wlv.extra.set(text);
            wlv.extra.advance(first);
            self.cell_char = schar_from_char(self.char_code);
            wlv.extra_todo = ::core::ffi::c_int::try_from(len - first).unwrap_or(0);
            wlv.extra_fill = NUL as ScreenChar;
            wlv.extra_last = NUL as ScreenChar;
            if self.area_attr == 0 && self.search_attr == 0 {
                wlv.n_attr = wlv.extra_todo + 1;
                wlv.extra_attr = win_hl_attr(window, HLF_8);
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
            self.overflow_attr = win_hl_attr(window, HLF_AT);
            // Steps back before the line when the character is its first;
            // the step forward below undoes it.
            self.pos = self.pos.wrapping_sub(1);
            self.did_decrement_ptr = true;
        } else if first != b'\0' {
            // Not 0: the character is not the end of the line.
            self.pos += usize::try_from(self.char_len - 1).unwrap_or(0);
        }

        if wlv.skip_cells > 0 && self.char_len > 1 && wlv.extra_todo == 0 {
            // A double-width character half scrolled off the left edge
            // shows a `<`. Not for unprintable characters, which took the
            // branch above.
            wlv.extra_todo = 1;
            wlv.extra_fill = schar_from_ascii(MB_FILLER_CHAR);
            wlv.extra_last = NUL as ScreenChar;
            self.cell_char = schar_from_ascii(b' ');
            self.char_code = ' ' as ::core::ffi::c_int;
            self.char_len = 1;
            if self.area_attr == 0 && self.search_attr == 0 {
                wlv.n_attr = wlv.extra_todo + 1;
                wlv.extra_attr = win_hl_attr(window, HLF_AT);
                self.attr_before_run = wlv.char_attr;
            }
        }
        self.pos = self.pos.wrapping_add(1);

        self.decor_attr = 0;
        if self.extra_check {
            self.slow_path(wlv, window, f, prev_at, c0);
        }

        if !vim_isprintc(self.char_code) {
            self.unprintable(wlv, window);
        }

        self.conceal(wlv, window);

        if wlv.skip_cells > 0 && self.did_decrement_ptr {
            // The `>` is not being shown, so put the pointer back or the
            // loop would not move on.
            self.pos += 1;
        }
    }

    /// Everything a character may need beyond being read: syntax and
    /// decoration attributes, spelling, `:terminal` colours, `'linebreak'` and
    /// the `'listchars'` space replacements.
    ///
    /// Reached only when the setup half decided the line needs it, which is
    /// what keeps a plain line's inner loop short. `prev_at` is where the
    /// character starts.
    pub(super) fn slow_path(
        &mut self,
        wlv: &mut WinLineVars,
        window: Win,
        f: &mut LineFrame<'_>,
        prev_at: usize,
        c0: ::core::ffi::c_int,
    ) {
        let no_plain_buffer =
            window.syn_block().b_p_spo_flags & kOptSpoFlagNoplainbuffer as uint32_t != 0;
        let mut can_spell = !no_plain_buffer;

        // Not at the start of the line only because a double-width
        // character did not fit: then there is nothing to ask about yet.
        let at = self.byte_col();
        if self.has_syntax && at > 0 {
            self.syntax_attr(window, f, at, &mut can_spell);
        }

        if self.has_decor && at > 0 {
            // Extmarks take precedence over syntax.
            self.decor_attr = hl_combine_attr(self.decor_attr, self.extmark_attr);
            self.decor_conceal = decor_conceal().0;
            // The decoration only speaks when it has an opinion.
            can_spell = decor_spell().unwrap_or(can_spell);
        }

        self.attr_base = hl_combine_attr(self.fold_attr, self.decor_attr);
        wlv.char_attr = hl_combine_attr(self.attr_base, self.attr_pri);

        self.spell_at(wlv, window, f, prev_at, can_spell);
        if self.spell_attr != 0 {
            self.attr_base = hl_combine_attr(self.attr_base, self.spell_attr);
            wlv.char_attr = hl_combine_attr(self.attr_base, self.attr_pri);
        }

        if !window.w_buffer.terminal.is_null() {
            let term_attr = usize::try_from(wlv.vcol)
                .ok()
                .and_then(|vcol| f.term_attrs.get(vcol))
                .copied()
                .unwrap_or(0);
            wlv.char_attr = hl_combine_attr(term_attr, wlv.char_attr);
        }

        self.gap_decor_attr = self.decor_attr;
        self.linebreak(wlv, window, c0);
        self.listchars(wlv, window, prev_at);
    }

    /// Ask the syntax state machine for this character's attribute, and for
    /// whether it is in the `@Spell` cluster.
    pub(super) fn syntax_attr(
        &mut self,
        mut window: Win,
        f: &LineFrame<'_>,
        at: ::core::ffi::c_int,
        can_spell: &mut bool,
    ) {
        // An error inside the syntax patterns turns highlighting off for
        // the buffer rather than being reported per character.
        let save_did_emsg = did_emsg.get();
        did_emsg.set(0);

        let has_spell = f.spv.spv_has_spell;
        let cell = get_syntax_attr(
            window.buffer(),
            at - 1,
            has_spell.then_some(can_spell),
            false,
        );
        self.decor_attr = cell.attr;

        if did_emsg.get() != 0 {
            window.syn_block_mut().b_syn_error = true;
            self.has_syntax = false;
        } else {
            did_emsg.set(save_did_emsg);
        }
        if window.syn_block().b_syn_slow {
            self.has_syntax = false;
        }
        // A multi-line regexp may have moved the memline's line, but not
        // the copy.

        // No concealing past the end of the line: it would interfere with
        // the line highlighting.
        self.syntax_flags = if self.cell_char == 0 {
            SynFlags::NONE
        } else {
            self.syntax_seqnr = cell.seqnr;
            cell.flags
        };
    }

    /// Spell-check the word starting at this character, unless the checker has
    /// already answered for it.
    ///
    /// The answer covers a whole word, so this runs once per word rather than
    /// once per cell: [`Cells::word_end`] is where the last answer reached and
    /// [`Cells::cur_checked_col`] is how far a word carried over from the
    /// previous buffer line already reached. `prev_at` is where the character
    /// starts.
    pub(super) fn spell_at(
        &mut self,
        wlv: &WinLineVars,
        window: Win,
        f: &mut LineFrame<'_>,
        prev_at: usize,
        can_spell: bool,
    ) {
        let mut at = self.byte_col();
        if !f.spv.spv_has_spell || at < self.word_end || at <= self.cur_checked_col {
            return;
        }
        self.spell_attr = 0;
        // No capital column at the end of the line, or when only white
        // space follows.
        let rest = self.line.text().get(prev_at..).unwrap_or_default();
        if self.cell_char == 0 || rest.iter().all(|&c| c == b' ' || c == b'\t') || !can_spell {
            return;
        }

        let mut spell_hlf: Hlf;
        at -= self.char_len - 1;

        // Use the look-ahead buffer where it reaches: it carries the start
        // of the next line, so a word at the end of this one is checked
        // against what follows it. `in_next` is where in it the word starts.
        let prev_col = ::core::ffi::c_int::try_from(prev_at).unwrap_or(::core::ffi::c_int::MAX);
        let in_next = usize::try_from(prev_col - self.nextlinecol).ok();
        f.spv.spv_cap_col -= prev_col;
        // The text to the end of the buffer it is in, terminator included:
        // the look-ahead buffer, or the line.
        let text = match in_next {
            Some(off) => &f.nextline[off..],
            None => self.line.from(prev_at),
        };
        let spv = &mut *f.spv;
        let tmplen;
        (tmplen, spell_hlf) =
            spell_check(window, text, Some(&mut spv.spv_cap_col), spv.spv_unchanged);
        let len = ::core::ffi::c_int::try_from(tmplen).unwrap_or(::core::ffi::c_int::MAX);
        self.word_end = at + len;

        // In Insert mode do not highlight a word the cursor is touching.
        if spell_hlf != HLF_COUNT
            && State.get() & MODE_INSERT != 0
            && window.w_cursor.lnum == wlv.lnum
            && window.w_cursor.col >= prev_col
            && window.w_cursor.col < self.word_end
        {
            spell_hlf = HLF_COUNT;
            spell_redraw_lnum.set(wlv.lnum);
        }

        // Where in the look-ahead buffer the word starts, as a column of it.
        let next_col = in_next.map(|off| ::core::ffi::c_int::try_from(off).unwrap_or(0));
        if spell_hlf == HLF_COUNT
            && let Some(next_col) = next_col
            && next_col + len > self.nextline_idx
        {
            // The good word continues at the start of the next line.
            f.spv.spv_checked_lnum = wlv.lnum + 1;
            f.spv.spv_checked_col = next_col + len - self.nextline_idx;
        }

        if spell_hlf != HLF_COUNT {
            self.spell_attr = default_hl_attr(spell_hlf as usize);
        }

        if f.spv.spv_cap_col > 0 {
            if let Some(next_col) = next_col
                && next_col + f.spv.spv_cap_col >= self.nextline_idx
            {
                // The word on the next line must start with a capital.
                f.spv.spv_capcol_lnum = wlv.lnum + 1;
                f.spv.spv_cap_col = next_col + f.spv.spv_cap_col - self.nextline_idx;
            } else {
                // Turn it back into a column of this line.
                f.spv.spv_cap_col += prev_col;
            }
        }
    }

    /// `'linebreak'`: when this character is the last blank before a word that
    /// will not fit, pad out to the end of the row so the word starts on the
    /// next one.
    pub(super) fn linebreak(&mut self, wlv: &mut WinLineVars, window: Win, c0: ::core::ffi::c_int) {
        if window.w_onebuf_opt.wo_lbr == 0 {
            return;
        }
        // Do not break a line that starts with blanks followed by a long
        // word: the break would land at the very beginning, which is not
        // what anyone means by 'linebreak'. So arm it only once a
        // character outside 'breakat' has been seen.
        if !wlv.linebreak_armed
            && self.cell_char != NUL as ScreenChar
            && !vim_isbreak(::core::ffi::c_int::from(self.byte()))
        {
            wlv.linebreak_armed = true;
        }
        // The last blank before a word: this is where the break goes.
        if !(c0 == self.char_code
            && self.char_code < 128
            && wlv.linebreak_armed
            && vim_isbreak(self.char_code)
            && !vim_isbreak(::core::ffi::c_int::from(self.byte())))
        {
            return;
        }
        // The character just read starts this far before its last byte.
        let Some(last) = self.pos.checked_sub(1) else {
            return;
        };
        let mb_off = head_off(self.line.with_nul(), last);
        let at = last - mb_off;

        // `lnum` 0: virtual text is not to be counted here.
        let chr = c_char_info(self.line.with_nul(), at).value;
        let (mut csarg, cstype) = CharsizeArg::new(window, 0, self.line.text());
        wlv.extra_todo = win_charsize(cstype, wlv.vcol, at, chr, &mut csarg).width - 1;

        // Do not bleed an underline, strikethrough or overline into the
        // filler for the pushed-down word: over blank cells it looks like a
        // broken line. A background or reverse-video highlight reads fine
        // there, so it stays. A Tab keeps its own full-width highlight, and
        // search highlighting also stops when its match ends right here.
        if self.char_code != TAB {
            if self.on_last_col || attr_has_line_deco(self.search_attr) {
                self.search_attr = 0;
            }
            if attr_has_line_deco(self.decor_attr) {
                self.decor_attr = 0;
            }
            if attr_has_line_deco(self.area_attr) {
                self.area_attr = 0;
            }
        }
        if self.char_code == TAB && wlv.extra_todo + wlv.col > self.view_width {
            wlv.extra_todo = window.w_buffer.tab_padding(wlv.vcol) - 1;
        }
        wlv.extra_fill = schar_from_ascii(if mb_off > 0 { MB_FILLER_CHAR } else { b' ' });
        wlv.extra_last = NUL as ScreenChar;
        if self.char_code < 128 && ascii_iswhite(self.char_code) {
            if self.char_code == TAB {
                // See "Tab alignment" in `unprintable`.
                wlv.fix_for_boguscols();
            }
            if window.w_onebuf_opt.wo_list == 0 {
                self.char_code = ' ' as ::core::ffi::c_int;
                self.cell_char = schar_from_ascii(b' ');
            }
        }
    }

    /// `'list'`: replace a space, a non-breaking space or a run of spaces with
    /// the `'listchars'` character that stands for it. `prev_at` is where the
    /// character starts.
    pub(super) fn listchars(&mut self, wlv: &mut WinLineVars, window: Win, prev_at: usize) {
        if window.w_onebuf_opt.wo_list != 0 {
            self.in_multispace = self.char_code == ' ' as ::core::ffi::c_int
                && (self.byte() == b' ' || (prev_at > 0 && self.line.byte(prev_at - 1) == b' '));
            if !self.in_multispace {
                self.multispace_pos = 0;
            }
        }

        let lcs = &window.w_p_lcs_chars;
        let at = self.pos as isize;

        // 'list': change 160 to "nbsp" and a space to "space" — but not
        // when a composing character follows, which `char_len` reveals.
        if window.w_onebuf_opt.wo_list != 0
            && ((((self.char_code == 160 && self.char_len == 2)
                || (self.char_code == 0x202f && self.char_len == 3))
                && lcs.nbsp != 0)
                || (self.char_code == ' ' as ::core::ffi::c_int
                    && self.char_len == 1
                    && (lcs.space != 0 || (self.in_multispace && lcs.multispace.is_some()))
                    && at >= self.leadcol as isize
                    && at <= self.trailcol as isize))
        {
            if self.in_multispace
                && let Some(run) = lcs.multispace.as_deref()
            {
                self.cell_char = cycle_run(run, &mut self.multispace_pos);
            } else {
                self.cell_char = if self.char_code == ' ' as ::core::ffi::c_int {
                    lcs.space
                } else {
                    lcs.nbsp
                };
            }
            wlv.n_attr = 1;
            wlv.extra_attr = win_hl_attr(window, HLF_0);
            self.attr_before_run = wlv.char_attr;
            self.char_code = schar_get_first_codepoint(self.cell_char);
        }

        // Leading and trailing whitespace get their own characters, and
        // this one is honoured with 'nolist' too.
        if self.char_code == ' ' as ::core::ffi::c_int
            && self.char_len == 1
            && ((self.trailcol != MAXCOL as ColNr && at > self.trailcol as isize)
                || (self.leadcol != 0 && at < self.leadcol as isize))
        {
            if self.leadcol != 0
                && self.in_multispace
                && at < self.leadcol as isize
                && let Some(run) = lcs.leadmultispace.as_deref()
            {
                self.cell_char = cycle_run(run, &mut self.multispace_pos);
            } else if at > self.trailcol as isize && lcs.trail != 0 {
                self.cell_char = lcs.trail;
            } else if at < self.leadcol as isize && lcs.lead != 0 {
                self.cell_char = lcs.lead;
            } else if self.leadcol != 0 && lcs.space != 0 {
                self.cell_char = lcs.space;
            }
            wlv.n_attr = 1;
            wlv.extra_attr = win_hl_attr(window, HLF_0);
            self.attr_before_run = wlv.char_attr;
            self.char_code = schar_get_first_codepoint(self.cell_char);
        }
    }
}

/// The character of a 'listchars' "multispace:"/"leadmultispace:" run at
/// `pos`, stepping `pos` on and back to the start past the run's end --
/// where upstream's terminator stops it.
fn cycle_run(run: &[ScreenChar], pos: &mut ::core::ffi::c_int) -> ScreenChar {
    let at = usize::try_from(*pos).unwrap_or(0);
    let c = run.get(at).copied().unwrap_or(run[0]);
    *pos += 1;
    if usize::try_from(*pos).is_ok_and(|next| next >= run.len()) {
        *pos = 0;
    }
    c
}

/// The grapheme cluster at the start of `rest` -- the line from the read
/// cursor on, its NUL included: its length in bytes (0 at the NUL), the cell
/// it makes, and its first codepoint. `utfc_ptr2len` and `utfc_ptr2schar` in
/// one, with the cell for a lone printable-or-control ASCII byte read off
/// directly: a byte under 0x80 followed by another is a cluster of one.
#[inline(always)]
fn cluster_at(rest: &[u8]) -> (::core::ffi::c_int, ScreenChar, ::core::ffi::c_int) {
    if let [b, next, ..] = *rest
        && b.wrapping_sub(1) < 0x7f
        && next < 0x80
    {
        return (1, schar_from_ascii(b), ::core::ffi::c_int::from(b));
    }
    let (len, cell, first) = c_cluster_cell(rest);
    (::core::ffi::c_int::try_from(len).unwrap_or(0), cell, first)
}
