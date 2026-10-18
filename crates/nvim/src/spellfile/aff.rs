//! Reading a Hunspell `.aff` file.
//!
//! An affix file is line-based: the first word is a keyword, the rest are
//! its arguments. [`spell_read_aff`] splits each line into items and works
//! down a list of keywords looking for one that matches.
//!
//! # The chain, and why it is a chain
//!
//! The keyword tests are tried in order and the *last* arm reports
//! "Unrecognized **or duplicate** item". That second word is the point: a
//! keyword that may only appear once carries a guard — `SYLLABLE` only
//! matches while no syllable string has been seen yet — and when the guard
//! fails the line does not match that arm, falls past every later one, and
//! comes out as an error. Ordering and fall-through are therefore part of
//! the behaviour, not an accident of how it was written, and the tables
//! below keep both: each table is tried at the position its keywords
//! occupied, and an entry whose guard fails leaves the line unclaimed.
//!
//! # Affix blocks
//!
//! `PFX`/`SFX` come in two shapes. A header line names the affix, says
//! whether it combines with affixes at the other end, and how many entries
//! follow; each entry then says what to chop off, what to add, and the
//! condition the word must satisfy. `aff_todo` counts the entries still
//! expected, which is also what tells a header apart from an entry.
//!
//! With `PFXPOSTPONE`, a prefix that only adds letters is not expanded into
//! the word list at all — it goes into the prefix tree with an id, and the
//! condition it needs is filed in `si_prefcond` for the reader to compile.
//!
//! # The items
//!
//! Upstream keeps the split line as `char *items[MAXITEMCNT]` pointing into
//! the line buffer, with a NUL written over each separator. The terminators
//! stay -- nearly every consumer hands an item straight on to a callee that
//! takes a C string, and a copy per item would be the only other way to say
//! that -- so an item is a **`&CStr` borrowed from the line**, and the
//! vector of them is what [`split_items`] answers.
//!
//! | upstream | here |
//! | --- | --- |
//! | `split_items(line, items)` | [`split_items`] answers the items |
//! | `spell_info_item(s)` | [`is_info_keyword`] |
//! | `*items[n]` | [`first_byte`], or `items[n].to_bytes()` |
//! | `atoi(items[n])` | [`item_number`] |
//! | `STRCMP(items[n], "X")` | `items[n] == c"X"` |

#![forbid(unsafe_code)]

use crate::semsg;
use crate::smsg;
use crate::tr_c;
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::ffi::OsStrExt;

use crate::charset::atoi;
use crate::cstr;
use crate::fileio::Fgets;
use crate::getchar::state::got_int;
use crate::mbyte::canonize;
use crate::message::msg;
use crate::message_fmt::{msg_bytes, msg_cstr, report_msg};
use crate::option::vars::p_enc;
use crate::os::cshim::gettext;
use crate::os::input::line_breakcheck;
use crate::spell::init_spell_chartab;
use crate::types::{CONV_NONE, NUL, uint8_t};

use super::affix::{handle_affix_entry, handle_affix_header};
use super::flags::{affitem2flag, process_compflags};
use super::tables::{add_comppat, add_rep_entry, append_info, handle_map, handle_sal};
use super::wordtree::AFF_FILE_BYTES;
use super::{
    AFT_CAPLONG, AFT_CHAR, AFT_LONG, AFT_NUM, AffFile, AffixKind, COMP_CHECKCASE, COMP_CHECKDUP,
    COMP_CHECKREP, COMP_CHECKTRIPLE, MAXLINELEN, SpellInfo, TAB, spell_message_fmt,
};

/// The most items one `.aff` line is split into; the rest are ignored.
pub(super) const MAXITEMCNT: usize = 30;

/// What one `.aff` file accumulates while its lines are read.
///
/// Most of this is not applied until the end: several keywords have to be
/// checked against what an *earlier* `.aff` file of the same run already
/// set, which cannot happen until the file is known to be complete.
pub(super) struct AffState {
    /// Entries still expected in the affix block being read.
    pub aff_todo: c_int,
    /// The header of the block being read: which table, and where in it.
    pub cur_aff: Option<(AffixKind, usize)>,
    /// Whether any entry of the current prefix block actually went into the
    /// prefix tree; if none did, its id is handed back.
    pub did_postpone_prefix: bool,
    /// The first `MAP` line is a count, not a mapping.
    pub found_map: bool,

