#![forbid(unsafe_code)]
// The constants here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::arglist::expand_arglist;
use crate::charset::getdigits_int_at;
use crate::cstr;
use crate::getchar::state::got_int;
use core::cell::RefCell;

use crate::memory::XString;
use crate::message::{e_exists, e_invarg};
use crate::message::{emsg, msg, verbose_enter, verbose_leave};
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::option::vars::{p_msm, p_verbose};
use crate::os::cshim::gettext;
use crate::os::fs::{dir_exists, path_exists};
use crate::path::{path_tail, tail_index};
use crate::semsg;
use crate::spell::SpellBuf;
use crate::spell::{did_set_spelltab, spell_enc, spelltab};
use crate::strings::has_bytes;
use crate::strings::has_char;
use crate::types::CmdIdx;
use crate::types::TAB;
use crate::types::{
    CONV_NONE, EStackType, ExArg, FAIL, Failed, FileComparison, MAXPATHL, NUL, OK, OptInt, RepItem,
    SpellAddType, SpellTab, VimConv, XDGVarType, time_t,
};
use crate::ui::ui_flush;
use crate::winlayer::Buf;
use core::ffi::CStr;
mod add;
mod aff;
mod affix;
mod dic;
mod flags;
mod keyed;
mod read;
mod sections;
mod spl;
mod sugfile;
mod tables;
mod wordfile;
mod wordtree;
mod write;
use crate::regexp::{OwnedProg, vim_regcomp, vim_regexec_prog};
pub use add::spell_add_word;
use aff::spell_read_aff;
use dic::spell_read_dic;
use keyed::{Keyed, KeyedTable};
use read::spell_reload_one;
pub use read::{spell_load_file, suggest_load_files};
use sugfile::spell_make_sugfile;
use wordfile::spell_read_wordfile;
use wordtree::{
    MSG_COMPRESSING, NodeId, SpellArena, WordNode, set_compression_limits, wordtree_alloc,
    wordtree_compress,
};
use write::write_vim_spell;
pub const ETYPE_SPELL: EStackType = 9;
pub const kXDGDataHome: XDGVarType = 1;
pub const kEqualFiles: FileComparison = 1;
pub use crate::spell::MAXWLEN;
pub const MAXREGIONS: ::core::ffi::c_uint = 8;
pub const WFP_COMPFORBID: ::core::ffi::c_uint = 16;
pub const WFP_COMPPERMIT: ::core::ffi::c_uint = 8;
pub const WFP_UP: ::core::ffi::c_uint = 4;
pub const WFP_NC: ::core::ffi::c_uint = 2;
pub const COMP_CHECKTRIPLE: ::core::ffi::c_uint = 8;
pub const COMP_CHECKCASE: ::core::ffi::c_uint = 4;
pub const COMP_CHECKREP: ::core::ffi::c_uint = 2;
pub const COMP_CHECKDUP: ::core::ffi::c_uint = 1;
pub const SP_OTHERERROR: ::core::ffi::c_int = -3;
pub const SP_FORMERROR: ::core::ffi::c_int = -2;
pub const SP_TRUNCERROR: ::core::ffi::c_int = -1;
pub const SPELL_ADD_RARE: SpellAddType = 2;
pub const SPELL_ADD_BAD: SpellAddType = 1;
pub const SPELL_ADD_GOOD: SpellAddType = 0;
pub const BY_FLAGS2: ::core::ffi::c_uint = 3;
pub const BY_FLAGS: ::core::ffi::c_uint = 2;
pub const BY_INDEX: ::core::ffi::c_uint = 1;
pub const BY_NOFLAGS: ::core::ffi::c_uint = 0;
pub const BY_SPECIAL: ::core::ffi::c_uint = 3;
pub const SN_SYLLABLE: ::core::ffi::c_uint = 9;
pub const SN_NOBREAK: ::core::ffi::c_uint = 10;
pub const SN_COMPOUND: ::core::ffi::c_uint = 8;
pub const SN_NOCOMPOUNDSUGS: ::core::ffi::c_uint = 16;
pub const SN_NOSPLITSUGS: ::core::ffi::c_uint = 14;
pub const SN_SUGFILE: ::core::ffi::c_uint = 11;
pub const SN_WORDS: ::core::ffi::c_uint = 13;
pub const SN_MAP: ::core::ffi::c_uint = 7;
pub const SN_SOFO: ::core::ffi::c_uint = 6;
pub const SAL_REM_ACCENTS: ::core::ffi::c_uint = 4;
pub const SAL_COLLAPSE: ::core::ffi::c_uint = 2;
pub const SAL_F0LLOWUP: ::core::ffi::c_uint = 1;
pub const SN_SAL: ::core::ffi::c_uint = 5;
pub const SN_REPSAL: ::core::ffi::c_uint = 12;
pub const SN_REP: ::core::ffi::c_uint = 4;
pub const SN_PREFCOND: ::core::ffi::c_uint = 3;
pub const SN_MIDWORD: ::core::ffi::c_uint = 2;
pub const CF_UPPER: ::core::ffi::c_uint = 2;
pub const CF_WORD: ::core::ffi::c_uint = 1;
pub const SN_CHARFLAGS: ::core::ffi::c_uint = 1;
pub const SN_REGION: ::core::ffi::c_uint = 0;
pub const SN_INFO: ::core::ffi::c_uint = 15;
pub const SN_END: ::core::ffi::c_uint = 255;
/// A spell file under construction: what `:mkspell` has read so far.
///
/// Parse-time only. The trees live in [`si_nodes`](Self::si_nodes) and are
/// reached through the three roots; every string is owned.
pub(crate) struct SpellInfo {
    si_foldroot: Option<NodeId>,
    si_foldwcount: ::core::ffi::c_int,
    si_keeproot: Option<NodeId>,
    si_keepwcount: ::core::ffi::c_int,
    si_prefroot: Option<NodeId>,
    si_sugtree: ::core::ffi::c_int,
    /// The nodes of every tree; see `wordtree`.
    si_nodes: Vec<WordNode>,
    si_arena: SpellArena,
    si_compress_cnt: ::core::ffi::c_int,
    si_first_free: Option<NodeId>,
    si_free_count: ::core::ffi::c_int,
    /// The `.sug` tree's runs of word ends: by head, the word end last
    /// filed; see `wordtree`.
    si_end_hint: wordtree::EndHints,
    si_spellbuf: Option<SpellBuf>,
    si_ascii: ::core::ffi::c_int,
    si_add: ::core::ffi::c_int,
    si_clear_chartab: ::core::ffi::c_int,
    si_region: ::core::ffi::c_int,
    si_conv: VimConv,
    si_memtot: ::core::ffi::c_int,
    si_verbose: ::core::ffi::c_int,
    si_msg_count: ::core::ffi::c_int,
    /// The `NAME`/`VERSION`/... lines `:spellinfo` shows.
    si_info: Option<Vec<u8>>,
    si_region_count: ::core::ffi::c_int,
    si_region_name: [u8; 17],
    /// `REP` pairs, sorted by the writer.
    si_rep: Vec<RepItem>,
    /// `REPSAL` pairs, likewise.
    si_repsal: Vec<RepItem>,
    /// `SAL` rules, which keep the order the affix file gave.
    si_sal: Vec<RepItem>,
    si_sofofr: Option<Vec<u8>>,
    si_sofoto: Option<Vec<u8>>,
    si_nosugfile: ::core::ffi::c_int,
    si_nosplitsugs: ::core::ffi::c_int,
    si_nocompoundsugs: ::core::ffi::c_int,
    si_followup: ::core::ffi::c_int,
    si_collapse: ::core::ffi::c_int,
    /// The `COMMON` words, written in the table's slot order.
    si_commonwords: KeyedTable<Box<[u8]>>,
    si_sugtime: time_t,
    si_rem_accents: ::core::ffi::c_int,
    /// `MAP` groups, `/`-separated, as the section's payload.
    si_map: Vec<u8>,
    si_midword: Option<Vec<u8>>,
    si_compmax: ::core::ffi::c_int,
    si_compminlen: ::core::ffi::c_int,
    si_compsylmax: ::core::ffi::c_int,
    si_compoptions: ::core::ffi::c_int,
    /// `CHECKCOMPOUNDPATTERN`'s strings, in pairs.
    si_comppat: Vec<Box<[u8]>>,
    /// The compound pattern, in compound ids.
    si_compflags: Option<Vec<u8>>,
    si_nobreak: ::core::ffi::c_char,
    si_syllable: Option<Vec<u8>>,
    /// One condition per postponed-prefix id; `None` where an id has
    /// none, which the writer emits as a zero length and no payload.
    si_prefcond: Vec<Option<Box<[u8]>>>,
    si_newpref_id: ::core::ffi::c_int,
    si_newcomp_id: ::core::ffi::c_int,
}
/// What one `.aff` file declared.
///
/// Parse-time only: `:mkspell` reads the affix and dictionary files into
/// this and the word trees, and [`write::write_vim_spell`] then emits the
/// `.spl` a field at a time.
pub(crate) struct AffFile {
    /// The `SET` encoding, canonized.
    af_enc: Option<XString>,
    af_flagtype: ::core::ffi::c_int,
    af_rare: ::core::ffi::c_uint,
    af_keepcase: ::core::ffi::c_uint,
    af_bad: ::core::ffi::c_uint,
    af_needaffix: ::core::ffi::c_uint,
    af_circumfix: ::core::ffi::c_uint,
    af_needcomp: ::core::ffi::c_uint,
    af_comproot: ::core::ffi::c_uint,
    af_compforbid: ::core::ffi::c_uint,
    af_comppermit: ::core::ffi::c_uint,
    af_nosuggest: ::core::ffi::c_uint,
    af_pfxpostpone: ::core::ffi::c_int,
    af_ignoreextra: bool,
    af_pref: KeyedTable<AffHeader>,
    af_suff: KeyedTable<AffHeader>,
    af_comp: KeyedTable<CompItem>,
}

