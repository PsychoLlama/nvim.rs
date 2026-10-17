//! Saving and restoring where the engine is, and the stack it saves onto.
//!
//! A saved position is a [`MatchPos`]: a pointer for a string match, a
//! line/column pair for a buffer match, and which one it is comes from the
//! run rather than from the value — see that type for why.
//!
//! [`RegStack`] is the saved-state stack the matcher pushes decisions onto,
//! and [`BackPos`] the record of where each loop back-edge has already been — a
//! [`SavedInput`]'s `backpos_len` is the `BackPos` length to put back, so
//! undoing a decision also forgets the loop positions discovered after it.
//!
//! Both are fields of [`BtState`], which the matcher borrows **once per
//! match** and threads down by `&mut`. That is not tidiness: `reg_save` and
//! `reg_restore` run millions of times in a regexp-heavy session, and a
//! `GlobalCell::with_mut` per call is a thread-local lookup and a hash-map
//! insert per call in a debug build.
//!
//! ## Why the stack charges C sizes
//!
//! Upstream is one byte garray with three record shapes packed into it: a
//! frame is a `RegItem`, and the two states that need more than a frame
//! carries — `\{n,m}` around a `SIMPLE` item, and a look-behind — reserve a
//! `RegStar` or a `RegBehind` *in front of* their frame and reach it as
//! `(rp as *mut RegStar).sub(1)`. Here each shape has its own `Vec` and the
//! pairing is the frame's state, which is what says whether a prefix is
//! there.
//!
//! 'maxmempattern' is expressed in kilobytes of that byte stack, and it is
//! the only bound on how far a pattern may backtrack — it is what makes a
//! runaway match end in E363 rather than in a dead editor. So [`RegStack`]
//! keeps `bytes`: the length upstream's `ga_len` would have, charged the C
//! size of a frame per frame and of a prefix per prefix
//! ([`REGITEM_BYTES`], [`REGSTAR_BYTES`], [`REGBEHIND_BYTES`]), so E363
//! fires at exactly the depth it used to — whatever the Rust records the
//! stack holds come to.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::message::emsg;
use crate::option::vars::p_mmp;
use crate::os::cshim::gettext;
use crate::regexp::{
    BACKPOS_INITIAL, E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, MatchPos, NSUBEXP,
    REGSTACK_INITIAL, RS_MCLOSE, RS_MOPEN, RS_ZOPEN, RegBehind, RegItem, RegStar, RegState, Rex,
    SavedInput, Slot, reg_getline,
};
use crate::types::int64_t;

/// What one frame cost on upstream's byte stack: a `regitem_T` — the state
/// (4), the group number (2, padded to 4), the node pointer (8) and the
/// `regsave_T`/`save_se_T` union (16). 'maxmempattern' is charged this per
/// frame, which is what keeps the depth E363 fires at where it was.
pub(crate) const REGITEM_BYTES: usize = 32;

/// A `regstar_T`, the `\{n,m}` counter a `RS_STAR_*` frame carries: two
/// `int`s and three `long`s.
pub(crate) const REGSTAR_BYTES: usize = 32;

/// A `regbehind_T`, the snapshot a look-behind frame carries: two
/// `regsave_T`s (32), an `int` padded to 8, and twenty `save_se_T`s (160).
pub(crate) const REGBEHIND_BYTES: usize = 200;

/// How many `\1`..`\9` slots a match has.
const NSUBEXP_SLOTS: usize = NSUBEXP as usize;

/// A frame as it goes on the stack; the pusher fills in the rest.
const BLANK_FRAME: RegItem = RegItem {
    rs_state: 0,
    rs_no: 0,
    rs_scan: 0,
    rs_saved: SavedInput::NOWHERE,
};

