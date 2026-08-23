//! Suggestions found by trying changes against the word tree.
//!
//! This is the engine behind `z=`. It walks the language's word tree
//! depth-first and, at every node, tries each of a fixed list of edits to
//! the bad word: accept the byte as it is, substitute it, delete one,
//! insert one, swap a pair, rotate a triple, apply a `REP` rule from the
//! `.aff` file, or split the bad word in two. Whenever the tree says a
//! word ends and the bad word has been consumed, what has been collected
//! is a suggestion.
//!
//! Because each edit can be combined with the ones already made, the walk
//! is a search over sequences of edits, not a single pass. Every edit adds
//! to a running score and the search is pruned as soon as that score
//! exceeds `su_maxscore`, which is what keeps the space finite. Kemal
//! Oflazer's "Error-tolerant Finite State Recognition with Applications to
//! Morphological Analysis and Spelling Correction" (1996) describes the
//! same idea; the version here needs no more stack than the word is long.
//!
//! # The state machine
//!
//! One [`Frame`] per level records where in the tree the walk is, how far
//! into the bad word it has got, and which edits at that level have been
//! tried. [`State`] is the "which edit next" cursor: each round of the
//! driver loop runs the current frame's state, which either
//!
//! - performs one operation, pushes a child frame and increments `depth`
//!   (the child always starts at [`State::Start`], so the whole list of
//!   edits is tried again one level down),
//! - moves the frame on to the next state, or
//! - falls off the end at [`State::Final`] and pops back up.
//!
//! The states run in the order they are declared, and each handler is
//! responsible for naming its own successor. A few of them deliberately
//! run on into the next without a round trip through the driver -- that is
//! written here as one handler calling the next, and the comments say so
//! where it happens.
//!
//! # Frames, and which frame `level` means
//!
//! Every handler starts by taking `let level = self.depth`, and from then
//! on `self.stack[level]` is *this* frame even after `self.depth` has been
//! incremented -- so a handler that has pushed a child refers to the child
//! as `self.stack[level + 1]`. This mirrors the C, which held a
//! `trystate_T *sp` into the stack across the increment, and it matters:
//! several states write the child's fields while still reading the
//! parent's, in the same statement.
//!
//! # Speed
//!
//! This is the `z=` hot loop -- it runs once per byte of the whole
//! dictionary trie -- so a few decisions here are made for the machine
//! rather than the reader, and all three were measured:
//!
//! - [`Frame`] is `#[repr(C)]` with a `#[repr(u32)]` `State`, which makes
//!   it exactly 32 bytes. At its natural Rust size of 28 every access to
//!   `stack[level]` costs a multiply instead of a shift.
//! - The cheap edits in [`edit`] are `#[inline(always)]`. They are the
//!   states the walk spends most of its rounds in, and leaving them as
//!   calls out of the driver loop costs about a tenth of the total.
//! - The small accessors below are `#[inline]`: the release profile has no
//!   LTO and several codegen units, so without the hint they do not cross
//!   the module boundary.
//!
//! The stack is indexed normally, with bounds checks, which costs a couple
//! of per cent against the unchecked form. That is deliberate: the walk
//! keeps `depth` under `MAXWLEN` itself, and if it ever stopped doing so a
//! panic is a far better answer than a silent read past the frames.
//!
//! # Widths are load-bearing
//!
//! Most of [`Frame`]'s fields are `u8` while `MAXWLEN` is 254, so sums of
//! two positions can and do wrap. Every such sum is written `as u8` on
//! purpose: the wrap is upstream behaviour that suggestion output depends
//! on, and a checked add would panic where the C quietly wrapped.

#![deny(unsafe_op_in_unsafe_fn)]

mod edit;
mod node;
mod rep;
mod split;
mod transpose;

use crate::main::got_int;
use crate::mbyte::utf_head_off;
use crate::os::input::os_breakcheck;
use crate::profile::{profile_passed_limit, profile_setlimit};
use crate::spellsuggest::{MAXWLEN, spell_suggest_timeout, suginfo_T};
use crate::types::{idx_T, int64_t, langp_T, proftime_T, slang_T};
use ::libc::strlen;
use core::ffi::{c_char, c_int};

/// One level per byte of the bad word is all the walk can ever need.
const STACK_SIZE: usize = MAXWLEN;