    pub compmax: c_int,
    pub compminlen: c_int,
    pub compsylmax: c_int,
    pub compoptions: c_int,
    pub compflags: Option<Vec<u8>>,

    pub midword: Option<Vec<u8>>,
    pub syllable: Option<Vec<u8>>,
    pub sofofrom: Option<Vec<u8>>,
    pub sofoto: Option<Vec<u8>>,
    /// Whether this file carried a `LOW`, `FOL` or `UPP` line. The tables
    /// themselves are **not** kept: upstream copies each one and frees it
    /// again without ever reading it (its own `TODO: also use FOL and UPP`),
    /// so all the parse takes from them is that one of the three was seen,
    /// which is what rebuilds the word-character table.
    pub low: bool,
    pub fol: bool,
    pub upp: bool,

    /// Only the first `.aff` file of a run contributes these tables.
    pub do_rep: bool,
    pub do_repsal: bool,
    pub do_sal: bool,
    pub do_mapline: bool,
}

/// A keyword that declares one flag, and the field it fills in.
#[derive(Copy, Clone, PartialEq, Eq)]
enum FlagField {
    Rare,
    KeepCase,
    Bad,
    NeedAffix,
    Circumfix,
    NoSuggest,
    NeedComp,
    CompRoot,
    CompForbid,
    CompPermit,
}

impl FlagField {
    /// The field of `aff` this keyword sets.
    fn slot(self, aff: &mut AffFile) -> &mut c_uint {
        match self {
            Self::Rare => &mut aff.af_rare,
            Self::KeepCase => &mut aff.af_keepcase,
            Self::Bad => &mut aff.af_bad,
            Self::NeedAffix => &mut aff.af_needaffix,
            Self::Circumfix => &mut aff.af_circumfix,
            Self::NoSuggest => &mut aff.af_nosuggest,
            Self::NeedComp => &mut aff.af_needcomp,
            Self::CompRoot => &mut aff.af_comproot,
            Self::CompForbid => &mut aff.af_compforbid,
            Self::CompPermit => &mut aff.af_comppermit,
        }
    }

    /// The two compounding flags change how already-read `PFX` entries
    /// would have been processed, so declaring one late is worth a warning.
    fn warn_after_pfx(self) -> Option<&'static CStr> {
        match self {
            Self::CompForbid => Some(
                c"Defining COMPOUNDFORBIDFLAG after PFX item may give wrong results in %s line %d",
            ),
            Self::CompPermit => Some(
                c"Defining COMPOUNDPERMITFLAG after PFX item may give wrong results in %s line %d",
            ),
            _ => None,
        }
    }
}

/// Keywords declaring a single flag, in the order the C tried them.
const FLAG_RULES: &[(&[&CStr], FlagField)] = &[
    (&[c"RAR", c"RARE"], FlagField::Rare),
    (&[c"KEP", c"KEEPCASE"], FlagField::KeepCase),
    (&[c"BAD", c"FORBIDDENWORD"], FlagField::Bad),
    (&[c"NEEDAFFIX"], FlagField::NeedAffix),
    (&[c"CIRCUMFIX"], FlagField::Circumfix),
    (&[c"NOSUGGEST"], FlagField::NoSuggest),
    (&[c"NEEDCOMPOUND", c"ONLYINCOMPOUND"], FlagField::NeedComp),
    (&[c"COMPOUNDROOT"], FlagField::CompRoot),
    (&[c"COMPOUNDFORBIDFLAG"], FlagField::CompForbid),
    (&[c"COMPOUNDPERMITFLAG"], FlagField::CompPermit),
];

/// A keyword whose argument is a number, and where it is kept. Every one of
/// them is a `COMPOUND*` keyword, so the variants name only the tail:
/// `COMPOUNDWORDMAX`, `COMPOUNDMIN`, `COMPOUNDSYLMAX`.
#[derive(Copy, Clone)]
enum NumField {
    WordMax,
    Min,
    SylMax,
}

