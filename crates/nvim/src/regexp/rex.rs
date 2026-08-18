//! [`Rex`], the handle both engines match through.
//!
//! `regexec_T` is the state a running match keeps: the buffer and window it
//! runs against, the line it is on and the byte within it, the capture slots
//! it fills, and the flags the compiler left for it. Upstream keeps it in a
//! file-scope `rex` and reaches it from everywhere; the port keeps the same
//! storage — [`super::api::with_rex`] saves and restores it around a nested
//! match, which is what lets a `\=` expression run a search of its own — but
//! the engines no longer reach through the global.
//!
//! Instead one `unsafe` promise is made per match, at the engine's entry
//! point, and the resulting handle is threaded down. `Rex` is `Copy` and
//! pointer-sized, so passing it costs a register; it deliberately has no
//! `Deref`, so every field the engines touch is named here and the
//! obligations that come with it are stated once rather than at each of the
//! several hundred use sites.
//!
//! The accessors are `#[inline(always)]` and the whole type is on the
//! per-character path: nothing here may allocate, bounds-check a slot it was
//! given by index, or route through `GlobalCell::with`, which is an outlined
//! call that pushes and pops a debug borrow-table entry.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use super::{regexec_T, rex};
use crate::charset::vim_iswordp_buf;
use crate::mbyte::{utf_ptr2char, utf_ptr2len, utfc_ptr2len};
use crate::types::{
    buf_T, colnr_T, linenr_T, lpos_T, regmatch_T, regmmatch_T, regprog_T, uint8_t, win_T,
};

/// A running match's context.
///
/// Obtained once per match with [`Rex::acquire`] and passed down by value.
/// Holding one is a claim that the context is set up and stays set up: see
/// that constructor for the whole of it.
#[derive(Clone, Copy)]
pub(crate) struct Rex(*mut regexec_T);

impl Rex {
    /// The context of the match that is about to run, or is running.
    ///
    /// # Safety
    ///
    /// The caller must hold the context — [`super::api::with_rex`] reserves
    /// it and restores any outer match's, and the compile path holds it
    /// because no match is running — and nothing else may form a reference
    /// into it while the handle lives, which is why every accessor below
    /// takes and returns copies rather than handing one out. That much is
    /// what the pattern compiler needs: it only reads `reg_buf` and writes
    /// the `nfa_*` findings.
    ///
    /// A *match* additionally has to have been set up ([`super::context::
    /// init_regexec`] or [`super::context::init_regexec_multi`]) before any
    /// accessor that reads the cursor or a capture slot, and for as long as
    /// the handle lives:
    ///
    /// - `line` and `input` point into the line being matched, `input` at or
    ///   after `line` and at or before its NUL;
    /// - `reg_buf` is a live buffer, and `reg_win` is null or a live window;
    /// - `reg_match` and `reg_mmatch` are the caller's match structures, one
    ///   of them null, and whichever capture arrays the live one implies
    ///   (`reg_startp`/`reg_endp` or `reg_startpos`/`reg_endpos`) hold
    ///   [`super::NSUBEXP`] entries each.
    ///
    /// The handle must not outlive the match: the line pointers dangle the
    /// moment the memline moves underneath it.
    #[inline(always)]
    pub(crate) unsafe fn acquire() -> Rex {
        Rex(rex.ptr())
    }

    // ------------------------------------------------------- the cursor

    /// The line the match is on, counted from `reg_firstlnum`.
    #[inline(always)]
    pub(crate) fn lnum(self) -> linenr_T {
        unsafe { (*self.0).lnum }
    }

    #[inline(always)]
    pub(crate) fn set_lnum(self, lnum: linenr_T) {
        unsafe { (*self.0).lnum = lnum }
    }

    /// The start of the line being matched.
    #[inline(always)]
    pub(crate) fn line(self) -> *mut uint8_t {
        unsafe { (*self.0).line }
    }

    #[inline(always)]
    pub(crate) fn set_line(self, line: *mut uint8_t) {
        unsafe { (*self.0).line = line }
    }

    /// Where in that line the match has got to.
    #[inline(always)]
    pub(crate) fn input(self) -> *mut uint8_t {
        unsafe { (*self.0).input }
    }

    #[inline(always)]
    pub(crate) fn set_input(self, input: *mut uint8_t) {
        unsafe { (*self.0).input = input }
    }