/// `preword` holds a prefix, any number of compounded words and a split
/// word one after another, so it needs room for several whole words.
const PREWORD_SIZE: usize = MAXWLEN * 3;

// Values for `Frame::prefix_depth` that are not a stack depth.
/// This level is not below a postponed prefix.
pub(crate) const PFD_NOPREFIX: u8 = 0xff;
/// This level is inside the postponed-prefix tree rather than the
/// case-folded one.
pub(crate) const PFD_PREFIXTREE: u8 = 0xfe;
/// The highest `prefix_depth` that really is a stack depth.
pub(crate) const PFD_NOTSPECIAL: u8 = 0xfd;

// Values for `Frame::diff`: how the character being assembled in `tword`
// relates to the one in the bad word.
/// No differing byte seen yet.
pub(crate) const DIFF_NONE: u8 = 0;
/// A differing byte was found; this character is a substitution.
pub(crate) const DIFF_YES: u8 = 1;
/// This character is being inserted, so the bad word does not advance.
pub(crate) const DIFF_INSERT: u8 = 2;

// Bits for `Frame::flags`.
/// The prefix in front of this word has already been checked.
pub(crate) const FLAG_PREFIX_OK: u8 = 1;
/// A split was already tried at this point; see [`split`] for why that has
/// to be remembered.
pub(crate) const FLAG_DID_SPLIT: u8 = 2;
/// A byte was deleted here, and `Frame::del_idx` says which.
pub(crate) const FLAG_DID_DEL: u8 = 4;

/// Which edit the walk tries next at one level of the tree.
///
/// The order is the order they are tried in. `Final` is the end of the
/// list and pops the level.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
#[repr(u32)]
pub(crate) enum State {
    /// At the start of a node: deal with the NUL bytes, which mark places
    /// where the good word may end.
    #[default]
    Start,
    /// The same, entered at the root of the prefix tree to try the word
    /// without any prefix first.
    NoPrefix,
    /// Undo a word split or a compound join.
    SplitUndo,
    /// Past the NUL bytes at the start of the node.
    EndNul,
    /// Take each byte of the node as it is, or as a substitution.
    Plain,
    /// Delete a byte from the bad word.
    Del,
    /// Find the first byte of the node worth inserting.
    InsPrep,
    /// Insert a byte into the bad word.
    Ins,
    /// Swap two characters: "12" -> "21".
    Swap,
    /// Undo the swap.
    UnSwap,
    /// Swap two characters over a third: "123" -> "321".
    Swap3,
    /// Undo that, then rotate left: "123" -> "231".
    UnSwap3,
    /// Undo the left rotation, then rotate right: "123" -> "312".
    UnRot3L,
    /// Undo the right rotation.
    UnRot3R,
    /// Prepare to try the `REP` items of the `.aff` file.
    RepIni,
    /// Try one matching `REP` item.
    Rep,
    /// Undo a `REP` replacement and go back for the next one.
    RepUndo,
    /// Every edit at this level has been tried.
    Final,
}

/// What the walk knows at one level of the tree.
///
/// A child frame starts as a copy of its parent (see [`Walk::go_deeper`]),
/// so anything not explicitly reset is inherited.
///
/// The layout is pinned to 32 bytes; see the module docs.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub(crate) struct Frame {
    /// The edit to try next at this level.
    pub state: State,
    /// What the changes made to reach this level have cost.
    pub score: c_int,
    /// Index into the tree array of the start of this node.
    pub node: idx_T,
    /// Which child of the node is being tried, counted from the node's
    /// length byte. `REP` reuses it as an index into the `REP` list, which
    /// is why it is wider than a byte.
    pub child: i16,
    /// How far into `fword`, the case-folded bad word, this level is.
    pub bad_idx: u8,
    /// The `bad_idx` from which bytes may still be changed. Everything
    /// before it has already been rewritten by an edit and must be taken
    /// as it stands.
    pub change_from: u8,
    /// Valid length of `tword`, the good word collected so far.
    pub good_len: u8,
    /// The stack depth at which the postponed prefix ends, or
    /// [`PFD_PREFIXTREE`] / [`PFD_NOPREFIX`].
    pub prefix_depth: u8,
    /// `FLAG_` bits.
    pub flags: u8,
    /// How many bytes the `tword` character being assembled has, and how
    /// many of them are in. Zero between characters.
    pub char_len: u8,
    pub char_idx: u8,
    /// A `DIFF_` value for that character.
    pub diff: u8,
    /// Where in `fword` the bad word's matching character started.
    pub bad_char_start: u8,
    /// Length of the word in `preword`.
    pub preword_len: u8,
    /// Index in `tword` just past the last split.
    pub split_off: u8,
    /// The `bad_idx` at that split.
    pub split_bad_idx: u8,
    /// How many compound words have been used, and where in `compflags`
    /// the last split was.
    pub comp_len: u8,
    pub comp_split: u8,
    /// `su_badflags` as it was before a split changed it.
    pub saved_badflags: u8,
    /// Where in `fword` the deleted character was; valid when `flags` has
    /// [`FLAG_DID_DEL`].
    pub del_idx: u8,
}