/// Keywords taking a number, with the complaint for a bad one.
const NUMBER_RULES: &[(&CStr, NumField, &CStr)] = &[
    (
        c"COMPOUNDWORDMAX",
        NumField::WordMax,
        c"Wrong COMPOUNDWORDMAX value in %s line %d: %s",
    ),
    (
        c"COMPOUNDMIN",
        NumField::Min,
        c"Wrong COMPOUNDMIN value in %s line %d: %s",
    ),
    (
        c"COMPOUNDSYLMAX",
        NumField::SylMax,
        c"Wrong COMPOUNDSYLMAX value in %s line %d: %s",
    ),
];

/// Bare keywords that only set a compound-checking option bit.
const COMPOPT_RULES: &[(&CStr, c_uint)] = &[
    (c"CHECKCOMPOUNDDUP", COMP_CHECKDUP),
    (c"CHECKCOMPOUNDREP", COMP_CHECKREP),
    (c"CHECKCOMPOUNDCASE", COMP_CHECKCASE),
    (c"CHECKCOMPOUNDTRIPLE", COMP_CHECKTRIPLE),
];

/// A bare keyword that turns one thing on.
#[derive(Copy, Clone)]
enum Toggle {
    NoBreak,
    NoSplitSugs,
    NoCompoundSugs,
    NoSugFile,
    PfxPostpone,
    IgnoreExtra,
}

const TOGGLE_RULES: &[(&CStr, Toggle)] = &[
    (c"NOBREAK", Toggle::NoBreak),
    (c"NOSPLITSUGS", Toggle::NoSplitSugs),
    (c"NOCOMPOUNDSUGS", Toggle::NoCompoundSugs),
    (c"NOSUGFILE", Toggle::NoSugFile),
    (c"PFXPOSTPONE", Toggle::PfxPostpone),
    (c"IGNOREEXTRA", Toggle::IgnoreExtra),
];

/// A keyword giving one of the case tables, kept until the file is done.
#[derive(Copy, Clone)]
enum CaseTable {
    Fol,
    Low,
    Upp,
}

const CASE_RULES: &[(&CStr, CaseTable)] = &[
    (c"FOL", CaseTable::Fol),
    (c"LOW", CaseTable::Low),
    (c"UPP", CaseTable::Upp),
];

/// Does this line start with `rulename` and carry the right number of
/// items? Trailing items are allowed when the first of them is a comment.
fn is_aff_rule(items: &[&CStr], rulename: &CStr, mincount: usize) -> bool {
    items[0] == rulename
        && (items.len() == mincount
            || (items.len() > mincount && first_byte(items[mincount]) == b'#'))
}

/// The item's first byte, or `NUL` when it is empty.
pub(super) fn first_byte(item: &CStr) -> uint8_t {
    item.to_bytes().first().copied().unwrap_or(NUL as uint8_t)
}

/// Keywords whose argument is free text kept for `:spellinfo`.
fn is_info_keyword(name: &[uint8_t]) -> bool {
    [
        &b"NAME"[..],
        b"HOME",
        b"VERSION",
        b"AUTHOR",
        b"EMAIL",
        b"COPYRIGHT",
    ]
    .contains(&name)
}

