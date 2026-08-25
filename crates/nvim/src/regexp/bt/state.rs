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
//! frame is a `regitem_T`, and the two states that need more than a frame
//! carries — `\{n,m}` around a `SIMPLE` item, and a look-behind — reserve a
//! `regstar_T` or a `regbehind_T` *in front of* their frame and reach it as
//! `(rp as *mut regstar_T).sub(1)`. Here each shape has its own `Vec` and the
//! pairing is the frame's state, which is what says whether a prefix is
//! there.
//!
//! 'maxmempattern' is expressed in kilobytes of that byte stack, and it is
//! the only bound on how far a pattern may backtrack — it is what makes a
//! runaway match end in E363 rather than in a dead editor. So [`RegStack`]
//! keeps `bytes`: the length upstream's `ga_len` would have, charged
//! `size_of::<regitem_T>()` per frame and the prefix's own size per prefix,
//! so E363 fires at exactly the depth it used to.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::main::p_mmp;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    BACKPOS_INITIAL, E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, MatchPos, NSUBEXP,
    REGSTACK_INITIAL, RS_MCLOSE, RS_MOPEN, RS_ZOPEN, Rex, SavedInput, reg_endzp, reg_endzpos,
    reg_getline, reg_startzp, reg_startzpos, regbehind_T, regitem_T, regstar_T, regstate_T,
};
use crate::types::{int64_t, lpos_T, uint8_t};

/// How many `\1`..`\9` slots a match has.
const NSUBEXP_SLOTS: usize = NSUBEXP as usize;

/// A frame as it goes on the stack; the pusher fills in the rest.
const BLANK_FRAME: regitem_T = regitem_T {
    rs_state: 0,
    rs_no: 0,
    rs_scan: core::ptr::null_mut(),
    rs_saved: SavedInput::NOWHERE,
};

/// A look-behind's capture snapshot as it goes on the stack.
const BLANK_BEHIND: regbehind_T = regbehind_T {
    save_after: SavedInput::NOWHERE,
    save_behind: SavedInput::NOWHERE,
    save_need_clear_subexpr: 0,
    save_start: [MatchPos::NOWHERE; NSUBEXP_SLOTS],
    save_end: [MatchPos::NOWHERE; NSUBEXP_SLOTS],
};

/// How many frames the stack keeps between matches. Upstream pre-grew its
/// byte stack to `REGSTACK_INITIAL` and freed it again whenever a match had
/// made it larger; this is the same threshold counted in frames.
const KEEP_FRAMES: usize = REGSTACK_INITIAL as usize / size_of::<regitem_T>();

/// How many back-edge entries the record keeps between matches, for the same
/// reason: upstream pre-grew its garray to this and freed it again whenever a
/// match had made it larger.
const KEEP_EDGES: usize = BACKPOS_INITIAL as usize;