    /// [`Rex::input`] as the `char *` the byte-level helpers want.
    #[inline(always)]
    pub(crate) fn input_str(self) -> *mut c_char {
        self.input().cast()
    }

    /// The byte at the cursor. NUL at the end of the line — which is a
    /// position both engines test for, not a bound they stop short of.
    #[inline(always)]
    pub(crate) fn byte(self) -> uint8_t {
        unsafe { *(*self.0).input }
    }

    /// The whole character at the cursor.
    #[inline(always)]
    pub(crate) fn char_here(self) -> c_int {
        unsafe { utf_ptr2char(self.input_str()) }
    }

    /// The character at the cursor with its combining characters, in bytes.
    #[inline(always)]
    pub(crate) fn char_len(self) -> c_int {
        unsafe { utfc_ptr2len(self.input_str()) }
    }

    /// The base character at the cursor alone, in bytes.
    #[inline(always)]
    pub(crate) fn base_len(self) -> c_int {
        unsafe { utf_ptr2len(self.input_str()) }
    }

    /// Step the cursor over `n` bytes, which the caller has measured from
    /// the cursor.
    #[inline(always)]
    pub(crate) fn advance(self, n: c_int) {
        unsafe { (*self.0).input = (*self.0).input.offset(n as isize) }
    }

    /// Step the cursor over one whole character.
    #[inline(always)]
    pub(crate) fn advance_char(self) {
        self.advance(self.char_len());
    }

    /// Is the cursor at the start of the line?
    #[inline(always)]
    pub(crate) fn at_bol(self) -> bool {
        self.input() == self.line()
    }

    /// The column the cursor is in, zero-based, in bytes.
    #[inline(always)]
    pub(crate) fn col(self) -> colnr_T {
        unsafe { (*self.0).input.offset_from((*self.0).line) as colnr_T }
    }

    /// Put the cursor in column `col` of the line it is already on.
    #[inline(always)]
    pub(crate) fn set_col(self, col: colnr_T) {
        unsafe { (*self.0).input = (*self.0).line.offset(col as isize) }
    }

    /// Put the cursor in column `col` of `line`, which becomes the line
    /// being matched.
    #[inline(always)]
    pub(crate) fn seek(self, line: *mut uint8_t, col: colnr_T) {
        self.set_line(line);
        self.set_col(col);
    }

    /// Is the character at the cursor a keyword character for the matched
    /// buffer? 'iskeyword' is buffer-local and that buffer is not always the
    /// current one.
    #[inline(always)]
    pub(crate) fn iswordp(self) -> bool {
        unsafe { vim_iswordp_buf(self.input_str(), (*self.0).reg_buf) }
    }

    // --------------------------------------------- what is being matched

    /// The buffer the match runs against. Set even for a string match, so
    /// that `\k` and friends have an 'iskeyword' to read.
    #[inline(always)]
    pub(crate) fn reg_buf(self) -> *mut buf_T {
        unsafe { (*self.0).reg_buf }
    }

    #[inline(always)]
    pub(crate) fn set_reg_buf(self, buf: *mut buf_T) {
        unsafe { (*self.0).reg_buf = buf }
    }

    /// The window the match runs in, or null: `\%#` and `\%V` need one.
    #[inline(always)]
    pub(crate) fn reg_win(self) -> *mut win_T {
        unsafe { (*self.0).reg_win }
    }

    #[inline(always)]
    pub(crate) fn set_reg_win(self, win: *mut win_T) {
        unsafe { (*self.0).reg_win = win }
    }

    /// The buffer line `lnum` 0 of the match sits on.
    #[inline(always)]
    pub(crate) fn reg_firstlnum(self) -> linenr_T {
        unsafe { (*self.0).reg_firstlnum }
    }

    #[inline(always)]
    pub(crate) fn set_reg_firstlnum(self, lnum: linenr_T) {
        unsafe { (*self.0).reg_firstlnum = lnum }
    }

    /// The last line the match may reach, relative to `reg_firstlnum`.
    #[inline(always)]
    pub(crate) fn reg_maxline(self) -> linenr_T {
        unsafe { (*self.0).reg_maxline }
    }

    #[inline(always)]
    pub(crate) fn set_reg_maxline(self, lnum: linenr_T) {
        unsafe { (*self.0).reg_maxline = lnum }
    }

