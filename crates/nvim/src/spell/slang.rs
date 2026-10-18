//! The lifetime of one loaded language.
//!
//! A [`SpellLang`] is everything a single `.spl` file turned into: three word
//! trees, the affix and compound rules, the sound-folding tables, the REP
//! list, and — once a `.sug` file has been read — a fourth tree of
//! sound-folded forms. They are listed in the global `loaded_langs`, and
//! shared by every window whose `'spelllang'` names them.
//!
//! Only three things happen to one here: it is allocated
//! ([`slang_alloc`]), emptied so the file can be read again
//! ([`slang_clear`]), or freed ([`slang_free`]). The reader in `spellfile`
//! fills it in between.
//!
//! Two per-language tables that nothing else owns also live here: the
//! `COMMON` word counts ([`count_common_word`]), used to prefer suggestions
//! that are ordinary words, and the syllable table
//! ([`init_syl_tab`]/[`count_syllables`]) that `COMPOUNDSYLMAX` is measured
//! against.

#![forbid(unsafe_code)]

use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::strings::find_char;
use crate::types::{Buffer, LangId};
use crate::winlayer::Buf;
use core::cell::RefCell;
use core::ffi::c_int;
use std::collections::HashMap;

use crate::allocator::Owned;
use crate::buffer::alloc_unregistered_buffer;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::mbyte::{c_cluster_len, char_at};
use crate::memline::{ml_close, ml_open, ml_open_file};
use crate::types::{OK, SpellLang, uint8_t, uint16_t};

use super::{MAXWLEN, MAXWORDCOUNT, SP_FORMERROR, SY_MAXLEN, SylItem, WordTree};

/// The hash the `COMMON` table keys words with: upstream's `hash_hash`
/// (`h = h * 101 + byte`), spread over the word by a multiplicative mix so
/// that short words do not all land in the table's low buckets. The table
/// is asked once per word checked while counting, where SipHash cost as
/// much as the rest of the count.
#[derive(Default)]
pub struct WordHasher(u64);

impl core::hash::Hasher for WordHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.wrapping_mul(101).wrapping_add(u64::from(b));
        }
    }

    fn write_usize(&mut self, n: usize) {
        // The length a slice's hash starts with: taken whole, not a byte
        // at a time.
        self.0 = n as u64;
    }

    fn finish(&self) -> u64 {
        self.0.wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }
}

/// `COMMON` word counts, by word.
pub type WordCounts = HashMap<Box<[u8]>, uint16_t, core::hash::BuildHasherDefault<WordHasher>>;

impl SpellLang {
    /// An empty language named `name` (which may be empty), read from
    /// `fname`. The caller fills it from the file and lists it with
    /// [`register_lang`].
    pub(crate) fn new(name: &[u8], fname: XString) -> Box<SpellLang> {
        Box::new(SpellLang {
            sl_name: name.to_vec(),
            sl_fname: fname,
            sl_add: false,
            sl_fold_tree: WordTree::default(),
            sl_keep_tree: WordTree::default(),
            sl_prefix_tree: WordTree::default(),
            sl_info: None,
            sl_regions: [0; 17],
            sl_midword: Vec::new(),
            sl_wordcount: WordCounts::default(),
            sl_compmax: MAXWLEN as c_int,
            sl_compminlen: 0,
            sl_compsylmax: MAXWLEN as c_int,
            sl_compoptions: 0,
            sl_comppat: Vec::new(),
            sl_compprog: RefCell::new(None),
            sl_comprules: None,
            sl_compstartflags: Vec::new(),
            sl_compallflags: Vec::new(),
            sl_nobreak: false,
            sl_syllable: None,
            sl_syl_items: Vec::new(),
            sl_prefixcnt: 0,
            sl_prefprog: Vec::new(),
            sl_rep: Vec::new(),
            sl_rep_first: [0; 256],
            sl_sal: Vec::new(),
            sl_sofo_map: Vec::new(),
            sl_sal_first: [0; 256],
            sl_followup: false,
            sl_collapse: false,
            sl_rem_accents: false,
            sl_sofo: false,
            sl_repsal: Vec::new(),
            sl_repsal_first: [0; 256],
            sl_nosplitsugs: false,
            sl_nocompoundsugs: false,
            sl_sugtime: 0,
            sl_sound_tree: WordTree::default(),
            sl_sugbuf: None,
            sl_sugloaded: false,
            sl_has_map: false,
            sl_map_hash: HashMap::new(),
            sl_map_array: [0; 256],
        })
    }
}

