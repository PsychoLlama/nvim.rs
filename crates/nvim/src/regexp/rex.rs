//! [`RegExec`], the state one running match keeps, and [`Rex`], the handle
//! both engines match through.
//!
//! The context is the buffer and window a match runs against, the line it
//! is on and the byte within it, the capture slots it fills, and the flags
//! the compiler left for it. Upstream keeps it in a file-scope `rex` and
//! reaches it from everywhere, saving and restoring the whole of it around
//! a nested match. Here the API layer builds one per match, on its own
//! stack, and the engines thread a [`Rex`] — a shared borrow of it — down
//! by value. A nested match (a `\=` expression that searches) builds its
//! own, so there is nothing to save and restore.
//!
//! ## The line
//!
//! The line being matched is a slice that **keeps its NUL**, and the cursor
//! is an offset into it. Both engines treat the terminator as a position
//! rather than a bound — `$` matches there, `\n` is tested for there, and
//! every character step reads it before deciding it has run out — so the
//! slice's last byte is always a NUL and every read at the cursor is
//! inside the slice.
//!
//! A string match's line is the caller's string, borrowed for the match. A
//! buffer match's lines are borrowed from the memline's one-line cache
//! through [`held_line`], whose contract is upstream's: the bytes are good
//! until the next read of a *different* line of the buffer. The engines keep
//! it — they fetch the next line through [`super::reg_getline`] and replace
//! the cursor's line at once, and after anything that may read another line
//! (`\%V`'s virtual columns, `\%'m`'s mark lookup, a multi-line
//! back-reference) they fetch their own line again. The offset survives
//! that; a pointer would not have.
//!
//! The accessors are `#[inline(always)]` and the whole type is on the
//! per-character path: nothing here may allocate, and the hot loops take the
//! line once ([`Rex::line`]) and index it rather than asking per byte.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::Cell;
use core::ffi::{CStr, c_int};

use super::pos::{MatchPos, PosKind, SavedInput};
use super::{ExtMatch, ExtMatchRef, NSUBEXP, TimeBudget};
use super::{char_at_nul, char_len_at, cluster_len_at};
use crate::charset::is_word_at;
use crate::memline::held_line;
use crate::types::{ColNr, LPos, LineNr, ProfTime};
use crate::winlayer::{Buf, Win};

/// A character's length as the `int` the engines count in. A character is a
/// few bytes, so the conversion never saturates.
#[inline(always)]
fn byte_count(len: usize) -> c_int {
    c_int::try_from(len).unwrap_or(c_int::MAX)
}

/// How many capture slots a match has: `\0`..`\9`.
const SLOTS: usize = NSUBEXP as usize;