impl AffFile {
    fn new() -> Self {
        AffFile {
            af_enc: None,
            af_flagtype: AFT_CHAR,
            af_rare: 0,
            af_keepcase: 0,
            af_bad: 0,
            af_needaffix: 0,
            af_circumfix: 0,
            af_needcomp: 0,
            af_comproot: 0,
            af_compforbid: 0,
            af_comppermit: 0,
            af_nosuggest: 0,
            af_pfxpostpone: 0,
            af_ignoreextra: false,
            af_pref: KeyedTable::new(),
            af_suff: KeyedTable::new(),
            af_comp: KeyedTable::new(),
        }
    }

    /// The prefix or the suffix table.
    fn affixes(&self, kind: AffixKind) -> &KeyedTable<AffHeader> {
        match kind {
            AffixKind::Prefix => &self.af_pref,
            AffixKind::Suffix => &self.af_suff,
        }
    }

    fn affixes_mut(&mut self, kind: AffixKind) -> &mut KeyedTable<AffHeader> {
        match kind {
            AffixKind::Prefix => &mut self.af_pref,
            AffixKind::Suffix => &mut self.af_suff,
        }
    }
}

/// Which of an affix file's two affix tables.
#[derive(Copy, Clone, PartialEq, Eq)]
enum AffixKind {
    Prefix,
    Suffix,
}