/// Read a `.aff` file and return what it describes, or `None` if it could
/// not be opened.
pub(super) fn spell_read_aff(spin: &mut SpellInfo, fname: &CStr) -> Option<AffFile> {
    let Ok(file) = File::open(OsStr::from_bytes(fname.to_bytes())) else {
        let fname = msg_cstr(fname);
        semsg!("E484: Can't open file {fname}");
        return None;
    };
    let mut fd = Fgets::new(file);
    let name = fname.to_string_lossy();
    spell_message_fmt(&*spin, format_args!("Reading affix file {name}..."));

    let mut st = AffState {
        aff_todo: 0,
        cur_aff: None,
        did_postpone_prefix: false,
        found_map: false,
        compmax: 0,
        compminlen: 0,
        compsylmax: 0,
        compoptions: 0,
        compflags: None,
        midword: None,
        syllable: None,
        sofofrom: None,
        sofoto: None,
        low: false,
        fol: false,
        upp: false,
        // Only take these from the first file that has them.
        do_rep: spin.si_rep.is_empty(),
        do_repsal: spin.si_repsal.is_empty(),
        do_sal: spin.si_sal.is_empty(),
        do_mapline: spin.si_map.is_empty(),
    };

    spin.si_arena.charge(AFF_FILE_BYTES, true);
    let mut aff = AffFile::new();

    let mut rline: Vec<uint8_t> = Vec::new();
    // The line as the splitter takes it: up to its first NUL, converted
    // when the file's encoding is not the editor's, and terminated.
    let mut line: Vec<uint8_t> = Vec::new();
    let mut lnum: c_int = 0;

    while fd.line_capped(&mut rline, MAXLINELEN as usize) && !got_int.get() {
        line_breakcheck();
        lnum += 1;
        let read = cstr::until_nul(&rline);
        if read.first() == Some(&b'#') {
            continue;
        }

        line.clear();
        if spin.si_conv.vc_type != CONV_NONE {
            let Some(converted) = spin.si_conv.convert(read) else {
                let fname = msg_cstr(fname);
                let rline = msg_bytes(read);
                smsg!(
                    0,
                    "Conversion failure for word in {fname} line {}: {rline}",
                    lnum
                );
                continue;
            };
            line.extend(converted.iter().take_while(|&&b| b != 0));
        } else {
            line.extend_from_slice(read);
        }
        line.push(NUL as uint8_t);

        let items = split_items(&mut line);
        if items.is_empty() {
            continue;
        }
        if !handle_line(spin, &mut aff, &mut st, &items, fname, lnum) {
            break;
        }
    }

    finish_aff(spin, &mut aff, &mut st, fname);
    Some(aff)
}

/// Split a line into white-space separated items, in place.
///
/// An informational keyword's argument is everything to the end of the
/// line, spaces and all, so `NAME Some Dictionary` is two items.
///
/// The items are spans of `line` with a terminator written after each,
/// which is what makes them `CStr`s rather than plain slices: nearly every
/// consumer hands one straight on to a callee that takes a C string --
/// `save_str`, `snprintf`, `atoi`, `spell_casefold` -- and a copy per item
/// would be the only other way to say that.
fn split_items(line: &mut [uint8_t]) -> Vec<&CStr> {
    // Each item as its first byte and the index of its terminator.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    loop {
        while line[at] != NUL as uint8_t && line[at] <= b' ' {
            at += 1;
        }
        if line[at] == NUL as uint8_t || spans.len() == MAXITEMCNT {
            break;
        }
        let start = at;

        if spans.len() == 1 && is_info_keyword(&line[spans[0].0..spans[0].1]) {
            // Take the rest of the line, stopping only at a control
            // character that is not a tab.
            while line[at] >= b' ' || line[at] as c_int == TAB {
                at += 1;
            }
        } else {
            while line[at] > b' ' {
                at += 1;
            }
        }
        spans.push((start, at));
        if line[at] == NUL as uint8_t {
            break;
        }
        line[at] = NUL as uint8_t;
        at += 1;
    }

    let line: &[uint8_t] = line;
    spans
        .iter()
        .map(|&(start, end)| {
            CStr::from_bytes_with_nul(&line[start..=end])
                .expect("every walk above stopped at a terminator")
        })
        .collect()
}

