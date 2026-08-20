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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{semsg_c, smsg_c};
use core::ffi::{CStr, c_char, c_int, c_uint};

use crate::charset::skipdigits;
use crate::fileio::vim_fgets;
use crate::hashtab::{hash_add, hash_find, hash_init, hash_removed};
use crate::main::{IObuff, e_notopen, got_int, p_enc};
use crate::mbyte::{convert_setup, enc_canonize, string_convert};
use crate::memory::{xfree, xstrdup};
use crate::message::msg;
use crate::os::cshim::{__ctype_b_loc, gettext};
use crate::os::fs::os_fopen;
use crate::os::input::line_breakcheck;
use crate::spell::init_spell_chartab;
use crate::strings::vim_snprintf;
use crate::types::{CONV_NONE, IOSIZE, NUL, size_t, uint8_t};
use ::libc::{atoi, fclose, strcat, strcmp, strcpy, strlen};

use super::affix::{handle_affix_entry, handle_affix_header};
use super::flags::{affitem2flag, process_compflags};
use super::tables::{add_comppat, add_rep_entry, append_info, handle_map, handle_sal};
use super::{
    _ISdigit, AFT_CAPLONG, AFT_CHAR, AFT_LONG, AFT_NUM, COMP_CHECKCASE, COMP_CHECKDUP,
    COMP_CHECKREP, COMP_CHECKTRIPLE, FAIL, MAXLINELEN, TAB, afffile_T, affheader_T, spell_message,
    spellinfo_T,
};

/// The most items one `.aff` line is split into; the rest are ignored.
pub const MAXITEMCNT: usize = 30;

/// What one `.aff` file accumulates while its lines are read.
///
/// Most of this is not applied until the end: several keywords have to be
/// checked against what an *earlier* `.aff` file of the same run already
/// set, which cannot happen until the file is known to be complete.
pub struct AffState {
    /// Entries still expected in the affix block being read.
    pub aff_todo: c_int,
    pub cur_aff: *mut affheader_T,
    /// Whether any entry of the current prefix block actually went into the
    /// prefix tree; if none did, its id is handed back.
    pub did_postpone_prefix: bool,
    /// The first `MAP` line is a count, not a mapping.
    pub found_map: bool,

    pub compmax: c_int,
    pub compminlen: c_int,
    pub compsylmax: c_int,
    pub compoptions: c_int,
    pub compflags: *mut c_char,

