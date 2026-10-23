//! `build_stl_str_hl()` -- the `'statusline'` format language.
//!
//! The parser and evaluator for the whole `%` alphabet (`%f`, `%l`,
//! `%{expr}`, `%(...%)` groups, `%=` separators, `%<` truncation, `%N.Mx`
//! widths, `%#Hl#` highlights, `%@Func@` click definitions), decomposed into
//! the stages the format is read in:
//!
//! | child | what |
//! | --- | --- |
//! | [`arena`] | the item stack, and the tables handed back through it |
//! | [`parse`] | the format string itself: widths, item letters, numbers |
//! | [`item`] | what one `%` item evaluates to |
//! | [`group`] | closing a `%(...%)`: elision, truncation, padding |
//! | [`fill`] | the finished line: truncation and separator spread |
//!
//! All five forbid unsafe code. What stays here is everything that talks to
//! the editor: the entry point, the window and buffer queries the item stage
//! asks ([`Env`] and the wrappers below it), and what the expansion answers.
//!
//! # Re-entrancy
//!
//! `build_stl_str_hl()` is re-entrant: a `%{}` item evaluates arbitrary Vim
//! script, which may call `nvim_eval_statusline()`, which lands back here.
//! Upstream's six function-local `static` arenas are therefore shared across
//! the recursion (see [`arena`]), and the discipline that follows for this
//! module is: **no borrow of the arenas may span an evaluation.** Every
//! stage takes `&mut StlScratch` and is called from inside one
//! [`arena::with_scratch`]; the two that evaluate ([`Env::eval`] and the
//! `%!` prologue) are called from outside every such borrow. B18-11's
//! `TvRef` and B19-6's momentary accessors are the same rule.
//!
//! # What it answers
//!
//! The text is the expander's own buffer, sized by the caller and cut at its
//! terminator; the highlight runs and click records are owned copies that
//! name their positions as byte offsets into it, so nothing the caller keeps
//! points into the arenas or the text.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use crate::normal::{set_visual_active, showcmd_buf, visual_active};
use core::cell::Cell;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use super::*;
use crate::buffer::{append_arg_number, buf_is_quickfix, buf_spname, calc_percentage, get_rel_pos};
use crate::charset::{ptr2cells_at, strsize, trans_characters_in};
use crate::decoration::SIGN_WIDTH;
use crate::digraph::keymap_str;
use crate::drawline::{fill_foldcolumn, use_cursor_line_highlight};
use crate::drawscreen::compute_foldcolumn;
use crate::drawscreen::state::{redraw_not_allowed, updating_screen};
use crate::eval::eval_to_string_safe;
use crate::eval::vars::{do_unlet, get_vim_var_nr, set_internal_string_var, set_var};
use crate::getchar::state::KeyTyped;
use crate::grid::{SCHAR_BUF_SIZE, SCharBuf, schar_copy, schar_get_adv};
use crate::highlight_group::{HLF_CLF, HLF_FC, syn_name2id_bytes};
use crate::mbyte::{c_cluster_len, char_at};
use crate::memline::ml_find_line_or_offset;
use crate::message::state::did_emsg;
use crate::option::vars::{p_sc, p_sloc};
use crate::option::{
    find_option, get_fileformat, get_option_default, set_option_direct, was_set_insecurely,
};
use crate::options::kOptInvalid;
use crate::os::cshim::{byte_toupper, gettext};
use crate::os::env::home_replace_in;
use crate::path::tail_index;
use crate::quickfix::{msg_loclist, msg_qflist};
use crate::sign::push_sign_text;
use crate::state::MODE_INSERT;
use crate::state::mode::State;
use crate::strings::{FmtArg, format_into};
use crate::types::{
    ColNr, FoldInfo, LineNr, MAXPATHL, OptIndex, ScreenChar, SignTextAttrs, StlClickRecord,
    StlHlRun, TypVal, VarNumber, Vv, int64_t,
};
use crate::undo::buf_is_changed;
use crate::winlayer::graph::{switch_buffer, switch_window};
use crate::winlayer::{Buf, Win};

// The stages of one expansion; see the module docs.
mod arena;
mod fill;
mod group;
mod item;
mod parse;

use self::arena::{
    Built, Kind, StlItem, StlScratch, collect_clicks, collect_highlights, with_scratch,
};

