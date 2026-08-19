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
//! asks ([`Env`] and the wrappers below it), and the writes through the four
//! out-parameters the caller passed.
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
//! # The out-parameters
//!
//! `hltab` and `tabtab` answer *raw pointers into two of the arenas*, and the
//! caller reads through them after the call returns -- `'statuscolumn'`
//! stores its `hlrec` in the `statuscol_T` and reads it again while drawing.
//! The arenas may therefore not be reallocated while such a pointer is live.
//! They only ever grow, and only when an item is recorded (see
//! [`arena::StlScratch::grow`]), which is exactly upstream's `xrealloc`
//! window; a
//! caller that holds a table across a *later* expansion has the same dangling
//! read it has in C.
//!
//! Original: `src/nvim/statusline.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{ptr, slice};
use std::ffi::CString;

use super::*;
use crate::buffer::{append_arg_number, bt_quickfix, buf_spname, calc_percentage, get_rel_pos};
use crate::charset::{ptr2cells, trans_characters, vim_strsize};
use crate::decoration::SIGN_WIDTH;
use crate::digraph::keymap_str;
use crate::drawline::{fill_foldcolumn, use_cursor_line_highlight};
use crate::drawscreen::compute_foldcolumn;
use crate::eval::eval_to_string_safe;
use crate::eval::vars::{do_unlet, get_vim_var_nr, set_internal_string_var, set_var};
use crate::grid::{MAX_SCHAR_SIZE, schar_get_adv};
use crate::highlight_group::{HLF_CLF, HLF_FC, syn_name2id_len};
use crate::main::{
    KeyTyped, State, VIsual_active, curbuf, curwin, did_emsg, msg_loclist, msg_qflist, p_sc,
    p_sloc, redraw_not_allowed, showcmd_buf, updating_screen,
};
use crate::mbyte::{utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_find_line_or_offset, ml_get_buf_len};
use crate::memory::{xfree, xmemdupz, xstrlcpy};
use crate::option::{
    find_option, get_fileformat, get_option_default, set_option_direct, was_set_insecurely,
};
use crate::options::kOptInvalid;
use crate::os::cshim::gettext;
use crate::os::env::home_replace;
use crate::path::path_tail;
use crate::sign::describe_sign_text;
use crate::state::MODE_INSERT;
use crate::strings::vim_snprintf_safelen;
use crate::types::{
    MAXPATHL, OptIndex, StlClickRecord, VAR_NUMBER, VAR_UNLOCKED, VV_LNUM, VV_VIRTNUM, VimVarIndex,
    colnr_T, int64_t, linenr_T, schar_T, size_t, statuscol_T, stl_hlrec_t, typval_T,
    typval_vval_union, varnumber_T, win_T,
};
use crate::undo::bufIsChanged;
use crate::winlayer::{Buf, Win};
use ::libc::{atoi, toupper};

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
pub(super) struct Env {
    /// The window the items describe. For `'tabline'` this is `curwin`.
    pub win: Win,
    /// Its buffer, which every item that names a file asks about.
    pub buf: Buf,
    /// The `'statuscolumn'` state, or null when this is not one.
    pub stcp: *mut statuscol_T,
    /// The option the format came from, or `kOptInvalid`.
    pub opt_idx: OptIndex,
    /// Whether `%{}` and `%!` evaluate in the sandbox.
    pub sandbox: bool,
    /// Whether the cursor line is empty, which is what makes `%c` read 0.
    pub empty_line: bool,
    /// The character under the cursor, read before anything can move it.
    pub byteval: c_int,
}

impl Env {
    /// Whether this is a `'statuscolumn'` expansion.
    pub fn is_statuscol(&self) -> bool {
        !self.stcp.is_null()
    }

