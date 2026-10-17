//! `:syntax keyword` and the keyword hash tables.
//!
//! Keywords are not patterns: they live in one of two hash tables (case
//! sensitive and not) keyed by the keyword text, so a lookup is a hash rather
//! than a scan of every item. [`add_keyword`] fills them, [`syn_clear_keyword`]
//! and [`clear_keywtab`] empty them.

#![forbid(unsafe_code)]

use crate::charset::display::foldcase;
use crate::cstr;
use crate::hashtab::{SlotEntry, hash_bytes};
use crate::mbyte::c_cluster_len;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::c_int;

use super::*;
use crate::types::NUL;

/// One `:syntax keyword` definition of one keyword text.
///
/// A text can have several, with different groups or options; they hang off
/// one [`KeyChain`] in the table, newest first.
pub(crate) struct KeyEntry {
    /// The group, and the `:syntax include` level it was defined at.
    pub(crate) syn: sp_syn,
    /// `containedin=`: this entry's own copy.
    pub(crate) cont_in_list: IdList,
    /// `nextgroup=`: this entry's own copy.
    pub(crate) next_list: IdList,
    pub(crate) flags: SynFlags,
    /// `cchar=`, or NUL.
    pub(crate) conceal_char: c_int,
}

/// Every entry for one keyword text.
///
/// Upstream chains the entries through `ke_next` from the hash slot, adding
/// each new one at the head; `entries` is that chain oldest first, so a walk
/// from the head is `entries.iter().rev()`.
struct KeyChain {
    keyword: Box<[u8]>,
    entries: Vec<KeyEntry>,
}

/// What a slot of a keyword table holds: the index of its [`KeyChain`].
#[derive(Copy, Clone, PartialEq, Eq)]
struct ChainId(u32);

impl SlotEntry for ChainId {
    const EMPTY: Self = ChainId(u32::MAX);

    fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    fn is_removed(self) -> bool {
        self == Self::removed()
    }

    fn removed() -> Self {
        ChainId(u32::MAX - 1)
    }
}

/// One of a syntax block's two keyword tables (case sensitive, and folded).
///
/// The slots are a [`HashTab`] because its slot order is what `:syntax list`
/// shows: the hash, probe sequence and resize points are upstream's, so each
/// keyword lands in the slot it did there. What the slots hold is an index
/// into `chains`, which owns the text and the entries; a chain that empties
/// goes on the free list and its slot becomes a tombstone, as upstream's
/// `hash_remove` left one.
pub(crate) struct KeywordTable {
    slots: HashTab<ChainId>,
    chains: Vec<KeyChain>,
    /// Indexes into `chains` whose chain was removed, for reuse.
    free: Vec<u32>,
}

impl KeywordTable {
    /// An empty table, which allocates nothing until the first keyword.
    pub(crate) const fn new() -> Self {
        KeywordTable {
            slots: HashTab::new(),
            chains: Vec::new(),
            free: Vec::new(),
        }
    }