/// One `PFX`/`SFX` line: what to chop, what to add, and when. Each string
/// is the item without its terminator; `None` is upstream's null (`0`).
pub(crate) struct AffEntry {
    ae_chop: Option<Vec<u8>>,
    ae_add: Option<Vec<u8>>,
    /// The flags the added form carries, after its `/`.
    ae_flags: Option<Vec<u8>>,
    ae_cond: Option<Vec<u8>>,
    /// `ae_cond` compiled. In a cell because matching may recompile it
    /// while the affix tables are borrowed for the walk.
    ae_prog: RefCell<Option<OwnedProg>>,
    ae_compforbid: bool,
    ae_comppermit: bool,
}

/// All the `PFX`/`SFX` blocks that share one affix name.
pub(crate) struct AffHeader {
    ah_key: Vec<u8>,
    ah_flag: ::core::ffi::c_uint,
    ah_new_id: ::core::ffi::c_int,
    ah_combine: ::core::ffi::c_int,
    ah_follows: ::core::ffi::c_int,
    /// The entries, oldest first; upstream's list ran newest first, so a
    /// walk is `ah_entries.iter().rev()`.
    ah_entries: Vec<AffEntry>,
}

impl Keyed for AffHeader {
    fn key(&self) -> &[u8] {
        &self.ah_key
    }
}

