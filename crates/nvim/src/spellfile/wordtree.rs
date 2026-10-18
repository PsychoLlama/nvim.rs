//! The word tree `:mkspell` builds, and the byte accounting that decides
//! when it is compressed.
//!
//! Every word of a dictionary is added to a trie: one node per byte, with
//! siblings chained through [`WordNode::wn_sibling`] in ascending byte
//! order and the continuation hanging off [`WordNode::wn_child`]. A node
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
//! The lookup is keyed on a five-byte digest of the chain
//! ([`WordNode::wn_digest`]); chains with the same digest are linked
//! through [`WordNode::wn_link`] and [`node_equal`] decides.
//!
//! Because a compressed sub-tree is shared, [`tree_add_word`] has to
//! un-share any node it is about to modify: a node with more than one
//! reference is copied first. Compression therefore runs *during* the
//! build as well as at the end, whenever the accounting says the build has
//! grown past the threshold `'mkspellmem'` sets.
//!
//! # The slab
//!
//! The nodes are a `Vec<WordNode>` in [`SpellInfo::si_nodes`], linked by
//! [`NodeId`] — an index, which is a node's identity: [`node_equal`]
//! compares children by id, so two child chains are only equal once
//! compression has made them the same node. After the first compression a
//! node can be reachable from several parents, which [`WordNode::wn_refs`]
//! counts; [`deref_wordnode`] puts a chain on the free list
//! (`si_first_free`, chained through `wn_child`, last in first out) when
//! its last reference goes, and [`get_wordnode`] takes from it before it
//! grows the slab. That is upstream's allocation order node for node, so a
//! node gets the same id here as its address had rank there.
//!
//! # The run of word ends
//!
//! A word end goes among its siblings in sorted order, so filing one walks
//! the run of word ends before it. The `.sug` tree files words in
//! ascending number, and words that fold alike share one run, so that walk
//! made the `.sug` pass quadratic -- and so did every longer word whose
//! byte sorts after such a run. `si_end_hint` remembers, by each run's
//! head, the word end last filed in it: the next walk of a run with that
//! head starts there, when that node still sorts before what is filed.
//! Runs only ever grow between compression runs (a copy of a shared run
//! has a new head), and compression clears the hint, so the walk ends where
//! it would have from the head.
//!
//! # What decides when compression runs
//!
//! Upstream allocated the nodes, the affix tables and their strings from
//! one arena of [`SBLOCKSIZE`]-byte blocks and compressed when it had taken
//! enough blocks. Nothing lives in an arena any more, but the *count* is
//! the contract: `'mkspellmem'` is expressed in it, and the node counts
//! `:mkspell` reports at each compression run follow from where the runs
//! fall. So [`SpellArena`] still charges every allocation the arena used
//! to make — same size, same alignment, same order — and keeps the block
//! count. The sizes are the records' sizes when they lived in the arena,
//! kept as named constants ([`WORD_NODE_BYTES`] and the ones beside it)
//! because the records themselves no longer have them.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::spell::WordFlags;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int, c_uint};
use core::hash::{BuildHasherDefault, Hasher};
use core::num::NonZeroU32;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mbyte::{c_cluster_len, valid_utf8};
use crate::message::state::{msg_col, msg_didout};
use crate::message::{msg_clr_eos, msg_start, msg_str};
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext;
use crate::os::input::veryfast_breakcheck;
use crate::spell::{captype, spell_casefold};
use crate::types::{Failed, NUL};
use crate::ui::ui_flush;

use super::{MAXWLEN, SpellInfo, spell_message_fmt};

/// Bytes per accounting block.
const SBLOCKSIZE: usize = 16000;

/// What one word node cost the arena.
pub(super) const WORD_NODE_BYTES: usize = 48;
/// What one `PFX`/`SFX` header cost the arena.
pub(super) const AFF_HEADER_BYTES: usize = 48;
/// What one `PFX`/`SFX` entry cost the arena.
pub(super) const AFF_ENTRY_BYTES: usize = 56;
/// What one `.aff` file's record cost the arena.
pub(super) const AFF_FILE_BYTES: usize = 928;
/// What one compound-flag item cost the arena.
pub(super) const COMP_ITEM_BYTES: usize = 28;
/// The alignment a record charge rounds the block's cursor up to.
const RECORD_ALIGN: usize = 8;

/// The part of a word node's flags that is stored in the tree; the caller
/// passes extra bits above it that only steer where the word is filed.
pub(super) const WN_MASK: c_int = 0xffff;