/// A look-behind's capture snapshot as it goes on the stack.
const BLANK_BEHIND: RegBehind = RegBehind {
    save_after: SavedInput::NOWHERE,
    save_behind: SavedInput::NOWHERE,
    save_need_clear_subexpr: 0,
    save_start: [MatchPos::NOWHERE; NSUBEXP_SLOTS],
    save_end: [MatchPos::NOWHERE; NSUBEXP_SLOTS],
};

/// How many frames the stack keeps between matches. Upstream pre-grew its
/// byte stack to `REGSTACK_INITIAL` and freed it again whenever a match had
/// made it larger; this is the same threshold counted in frames.
const KEEP_FRAMES: usize = REGSTACK_INITIAL as usize / REGITEM_BYTES;

/// How many back-edge entries the record keeps between matches, for the same
/// reason: upstream pre-grew its garray to this and freed it again whenever a
/// match had made it larger.
const KEEP_EDGES: usize = BACKPOS_INITIAL as usize;

/// The working state matches leave behind for the next one.
///
/// It is kept between matches so that an ordinary match never allocates:
/// the buffers a previous one needed are still there. A match borrows it
/// for its whole run ([`GlobalCell::with_mut`], once per `bt_regexec` call
/// rather than once per start column — moving it out and back cost three
/// copies of the brace counters per call). Nothing the matcher runs starts
/// another backtracking match, and a debug build would say so if it did.
///
/// [`super::matcher::regmatch`] threads its three fields down separately,
/// by `&mut`: `reg_save` and `reg_restore` run millions of times in a
/// regexp-heavy session.
pub(crate) static BT_STATE: GlobalCell<BtState> = GlobalCell::new(BtState::new());

/// The backtracker's working state for one match.
pub(crate) struct BtState {
    /// What the forward walk decided, and how to undo it.
    pub(crate) stack: RegStack,
    /// Where each loop back-edge has already been.
    pub(crate) backpos: BackPos,
    /// The `\{n,m}` bounds and counters of the complex repeats.
    pub(crate) braces: Braces,
}

impl Default for BtState {
    fn default() -> BtState {
        BtState::new()
    }
}

impl BtState {
    const fn new() -> BtState {
        BtState {
            stack: RegStack::new(),
            backpos: BackPos::new(),
            braces: Braces::new(),
        }
    }

    /// Start a match with nothing remembered.
    pub(crate) fn begin(&mut self) {
        self.stack.begin();
        self.backpos.begin();
    }

    /// Hand back what a pathological pattern made the buffers grow to.
    pub(crate) fn trim(&mut self) {
        self.stack.trim();
        self.backpos.trim();
    }
}

/// One `BACK` node and the input position the walk last reached it at.
struct BackEdge {
    /// The `BACK` node, which is what identifies the loop.
    scan: usize,
    /// Where the input was the last time the walk came round to it.
    pos: MatchPos,
}

/// Where each loop back-edge has already been, this match.
///
/// A loop that arrives back at its own `BACK` node without the input having
/// moved cannot do anything the last pass did not, so the match fails there
/// rather than spinning. That is all these entries are for.
///
/// ## Why the buffer is longer than the record
///
/// Undoing a decision puts the record back to the length
/// [`SavedInput::backpos_len`] kept — and that length can be *larger* than
/// the current one, because a look-behind restores to `save_after` after an
/// attempt that discovered fewer edges than were live when it was saved.
/// Upstream is a garray and `ga_len` is a bare cursor, so those entries come
/// back with whatever they last held. `live` is that cursor and `seen` keeps
/// what is past it, so they come back here too.
///
/// This is not theoretical and it is not rare: `\(a\)\@<=\(\(b\)*\)*c`
/// against `abbbc` restores the cursor from 0 back up to 2 on the first
/// attempt. A `Vec::truncate` in [`BackPos::rewind`] would silently drop
/// those two — which is why the `debug_assert` below is the tripwire for
/// anyone who tries it, and it fires on that pattern immediately.
pub(crate) struct BackPos {
    /// Every entry written this match, whether or not it is live.
    seen: Vec<BackEdge>,
    /// How many of them the record currently holds.
    live: usize,
}

