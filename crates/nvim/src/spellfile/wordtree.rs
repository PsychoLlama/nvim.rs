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
//! node itself ([`wordnode_S::wn_u1`]), so no separate key allocation is
//! needed — the table's key pointer *is* the node pointer, which is how
//! [`node_compress`] casts one back to the other.
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
use crate::main::{IObuff, curwin, got_int, msg_col, msg_didout, p_verbose};
use crate::mbyte::{utf_valid_string, utfc_ptr2len};
use crate::message::{msg_clr_eos, msg_puts, msg_start};
use crate::os::cshim::gettext;
use crate::os::input::veryfast_breakcheck;
use crate::spell::{captype, spell_casefold};
use crate::strings::vim_snprintf;
use crate::types::{NUL, hashtab_T, int16_t, uint8_t, uint16_t};
use crate::ui::ui_flush;
use ::libc::strlen;

use super::{FAIL, IOSIZE, MAXWLEN, OK, WF_KEEPCAP, spell_message, spellinfo_T};

/// Bytes handed out per arena block.
const SBLOCKSIZE: usize = 16000;

/// The part of a word node's flags that is stored in the tree; the caller
/// passes extra bits above it that only steer where the word is filed.
pub const WN_MASK: c_int = 0xffff;

pub const MSG_COMPRESSING: &core::ffi::CStr = c"Compressing word tree...";

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
            self.used = self.used.next_multiple_of(mem::align_of::<*mut c_char>());
        }
        if self.head.is_null() || self.used + len > SBLOCKSIZE {
            let mut block = vec![0u64; SBLOCKSIZE / mem::size_of::<u64>()];
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
        const { assert!(mem::align_of::<T>() <= mem::align_of::<*mut c_char>()) };
        self.alloc_bytes(mem::size_of::<T>(), true).cast::<T>()
    }

    /// Copy a NUL-terminated string into the arena.
    ///
    /// # Safety
    ///
    /// `s` must point at a NUL-terminated string.
    pub unsafe fn save_str(&mut self, s: *const c_char) -> *mut c_char {
        // SAFETY: the caller promises a terminated string, and the arena
        // just handed out `size` writable bytes.
        unsafe {
            let size = strlen(s) + 1;
            let p = self.alloc_bytes(size, false);
            ptr::copy_nonoverlapping(s, p, size);
            p
        }
    }

    /// Release every block. All pointers handed out become dangling.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.head = ptr::null_mut();
        self.used = 0;
        self.block_count = 0;
    }
}

pub type wordnode_T = wordnode_S;