/// Widest a `'statuscolumn'` may get before the truncation pass bothers with
/// it: upstream's `MAX_STCWIDTH`.
const MAX_STCWIDTH: c_int = MAX_NUMBERWIDTH + SIGN_SHOW_MAX * SIGN_WIDTH as c_int + 9;

// ---------------------------------------------------------------------------
// The editor, wrapped
// ---------------------------------------------------------------------------

/// Everything one expansion reads about the editor.
///
/// The window and buffer are [`Win`]/[`Buf`], so the item stage reads their
/// fields as ordinary Rust fields; everything that calls back into the
/// editor goes through a method here, which is what lets that stage forbid
/// unsafe code.
pub(super) struct Env<'a> {
    /// The window the items describe. For `'tabline'` this is `curwin`.
    pub win: Win,
    /// Its buffer, which every item that names a file asks about.
    pub buf: Buf,
    /// The `'statuscolumn'` line, or `None` when this is not one.
    pub stc: Option<StcLine<'a>>,
    /// The option the format came from, or `kOptInvalid`.
    pub opt_idx: OptIndex,
    /// Whether `%{}` and `%!` evaluate in the sandbox.
    pub sandbox: bool,
    /// Whether the cursor line is empty, which is what makes `%c` read 0.
    pub empty_line: bool,
    /// The character under the cursor, read before anything can move it.
    pub byteval: c_int,
}

/// What a `'statuscolumn'` expansion reads about its line, and the fold
/// column's virtual columns it answers.
pub struct StcLine<'a> {
    /// The line's fold, and the line `%C` draws the fold column for.
    pub foldinfo: FoldInfo,
    pub lnum: LineNr,
    /// The cursor line's sign highlight, or 0 when this is not it.
    pub sign_cul_id: c_int,
    /// The line's signs, one per sign column; never empty.
    pub signs: &'a [SignTextAttrs],
    /// Written by `%C`, read back by the caller once the expansion is done.
    pub fold_vcol: Cell<[ColNr; 9]>,
}

