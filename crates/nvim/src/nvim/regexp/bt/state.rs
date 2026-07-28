//! Saving and restoring where the engine is, and the stack it saves onto.
//!
//! Two shapes of position, because a match runs over either a string or a
//! range of buffer lines: a `regsave_T` holds a pointer for the first and a
//! line/column pair for the second, and which one is live is `rex.reg_match`
//! being null. Everything here comes in that pair.
//!
//! [`RegStack`] is the saved-state stack the matcher pushes decisions onto,
//! and `backpos` the record of where each loop back-edge has already been — a
//! `regsave_T`'s `rs_len` is the `backpos` length to truncate to, so undoing a
//! decision also forgets the loop positions discovered after it.
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

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::p_mmp;
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::regexp::{
    C2Rust_Unnamed_21, C2Rust_Unnamed_22, C2Rust_Unnamed_23,
    E_PATTERN_USES_MORE_MEMORY_THAN_MAXMEMPATTERN, NSUBEXP, REGSTACK_INITIAL, reg_getline,
    regbehind_T, regitem_T, regsave_T, regstar_T, regstate_T, rex, save_se_T,
};
use crate::src::nvim::types::{colnr_T, garray_T, lpos_T, uint8_t};

/// A frame as it goes on the stack; the pusher fills in the rest.
const BLANK_FRAME: regitem_T = regitem_T {
    rs_state: 0,
    rs_no: 0,
    rs_scan: core::ptr::null_mut(),
    rs_un: C2Rust_Unnamed_22 {
        regsave: regsave_T {
            rs_u: C2Rust_Unnamed_23 {
                ptr: core::ptr::null_mut(),
            },
            rs_len: 0,
        },
    },
};

/// A look-behind's capture snapshot as it goes on the stack.
const BLANK_BEHIND: regbehind_T = regbehind_T {
    save_after: BLANK_FRAME_SAVE,
    save_behind: BLANK_FRAME_SAVE,
    save_need_clear_subexpr: 0,
    save_start: [BLANK_SE; NSUBEXP as usize],
    save_end: [BLANK_SE; NSUBEXP as usize],
};

const BLANK_FRAME_SAVE: regsave_T = regsave_T {
    rs_u: C2Rust_Unnamed_23 {
        ptr: core::ptr::null_mut(),
    },
    rs_len: 0,
};

const BLANK_SE: save_se_T = save_se_T {
    se_u: C2Rust_Unnamed_21 {
        ptr: core::ptr::null_mut(),
    },
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
        if (self.bytes >> 10) as i64 >= p_mmp.get() {
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
pub(crate) fn reg_save(save: &mut regsave_T, gap: *mut garray_T) {
    // SAFETY: `gap` is the backpos garray and `rex` a live match.
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            save.rs_u.pos.col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
            save.rs_u.pos.lnum = (*rex.ptr()).lnum;
        } else {
            save.rs_u.ptr = (*rex.ptr()).input;
        }
        save.rs_len = (*gap).ga_len;
    }
}

/// Put the input position back to what [`reg_save`] recorded, refetching the
/// line if the match has moved off it since.
pub(crate) fn reg_restore(save: &regsave_T, gap: *mut garray_T) {
    // SAFETY: as `reg_save`.
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            if (*rex.ptr()).lnum != save.rs_u.pos.lnum {
                (*rex.ptr()).lnum = save.rs_u.pos.lnum;
                (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum).cast();
            }
            (*rex.ptr()).input = (*rex.ptr()).line.add(save.rs_u.pos.col as usize);
        } else {
            (*rex.ptr()).input = save.rs_u.ptr;
        }
        (*gap).ga_len = save.rs_len;
    }
}

/// Is the input exactly where `save` recorded? The `backpos` length is not
/// part of the comparison.
pub(crate) fn reg_save_equal(save: &regsave_T) -> bool {
    // SAFETY: as `reg_save`.
    unsafe {
        if (*rex.ptr()).reg_match.is_null() {
            (*rex.ptr()).lnum == save.rs_u.pos.lnum
                && (*rex.ptr()).input == (*rex.ptr()).line.add(save.rs_u.pos.col as usize)
        } else {
            (*rex.ptr()).input == save.rs_u.ptr
        }
    }
}

/// Move the current position into the capture slot `posp`, keeping what was
/// there in `savep`. The multi-line half of the pair.
pub(crate) fn save_se_multi(savep: &mut save_se_T, posp: *mut lpos_T) {
    // SAFETY: `posp` is a capture slot of the running match.
    unsafe {
        savep.se_u.pos = *posp;
        (*posp).lnum = (*rex.ptr()).lnum;
        (*posp).col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
    }
}

/// [`save_se_multi`] for a string match, where a capture is a pointer.
pub(crate) fn save_se_one(savep: &mut save_se_T, pp: *mut *mut uint8_t) {
    // SAFETY: as `save_se_multi`.
    unsafe {
        savep.se_u.ptr = *pp;
        *pp = (*rex.ptr()).input;
    }
}

/// Copy every `\1`..`\9` capture into `bp`, so that a look-behind attempt can
/// be undone whole.
///
/// `need_clear_subexpr` means the captures have not been touched yet this
/// match, and then there is nothing to copy — the flag alone restores them.
pub(crate) fn save_subexpr(bp: &mut regbehind_T) {
    // SAFETY: the capture arrays the match context holds have `NSUBEXP`
    // slots.
    unsafe {
        bp.save_need_clear_subexpr = (*rex.ptr()).need_clear_subexpr;
        if (*rex.ptr()).need_clear_subexpr != 0 {
            return;
        }
        for i in 0..NSUBEXP as usize {
            if (*rex.ptr()).reg_match.is_null() {
                bp.save_start[i].se_u.pos = *(*rex.ptr()).reg_startpos.add(i);
                bp.save_end[i].se_u.pos = *(*rex.ptr()).reg_endpos.add(i);
            } else {
                bp.save_start[i].se_u.ptr = *(*rex.ptr()).reg_startp.add(i);
                bp.save_end[i].se_u.ptr = *(*rex.ptr()).reg_endp.add(i);
            }
        }
    }
}

/// Undo [`save_subexpr`].
pub(crate) fn restore_subexpr(bp: &regbehind_T) {
    // SAFETY: as `save_subexpr`.
    unsafe {
        (*rex.ptr()).need_clear_subexpr = bp.save_need_clear_subexpr;
        if (*rex.ptr()).need_clear_subexpr != 0 {
            return;
        }
        for i in 0..NSUBEXP as usize {
            if (*rex.ptr()).reg_match.is_null() {
                *(*rex.ptr()).reg_startpos.add(i) = bp.save_start[i].se_u.pos;
                *(*rex.ptr()).reg_endpos.add(i) = bp.save_end[i].se_u.pos;
            } else {
                *(*rex.ptr()).reg_startp.add(i) = bp.save_start[i].se_u.ptr;
                *(*rex.ptr()).reg_endp.add(i) = bp.save_end[i].se_u.ptr;
            }
        }
    }
}