/// Handle one line. Returns false to stop reading the file.
fn handle_line(
    spin: &mut SpellInfo,
    aff: &mut AffFile,
    st: &mut AffState,
    items: &[&CStr],
    fname: &CStr,
    lnum: c_int,
) -> bool {
    // SET must come before anything that could need converting.
    if is_aff_rule(items, c"SET", 2) && aff.af_enc.is_none() {
        let enc = aff.af_enc.insert(canonize(items[1].to_bytes()));
        if spin.si_ascii == 0
            && p_enc(|value| spin.si_conv.setup(Some(enc.as_cstr()), Some(value))).is_err()
        {
            let to = p_enc(|value| value.to_bytes().to_vec());
            let (fname, af_enc, arg2) = (msg_cstr(fname), msg_bytes(enc), msg_bytes(&to));
            smsg!(
                0,
                "Conversion in {fname} not supported: from {af_enc} to {arg2}"
            );
        }
        spin.si_conv.vc_fail = true;
        return true;
    }

    if is_aff_rule(items, c"FLAG", 2) && aff.af_flagtype == AFT_CHAR {
        handle_flag_type(aff, items, fname, lnum);
        return true;
    }

    if is_info_keyword(items[0].to_bytes()) && items.len() > 1 {
        append_info(spin, items);
        return true;
    }

    if is_aff_rule(items, c"MIDWORD", 2) && st.midword.is_none() {
        st.midword = Some(saved(spin, items[1]));
        return true;
    }

    // TRY is Hunspell's suggestion alphabet; nvim does not use it.
    if is_aff_rule(items, c"TRY", 2) {
        return true;
    }

    for (names, field) in FLAG_RULES {
        if !names.iter().any(|n| is_aff_rule(items, n, 2)) {
            continue;
        }
        // A second declaration is not this arm's business; it falls
        // through and is reported as a duplicate.
        if *field.slot(aff) != 0 {
            break;
        }
        let flag = affitem2flag(aff.af_flagtype, items[1].to_bytes(), fname, lnum);
        *field.slot(aff) = flag;
        if let Some(warning) = field.warn_after_pfx()
            && aff.af_pref.len() > 0
        {
            let fname = msg_cstr(fname);
            let _: bool = report_msg(0, || tr_c!(warning, fname, lnum));
        }
        return true;
    }

    if is_aff_rule(items, c"COMPOUNDFLAG", 2) && st.compflags.is_none() {
        // One flag becomes a pattern matching one or more of it.
        let mut pattern = items[1].to_bytes().to_vec();
        pattern.push(b'+');
        spin.si_arena.charge_str(&pattern);
        st.compflags = Some(pattern);
        return true;
    }

    if is_aff_rule(items, c"COMPOUNDRULES", 2) {
        if item_number(items[1]) == 0 {
            let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
            smsg!(
                0,
                "Wrong COMPOUNDRULES value in {fname} line {}: {arg2}",
                lnum
            );
        }
        return true;
    }

    if is_aff_rule(items, c"COMPOUNDRULE", 2) {
        // A rule that is only digits is the count line, unless a
        // pattern has already been started.
        let all_digits = items[1].to_bytes().iter().all(u8::is_ascii_digit);
        if st.compflags.is_some() || !all_digits {
            let mut pattern = Vec::new();
            if let Some(old) = &st.compflags {
                pattern.extend_from_slice(old);
                pattern.push(b'/');
            }
            pattern.extend_from_slice(items[1].to_bytes());
            spin.si_arena.charge_str(&pattern);
            st.compflags = Some(pattern);
        }
        return true;
    }

    for (name, field, complaint) in NUMBER_RULES {
        if !is_aff_rule(items, name, 2) {
            continue;
        }
        let slot = match field {
            NumField::WordMax => &mut st.compmax,
            NumField::Min => &mut st.compminlen,
            NumField::SylMax => &mut st.compsylmax,
        };
        if *slot != 0 {
            break;
        }
        *slot = item_number(items[1]);
        if *slot == 0 {
            let (fname, item) = (msg_cstr(fname), msg_cstr(items[1]));
            let _: bool = report_msg(0, || tr_c!(complaint, fname, lnum, item));
        }
        return true;
    }

    for (name, bit) in COMPOPT_RULES {
        if is_aff_rule(items, name, 1) {
            st.compoptions |= *bit as c_int;
            return true;
        }
    }

    // The two-item form is the count line; the three-item form is a
    // pattern pair.
    if is_aff_rule(items, c"CHECKCOMPOUNDPATTERN", 2) {
        if item_number(items[1]) == 0 {
            let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
            smsg!(
                0,
                "Wrong CHECKCOMPOUNDPATTERN value in {fname} line {}: {arg2}",
                lnum
            );
        }
        return true;
    }
    if is_aff_rule(items, c"CHECKCOMPOUNDPATTERN", 3) {
        add_comppat(spin, items);
        return true;
    }

    if is_aff_rule(items, c"SYLLABLE", 2) && st.syllable.is_none() {
        st.syllable = Some(saved(spin, items[1]));
        return true;
    }

    for (name, toggle) in TOGGLE_RULES {
        if !is_aff_rule(items, name, 1) {
            continue;
        }
        match toggle {
            Toggle::NoBreak => spin.si_nobreak = 1,
            Toggle::NoSplitSugs => spin.si_nosplitsugs = 1,
            Toggle::NoCompoundSugs => spin.si_nocompoundsugs = 1,
            Toggle::NoSugFile => spin.si_nosugfile = 1,
            Toggle::PfxPostpone => aff.af_pfxpostpone = 1,
            Toggle::IgnoreExtra => aff.af_ignoreextra = true,
        }
        return true;
    }

    let is_affix = items[0] == c"PFX" || items[0] == c"SFX";
    if is_affix && st.aff_todo == 0 && items.len() >= 4 {
        return handle_affix_header(spin, aff, st, items, fname, lnum);
    }
    if is_affix
        && st.aff_todo > 0
        && st
            .cur_aff
            .is_some_and(|(kind, at)| aff.affixes(kind).get(at).ah_key == items[1].to_bytes())
        && items.len() >= 5
    {
        handle_affix_entry(spin, aff, st, items, fname, lnum);
        return true;
    }

    for (name, table) in CASE_RULES {
        if !is_aff_rule(items, name, 2) {
            continue;
        }
        let slot = match table {
            CaseTable::Fol => &mut st.fol,
            CaseTable::Low => &mut st.low,
            CaseTable::Upp => &mut st.upp,
        };
        if *slot {
            break;
        }
        *slot = true;
        return true;
    }

    // The two-item form of REP/REPSAL is the count line.
    if is_aff_rule(items, c"REP", 2) || is_aff_rule(items, c"REPSAL", 2) {
        if !first_byte(items[1]).is_ascii_digit() {
            let fname = msg_cstr(fname);
            smsg!(0, "Expected REP(SAL) count in {fname} line {}", lnum);
        }
        return true;
    }
    let is_rep = items[0] == c"REP" || items[0] == c"REPSAL";
    if is_rep && items.len() >= 3 {
        add_rep_entry(spin, st, items, fname, lnum);
        return true;
    }

    if is_aff_rule(items, c"MAP", 2) {
        handle_map(spin, st, items, fname, lnum);
        return true;
    }

    if is_aff_rule(items, c"SAL", 3) {
        if st.do_sal {
            handle_sal(spin, items);
        }
        return true;
    }

    if is_aff_rule(items, c"SOFOFROM", 2) && st.sofofrom.is_none() {
        st.sofofrom = Some(saved(spin, items[1]));
        return true;
    }
    if is_aff_rule(items, c"SOFOTO", 2) && st.sofoto.is_none() {
        st.sofoto = Some(saved(spin, items[1]));
        return true;
    }

    if items[0] == c"COMMON" {
        for item in &items[1..] {
            let word = item.to_bytes();
            if spin.si_commonwords.find(word).is_none() {
                spin.si_commonwords.insert(word.into());
            }
        }
        return true;
    }

    let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[0]));
    smsg!(
        0,
        "Unrecognized or duplicate item in {fname} line {}: {arg2}",
        lnum
    );
    true
}

