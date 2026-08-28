//! The word tree `:mkspell` builds, and the arena it lives in.
//!
//! Every word of a dictionary is added to a trie: one node per byte, with
//! siblings chained through [`wordnode_S::wn_sibling`] in ascending byte
//! order and the continuation hanging off [`wordnode_S::wn_child`]. A node
//! whose byte is NUL terminates a word and carries that word's flags,
//! region mask and affix id instead of a child.
//!
//! Three trees get built side by side — case-folded words, keep-case
//! words, and prefixes — plus, for a `.sug` file, a fourth over
//! sound-folded words. [`store_word`] feeds the first two; the rest call
//! [`tree_add_word`] directly.
//!
//! # Sharing tails
//!
//! A dictionary's words share endings, so the finished trie is compressed:
//! [`wordtree_compress`] walks it bottom-up and replaces each child chain
//! with an identical one already seen, bumping that one's reference count.
//! The lookup is a hash table keyed on a five-byte digest stored inside the
//! node itself ([`wordnode_S::wn_digest`]), so no separate key allocation is
//! needed — the table's key pointer points *into* the node, and
//! [`node_of_digest_key`] is how [`node_compress`] gets back out.
//!
//! Because a compressed sub-tree is shared, [`tree_add_word`] has to
//! un-share any node it is about to modify: a node with more than one
//! reference is copied first. Compression therefore runs *during* the
//! build as well as at the end, whenever the arena has grown past the
//! threshold `'mkspellmem'` sets.
//!
//! # Node identity is load-bearing
//!
//! Nodes stay raw pointers rather than indices. Their addresses feed the
//! digest in [`node_compress`], and [`node_equal`] tests two sub-trees for
//! equality by comparing child *pointers* — a child chain is only equal to
//! another if compression has already made them the same object. Both fall
//! out of the arena never moving anything it has handed out.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint};
use core::{mem, ptr};

use crate::global_cell::GlobalCell;
use crate::hashtab::{hash_add_item, hash_clear, hash_hash, hash_init, hash_lookup, hash_removed};
use crate::main::{curwin, got_int, msg_col, msg_didout, p_verbose};
use crate::mbyte::{utf_valid_string, utfc_ptr2len};
use crate::message::{msg_clr_eos, msg_puts, msg_start};
use crate::os::cshim::gettext;
use crate::os::input::veryfast_breakcheck;
use crate::spell::{captype, spell_casefold};
use crate::types::{NUL, hashtab_T, int16_t, uint8_t, uint16_t};
use crate::ui::ui_flush;
use ::libc::strlen;

use super::{FAIL, MAXWLEN, OK, WF_KEEPCAP, spell_message_fmt, spellinfo_T};

/// Bytes handed out per arena block.
const SBLOCKSIZE: usize = 16000;

/// The part of a word node's flags that is stored in the tree; the caller
/// passes extra bits above it that only steer where the word is filed.
pub(super) const WN_MASK: c_int = 0xffff;

pub(super) const MSG_COMPRESSING: &core::ffi::CStr = c"Compressing word tree...";

/// A bump allocator for everything a spell file under construction owns.
///
/// The affix tables, the condition strings, and every word node are
/// allocated once and released together, so individual frees would only be
/// bookkeeping. Blocks are `u64`-typed to make the base eight-byte aligned;
/// `align` on [`SpellArena::alloc_bytes`] rounds the cursor up to the same
/// boundary for allocations that will hold a struct.
///
/// Blocks are never reallocated, so a pointer handed out stays valid until
/// [`SpellArena::clear`] — which invalidates all of them at once, and is
/// only called where the whole tree is being abandoned.
pub struct SpellArena {
    blocks: Vec<Vec<u64>>,
    /// Start of the newest block, or null while there is none.
    head: *mut u8,
    /// Bytes handed out from the newest block.
    used: usize,
    /// Blocks taken since the compression heuristic last reset the count.
    /// It is not `blocks.len()`: [`tree_add_word`] subtracts from it after
    /// each compression run, and the `.sug` pass zeroes it per word.
    pub block_count: c_int,
}

impl SpellArena {
    pub const fn new() -> Self {
        Self {
            blocks: Vec::new(),
            head: ptr::null_mut(),
            used: 0,
            block_count: 0,
        }
    }