    /// Whether the table holds no keyword at all.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.slots.ht_used == 0
    }

    /// Drop every keyword, releasing the storage.
    pub(crate) fn clear(&mut self) {
        *self = KeywordTable::new();
    }

    /// The slot `keyword` occupies, or the one it belongs in.
    fn probe(&self, keyword: &[u8], hash: usize) -> crate::hashtab::Slot<ChainId> {
        self.slots
            .probe(hash, |id| &*self.chains[id.0 as usize].keyword == keyword)
    }

    /// The entries for `keyword`, newest first; none when it is not in the
    /// table.
    #[inline]
    pub(crate) fn lookup(&self, keyword: &[u8]) -> impl Iterator<Item = &KeyEntry> {
        let entries: &[KeyEntry] = if self.is_empty() {
            &[]
        } else {
            let hi = self.probe(keyword, hash_bytes(keyword));
            if hi.is_kept() {
                &self.chains[hi.hi_key.0 as usize].entries
            } else {
                &[]
            }
        };
        entries.iter().rev()
    }

    /// Add `entry` for `keyword`, ahead of any it already has.
    fn add(&mut self, keyword: &[u8], entry: KeyEntry) {
        if self.slots.size() == 0 {
            self.slots = HashTab::init();
        }
        let hash = hash_bytes(keyword);
        let hi = self.probe(keyword, hash);
        if hi.is_kept() {
            self.chains[hi.hi_key.0 as usize].entries.push(entry);
            return;
        }
        let chain = KeyChain {
            keyword: keyword.into(),
            entries: vec![entry],
        };
        let id = match self.free.pop() {
            Some(id) => {
                self.chains[id as usize] = chain;
                id
            }
            None => {
                self.chains.push(chain);
                u32::try_from(self.chains.len() - 1).expect("fewer keywords than u32::MAX")
            }
        };
        self.slots.insert_at(hi, ChainId(id), hash);
    }

    /// Drop every keyword entry of group `id`.
    ///
    /// The table is locked for the walk, as upstream's was: a removal may
    /// not resize it under the slot index, and the resize the removals call
    /// for happens once, at the unlock.
    pub(crate) fn remove_group(&mut self, id: c_int) {
        self.slots.lock();
        for idx in 0..self.slots.size() {
            let hi = self.slots.slot(idx);
            if !hi.is_kept() {
                continue;
            }
            let chain_id = hi.hi_key.0;
            let chain = &mut self.chains[chain_id as usize];
            chain
                .entries
                .retain(|entry| c_int::from(entry.syn.id) != id);
            if chain.entries.is_empty() {
                *chain = KeyChain {
                    keyword: Box::default(),
                    entries: Vec::new(),
                };
                self.free.push(chain_id);
                self.slots.remove_at(hi);
            }
        }
        self.slots.unlock();
    }

    /// Every keyword and entry, in slot order and newest first within a
    /// keyword: the order `:syntax list` prints them in.
    pub(crate) fn entries(&self) -> impl Iterator<Item = (&[u8], &KeyEntry)> {
        self.slots.items().flat_map(|hi| {
            let chain = &self.chains[hi.hi_key.0 as usize];
            chain
                .entries
                .iter()
                .rev()
                .map(|entry| (&*chain.keyword, entry))
        })
    }
}

impl Default for KeywordTable {
    fn default() -> Self {
        KeywordTable::new()
    }
}

/// Everything a run of `:syntax keyword` gives every keyword it defines.
struct KeywordDef<'a> {
    /// The syntax group the keywords belong to.
    id: c_int,
    /// `HL_*` flags from the options.
    flags: SynFlags,
    /// `containedin=`, copied into each entry.
    cont_in_list: &'a IdList,
    /// `nextgroup=`, copied into each entry.
    next_list: &'a IdList,
    /// `cchar=`, or NUL.
    conceal_char: c_int,
}

/// Add the keyword `name` to the table its case sensitivity selects.
fn add_keyword(name: &[u8], def: &KeywordDef) {
    let mut block = cur_syn_block();
    // With `:syntax case ignore` the table is keyed on the folded form,
    // and the lookup folds too.
    let ignore_case = block.b_syn_ic != 0;
    let folded;
    let key = if ignore_case {
        folded = foldcase(name, Some(MAXKEYWLEN as usize + 1));
        &folded[..]
    } else {
        name
    };
    let entry = KeyEntry {
        syn: sp_syn {
            inc_tag: current_syn_inc_tag.get(),
            id: def.id as int16_t,
        },
        // Each entry gets its own copy, as upstream's did: `:syntax list`
        // tells two keywords' lists apart by identity.
        cont_in_list: def.cont_in_list.copy(),
        next_list: def.next_list.copy(),
        flags: def.flags,
        conceal_char: def.conceal_char,
    };
    if !def.cont_in_list.is_none() {
        block.b_syn_containedin = 1;
    }
    if ignore_case {
        block.b_keywtab_ic.add(key, entry);
    } else {
        block.b_keywtab.add(key, entry);
    }
}