impl Env<'_> {
    /// Whether this is a `'statuscolumn'` expansion.
    pub(super) fn is_statuscol(&self) -> bool {
        self.stc.is_some()
    }

    /// The `'statuscolumn'` line, on a path only a `'statuscolumn'`
    /// expansion reaches.
    fn stc(&self) -> &StcLine<'_> {
        self.stc.as_ref().expect("a 'statuscolumn' item")
    }

    /// Evaluate `expr` for a `%{}` item, with the window and buffer the
    /// items describe made current.
    ///
    /// No arena borrow may be live across this: it re-enters the evaluator,
    /// which can reach `nvim_eval_statusline()` and land back in
    /// [`build_stl_str_hl`].
    pub(super) fn eval(&self, expr: &CStr) -> Option<Vec<u8>> {
        // Upstream publishes the *real* current buffer and window under
        // `g:actual_curbuf`/`g:actual_curwin`, because the two below are
        // about to be swapped out from under the expression.
        let (real_buf, real_win) = (Buf::current(), Win::current());
        set_str_var(c"g:actual_curbuf", real_buf.handle);
        set_str_var(c"g:actual_curwin", real_win.handle);

        let save_visual = visual_active();
        // Asked before the switch, because after it the answer is always yes.
        let moved = !self.win.is_current();
        // The window and the buffer move separately: a statusline is drawn
        // for a (window, buffer) pair that need not be a window and the
        // buffer it shows.
        let saved_win = switch_window(self.win);
        let saved_buf = switch_buffer(self.buf);
        if moved {
            // Visual mode is only valid in the current window.
            set_visual_active(false);
        }
        // `expr` is the caller's own copy of the item.
        let str = eval_to_string_safe(expr.to_bytes(), self.sandbox, false);
        saved_win.restore();
        saved_buf.restore();
        set_visual_active(save_visual);

        unlet(c"g:actual_curbuf");
        unlet(c"g:actual_curwin");
        str.map(|text| text.to_vec())
    }

    /// The `%f`/`%F`/`%t` file name: the buffer's special name if it has one,
    /// otherwise its path with `$HOME` folded away, made printable and
    /// optionally cut down to its last component.
    pub(super) fn file_name(&self, full: bool, tail: bool, text: &mut Vec<u8>) {
        let room = MAXPATHL as usize;
        // Into a `MAXPATHL` buffer, where upstream's cuts the name.
        let mut buf = [0u8; MAXPATHL as usize];
        let name = match buf_spname(&self.buf) {
            Some(name) => name.to_bytes().to_vec(),
            None => {
                let path = if full {
                    self.buf.name.full()
                } else {
                    self.buf.name.shown()
                };
                path.map_or_else(Vec::new, |path| {
                    home_replace_in(Some(self.buf), path, room, true).to_vec()
                })
            }
        };
        let len = name.len().min(room - 1);
        buf[..len].copy_from_slice(&name[..len]);
        trans_characters_in(&mut buf);
        let bytes = &buf[..buf.iter().position(|&b| b == 0).unwrap_or(room)];
        let from = if tail { tail_index(bytes) } else { 0 };
        text.extend_from_slice(&bytes[from.min(bytes.len())..]);
    }

    /// The byte offset of the cursor line, for `%o`/`%O`.
    pub(super) fn line_offset(&self) -> c_int {
        ml_find_line_or_offset(self.buf, self.win.w_cursor.lnum, None, false)
    }

    /// `%p`: how far through the buffer the cursor is, as a percentage.
    pub(super) fn percentage(&self) -> c_int {
        calc_percentage(
            self.win.w_cursor.lnum as int64_t,
            self.buf.b_ml.ml_line_count as int64_t,
        )
    }

    /// `%P`: the same, but as `Top`/`Bot`/`All` when it has a name.
    pub(super) fn rel_pos(&self, text: &mut Vec<u8>) {
        text.extend_from_slice(&get_rel_pos(self.win, TMPLEN as usize));
    }

    /// `%a`: the argument list position, when there is one.
    pub(super) fn arg_number(&self, text: &mut Vec<u8>) {
        text.extend_from_slice(&append_arg_number(self.win, TMPLEN as usize));
    }

    /// `%k`: the active `'keymap'`, in angle brackets.
    pub(super) fn keymap(&self, text: &mut Vec<u8>) {
        let Some(name) = keymap_str(self.win) else {
            return;
        };
        let name = name.as_bytes();
        // Upstream formats through a `TMPLEN` buffer and drops the item when
        // the name does not fit, brackets included.
        if name.len() + 2 < TMPLEN as usize {
            text.push(b'<');
            text.extend_from_slice(name);
            text.push(b'>');
        }
    }

    /// `%q`: the quickfix or location list title, when this is one.
    pub(super) fn quickfix_title(&self, text: &mut Vec<u8>) {
        if !buf_is_quickfix(Some(self.buf)) {
            return;
        }
        let msg = if self.win.w_llist_ref.is_none() {
            msg_qflist
        } else {
            msg_loclist
        };
        text.extend_from_slice(gettext(msg).to_bytes());
    }

    /// `%m`/`%M`: whether the buffer has unsaved changes.
    pub(super) fn is_changed(&self) -> bool {
        buf_is_changed(self.buf)
    }

    /// `%b`/`%B`: the character under the cursor, with the line ending the
    /// file format actually uses.
    pub(super) fn byte_value(&self) -> c_int {
        let num = self.byteval;
        if num == NL {
            0
        } else if num == CAR && self.fileformat() == EOL_MAC {
            NL
        } else {
            num
        }
    }

    /// The buffer's `'fileformat'`, resolved.
    fn fileformat(&self) -> c_int {
        get_fileformat(self.buf)
    }

    /// `%S`: `'showcmd'`'s pending keys, but only in the option
    /// `'showcmdloc'` names.
    pub(super) fn showcmd(&self, text: &mut Vec<u8>) {
        if !p_sc() {
            return;
        }
        if self.opt_idx as c_int != kOptInvalid as c_int {
            let loc = p_sloc(find_option);
            if loc as c_int != self.opt_idx as c_int {
                return;
            }
        }
        text.extend_from_slice(showcmd_buf.get().as_cstr().to_bytes());
    }

    /// How wide the fold column is here, which is what `%C` draws.
    pub(super) fn fold_column_width(&self) -> c_int {
        compute_foldcolumn(self.win, 0)
    }

    /// Draw the fold column's `fdc` glyphs into `text`, answering the
    /// highlight id they draw in.
    pub(super) fn fold_glyphs(&self, fdc: c_int, text: &mut Vec<u8>) -> c_int {
        let mut glyphs = [0 as ScreenChar; 9];
        // The line the fold item describes is `v:lnum`, not the cursor line.
        let lnum = vim_var(Vv::Lnum) as LineNr;
        let stc = self.stc();
        let mut fold_vcol = stc.fold_vcol.get();
        fill_foldcolumn(
            self.win,
            stc.foldinfo,
            stc.lnum,
            fdc,
            vim_var(Vv::Virtnum) < 0,
            &mut fold_vcol,
            &mut glyphs,
        );
        stc.fold_vcol.set(fold_vcol);
        let mut buf = [0u8; TMPLEN as usize];
        let mut len = 0usize;
        for &glyph in &glyphs[..fdc as usize] {
            len += put_schar(&mut buf, len, glyph);
        }
        text.extend_from_slice(&buf[..len]);
        let cul = use_cursor_line_highlight(self.win, lnum);
        -if cul { HLF_CLF } else { HLF_FC }
    }

    /// The buffer's `'filetype'`, which `%y` and `%Y` bracket.
    pub(super) fn with_filetype<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        // The borrow ends before anything can `:set` it.
        f(self.buf.b_p_ft.bytes())
    }

    /// The sign in column `i`, and the highlight id it draws in.
    ///
    /// Answers `None` where there is no sign, which is drawn as two blanks
    /// in the default highlight.
    pub(super) fn sign_text(&self, i: usize, text: &mut Vec<u8>) -> Option<c_int> {
        // The caller's loop is bounded by `w_scwidth`, which the signs
        // cover.
        let stc = self.stc();
        let sattr = stc.signs[i];
        if sattr.text[0] == 0 || vim_var(Vv::Virtnum) != 0 {
            text.extend_from_slice(b"  ");
            return None;
        }
        push_sign_text(&sattr.text, text);
        let cul = stc.sign_cul_id;
        Some(-if cul != 0 { cul } else { sattr.hl_id })
    }

    /// Whether the sign column is the number column and already has a sign
    /// in it, which is what makes `%l` draw the sign instead.
    pub(super) fn number_column_has_sign(&self) -> bool {
        self.stc().signs[0].text[0] != 0
    }
}