pub(super) const MSG_COMPRESSING: &CStr = c"Compressing word tree...";

/// The byte count of everything a spell file under construction would
/// have allocated from upstream's arena; see the module doc.
pub(crate) struct SpellArena {
    /// Whether a block has been taken since the last [`clear`](Self::clear).
    started: bool,
    /// Bytes charged to the newest block.
    used: usize,
    /// Blocks taken since the compression heuristic last reset the count.
    /// It is not the number of blocks: [`tree_add_word`] subtracts from it
    /// after each compression run, and the `.sug` pass zeroes it per word.
    pub(crate) block_count: c_int,
}

impl SpellArena {
    pub(crate) const fn new() -> Self {
        Self {
            started: false,
            used: 0,
            block_count: 0,
        }
    }

    /// Charge `len` bytes, aligned for a record when `align`: a new block
    /// when the newest cannot hold them.
    pub(crate) fn charge(&mut self, len: usize, align: bool) {
        debug_assert!(len <= SBLOCKSIZE);
        if align && self.started {
            self.used = self.used.next_multiple_of(RECORD_ALIGN);
        }
        if !self.started || self.used + len > SBLOCKSIZE {
            self.started = true;
            self.used = 0;
            self.block_count += 1;
        }
        self.used += len;
    }

    /// Charge a copy of the string `s` with its terminator.
    pub(crate) fn charge_str(&mut self, s: &[u8]) {
        self.charge(s.len() + 1, false);
    }

    /// Start over with no blocks.
    pub(crate) fn clear(&mut self) {
        *self = Self::new();
    }
}

/// A node of the slab: its index. Slot 0 is a placeholder no link names,
/// so that `Option<NodeId>` is the null link at no cost.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct NodeId(NonZeroU32);

impl NodeId {
    fn index(self) -> usize {
        self.0.get() as usize
    }

    fn of_index(idx: usize) -> Self {
        let n = u32::try_from(idx).expect("fewer word nodes than u32::MAX");
        NodeId(NonZeroU32::new(n).expect("slot 0 is never handed out"))
    }

    /// The id as the digest reads it.
    fn raw(self) -> u32 {
        self.0.get()
    }
}

/// One byte of one word, or — when [`wn_byte`](Self::wn_byte) is NUL — the
/// end of a word and the properties that go with it.
#[derive(Clone, Default)]
pub(crate) struct WordNode {
    /// Compression phase: this node's sibling chain's length and four hash
    /// bytes, none of them zero (upstream keyed a string on them). See
    /// [`write_digest`].
    pub(crate) wn_digest: [u8; 5],
    /// Write phase: this node's index in the written tree, or 0 while it
    /// has not been placed. See [`put_node`](super::write::put_node).
    pub(crate) wn_index: c_int,
    /// Whichever node currently has a claim on this one — the two phases
    /// mean different things by that, and never overlap:
    ///
    /// - while compressing, the next node whose sibling chain has the same
    ///   [`wn_digest`](Self::wn_digest), so that [`node_compress`] can walk
    ///   a bucket looking for a chain worth sharing;
    /// - while writing, the sibling chain that claimed this node and will
    ///   therefore emit it; every other parent emits a
    ///   [`BY_INDEX`](super::BY_INDEX) reference instead.
    ///
    /// [`clear_node`](super::write::clear_node) is the handover. A link a
    /// compression run left is *not* cleared by the next run, as upstream
    /// did not clear it.
    pub(crate) wn_link: Option<NodeId>,
    pub(crate) wn_child: Option<NodeId>,
    pub(crate) wn_sibling: Option<NodeId>,
    /// How many parents point here; above one the sub-tree is shared.
    pub(crate) wn_refs: c_int,
    pub(crate) wn_byte: u8,
    pub(crate) wn_affix_id: u8,
    pub(crate) wn_flags: u16,
    pub(crate) wn_region: i16,
}

impl SpellInfo {
    pub(super) fn node(&self, id: NodeId) -> &WordNode {
        &self.si_nodes[id.index()]
    }

    pub(super) fn node_mut(&mut self, id: NodeId) -> &mut WordNode {
        &mut self.si_nodes[id.index()]
    }

    /// Drop every tree and the byte count with them.
    pub(super) fn clear_trees(&mut self) {
        self.si_arena.clear();
        self.si_nodes = empty_slab();
        self.si_first_free = None;
        self.si_free_count = 0;
        self.si_end_hint.clear();
    }
}