    /// The buffer line the cursor is on.
    #[inline(always)]
    pub(crate) fn buf_lnum(self) -> linenr_T {
        self.reg_firstlnum() + self.lnum()
    }

    /// Give up once a match starts past this column, or 0 for no bound.
    #[inline(always)]
    pub(crate) fn reg_maxcol(self) -> colnr_T {
        unsafe { (*self.0).reg_maxcol }
    }

    #[inline(always)]
    pub(crate) fn set_reg_maxcol(self, col: colnr_T) {
        unsafe { (*self.0).reg_maxcol = col }
    }

    // -------------------------------------------------------- the flags

    /// Is the match case-insensitive?
    #[inline(always)]
    pub(crate) fn reg_ic(self) -> bool {
        unsafe { (*self.0).reg_ic }
    }

    #[inline(always)]
    pub(crate) fn set_reg_ic(self, ic: bool) {
        unsafe { (*self.0).reg_ic = ic }
    }

    /// Must combining characters match exactly? Set by `\Z`.
    #[inline(always)]
    pub(crate) fn reg_icombine(self) -> bool {
        unsafe { (*self.0).reg_icombine }
    }

    #[inline(always)]
    pub(crate) fn set_reg_icombine(self, icombine: bool) {
        unsafe { (*self.0).reg_icombine = icombine }
    }

    /// Is the line break in the text a character to match rather than the
    /// end of the line? Set for a string that holds newlines.
    #[inline(always)]
    pub(crate) fn reg_line_lbr(self) -> bool {
        unsafe { (*self.0).reg_line_lbr }
    }

    #[inline(always)]
    pub(crate) fn set_reg_line_lbr(self, lbr: bool) {
        unsafe { (*self.0).reg_line_lbr = lbr }
    }

    /// May the user interrupt this match? `RE_NOBREAK` says no, for matches
    /// run where input cannot be read.
    #[inline(always)]
    pub(crate) fn reg_nobreak(self) -> bool {
        unsafe { (*self.0).reg_nobreak }
    }

    #[inline(always)]
    pub(crate) fn set_reg_nobreak(self, nobreak: bool) {
        unsafe { (*self.0).reg_nobreak = nobreak }
    }

    // ------------------------------------------------ the capture slots

    /// Is this a buffer match? A buffer match records positions in
    /// `reg_startpos`/`reg_endpos`; a string match records pointers in
    /// `reg_startp`/`reg_endp`. Every slot accessor below picks between the
    /// two on this, and so does most of the code that spans lines.
    #[inline(always)]
    pub(crate) fn multi(self) -> bool {
        unsafe { (*self.0).reg_match.is_null() }
    }

    /// The string match's structure, null for a buffer match.
    #[inline(always)]
    pub(crate) fn reg_match(self) -> *mut regmatch_T {
        unsafe { (*self.0).reg_match }
    }

    #[inline(always)]
    pub(crate) fn set_reg_match(self, rm: *mut regmatch_T) {
        unsafe { (*self.0).reg_match = rm }
    }

    /// The buffer match's structure, null for a string match.
    #[inline(always)]
    pub(crate) fn reg_mmatch(self) -> *mut regmmatch_T {
        unsafe { (*self.0).reg_mmatch }
    }

    #[inline(always)]
    pub(crate) fn set_reg_mmatch(self, rmm: *mut regmmatch_T) {
        unsafe { (*self.0).reg_mmatch = rmm }
    }

    /// The program being run, from whichever match structure is live.
    #[inline(always)]
    pub(crate) fn regprog(self) -> *mut regprog_T {
        unsafe {
            if self.multi() {
                (*(*self.0).reg_mmatch).regprog
            } else {
                (*(*self.0).reg_match).regprog
            }
        }
    }

    /// The `\1`..`\9` start slots of a string match.
    #[inline(always)]
    pub(crate) fn reg_startp(self) -> *mut *mut uint8_t {
        unsafe { (*self.0).reg_startp }
    }

    #[inline(always)]
    pub(crate) fn set_reg_startp(self, p: *mut *mut uint8_t) {
        unsafe { (*self.0).reg_startp = p }
    }

    /// The `\1`..`\9` end slots of a string match.
    #[inline(always)]
    pub(crate) fn reg_endp(self) -> *mut *mut uint8_t {
        unsafe { (*self.0).reg_endp }
    }