/// Set `name` to the number `value` as a string, the way `%{}` publishes the
/// real current buffer and window.
fn set_str_var(name: &CStr, value: c_int) {
    set_internal_string_var(name.to_bytes(), Some(value.to_string().as_bytes()));
}

/// `:unlet!` a variable this expansion published.
fn unlet(name: &CStr) {
    let _ = do_unlet(name.to_bytes(), true);
}

/// `v:lnum`, `v:relnum` and `v:virtnum`, which `'statuscolumn'` items read.
pub(super) fn vim_var(idx: Vv) -> VarNumber {
    get_vim_var_nr(idx)
}

/// Whether the editor is in Insert mode, which `%c`/`%o` ask about.
pub(super) fn in_insert_mode() -> bool {
    State.get() & MODE_INSERT != 0
}

/// The syntax group `name` names, for `%#name#` and `%$name$`.
pub(super) fn syntax_id(name: &[u8]) -> c_int {
    syn_name2id_bytes(name)
}

/// A translated message, as bytes.
pub(super) fn tr(msg: &'static CStr) -> &'static [u8] {
    gettext(msg).to_bytes()
}

/// `toupper()` in the current locale, which is what `%Y` upper-cases with.
pub(super) fn upper(byte: u8) -> u8 {
    byte_toupper(c_int::from(byte)) as u8
}

/// The number `text` spells, when it spells one at all.
///
/// A `%{}` result made only of digits becomes a number item, so that the
/// item's width and zero-padding apply to it.
pub(super) fn as_number(text: &[u8]) -> Option<c_int> {
    if text.is_empty() || !text.iter().all(u8::is_ascii_digit) {
        return None;
    }
    // `atoi`, which is glibc's `(int) strtol`: the digits saturate at a
    // `long`, and the `long` is then cut to an `int`.
    let value = text.iter().fold(0i64, |acc, &digit| {
        acc.saturating_mul(10)
            .saturating_add(i64::from(digit - b'0'))
    });
    Some(value as c_int)
}

/// A copy of `bytes` as a C string, cut at a NUL as upstream's `xmemdupz`
/// copy reads: what a `%@Func@` item hands to the caller's click table.
pub(super) fn dup_cstring(bytes: &[u8]) -> CString {
    let name = crate::cstr::until_nul(bytes);
    CString::new(name).expect("cut at the first NUL")
}

// ---------------------------------------------------------------------------
// The output buffer, wrapped
// ---------------------------------------------------------------------------

/// How many screen cells the string at `at` takes, up to its terminator.
pub(super) fn strsize_at(out: &[u8], at: usize) -> c_int {
    strsize(&out[at..])
}

/// How many screen cells the *character* at `at` takes.
pub(super) fn cells_at(out: &[u8], at: usize) -> c_int {
    ptr2cells_at(&out[at..])
}

/// How many bytes the character at `at` takes, combining marks included.
pub(super) fn char_len_at(out: &[u8], at: usize) -> usize {
    c_cluster_len(&out[at..])
}

/// The character an expansion pads with, resolved to bytes once.
///
/// Upstream re-derives it from the glyph cache for every cell it writes --
/// `schar_get_adv` is a cache lookup, a `strlen` and a `memcpy` per cell,
/// and a padded line is nothing but such cells. Resolving it once turns
/// padding into a short `copy_from_slice`.
pub(super) struct Fill {
    /// The glyph itself, for the two arms that compare it to `-`.
    schar: ScreenChar,
    bytes: SCharBuf,
    len: usize,
}

impl Fill {
    /// Resolve `schar`, defaulting a zero to the blank upstream uses.
    fn of(schar: ScreenChar) -> Self {
        let schar = if schar == 0 {
            b' ' as ScreenChar
        } else {
            schar
        };
        let mut bytes: SCharBuf = [0; SCHAR_BUF_SIZE];
        let len = schar_copy(&mut bytes, schar);
        Fill { schar, bytes, len }
    }

    /// How many bytes one of it takes.
    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// Whether it is the `-` that must not be put in front of a digit.
    pub(super) fn is_dash(&self) -> bool {
        self.schar == b'-' as ScreenChar
    }

    /// Write one at `at`, answering where the next byte goes.
    ///
    /// The slice bound is real: upstream guards only that `at` is before the
    /// last byte of the buffer, so a fill character several bytes wide could
    /// write past its end. Here that panics instead.
    pub(super) fn put(&self, out: &mut [u8], at: usize) -> usize {
        out[at..at + self.len].copy_from_slice(&self.bytes[..self.len]);
        at + self.len
    }
}

/// Write the glyph `sc` into `buf` at `at`, answering its byte length.
///
/// # Panics
/// When the glyph does not fit what is left of `buf`; upstream writes past
/// the end.
fn put_schar(buf: &mut [u8], at: usize, sc: ScreenChar) -> usize {
    let mut dst = &mut buf[at..];
    schar_get_adv(&mut dst, sc)
}

/// Print a number item through `template` at `at`, answering where the next
/// byte goes.
///
/// `args` is what the template's `*` and conversions consume: a width and a
/// value, or a value and an exponent for the reduced form.
fn put_number(out: &mut [u8], at: usize, plan: &parse::NumPlan) -> usize {
    let room = out.len() - at;
    if room == 0 {
        return at;
    }
    let template = plan.template().to_bytes();
    // [`parse::NumPlan`] built the template to take exactly these two.
    let args = match plan.exp {
        Some(exp) => [FmtArg::from(0), FmtArg::from(plan.num), FmtArg::from(exp)],
        None => [
            FmtArg::from(plan.width),
            FmtArg::from(plan.num),
            FmtArg::from(0),
        ],
    };
    let len = format_into(
        &mut out[at..],
        template,
        &args[..2 + usize::from(plan.exp.is_some())],
    );
    // `vim_snprintf_safelen`: never past the last byte, the terminator's.
    at + len.min(room - 1)
}

// ---------------------------------------------------------------------------
// The entry point
// ---------------------------------------------------------------------------

/// Where a format came from: the option that holds it, and the scope that
/// option was read in.
///
/// The pair decides two things and is never split: whether the expansion
/// runs in the sandbox (a `'statusline'` set from a modeline is not
/// trusted), and which option to reset to its default when expanding it
/// provokes an error.
pub struct FmtSource {
    pub opt_idx: OptIndex,
    pub opt_scope: OptionSetFlags,
}

impl FmtSource {
    /// A format that is nobody's option — which is what switches the
    /// sandbox off, so `nvim_eval_statusline()` can never reach it.
    pub const NONE: FmtSource = FmtSource {
        opt_idx: kOptInvalid,
        opt_scope: OptionSetFlags::NONE,
    };
}

/// What [`build_stl_str_hl`] is asked to record on the way, beyond the text
/// itself.
pub struct StlWants<'a> {
    /// The highlight runs, one per group change.
    pub hl: bool,
    /// The `%@Func@` and tab page click records.
    pub clicks: bool,
    /// The `'statuscolumn'` line, when that is what is being built.
    pub stc: Option<StcLine<'a>>,
}