/// One `COMPOUNDFLAG` value and the internal id standing in for it.
pub(crate) struct CompItem {
    ci_key: Vec<u8>,
    ci_new_id: ::core::ffi::c_int,
}

impl Keyed for CompItem {
    fn key(&self) -> &[u8] {
        &self.ci_key
    }
}
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SPL_FNAME_ADD: &::core::ffi::CStr = c".add.";
pub const SPL_FNAME_ASCII: &::core::ffi::CStr = c".ascii.";
pub const VIMSUGMAGIC: &::core::ffi::CStr = c"VIMsug";
pub const VIMSUGMAGICL: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const VIMSUGVERSION: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ZERO_FLAG: ::core::ffi::c_int = 65009 as ::core::ffi::c_int;
pub const VIMSPELLMAGIC: &::core::ffi::CStr = c"VIMspell";
pub const VIMSPELLMAGICL: usize =
    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1_usize);
pub const VIMSPELLVERSION: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
pub const SNF_REQUIRED: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const COMPOUND_MAX_LEN: ::core::ffi::c_int = 100000 as ::core::ffi::c_int;
static e_spell_trunc: &::core::ffi::CStr = c"E758: Truncated spell file";
static e_duplicate_char_in_map_entry: &::core::ffi::CStr = c"E783: Duplicate char in MAP entry";
static e_illegal_character_in_word: &::core::ffi::CStr = c"E1280: Illegal character in word";
pub const MAXLINELEN: ::core::ffi::c_int = 500 as ::core::ffi::c_int;
pub const AFT_CHAR: ::core::ffi::c_int = 0;
pub const AFT_LONG: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const AFT_CAPLONG: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AFT_NUM: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const AH_KEY_LEN: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const PFX_FLAGS: ::core::ffi::c_int = -256 as ::core::ffi::c_int;
pub const CONDIT_COMB: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const CONDIT_CFIX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CONDIT_SUF: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const CONDIT_AFF: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
impl SpellInfo {
    /// The `.sug` scratch buffer `sug_maketable` opened.
    ///
    /// # Panics
    ///
    /// Before `sug_maketable` has run.
    pub(super) fn spellbuf(&self) -> Buf {
        self.si_spellbuf
            .as_ref()
            .expect("sug_maketable opens the scratch buffer first")
            .buf()
    }