/// Indexing the stack must stay a shift; see the module docs.
const _: () = assert!(size_of::<Frame>() == 32);

/// Everything one run of the walk carries.
///
/// The raw pointers are the language's loaded tables and the caller's
/// buffers, none of which this module owns; the arrays are its own working
/// storage.
pub(crate) struct Walk {
    /// The suggestion list being filled, and the language being searched.
    pub su: *mut suginfo_T,
    pub lp: *mut langp_T,
    pub slang: *mut slang_T,
    /// Walking the sound-fold tree rather than the case-folded one. Word
    /// flags, case, banned words, splitting and `similar_chars` all do not
    /// apply then; see [`suggest_trie_walk`].
    pub soundfold: bool,

    /// The tree of case-folded (or, when `soundfold`, sound-folded) words.
    pub fbyts: *mut u8,
    pub fidxs: *mut idx_T,
    /// The tree of postponed prefixes, null when the language has none.
    pub pbyts: *mut u8,
    pub pidxs: *mut idx_T,
    /// When to give up. The walk can otherwise run for an unbounded time.
    pub time_limit: proftime_T,

    /// The tree currently being walked: `pbyts`/`pidxs` while inside a
    /// postponed prefix, `fbyts`/`fidxs` otherwise.
    pub byts: *mut u8,
    pub idxs: *mut idx_T,

    /// The bad word, case-folded, as the caller's buffer. `REP` items
    /// rewrite stretches of it in place and undo the change on the way
    /// back up, which is why it stays a pointer: a replacement that grows
    /// the word shifts the tail along and the C let that run to the end of
    /// the buffer.
    pub fword: *mut c_char,
    /// How many bytes `REP` items have added to `fword` so far. The
    /// suggestion has to be told how much of the *original* bad word it
    /// replaces, which is the position in `fword` less this.
    pub repextra: c_int,

    /// The good word collected from the tree so far.
    pub tword: [c_char; MAXWLEN],
    /// The suggestion with its proper case: prefix, compounded words and
    /// split words concatenated. NUL-terminated while going deeper, but
    /// not on the way back up.
    pub preword: [c_char; PREWORD_SIZE],
    /// One compound flag per word collected in `preword`.
    pub compflags: [u8; MAXWLEN],

    pub stack: [Frame; STACK_SIZE],
    pub depth: c_int,
    /// Counts down to the next check for CTRL-C, which is too slow to do
    /// every round.
    breakcheckcount: c_int,
}

/// Try finding suggestions by adding, removing and swapping letters.
///
/// This is also used for the sound-folded word, with `soundfold` true.
/// The mechanism is the same, but the match is with a sound-folded word
/// that stands for one or more real words; turning it back into those is
/// `add_sound_suggest`'s job. In that mode the walk must not use the
/// prefix tree or the keep-case tree, `su_badlen`, anything to do with
/// case, word versus non-word characters, banned words, word flags (rare,
/// region, compounding), word splitting or `similar_chars`, and it takes
/// its `REP` items from `sl_repsal` rather than the replacement language's
/// `sl_rep`.
///
/// # Safety
///
/// `su` and `lp` must be valid, `lp`'s language must have its trees
/// loaded, and `fword` must be a NUL-terminated buffer of `MAXWLEN` bytes.
pub(super) unsafe fn suggest_trie_walk(
    su: *mut suginfo_T,
    lp: *mut langp_T,
    fword: *mut c_char,
    soundfold: bool,
) {
    // SAFETY: the caller guarantees the pointers and the loaded trees.
    unsafe {
        let mut walk = Walk::new(su, lp, fword, soundfold);
        walk.run();
    }
}