/// A slab with nothing in it but the placeholder in slot 0.
pub(super) fn empty_slab() -> Vec<WordNode> {
    vec![WordNode::default()]
}

/// Arena blocks in use before the first compression run.
static COMPRESS_START: GlobalCell<c_int> = GlobalCell::new(30_000);
/// Blocks that have to be taken again before compressing again.
static COMPRESS_INC: GlobalCell<c_int> = GlobalCell::new(100);
/// Words to add after a compression run before considering another.
static COMPRESS_ADDED: GlobalCell<c_int> = GlobalCell::new(500_000);

/// Install the `'mkspellmem'` values, already scaled to blocks.
pub(super) fn set_compression_limits(start: c_int, incr: c_int, added: c_int) {
    COMPRESS_START.set(start);
    COMPRESS_INC.set(incr);
    COMPRESS_ADDED.set(added);
}

/// The size of an accounting block, for scaling `'mkspellmem'`.
pub(super) const fn block_size() -> c_int {
    SBLOCKSIZE as c_int
}

/// Allocate a tree's root node. Always a fresh one, never the free list's.
pub(super) fn wordtree_alloc(spin: &mut SpellInfo) -> NodeId {
    fresh_node(spin)
}

/// Reject words a spell file cannot represent: invalid UTF-8, control
/// characters, and a trailing `/` (which would read as a flag separator).
///
/// `word` is read as the C string it was: a NUL in it ends it.
pub(super) fn valid_spell_word(word: &[u8]) -> bool {
    if !valid_utf8(word) {
        return false;
    }
    let mut at = 0;
    while at < word.len() && word[at] != NUL as u8 {
        if word[at] < b' ' || (word[at] == b'/' && cstr::byte_at(word, at + 1) == NUL as u8) {
            return false;
        }
        at += c_cluster_len(&word[at..]);
    }
    true
}

/// File one word in the case-folded tree, and in the keep-case tree when
/// its capitalisation cannot be derived from the folded form.
///
/// `pfxlist` is the set of affix ids the word takes; the word is added once
/// per id, since which prefixes apply is part of what the tree records.
/// When `need_affix` is set the word is only added for real affix ids, not
/// for the bare word. Both are read as the C strings they were.
pub(super) fn store_word(
    spin: &mut SpellInfo,
    word: &[u8],
    flags: WordFlags,
    region: c_int,
    pfxlist: &[u8],
    need_affix: bool,
) -> Result<(), Failed> {
    let text = cstr::until_nul(word);
    let pfxlist = cstr::until_nul(pfxlist);
    let ct = captype(text, text.len());
    if !valid_spell_word(text) {
        return Err(Failed);
    }

    let mut foldword = [0u8; MAXWLEN];
    let _ = spell_casefold(Win::current(), text, text.len(), &mut foldword);

    let root = spin.si_foldroot.expect("mkspell allocates the roots first");
    let with_case = ct | flags;
    let folded = cstr::until_nul(&foldword);
    add_per_affix(spin, folded, root, with_case, region, pfxlist, need_affix);
    spin.si_foldwcount += 1;

    if ct == WordFlags::KEEPCAP || flags.has(WordFlags::KEEPCAP) {
        let root = spin.si_keeproot.expect("mkspell allocates the roots first");
        add_per_affix(spin, text, root, flags, region, pfxlist, need_affix);
        spin.si_keepwcount += 1;
    }
    Ok(())
}

/// Add `word` to `root` once per affix id in `pfxlist`, then once more
/// under affix id 0 — unless `need_affix`, which means only real ids count.
///
/// An id is a `char` the way upstream read it, so one above 127 is
/// negative.
fn add_per_affix(
    spin: &mut SpellInfo,
    word: &[u8],
    root: NodeId,
    flags: WordFlags,
    region: c_int,
    pfxlist: &[u8],
    need_affix: bool,
) {
    for id in pfxlist.iter().copied().chain([0]) {
        let affix_id = c_int::from(id.cast_signed());
        if !need_affix || affix_id != NUL {
            tree_add_word(spin, word, root, flags.bits(), region, affix_id);
        }
    }
}

/// The link a node was reached through, which is where a replacement for
/// it has to be written back.
#[derive(Copy, Clone)]
enum Link {
    /// The tree's root itself, which nothing points at.
    Root,
    Child(NodeId),
    Sibling(NodeId),
}