    /// A spell file under construction, with nothing read yet.
    ///
    /// The C listed an initialiser and then `memset` the whole struct to
    /// zero over the top of it, so every field really does start at zero;
    /// only the four values [`mkspell`] assigns straight afterwards differ.
    fn new() -> Self {
        Self {
            si_foldroot: None,
            si_foldwcount: 0,
            si_keeproot: None,
            si_keepwcount: 0,
            si_prefroot: None,
            si_sugtree: 0,
            si_nodes: wordtree::empty_slab(),
            si_arena: SpellArena::new(),
            si_compress_cnt: 0,
            si_first_free: None,
            si_free_count: 0,
            si_end_hint: wordtree::EndHints::default(),
            si_spellbuf: None,
            si_ascii: 0,
            si_add: 0,
            si_clear_chartab: 0,
            si_region: 0,
            si_conv: VimConv {
                vc_type: 0,
                vc_factor: 0,
                vc_fd: None,
                vc_fail: false,
            },
            si_memtot: 0,
            si_verbose: 0,
            si_msg_count: 0,
            si_info: None,
            si_region_count: 0,
            si_region_name: [0; 17],
            si_rep: Vec::new(),
            si_repsal: Vec::new(),
            si_sal: Vec::new(),
            si_sofofr: None,
            si_sofoto: None,
            si_nosugfile: 0,
            si_nosplitsugs: 0,
            si_nocompoundsugs: 0,
            si_followup: 0,
            si_collapse: 0,
            si_commonwords: KeyedTable::new(),
            si_sugtime: 0,
            si_rem_accents: 0,
            si_map: Vec::new(),
            si_midword: None,
            si_compmax: 0,
            si_compminlen: 0,
            si_compsylmax: 0,
            si_compoptions: 0,
            si_comppat: Vec::new(),
            si_compflags: None,
            si_nobreak: 0,
            si_syllable: None,
            si_prefcond: Vec::new(),
            si_newpref_id: 0,
            si_newcomp_id: 0,
        }
    }
}

/// Validate `'mkspellmem'` and install the compression limits it names.
///
/// The option is three comma-separated numbers: the arena size at which the
/// first compression run happens, how much more has to be taken before the
/// next one, and how many words may be added in between. The first two are
/// given in megabytes and scaled to arena blocks here; the third is in
/// thousands of words.
pub fn spell_check_msm() -> Result<(), Failed> {
    let parsed = p_msm(|value| parse_mkspellmem(value.to_bytes()));
    let Some((start, incr, added)) = parsed else {
        return Err(Failed);
    };

    let block = wordtree::block_size();
    let start = start * 10 / (block / 102);
    let incr = incr * 102 / (block / 10);
    let added = added * 1024;
    if start == 0 || incr == 0 || added == 0 || incr > start {
        return Err(Failed);
    }
    set_compression_limits(start, incr, added);
    Ok(())
}

/// The three unscaled numbers of a `'mkspellmem'` value, or `None` when the
/// string is not exactly `<digits>,<digits>,<digits>`.
fn parse_mkspellmem(
    value: &[u8],
) -> Option<(::core::ffi::c_int, ::core::ffi::c_int, ::core::ffi::c_int)> {
    let mut at = 0;
    let start = digits_then(value, &mut at, b',')?;
    let incr = digits_then(value, &mut at, b',')?;
    let added = digits_then(value, &mut at, NUL as u8)?;
    Some((start, incr, added))
}

/// Read a decimal number that `sep` must follow, stepping `at` past both.
/// A `sep` of [`NUL`] means "and that must be the end of the string", where
/// there is nothing to step over.
fn digits_then(text: &[u8], at: &mut usize, sep: u8) -> Option<::core::ffi::c_int> {
    if !cstr::byte_at(text, *at).is_ascii_digit() {
        return None;
    }
    let (n, past) = getdigits_int_at(text, *at, true, 0);
    *at = past;
    if cstr::byte_at(text, *at) != sep {
        return None;
    }
    if sep != NUL as u8 {
        *at += 1;
    }
    Some(n)
}

/// `:mkspell[!] [-ascii] {outfile} {infile} ...`
pub fn ex_mkspell(excmd: &mut ExArg) {
    let mut arg = excmd.line.arg();
    let mut ascii = false;
    if arg.starts_with(b"-ascii") {
        ascii = true;
        arg = &arg[6..];
        arg = &arg[crate::charset::skip::white(arg)..];
    }

    let Ok(names) = expand_arglist(arg, false) else {
        return;
    };
    let fnames: Vec<&[u8]> = names.iter().map(|name| &name[..]).collect();
    mkspell(&fnames, ascii, excmd.forceit, false);
}