/// A copy of `item`, charged to the arena as upstream's `getroom_save`.
fn saved(spin: &mut SpellInfo, item: &CStr) -> Vec<u8> {
    let bytes = item.to_bytes();
    spin.si_arena.charge_str(bytes);
    bytes.to_vec()
}

/// An item read as a number, which is `atoi`: leading blanks and a sign,
/// then digits, and zero for anything else.
pub(super) fn item_number(item: &CStr) -> c_int {
    atoi(item.to_bytes())
}

/// `FLAG`: how flags are spelled in the rest of the file.
fn handle_flag_type(aff: &mut AffFile, items: &[&CStr], fname: &CStr, lnum: c_int) {
    if items[1] == c"long" {
        aff.af_flagtype = AFT_LONG;
    } else if items[1] == c"num" {
        aff.af_flagtype = AFT_NUM;
    } else if items[1] == c"caplong" {
        aff.af_flagtype = AFT_CAPLONG;
    } else {
        let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
        smsg!(0, "Invalid value for FLAG in {fname} line {}: {arg2}", lnum);
    }
    // Anything already read used the old spelling, so it would be
    // interpreted wrongly.
    let used = aff.af_rare != 0
        || aff.af_keepcase != 0
        || aff.af_bad != 0
        || aff.af_needaffix != 0
        || aff.af_circumfix != 0
        || aff.af_needcomp != 0
        || aff.af_comproot != 0
        || aff.af_nosuggest != 0
        || aff.af_suff.len() > 0
        || aff.af_pref.len() > 0;
    if used {
        let (fname, arg2) = (msg_cstr(fname), msg_cstr(items[1]));
        smsg!(0, "FLAG after using flags in {fname} line {}: {arg2}", lnum);
    }
}