impl BackPos {
    const fn new() -> BackPos {
        BackPos {
            seen: Vec::new(),
            live: 0,
        }
    }

    /// Start a match with nothing seen.
    fn begin(&mut self) {
        self.live = 0;
        if self.seen.capacity() < KEEP_EDGES {
            self.seen.reserve(KEEP_EDGES);
        }
    }

    /// Hand back what a pathological pattern made the record grow to.
    fn trim(&mut self) {
        if self.seen.capacity() > KEEP_EDGES {
            self.seen = Vec::new();
            self.live = 0;
        }
    }

    /// How many entries are live — what a [`SavedInput`] remembers.
    pub(crate) fn len(&self) -> usize {
        self.live
    }

    /// Put the record back to the length a [`SavedInput`] remembered — which
    /// may be longer than it is now; see the type's docs.
    fn rewind(&mut self, len: usize) {
        // Only the *cursor* moves. Shortening `seen` would make this fail.
        debug_assert!(len <= self.seen.len());
        self.live = len.min(self.seen.len());
    }

    /// A loop's back edge has come round to `scan`. Records where the input
    /// is now and reports whether it moved since the last time the walk was
    /// here — a loop that did not move can only spin, so `false` means the
    /// match has to fail at this node.
    pub(crate) fn stepped(&mut self, rex: Rex, scan: usize) -> bool {
        let found = self.seen[..self.live].iter().position(|e| e.scan == scan);
        match found {
            Some(i) if rex.is_at(self.seen[i].pos) => false,
            Some(i) => {
                self.seen[i].pos = rex.here();
                true
            }
            None => {
                let edge = BackEdge {
                    scan,
                    pos: rex.here(),
                };
                // Past the cursor the buffer still holds an older match's
                // entries; overwrite one rather than growing.
                if self.live < self.seen.len() {
                    self.seen[self.live] = edge;
                } else {
                    self.seen.push(edge);
                }
                self.live += 1;
                true
            }
        }
    }
}

/// One complex `\{n,m}`: the bounds the `BRACE_LIMITS` node in front of it
/// left, and how many passes the walk has taken.
#[derive(Clone, Copy)]
pub(crate) struct Brace {
    /// The lower bound — the *upper* one when the repeat is non-greedy, which
    /// is how `\{-n,m}` is spelled in the program.
    pub(crate) min: int64_t,
    /// The other bound.
    pub(crate) max: int64_t,
    /// Passes taken so far.
    pub(crate) count: int64_t,
}

/// The complex repeats' bounds and counters, one per `BRACE_COMPLEX` operand.
///
/// The compiler bounds the operand to nine, so ten slots is all there can be.
pub(crate) struct Braces([Brace; NSUBEXP_SLOTS]);

impl Braces {
    const fn new() -> Braces {
        Braces(
            [Brace {
                min: 0,
                max: 0,
                count: 0,
            }; NSUBEXP_SLOTS],
        )
    }

    /// The slot a `BRACE_COMPLEX` operand names.
    ///
    /// # Panics
    /// If `no` is not an operand the compiler could have emitted.
    pub(crate) fn slot(&mut self, no: usize) -> &mut Brace {
        &mut self.0[no]
    }
}

pub(crate) struct RegStack {
    /// One frame per decision, innermost last.
    frames: Vec<RegItem>,
    /// The `\{n,m}` counters, one per `RS_STAR_LONG`/`RS_STAR_SHORT` frame.
    stars: Vec<RegStar>,
    /// The look-behind snapshots, one per `RS_BEHIND1`/`RS_BEHIND2` frame.
    behinds: Vec<RegBehind>,
    /// What upstream's byte stack would be this long — see the module docs.
    bytes: usize,
}

impl RegStack {
    pub(crate) const fn new() -> RegStack {
        RegStack {
            frames: Vec::new(),
            stars: Vec::new(),
            behinds: Vec::new(),
            bytes: 0,
        }
    }

