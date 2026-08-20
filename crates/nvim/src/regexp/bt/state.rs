//! Saving and restoring where the engine is, and the stack it saves onto.
//!
//! A saved position is a [`MatchPos`]: a pointer for a string match, a
//! line/column pair for a buffer match, and which one it is comes from the
//! run rather than from the value — see that type for why.
//!
//! [`RegStack`] is the saved-state stack the matcher pushes decisions onto,
//! and `backpos` the record of where each loop back-edge has already been — a
//! [`SavedInput`]'s `backpos_len` is the `backpos` length to truncate to, so
//! undoing a decision also forgets the loop positions discovered after it.
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
    E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, MatchPos, NSUBEXP, REGSTACK_INITIAL, Rex,
    SavedInput, reg_getline, regbehind_T, regitem_T, regstar_T, regstate_T,
};
use crate::types::{garray_T, lpos_T, uint8_t};

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

/// The saved-state stack, live for the duration of one `regmatch`.
///
/// It is a global rather than a local of the match so that an ordinary match
/// never allocates: the frames a previous one needed are still there. Nothing
/// the matcher runs re-enters `regmatch`, so one stack is enough.
pub(crate) static regstack: GlobalCell<RegStack> = GlobalCell::new(RegStack::new());

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

/// Record the current input position, and how much of `gap` — always
/// `backpos` — belongs to it.
pub(crate) fn reg_save(rex: Rex, save: &mut SavedInput, gap: *mut garray_T) {
    save.pos = rex.here();
    // SAFETY: `gap` is the backpos garray of the running match.
    save.backpos_len = unsafe { (*gap).ga_len };
}

/// Put the input position back to what [`reg_save`] recorded, refetching the
/// line if the match has moved off it since.
pub(crate) fn reg_restore(rex: Rex, save: &SavedInput, gap: *mut garray_T) {
    if rex.multi() && rex.lnum() != save.pos.as_pos().lnum {
        rex.set_lnum(save.pos.as_pos().lnum);
        rex.set_line(reg_getline(rex, rex.lnum()).cast());
    }
    rex.seek_col_of(save.pos);
    // SAFETY: as `reg_save`.
    unsafe { (*gap).ga_len = save.backpos_len };
}

/// Move the current position into capture slot `no`, keeping what the slot
/// held in `savep` so the unwinder can put it back.
///
/// A buffer match's slots are the `posp` array and a string match's the `pp`
/// array; only the one this match's kind names is touched.
///
/// # Safety
///
/// `posp` and `pp` must be the pair of slots the running match holds for one
/// capture group.
pub(crate) unsafe fn save_capture(
    rex: Rex,
    savep: &mut MatchPos,
    posp: *mut lpos_T,
    pp: *mut *mut uint8_t,
) {
    // SAFETY: the caller hands over a live pair of slots, of which this
    // match's kind picks one.
    unsafe {
        if rex.multi() {
            *savep = MatchPos::from_pos(*posp);
            *posp = lpos_T {
                lnum: rex.lnum(),
                col: rex.col(),
            };
        } else {
            *savep = MatchPos::from_ptr(*pp);
            *pp = rex.input();
        }
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