/// One running match: see the module docs.
pub(crate) struct RegExec<'t> {
    // ------------------------------------------------ fixed for the match
    /// A buffer match (positions are line and column) rather than a string
    /// match (positions are columns of the one string).
    multi: bool,
    /// The buffer whose lines a buffer match reads, and whose 'iskeyword'
    /// `\k` and friends read in either kind of match.
    buf: &'t Buf,
    /// The window the match runs in, if there is one: `\%#` and `\%V`
    /// need one, and a string match has none.
    win: Option<Win>,
    /// The buffer line the match's line 0 sits on.
    firstlnum: LineNr,
    /// The last line the match may reach, relative to `firstlnum`.
    maxline: LineNr,
    /// Is a line break in the text a character to match rather than the
    /// end of the line? Set for a string that holds newlines.
    line_lbr: bool,
    /// May the user interrupt this match? `RE_NOBREAK` says no.
    nobreak: bool,
    /// Give up once a match starts past this column, or 0 for no bound.
    maxcol: ColNr,
    /// The caller's deadline, if it gave one.
    deadline: Option<ProfTime>,
    /// What `\z1`..`\z9` match: a syntax region's start captures.
    ext_in: Option<&'t ExtMatch>,

    // --------------------------------------- what the pattern overrides
    ic: Cell<bool>,
    icombine: Cell<bool>,

    // ---------------------------------------------------------- the cursor
    lnum: Cell<LineNr>,
    line: Cell<&'t [u8]>,
    input: Cell<usize>,

    // ----------------------------------------------------- the captures
    // Each family is one `Cell` of the whole array, reached a slot at a time
    // through `as_array_of_cells`, so that blanking one is one store of the
    // array (upstream's `memset`) rather than ten.
    //
    // The `\0`..`\9` arrays are the caller's: a buffer match's are its
    // `RegMMatch`'s own `startpos`/`endpos`, which upstream matched into in
    // place, and a string match's a [`GroupSlots`] the API layer turns into
    // offsets afterwards.
    starts: &'t Slots,
    ends: &'t Slots,
    zstarts: Slots,
    zends: Slots,
    need_clear_subexpr: Cell<bool>,
    need_clear_zsubexpr: Cell<bool>,

    // ------------------------------------------- what the match reports
    matchcol: Cell<Option<ColNr>>,
    timed_out: Cell<bool>,
    /// The `\z(` captures a successful match left, once one has: `None`
    /// until then, so that a caller's slot is only written by a match that
    /// wrote it upstream.
    ext_out: Cell<Option<Option<ExtMatchRef>>>,

    // ----------------------------------------- the backtracker's own state
    behind_pos: Cell<SavedInput>,
    brace_limits: Cell<(i64, i64)>,

    // -------------------------------------------- the NFA engine's state
    nfa_has_zend: Cell<c_int>,
    nfa_has_backref: Cell<c_int>,
    nfa_nsubexpr: Cell<c_int>,
    nfa_listid: Cell<c_int>,
    nfa_alt_listid: Cell<c_int>,
    nfa_has_zsubexpr: Cell<c_int>,
    nfa_match: Cell<c_int>,
    nfa_endp: Cell<Option<MatchPos>>,
    nfa_ll_index: Cell<usize>,
    nfa_time: Cell<TimeBudget>,
}

/// One family of capture slots, `\0`..`\9`.
pub(crate) type Slots = Cell<[LPos; SLOTS]>;

/// What an unset capture slot holds; see [`MatchPos::unset`].
const UNSET: LPos = LPos { lnum: -1, col: -1 };

/// The two slot arrays a string match records its groups in.
pub(crate) struct GroupSlots {
    pub(crate) starts: Slots,
    pub(crate) ends: Slots,
}

impl GroupSlots {
    pub(crate) fn new() -> GroupSlots {
        GroupSlots {
            starts: Cell::new([UNSET; SLOTS]),
            ends: Cell::new([UNSET; SLOTS]),
        }
    }
}

/// One capture slot, read and written as a [`MatchPos`].
#[derive(Clone, Copy)]
pub(crate) struct Slot<'t>(&'t Cell<LPos>);

impl Slot<'_> {
    #[inline(always)]
    pub(crate) fn get(self) -> MatchPos {
        MatchPos::from_pos(self.0.get())
    }

    #[inline(always)]
    pub(crate) fn set(self, at: MatchPos) {
        self.0.set(at.as_pos());
    }

    /// Put `at` in the slot and answer what it held.
    #[inline(always)]
    pub(crate) fn replace(self, at: MatchPos) -> MatchPos {
        MatchPos::from_pos(self.0.replace(at.as_pos()))
    }
}

/// What every match is set up with, whichever kind it is.
pub(crate) struct MatchSetup<'t> {
    /// The buffer: lines for a buffer match, 'iskeyword' for both.
    pub(crate) buf: &'t Buf,
    pub(crate) win: Option<Win>,
    /// 'ignorecase', as the caller's match structure asked.
    pub(crate) ic: bool,
    /// The program's `RE_NOBREAK`.
    pub(crate) nobreak: bool,
    pub(crate) deadline: Option<ProfTime>,
    pub(crate) ext_in: Option<&'t ExtMatch>,
}