/// Apply what the file collected, checking it against what earlier `.aff`
/// files of the same run already set.
fn finish_aff(spin: &mut SpellInfo, aff: &mut AffFile, st: &mut AffState, fname: &CStr) {
    // The case tables are only used to decide whether the word characters
    // need rebuilding; their contents are not kept.
    if (st.fol || st.low || st.upp) && spin.si_clear_chartab != 0 {
        init_spell_chartab();
        spin.si_clear_chartab = 0;
    }

    if st.compmax != 0 {
        aff_check_number(spin.si_compmax, st.compmax, c"COMPOUNDWORDMAX");
        spin.si_compmax = st.compmax;
    }
    if st.compminlen != 0 {
        aff_check_number(spin.si_compminlen, st.compminlen, c"COMPOUNDMIN");
        spin.si_compminlen = st.compminlen;
    }
    if st.compsylmax != 0 {
        if st.syllable.is_none() {
            let fmt = gettext(c"COMPOUNDSYLMAX used without SYLLABLE");
            let arg0 = msg_cstr(fmt);
            smsg!(0, "{arg0}");
        }
        aff_check_number(spin.si_compsylmax, st.compsylmax, c"COMPOUNDSYLMAX");
        spin.si_compsylmax = st.compsylmax;
    }
    if st.compoptions != 0 {
        aff_check_number(spin.si_compoptions, st.compoptions, c"COMPOUND options");
        spin.si_compoptions |= st.compoptions;
    }
    if let Some(compflags) = &st.compflags {
        process_compflags(spin, aff, compflags);
    }

    // Prefix ids count up and compound ids down; meeting means one kind
    // ran out of room.
    if spin.si_newcomp_id < spin.si_newpref_id {
        let complaint = if spin.si_newcomp_id == 127 || spin.si_newcomp_id == 255 {
            c"Too many postponed prefixes"
        } else if spin.si_newpref_id == 0 || spin.si_newpref_id == 127 {
            c"Too many compound flags"
        } else {
            c"Too many postponed prefixes and/or compound flags"
        };
        msg(gettext(complaint), 0);
    }

    if let Some(syllable) = st.syllable.take() {
        aff_check_string(spin.si_syllable.as_deref(), &syllable, c"SYLLABLE");
        spin.si_syllable = Some(syllable);
    }

    if st.sofofrom.is_some() || st.sofoto.is_some() {
        if st.sofofrom.is_none() || st.sofoto.is_none() {
            let which = if st.sofofrom.is_none() { "FROM" } else { "TO" };
            let fname = msg_cstr(fname);
            smsg!(0, "Missing SOFO{which} line in {fname}");
        } else if !spin.si_sal.is_empty() {
            // SAL rules and a SOFO pair are two ways to do the same
            // thing; taking both would be ambiguous.
            let fname = msg_cstr(fname);
            smsg!(0, "Both SAL and SOFO lines in {fname}");
        } else if let (Some(from), Some(to)) = (st.sofofrom.take(), st.sofoto.take()) {
            aff_check_string(spin.si_sofofr.as_deref(), &from, c"SOFOFROM");
            aff_check_string(spin.si_sofoto.as_deref(), &to, c"SOFOTO");
            spin.si_sofofr = Some(from);
            spin.si_sofoto = Some(to);
        }
    }

    if let Some(midword) = st.midword.take() {
        aff_check_string(spin.si_midword.as_deref(), &midword, c"MIDWORD");
        spin.si_midword = Some(midword);
    }
}

/// Warn when two `.aff` files of one run disagree about a number.
fn aff_check_number(spinval: c_int, affval: c_int, name: &CStr) {
    if spinval != 0 && spinval != affval {
        let name = msg_cstr(name);
        smsg!(
            0,
            "{name} value differs from what is used in another .aff file"
        );
    }
}