/// Read `.aff`/`.dic` pairs or word lists and write the `.spl` they make.
///
/// `fnames[0]` names the output; the rest are inputs, one per region. A
/// single name is both the input stem and, with a suffix, the output.
///
/// `pub` for `crates/nvim/tests/unit/spellfile.rs`: the `.spl` byte golden
/// drives the writer directly rather than through `:mkspell`, so that a
/// change to the format fails a test that names the format rather than one
/// that happens to spell-check a word.
pub fn mkspell(fnames: &[&[u8]], ascii: bool, over_write: bool, added_word: bool) {
    let fcount = ::core::ffi::c_int::try_from(fnames.len()).unwrap_or(::core::ffi::c_int::MAX);
    let mut afile: Vec<Option<AffFile>> = Vec::new();
    let mut spin = SpellInfo::new();
    spin.si_verbose = ::core::ffi::c_int::from(!added_word);
    spin.si_ascii = ::core::ffi::c_int::from(ascii);
    spin.si_followup = 1;
    spin.si_rem_accents = 1;
    spin.si_newcomp_id = 127;

    // With one name it is both the input stem and the output; with
    // more, the first is the output and the rest are the inputs.
    let innames = fnames.get(usize::from(fcount != 1)..).unwrap_or_default();
    let mut incount = fcount - 1;
    let mut wfname = XString::new();
    if let Some(&first) = fnames.first() {
        (wfname, incount) = output_name(first, fcount, spin.si_ascii != 0, incount);
        let tail = path_tail(wfname.as_cstr());
        if has_bytes(tail, SPL_FNAME_ASCII.to_bytes()) {
            spin.si_ascii = 1;
        }
        if has_bytes(tail, SPL_FNAME_ADD.to_bytes()) {
            spin.si_add = 1;
        }
    }
    let wfname = wfname.as_cstr();

    '_theend: {
        if !output_is_writable(wfname, incount, over_write) {
            break '_theend;
        }
        let innames = &innames[..usize::try_from(incount).unwrap_or(0).min(innames.len())];
        if !read_region_names(&mut spin, innames) {
            break '_theend;
        }
        spin.si_region_count = incount;
        spin.si_foldroot = Some(wordtree_alloc(&mut spin));
        spin.si_keeproot = Some(wordtree_alloc(&mut spin));
        spin.si_prefroot = Some(wordtree_alloc(&mut spin));
        if spin.si_add == 0 {
            spin.si_clear_chartab = 1;
        }

        let mut error = read_inputs(&mut spin, innames, &mut afile);
        if spin.si_compflags.is_some() && spin.si_nobreak != 0 {
            let text = c"Warning: both compounding and NOBREAK specified";
            msg(gettext(text), 0);
        }
        if !error && !got_int.get() {
            spell_message(&spin, MSG_COMPRESSING);
            for (root, name) in [
                (spin.si_foldroot, c"case-folded"),
                (spin.si_keeproot, c"keep-case"),
                (spin.si_prefroot, c"prefixes"),
            ] {
                wordtree_compress(&mut spin, root, name);
            }
        }
        if !error && !got_int.get() {
            let name = wfname.to_string_lossy();
            spell_message_fmt(&spin, format_args!("Writing spell file {name}..."));
            error = write_vim_spell(&mut spin, wfname).is_err();
            spell_message(&spin, c"Done!");
            let used = spin.si_memtot;
            spell_message_fmt(
                &spin,
                format_args!("Estimated runtime memory use: {used} bytes"),
            );
            if !error {
                spell_reload_one(wfname, added_word);
            }
        }

        spin.si_rep = Vec::new();
        spin.si_repsal = Vec::new();
        spin.si_sal = Vec::new();
        spin.si_map = Vec::new();
        spin.si_comppat = Vec::new();
        spin.si_prefcond = Vec::new();
        spin.si_commonwords = KeyedTable::new();
        afile = Vec::new();
        // The `.sug` pass reads the file back and builds its own tree,
        // so the trees go first.
        spin.clear_trees();
        if spin.si_sugtime != 0 && !error && !got_int.get() {
            spell_make_sugfile(&mut spin, wfname);
        }
    }
    drop(afile);
}