impl<'t> RegExec<'t> {
    fn new(
        setup: MatchSetup<'t>,
        multi: bool,
        line: &'t [u8],
        starts: &'t Slots,
        ends: &'t Slots,
    ) -> RegExec<'t> {
        RegExec {
            multi,
            buf: setup.buf,
            win: setup.win,
            firstlnum: 0,
            maxline: 0,
            line_lbr: false,
            nobreak: setup.nobreak,
            maxcol: 0,
            deadline: setup.deadline,
            ext_in: setup.ext_in,
            ic: Cell::new(setup.ic),
            icombine: Cell::new(false),
            lnum: Cell::new(0),
            line: Cell::new(line),
            input: Cell::new(0),
            starts,
            ends,
            zstarts: Cell::new([UNSET; SLOTS]),
            zends: Cell::new([UNSET; SLOTS]),
            need_clear_subexpr: Cell::new(false),
            need_clear_zsubexpr: Cell::new(false),
            matchcol: Cell::new(None),
            timed_out: Cell::new(false),
            ext_out: Cell::new(None),
            behind_pos: Cell::new(SavedInput::NOWHERE),
            brace_limits: Cell::new((0, 0)),
            nfa_has_zend: Cell::new(0),
            nfa_has_backref: Cell::new(0),
            nfa_nsubexpr: Cell::new(0),
            nfa_listid: Cell::new(0),
            nfa_alt_listid: Cell::new(0),
            nfa_has_zsubexpr: Cell::new(0),
            nfa_match: Cell::new(0),
            nfa_endp: Cell::new(None),
            nfa_ll_index: Cell::new(0),
            nfa_time: Cell::new(TimeBudget::NONE),
        }
    }

    /// A match over `text`, the string with its NUL. `line_lbr` says the
    /// text holds newlines to be matched rather than ends of line.
    ///
    /// `firstlnum` is the first line of the last buffer match: upstream's
    /// string match never set `reg_firstlnum`, so `\%'m` compared the mark
    /// against whatever line the last buffer match started on (see
    /// [`super::api::in_match_scope`]).
    ///
    /// # Panics
    /// If `text` does not end in a NUL.
    pub(crate) fn for_string(
        setup: MatchSetup<'t>,
        text: &'t [u8],
        line_lbr: bool,
        firstlnum: LineNr,
        slots: &'t GroupSlots,
    ) -> Self {
        assert_eq!(text.last(), Some(&0), "a match runs over a terminated line");
        let mut ctx = RegExec::new(setup, false, text, &slots.starts, &slots.ends);
        ctx.line_lbr = line_lbr;
        ctx.firstlnum = firstlnum;
        ctx
    }

    /// A match over `setup.buf` from buffer line `lnum` on, which may start
    /// no later than column `maxcol` (0 for anywhere).
    ///
    /// The capture slots are `starts`/`ends`, the caller's match
    /// structure's own, which the match writes in place as upstream's did.
    pub(crate) fn for_buffer(
        setup: MatchSetup<'t>,
        lnum: LineNr,
        maxcol: ColNr,
        starts: &'t Slots,
        ends: &'t Slots,
    ) -> Self {
        let line_count = setup.buf.b_ml.ml_line_count;
        let mut ctx = RegExec::new(setup, true, b"\0", starts, ends);
        ctx.firstlnum = lnum;
        ctx.maxline = line_count - lnum;
        ctx.maxcol = maxcol;
        ctx
    }

    /// The handle the engines run the match through.
    pub(crate) fn rex(&'t self) -> Rex<'t> {
        Rex(self)
    }

    // ----------------------------------- what the API layer reads back

    /// The column a reported match starts at, when the engine recorded one.
    pub(crate) fn matchcol(&self) -> Option<ColNr> {
        self.matchcol.get()
    }

    /// Did the caller's deadline pass?
    pub(crate) fn timed_out(&self) -> bool {
        self.timed_out.get()
    }

    /// A successful syntax match's own `\z(` captures, taken: `None` when
    /// no match reported any, `Some(None)` when one reported it had none.
    pub(crate) fn take_ext_out(&self) -> Option<Option<ExtMatchRef>> {
        self.ext_out.take()
    }
}

/// A running match's context, by shared borrow: `Copy`, one word, passed
/// down by value. It deliberately has no `Deref`, so every field the
/// engines touch is named by an accessor here.
#[derive(Clone, Copy)]
pub(crate) struct Rex<'t>(&'t RegExec<'t>);

impl<'t> Rex<'t> {
    // ------------------------------------------------------- the cursor

    /// The line the match is on, counted from `reg_firstlnum`.
    #[inline(always)]
    pub(crate) fn lnum(self) -> LineNr {
        self.0.lnum.get()
    }