    /// Start a match with an empty stack.
    pub(crate) fn begin(&mut self) {
        self.frames.clear();
        self.stars.clear();
        self.behinds.clear();
        self.bytes = 0;
        if self.frames.capacity() < KEEP_FRAMES {
            self.frames.reserve(KEEP_FRAMES);
        }
    }

    /// Hand back what a pathological pattern made the stack grow to.
    pub(crate) fn trim(&mut self) {
        if self.frames.capacity() > KEEP_FRAMES {
            self.frames = Vec::with_capacity(KEEP_FRAMES);
        }
        // A look-behind record is twenty times the size of a frame, so it is
        // only worth keeping while it is small.
        if self.behinds.capacity() * size_of::<RegBehind>() > REGSTACK_INITIAL as usize {
            self.behinds = Vec::new();
        }
        if self.stars.capacity() * size_of::<RegStar>() > REGSTACK_INITIAL as usize {
            self.stars = Vec::new();
        }
    }

    /// How many frames are on the stack.
    pub(crate) fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Would `bytes` more put the stack over 'maxmempattern'? Reports E363
    /// and refuses if so.
    fn charge(&mut self, bytes: usize) -> bool {
        // 'maxmempattern' is bounded far below `i64::MAX`, so a stack that
        // does not fit in one is over any limit there could be.
        let kbytes = i64::try_from(self.bytes >> 10).unwrap_or(i64::MAX);
        if kbytes >= p_mmp() {
            emsg(gettext(E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN));
            return false;
        }
        self.bytes += bytes;
        true
    }

    /// Push a frame for `state` and hand it back for the caller to fill in.
    ///
    /// `None` when 'maxmempattern' has been reached.
    pub(crate) fn push(&mut self, state: RegState, scan: usize) -> Option<&mut RegItem> {
        if !self.charge(REGITEM_BYTES) {
            return None;
        }
        self.frames.push(RegItem {
            rs_state: state,
            rs_scan: scan,
            ..BLANK_FRAME
        });
        self.frames.last_mut()
    }

    /// Push a `\{n,m}` counter and the frame that reads it.
    pub(crate) fn push_star(&mut self, state: RegState, scan: usize, counter: RegStar) -> bool {
        if !self.charge(REGSTAR_BYTES) {
            return false;
        }
        self.stars.push(counter);
        self.push(state, scan).is_some()
    }

    /// Push a look-behind snapshot and the frame that reads it.
    pub(crate) fn push_behind(&mut self, state: RegState, scan: usize) -> bool {
        if !self.charge(REGBEHIND_BYTES) {
            return false;
        }
        self.behinds.push(BLANK_BEHIND);
        self.push(state, scan).is_some()
    }

    /// The frame on top.
    pub(crate) fn top(&self) -> &RegItem {
        self.frames.last().expect("a frame to resume")
    }

    /// The frame on top, to write to.
    pub(crate) fn top_mut(&mut self) -> &mut RegItem {
        self.frames.last_mut().expect("a frame to resume")
    }

    /// The top frame and the `\{n,m}` counter in front of it.
    pub(crate) fn top_star(&mut self) -> (&mut RegItem, &mut RegStar) {
        (
            self.frames.last_mut().expect("a frame to resume"),
            self.stars.last_mut().expect("a counter for the frame"),
        )
    }

    /// The top frame and the look-behind snapshot in front of it.
    pub(crate) fn top_behind(&mut self) -> (&mut RegItem, &mut RegBehind) {
        (
            self.frames.last_mut().expect("a frame to resume"),
            self.behinds.last_mut().expect("a snapshot for the frame"),
        )
    }

    /// Pop the top frame, resuming at the node it was pushed for.
    pub(crate) fn pop(&mut self, scan: &mut usize) {
        let frame = self.frames.pop().expect("a frame to pop");
        *scan = frame.rs_scan;
        self.bytes -= REGITEM_BYTES;
    }