/// One byte of one word, or — when [`wn_byte`](Self::wn_byte) is NUL — the
/// end of a word and the properties that go with it.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct wordnode_S {
    /// The compression digest while compressing, then the node's index in
    /// the written tree while the file is being put together. Must stay
    /// first: [`node_compress`] relies on a node's address and its
    /// digest's address being the same.
    pub wn_u1: WordNodeKey,
    /// The next node with the same digest while compressing, then the
    /// already-written twin while the file is being put together.
    pub wn_u2: WordNodeLink,
    pub wn_child: *mut wordnode_T,
    pub wn_sibling: *mut wordnode_T,
    /// How many parents point here; above one the sub-tree is shared.
    pub wn_refs: c_int,
    pub wn_byte: uint8_t,
    pub wn_affixID: uint8_t,
    pub wn_flags: uint16_t,
    pub wn_region: int16_t,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union WordNodeKey {
    /// Five digest bytes and a terminator, so the hash table can treat it
    /// as a string. None of the five is ever zero.
    pub hashkey: [uint8_t; 6],
    pub index: c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union WordNodeLink {
    pub next: *mut wordnode_T,
    pub wnode: *mut wordnode_T,
}

const _: () = assert!(mem::offset_of!(wordnode_S, wn_u1) == 0);

/// Arena blocks in use before the first compression run.
static compress_start: GlobalCell<c_int> = GlobalCell::new(30_000);
/// Blocks that have to be taken again before compressing again.
static compress_inc: GlobalCell<c_int> = GlobalCell::new(100);
/// Words to add after a compression run before considering another.
static compress_added: GlobalCell<c_int> = GlobalCell::new(500_000);

/// Install the `'mkspellmem'` values, already scaled to blocks.
pub fn set_compression_limits(start: c_int, incr: c_int, added: c_int) {
    compress_start.set(start);
    compress_inc.set(incr);
    compress_added.set(added);
}

/// The `'mkspellmem'` block count a run starts at, for validating the option.
pub const fn block_size() -> c_int {
    SBLOCKSIZE as c_int
}

/// Allocate a tree's root node.
pub fn wordtree_alloc(spin: &mut spellinfo_T) -> *mut wordnode_T {
    spin.si_arena.alloc::<wordnode_T>()
}

/// Reject words a spell file cannot represent: invalid UTF-8, control
/// characters, and a trailing `/` (which would read as a flag separator).
///
/// # Safety
///
/// `word` and `end` must delimit one readable range.
pub unsafe fn valid_spell_word(word: *const c_char, end: *const c_char) -> bool {
    // SAFETY: the caller promises the range; the walk stops at `end`.
    unsafe {
        if !utf_valid_string(word, end) {
            return false;
        }
        let mut p = word;
        while *p as c_int != NUL && p < end {
            if (*p as uint8_t as c_int) < b' ' as c_int
                || (*p as c_int == b'/' as c_int && *p.add(1) as c_int == NUL)
            {
                return false;
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }
        true
    }
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
pub unsafe fn store_word(
    spin: &mut spellinfo_T,
    word: *mut c_char,
    flags: c_int,
    region: c_int,
    pfxlist: *const c_char,
    need_affix: bool,
) -> c_int {
    // SAFETY: the caller promises terminated strings; `foldword` is a
    // MAXWLEN buffer, which is the bound spell_casefold is given.
    unsafe {
        let len = strlen(word) as c_int;
        let ct = captype(word, word.offset(len as isize));
        let mut foldword: [c_char; MAXWLEN] = [0; MAXWLEN];
        let mut res = OK;

        if !valid_spell_word(word, word.offset(len as isize)) {
            return FAIL;
        }

        spell_casefold(
            curwin.get(),
            word,
            len,
            foldword.as_mut_ptr(),
            MAXWLEN as c_int,
        );

        let mut p = pfxlist;
        while res == OK {
            if !need_affix || (!p.is_null() && *p as c_int != NUL) {
                let affix_id = if p.is_null() { 0 } else { *p as c_int };
                res = tree_add_word(
                    spin,
                    foldword.as_ptr(),
                    spin.si_foldroot,
                    ct | flags,
                    region,
                    affix_id,
                );
            }
            if p.is_null() || *p as c_int == NUL {
                break;
            }
            p = p.add(1);
        }
        spin.si_foldwcount += 1;

        if res == OK && (ct == WF_KEEPCAP as c_int || flags & WF_KEEPCAP as c_int != 0) {
            let mut p = pfxlist;
            while res == OK {
                if !need_affix || (!p.is_null() && *p as c_int != NUL) {
                    let affix_id = if p.is_null() { 0 } else { *p as c_int };
                    res = tree_add_word(spin, word, spin.si_keeproot, flags, region, affix_id);
                }
                if p.is_null() || *p as c_int == NUL {
                    break;
                }
                p = p.add(1);
            }
            spin.si_keepwcount += 1;
        }
        res
    }
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
pub unsafe fn tree_add_word(
    spin: &mut spellinfo_T,
    word: *const c_char,
    root: *mut wordnode_T,
    flags: c_int,
    region: c_int,
    affixID: c_int,
) -> c_int {
    // SAFETY: nodes come from the arena and outlive the call; the walk
    // follows `word` only up to its NUL.
    unsafe {
        let mut node = root;
        // Where to write back a replaced node: the parent's child or
        // sibling slot, or null at the root.
        let mut prev: *mut *mut wordnode_T = ptr::null_mut();
        let mut i: isize = 0;

        loop {
            // A shared chain has to be copied before it can be changed.
            if !node.is_null() && (*node).wn_refs > 1 {
                (*node).wn_refs -= 1;
                let mut copyprev = prev;
                let mut copyp = node;
                while !copyp.is_null() {
                    let np = get_wordnode(spin);
                    if np.is_null() {
                        return FAIL;
                    }
                    (*np).wn_child = (*copyp).wn_child;
                    if !(*np).wn_child.is_null() {
                        (*(*np).wn_child).wn_refs += 1;
                    }
                    (*np).wn_byte = (*copyp).wn_byte;
                    if (*np).wn_byte as c_int == NUL {
                        (*np).wn_flags = (*copyp).wn_flags;
                        (*np).wn_region = (*copyp).wn_region;
                        (*np).wn_affixID = (*copyp).wn_affixID;
                    }
                    (*np).wn_refs = 1;
                    if !copyprev.is_null() {
                        *copyprev = np;
                    }
                    copyprev = &raw mut (*np).wn_sibling;
                    if copyp == node {
                        node = np;
                    }
                    copyp = (*copyp).wn_sibling;
                }
            }

            // Skip siblings that sort before what is being added.
            while !node.is_null()
                && sorts_before(spin, node, *word.offset(i), flags, region, affixID)
            {
                prev = &raw mut (*node).wn_sibling;
                node = *prev;
            }

            // Insert a node when there is no matching one here. Word ends
            // never merge in the prefix or sound-fold trees, and only merge
            // elsewhere when the flags and affix id match too.
            let need_new = if node.is_null() {
                true
            } else if (*node).wn_byte as c_int != *word.offset(i) as uint8_t as c_int {
                true
            } else {
                *word.offset(i) as c_int == NUL
                    && (flags < 0
                        || spin.si_sugtree != 0
                        || (*node).wn_flags as c_int != flags & WN_MASK
                        || (*node).wn_affixID as c_int != affixID)
            };
            if need_new && !insert_before(spin, &mut node, prev, *word.offset(i)) {
                return FAIL;
            }

            if *word.offset(i) as c_int == NUL {
                (*node).wn_flags = flags as uint16_t;
                (*node).wn_region |= region as int16_t;
                (*node).wn_affixID = affixID as uint8_t;
                break;
            }
            prev = &raw mut (*node).wn_child;
            node = *prev;
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
                msg_start();
                msg_puts(gettext(MSG_COMPRESSING.as_ptr()));
                msg_clr_eos();
                msg_didout.set(false);
                msg_col.set(0);
                ui_flush();
            }
            wordtree_compress(spin, spin.si_foldroot, c"case-folded");
            if affixID >= 0 {
                wordtree_compress(spin, spin.si_keeproot, c"keep-case");
            }
        }
        OK
    }
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
    unsafe {
        if ((*node).wn_byte as c_int) < byte as uint8_t as c_int {
            return true;
        }
        if (*node).wn_byte as c_int != NUL {
            return false;
        }
        // Word ends sort among themselves: by affix id in the prefix tree,
        // otherwise by flags and then by region or affix id.
        if flags < 0 {
            return ((*node).wn_affixID as c_uint) < affixID as c_uint;
        }
        if ((*node).wn_flags as c_uint) < (flags & WN_MASK) as c_uint {
            return true;
        }
        if (*node).wn_flags as c_int != flags & WN_MASK {
            return false;
        }
        if spin.si_sugtree != 0 {
            ((*node).wn_region as c_int & 0xffff) < region
        } else {
            ((*node).wn_affixID as c_uint) < affixID as c_uint
        }
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
    unsafe {
        (*np).wn_byte = byte as uint8_t;
        // The new node inherits the chain's reference count; the old head
        // is now reached only through it.
        if node.is_null() {
            (*np).wn_refs = 1;
        } else {
            (*np).wn_refs = (**node).wn_refs;
            (**node).wn_refs = 1;
        }
        if !prev.is_null() {
            *prev = np;
        }
        (*np).wn_sibling = *node;
    }
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
    unsafe {
        spin.si_first_free = (*n).wn_child;
        *n = mem::zeroed();
    }
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
    unsafe {
        let mut cnt = 0;
        (*node).wn_refs -= 1;
        if (*node).wn_refs == 0 {
            let mut np = node;
            while !np.is_null() {
                if !(*np).wn_child.is_null() {
                    cnt += deref_wordnode(spin, (*np).wn_child);
                }
                free_wordnode(spin, np);
                cnt += 1;
                np = (*np).wn_sibling;
            }
            cnt += 1;
        }
        cnt
    }
}

/// Put one node on the free list, chained through its child slot.
///
/// # Safety
///
/// `n` must be a node no longer reachable from any tree.
unsafe fn free_wordnode(spin: &mut spellinfo_T, n: *mut wordnode_T) {
    // SAFETY: the caller promises the node is unreachable.
    unsafe {
        (*n).wn_child = spin.si_first_free;
    }
    spin.si_first_free = n;
    spin.si_free_count += 1;
}

/// Compress a whole tree and report how much it shrank.
///
/// # Safety
///
/// `root` must be the root node of one of this arena's trees.
pub unsafe fn wordtree_compress(
    spin: &mut spellinfo_T,
    root: *mut wordnode_T,
    name: &core::ffi::CStr,
) {
    // SAFETY: the caller promises a live root.
    unsafe {
        // The root's own chain is what holds the tree; an empty one has
        // nothing to share.
        if (*root).wn_sibling.is_null() {
            return;
        }

        let mut ht: hashtab_T = mem::zeroed();
        hash_init(&raw mut ht);
        let mut tot: c_int = 0;
        let n = node_compress(spin, (*root).wn_sibling, &raw mut ht, &mut tot);

        if spin.si_verbose != 0 || p_verbose.get() > 2 {
            // Scale down first on big trees so the product cannot overflow.
            let perc: core::ffi::c_long = if tot > 1000000 {
                ((tot - n) / (tot / 100)) as core::ffi::c_long
            } else if tot == 0 {
                0
            } else {
                ((tot - n) * 100 / tot) as core::ffi::c_long
            };
            vim_snprintf(
                IObuff.ptr().cast::<c_char>(),
                IOSIZE as usize,
                gettext(c"Compressed %s: %d of %d nodes; %d (%ld%%) remaining".as_ptr()),
                name.as_ptr(),
                n,
                tot,
                tot - n,
                perc,
            );
            spell_message(spin, IObuff.ptr().cast::<c_char>());
        }
        hash_clear(&raw mut ht);
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
    unsafe {
        let mut compressed = 0;
        let mut len = 0;

        let mut np = node;
        while !np.is_null() && !got_int.get() {
            len += 1;
            let child = (*np).wn_child;
            if !child.is_null() {
                // Depth first: the child's digest is only meaningful once
                // everything below it has been compressed.
                compressed += node_compress(spin, child, ht, tot);

                // The key is the node's own digest, so the table's key
                // pointer doubles as the node pointer. `wn_u1` sits at
                // offset zero, which the assertion above pins down.
                let key = child.cast::<c_char>();
                let hash = hash_hash(key);
                let hi = hash_lookup(ht, key, strlen(key), hash);
                if (*hi).hi_key.is_null()
                    || (*hi).hi_key == (&raw const hash_removed).cast_mut().cast()
                {
                    hash_add_item(ht, hi, key, hash);
                    np = (*np).wn_sibling;
                    continue;
                }

                // Same digest: walk the chain of nodes sharing it looking
                // for a genuinely equal sub-tree to point at instead.
                let mut tp = (*hi).hi_key.cast::<wordnode_T>();
                while !tp.is_null() {
                    if node_equal(child, tp) {
                        (*tp).wn_refs += 1;
                        compressed += deref_wordnode(spin, child);
                        (*np).wn_child = tp;
                        break;
                    }
                    tp = (*tp).wn_u2.next;
                }
                if tp.is_null() {
                    // No match; join the chain for the next comer.
                    let head = (*hi).hi_key.cast::<wordnode_T>();
                    (*child).wn_u2.next = (*head).wn_u2.next;
                    (*head).wn_u2.next = child;
                }
            }
            np = (*np).wn_sibling;
        }
        *tot += len + 1;

        // Digest this chain: its length, then a rolling hash over every
        // sibling's byte and either its child pointer or, at a word end,
        // its flags. Each byte is forced non-zero so the five together form
        // a string the hash table can key on.
        (*node).wn_u1.hashkey[0] = len as uint8_t;
        let mut nr: c_uint = 0;
        let mut np = node;
        while !np.is_null() {
            let n: c_uint = if (*np).wn_byte as c_int == NUL {
                ((*np).wn_flags as c_int
                    + (((*np).wn_region as c_int) << 8)
                    + (((*np).wn_affixID as c_int) << 16)) as c_uint
            } else {
                ((*np).wn_byte as usize)
                    .wrapping_add((*np).wn_child.expose_provenance().wrapping_shl(8))
                    as c_uint
            };
            nr = nr.wrapping_mul(101).wrapping_add(n);
            np = (*np).wn_sibling;
        }
        for (i, shift) in [0, 8, 16, 24].into_iter().enumerate() {
            let b = (nr >> shift & 0xff) as uint8_t;
            (*node).wn_u1.hashkey[i + 1] = if b == 0 { 1 } else { b };
        }
        (*node).wn_u1.hashkey[5] = NUL as uint8_t;

        veryfast_breakcheck();
        compressed
    }
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
    unsafe {
        let mut p1 = n1;
        let mut p2 = n2;
        while !p1.is_null() && !p2.is_null() {
            if (*p1).wn_byte != (*p2).wn_byte {
                break;
            }
            let differs = if (*p1).wn_byte as c_int == NUL {
                (*p1).wn_flags != (*p2).wn_flags
                    || (*p1).wn_region != (*p2).wn_region
                    || (*p1).wn_affixID != (*p2).wn_affixID
            } else {
                (*p1).wn_child != (*p2).wn_child
            };
            if differs {
                break;
            }
            p1 = (*p1).wn_sibling;
            p2 = (*p2).wn_sibling;
        }
        p1.is_null() && p2.is_null()
    }
}