impl Walk {
    /// Set the walk up at the root of whichever tree it starts in.
    ///
    /// # Safety
    ///
    /// As [`suggest_trie_walk`].
    unsafe fn new(
        su: *mut suginfo_T,
        lp: *mut langp_T,
        fword: *mut c_char,
        soundfold: bool,
    ) -> Walk {
        // SAFETY: the caller guarantees the pointers and the loaded trees.
        unsafe {
            let slang = (*lp).lp_slang;
            let mut walk = Walk {
                su,
                lp,
                slang,
                soundfold,
                fbyts: core::ptr::null_mut(),
                fidxs: core::ptr::null_mut(),
                pbyts: core::ptr::null_mut(),
                pidxs: core::ptr::null_mut(),
                time_limit: 0,
                byts: core::ptr::null_mut(),
                idxs: core::ptr::null_mut(),
                fword,
                repextra: 0,
                tword: [0; MAXWLEN],
                preword: [0; PREWORD_SIZE],
                compflags: [0; MAXWLEN],
                stack: [Frame::default(); STACK_SIZE],
                depth: 0,
                breakcheckcount: 1000,
            };
            walk.stack[0].child = 1;

            if soundfold {
                // The sound-fold tree has no prefixes.
                walk.fbyts = (*slang).sl_sbyts;
                walk.fidxs = (*slang).sl_sidxs;
                walk.byts = walk.fbyts;
                walk.idxs = walk.fidxs;
                walk.stack[0].prefix_depth = PFD_NOPREFIX;
                walk.stack[0].state = State::Start;
            } else {
                walk.fbyts = (*slang).sl_fbyts;
                walk.fidxs = (*slang).sl_fidxs;
                walk.pbyts = (*slang).sl_pbyts;
                walk.pidxs = (*slang).sl_pidxs;
                if walk.pbyts.is_null() {
                    walk.byts = walk.fbyts;
                    walk.idxs = walk.fidxs;
                    walk.stack[0].prefix_depth = PFD_NOPREFIX;
                    walk.stack[0].state = State::Start;
                } else {
                    // Postponed prefixes have to be used first; the
                    // case-folded tree continues at the end of the prefix.
                    walk.byts = walk.pbyts;
                    walk.idxs = walk.pidxs;
                    walk.stack[0].prefix_depth = PFD_PREFIXTREE;
                    walk.stack[0].state = State::NoPrefix; // without a prefix first
                }
            }

            let timeout = spell_suggest_timeout.get();
            if timeout > 0 {
                walk.time_limit = profile_setlimit(timeout as int64_t);
            }
            walk
        }
    }

    /// Run every state of every level until the stack empties or the user
    /// interrupts.
    ///
    /// # Safety
    ///
    /// The walk must have been set up by [`Walk::new`].
    unsafe fn run(&mut self) {
        while self.depth >= 0 && !got_int.get() {
            // SAFETY: the walk's pointers are the ones `new` was given and
            // the tree indices stay inside the trees.
            unsafe { self.step() };
        }
    }

    /// Run the current level's current state once.
    ///
    /// # Safety
    ///
    /// `self.depth` must be a live level.
    unsafe fn step(&mut self) {
        // SAFETY: every handler reads the language's trees at indices the
        // trees' own child counts bound, and the bad word within the
        // buffer the caller supplied.
        unsafe {
            match self.stack[self.depth as usize].state {
                State::Start | State::NoPrefix => self.node_start(),
                State::SplitUndo => self.split_undo(),
                State::EndNul => self.end_nul(),
                State::Plain => self.plain(),
                State::Del => self.delete(),
                State::InsPrep => self.ins_prep(),
                State::Ins => self.insert(),
                State::Swap => self.swap(),
                State::UnSwap => self.un_swap(),
                State::Swap3 => self.swap3(),
                State::UnSwap3 => self.un_swap3(),
                State::UnRot3L => self.un_rot3l(),
                State::UnRot3R => self.un_rot3r(),
                State::RepIni => self.rep_ini(),
                State::Rep => self.rep(),
                State::RepUndo => self.rep_undo(),
                State::Final => self.leave_level(),
            }
        }
    }