impl StlWants<'_> {
    /// A caller that wants the text and nothing else.
    pub const NONE: StlWants<'static> = StlWants {
        hl: false,
        clicks: false,
        stc: None,
    };
}

/// What one expansion answered.
pub struct StlOutput {
    /// The text, without a terminator.
    pub text: Vec<u8>,
    /// Its width in screen cells.
    pub width: c_int,
    /// The highlight runs, when they were asked for. A run's `start` is a
    /// byte of the expander's buffer, which can be past the end of the text
    /// (the truncation pass records one at the cut).
    pub hl: Vec<StlHlRun>,
    /// How many *items* this expansion wrote, which is what
    /// `nvim_eval_statusline()` sizes its answer by -- upstream's
    /// `hltab_len`, not the number of runs.
    pub item_count: usize,
    /// The click records, when they were asked for.
    pub clicks: Vec<StlClickRecord>,
    /// The fold column's virtual columns a `'statuscolumn'` `%C` wrote.
    pub fold_vcol: Option<[ColNr; 9]>,
}

/// [`build_stl_str_hl`] for a caller that wants the text and nothing else it
/// can report -- no fill character, no highlight or click records -- from a
/// format it holds: `'titlestring'` and `'iconstring'`.
pub(crate) fn build_stl_text(
    window: Win,
    room: usize,
    fmt: Vec<u8>,
    from: FmtSource,
    maxwidth: c_int,
) -> Vec<u8> {
    build_stl_str_hl(window, room, fmt, from, 0, maxwidth, StlWants::NONE).text
}