    /// Pop a `RS_STAR_*` frame and the counter in front of it.
    pub(crate) fn pop_star(&mut self, scan: &mut usize) {
        self.pop(scan);
        self.stars.pop().expect("a counter for the frame");
        self.bytes -= REGSTAR_BYTES;
    }

    /// Pop a `RS_BEHIND*` frame and the snapshot in front of it.
    pub(crate) fn pop_behind(&mut self, scan: &mut usize) {
        self.pop(scan);
        self.behinds.pop().expect("a snapshot for the frame");
        self.bytes -= REGBEHIND_BYTES;
    }
}

/// Where the input is now, and how much of the back-edge record belongs to
/// that — everything undoing a decision needs to put both back.
///
/// This is by value rather than through an out-parameter because the one
/// caller that saves *into* the record ([`BackPos::stepped`]) would otherwise
/// be borrowing it twice.
#[inline(always)]
pub(crate) fn reg_save(rex: Rex, backpos: &BackPos) -> SavedInput {
    SavedInput {
        pos: rex.here(),
        backpos_len: backpos.len(),
    }
}

/// Put the input position back to what [`reg_save`] recorded, refetching the
/// line if the match has moved off it since, and forget the loop positions
/// discovered after it.
#[inline(always)]
pub(crate) fn reg_restore(rex: Rex, save: &SavedInput, backpos: &mut BackPos) {
    if rex.multi() && rex.lnum() != save.pos.as_pos().lnum {
        rex.set_lnum(save.pos.as_pos().lnum);
        rex.set_line(reg_getline(rex, rex.lnum()).unwrap_or(b"\0"));
    }
    rex.seek_col_of(save.pos);
    backpos.rewind(save.backpos_len);
}

/// Which slot a capture frame is about: its state says which end of which
/// family — the `\1` groups or the `\z(` ones — and `no` which group.
///
/// # Panics
/// If `no` is not a group, `\0`..`\9`; the compiler bounds the operand.
#[inline(always)]
pub(crate) fn capture_slot<'t>(rex: Rex<'t>, state: RegState, no: usize) -> Slot<'t> {
    match state {
        RS_MOPEN => rex.start(no),
        RS_MCLOSE => rex.end(no),
        RS_ZOPEN => rex.zstart(no),
        _ => rex.zend(no),
    }
}

/// Move the current position into `slot`, keeping what it held in `savep` so
/// the unwinder can put it back.
#[inline(always)]
pub(crate) fn save_capture(rex: Rex, savep: &mut MatchPos, slot: Slot) {
    *savep = slot.replace(rex.here());
}

/// Copy every `\1`..`\9` capture into `bp`, so that a look-behind attempt
/// can be undone whole.
///
/// `need_clear_subexpr` means the captures have not been touched yet this
/// match, and then there is nothing to copy — the flag alone restores them.
pub(crate) fn save_subexpr(rex: Rex, behind: &mut RegBehind) {
    behind.save_need_clear_subexpr = c_int_from(rex.need_clear_subexpr());
    if rex.need_clear_subexpr() {
        return;
    }
    behind.save_start = rex.starts().get().map(MatchPos::from_pos);
    behind.save_end = rex.ends().get().map(MatchPos::from_pos);
}

/// Undo [`save_subexpr`].
pub(crate) fn restore_subexpr(rex: Rex, behind: &RegBehind) {
    rex.set_need_clear_subexpr(behind.save_need_clear_subexpr != 0);
    if behind.save_need_clear_subexpr != 0 {
        return;
    }
    rex.starts().set(behind.save_start.map(MatchPos::as_pos));
    rex.ends().set(behind.save_end.map(MatchPos::as_pos));
}

/// The flag as the `int` the snapshot keeps.
fn c_int_from(flag: bool) -> core::ffi::c_int {
    core::ffi::c_int::from(flag)
}