    /// Every edit at this level has been tried: pop it.
    ///
    /// # Safety
    ///
    /// `self.depth` must be a live level.
    unsafe fn leave_level(&mut self) {
        self.depth -= 1;

        if self.depth >= 0 && self.stack[self.depth as usize].prefix_depth == PFD_PREFIXTREE {
            // Continue in, or go back to, the prefix tree.
            self.byts = self.pbyts;
            self.idxs = self.pidxs;
        }

        // Checking for CTRL-C takes time, so only do it now and then.
        self.breakcheckcount -= 1;
        if self.breakcheckcount == 0 {
            // SAFETY: reads the pending-input queue, which is main-thread
            // editor state.
            os_breakcheck();
            self.breakcheckcount = 1000;
            if spell_suggest_timeout.get() > 0 && profile_passed_limit(self.time_limit) {
                got_int.set(true);
            }
        }
    }

    /// Push a copy of this level as a new level, ready to try every edit
    /// again one byte further on.
    ///
    /// The copy is taken *before* the caller sets this level's next state,
    /// which is why several handlers call this first and only then say
    /// where the parent goes next: doing it the other way round would give
    /// the child the parent's successor state instead of [`State::Start`].
    #[inline]
    fn go_deeper(&mut self, score_add: c_int) {
        let level = self.depth as usize;
        let mut child = self.stack[level];
        child.state = State::Start;
        child.score = self.stack[level].score + score_add;
        child.child = 1; // start just after the length byte
        child.flags = 0;
        self.stack[level + 1] = child;
    }

    /// Would going one level deeper stay inside the stack and under the
    /// score ceiling? A change that cannot beat the worst suggestion
    /// already found is not worth trying.
    ///
    /// # Safety
    ///
    /// `self.su` must be valid.
    #[inline]
    unsafe fn try_deeper(&self, score_add: c_int) -> bool {
        // SAFETY: the caller guarantees `su`.
        unsafe {
            self.depth < MAXWLEN as c_int - 1
                && self.stack[self.depth as usize].score + score_add < (*self.su).su_maxscore
        }
    }

    /// One byte of the tree currently being walked.
    ///
    /// The first byte of a node is how many children follow it, so every
    /// index the walk forms is bounded by a count the tree itself stores.
    ///
    /// # Safety
    ///
    /// `at` must be an index into the current tree.
    #[inline]
    unsafe fn byte_at(&self, at: idx_T) -> u8 {
        // SAFETY: the caller guarantees the index.
        unsafe { *self.byts.offset(at as isize) }
    }

    /// The tree entry beside a byte: for a child byte, where its node
    /// starts; for a NUL byte, the word's flags.
    ///
    /// # Safety
    ///
    /// `at` must be an index into the current tree.
    #[inline]
    unsafe fn idx_at(&self, at: idx_T) -> idx_T {
        // SAFETY: the caller guarantees the index.
        unsafe { *self.idxs.offset(at as isize) }
    }

    /// One byte of the bad word, unsigned.
    ///
    /// `char` is signed here, and the comparisons against tree bytes are
    /// all against unsigned ones, so widening has to be unsigned or every
    /// byte from 0x80 up would compare wrong.
    ///
    /// # Safety
    ///
    /// `at` must be within the bad word's buffer.
    #[inline]
    unsafe fn fword_at(&self, at: usize) -> c_int {
        // SAFETY: the caller guarantees the index.
        unsafe { *self.fword.add(at) as u8 as c_int }
    }

    /// A pointer into the bad word, for the helpers that take one.
    ///
    /// # Safety
    ///
    /// `at` must be within the bad word's buffer.
    #[inline]
    unsafe fn fword_ptr(&self, at: usize) -> *mut c_char {
        // SAFETY: the caller guarantees the index.
        unsafe { self.fword.add(at) }
    }

    /// The length of the word currently in `preword`.
    fn preword_len(&self) -> usize {
        // SAFETY: `preword` is NUL-terminated wherever this is called, and
        // it is this module's own buffer.
        unsafe { strlen(self.preword.as_ptr()) as usize }
    }

    /// Step a pointer back over the character before it, as the C's
    /// `MB_PTR_BACK` did.
    ///
    /// # Safety
    ///
    /// `p` must be inside the string starting at `base`, past its start.
    #[inline]
    unsafe fn char_back(base: *const c_char, p: *mut c_char) -> *mut c_char {
        // SAFETY: the caller guarantees the pointers.
        unsafe { p.sub(utf_head_off(base, p.sub(1)) as usize + 1) }
    }
}