/// Warn when two `.aff` files of one run disagree about a string.
fn aff_check_string(spinval: Option<&[u8]>, affval: &[u8], name: &CStr) {
    if spinval.is_some_and(|spinval| spinval != affval) {
        let name = msg_cstr(name);
        smsg!(
            0,
            "{name} value differs from what is used in another .aff file"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Split `text` the way [`spell_read_aff`] splits one line of a `.aff`
    /// file, and answer the items as bytes.
    fn items_of(text: &str) -> Vec<Vec<uint8_t>> {
        let mut line: Vec<uint8_t> = text.bytes().collect();
        line.push(NUL as uint8_t);
        split_items(&mut line)
            .iter()
            .map(|item| item.to_bytes().to_vec())
            .collect()
    }

    fn strs(items: &[Vec<uint8_t>]) -> Vec<&str> {
        items
            .iter()
            .map(|item| core::str::from_utf8(item).expect("ASCII test input"))
            .collect()
    }

    #[test]
    fn items_are_separated_by_runs_of_white_space() {
        let items = items_of("SET UTF-8");
        assert_eq!(strs(&items), ["SET", "UTF-8"]);
        let items = items_of("  PFX \t A\tY   1  ");
        assert_eq!(strs(&items), ["PFX", "A", "Y", "1"]);
    }

    #[test]
    fn a_blank_line_has_no_items() {
        assert_eq!(items_of("").len(), 0);
        assert_eq!(items_of("   \t  ").len(), 0);
    }

    /// `#` is not comment syntax to the splitter: it is an ordinary item,
    /// and it is the *keyword tests* that let a rule end with one.
    #[test]
    fn a_hash_is_an_item_like_any_other() {
        let items = items_of("MIDWORD ' # why it is here");
        assert_eq!(
            strs(&items),
            ["MIDWORD", "'", "#", "why", "it", "is", "here"]
        );
    }

    /// An informational keyword's argument is the rest of the line, spaces
    /// and all -- **trailing ones included**, because the walk stops at the
    /// terminator rather than at the last printing character.
    #[test]
    fn an_info_keyword_takes_the_rest_of_the_line() {
        let items = items_of("NAME  Some Dictionary  ");
        assert_eq!(strs(&items), ["NAME", "Some Dictionary  "]);
        for keyword in ["HOME", "VERSION", "AUTHOR", "EMAIL", "COPYRIGHT"] {
            let items = items_of(&format!("{keyword} a b"));
            assert_eq!(strs(&items), [keyword, "a b"]);
        }
        // Only the *second* item swallows the rest, and only for these
        // keywords.
        assert_eq!(strs(&items_of("NAME")), ["NAME"]);
        assert_eq!(strs(&items_of("TRY a b")), ["TRY", "a", "b"]);
    }

    /// The rest-of-the-line walk stops at a control character that is not a
    /// tab, and the ordinary splitting picks up again after it.
    #[test]
    fn a_control_character_ends_an_info_keywords_argument() {
        let items = items_of("NAME Some\u{1}Dictionary");
        assert_eq!(strs(&items), ["NAME", "Some", "Dictionary"]);
        let items = items_of("NAME Some\tDictionary");
        assert_eq!(strs(&items), ["NAME", "Some\tDictionary"]);
    }

    /// Past `MAXITEMCNT` the rest of the line is dropped -- and the last
    /// item kept is a whole one, because its terminator is written before
    /// the count is looked at again.
    #[test]
    fn a_line_stops_at_the_item_limit() {
        let text: Vec<String> = (0..MAXITEMCNT + 5).map(|at| format!("i{at}")).collect();
        let items = items_of(&text.join(" "));
        assert_eq!(items.len(), MAXITEMCNT);
        assert_eq!(items[0], b"i0");
        assert_eq!(
            items[MAXITEMCNT - 1],
            format!("i{}", MAXITEMCNT - 1).as_bytes()
        );
    }

    /// A rule is its keyword, a count of items, and one allowance: a
    /// trailing comment does not make the line a different rule.
    #[test]
    fn a_rule_is_its_keyword_and_a_count_of_items() {
        let items = [c"SET", c"UTF-8", c"# and a comment"];
        assert!(is_aff_rule(&items[..2], c"SET", 2));
        assert!(!is_aff_rule(&items[..2], c"SET", 3));
        assert!(!is_aff_rule(&items[..2], c"FLAG", 2));
        // A trailing comment does not make the line a different rule.
        assert!(is_aff_rule(&items, c"SET", 2));
        assert!(!is_aff_rule(&items[..1], c"SET", 2));
    }
}