fn set_link(spin: &mut SpellInfo, link: Link, to: NodeId) {
    match link {
        Link::Root => {}
        Link::Child(n) => spin.node_mut(n).wn_child = Some(to),
        Link::Sibling(n) => spin.node_mut(n).wn_sibling = Some(to),
    }
}

/// Add one word to `root`, and compress the trees when the accounting says
/// the build has grown past the threshold.
///
/// A negative `flags` means the prefix tree, where `affix_id` rather than
/// the flags decides sibling order and no two entries are ever merged.
/// `word` is read as the C string it was, up to a NUL or its end.
pub(super) fn tree_add_word(
    spin: &mut SpellInfo,
    word: &[u8],
    root: NodeId,
    flags: c_int,
    region: c_int,
    affix_id: c_int,
) {
    let word = cstr::until_nul(word);
    let order = EndOrder {
        flags,
        masked: flags & WN_MASK,
        sugtree: spin.si_sugtree != 0,
        region,
        affix_id,
    };
    let mut node = Some(root);
    let mut prev = Link::Root;
    let mut i = 0;

    loop {
        let byte = cstr::byte_at(word, i);
        // A shared chain has to be copied before it can be changed.
        if let Some(n) = node
            && spin.node(n).wn_refs > 1
        {
            spin.node_mut(n).wn_refs -= 1;
            let mut copyprev = prev;
            let mut copyp = Some(n);
            while let Some(cp) = copyp {
                let np = get_wordnode(spin);
                let from = spin.node(cp).clone();
                if let Some(child) = from.wn_child {
                    spin.node_mut(child).wn_refs += 1;
                }
                let new = spin.node_mut(np);
                new.wn_child = from.wn_child;
                new.wn_byte = from.wn_byte;
                if from.wn_byte == NUL as u8 {
                    new.wn_flags = from.wn_flags;
                    new.wn_region = from.wn_region;
                    new.wn_affix_id = from.wn_affix_id;
                }
                new.wn_refs = 1;
                set_link(spin, copyprev, np);
                copyprev = Link::Sibling(np);
                if cp == n {
                    node = Some(np);
                }
                copyp = from.wn_sibling;
            }
        }

        // Skip siblings that sort before what is being added. In the
        // `.sug` tree, start from where the last word end of this run went
        // when that still sorts before; see `si_end_hint`.
        let head = node;
        let hinted = head
            .filter(|_| order.sugtree)
            .and_then(|head| spin.si_end_hint.get(&head).copied())
            .map(|from| skip_siblings(&spin.si_nodes, Some(from), byte, &order))
            .filter(|&(_, last)| last.is_some());
        let last;
        (node, last) = hinted.unwrap_or_else(|| skip_siblings(&spin.si_nodes, node, byte, &order));
        if let Some(last) = last {
            prev = Link::Sibling(last);
        }

        // Insert a node when there is no matching one here. Word ends
        // never merge in the prefix or sound-fold trees, and only merge
        // elsewhere when the flags and affix id match too.
        let need_new = match node {
            None => true,
            Some(n) => {
                let n = spin.node(n);
                n.wn_byte != byte
                    || (byte == NUL as u8
                        && (flags < 0
                            || spin.si_sugtree != 0
                            || c_int::from(n.wn_flags) != flags & WN_MASK
                            || c_int::from(n.wn_affix_id) != affix_id))
            }
        };
        let n = if need_new {
            insert_before(spin, node, prev, byte)
        } else {
            node.expect("a matching node is a node")
        };

        if byte == NUL as u8 {
            if order.sugtree {
                let run_head = match (last, head) {
                    (Some(_), Some(head)) => head,
                    _ => {
                        // A new head: the run is known by it from now on.
                        if let Some(old) = head {
                            spin.si_end_hint.remove(&old);
                        }
                        n
                    }
                };
                spin.si_end_hint.insert(run_head, n);
            }
            let end = spin.node_mut(n);
            end.wn_flags = flags as u16;
            end.wn_region |= region as i16;
            end.wn_affix_id = affix_id as u8;
            break;
        }
        prev = Link::Child(n);
        node = spin.node(n).wn_child;
        i += 1;
    }

    spin.si_msg_count += 1;

    // Count down to the next compression run, then arm the "arena has
    // grown again" test by adding the increment back.
    if spin.si_compress_cnt > 1 {
        spin.si_compress_cnt -= 1;
        if spin.si_compress_cnt == 1 {
            spin.si_arena.block_count += COMPRESS_INC.get();
        }
    }
    let due = if spin.si_compress_cnt == 1 {
        spin.si_free_count < MAXWLEN as c_int
    } else {
        spin.si_arena.block_count >= COMPRESS_START.get()
    };
    if due {
        spin.si_arena.block_count -= COMPRESS_INC.get();
        spin.si_compress_cnt = COMPRESS_ADDED.get();
        if spin.si_verbose != 0 {
            msg_start();
            msg_str(gettext(MSG_COMPRESSING));
            msg_clr_eos();
            msg_didout.set(false);
            msg_col.set(0);
            ui_flush();
        }
        wordtree_compress(spin, spin.si_foldroot, c"case-folded");
        if affix_id >= 0 {
            wordtree_compress(spin, spin.si_keeproot, c"keep-case");
        }
    }
}