/// Empty a language so its file can be read again, leaving its name and
/// file name intact.
pub fn slang_clear(slang: &mut SpellLang) {
    slang.sl_fold_tree = WordTree::default();
    slang.sl_keep_tree = WordTree::default();
    slang.sl_prefix_tree = WordTree::default();

    slang.sl_rep = Vec::new();
    slang.sl_repsal = Vec::new();

    slang.sl_sal = Vec::new();
    slang.sl_sofo_map = Vec::new();

    slang.sl_prefprog = Vec::new();
    slang.sl_prefixcnt = 0;
    slang.sl_info = None;
    slang.sl_midword = Vec::new();

    *slang.sl_compprog.get_mut() = None;
    slang.sl_comprules = None;
    slang.sl_compstartflags = Vec::new();
    slang.sl_compallflags = Vec::new();

    slang.sl_syllable = None;
    slang.sl_syl_items = Vec::new();

    slang.sl_comppat = Vec::new();

    slang.sl_wordcount = WordCounts::default();
    slang.sl_map_hash = HashMap::new();

    slang_clear_sug(slang);

    slang.sl_compmax = MAXWLEN as c_int;
    slang.sl_compminlen = 0;
    slang.sl_compsylmax = MAXWLEN as c_int;
    slang.sl_regions[0] = 0;
}

/// Drop what the `.sug` file contributed, so it can be read again.
pub fn slang_clear_sug(slang: &mut SpellLang) {
    slang.sl_sound_tree = WordTree::default();
    slang.sl_sugbuf = None;
    slang.sl_sugloaded = false;
    slang.sl_sugtime = 0;
}

/// Note that `word` is a `COMMON` word of `slang`, or bump its count if it is
/// already known.
///
/// `count` is 1 to count one use and 10 to seed a word the `.spl` file
/// declared common. The count saturates rather than wrapping. A word of
/// [`MAXWLEN`] bytes or more is not counted.
pub fn count_common_word(slang: &mut SpellLang, word: &[u8], count: uint8_t) {
    if word.len() >= MAXWLEN {
        return;
    }
    let count = uint16_t::from(count);
    if let Some(total) = slang.sl_wordcount.get_mut(word) {
        // The C adds and then checks for the wrap, which is a saturate
        // spelled the long way round.
        *total = total.checked_add(count).unwrap_or(MAXWORDCOUNT as uint16_t);
    } else {
        slang.sl_wordcount.insert(word.into(), count);
    }
}

/// Split `SYLLABLE`'s value at its slashes: the part before the first
/// becomes `sl_syllable`, the set of single syllable characters, and each
/// part after it becomes an entry in `sl_syl_items`.
///
/// Returns `SP_FORMERROR` for an entry longer than [`SY_MAXLEN`].
pub fn init_syl_tab(slang: &mut SpellLang, value: &[u8]) -> c_int {
    let mut parts = value.split(|&b| b == b'/');
    slang.sl_syllable = Some(parts.next().unwrap_or_default().to_vec());
    let mut items: Vec<SylItem> = Vec::new();
    for item in parts {
        if item.is_empty() {
            // A trailing slash, or an empty item, which no word matches.
            continue;
        }
        if item.len() >= SY_MAXLEN as usize {
            return SP_FORMERROR;
        }
        items.push(SylItem {
            sy_chars: item.into(),
        });
    }
    slang.sl_syl_items = items;
    OK
}

/// How many syllables `word` has, by the language's syllable definition.
///
/// A space resets the count, so what is returned is the count after the
/// last space. Zero means the language defines no syllables. `word` ends
/// at its end or at a NUL.
pub(super) fn count_syllables(slang: &SpellLang, word: &[u8]) -> c_int {
    let Some(syllable) = &slang.sl_syllable else {
        return 0;
    };

    let mut cnt = 0;
    let mut skip = false;
    let mut p = 0;
    while p < word.len() && word[p] != 0 {
        if word[p] == b' ' {
            cnt = 0;
            p += 1;
            continue;
        }

        // The longest matching syllable item wins.
        let rest = &word[p..];
        let mut len = 0;
        for syl in &slang.sl_syl_items {
            if syl.sy_chars.len() > len && rest.starts_with(&syl.sy_chars) {
                len = syl.sy_chars.len();
            }
        }

        if len != 0 {
            cnt += 1;
            skip = false;
        } else {
            // No item matched; a bare syllable character still counts,
            // but only the first of a run.
            let c = char_at(rest);
            len = c_cluster_len(rest).max(1);
            if find_char(syllable, c).is_none() {
                skip = false;
            } else if !skip {
                cnt += 1;
                skip = true;
            }
        }
        p += len;
    }
    cnt
}