/// Add the keyword at `kw` in `buf` and every variant its `[optional tail]`
/// names, answering where the next keyword of `buf` starts.
///
/// `buf` holds the keywords one after another, each NUL-terminated.
/// `ab[cde]` defines `ab`, `abc`, `abcd` and `abcde`. The variants are made
/// by editing `buf` in place: the `[` becomes the NUL of the short form, then
/// each following character is shifted left over it. `None` means the
/// notation was malformed and the message has been given.
fn add_keyword_variants(buf: &mut [u8], mut kw: usize, def: &KeywordDef) -> Option<usize> {
    let mut kwlen;
    let mut bracket = cstr::until_nul(&buf[kw..])
        .iter()
        .position(|&b| b == b'[')
        .map(|at| kw + at);
    loop {
        match bracket {
            None => kwlen = cstr::until_nul(&buf[kw..]).len(),
            Some(p) => {
                buf[p] = NUL as u8;
                kwlen = p - kw;
            }
        }
        add_keyword(&buf[kw..kw + kwlen], def);
        let Some(p) = bracket else {
            break;
        };
        let next = c_int::from(cstr::byte_at(buf, p + 1));
        if next == NUL {
            let kw = msg_bytes(cstr::until_nul(&buf[kw..]));
            semsg!("E789: Missing ']': {kw}");
            return None;
        }
        if next == ']' as c_int {
            if c_int::from(cstr::byte_at(buf, p + 2)) != NUL {
                let (kw, arg1) = (
                    msg_bytes(cstr::until_nul(&buf[kw..])),
                    msg_bytes(cstr::until_nul(&buf[p + 2..])),
                );
                semsg!("E890: Trailing char after ']': {kw}]{arg1}");
                return None;
            }
            // Step over the `]`: it and the NUL after it are exactly the
            // two bytes between here and the next keyword.
            kw = p + 1;
            kwlen = 1;
            break;
        }
        // Shift the next character left over the NUL, lengthening the
        // keyword by one, and look again from there.
        let len = c_cluster_len(&buf[p + 1..]);
        buf.copy_within(p + 1..p + 1 + len, p);
        bracket = Some(p + len);
    }
    Some(kw + kwlen + 1)
}

/// `:syntax keyword {group} [{options}] {keyword} ..`.
pub(crate) fn syn_cmd_keyword(args: &mut ExArg, _syncing: c_int) {
    let line = args.line.arg().to_vec();
    let mut conceal_char: c_int = NUL;

    let name = split_group_name(&line);
    let mut end = name.as_ref().map(|name| name.rest);
    if let Some(name) = &name {
        let syn_id = if args.skip {
            -1
        } else {
            syn_check_group(&line[..name.len])
        };
        if syn_id != 0 {
            // A buffer for the keywords with their backslashes removed, each
            // NUL-terminated; it can only shrink, so the argument's length
            // is enough. Pass 2 shifts bytes around inside it.
            let mut buf = vec![0u8; line.len() - name.rest + 1];
            let mut opt = SynOptArg {
                flags: SynFlags::NONE,
                keyword: true,
                takes_sync_idx: false,
                sync_idx: 0,
                has_cont_list: false,
                cont_list: IdList::NONE,
                cont_in_list: IdList::NONE,
                next_list: IdList::NONE,
            };

            // The options apply to ALL the keywords, so every option has
            // to be read before any keyword can be created. Pass 1
            // collects them and copies the keywords into the buffer.
            let mut cnt = 0;
            let mut out = 0;
            let byte = |at: usize| c_int::from(cstr::byte_at(&line, at));
            while let Some(mut at) = end.filter(|&at| ends_excmd(byte(at)) == 0) {
                end = read_item_options(&line, at, &mut opt, &mut conceal_char, args.skip);
                match end {
                    Some(next) if ends_excmd(byte(next)) == 0 => at = next,
                    _ => break,
                }
                while byte(at) != NUL && !ascii_iswhite(byte(at)) {
                    // A backslash hides the byte after it, itself included.
                    if byte(at) == '\\' as c_int && byte(at + 1) != NUL {
                        at += 1;
                    }
                    buf[out] = line[at];
                    out += 1;
                    at += 1;
                }
                buf[out] = NUL as u8;
                out += 1;
                cnt += 1;
                end = Some(at + skip::white(&line[at..]));
            }

            // Pass 2: an entry per keyword.
            if !args.skip {
                syn_incl_toplevel(syn_id, &mut opt.flags);
                let def = KeywordDef {
                    id: syn_id,
                    flags: opt.flags,
                    cont_in_list: &opt.cont_in_list,
                    next_list: &opt.next_list,
                    conceal_char,
                };
                let mut kw = 0;
                while cnt > 0 {
                    cnt -= 1;
                    match add_keyword_variants(&mut buf, kw, &def) {
                        Some(next) => kw = next,
                        None => break,
                    }
                }
            }

            // `buf` and `opt`'s lists go out of scope here; every entry has
            // its own copy of the lists.
            drop(buf);
        }
    }

    match end {
        None => {
            let shown = msg_bytes(&line);
            semsg!("E475: Invalid argument: {shown}");
        }
        // SAFETY: an offset within the command line the caller still owns.
        Some(at) => args.line.next = args.line.check_next(args.line.arg + at),
    }

    redraw_curbuf_later(UPD_SOME_VALID);
    syn_stack_free_all(cur_syn_block()); // Need to recompute all syntax.
}