/// What decides where a word end goes among its siblings: the entry's
/// flags (negative for the prefix tree), those flags as stored, whether
/// this is the `.sug` tree, and its region and affix id.
struct EndOrder {
    flags: c_int,
    masked: c_int,
    sugtree: bool,
    region: c_int,
    affix_id: c_int,
}

/// Step over the siblings from `node` on that sort before the entry being
/// added, whose byte is `byte`: the first that does not, and the last that
/// did.
///
/// A long run of word ends (the `.sug` tree's, where words that fold
/// alike pile up) makes this the loop `:mkspell` spends its time in, so
/// the three ways word ends are ordered each get a loop of their own.
#[inline]
fn skip_siblings(
    nodes: &[WordNode],
    node: Option<NodeId>,
    byte: u8,
    order: &EndOrder,
) -> (Option<NodeId>, Option<NodeId>) {
    if order.flags < 0 {
        // The prefix tree: by affix id.
        let affix = order.affix_id as c_uint;
        skip_while(nodes, node, byte, |n| c_uint::from(n.wn_affix_id) < affix)
    } else if order.sugtree {
        // By flags, then region: the `.sug` tree's word number.
        let (flags, region) = (order.masked, order.region);
        skip_while(nodes, node, byte, |n| {
            let f = c_int::from(n.wn_flags);
            f < flags || (f == flags && (c_int::from(n.wn_region) & 0xffff) < region)
        })
    } else {
        // By flags, then affix id.
        let (flags, affix) = (order.masked, order.affix_id as c_uint);
        skip_while(nodes, node, byte, |n| {
            let f = c_int::from(n.wn_flags);
            f < flags || (f == flags && c_uint::from(n.wn_affix_id) < affix)
        })
    }
}

/// [`skip_siblings`] with the word ends' order as `end_before`. A node
/// sorts before the entry when its byte is lower, or when both are word
/// ends and `end_before` says so.
#[inline(always)]
fn skip_while(
    nodes: &[WordNode],
    mut node: Option<NodeId>,
    byte: u8,
    end_before: impl Fn(&WordNode) -> bool,
) -> (Option<NodeId>, Option<NodeId>) {
    let mut last = None;
    while let Some(n) = node {
        let sibling = &nodes[n.index()];
        let before =
            sibling.wn_byte < byte || (sibling.wn_byte == NUL as u8 && end_before(sibling));
        if !before {
            break;
        }
        last = node;
        node = sibling.wn_sibling;
    }
    (node, last)
}

/// Splice a fresh node carrying `byte` in ahead of `node`, writing it to
/// the link `node` was reached through, and answer it.
#[inline]
fn insert_before(spin: &mut SpellInfo, node: Option<NodeId>, prev: Link, byte: u8) -> NodeId {
    let np = get_wordnode(spin);
    // The new node inherits the chain's reference count; the old head is
    // now reached only through it.
    let refs = match node {
        None => 1,
        Some(old) => core::mem::replace(&mut spin.node_mut(old).wn_refs, 1),
    };
    let new = spin.node_mut(np);
    new.wn_byte = byte;
    new.wn_refs = refs;
    new.wn_sibling = node;
    set_link(spin, prev, np);
    np
}

/// A node the slab grows by, charged as the arena would have.
fn fresh_node(spin: &mut SpellInfo) -> NodeId {
    spin.si_arena.charge(WORD_NODE_BYTES, true);
    spin.si_nodes.push(WordNode::default());
    NodeId::of_index(spin.si_nodes.len() - 1)
}

/// Take a node from the free list, or a fresh one.
fn get_wordnode(spin: &mut SpellInfo) -> NodeId {
    let Some(n) = spin.si_first_free else {
        return fresh_node(spin);
    };
    spin.si_first_free = spin.node(n).wn_child;
    *spin.node_mut(n) = WordNode::default();
    spin.si_free_count -= 1;
    n
}