/// Every language loaded: a slot per language, never reused while the
/// table lives, and the order they were loaded in.
pub(crate) struct LoadedLangs {
    /// The languages, by [`LangId`]. A slot is empty while its language is
    /// being reloaded, and once [`spell_free_all`](super::spell_free_all)
    /// has dropped it; a window still naming it then sees no language.
    slots: Vec<Option<Box<SpellLang>>>,
    /// Newest first, which is the order a search for a language by name
    /// or file walks.
    order: Vec<LangId>,
}

impl LoadedLangs {
    const fn new() -> Self {
        LoadedLangs {
            slots: Vec::new(),
            order: Vec::new(),
        }
    }

    /// The language in slot `id`, if it is there.
    #[inline]
    pub(crate) fn get(&self, id: LangId) -> Option<&SpellLang> {
        self.slots.get(id.0 as usize)?.as_deref()
    }

    /// [`LoadedLangs::get`], writable.
    #[inline]
    pub(crate) fn get_mut(&mut self, id: LangId) -> Option<&mut SpellLang> {
        self.slots.get_mut(id.0 as usize)?.as_deref_mut()
    }

    /// The languages, newest first, with their slots.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (LangId, &SpellLang)> {
        self.order
            .iter()
            .filter_map(|&id| self.get(id).map(|slang| (id, slang)))
    }
}

/// The language table. Windows name its slots in their `b_langp`.
pub(crate) static loaded_langs: GlobalCell<LoadedLangs> = GlobalCell::new(LoadedLangs::new());

/// The loaded languages' slots, newest first: a copy to walk, because a
/// walk may load another language.
pub(crate) fn languages() -> Vec<LangId> {
    loaded_langs.with(|langs| langs.order.clone())
}

/// List a freshly read language as the newest, and answer its slot.
pub fn register_lang(slang: Box<SpellLang>) -> LangId {
    loaded_langs.with_mut(|langs| {
        let id = LangId(u32::try_from(langs.slots.len()).expect("fewer than 2^32 spell files"));
        langs.slots.push(Some(slang));
        langs.order.insert(0, id);
        id
    })
}

/// Run `f` over the language in slot `id`, if it is there.
pub(crate) fn with_lang<R>(id: LangId, f: impl FnOnce(&SpellLang) -> R) -> Option<R> {
    loaded_langs.with(|langs| langs.get(id).map(f))
}

/// Take the language out of slot `id` to change it with the table unlocked
/// -- a reload reads a file and gives messages -- leaving the slot empty
/// until [`put_lang`] returns it. A lookup meanwhile sees no language
/// there, as upstream's saw a cleared one.
pub fn take_lang(id: LangId) -> Option<Box<SpellLang>> {
    loaded_langs.with_mut(|langs| langs.slots.get_mut(id.0 as usize)?.take())
}

/// Return a language [`take_lang`] took to its slot.
pub(crate) fn put_lang(id: LangId, slang: Box<SpellLang>) {
    loaded_langs.with_mut(|langs| langs.slots[id.0 as usize] = Some(slang));
}

/// Drop every language. The slots stay allocated, empty, so that an id a
/// window still holds names nothing rather than a later language.
pub(crate) fn free_langs() {
    let dropped = loaded_langs.with_mut(|langs| {
        langs.order.clear();
        langs.slots.iter_mut().map(Option::take).collect::<Vec<_>>()
    });
    // Dropped outside the borrow: a `.sug` buffer's drop closes a memline.
    drop(dropped);
}

/// A nameless, unlisted buffer holding nothing but text lines, backed by a
/// swap file so that a big `.sug` word list need not stay in memory. Made by
/// [`open_spellbuf`].
///
/// Never registered and never on the buffer list -- see
/// `alloc_unregistered_buffer` -- so it has no number and nothing can find
/// it: this value is its only owner. Dropping it closes the memline
/// (deleting the swap file) and frees the buffer.
pub struct SpellBuf(Owned<Buffer>);

impl SpellBuf {
    /// The buffer, to read and append lines through. Live while `self` is.
    pub(crate) fn buf(&self) -> Buf {
        Buf::owned_by(&self.0)
    }
}

impl Drop for SpellBuf {
    fn drop(&mut self) {
        ml_close(self.buf(), 1);
        // The `Owned` frees the buffer once this returns.
    }
}

/// Open a [`SpellBuf`].
///
/// Most of its fields are invalid: string options are null and there is no
/// undo information.
pub fn open_spellbuf() -> SpellBuf {
    let owner = SpellBuf(alloc_unregistered_buffer());
    let mut buf = owner.buf();

    buf.b_spell = true;
    buf.b_p_swf = 1;
    if ml_open(buf).is_err() {
        logmsg!(
            LOGLVL_ERR,
            c"open_spellbuf",
            line!() as c_int,
            "Error opening a new memline"
        );
    }
    ml_open_file(buf); // create the swap file now

    owner
}