    #[inline(always)]
    pub(crate) fn set_reg_endp(self, p: *mut *mut uint8_t) {
        unsafe { (*self.0).reg_endp = p }
    }

    /// The `\1`..`\9` start slots of a buffer match.
    #[inline(always)]
    pub(crate) fn reg_startpos(self) -> *mut lpos_T {
        unsafe { (*self.0).reg_startpos }
    }

    #[inline(always)]
    pub(crate) fn set_reg_startpos(self, p: *mut lpos_T) {
        unsafe { (*self.0).reg_startpos = p }
    }

    /// The `\1`..`\9` end slots of a buffer match.
    #[inline(always)]
    pub(crate) fn reg_endpos(self) -> *mut lpos_T {
        unsafe { (*self.0).reg_endpos }
    }

    #[inline(always)]
    pub(crate) fn set_reg_endpos(self, p: *mut lpos_T) {
        unsafe { (*self.0).reg_endpos = p }
    }

    /// Do the `\1`..`\9` slots still hold a previous attempt's captures?
    /// The clearing is lazy: an engine only pays for it if a match reaches
    /// a back-reference.
    #[inline(always)]
    pub(crate) fn need_clear_subexpr(self) -> c_int {
        unsafe { (*self.0).need_clear_subexpr }
    }

    #[inline(always)]
    pub(crate) fn set_need_clear_subexpr(self, need: c_int) {
        unsafe { (*self.0).need_clear_subexpr = need }
    }

    /// As [`Rex::need_clear_subexpr`], for the `\z1`..`\z9` slots.
    #[inline(always)]
    pub(crate) fn need_clear_zsubexpr(self) -> c_int {
        unsafe { (*self.0).need_clear_zsubexpr }
    }

    #[inline(always)]
    pub(crate) fn set_need_clear_zsubexpr(self, need: c_int) {
        unsafe { (*self.0).need_clear_zsubexpr = need }
    }

    // ------------------------------------------- what the NFA compiler left

    /// Did the pattern use `\ze`?
    #[inline(always)]
    pub(crate) fn nfa_has_zend(self) -> c_int {
        unsafe { (*self.0).nfa_has_zend }
    }

    #[inline(always)]
    pub(crate) fn set_nfa_has_zend(self, has: c_int) {
        unsafe { (*self.0).nfa_has_zend = has }
    }

    /// Did the pattern use a back-reference? One forces the slow path.
    #[inline(always)]
    pub(crate) fn nfa_has_backref(self) -> c_int {
        unsafe { (*self.0).nfa_has_backref }
    }

    #[inline(always)]
    pub(crate) fn set_nfa_has_backref(self, has: c_int) {
        unsafe { (*self.0).nfa_has_backref = has }
    }

    /// Did the pattern use a `\z(` group?
    #[inline(always)]
    pub(crate) fn nfa_has_zsubexpr(self) -> c_int {
        unsafe { (*self.0).nfa_has_zsubexpr }
    }

    #[inline(always)]
    pub(crate) fn set_nfa_has_zsubexpr(self, has: c_int) {
        unsafe { (*self.0).nfa_has_zsubexpr = has }
    }

    /// How many capture groups the NFA program has, so that only the slots
    /// in use are copied around.
    #[inline(always)]
    pub(crate) fn nfa_nsubexpr(self) -> c_int {
        unsafe { (*self.0).nfa_nsubexpr }
    }

    #[inline(always)]
    pub(crate) fn set_nfa_nsubexpr(self, n: c_int) {
        unsafe { (*self.0).nfa_nsubexpr = n }
    }

    /// The generation stamp that says whether a state is already on the
    /// list being built. Bumped once per input position.
    #[inline(always)]
    pub(crate) fn nfa_listid(self) -> c_int {
        unsafe { (*self.0).nfa_listid }
    }

    #[inline(always)]
    pub(crate) fn set_nfa_listid(self, id: c_int) {
        unsafe { (*self.0).nfa_listid = id }
    }

    /// The stamp a `\@=` sub-match runs under, kept apart from the outer
    /// match's so that neither invalidates the other's lists.
    #[inline(always)]
    pub(crate) fn nfa_alt_listid(self) -> c_int {
        unsafe { (*self.0).nfa_alt_listid }
    }

    #[inline(always)]
    pub(crate) fn set_nfa_alt_listid(self, id: c_int) {
        unsafe { (*self.0).nfa_alt_listid = id }
    }
}