    /// Evaluate `expr` for a `%{}` item, with the window and buffer the
    /// items describe made current.
    ///
    /// No arena borrow may be live across this: it re-enters the evaluator,
    /// which can reach `nvim_eval_statusline()` and land back in
    /// [`build_stl_str_hl`].
    pub fn eval(&self, expr: &CStr) -> Option<Vec<u8>> {
        // Upstream publishes the *real* current buffer and window under
        // `g:actual_curbuf`/`g:actual_curwin`, because the two below are
        // about to be swapped out from under the expression.
        // SAFETY: `curbuf`/`curwin` are set from startup to exit.
        let (real_buf, real_win) = unsafe { (Buf::current(), Win::current()) };
        set_str_var(c"g:actual_curbuf", real_buf.handle);
        set_str_var(c"g:actual_curwin", real_win.handle);

        let (save_curbuf, save_curwin) = (curbuf.get(), curwin.get());
        let save_visual = VIsual_active.get();
        curwin.set(self.win.raw());
        curbuf.set(self.buf.raw());
        if curwin.get() != save_curwin {
            // Visual mode is only valid in the current window.
            VIsual_active.set(false);
        }
        // SAFETY: `expr` is NUL-terminated, and the result is a string this
        // frame owns.
        let str = unsafe { eval_to_string_safe(expr.as_ptr().cast_mut(), self.sandbox, false) };
        curwin.set(save_curwin);
        curbuf.set(save_curbuf);
        VIsual_active.set(save_visual);

        unlet(c"g:actual_curbuf");
        unlet(c"g:actual_curwin");
        take_cstring(str)
    }

    /// The `%f`/`%F`/`%t` file name: the buffer's special name if it has one,
    /// otherwise its path with `$HOME` folded away, made printable and
    /// optionally cut down to its last component.
    pub fn file_name(&self, full: bool, tail: bool, text: &mut Vec<u8>) {
        // SAFETY: a live buffer; `buf_spname` answers a string or null.
        let name = unsafe { buf_spname(self.buf.raw()) };
        with_name_buff(|nb| {
            if name.is_null() {
                let path = if full {
                    self.buf.b_ffname
                } else {
                    self.buf.b_fname
                };
                // SAFETY: `NameBuff` is `MAXPATHL` bytes, which is the size
                // both of these are told about.
                unsafe {
                    home_replace(
                        self.buf.raw(),
                        path,
                        nb.as_mut_ptr(),
                        MAXPATHL as size_t,
                        true,
                    )
                };
            } else {
                // SAFETY: as above; `name` is NUL-terminated.
                unsafe { xstrlcpy(nb.as_mut_ptr(), name, MAXPATHL as size_t) };
            }
            // SAFETY: as above.
            unsafe { trans_characters(nb.as_mut_ptr(), MAXPATHL) };
        });
        with_name_buff(|nb| {
            let bytes = as_cstr(nb).to_bytes();
            let from = if tail {
                // SAFETY: `NameBuff` was just NUL-terminated by the fill
                // above; `path_tail` answers a position inside it.
                let p = unsafe { path_tail(nb.as_ptr()) };
                (p as usize).saturating_sub(nb.as_ptr() as usize)
            } else {
                0
            };
            text.extend_from_slice(&bytes[from.min(bytes.len())..]);
        });
    }

    /// The byte offset of the cursor line, for `%o`/`%O`.
    pub fn line_offset(&self) -> c_int {
        // SAFETY: a live buffer and its own cursor line.
        unsafe {
            ml_find_line_or_offset(
                self.buf.raw(),
                self.win.w_cursor.lnum,
                ptr::null_mut(),
                false,
            )
        }
    }

    /// `%p`: how far through the buffer the cursor is, as a percentage.
    pub fn percentage(&self) -> c_int {
        calc_percentage(
            self.win.w_cursor.lnum as int64_t,
            self.buf.b_ml.ml_line_count as int64_t,
        )
    }

    /// `%P`: the same, but as `Top`/`Bot`/`All` when it has a name.
    pub fn rel_pos(&self, text: &mut Vec<u8>) {
        let mut buf = [0u8; TMPLEN as usize];
        // SAFETY: the buffer is this frame's, and its length is what
        // `get_rel_pos` is told.
        unsafe { get_rel_pos(self.win.raw(), buf.as_mut_ptr().cast::<c_char>(), TMPLEN) };
        text.extend_from_slice(cstr_at(&buf, 0).to_bytes());
    }