/// Drop one reference to `node`; when the last goes, return its whole
/// sibling chain and their sub-trees to the free list.
///
/// Returns how many nodes were freed, for the compression report.
fn deref_wordnode(spin: &mut SpellInfo, node: NodeId) -> c_int {
    let mut cnt = 0;
    spin.node_mut(node).wn_refs -= 1;
    if spin.node(node).wn_refs == 0 {
        let mut np = Some(node);
        while let Some(p) = np {
            if let Some(child) = spin.node(p).wn_child {
                cnt += deref_wordnode(spin, child);
            }
            free_wordnode(spin, p);
            cnt += 1;
            np = spin.node(p).wn_sibling;
        }
        cnt += 1;
    }
    cnt
}

/// Put one node on the free list, chained through its child link. Its
/// other fields stay as they were, as upstream left them.
fn free_wordnode(spin: &mut SpellInfo, n: NodeId) {
    spin.node_mut(n).wn_child = spin.si_first_free;
    spin.si_first_free = Some(n);
    spin.si_free_count += 1;
}

/// Compress a whole tree and report how much it shrank. A tree that is not
/// there (the `.sug` pass has no keep-case tree) is left alone.
pub(super) fn wordtree_compress(spin: &mut SpellInfo, root: Option<NodeId>, name: &CStr) {
    // The root's own chain is what holds the tree; an empty one has
    // nothing to share.
    let Some(tree) = root.and_then(|root| spin.node(root).wn_sibling) else {
        return;
    };
    // Compression frees nodes, and a freed id may come back as part of a
    // different run.
    spin.si_end_hint.clear();

    let mut ht = DigestTable::default();
    let mut tot: c_int = 0;
    let n = node_compress(spin, tree, &mut ht, &mut tot);

    if spin.si_verbose != 0 || p_verbose() > 2 {
        let perc = remaining_percentage(n, tot);
        let (name, left) = (name.to_string_lossy(), tot - n);
        spell_message_fmt(
            spin,
            format_args!("Compressed {name}: {n} of {tot} nodes; {left} ({perc}%) remaining"),
        );
    }
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
/// `ht` maps a digest to the first chain seen with it; the rest with the
/// same digest hang off that one's [`WordNode::wn_link`].
fn node_compress(
    spin: &mut SpellInfo,
    node: NodeId,
    ht: &mut DigestTable,
    tot: &mut c_int,
) -> c_int {
    let mut compressed = 0;
    let mut len = 0;

    let mut next = Some(node);
    while let Some(np) = next {
        next = spin.node(np).wn_sibling;
        if got_int.get() {
            break;
        }
        len += 1;
        let Some(child) = spin.node(np).wn_child else {
            continue;
        };
        // Depth first: the child's digest is only meaningful once
        // everything below it has been compressed.
        compressed += node_compress(spin, child, ht, tot);

        let head = match ht.entry(digest_key(spin.node(child).wn_digest)) {
            Entry::Vacant(slot) => {
                slot.insert(child);
                continue;
            }
            Entry::Occupied(slot) => *slot.get(),
        };

        // Same digest: walk the chain of nodes sharing it looking for a
        // genuinely equal sub-tree to point at instead.
        let mut tp = Some(head);
        while let Some(t) = tp {
            if node_equal(spin, child, t) {
                spin.node_mut(t).wn_refs += 1;
                compressed += deref_wordnode(spin, child);
                spin.node_mut(np).wn_child = Some(t);
                break;
            }
            tp = spin.node(t).wn_link;
        }
        if tp.is_none() {
            // No match; join the chain for the next comer.
            spin.node_mut(child).wn_link = spin.node(head).wn_link;
            spin.node_mut(head).wn_link = Some(child);
        }
    }
    *tot += len + 1;
    write_digest(spin, node, len);

    veryfast_breakcheck();
    compressed
}

/// Digest `node`'s whole sibling chain into [`WordNode::wn_digest`]: the
/// chain's length, then a rolling hash over every sibling's byte and either
/// its child's id or, at a word end, its flags.
///
/// Each of the four hash bytes is forced non-zero, as upstream needed for
/// a string key. `len` is the chain's length as [`node_compress`] counted
/// it, truncated — a longer chain only collides more often, and
/// [`node_equal`] is what decides equality.
fn write_digest(spin: &mut SpellInfo, node: NodeId, len: c_int) {
    let mut nr: c_uint = 0;
    let mut next = Some(node);
    while let Some(np) = next {
        let n = spin.node(np);
        let v: c_uint = if n.wn_byte == NUL as u8 {
            (c_int::from(n.wn_flags)
                + (c_int::from(n.wn_region) << 8)
                + (c_int::from(n.wn_affix_id) << 16)) as c_uint
        } else {
            let child = n.wn_child.map_or(0, NodeId::raw);
            c_uint::from(n.wn_byte).wrapping_add(child.wrapping_shl(8))
        };
        nr = nr.wrapping_mul(101).wrapping_add(v);
        next = n.wn_sibling;
    }

    let mut digest = [len as u8, 0, 0, 0, 0];
    for (i, shift) in [0, 8, 16, 24].into_iter().enumerate() {
        digest[i + 1] = ((nr >> shift & 0xff) as u8).max(1);
    }
    spin.node_mut(node).wn_digest = digest;
}

/// The hasher of the two node tables. A key is a node id or a digest
/// (already four hash bytes and a length), so it only needs spreading over
/// the table's index bits.
#[derive(Default)]
pub(super) struct DigestHasher(u64);

impl Hasher for DigestHasher {
    fn finish(&self) -> u64 {
        self.0.wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }

    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 << 8) ^ u64::from(b);
        }
    }

    fn write_u64(&mut self, v: u64) {
        self.0 = v;
    }

    fn write_u32(&mut self, v: u32) {
        self.0 = u64::from(v);
    }
}