/// Everything one `regmatch` works in, live for the duration of that call.
///
/// It is a global rather than a local of the match so that an ordinary match
/// never allocates: the buffers a previous one needed are still there.
/// Nothing the matcher runs re-enters `regmatch` — `bt_regexec_both` calls
/// nothing that calls back into the editor — so one of these is enough, and
/// [`super::matcher::regmatch`] takes it once and threads its three fields
/// down separately.
///
/// **It is taken raw, not through [`GlobalCell::with_mut`], and that is
/// measured rather than sloppy.** `regmatch` runs once per start column, so
/// the debug borrow table's thread-local lookup and hash-map insert land on
/// it hundreds of thousands of times in a single `:%s`: wrapping the two
/// entry points in `with_mut` made a regexp-heavy debug workload **3.7×
/// slower** (2.7 s → 10.2 s). It is the same bargain
/// [`super::super::rex::Rex::acquire`] makes, for the same reason, and it is
/// why `reg_save`/`reg_restore` take a `&mut` rather than reaching the cell.
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
#[derive(Clone, Copy)]
struct BackEdge {
    /// The `BACK` node, which is what identifies the loop.
    scan: *mut uint8_t,
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
    pub(crate) fn stepped(&mut self, rex: Rex, scan: *mut uint8_t) -> bool {
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
    frames: Vec<regitem_T>,
    /// The `\{n,m}` counters, one per `RS_STAR_LONG`/`RS_STAR_SHORT` frame.
    stars: Vec<regstar_T>,
    /// The look-behind snapshots, one per `RS_BEHIND1`/`RS_BEHIND2` frame.
    behinds: Vec<regbehind_T>,
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
        if self.behinds.capacity() * size_of::<regbehind_T>() > REGSTACK_INITIAL as usize {
            self.behinds = Vec::new();
        }
        if self.stars.capacity() * size_of::<regstar_T>() > REGSTACK_INITIAL as usize {
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
        if kbytes >= p_mmp.get() {
            // SAFETY: a `&CStr`'s pointer, handed to the message layer.
            unsafe {
                emsg(gettext(
                    E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN.as_ptr(),
                ));
            }
            return false;
        }
        self.bytes += bytes;
        true
    }

    /// Push a frame for `state` and hand it back for the caller to fill in.
    ///
    /// `None` when 'maxmempattern' has been reached.
    pub(crate) fn push(&mut self, state: regstate_T, scan: *mut uint8_t) -> Option<&mut regitem_T> {
        if !self.charge(size_of::<regitem_T>()) {
            return None;
        }
        self.frames.push(regitem_T {
            rs_state: state,
            rs_scan: scan,
            ..BLANK_FRAME
        });
        self.frames.last_mut()
    }

    /// Push a `\{n,m}` counter and the frame that reads it.
    pub(crate) fn push_star(
        &mut self,
        state: regstate_T,
        scan: *mut uint8_t,
        counter: regstar_T,
    ) -> bool {
        if !self.charge(size_of::<regstar_T>()) {
            return false;
        }
        self.stars.push(counter);
        self.push(state, scan).is_some()
    }

    /// Push a look-behind snapshot and the frame that reads it.
    pub(crate) fn push_behind(&mut self, state: regstate_T, scan: *mut uint8_t) -> bool {
        if !self.charge(size_of::<regbehind_T>()) {
            return false;
        }
        self.behinds.push(BLANK_BEHIND);
        self.push(state, scan).is_some()
    }

    /// The frame on top.
    pub(crate) fn top(&self) -> &regitem_T {
        self.frames.last().expect("a frame to resume")
    }

    /// The frame on top, to write to.
    pub(crate) fn top_mut(&mut self) -> &mut regitem_T {
        self.frames.last_mut().expect("a frame to resume")
    }

    /// The top frame and the `\{n,m}` counter in front of it.
    pub(crate) fn top_star(&mut self) -> (&mut regitem_T, &mut regstar_T) {
        (
            self.frames.last_mut().expect("a frame to resume"),
            self.stars.last_mut().expect("a counter for the frame"),
        )
    }

    /// The top frame and the look-behind snapshot in front of it.
    pub(crate) fn top_behind(&mut self) -> (&mut regitem_T, &mut regbehind_T) {
        (
            self.frames.last_mut().expect("a frame to resume"),
            self.behinds.last_mut().expect("a snapshot for the frame"),
        )
    }

    /// Pop the top frame, resuming at the node it was pushed for.
    pub(crate) fn pop(&mut self, scan: &mut *mut uint8_t) {
        let frame = self.frames.pop().expect("a frame to pop");
        *scan = frame.rs_scan;
        self.bytes -= size_of::<regitem_T>();
    }

    /// Pop a `RS_STAR_*` frame and the counter in front of it.
    pub(crate) fn pop_star(&mut self, scan: &mut *mut uint8_t) {
        self.pop(scan);
        self.stars.pop().expect("a counter for the frame");
        self.bytes -= size_of::<regstar_T>();
    }

    /// Pop a `RS_BEHIND*` frame and the snapshot in front of it.
    pub(crate) fn pop_behind(&mut self, scan: &mut *mut uint8_t) {
        self.pop(scan);
        self.behinds.pop().expect("a snapshot for the frame");
        self.bytes -= size_of::<regbehind_T>();
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
        rex.set_line(reg_getline(rex, rex.lnum()).cast());
    }
    rex.seek_col_of(save.pos);
    backpos.rewind(save.backpos_len);
}

/// One end of one capture group, in whichever of the four slot arrays holds
/// it.
///
/// A buffer match records `lpos_T`s and a string match pointers, and *only the
/// pair its own kind names exists*: the other pair is null for the whole run.
/// So the kind has to be settled before a slot address is formed at all —
/// `null.add(no)` is undefined even when the address is thrown away
/// unexamined, which is why this is an enum of two addresses rather than a
/// pair of them.
#[derive(Clone, Copy)]
pub(crate) enum GroupSlot {
    /// A buffer match's slot, in the caller's match structure.
    Pos(*mut lpos_T),
    /// A string match's slot, in the caller's match structure.
    Ptr(*mut *mut uint8_t),
    /// A `\z(` group's slot. Those arrays are the engine's own rather than
    /// the caller's, so the slot is *named* — the array and the index — and
    /// never addressed.
    Z(ZSlot, usize),
}

/// Which of the four `\z(` arrays a [`GroupSlot::Z`] is in.
#[derive(Clone, Copy)]
pub(crate) enum ZSlot {
    /// `reg_startzpos`: where a buffer match's `\z(` group opened.
    PosStart,
    /// `reg_endzpos`: where it closed.
    PosEnd,
    /// `reg_startzp`: where a string match's `\z(` group opened.
    PtrStart,
    /// `reg_endzp`: where it closed.
    PtrEnd,
}

impl ZSlot {
    /// What the slot holds.
    fn get(self, no: usize) -> MatchPos {
        match self {
            ZSlot::PosStart => MatchPos::from_pos(reg_startzpos.get()[no]),
            ZSlot::PosEnd => MatchPos::from_pos(reg_endzpos.get()[no]),
            ZSlot::PtrStart => MatchPos::from_ptr(reg_startzp.get()[no]),
            ZSlot::PtrEnd => MatchPos::from_ptr(reg_endzp.get()[no]),
        }
    }