    pub midword: *mut c_char,
    pub syllable: *mut c_char,
    pub sofofrom: *mut c_char,
    pub sofoto: *mut c_char,
    pub low: *mut c_char,
    pub fol: *mut c_char,
    pub upp: *mut c_char,

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
    /// # Safety
    ///
    /// `aff` must be live.
    unsafe fn slot(self, aff: *mut afffile_T) -> *mut c_uint {
        // SAFETY: the caller promises `aff`.
        unsafe {
            match self {
                Self::Rare => &raw mut (*aff).af_rare,
                Self::KeepCase => &raw mut (*aff).af_keepcase,
                Self::Bad => &raw mut (*aff).af_bad,
                Self::NeedAffix => &raw mut (*aff).af_needaffix,
                Self::Circumfix => &raw mut (*aff).af_circumfix,
                Self::NoSuggest => &raw mut (*aff).af_nosuggest,
                Self::NeedComp => &raw mut (*aff).af_needcomp,
                Self::CompRoot => &raw mut (*aff).af_comproot,
                Self::CompForbid => &raw mut (*aff).af_compforbid,
                Self::CompPermit => &raw mut (*aff).af_comppermit,
            }
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
///
/// # Safety
///
/// `items` must hold live NUL-terminated strings.
unsafe fn is_aff_rule(items: &[*mut c_char], rulename: &CStr, mincount: usize) -> bool {
    // SAFETY: the caller promises the items.
    unsafe {
        strcmp(items[0], rulename.as_ptr()) == 0
            && (items.len() == mincount
                || (items.len() > mincount && *items[mincount] as c_int == b'#' as c_int))
    }
}

/// Keywords whose argument is free text kept for `:spellinfo`.
///
/// # Safety
///
/// `s` must be NUL-terminated.
unsafe fn spell_info_item(s: *mut c_char) -> bool {
    // SAFETY: the caller promises the string.
    unsafe {
        [
            c"NAME",
            c"HOME",
            c"VERSION",
            c"AUTHOR",
            c"EMAIL",
            c"COPYRIGHT",
        ]
        .into_iter()
        .any(|name| strcmp(s, name.as_ptr()) == 0)
    }
}

/// Read a `.aff` file and return what it describes, or null if it could not
/// be opened.
///
/// # Safety
///
/// `fname` must be a NUL-terminated path.
pub unsafe fn spell_read_aff(spin: *mut spellinfo_T, fname: *mut c_char) -> *mut afffile_T {
    // SAFETY: the caller promises the path; `rline` is MAXLINELEN, the
    // bound `vim_fgets` is given.
    unsafe {
        let fd = os_fopen(fname, c"r".as_ptr());
        if fd.is_null() {
            semsg_c!(gettext((&raw const e_notopen).cast()), fname);
            return core::ptr::null_mut();
        }
        vim_snprintf(
            IObuff.ptr().cast::<c_char>(),
            IOSIZE as size_t,
            gettext(c"Reading affix file %s...".as_ptr()),
            fname,
        );
        spell_message(&*spin, IObuff.ptr().cast::<c_char>());

        let mut st = AffState {
            aff_todo: 0,
            cur_aff: core::ptr::null_mut(),
            did_postpone_prefix: false,
            found_map: false,
            compmax: 0,
            compminlen: 0,
            compsylmax: 0,
            compoptions: 0,
            compflags: core::ptr::null_mut(),
            midword: core::ptr::null_mut(),
            syllable: core::ptr::null_mut(),
            sofofrom: core::ptr::null_mut(),
            sofoto: core::ptr::null_mut(),
            low: core::ptr::null_mut(),
            fol: core::ptr::null_mut(),
            upp: core::ptr::null_mut(),
            // Only take these from the first file that has them.
            do_rep: (*spin).si_rep.ga_len <= 0,
            do_repsal: (*spin).si_repsal.ga_len <= 0,
            do_sal: (*spin).si_sal.ga_len <= 0,
            do_mapline: (*spin).si_map.ga_len <= 0,
        };

        let aff = (*spin).si_arena.alloc::<afffile_T>();
        hash_init(&raw mut (*aff).af_pref);
        hash_init(&raw mut (*aff).af_suff);
        hash_init(&raw mut (*aff).af_comp);

        let mut rline: [c_char; MAXLINELEN as usize] = [0; MAXLINELEN as usize];
        let mut items: [*mut c_char; MAXITEMCNT] = [core::ptr::null_mut(); MAXITEMCNT];
        let mut pc: *mut c_char = core::ptr::null_mut();
        let mut lnum: c_int = 0;

        while !vim_fgets(rline.as_mut_ptr(), MAXLINELEN, fd) && !got_int.get() {
            line_breakcheck();
            lnum += 1;
            if rline[0] as c_int == b'#' as c_int {
                continue;
            }

            xfree(pc.cast());
            pc = core::ptr::null_mut();
            let line = if (*spin).si_conv.vc_type != CONV_NONE {
                pc = string_convert(
                    &raw mut (*spin).si_conv,
                    rline.as_mut_ptr(),
                    core::ptr::null_mut(),
                );
                if pc.is_null() {
                    smsg_c!(
                        0,
                        gettext(c"Conversion failure for word in %s line %d: %s".as_ptr()),
                        fname,
                        lnum,
                        rline.as_mut_ptr(),
                    );
                    continue;
                }
                pc
            } else {
                rline.as_mut_ptr()
            };

            let itemcnt = split_items(line, &mut items);
            if itemcnt == 0 {
                continue;
            }
            if !handle_line(spin, aff, &mut st, &items[..itemcnt], fname, lnum) {
                break;
            }
        }

        finish_aff(spin, aff, &mut st, fname);
        xfree(pc.cast());
        fclose(fd);
        aff
    }
}

/// Split a line into white-space separated items, in place.
///
/// An informational keyword's argument is everything to the end of the
/// line, spaces and all, so `NAME Some Dictionary` is two items.
///
/// # Safety
///
/// `line` must be NUL-terminated and writable.
unsafe fn split_items(line: *mut c_char, items: &mut [*mut c_char; MAXITEMCNT]) -> usize {
    // SAFETY: the caller promises the line; the walk stops at its NUL.
    unsafe {
        let mut itemcnt = 0;
        let mut p = line;
        loop {
            while *p as c_int != NUL && *p as uint8_t as c_int <= b' ' as c_int {
                p = p.add(1);
            }
            if *p as c_int == NUL || itemcnt == MAXITEMCNT {
                break;
            }
            items[itemcnt] = p;
            itemcnt += 1;

            if itemcnt == 2 && spell_info_item(items[0]) {
                // Take the rest of the line, stopping only at a control
                // character that is not a tab.
                while *p as uint8_t as c_int >= b' ' as c_int || *p as c_int == TAB {
                    p = p.add(1);
                }
            } else {
                while *p as uint8_t as c_int > b' ' as c_int {
                    p = p.add(1);
                }
            }
            if *p as c_int == NUL {
                break;
            }
            *p = NUL as c_char;
            p = p.add(1);
        }
        itemcnt
    }
}

/// Handle one line. Returns false to stop reading the file.
///
/// # Safety
///
/// `items` must hold live NUL-terminated strings, and `aff` and `spin` be
/// live.
unsafe fn handle_line(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    st: &mut AffState,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) -> bool {
    // SAFETY: the caller promises the items and the two structures.
    unsafe {
        // SET must come before anything that could need converting.
        if is_aff_rule(items, c"SET", 2) && (*aff).af_enc.is_null() {
            (*aff).af_enc = enc_canonize(items[1]);
            if (*spin).si_ascii == 0
                && convert_setup(&raw mut (*spin).si_conv, (*aff).af_enc, p_enc.get()) == FAIL
            {
                smsg_c!(
                    0,
                    gettext(c"Conversion in %s not supported: from %s to %s".as_ptr()),
                    fname,
                    (*aff).af_enc,
                    p_enc.get(),
                );
            }
            (*spin).si_conv.vc_fail = true;
            return true;
        }

        if is_aff_rule(items, c"FLAG", 2) && (*aff).af_flagtype == AFT_CHAR {
            handle_flag_type(aff, items, fname, lnum);
            return true;
        }

        if spell_info_item(items[0]) && items.len() > 1 {
            append_info(spin, items);
            return true;
        }

        if is_aff_rule(items, c"MIDWORD", 2) && st.midword.is_null() {
            st.midword = (*spin).si_arena.save_str(items[1]);
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
            let slot = field.slot(aff);
            // A second declaration is not this arm's business; it falls
            // through and is reported as a duplicate.
            if *slot != 0 {
                break;
            }
            *slot = affitem2flag((*aff).af_flagtype, items[1], fname, lnum);
            if let Some(warning) = field.warn_after_pfx()
                && (*aff).af_pref.ht_used > 0
            {
                smsg_c!(0, gettext(warning.as_ptr()), fname, lnum);
            }
            return true;
        }

        if is_aff_rule(items, c"COMPOUNDFLAG", 2) && st.compflags.is_null() {
            // One flag becomes a pattern matching one or more of it.
            let p = (*spin).si_arena.alloc_bytes(strlen(items[1]) + 2, false);
            strcpy(p, items[1]);
            strcat(p, c"+".as_ptr());
            st.compflags = p;
            return true;
        }

        if is_aff_rule(items, c"COMPOUNDRULES", 2) {
            if atoi(items[1]) == 0 {
                smsg_c!(
                    0,
                    gettext(c"Wrong COMPOUNDRULES value in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    items[1],
                );
            }
            return true;
        }

        if is_aff_rule(items, c"COMPOUNDRULE", 2) {
            // A rule that is only digits is the count line, unless a
            // pattern has already been started.
            if !st.compflags.is_null() || *skipdigits(items[1]) as c_int != NUL {
                let mut len = strlen(items[1]) + 1;
                if !st.compflags.is_null() {
                    len += strlen(st.compflags) + 1;
                }
                let p = (*spin).si_arena.alloc_bytes(len, false);
                if !st.compflags.is_null() {
                    strcpy(p, st.compflags);
                    strcat(p, c"/".as_ptr());
                }
                strcat(p, items[1]);
                st.compflags = p;
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
            *slot = atoi(items[1]);
            if *slot == 0 {
                smsg_c!(0, gettext(complaint.as_ptr()), fname, lnum, items[1]);
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
            if atoi(items[1]) == 0 {
                smsg_c!(
                    0,
                    gettext(c"Wrong CHECKCOMPOUNDPATTERN value in %s line %d: %s".as_ptr()),
                    fname,
                    lnum,
                    items[1],
                );
            }
            return true;
        }
        if is_aff_rule(items, c"CHECKCOMPOUNDPATTERN", 3) {
            add_comppat(spin, items);
            return true;
        }

        if is_aff_rule(items, c"SYLLABLE", 2) && st.syllable.is_null() {
            st.syllable = (*spin).si_arena.save_str(items[1]);
            return true;
        }

        for (name, toggle) in TOGGLE_RULES {
            if !is_aff_rule(items, name, 1) {
                continue;
            }
            match toggle {
                Toggle::NoBreak => (*spin).si_nobreak = 1,
                Toggle::NoSplitSugs => (*spin).si_nosplitsugs = 1,
                Toggle::NoCompoundSugs => (*spin).si_nocompoundsugs = 1,
                Toggle::NoSugFile => (*spin).si_nosugfile = 1,
                Toggle::PfxPostpone => (*aff).af_pfxpostpone = 1,
                Toggle::IgnoreExtra => (*aff).af_ignoreextra = true,
            }
            return true;
        }

        let is_affix =
            strcmp(items[0], c"PFX".as_ptr()) == 0 || strcmp(items[0], c"SFX".as_ptr()) == 0;
        if is_affix && st.aff_todo == 0 && items.len() >= 4 {
            return handle_affix_header(spin, aff, st, items, fname, lnum);
        }
        if is_affix
            && st.aff_todo > 0
            && strcmp(affheader_T::key(st.cur_aff), items[1]) == 0
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
            if !slot.is_null() {
                break;
            }
            *slot = xstrdup(items[1]);
            return true;
        }

        // The two-item form of REP/REPSAL is the count line.
        if is_aff_rule(items, c"REP", 2) || is_aff_rule(items, c"REPSAL", 2) {
            if !is_digit_byte(*items[1]) {
                smsg_c!(
                    0,
                    gettext(c"Expected REP(SAL) count in %s line %d".as_ptr()),
                    fname,
                    lnum,
                );
            }
            return true;
        }
        let is_rep =
            strcmp(items[0], c"REP".as_ptr()) == 0 || strcmp(items[0], c"REPSAL".as_ptr()) == 0;
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

        if is_aff_rule(items, c"SOFOFROM", 2) && st.sofofrom.is_null() {
            st.sofofrom = (*spin).si_arena.save_str(items[1]);
            return true;
        }
        if is_aff_rule(items, c"SOFOTO", 2) && st.sofoto.is_null() {
            st.sofoto = (*spin).si_arena.save_str(items[1]);
            return true;
        }

        if strcmp(items[0], c"COMMON".as_ptr()) == 0 {
            for &item in &items[1..] {
                let hi = hash_find(&raw mut (*spin).si_commonwords, item);
                if (*hi).hi_key.is_null()
                    || (*hi).hi_key == (&raw const hash_removed).cast_mut().cast()
                {
                    hash_add(&raw mut (*spin).si_commonwords, xstrdup(item));
                }
            }
            return true;
        }

        smsg_c!(
            0,
            gettext(c"Unrecognized or duplicate item in %s line %d: %s".as_ptr()),
            fname,
            lnum,
            items[0],
        );
        true
    }
}

/// Is this byte a digit, by the C library's classification?
///
/// # Safety
///
/// None beyond reading the locale table.
pub unsafe fn is_digit_byte(c: c_char) -> bool {
    // SAFETY: the index is a byte value, which the table covers.
    unsafe {
        *(*__ctype_b_loc()).offset(c as uint8_t as c_int as isize) as c_int
            & _ISdigit as c_int as core::ffi::c_ushort as c_int
            != 0
    }
}

/// `FLAG`: how flags are spelled in the rest of the file.
///
/// # Safety
///
/// As [`handle_line`].
unsafe fn handle_flag_type(
    aff: *mut afffile_T,
    items: &[*mut c_char],
    fname: *mut c_char,
    lnum: c_int,
) {
    // SAFETY: the caller promises the items.
    unsafe {
        if strcmp(items[1], c"long".as_ptr()) == 0 {
            (*aff).af_flagtype = AFT_LONG;
        } else if strcmp(items[1], c"num".as_ptr()) == 0 {
            (*aff).af_flagtype = AFT_NUM;
        } else if strcmp(items[1], c"caplong".as_ptr()) == 0 {
            (*aff).af_flagtype = AFT_CAPLONG;
        } else {
            smsg_c!(
                0,
                gettext(c"Invalid value for FLAG in %s line %d: %s".as_ptr()),
                fname,
                lnum,
                items[1],
            );
        }
        // Anything already read used the old spelling, so it would be
        // interpreted wrongly.
        let used = (*aff).af_rare != 0
            || (*aff).af_keepcase != 0
            || (*aff).af_bad != 0
            || (*aff).af_needaffix != 0
            || (*aff).af_circumfix != 0
            || (*aff).af_needcomp != 0
            || (*aff).af_comproot != 0
            || (*aff).af_nosuggest != 0
            || (*aff).af_suff.ht_used > 0
            || (*aff).af_pref.ht_used > 0;
        if used {
            smsg_c!(
                0,
                gettext(c"FLAG after using flags in %s line %d: %s".as_ptr()),
                fname,
                lnum,
                items[1],
            );
        }
    }
}

/// Apply what the file collected, checking it against what earlier `.aff`
/// files of the same run already set.
///
/// # Safety
///
/// `spin`, `aff` and the state must be live.
unsafe fn finish_aff(
    spin: *mut spellinfo_T,
    aff: *mut afffile_T,
    st: &mut AffState,
    fname: *mut c_char,
) {
    // SAFETY: the caller promises the structures.
    unsafe {
        // The case tables are only used to decide whether the word
        // characters need rebuilding; their contents are not kept.
        if !st.fol.is_null() || !st.low.is_null() || !st.upp.is_null() {
            if (*spin).si_clear_chartab != 0 {
                init_spell_chartab();
                (*spin).si_clear_chartab = 0;
            }
            xfree(st.fol.cast());
            xfree(st.low.cast());
            xfree(st.upp.cast());
        }

        if st.compmax != 0 {
            aff_check_number((*spin).si_compmax, st.compmax, c"COMPOUNDWORDMAX");
            (*spin).si_compmax = st.compmax;
        }
        if st.compminlen != 0 {
            aff_check_number((*spin).si_compminlen, st.compminlen, c"COMPOUNDMIN");
            (*spin).si_compminlen = st.compminlen;
        }
        if st.compsylmax != 0 {
            if st.syllable.is_null() {
                smsg_c!(
                    0,
                    c"%s".as_ptr(),
                    gettext(c"COMPOUNDSYLMAX used without SYLLABLE".as_ptr()),
                );
            }
            aff_check_number((*spin).si_compsylmax, st.compsylmax, c"COMPOUNDSYLMAX");
            (*spin).si_compsylmax = st.compsylmax;
        }
        if st.compoptions != 0 {
            aff_check_number((*spin).si_compoptions, st.compoptions, c"COMPOUND options");
            (*spin).si_compoptions |= st.compoptions;
        }
        if !st.compflags.is_null() {
            process_compflags(spin, aff, st.compflags);
        }

        // Prefix ids count up and compound ids down; meeting means one kind
        // ran out of room.
        if (*spin).si_newcompID < (*spin).si_newprefID {
            let complaint = if (*spin).si_newcompID == 127 || (*spin).si_newcompID == 255 {
                c"Too many postponed prefixes"
            } else if (*spin).si_newprefID == 0 || (*spin).si_newprefID == 127 {
                c"Too many compound flags"
            } else {
                c"Too many postponed prefixes and/or compound flags"
            };
            msg(gettext(complaint.as_ptr()), 0);
        }

        if !st.syllable.is_null() {
            aff_check_string((*spin).si_syllable, st.syllable, c"SYLLABLE");
            (*spin).si_syllable = st.syllable;
        }

        if !st.sofofrom.is_null() || !st.sofoto.is_null() {
            if st.sofofrom.is_null() || st.sofoto.is_null() {
                smsg_c!(
                    0,
                    gettext(c"Missing SOFO%s line in %s".as_ptr()),
                    if st.sofofrom.is_null() {
                        c"FROM".as_ptr()
                    } else {
                        c"TO".as_ptr()
                    },
                    fname,
                );
            } else if (*spin).si_sal.ga_len > 0 {
                // SAL rules and a SOFO pair are two ways to do the same
                // thing; taking both would be ambiguous.
                smsg_c!(0, gettext(c"Both SAL and SOFO lines in %s".as_ptr()), fname);
            } else {
                aff_check_string((*spin).si_sofofr, st.sofofrom, c"SOFOFROM");
                aff_check_string((*spin).si_sofoto, st.sofoto, c"SOFOTO");
                (*spin).si_sofofr = st.sofofrom;
                (*spin).si_sofoto = st.sofoto;
            }
        }

        if !st.midword.is_null() {
            aff_check_string((*spin).si_midword, st.midword, c"MIDWORD");
            (*spin).si_midword = st.midword;
        }
    }
}

/// Warn when two `.aff` files of one run disagree about a number.
unsafe fn aff_check_number(spinval: c_int, affval: c_int, name: &CStr) {
    if spinval != 0 && spinval != affval {
        // SAFETY: the format and the name are both static strings.
        unsafe {
            smsg_c!(
                0,
                gettext(c"%s value differs from what is used in another .aff file".as_ptr()),
                name.as_ptr(),
            );
        }
    }
}

/// Warn when two `.aff` files of one run disagree about a string.
///
/// # Safety
///
/// Both values must be null or NUL-terminated.
unsafe fn aff_check_string(spinval: *mut c_char, affval: *mut c_char, name: &CStr) {
    // SAFETY: the caller promises the strings.
    unsafe {
        if !spinval.is_null() && strcmp(spinval, affval) != 0 {
            smsg_c!(
                0,
                gettext(c"%s value differs from what is used in another .aff file".as_ptr()),
                name.as_ptr(),
            );
        }
    }
}

/// Compare two strings, treating null as a value of its own.
///
/// # Safety
///
/// Non-null arguments must be NUL-terminated.
pub unsafe fn str_equal(s1: *mut c_char, s2: *mut c_char) -> bool {
    // SAFETY: the caller promises the strings.
    unsafe {
        if s1.is_null() || s2.is_null() {
            return s1 == s2;
        }
        strcmp(s1, s2) == 0
    }
}