/// Build a string from the status line items in `fmt`, the expansion working
/// in a `room`-byte buffer (whose size decides where the text is cut and
/// whether the separators spread).
///
/// Normally works for window `window`, except when working for `'tabline'`,
/// when it is `curwin`. `fmt` is the caller's own copy: expanding it can run
/// a function that `:set`s the option it came from. This re-enters the
/// editor, so nothing may be held across it.
pub fn build_stl_str_hl(
    window: Win,
    room: usize,
    fmt: Vec<u8>,
    from: FmtSource,
    fillchar: ScreenChar,
    maxwidth: c_int,
    wants: StlWants<'_>,
) -> StlOutput {
    let StlWants { hl, clicks, stc } = wants;
    let FmtSource { opt_idx, opt_scope } = from;
    let mut out = vec![0u8; room];
    let win = window;
    let save_redraw_not_allowed = redraw_not_allowed.get();
    let save_key_typed = KeyTyped.get();
    let did_emsg_before = did_emsg.get();

    // Inside update_screen() we do not want redrawing a statusline, ruler or
    // title to trigger another redraw; it may loop forever.
    if updating_screen.get() {
        redraw_not_allowed.set(true);
    }

    // A format set insecurely is evaluated in the sandbox. `opt_idx` is
    // `kOptInvalid` when the caller is nvim_eval_statusline(), which is
    // therefore never sandboxed.
    let sandbox = opt_idx as c_int != kOptInvalid as c_int && {
        was_set_insecurely(window, opt_idx, opt_scope)
    };

    // The format is read as the C string it stands for: to its first NUL.
    let mut fmt_bytes = fmt;
    fmt_bytes.truncate(crate::cstr::until_nul(&fmt_bytes).len());
    // A format starting with "%!" is itself an expression, whose result is
    // the format actually used. Evaluating it can fail, in which case the
    // literal text is what gets rendered.
    let usefmt = if fmt_bytes.starts_with(b"%!") {
        let mut winid = TypVal::Number(win.handle as VarNumber);
        let name = c"g:statusline_winid";
        set_var(name.to_bytes(), &mut winid, false);
        let expanded = eval_to_string_safe(&fmt_bytes[2..], sandbox, false);
        unlet(name);
        expanded.map_or(fmt_bytes, |text| text.to_vec())
    } else {
        fmt_bytes
    };

    let fill = Fill::of(fillchar);

    // The cursor in windows other than the current one is not always
    // up to date, because of autocommands and timers.
    let mut win = win;
    let buf = win.buffer();
    let lnum = win.w_cursor.lnum.min(buf.b_ml.ml_line_count);
    win.w_cursor.lnum = lnum;

    // Read the byte under the cursor now, in case an item needs it.
    let mut lines = buf.lines();
    let line = lines.line(lnum);
    let empty_line = line.is_empty();
    let len = line.len() as ColNr;
    let byteval = if win.w_cursor.col > len {
        // The line may have changed since the cursor column was checked, or
        // the line number was adjusted above.
        win.w_cursor.col = len;
        win.w_cursor.coladd = 0 as ColNr;
        0
    } else {
        char_at(&line[win.w_cursor.col as usize..])
    };

    let env = Env {
        win,
        buf,
        stc,
        opt_idx,
        sandbox,
        empty_line,
        byteval,
    };
    let built = expand(&env, &mut out, usefmt, &fill, maxwidth, !clicks);

    // Hand back the highlight runs and the click records, which name their
    // places in the expander's buffer.
    let (hl, clicks) = with_scratch(|s| {
        let runs = if hl {
            collect_highlights(s, room, &built)
        } else {
            Vec::new()
        };
        let recs = if clicks {
            collect_clicks(s, room, &built)
        } else {
            Vec::new()
        };
        if built.evalstart == 0 {
            // The outermost expansion: no item refers to a name any more.
            s.clear_cmds();
        }
        (runs, recs)
    });

    redraw_not_allowed.set(save_redraw_not_allowed);

    // An error here would mess up the display and might loop redrawing;
    // avoid that by setting the option back to its default.
    if opt_idx as c_int != kOptInvalid as c_int && did_emsg.get() > did_emsg_before {
        let default = get_option_default(opt_idx, opt_scope);
        set_option_direct(opt_idx, default, opt_scope, SID_ERROR);
    }

    // A user function may reset KeyTyped; restore it.
    KeyTyped.set(save_key_typed);
    out.truncate(crate::cstr::until_nul(&out).len());
    StlOutput {
        text: out,
        width: built.width,
        hl,
        item_count: built.itemcnt,
        clicks,
        fold_vcol: env.stc.map(|stc| stc.fold_vcol.get()),
    }
}