    /// Put `at` in the slot. The arrays are ten entries of a pointer each,
    /// so a whole-value read/modify/write is cheaper than a borrow would be
    /// and needs no `unsafe` at all.
    fn set(self, no: usize, at: MatchPos) {
        match self {
            ZSlot::PosStart => {
                let mut a = reg_startzpos.get();
                a[no] = at.as_pos();
                reg_startzpos.set(a);
            }
            ZSlot::PosEnd => {
                let mut a = reg_endzpos.get();
                a[no] = at.as_pos();
                reg_endzpos.set(a);
            }
            ZSlot::PtrStart => {
                let mut a = reg_startzp.get();
                a[no] = at.as_ptr();
                reg_startzp.set(a);
            }
            ZSlot::PtrEnd => {
                let mut a = reg_endzp.get();
                a[no] = at.as_ptr();
                reg_endzp.set(a);
            }
        }
    }
}

impl GroupSlot {
    /// What the slot holds.
    ///
    /// # Safety
    ///
    /// The slot must still belong to the running match.
    #[inline(always)]
    pub(crate) unsafe fn get(self) -> MatchPos {
        match self {
            // SAFETY: the caller promises a live slot, and the variant is the
            // shape it holds because the match's own kind chose it.
            GroupSlot::Pos(p) => MatchPos::from_pos(unsafe { *p }),
            GroupSlot::Ptr(p) => MatchPos::from_ptr(unsafe { *p }),
            GroupSlot::Z(which, no) => which.get(no),
        }
    }