    /// `%a`: the argument list position, when there is one.
    pub fn arg_number(&self, text: &mut Vec<u8>) {
        let mut buf = [0u8; TMPLEN as usize];
        // SAFETY: as [`Env::rel_pos`]. The buffer starts empty because
        // `append_arg_number` appends to what is already there.
        let len = unsafe {
            append_arg_number(
                self.win.raw(),
                buf.as_mut_ptr().cast::<c_char>(),
                TMPLEN as usize,
            )
        };
        if len > 0 {
            text.extend_from_slice(cstr_at(&buf, 0).to_bytes());
        }
    }

    /// `%k`: the active `'keymap'`, in angle brackets.
    pub fn keymap(&self, text: &mut Vec<u8>) {
        // SAFETY: a live window.
        let Some(name) = (unsafe { keymap_str(self.win.raw()) }) else {
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
    pub fn quickfix_title(&self, text: &mut Vec<u8>) {
        // SAFETY: a live buffer.
        if !unsafe { bt_quickfix(self.buf.raw()) } {
            return;
        }
        let msg = if self.win.w_llist_ref.is_null() {
            msg_qflist.get()
        } else {
            msg_loclist.get()
        };
        // SAFETY: both globals hold a NUL-terminated message, and `gettext`
        // answers one for it.
        text.extend_from_slice(unsafe { CStr::from_ptr(gettext(msg)) }.to_bytes());
    }

    /// `%m`/`%M`: whether the buffer has unsaved changes.
    pub fn is_changed(&self) -> bool {
        // SAFETY: a live buffer.
        unsafe { bufIsChanged(self.buf.raw()) }
    }

    /// `%b`/`%B`: the character under the cursor, with the line ending the
    /// file format actually uses.
    pub fn byte_value(&self) -> c_int {
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
        // SAFETY: a live buffer.
        unsafe { get_fileformat(self.buf.raw()) }
    }

    /// `%S`: `'showcmd'`'s pending keys, but only in the option
    /// `'showcmdloc'` names.
    pub fn showcmd(&self, text: &mut Vec<u8>) {
        if p_sc.get() == 0 {
            return;
        }
        if self.opt_idx as c_int != kOptInvalid as c_int {
            // SAFETY: `p_sloc` holds a NUL-terminated option value.
            let loc = unsafe { find_option(p_sloc.get()) };
            if loc as c_int != self.opt_idx as c_int {
                return;
            }
        }
        showcmd_buf.with(|buf| text.extend_from_slice(as_cstr(buf).to_bytes()));
    }

    /// How wide the fold column is here, which is what `%C` draws.
    pub fn fold_column_width(&self) -> c_int {
        // SAFETY: a live window.
        unsafe { compute_foldcolumn(self.win.raw(), 0) }
    }

    /// Draw the fold column's `fdc` glyphs into `text`, answering the
    /// highlight id they draw in.
    pub fn fold_glyphs(&self, fdc: c_int, text: &mut Vec<u8>) -> c_int {
        let mut glyphs = [0 as schar_T; 9];
        // The line the fold item describes is `v:lnum`, not the cursor line.
        let lnum = vim_var(VV_LNUM) as linenr_T;
        // SAFETY: `stcp` is non-null on every path that reaches a fold item,
        // `glyphs` is this frame's, and `fdc` is what `compute_foldcolumn`
        // just answered.
        unsafe {
            fill_foldcolumn(
                self.win.raw(),
                (*self.stcp).foldinfo,
                (*self.stcp).lnum,
                fdc,
                vim_var(VV_VIRTNUM) < 0,
                &raw mut (*self.stcp).fold_vcol as *mut colnr_T,
                glyphs.as_mut_ptr(),
            );
        }
        let mut buf = [0u8; TMPLEN as usize];
        let mut len = 0usize;
        for &glyph in &glyphs[..fdc as usize] {
            len += put_schar(&mut buf, len, glyph);
        }
        text.extend_from_slice(&buf[..len]);
        // SAFETY: a live window and the line the fold describes.
        let cul = unsafe { use_cursor_line_highlight(self.win.raw(), lnum) };
        -if cul { HLF_CLF } else { HLF_FC }
    }

    /// The buffer's `'filetype'`, which `%y` and `%Y` bracket.
    pub fn with_filetype<R>(&self, f: impl FnOnce(&[u8]) -> R) -> R {
        // SAFETY: a string option always holds a NUL-terminated string, and
        // the borrow ends before anything can `:set` it.
        f(unsafe { CStr::from_ptr(self.buf.b_p_ft) }.to_bytes())
    }

    /// The sign in column `i`, and the highlight id it draws in.
    ///
    /// Answers `None` where there is no sign, which is drawn as two blanks
    /// in the default highlight.
    pub fn sign_text(&self, i: usize, text: &mut Vec<u8>) -> Option<c_int> {
        // SAFETY: `stcp` is non-null here, and `sattrs` holds at least
        // `w_scwidth` entries -- which is the loop bound the caller uses.
        let mut sattr = unsafe { *(*self.stcp).sattrs.add(i) };
        if sattr.text[0] == 0 || vim_var(VV_VIRTNUM) != 0 {
            text.extend_from_slice(b"  ");
            return None;
        }
        let mut buf = [0u8; TMPLEN as usize];
        // SAFETY: `buf` is this frame's and far longer than a sign text;
        // `sattr` is a copy this frame owns.
        let len = unsafe {
            describe_sign_text(
                buf.as_mut_ptr().cast::<c_char>(),
                &raw mut sattr.text as *mut schar_T,
            )
        };
        text.extend_from_slice(&buf[..(len as usize).min(buf.len())]);
        // SAFETY: `stcp` is non-null here.
        let cul = unsafe { (*self.stcp).sign_cul_id };
        Some(-if cul != 0 { cul } else { sattr.hl_id })
    }

    /// Whether the sign column is the number column and already has a sign
    /// in it, which is what makes `%l` draw the sign instead.
    pub fn number_column_has_sign(&self) -> bool {
        // SAFETY: `stcp` is non-null here, and `sattrs` is never empty.
        unsafe { (*(*self.stcp).sattrs).text[0] != 0 }
    }
}

/// Set `name` to the number `value` as a string, the way `%{}` publishes the
/// real current buffer and window.
fn set_str_var(name: &CStr, value: c_int) {
    let mut text = itoa(value);
    // SAFETY: both arguments are NUL-terminated strings this frame owns, and
    // `set_internal_string_var` copies the value.
    unsafe { set_internal_string_var(name.as_ptr(), text.as_mut_ptr()) };
}

/// `value` in decimal, NUL-terminated.
fn itoa(value: c_int) -> [c_char; 12] {
    let mut out = [0 as c_char; 12];
    let text = value.to_string();
    for (slot, byte) in out.iter_mut().zip(text.bytes()) {
        *slot = byte as c_char;
    }
    out
}

/// `:unlet!` a variable this expansion published.
fn unlet(name: &CStr) {
    // SAFETY: `name` is a NUL-terminated string with its own length.
    unsafe { do_unlet(name.as_ptr(), name.to_bytes().len() as size_t, true) };
}

/// Take ownership of an `xmalloc`ed C string, as a byte vector.
fn take_cstring(str: *mut c_char) -> Option<Vec<u8>> {
    if str.is_null() {
        return None;
    }
    // SAFETY: the callers below own the only reference to a NUL-terminated
    // string the editor allocated, and free it exactly once here.
    let bytes = unsafe { CStr::from_ptr(str) }.to_bytes().to_vec();
    // SAFETY: as above.
    unsafe { xfree(str.cast::<c_void>()) };
    Some(bytes)
}

/// `v:lnum`, `v:relnum` and `v:virtnum`, which `'statuscolumn'` items read.
pub(super) fn vim_var(idx: VimVarIndex) -> varnumber_T {
    // SAFETY: the index is one of the compile-time `VV_*` constants.
    unsafe { get_vim_var_nr(idx) }
}

/// Whether the editor is in Insert mode, which `%c`/`%o` ask about.
pub(super) fn in_insert_mode() -> bool {
    State.get() & MODE_INSERT != 0
}

/// The syntax group `name` names, for `%#name#` and `%$name$`.
pub(super) fn syntax_id(name: &[u8]) -> c_int {
    // SAFETY: the name is a run of the format string, with its own length.
    unsafe { syn_name2id_len(name.as_ptr().cast::<c_char>(), name.len() as size_t) }
}

/// A translated message, as bytes.
pub(super) fn tr(msg: &CStr) -> &'static [u8] {
    // SAFETY: `gettext` answers either its argument or a string owned by the
    // message catalogue; both outlive the expansion.
    unsafe { CStr::from_ptr(gettext(msg.as_ptr())) }.to_bytes()
}

/// `toupper()` in the current locale, which is what `%Y` upper-cases with.
pub(super) fn upper(byte: u8) -> u8 {
    // SAFETY: a plain `ctype` call on one byte.
    unsafe { toupper(c_int::from(byte)) as u8 }
}

/// The number `text` spells, when it spells one at all.
///
/// A `%{}` result made only of digits becomes a number item, so that the
/// item's width and zero-padding apply to it.
pub(super) fn as_number(text: &[u8]) -> Option<c_int> {
    if text.is_empty() || !text.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let cstr = CString::new(text).ok()?;
    // SAFETY: a NUL-terminated string this frame owns.
    Some(unsafe { atoi(cstr.as_ptr()) })
}

/// A private `xmalloc`ed copy of `bytes`, NUL-terminated: what a `%@Func@`
/// item hands to the caller's click table.
pub(super) fn dup_cstring(bytes: &[u8]) -> *mut c_char {
    // SAFETY: `bytes` is a run of the format string with its own length.
    unsafe { xmemdupz(bytes.as_ptr().cast::<c_void>(), bytes.len()) }.cast::<c_char>()
}

/// Release a `%@Func@` name the caller will never see.
pub(super) fn free_cstring(cmd: *mut c_char) {
    // SAFETY: [`dup_cstring`]'s allocation, freed exactly once -- the item it
    // came from is dropped from the table in the same step.
    unsafe { xfree(cmd.cast::<c_void>()) };
}

// ---------------------------------------------------------------------------
// The output buffer, wrapped
// ---------------------------------------------------------------------------

/// The NUL-terminated string starting at `at`.
///
/// Every stage keeps the write cursor NUL-terminated before it measures, so
/// the terminator is always inside the buffer.
pub(super) fn cstr_at(out: &[u8], at: usize) -> &CStr {
    CStr::from_bytes_until_nul(&out[at..]).unwrap_or(c"")
}

/// How many screen cells the string at `at` takes.
pub(super) fn strsize_at(out: &[u8], at: usize) -> c_int {
    // SAFETY: [`cstr_at`] has established the terminator.
    unsafe { vim_strsize(cstr_at(out, at).as_ptr()) }
}

/// How many screen cells the *character* at `at` takes.
pub(super) fn cells_at(out: &[u8], at: usize) -> c_int {
    // SAFETY: the buffer is NUL-terminated at or after `at`, so the read
    // stops inside it.
    unsafe { ptr2cells(out[at..].as_ptr().cast::<c_char>()) }
}

/// How many bytes the character at `at` takes, combining marks included.
pub(super) fn char_len_at(out: &[u8], at: usize) -> usize {
    // SAFETY: as [`cells_at`].
    unsafe { utfc_ptr2len(out[at..].as_ptr().cast::<c_char>()) as usize }
}

/// The character an expansion pads with, resolved to bytes once.
///
/// Upstream re-derives it from the glyph cache for every cell it writes --
/// `schar_get_adv` is a cache lookup, a `strlen` and a `memcpy` per cell,
/// and a padded line is nothing but such cells. Resolving it once turns
/// padding into a short `copy_from_slice`.
#[derive(Clone, Copy)]
pub(super) struct Fill {
    /// The glyph itself, for the two arms that compare it to `-`.
    schar: schar_T,
    bytes: [u8; MAX_SCHAR_SIZE as usize],
    len: usize,
}

impl Fill {
    /// Resolve `schar`, defaulting a zero to the blank upstream uses.
    fn of(schar: schar_T) -> Self {
        let schar = if schar == 0 { b' ' as schar_T } else { schar };
        let mut bytes = [0u8; MAX_SCHAR_SIZE as usize];
        let mut p = bytes.as_mut_ptr().cast::<c_char>();
        // SAFETY: `bytes` has room for `MAX_SCHAR_SIZE`, which is what
        // `schar_get_adv` is allowed; `schar` is a glyph this process
        // produced, from `'fillchars'` or the blank above.
        let len = unsafe { schar_get_adv(&raw mut p, schar) } as usize;
        Fill { schar, bytes, len }
    }