/// Expand `usefmt` into `out`: the item loop, then the post-processing that
/// makes the result exactly `maxwidth` cells wide.
fn expand(
    env: &Env,
    out: &mut [u8],
    usefmt: Vec<u8>,
    fill: &Fill,
    maxwidth: c_int,
    discard_clicks: bool,
) -> Built {
    // The arenas exist from the first expansion on, so the tables handed
    // back below always have room for at least their terminator.
    let evalstart = with_scratch(|s| {
        s.grow();
        s.curitem
    });
    let pos = item::run(env, out, usefmt, fill, discard_clicks);
    out[pos] = 0;
    // Bytes of `out` used, excluding the NUL. Taken before post-processing
    // moves the text around.
    let outputlen = pos;

    let mut built = with_scratch(|s| {
        let itemcnt = s.curitem - evalstart;
        s.curitem = evalstart;
        Built {
            width: 0,
            evalstart,
            itemcnt,
        }
    });
    built.width = strsize_at(out, 0);

    with_scratch(|s| {
        let too_long = maxwidth > 0
            && built.width > maxwidth
            && (!env.is_statuscol() || built.width > MAX_STCWIDTH);
        if too_long {
            fill::truncate(s, out, &mut built, outputlen, maxwidth, fill);
        } else if built.width < maxwidth
            && outputlen + (maxwidth - built.width) as usize * fill.len() + 1 < out.len()
        {
            fill::spread(s, out, &mut built, maxwidth, fill);
        }
    });
    built
}