    /// Hand out `len` zeroed bytes, aligned for a struct when `align`.
    pub fn alloc_bytes(&mut self, len: usize, align: bool) -> *mut c_char {
        debug_assert!(len <= SBLOCKSIZE);
        if align && !self.head.is_null() {
            self.used = self.used.next_multiple_of(align_of::<*mut c_char>());
        }
        if self.head.is_null() || self.used + len > SBLOCKSIZE {
            let mut block = vec![0u64; SBLOCKSIZE / size_of::<u64>()];
            self.head = block.as_mut_ptr().cast::<u8>();
            self.blocks.push(block);
            self.used = 0;
            self.block_count += 1;
        }
        // SAFETY: the block holds SBLOCKSIZE bytes and the test above
        // leaves `used + len` within it.
        let p = unsafe { self.head.add(self.used) };
        self.used += len;
        p.cast::<c_char>()
    }

    /// Hand out one zeroed `T`.
    pub fn alloc<T>(&mut self) -> *mut T {
        const { assert!(align_of::<T>() <= align_of::<*mut c_char>()) };
        self.alloc_bytes(size_of::<T>(), true).cast::<T>()
    }

    /// Copy a NUL-terminated string into the arena.
    ///
    /// # Safety
    ///
    /// `s` must point at a NUL-terminated string.
    pub unsafe fn save_str(&mut self, s: *const c_char) -> *mut c_char {
        // SAFETY: the caller promises a terminated string, and the arena
        // just handed out `size` writable bytes.
        let size = unsafe { strlen(s) } + 1;
        let p = self.alloc_bytes(size, false);
        unsafe { ptr::copy_nonoverlapping(s, p, size) };
        p
    }

    /// Release every block. All pointers handed out become dangling.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.head = ptr::null_mut();
        self.used = 0;
        self.block_count = 0;
    }
}

pub(super) type wordnode_T = wordnode_S;

/// One byte of one word, or — when [`wn_byte`](Self::wn_byte) is NUL — the
/// end of a word and the properties that go with it.
///
/// # Two phases, three scratch fields
///
/// A node is scribbled on twice over its life, and upstream overlays each
/// pair of uses in a union. Both overlays are gone here, for different
/// reasons.
///
/// The digest and the index are genuinely different types — five bytes the
/// hash table reads as a string, against an `int` — so they are two fields
/// ([`wn_digest`](Self::wn_digest), [`wn_index`](Self::wn_index)). That
/// costs nothing: the union's own tail padding was four bytes wide, so the
/// node is the same size either way, which
/// `a_node_costs_no_more_than_the_union` pins down.
///
/// [`wn_link`](Self::wn_link) is the other overlay, whose two arms were
/// *both* `*mut wordnode_T` — documentation of the phase change rather
/// than a representation. It is one field, and the phases are written down
/// on it instead.
#[derive(Copy, Clone)]
pub struct wordnode_S {
    /// Compression phase: five digest bytes over this node's whole sibling
    /// chain, plus a terminator, so the hash table can key on it as a
    /// string. None of the five is ever zero. See [`node_compress`].
    pub wn_digest: [uint8_t; 6],
    /// Write phase: this node's index in the written tree, or 0 while it
    /// has not been placed. See [`put_node`](super::write::put_node).
    pub wn_index: c_int,
    /// Whichever node currently has a claim on this one — the two phases
    /// mean different things by that, and never overlap:
    ///
    /// - while compressing, the next node whose sibling chain hashes to the
    ///   same [`wn_digest`](Self::wn_digest), so that [`node_compress`] can
    ///   walk a bucket looking for a chain worth sharing;
    /// - while writing, the sibling chain that claimed this node and will
    ///   therefore emit it; every other parent emits a
    ///   [`BY_INDEX`](super::BY_INDEX) reference instead.
    ///
    /// [`clear_node`](super::write::clear_node) is the handover: it drops
    /// the compression meaning and starts the write one.
    pub wn_link: *mut wordnode_T,
    pub wn_child: *mut wordnode_T,
    pub wn_sibling: *mut wordnode_T,
    /// How many parents point here; above one the sub-tree is shared.
    pub wn_refs: c_int,
    pub wn_byte: uint8_t,
    pub wn_affixID: uint8_t,
    pub wn_flags: uint16_t,
    pub wn_region: int16_t,
}

/// The digest of `node`'s sibling chain, as the NUL-terminated string the
/// compression hash table keys on.
///
/// The table stores key *pointers*, so this is also how a node gets into
/// the table at all: [`node_of_digest_key`] is the way back.
fn digest_key(node: *mut wordnode_T) -> *mut c_char {
    // SAFETY: the offset lands inside the node; nothing is read.
    unsafe { (&raw mut (*node).wn_digest).cast::<c_char>() }
}