    /// Put `at` in the slot.
    ///
    /// # Safety
    ///
    /// As [`GroupSlot::get`].
    #[inline(always)]
    pub(crate) unsafe fn set(self, at: MatchPos) {
        match self {
            // SAFETY: as `get`.
            GroupSlot::Pos(p) => unsafe { *p = at.as_pos() },
            GroupSlot::Ptr(p) => unsafe { *p = at.as_ptr() },
            GroupSlot::Z(which, no) => which.set(no, at),
        }
    }
}

/// Which slot a capture frame is about: its state says which end of which
/// family, and `no` which group.
///
/// The `\z(` groups live in this module's own arrays rather than in the
/// caller's match structure, which is the only reason there are four families
/// and not two — and it is why those two come back *named* rather than
/// addressed: nothing outside needs their address, so nothing outside gets
/// one, and their slots are reached with `get`/`set`.
///
/// The caller's two are still addresses, and there the *kind* has to be
/// settled before one is formed at all, because the pair this match does not
/// use is null for the whole run.
///
/// # Safety
///
/// `state` must be one of `RS_MOPEN`, `RS_MCLOSE`, `RS_ZOPEN`, `RS_ZCLOSE`,
/// and `no` must name a capture group the running match holds slots for.
#[inline(always)]
pub(crate) unsafe fn capture_slot(rex: Rex, state: regstate_T, no: usize) -> GroupSlot {
    if rex.multi() {
        let array = match state {
            RS_MOPEN => rex.reg_startpos(),
            RS_MCLOSE => rex.reg_endpos(),
            RS_ZOPEN => return GroupSlot::Z(ZSlot::PosStart, no),
            _ => return GroupSlot::Z(ZSlot::PosEnd, no),
        };
        // SAFETY: the caller promises the group, and this is the array a
        // buffer match fills.
        GroupSlot::Pos(unsafe { array.add(no) })
    } else {
        let array = match state {
            RS_MOPEN => rex.reg_startp(),
            RS_MCLOSE => rex.reg_endp(),
            RS_ZOPEN => return GroupSlot::Z(ZSlot::PtrStart, no),
            _ => return GroupSlot::Z(ZSlot::PtrEnd, no),
        };
        // SAFETY: as above, for a string match's array.
        GroupSlot::Ptr(unsafe { array.add(no) })
    }
}

/// Move the current position into `slot`, keeping what it held in `savep` so
/// the unwinder can put it back.
///
/// # Safety
///
/// `slot` must still belong to the running match.
pub(crate) unsafe fn save_capture(rex: Rex, savep: &mut MatchPos, slot: GroupSlot) {
    // SAFETY: the caller promises the slot.
    unsafe {
        *savep = slot.get();
        slot.set(rex.here());
    }
}

/// Copy every `\1`..`\9` capture into `bp`, so that a look-behind attempt can
/// be undone whole.
///
/// `need_clear_subexpr` means the captures have not been touched yet this
/// match, and then there is nothing to copy — the flag alone restores them.
pub(crate) fn save_subexpr(rex: Rex, bp: &mut regbehind_T) {
    bp.save_need_clear_subexpr = rex.need_clear_subexpr();
    if bp.save_need_clear_subexpr != 0 {
        return;
    }
    // SAFETY: whichever pair of capture arrays this match's kind names holds
    // `NSUBEXP` live entries for as long as the match runs.
    unsafe {
        if rex.multi() {
            let (starts, ends) = (rex.reg_startpos(), rex.reg_endpos());
            bp.save_start = core::array::from_fn(|i| MatchPos::from_pos(*starts.add(i)));
            bp.save_end = core::array::from_fn(|i| MatchPos::from_pos(*ends.add(i)));
        } else {
            let (starts, ends) = (rex.reg_startp(), rex.reg_endp());
            bp.save_start = core::array::from_fn(|i| MatchPos::from_ptr(*starts.add(i)));
            bp.save_end = core::array::from_fn(|i| MatchPos::from_ptr(*ends.add(i)));
        }
    }
}

/// Undo [`save_subexpr`].
pub(crate) fn restore_subexpr(rex: Rex, bp: &regbehind_T) {
    rex.set_need_clear_subexpr(bp.save_need_clear_subexpr);
    if bp.save_need_clear_subexpr != 0 {
        return;
    }
    // SAFETY: as `save_subexpr`.
    unsafe {
        if rex.multi() {
            let starts = core::slice::from_raw_parts_mut(rex.reg_startpos(), NSUBEXP_SLOTS);
            let ends = core::slice::from_raw_parts_mut(rex.reg_endpos(), NSUBEXP_SLOTS);
            for (slot, saved) in starts.iter_mut().zip(&bp.save_start) {
                *slot = saved.as_pos();
            }
            for (slot, saved) in ends.iter_mut().zip(&bp.save_end) {
                *slot = saved.as_pos();
            }
        } else {
            let starts = core::slice::from_raw_parts_mut(rex.reg_startp(), NSUBEXP_SLOTS);
            let ends = core::slice::from_raw_parts_mut(rex.reg_endp(), NSUBEXP_SLOTS);
            for (slot, saved) in starts.iter_mut().zip(&bp.save_start) {
                *slot = saved.as_ptr();
            }
            for (slot, saved) in ends.iter_mut().zip(&bp.save_end) {
                *slot = saved.as_ptr();
            }
        }
    }
}