    /// How many bytes one of it takes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether it is the `-` that must not be put in front of a digit.
    pub fn is_dash(&self) -> bool {
        self.schar == b'-' as schar_T
    }

    /// Write one at `at`, answering where the next byte goes.
    ///
    /// The slice bound is real: upstream guards only that `at` is before the
    /// last byte of the buffer, so a fill character several bytes wide could
    /// write past its end. Here that panics instead.
    pub fn put(&self, out: &mut [u8], at: usize) -> usize {
        out[at..at + self.len].copy_from_slice(&self.bytes[..self.len]);
        at + self.len
    }
}

/// Write the glyph `sc` into `buf` at `at`, answering its byte length.
fn put_schar(buf: &mut [u8], at: usize, sc: schar_T) -> usize {
    // SAFETY: `schar_get_adv` writes at most `MAX_SCHAR_SIZE` bytes and `sc`
    // is a glyph this process produced; the slice bounds the write.
    let dst = &mut buf[at..];
    let mut p = dst.as_mut_ptr().cast::<c_char>();
    unsafe { schar_get_adv(&raw mut p, sc) as usize }
}

/// Print a number item through `template` at `at`, answering where the next
/// byte goes.
///
/// `args` is what the template's `*` and conversions consume: a width and a
/// value, or a value and an exponent for the reduced form.
fn put_number(out: &mut [u8], at: usize, plan: &parse::NumPlan) -> usize {
    let room = out.len() - at;
    let dst = out[at..].as_mut_ptr().cast::<c_char>();
    let template = plan.template();
    // SAFETY: `dst` has `room` bytes, `template` is NUL-terminated, and
    // [`parse::NumPlan`] built it to take exactly the two arguments below.
    let len = match plan.exp {
        Some(exp) => unsafe {
            vim_snprintf_safelen(dst, room as size_t, template.as_ptr(), 0, plan.num, exp)
        },
        // SAFETY: as above.
        None => unsafe {
            vim_snprintf_safelen(dst, room as size_t, template.as_ptr(), plan.width, plan.num)
        },
    };
    at + len
}

// ---------------------------------------------------------------------------
// The entry point
// ---------------------------------------------------------------------------

/// Build a string from the status line items in `fmt`, answering its width in
/// screen cells.
///
/// Normally works for window `wp`, except when working for `'tabline'`, when
/// it is `curwin`. `out` is the buffer to write into and must not be
/// `NameBuff`, which the expander uses as scratch; `hltab`, `hltab_len`,
/// `tabtab` and `stcp` may each be null.
///
/// # Safety
/// `wp` must be a live window, `out` must have `outlen` writable bytes, `fmt`
/// must be NUL-terminated, and the four out-parameters must each be null or
/// writable. This re-enters the editor, so nothing may be held across it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn build_stl_str_hl(
    wp: *mut win_T,
    out: *mut c_char,
    outlen: size_t,
    fmt: *mut c_char,
    opt_idx: OptIndex,
    opt_scope: OptionSetFlags,
    fillchar: schar_T,
    maxwidth: c_int,
    hltab: *mut *mut stl_hlrec_t,
    hltab_len: *mut size_t,
    tabtab: *mut *mut StlClickRecord,
    stcp: *mut statuscol_T,
) -> c_int {
    // SAFETY: the caller's buffer with its own length. The expander works in
    // bytes; the C signature spells them `char`.
    let out = unsafe { slice::from_raw_parts_mut(out.cast::<u8>(), outlen) };
    // SAFETY: the caller's live window.
    let win = unsafe { Win::new(wp) };
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
        // SAFETY: a live window and one of the option indices.
        unsafe { was_set_insecurely(wp, opt_idx, opt_scope) }
    };

    // SAFETY: the caller's NUL-terminated format string.
    let fmt_bytes = unsafe { CStr::from_ptr(fmt) }.to_bytes();
    // A format starting with "%!" is itself an expression, whose result is
    // the format actually used. Evaluating it can fail, in which case the
    // literal text is what gets rendered.
    let usefmt = if fmt_bytes.starts_with(b"%!") {
        let mut winid = typval_T {
            v_type: VAR_NUMBER,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_number: win.handle as varnumber_T,
            },
        };
        let name = c"g:statusline_winid";
        // SAFETY: a NUL-terminated name with its own length, and a typval
        // this frame owns, which `set_var` copies.
        unsafe {
            set_var(
                name.as_ptr(),
                name.to_bytes().len() as size_t,
                &raw mut winid,
                false,
            )
        };
        // SAFETY: `fmt` is NUL-terminated and at least two bytes long.
        let expanded = take_cstring(unsafe { eval_to_string_safe(fmt.add(2), sandbox, false) });
        unlet(name);
        expanded.unwrap_or_else(|| fmt_bytes.to_vec())
    } else {
        fmt_bytes.to_vec()
    };

    let fill = Fill::of(fillchar);

    // The cursor in windows other than the current one is not always
    // up to date, because of autocommands and timers.
    let mut win = win;
    let buf = win.buffer();
    let lnum = win.w_cursor.lnum.min(buf.b_ml.ml_line_count);
    win.w_cursor.lnum = lnum;

    // Read the byte under the cursor now, in case an item needs it: cheaper
    // than copying the line.
    // SAFETY: a live buffer and one of its lines.
    let line_ptr = unsafe { buf.line(lnum) }.raw();
    // SAFETY: `ml_get_buf` answers a NUL-terminated line.
    let empty_line = unsafe { *line_ptr } == 0;
    // SAFETY: as above.
    let len = unsafe { ml_get_buf_len(buf.raw(), lnum) };
    let byteval = if win.w_cursor.col > len {
        // The line may have changed since the cursor column was checked, or
        // the line number was adjusted above.
        win.w_cursor.col = len;
        win.w_cursor.coladd = 0 as colnr_T;
        0
    } else {
        // SAFETY: the cursor column is now inside the line.
        unsafe { utf_ptr2char(line_ptr.add(win.w_cursor.col as usize)) }
    };

    let env = Env {
        win,
        buf,
        stcp,
        opt_idx,
        sandbox,
        empty_line,
        byteval,
    };
    let built = expand(&env, out, usefmt, &fill, maxwidth, tabtab.is_null());

    // Hand back the highlight runs and the click records, which are views
    // into two of the arenas.
    with_scratch(|s| {
        if !hltab.is_null() {
            let runs = collect_highlights(s, out, &built);
            // SAFETY: the caller's out-parameter.
            unsafe { *hltab = runs };
        }
        if !hltab_len.is_null() {
            // Upstream answers the item count, not the run count.
            // SAFETY: the caller's out-parameter.
            unsafe { *hltab_len = built.itemcnt as size_t };
        }
        if !tabtab.is_null() {
            let recs = collect_clicks(s, out, &built);
            // SAFETY: the caller's out-parameter.
            unsafe { *tabtab = recs };
        }
    });

    redraw_not_allowed.set(save_redraw_not_allowed);

    // An error here would mess up the display and might loop redrawing;
    // avoid that by setting the option back to its default.
    if opt_idx as c_int != kOptInvalid as c_int && did_emsg.get() > did_emsg_before {
        set_option_direct(
            opt_idx,
            get_option_default(opt_idx, opt_scope),
            opt_scope,
            SID_ERROR,
        );
    }

    // A user function may reset KeyTyped; restore it.
    KeyTyped.set(save_key_typed);
    built.width
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