    #[inline(always)]
    pub(crate) fn set_lnum(self, lnum: LineNr) {
        self.0.lnum.set(lnum);
    }

    /// The line being matched, its NUL included.
    #[inline(always)]
    pub(crate) fn line(self) -> &'t [u8] {
        self.0.line.get()
    }

    #[inline(always)]
    pub(crate) fn set_line(self, line: &'t [u8]) {
        self.0.line.set(line);
    }

    /// Where in that line the match has got to: a byte offset.
    #[inline(always)]
    pub(crate) fn input(self) -> usize {
        self.0.input.get()
    }

    #[inline(always)]
    pub(crate) fn set_input(self, input: usize) {
        self.0.input.set(input);
    }

    /// The line from the cursor on, its NUL included.
    #[inline(always)]
    pub(crate) fn rest(self) -> &'t [u8] {
        let line = self.line();
        line.get(self.input()..).unwrap_or(&line[line.len() - 1..])
    }

    /// The byte at the cursor. NUL at the end of the line — which is a
    /// position both engines test for, not a bound they stop short of.
    #[inline(always)]
    pub(crate) fn byte(self) -> u8 {
        self.line().get(self.input()).copied().unwrap_or(0)
    }

    /// The byte `off` bytes past the cursor; NUL past the line's end.
    #[inline(always)]
    pub(crate) fn byte_at(self, off: usize) -> u8 {
        self.line().get(self.input() + off).copied().unwrap_or(0)
    }

    /// The whole character at the cursor.
    #[inline(always)]
    pub(crate) fn char_here(self) -> c_int {
        char_at_nul(self.line(), self.input())
    }

    /// The character at the cursor with its combining characters, in bytes.
    #[inline(always)]
    pub(crate) fn char_len(self) -> c_int {
        byte_count(cluster_len_at(self.line(), self.input()))
    }

    /// The base character at the cursor alone, in bytes.
    #[inline(always)]
    pub(crate) fn base_len(self) -> c_int {
        byte_count(char_len_at(self.line(), self.input()))
    }

    /// The character `off` bytes past the cursor.
    #[inline(always)]
    pub(crate) fn char_at_offset(self, off: c_int) -> c_int {
        char_at_nul(self.line(), self.input().wrapping_add_signed(off as isize))
    }

    /// Step the cursor over `n` bytes, which the caller has measured from
    /// the cursor (negative to step back).
    #[inline(always)]
    pub(crate) fn advance(self, n: c_int) {
        self.set_input(self.input().wrapping_add_signed(n as isize));
    }

    /// Step the cursor over one whole character.
    #[inline(always)]
    pub(crate) fn advance_char(self) {
        self.advance(self.char_len());
    }

    /// Is the cursor at the start of the line?
    #[inline(always)]
    pub(crate) fn at_bol(self) -> bool {
        self.input() == 0
    }

    /// The column the cursor is in, zero-based, in bytes.
    ///
    /// A line is at most `MAXCOL` bytes, so the offset always is a `ColNr`.
    #[inline(always)]
    pub(crate) fn col(self) -> ColNr {
        ColNr::try_from(self.input()).unwrap_or(ColNr::MAX)
    }

    /// Put the cursor in column `col` of the line it is already on.
    #[inline(always)]
    pub(crate) fn set_col(self, col: ColNr) {
        self.set_input(usize::try_from(col).unwrap_or(0));
    }

    /// Put the cursor in column `col` of `line`, which becomes the line
    /// being matched.
    #[inline(always)]
    pub(crate) fn seek(self, line: &'t [u8], col: ColNr) {
        self.set_line(line);
        self.set_col(col);
    }

    /// Is the character at the cursor a keyword character for the matched
    /// buffer? 'iskeyword' is buffer-local and that buffer is not always the
    /// current one.
    #[inline(always)]
    pub(crate) fn iswordp(self) -> bool {
        is_word_at(self.rest(), self.reg_buf())
    }

    // --------------------------------------------- what is being matched

    /// The buffer the match runs against. Set even for a string match, so
    /// that `\k` and friends have an 'iskeyword' to read.
    #[inline(always)]
    pub(crate) fn reg_buf(self) -> Buf {
        *self.0.buf
    }

    /// The window the match runs in, if there is one.
    #[inline(always)]
    pub(crate) fn reg_win(self) -> Option<Win> {
        self.0.win
    }

    /// The buffer line `lnum` 0 of the match sits on.
    #[inline(always)]
    pub(crate) fn reg_firstlnum(self) -> LineNr {
        self.0.firstlnum
    }

    /// The last line the match may reach, relative to `reg_firstlnum`.
    #[inline(always)]
    pub(crate) fn reg_maxline(self) -> LineNr {
        self.0.maxline
    }

    /// The buffer line the cursor is on.
    #[inline(always)]
    pub(crate) fn buf_lnum(self) -> LineNr {
        self.reg_firstlnum() + self.lnum()
    }

    /// Give up once a match starts past this column, or 0 for no bound.
    #[inline(always)]
    pub(crate) fn reg_maxcol(self) -> ColNr {
        self.0.maxcol
    }

    /// Buffer line `lnum`, its NUL included, borrowed for the match: see
    /// the module docs for how long the bytes stay good.
    #[inline(always)]
    pub(crate) fn buffer_line(self, lnum: LineNr) -> &'t [u8] {
        held_line(self.0.buf, lnum)
    }

    // -------------------------------------------------------- the flags

    /// Is the match case-insensitive?
    #[inline(always)]
    pub(crate) fn reg_ic(self) -> bool {
        self.0.ic.get()
    }

    #[inline(always)]
    pub(crate) fn set_reg_ic(self, ic: bool) {
        self.0.ic.set(ic);
    }

    /// Must combining characters match exactly? Set by `\Z`.
    #[inline(always)]
    pub(crate) fn reg_icombine(self) -> bool {
        self.0.icombine.get()
    }

    #[inline(always)]
    pub(crate) fn set_reg_icombine(self, icombine: bool) {
        self.0.icombine.set(icombine);
    }

    /// Is the line break in the text a character to match rather than the
    /// end of the line?
    #[inline(always)]
    pub(crate) fn reg_line_lbr(self) -> bool {
        self.0.line_lbr
    }

    /// May the user interrupt this match?
    #[inline(always)]
    pub(crate) fn reg_nobreak(self) -> bool {
        self.0.nobreak
    }

    // ------------------------------------------------ the capture slots

    /// Is this a buffer match? Every slot accessor below records positions
    /// for one, and so does most of the code that spans lines.
    #[inline(always)]
    pub(crate) fn multi(self) -> bool {
        self.0.multi
    }

    /// Which shape this match saves positions in.
    #[inline(always)]
    pub(crate) fn pos_kind(self) -> PosKind {
        if self.multi() {
            PosKind::Buf
        } else {
            PosKind::Str
        }
    }

    /// Where the cursor is now. A string match is all on line 0.
    #[inline(always)]
    pub(crate) fn here(self) -> MatchPos {
        MatchPos::from_pos(LPos {
            lnum: self.lnum(),
            col: self.col(),
        })
    }

    /// The position `off` bytes past the cursor.
    ///
    /// `off` is `-1` when a thread is about to cross a line break, and a
    /// buffer match then names the start of the *next* line rather than the
    /// byte before the cursor. A string match has no line to cross and steps
    /// back the one byte, which is what upstream did.
    #[inline(always)]
    pub(crate) fn at_offset(self, off: c_int) -> MatchPos {
        MatchPos::from_pos(if self.multi() && off == -1 {
            LPos {
                lnum: self.lnum() + 1,
                col: 0,
            }
        } else {
            LPos {
                lnum: self.lnum(),
                col: self.col() + off,
            }
        })
    }

    /// How far past the cursor `at` is, in bytes of the line the cursor is
    /// on.
    #[inline(always)]
    pub(crate) fn bytes_ahead(self, at: MatchPos) -> c_int {
        at.as_pos().col - self.col()
    }

    /// Is the cursor exactly at `at`?
    #[inline(always)]
    pub(crate) fn is_at(self, at: MatchPos) -> bool {
        self.here().same(at, self.pos_kind())
    }

    /// Is the cursor strictly before `at`?
    #[inline(always)]
    pub(crate) fn is_before(self, at: MatchPos) -> bool {
        self.here().is_before(at, self.pos_kind())
    }

    /// Put the cursor at `at`'s column of the line it is already on.
    ///
    /// A buffer match's line number is ignored: the caller has already put
    /// the cursor on the right line, or knows `at` is on it.
    #[inline(always)]
    pub(crate) fn seek_col_of(self, at: MatchPos) {
        self.set_col(at.as_pos().col);
    }

    /// Record the column the reported match starts at, for a `:substitute`
    /// or a search to resume from.
    #[inline(always)]
    pub(crate) fn set_matchcol(self, col: ColNr) {
        self.0.matchcol.set(Some(col));
    }

    /// The start slot of group `no`.
    ///
    /// # Panics
    /// When `no` is not a group, `\0`..`\9`.
    #[inline(always)]
    pub(crate) fn start(self, no: usize) -> Slot<'t> {
        Slot(&self.0.starts.as_array_of_cells()[no])
    }

    /// The end slot of group `no`.
    #[inline(always)]
    pub(crate) fn end(self, no: usize) -> Slot<'t> {
        Slot(&self.0.ends.as_array_of_cells()[no])
    }

    /// The start slot of `\z(` group `no`.
    #[inline(always)]
    pub(crate) fn zstart(self, no: usize) -> Slot<'t> {
        Slot(&self.0.zstarts.as_array_of_cells()[no])
    }

    /// The end slot of `\z(` group `no`.
    #[inline(always)]
    pub(crate) fn zend(self, no: usize) -> Slot<'t> {
        Slot(&self.0.zends.as_array_of_cells()[no])
    }

    /// The `\1`..`\9` start slots.
    #[inline(always)]
    pub(crate) fn starts(self) -> &'t Slots {
        self.0.starts
    }

    /// The `\1`..`\9` end slots.
    #[inline(always)]
    pub(crate) fn ends(self) -> &'t Slots {
        self.0.ends
    }

    /// The `\z1`..`\z9` start slots.
    #[inline(always)]
    pub(crate) fn zstarts(self) -> &'t Slots {
        &self.0.zstarts
    }

    /// The `\z1`..`\z9` end slots.
    #[inline(always)]
    pub(crate) fn zends(self) -> &'t Slots {
        &self.0.zends
    }

    /// Do the `\1`..`\9` slots still hold a previous attempt's captures?
    /// The clearing is lazy: an engine only pays for it if a match reaches
    /// a back-reference.
    #[inline(always)]
    pub(crate) fn need_clear_subexpr(self) -> bool {
        self.0.need_clear_subexpr.get()
    }

    #[inline(always)]
    pub(crate) fn set_need_clear_subexpr(self, need: bool) {
        self.0.need_clear_subexpr.set(need);
    }

    /// As [`Rex::need_clear_subexpr`], for the `\z1`..`\z9` slots.
    #[inline(always)]
    pub(crate) fn need_clear_zsubexpr(self) -> bool {
        self.0.need_clear_zsubexpr.get()
    }

    #[inline(always)]
    pub(crate) fn set_need_clear_zsubexpr(self, need: bool) {
        self.0.need_clear_zsubexpr.set(need);
    }

    // ------------------------------------- what the NFA program says

    /// Did the pattern use `\ze`?
    #[inline(always)]
    pub(crate) fn nfa_has_zend(self) -> c_int {
        self.0.nfa_has_zend.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_has_zend(self, has: c_int) {
        self.0.nfa_has_zend.set(has);
    }

    /// Did the pattern use a back-reference? One forces the slow path.
    #[inline(always)]
    pub(crate) fn nfa_has_backref(self) -> c_int {
        self.0.nfa_has_backref.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_has_backref(self, has: c_int) {
        self.0.nfa_has_backref.set(has);
    }

    /// Did the pattern use a `\z(` group?
    #[inline(always)]
    pub(crate) fn nfa_has_zsubexpr(self) -> c_int {
        self.0.nfa_has_zsubexpr.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_has_zsubexpr(self, has: c_int) {
        self.0.nfa_has_zsubexpr.set(has);
    }

    /// How many capture groups the NFA program has, so that only the slots
    /// in use are copied around.
    #[inline(always)]
    pub(crate) fn nfa_nsubexpr(self) -> c_int {
        self.0.nfa_nsubexpr.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_nsubexpr(self, n: c_int) {
        self.0.nfa_nsubexpr.set(n);
    }

    /// The generation stamp that says whether a state is already on the
    /// list being built. Bumped once per input position.
    #[inline(always)]
    pub(crate) fn nfa_listid(self) -> c_int {
        self.0.nfa_listid.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_listid(self, id: c_int) {
        self.0.nfa_listid.set(id);
    }

    /// The stamp a `\@=` sub-match runs under, kept apart from the outer
    /// match's so that neither invalidates the other's lists.
    #[inline(always)]
    pub(crate) fn nfa_alt_listid(self) -> c_int {
        self.0.nfa_alt_listid.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_alt_listid(self, id: c_int) {
        self.0.nfa_alt_listid.set(id);
    }

    // --------------------------------------------- the NFA engine's run

    /// The verdict so far: 0, a match, or `NFA_TOO_EXPENSIVE`.
    #[inline(always)]
    pub(crate) fn nfa_match(self) -> c_int {
        self.0.nfa_match.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_match(self, verdict: c_int) {
        self.0.nfa_match.set(verdict);
    }

    /// Where the lookbehind being matched has to stop, or `None` outside
    /// one.
    #[inline(always)]
    pub(crate) fn nfa_endp(self) -> Option<MatchPos> {
        self.0.nfa_endp.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_endp(self, endp: Option<MatchPos>) {
        self.0.nfa_endp.set(endp);
    }

    /// Which of a state's two `lastlist` slots the running match stamps.
    #[inline(always)]
    pub(crate) fn nfa_ll_index(self) -> usize {
        self.0.nfa_ll_index.get()
    }

    #[inline(always)]
    pub(crate) fn set_nfa_ll_index(self, index: usize) {
        self.0.nfa_ll_index.set(index);
    }

    /// Start the clock for one try.
    #[inline(always)]
    pub(crate) fn set_time_budget(self, budget: TimeBudget) {
        self.0.nfa_time.set(budget);
    }

    /// Count one step towards the next read of the clock: see
    /// [`TimeBudget::tick`].
    #[inline(always)]
    pub(crate) fn time_check_due(self, interval: c_int) -> bool {
        let mut budget = self.0.nfa_time.get();
        let due = budget.tick(interval);
        self.0.nfa_time.set(budget);
        due
    }

    // ------------------------------------------------------- the clock

    /// The caller's deadline, if it gave one.
    #[inline(always)]
    pub(crate) fn deadline(self) -> Option<ProfTime> {
        self.0.deadline
    }

    /// Report that the deadline passed.
    #[inline(always)]
    pub(crate) fn set_timed_out(self) {
        self.0.timed_out.set(true);
    }

    // ------------------------------------------ a syntax item's `\z` sets

    /// Run `f` over what `\z{no}` matches: the text the enclosing syntax
    /// region's start match captured for it, or `None` -- which matches the
    /// empty string.
    #[inline(always)]
    pub(crate) fn with_ext_capture<R>(self, no: usize, f: impl FnOnce(Option<&CStr>) -> R) -> R {
        f(self.0.ext_in.and_then(|ext| ext.capture(no)))
    }

    /// Hand a successful match's `\z(` captures to the caller.
    #[inline(always)]
    pub(crate) fn put_ext_out(self, captures: Option<ExtMatchRef>) {
        self.0.ext_out.set(Some(captures));
    }

    // --------------------------------------- the backtracker's own slots

    /// Where the look-behind being tried has to end.
    #[inline(always)]
    pub(crate) fn behind_pos(self) -> SavedInput {
        self.0.behind_pos.get()
    }

    #[inline(always)]
    pub(crate) fn set_behind_pos(self, pos: SavedInput) {
        self.0.behind_pos.set(pos);
    }

    /// The bounds of the `\{n,m}` about to be entered, read off its
    /// `BRACE_LIMITS` node and consumed by the `BRACE_SIMPLE` after it.
    #[inline(always)]
    pub(crate) fn brace_limits(self) -> (i64, i64) {
        self.0.brace_limits.get()
    }

    #[inline(always)]
    pub(crate) fn set_brace_limits(self, minval: i64, maxval: i64) {
        self.0.brace_limits.set((minval, maxval));
    }
}