/// Chain heads by digest; see [`node_compress`].
type DigestTable = HashMap<u64, NodeId, BuildHasherDefault<DigestHasher>>;

/// The `.sug` tree's runs of word ends, by head: the word end last filed
/// in each. See the module doc.
pub(super) type EndHints = HashMap<NodeId, NodeId, BuildHasherDefault<DigestHasher>>;

/// A digest as the compression table's key.
fn digest_key(digest: [u8; 5]) -> u64 {
    let [a, b, c, d, e] = digest;
    u64::from_le_bytes([a, b, c, d, e, 0, 0, 0])
}

/// Are two sibling chains interchangeable?
///
/// Children are compared by id, not by content: by the time a chain is
/// looked at, everything below it has already been compressed, so equal
/// sub-trees are the same node.
fn node_equal(spin: &SpellInfo, n1: NodeId, n2: NodeId) -> bool {
    let mut p1 = Some(n1);
    let mut p2 = Some(n2);
    while let (Some(a), Some(b)) = (p1, p2) {
        let (a, b) = (spin.node(a), spin.node(b));
        if a.wn_byte != b.wn_byte {
            break;
        }
        let differs = if a.wn_byte == NUL as u8 {
            a.wn_flags != b.wn_flags || a.wn_region != b.wn_region || a.wn_affix_id != b.wn_affix_id
        } else {
            a.wn_child != b.wn_child
        };
        if differs {
            break;
        }
        p1 = a.wn_sibling;
        p2 = b.wn_sibling;
    }
    p1.is_none() && p2.is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spin() -> SpellInfo {
        let mut spin = SpellInfo::new();
        spin.si_foldroot = Some(wordtree_alloc(&mut spin));
        spin
    }

    /// The words of a tree, in tree order, as `byte...` strings with the
    /// word end's affix id.
    fn words(spin: &SpellInfo, node: Option<NodeId>, prefix: &mut Vec<u8>, out: &mut Vec<String>) {
        let mut next = node;
        while let Some(n) = next {
            let node = spin.node(n);
            if node.wn_byte == 0 {
                out.push(format!(
                    "{}/{}",
                    String::from_utf8_lossy(prefix),
                    node.wn_affix_id
                ));
            } else {
                prefix.push(node.wn_byte);
                words(spin, node.wn_child, prefix, out);
                prefix.pop();
            }
            next = node.wn_sibling;
        }
    }

    fn tree_of(spin: &SpellInfo) -> Vec<String> {
        let root = spin.si_foldroot.expect("allocated");
        let mut out = Vec::new();
        words(spin, spin.node(root).wn_sibling, &mut Vec::new(), &mut out);
        out
    }

    #[test]
    fn the_arena_takes_a_block_when_the_newest_is_full() {
        let mut arena = SpellArena::new();
        arena.charge(10, false);
        assert_eq!(arena.block_count, 1);
        // Aligned: 10 rounds up to 16, and 16 + 15984 fills the block.
        arena.charge(SBLOCKSIZE - 16, true);
        assert_eq!(arena.block_count, 1);
        arena.charge(1, false);
        assert_eq!(arena.block_count, 2);
        arena.charge_str(b"abc");
        assert_eq!(arena.used, 5);
        arena.clear();
        assert_eq!(arena.block_count, 0);
    }

    #[test]
    fn words_are_filed_in_byte_order() {
        let mut spin = spin();
        let root = spin.si_foldroot.unwrap();
        for w in [&b"cab"[..], b"ab", b"abc", b"b"] {
            tree_add_word(&mut spin, w, root, 0, 1, 0);
        }
        assert_eq!(tree_of(&spin), ["ab/0", "abc/0", "b/0", "cab/0"]);
        // One node per byte and word end, plus the root: "cab" takes four,
        // "ab" three, "abc" and "b" two each. Slot 0 is the placeholder,
        // which the arena never paid for.
        let nodes = spin.si_nodes.len() - 1;
        assert_eq!(nodes, 1 + 4 + 3 + 2 + 2);
        assert_eq!(spin.si_arena.used, nodes * WORD_NODE_BYTES);
    }

    /// Compression shares equal tails; adding below a shared node copies
    /// it first and leaves the other word as it was; freed nodes come back
    /// before the slab grows.
    #[test]
    fn a_shared_tail_is_copied_before_it_changes() {
        let mut spin = spin();
        let root = spin.si_foldroot.unwrap();
        tree_add_word(&mut spin, b"xa", root, 0, 1, 0);
        tree_add_word(&mut spin, b"ya", root, 0, 1, 0);
        let before = spin.si_nodes.len();
        wordtree_compress(&mut spin, Some(root), c"t");
        // "a" + end under "y" went to the free list.
        assert_eq!(spin.si_free_count, 2);
        let x = spin.node(root).wn_sibling.unwrap();
        let y = spin.node(x).wn_sibling.unwrap();
        assert_eq!(spin.node(x).wn_child, spin.node(y).wn_child);
        assert_eq!(spin.node(spin.node(x).wn_child.unwrap()).wn_refs, 2);

        tree_add_word(&mut spin, b"yab", root, 0, 1, 0);
        assert_eq!(tree_of(&spin), ["xa/0", "ya/0", "yab/0"]);
        // The two copies came off the free list; "b" and its end are new.
        assert_eq!(spin.si_free_count, 0);
        assert_eq!(spin.si_nodes.len(), before + 2);
    }

    /// The `.sug` tree's word ends, filed in any order, end up sorted by
    /// their number whether the walk starts at the run's head or at the
    /// hint.
    #[test]
    fn word_ends_are_sorted_however_they_arrive() {
        let mut spin = spin();
        spin.si_sugtree = 1;
        let root = spin.si_foldroot.unwrap();
        let order = [5, 6, 7, 1, 8, 2, 9, 3, 10, 4, 11, 12];
        for (i, &nr) in order.iter().enumerate() {
            tree_add_word(&mut spin, b"ab", root, 0, nr, 0);
            if i % 3 == 0 {
                // Another run in between moves the hint away.
                tree_add_word(&mut spin, b"x", root, 0, nr, 0);
            }
        }
        let a = spin.node(root).wn_sibling.unwrap();
        let b = spin.node(a).wn_child.unwrap();
        let mut run = Vec::new();
        let mut next = spin.node(b).wn_child;
        while let Some(n) = next {
            run.push(spin.node(n).wn_region);
            next = spin.node(n).wn_sibling;
        }
        assert_eq!(run, (1..=12).collect::<Vec<i16>>());
    }

    #[test]
    fn a_digest_has_no_zero_hash_byte() {
        let mut spin = spin();
        let root = spin.si_foldroot.unwrap();
        tree_add_word(&mut spin, b"x", root, 0, 1, 0);
        let x = spin.node(root).wn_sibling.unwrap();
        write_digest(&mut spin, x, 1);
        let d = spin.node(x).wn_digest;
        assert_eq!(d[0], 1);
        assert!(d[1..].iter().all(|&b| b != 0));
    }

    #[test]
    fn a_word_with_a_control_character_or_trailing_slash_is_refused() {
        assert!(valid_spell_word(b"word"));
        assert!(!valid_spell_word(b"wo\x01rd"));
        assert!(!valid_spell_word(b"word/"));
        assert!(valid_spell_word(b"wo/rd"));
        assert!(!valid_spell_word(b"\xff"));
    }
}