/// The output path, and how many inputs there are.
///
/// A lone `foo.add` writes `foo.add.spl`; any other lone name is a stem the
/// encoding is appended to; a first name already ending in `.spl` is taken
/// as it stands. `default_incount` is what to report when the first name
/// was only the output.
///
/// The cap stays: upstream truncated at `MAXPATHL`, and a path longer than
/// that only fails to open, so truncating here keeps the same answer.
fn output_name(
    first: &[u8],
    fcount: ::core::ffi::c_int,
    ascii: bool,
    default_incount: ::core::ffi::c_int,
) -> (XString, ::core::ffi::c_int) {
    let ends_with = |ext: &CStr| {
        let ext = ext.to_bytes();
        first.len() > ext.len() && first.ends_with(ext)
    };
    let encoding = spell_enc();
    let enc: &[u8] = if ascii { b"ascii" } else { &encoding };

    let mut name = XString::new();
    if ends_with(c".spl") && fcount != 1 {
        name.push_bytes(first);
    } else {
        name.push_bytes(first);
        if !(fcount == 1 && ends_with(c".add")) {
            name.push_byte(b'.');
            name.push_bytes(enc);
        }
        name.push_bytes(b".spl");
    }
    name.truncate(name.len().min(MAXPATHL as usize - 1));
    (name, if fcount == 1 { 1 } else { default_incount })
}

/// Can `:mkspell` write here? Reports why not, if not.
fn output_is_writable(wfname: &CStr, incount: ::core::ffi::c_int, over_write: bool) -> bool {
    if incount <= 0 {
        emsg(gettext(e_invarg));
    } else if has_char(path_tail(wfname), '_' as ::core::ffi::c_int) {
        emsg(gettext(c"E751: Output file name must not have region name"));
    } else if incount > MAXREGIONS as ::core::ffi::c_int {
        semsg!(
            "E754: Only up to {} regions supported",
            MAXREGIONS as ::core::ffi::c_int
        );
    } else if !over_write && path_exists(wfname) {
        emsg(gettext(e_exists));
    } else if dir_exists(wfname) {
        let wfname = msg_cstr(wfname);
        semsg!("E17: \"{wfname}\" is a directory");
    } else {
        return true;
    }
    false
}

/// Fill in `si_region_name` from the `_xy` each input name ends with.
///
/// A single input covers every region and carries no suffix. Returns false,
/// having reported the name, when one of several does not carry one.
fn read_region_names(spin: &mut SpellInfo, innames: &[&[u8]]) -> bool {
    if innames.len() <= 1 {
        return true;
    }
    for (i, name) in innames.iter().enumerate() {
        let len = name.len();
        if len - tail_index(name) < 5 || name[len - 3] != b'_' {
            let name = msg_bytes(name);
            semsg!("E755: Invalid region in {name}");
            return false;
        }
        spin.si_region_name[i * 2] = name[len - 2].to_ascii_lowercase();
        spin.si_region_name[i * 2 + 1] = name[len - 1].to_ascii_lowercase();
    }
    true
}