/// The node a [`digest_key`] came from.
///
/// # Safety
///
/// `key` must be a pointer [`digest_key`] returned for a still-live node.
unsafe fn node_of_digest_key(key: *mut c_char) -> *mut wordnode_T {
    // SAFETY: the caller promises the pointer names a live node's digest
    // field, so stepping back over the field's offset lands on the node.
    unsafe { key.byte_sub(mem::offset_of!(wordnode_S, wn_digest)).cast() }
}

/// Arena blocks in use before the first compression run.
static compress_start: GlobalCell<c_int> = GlobalCell::new(30_000);
/// Blocks that have to be taken again before compressing again.
static compress_inc: GlobalCell<c_int> = GlobalCell::new(100);
/// Words to add after a compression run before considering another.
static compress_added: GlobalCell<c_int> = GlobalCell::new(500_000);

/// Install the `'mkspellmem'` values, already scaled to blocks.
pub(super) fn set_compression_limits(start: c_int, incr: c_int, added: c_int) {
    compress_start.set(start);
    compress_inc.set(incr);
    compress_added.set(added);
}

/// The `'mkspellmem'` block count a run starts at, for validating the option.
pub(super) const fn block_size() -> c_int {
    SBLOCKSIZE as c_int
}

/// Allocate a tree's root node.
pub(super) fn wordtree_alloc(spin: &mut spellinfo_T) -> *mut wordnode_T {
    spin.si_arena.alloc::<wordnode_T>()
}

/// Reject words a spell file cannot represent: invalid UTF-8, control
/// characters, and a trailing `/` (which would read as a flag separator).
///
/// # Safety
///
/// `word` and `end` must delimit one readable range.
pub(super) unsafe fn valid_spell_word(word: *const c_char, end: *const c_char) -> bool {
    // SAFETY: the caller promises the range; the walk stops at `end`.
    if !unsafe { utf_valid_string(word, end) } {
        return false;
    }
    let mut p = word;
    while unsafe { *p } as c_int != NUL && p < end {
        if (unsafe { *p } as uint8_t as c_int) < b' ' as c_int
            || (unsafe { *p } as c_int == b'/' as c_int && unsafe { *p.add(1) } as c_int == NUL)
        {
            return false;
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    true
}

/// File one word in the case-folded tree, and in the keep-case tree when
/// its capitalisation cannot be derived from the folded form.
///
/// `pfxlist` is the set of affix ids the word takes; the word is added once
/// per id, since which prefixes apply is part of what the tree records.
/// When `need_affix` is set the word is only added for real affix ids, not
/// for the bare word.
///
/// # Safety
///
/// `word` must be NUL-terminated, and `pfxlist` either null or likewise.
pub(super) unsafe fn store_word(
    spin: &mut spellinfo_T,
    word: *mut c_char,
    flags: c_int,
    region: c_int,
    pfxlist: *const c_char,
    need_affix: bool,
) -> c_int {
    // SAFETY: the caller promises terminated strings; `foldword` is a
    // MAXWLEN buffer, which is the bound spell_casefold is given.
    let len = unsafe { strlen(word) } as c_int;
    let ct = unsafe { captype(word, word.offset(len as isize)) };
    let mut foldword: [c_char; MAXWLEN] = [0; MAXWLEN];
    let mut res = OK;

    if !unsafe { valid_spell_word(word, word.offset(len as isize)) } {
        return FAIL;
    }

    let (win, out) = (curwin.get(), foldword.as_mut_ptr());
    unsafe { spell_casefold(win, word, len, out, MAXWLEN as c_int) };

    let root = spin.si_foldroot;
    let folded = foldword.as_ptr();
    let with_case = ct | flags;
    res = unsafe { add_per_affix(spin, folded, root, with_case, region, pfxlist, need_affix) };
    spin.si_foldwcount += 1;

    if res == OK && (ct == WF_KEEPCAP as c_int || flags & WF_KEEPCAP as c_int != 0) {
        let root = spin.si_keeproot;
        res = unsafe { add_per_affix(spin, word, root, flags, region, pfxlist, need_affix) };
        spin.si_keepwcount += 1;
    }
    res
}

/// Add `word` to `root` once per affix id in `pfxlist`, stopping at the
/// first failure.
///
/// A null or empty `pfxlist` still adds the word once, under affix id 0 —
/// unless `need_affix`, which means only real ids count.
///
/// # Safety
///
/// `word` must be NUL-terminated, `pfxlist` null or NUL-terminated, and
/// `root` a node of this arena's tree.
unsafe fn add_per_affix(
    spin: &mut spellinfo_T,
    word: *const c_char,
    root: *mut wordnode_T,
    flags: c_int,
    region: c_int,
    pfxlist: *const c_char,
    need_affix: bool,
) -> c_int {
    // SAFETY: the caller promises the strings and the root; the walk stops
    // at `pfxlist`'s NUL.
    let mut res = OK;
    let mut p = pfxlist;
    while res == OK {
        let affix_id = if p.is_null() {
            0
        } else {
            unsafe { *p as c_int }
        };
        if !need_affix || affix_id != NUL {
            res = unsafe { tree_add_word(spin, word, root, flags, region, affix_id) };
        }
        if p.is_null() || unsafe { *p } as c_int == NUL {
            break;
        }
        p = unsafe { p.add(1) };
    }
    res
}

/// Add one word to `root`, and compress the trees when the arena has grown
/// past the threshold.
///
/// A negative `flags` means the prefix tree, where `affixID` rather than
/// the flags decides sibling order and no two entries are ever merged.
///
/// # Safety
///
/// `word` must be NUL-terminated and `root` a node of this arena's tree.
pub(super) unsafe fn tree_add_word(
    spin: &mut spellinfo_T,
    word: *const c_char,
    root: *mut wordnode_T,
    flags: c_int,
    region: c_int,
    affixID: c_int,
) -> c_int {
    // SAFETY: nodes come from the arena and outlive the call; the walk
    // follows `word` only up to its NUL.
    let mut node = root;
    // Where to write back a replaced node: the parent's child or
    // sibling slot, or null at the root.
    let mut prev: *mut *mut wordnode_T = ptr::null_mut();
    let mut i: isize = 0;

    loop {
        // A shared chain has to be copied before it can be changed.
        if !node.is_null() && unsafe { (*node).wn_refs } > 1 {
            unsafe { (*node).wn_refs -= 1 };
            let mut copyprev = prev;
            let mut copyp = node;
            while !copyp.is_null() {
                let np = get_wordnode(spin);
                if np.is_null() {
                    return FAIL;
                }
                unsafe { (*np).wn_child = (*copyp).wn_child };
                if !unsafe { (*np).wn_child }.is_null() {
                    unsafe { (*(*np).wn_child).wn_refs += 1 };
                }
                unsafe { (*np).wn_byte = (*copyp).wn_byte };
                if unsafe { (*np).wn_byte } as c_int == NUL {
                    unsafe { (*np).wn_flags = (*copyp).wn_flags };
                    unsafe { (*np).wn_region = (*copyp).wn_region };
                    unsafe { (*np).wn_affixID = (*copyp).wn_affixID };
                }
                unsafe { (*np).wn_refs = 1 };
                if !copyprev.is_null() {
                    unsafe { *copyprev = np };
                }
                copyprev = unsafe { &raw mut (*np).wn_sibling };
                if copyp == node {
                    node = np;
                }
                copyp = unsafe { (*copyp).wn_sibling };
            }
        }

        // Skip siblings that sort before what is being added.
        while !node.is_null()
            && unsafe { sorts_before(spin, node, *word.offset(i), flags, region, affixID) }
        {
            prev = unsafe { &raw mut (*node).wn_sibling };
            node = unsafe { *prev };
        }

        // Insert a node when there is no matching one here. Word ends
        // never merge in the prefix or sound-fold trees, and only merge
        // elsewhere when the flags and affix id match too.
        let need_new = node.is_null()
            || unsafe { (*node).wn_byte } as c_int
                != unsafe { *word.offset(i) } as uint8_t as c_int
            || (unsafe { *word.offset(i) } as c_int == NUL
                && (flags < 0
                    || spin.si_sugtree != 0
                    || unsafe { (*node).wn_flags } as c_int != flags & WN_MASK
                    || unsafe { (*node).wn_affixID } as c_int != affixID));
        if need_new && !unsafe { insert_before(spin, &mut node, prev, *word.offset(i)) } {
            return FAIL;
        }

        if unsafe { *word.offset(i) } as c_int == NUL {
            unsafe { (*node).wn_flags = flags as uint16_t };
            unsafe { (*node).wn_region |= region as int16_t };
            unsafe { (*node).wn_affixID = affixID as uint8_t };
            break;
        }
        prev = unsafe { &raw mut (*node).wn_child };
        node = unsafe { *prev };
        i += 1;
    }

    spin.si_msg_count += 1;

    // Count down to the next compression run, then arm the "arena has
    // grown again" test by adding the increment back.
    if spin.si_compress_cnt > 1 {
        spin.si_compress_cnt -= 1;
        if spin.si_compress_cnt == 1 {
            spin.si_arena.block_count += compress_inc.get();
        }
    }
    let due = if spin.si_compress_cnt == 1 {
        spin.si_free_count < MAXWLEN as c_int
    } else {
        spin.si_arena.block_count >= compress_start.get()
    };
    if due {
        spin.si_arena.block_count -= compress_inc.get();
        spin.si_compress_cnt = compress_added.get();
        if spin.si_verbose != 0 {
            unsafe { msg_start() };
            unsafe { msg_puts(gettext(MSG_COMPRESSING.as_ptr())) };
            unsafe { msg_clr_eos() };
            msg_didout.set(false);
            msg_col.set(0);
            unsafe { ui_flush() };
        }
        unsafe { wordtree_compress(spin, spin.si_foldroot, c"case-folded") };
        if affixID >= 0 {
            unsafe { wordtree_compress(spin, spin.si_keeproot, c"keep-case") };
        }
    }
    OK
}

/// Does `node` sort before the entry being added?
///
/// # Safety
///
/// `node` must be a live node.
#[inline]
unsafe fn sorts_before(
    spin: &spellinfo_T,
    node: *mut wordnode_T,
    byte: c_char,
    flags: c_int,
    region: c_int,
    affixID: c_int,
) -> bool {
    // SAFETY: the caller promises a live node.
    if (unsafe { (*node).wn_byte } as c_int) < byte as uint8_t as c_int {
        return true;
    }
    if unsafe { (*node).wn_byte } as c_int != NUL {
        return false;
    }
    // Word ends sort among themselves: by affix id in the prefix tree,
    // otherwise by flags and then by region or affix id.
    if flags < 0 {
        return (unsafe { (*node).wn_affixID } as c_uint) < affixID as c_uint;
    }
    if (unsafe { (*node).wn_flags } as c_uint) < (flags & WN_MASK) as c_uint {
        return true;
    }
    if unsafe { (*node).wn_flags } as c_int != flags & WN_MASK {
        return false;
    }
    if spin.si_sugtree != 0 {
        (unsafe { (*node).wn_region } as c_int & 0xffff) < region
    } else {
        (unsafe { (*node).wn_affixID } as c_uint) < affixID as c_uint
    }
}

/// Splice a fresh node carrying `byte` in ahead of `*node`. Returns false
/// when no node could be had.
///
/// # Safety
///
/// `prev`, when non-null, must point at the slot holding `*node`.
#[inline]
unsafe fn insert_before(
    spin: &mut spellinfo_T,
    node: &mut *mut wordnode_T,
    prev: *mut *mut wordnode_T,
    byte: c_char,
) -> bool {
    let np = get_wordnode(spin);
    if np.is_null() {
        return false;
    }
    // SAFETY: `np` is a fresh arena node, `*node` a live one or null, and
    // `prev` the slot that holds it.
    unsafe { (*np).wn_byte = byte as uint8_t };
    // The new node inherits the chain's reference count; the old head
    // is now reached only through it.
    if node.is_null() {
        unsafe { (*np).wn_refs = 1 };
    } else {
        unsafe { (*np).wn_refs = (**node).wn_refs };
        unsafe { (**node).wn_refs = 1 };
    }
    if !prev.is_null() {
        unsafe { *prev = np };
    }
    unsafe { (*np).wn_sibling = *node };
    *node = np;
    true
}

/// Take a node from the free list, or a fresh one from the arena.
fn get_wordnode(spin: &mut spellinfo_T) -> *mut wordnode_T {
    let n = spin.si_first_free;
    if n.is_null() {
        return spin.si_arena.alloc::<wordnode_T>();
    }
    // SAFETY: the free list only holds nodes this module released.
    spin.si_first_free = unsafe { (*n).wn_child };
    unsafe { *n = mem::zeroed() };
    spin.si_free_count -= 1;
    n
}

/// Drop one reference to `node`; when the last goes, return its whole
/// sibling chain and their sub-trees to the free list.
///
/// Returns how many nodes were freed, for the compression report.
///
/// # Safety
///
/// `node` must be a live node of this arena's tree.
unsafe fn deref_wordnode(spin: &mut spellinfo_T, node: *mut wordnode_T) -> c_int {
    // SAFETY: the caller promises a live node; the walk stays inside the
    // tree it roots.
    let mut cnt = 0;
    unsafe { (*node).wn_refs -= 1 };
    if unsafe { (*node).wn_refs } == 0 {
        let mut np = node;
        while !np.is_null() {
            if !unsafe { (*np).wn_child }.is_null() {
                cnt += unsafe { deref_wordnode(spin, (*np).wn_child) };
            }
            unsafe { free_wordnode(spin, np) };
            cnt += 1;
            np = unsafe { (*np).wn_sibling };
        }
        cnt += 1;
    }
    cnt
}

/// Put one node on the free list, chained through its child slot.
///
/// # Safety
///
/// `n` must be a node no longer reachable from any tree.
unsafe fn free_wordnode(spin: &mut spellinfo_T, n: *mut wordnode_T) {
    // SAFETY: the caller promises the node is unreachable.
    unsafe { (*n).wn_child = spin.si_first_free };
    spin.si_first_free = n;
    spin.si_free_count += 1;
}

/// Compress a whole tree and report how much it shrank.
///
/// # Safety
///
/// `root` must be the root node of one of this arena's trees.
pub(super) unsafe fn wordtree_compress(
    spin: &mut spellinfo_T,
    root: *mut wordnode_T,
    name: &core::ffi::CStr,
) {
    // SAFETY: the caller promises a live root.
    // The root's own chain is what holds the tree; an empty one has
    // nothing to share.
    if unsafe { (*root).wn_sibling }.is_null() {
        return;
    }

    let mut ht: hashtab_T = unsafe { mem::zeroed() };
    unsafe { hash_init(&raw mut ht) };
    let mut tot: c_int = 0;
    let n = unsafe { node_compress(spin, (*root).wn_sibling, &raw mut ht, &mut tot) };

    if spin.si_verbose != 0 || p_verbose.get() > 2 {
        let perc = remaining_percentage(n, tot);
        let (name, left) = (name.to_string_lossy(), tot - n);
        spell_message_fmt(
            spin,
            format_args!("Compressed {name}: {n} of {tot} nodes; {left} ({perc}%) remaining"),
        );
    }
    unsafe { hash_clear(&raw mut ht) };
}

/// What share of `total` nodes survived compression, as a percentage, given
/// that `compressed` of them were shared away.
///
/// Big trees are scaled down before the multiply rather than after, so the
/// product cannot overflow; the two arms therefore disagree by a rounding
/// step, which is what the reported figure has always done.
fn remaining_percentage(compressed: c_int, total: c_int) -> core::ffi::c_long {
    let remaining = total - compressed;
    if total > 1_000_000 {
        (remaining / (total / 100)).into()
    } else if total == 0 {
        0
    } else {
        (remaining * 100 / total).into()
    }
}

/// Compress one sibling chain and everything below it, returning how many
/// nodes the sharing removed. `tot` accumulates the nodes seen.
///
/// # Safety
///
/// `node` must head a live sibling chain and `ht` be an initialised table
/// holding only nodes of the same tree.
unsafe fn node_compress(
    spin: &mut spellinfo_T,
    node: *mut wordnode_T,
    ht: *mut hashtab_T,
    tot: &mut c_int,
) -> c_int {
    // SAFETY: the caller promises a live chain and a live table; every key
    // in the table was entered below as a node address.
    let mut compressed = 0;
    let mut len = 0;

    for np in unsafe { siblings(node) } {
        if got_int.get() {
            break;
        }
        len += 1;
        let child = unsafe { (*np).wn_child };
        if !child.is_null() {
            // Depth first: the child's digest is only meaningful once
            // everything below it has been compressed.
            compressed += unsafe { node_compress(spin, child, ht, tot) };

            // The key is the node's own digest field, so the table
            // borrows storage the node already owns and every entry
            // names a node.
            let key = digest_key(child);
            let hash = unsafe { hash_hash(key) };
            let hi = unsafe { hash_lookup(ht, key, strlen(key), hash) };
            if unsafe { (*hi).hi_key }.is_null()
                || unsafe { (*hi).hi_key } == (&raw const hash_removed).cast_mut().cast()
            {
                unsafe { hash_add_item(ht, hi, key, hash) };
                continue;
            }

            // Same digest: walk the chain of nodes sharing it looking
            // for a genuinely equal sub-tree to point at instead.
            let mut tp = unsafe { node_of_digest_key((*hi).hi_key) };
            while !tp.is_null() {
                if unsafe { node_equal(child, tp) } {
                    unsafe { (*tp).wn_refs += 1 };
                    compressed += unsafe { deref_wordnode(spin, child) };
                    unsafe { (*np).wn_child = tp };
                    break;
                }
                tp = unsafe { (*tp).wn_link };
            }
            if tp.is_null() {
                // No match; join the chain for the next comer.
                let head = unsafe { node_of_digest_key((*hi).hi_key) };
                unsafe { (*child).wn_link = (*head).wn_link };
                unsafe { (*head).wn_link = child };
            }
        }
    }
    *tot += len + 1;
    unsafe { write_digest(node, len) };

    veryfast_breakcheck();
    compressed
}

/// Digest `node`'s whole sibling chain into [`wordnode_S::wn_digest`]: the
/// chain's length, then a rolling hash over every sibling's byte and either
/// its child pointer or, at a word end, its flags.
///
/// Each of the five bytes is forced non-zero so that together they form a
/// NUL-terminated string the hash table can key on. `len` is the chain's
/// length as [`node_compress`] counted it, truncated — a longer chain only
/// collides more often, and [`node_equal`] is what decides equality.
///
/// # Safety
///
/// `node` must head a live sibling chain.
unsafe fn write_digest(node: *mut wordnode_T, len: c_int) {
    // SAFETY: the caller promises a live chain, and `siblings` walks it to
    // its null terminator without changing it.
    let mut nr: c_uint = 0;
    for np in unsafe { siblings(node) } {
        let n: c_uint = if unsafe { (*np).wn_byte } as c_int == NUL {
            (unsafe { (*np).wn_flags } as c_int
                + ((unsafe { (*np).wn_region } as c_int) << 8)
                + ((unsafe { (*np).wn_affixID } as c_int) << 16)) as c_uint
        } else {
            (unsafe { (*np).wn_byte } as usize).wrapping_add(
                unsafe { (*np).wn_child }
                    .expose_provenance()
                    .wrapping_shl(8),
            ) as c_uint
        };
        nr = nr.wrapping_mul(101).wrapping_add(n);
    }

    let digest = unsafe { &mut (*node).wn_digest };
    digest[0] = len as uint8_t;
    for (i, shift) in [0, 8, 16, 24].into_iter().enumerate() {
        let b = (nr >> shift & 0xff) as uint8_t;
        digest[i + 1] = if b == 0 { 1 } else { b };
    }
    digest[5] = NUL as uint8_t;
}

/// The nodes of a sibling chain, `node` first.
///
/// # Safety
///
/// `node` must be null or head a live sibling chain, and nothing may change
/// a `wn_sibling` link in it while the iterator is alive.
unsafe fn siblings(node: *mut wordnode_T) -> impl Iterator<Item = *mut wordnode_T> {
    let mut next = node;
    core::iter::from_fn(move || {
        let cur = next;
        if cur.is_null() {
            return None;
        }
        // SAFETY: the caller promises a live chain, so every node reached
        // this way is live and its sibling link readable.
        next = unsafe { (*cur).wn_sibling };
        Some(cur)
    })
}

/// Are two sibling chains interchangeable?
///
/// Children are compared by pointer, not by content: by the time a chain is
/// looked at, everything below it has already been compressed, so equal
/// sub-trees are the same object.
///
/// # Safety
///
/// Both arguments must head live sibling chains.
unsafe fn node_equal(n1: *mut wordnode_T, n2: *mut wordnode_T) -> bool {
    // SAFETY: the caller promises live chains.
    let mut p1 = n1;
    let mut p2 = n2;
    while !p1.is_null() && !p2.is_null() {
        if unsafe { (*p1).wn_byte } != unsafe { (*p2).wn_byte } {
            break;
        }
        let differs = if unsafe { (*p1).wn_byte } as c_int == NUL {
            unsafe {
                (*p1).wn_flags != (*p2).wn_flags
                    || (*p1).wn_region != (*p2).wn_region
                    || (*p1).wn_affixID != (*p2).wn_affixID
            }
        } else {
            unsafe { (*p1).wn_child != (*p2).wn_child }
        };
        if differs {
            break;
        }
        p1 = unsafe { (*p1).wn_sibling };
        p2 = unsafe { (*p2).wn_sibling };
    }
    p1.is_null() && p2.is_null()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_node() -> wordnode_S {
        wordnode_S {
            wn_digest: [0; 6],
            wn_index: 0,
            wn_link: ptr::null_mut(),
            wn_child: ptr::null_mut(),
            wn_sibling: ptr::null_mut(),
            wn_refs: 0,
            wn_byte: 0,
            wn_affixID: 0,
            wn_flags: 0,
            wn_region: 0,
        }
    }

    /// What the overlay was worth, measured rather than assumed: the union
    /// of `[u8; 6]` and `c_int` was eight bytes with four of them padding,
    /// which is exactly what an `int` beside the array costs. One node per
    /// byte of dictionary is why anyone cares.
    ///
    /// Measured over the *fields*, not over `size_of::<wordnode_S>()`.
    /// `wordnode_S` is `repr(Rust)`, so a layout-randomising build is free
    /// to order it badly and pad it past the `repr(C)` original — which
    /// would say something about that build and nothing about this struct.
    /// Summing the declared fields is the comparison that means what the
    /// paragraph above says, and it holds whatever the compiler does with
    /// the order.
    #[test]
    fn a_node_costs_no_more_than_the_union() {
        /// `wordnode_S` as upstream lays it out, with both overlays.
        #[repr(C)]
        struct AsUnion {
            u1: [uint8_t; 8],
            u2: *mut u8,
            child: *mut u8,
            sibling: *mut u8,
            refs: c_int,
            byte: uint8_t,
            affix_id: uint8_t,
            flags: uint16_t,
            region: int16_t,
        }
        // Exhaustive: a field added to `wordnode_S` stops compiling here,
        // which is the half of the claim a size cannot make.
        let wordnode_S {
            wn_digest,
            wn_index,
            wn_link,
            wn_child,
            wn_sibling,
            wn_refs,
            wn_byte,
            wn_affixID,
            wn_flags,
            wn_region,
        } = blank_node();
        let fields = size_of_val(&wn_digest)
            + size_of_val(&wn_index)
            + size_of_val(&wn_link)
            + size_of_val(&wn_child)
            + size_of_val(&wn_sibling)
            + size_of_val(&wn_refs)
            + size_of_val(&wn_byte)
            + size_of_val(&wn_affixID)
            + size_of_val(&wn_flags)
            + size_of_val(&wn_region);
        assert!(
            fields <= size_of::<AsUnion>(),
            "{fields} bytes of fields against {} for the union",
            size_of::<AsUnion>()
        );
        assert_eq!(align_of::<wordnode_S>(), align_of::<AsUnion>());
    }

    /// The compression table stores a pointer into the node, not to it, so
    /// the way back is the field's offset — no longer a promise that the
    /// digest sits first.
    #[test]
    fn a_digest_key_names_the_node_it_came_from() {
        let mut node = blank_node();
        let at = &raw mut node;
        let key = digest_key(at);
        assert_eq!(key.cast::<uint8_t>(), (&raw mut node.wn_digest).cast());
        // SAFETY: `key` is this node's digest field and the node is alive.
        assert_eq!(unsafe { node_of_digest_key(key) }, at);
    }

    /// The five digest bytes have to read as a NUL-terminated string of
    /// length five, whatever the chain hashes to: the table calls `strlen`
    /// on them.
    #[test]
    fn a_digest_is_five_non_zero_bytes_and_a_terminator() {
        let mut tail = blank_node();
        tail.wn_byte = b'x';
        let mut head = blank_node();
        head.wn_sibling = &raw mut tail;
        // SAFETY: the two nodes form a live chain that outlives the call.
        unsafe { write_digest(&raw mut head, 2) };
        assert_eq!(head.wn_digest[0], 2);
        assert!(head.wn_digest[1..5].iter().all(|&b| b != 0));
        assert_eq!(head.wn_digest[5], NUL as uint8_t);
    }

    /// The point of the split: the write phase's index no longer lands on
    /// top of the compression phase's digest. Under the union, storing an
    /// index cleared the first four digest bytes, which is why
    /// `clear_node` could be read as "forget the digests" as well.
    #[test]
    fn the_index_and_the_digest_are_separate_storage() {
        let mut node = blank_node();
        // SAFETY: a one-node chain, alive for the call.
        unsafe { write_digest(&raw mut node, 1) };
        let digest = node.wn_digest;
        node.wn_index = 0x0102_0304;
        node.wn_link = &raw mut node;
        assert_eq!(node.wn_digest, digest);
        assert_ne!(node.wn_digest[0], 0);
    }
}