/// Read every input into `spin`: an `.aff`/`.dic` pair where an `.aff`
/// exists, a plain word list otherwise. Returns whether one of them failed,
/// which stops the rest. The affix files read go to `afile`, which owns
/// them until the run is over.
fn read_inputs(spin: &mut SpellInfo, innames: &[&[u8]], afile: &mut Vec<Option<AffFile>>) -> bool {
    for (i, &stem) in innames.iter().enumerate() {
        spin.si_conv.vc_type = CONV_NONE;
        spin.si_region = 1 << i;
        let with_ext = |ext: &[u8]| {
            let mut name = XString::from_bytes(stem);
            name.push_bytes(ext);
            name
        };

        let aff_name = with_ext(b".aff");
        let failed = if path_exists(aff_name.as_cstr()) {
            let aff = spell_read_aff(spin, aff_name.as_cstr());
            let failed = match aff {
                None => true,
                Some(ref aff) => {
                    let dic_name = with_ext(b".dic");
                    spell_read_dic(spin, dic_name.as_cstr(), aff).is_err()
                }
            };
            afile.push(aff);
            failed
        } else {
            spell_read_wordfile(spin, XString::from_bytes(stem).as_cstr()).is_err()
        };
        spin.si_conv = crate::types::CONV_NONE_INIT;
        if failed {
            return true;
        }
    }
    false
}

/// Show `text` while `:mkspell` runs, quietly unless it was asked to be
/// verbose or `'verbose'` is high enough to want it anyway.
fn spell_message(spin: &SpellInfo, text: &CStr) {
    if spin.si_verbose == 0 && p_verbose() <= 2 as OptInt {
        return;
    }
    let quiet = spin.si_verbose == 0;
    if quiet {
        verbose_enter();
    }
    msg(text, 0);
    ui_flush();
    if quiet {
        verbose_leave();
    }
}

/// [`spell_message`] with a compile-checked format string.
///
/// Every progress line upstream formats goes through the shared `IObuff`,
/// which `msg` -- and the autocommands it can run -- may write in the middle
/// of the report. The message is built here and owned until it is shown.
pub(super) fn spell_message_fmt(spin: &SpellInfo, args: core::fmt::Arguments<'_>) {
    if spin.si_verbose == 0 && p_verbose() <= 2 as OptInt {
        return;
    }
    spell_message(spin, &cstr::owned(args.to_string().as_bytes()));
}

/// `:spellgood`, `:spellwrong`, `:spellrare` and their `:spell*undo` forms.
pub fn ex_spell(excmd: &mut ExArg) {
    let (cmdidx, forceit, line2) = (excmd.cmdidx, excmd.forceit, excmd.line2);
    // A copy of the word: adding it writes files and may run autocommands.
    let word = excmd.line.arg().to_vec();

    let kind = if cmdidx == CmdIdx::spellwrong {
        SPELL_ADD_BAD
    } else if cmdidx == CmdIdx::spellrare {
        SPELL_ADD_RARE
    } else {
        SPELL_ADD_GOOD
    };
    // `:N spellgood` files the word in the Nth 'spellfile'; `!` means the
    // internal word list instead.
    let which = if forceit {
        0
    } else {
        line2 as ::core::ffi::c_int
    };

    let undo = cmdidx == CmdIdx::spellundo;
    spell_add_word(&word, kind as SpellAddType, which, undo);
}

/// Adopt `new_st` as the word character table, or check that it agrees with
/// the one already in force.
///
/// The table is global, so two spell files that disagree about which bytes
/// are word characters cannot be loaded together.
fn set_spell_finish(new_st: &SpellTab) -> Result<(), Failed> {
    if !did_set_spelltab.get() {
        spelltab.set(*new_st);
        did_set_spelltab.set(true);
        return Ok(());
    }
    let agrees = spelltab.with(|st| {
        (0..256).all(|i| {
            st.st_isw[i] == new_st.st_isw[i]
                && st.st_isu[i] == new_st.st_isu[i]
                && st.st_fold[i] == new_st.st_fold[i]
                && st.st_upper[i] == new_st.st_upper[i]
        })
    });
    if !agrees {
        emsg(gettext(c"E763: Word characters differ between spell files"));
        return Err(Failed);
    }
    Ok(())
}
