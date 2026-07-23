//! Unicode character properties, case mapping, and grapheme-cluster breaks.
//!
//! This is an in-tree port of the subset of utf8proc v2.11.3 (Unicode
//! 17.0.0) that nvim used: `utf8proc_get_property` and its property table,
//! the extended-grapheme-cluster break functions, `utf8proc_tolower` /
//! `utf8proc_toupper`, and `utf8proc_decompose_char`. The composition and
//! whole-string map/normalize APIs had no callers and are not ported.
//!
//! Unicode behavior is user-visible (rendering width, cursor motion, case
//! folding), so this follows the C line by line rather than delegating to a
//! Unicode crate that tracks a different Unicode version. The data tables
//! in `tables.rs` are generated from the same upstream release by
//! `tools/utf8proc-tables/gen.py`.
//!
//! utf8proc is Copyright (c) 2014-2021 Steven G. Johnson, Jiahao Chen,
//! Tony Kelman, Jonas Fonseca, and other contributors, and Copyright (c)
//! 2009, 2013 Public Software Group e. V., Berlin, Germany, under the MIT
//! "expat" license; the data derives from the Unicode data files under the
//! Unicode, Inc. license. This port stays under those terms; both notices
//! are reproduced in full in licenses/utf8proc-LICENSE.md.

mod tables;

use tables::{PROPERTIES, SEQUENCES, STAGE1, STAGE2};

/// The properties of a single codepoint. Field names and meanings match the
/// C `utf8proc_property_t`; the C bitfields are widened to plain fields
/// (this struct no longer crosses an FFI boundary).
///
/// The `*_seqindex` fields index [`SEQUENCES`] (`0xFFFF` = none) with the
/// length packed in the top two bits; the `comb_*` fields describe the
/// canonical-composition pairing of the unported compose path and are kept
/// so the tables remain a faithful translation of upstream's.
pub struct utf8proc_property_t {
    pub category: i16,
    pub combining_class: i16,
    pub bidi_class: i16,
    pub decomp_type: i16,
    pub decomp_seqindex: u16,
    pub casefold_seqindex: u16,
    pub uppercase_seqindex: u16,
    pub lowercase_seqindex: u16,
    pub titlecase_seqindex: u16,
    pub comb_index: u16,
    pub comb_length: u8,
    pub comb_issecond: bool,
    pub bidi_mirrored: bool,
    pub comp_exclusion: bool,
    pub ignorable: bool,
    pub control_boundary: bool,
    pub charwidth: u8,
    pub ambiguous_width: bool,
    pub boundclass: u8,
    pub indic_conjunct_break: u8,
}

/// "No entry" sentinel for the `*_seqindex` fields (C `UINT16_MAX`).
const SEQINDEX_NONE: u16 = u16::MAX;

// Unicode general categories (utf8proc_category_t), in `category` field
// order.
pub const UTF8PROC_CATEGORY_CN: i16 = 0;
pub const UTF8PROC_CATEGORY_LU: i16 = 1;
pub const UTF8PROC_CATEGORY_LL: i16 = 2;
pub const UTF8PROC_CATEGORY_LT: i16 = 3;
pub const UTF8PROC_CATEGORY_LM: i16 = 4;
pub const UTF8PROC_CATEGORY_LO: i16 = 5;
pub const UTF8PROC_CATEGORY_MN: i16 = 6;
pub const UTF8PROC_CATEGORY_MC: i16 = 7;
pub const UTF8PROC_CATEGORY_ME: i16 = 8;
pub const UTF8PROC_CATEGORY_ND: i16 = 9;
pub const UTF8PROC_CATEGORY_NL: i16 = 10;
pub const UTF8PROC_CATEGORY_NO: i16 = 11;
pub const UTF8PROC_CATEGORY_PC: i16 = 12;
pub const UTF8PROC_CATEGORY_PD: i16 = 13;
pub const UTF8PROC_CATEGORY_PS: i16 = 14;
pub const UTF8PROC_CATEGORY_PE: i16 = 15;
pub const UTF8PROC_CATEGORY_PI: i16 = 16;
pub const UTF8PROC_CATEGORY_PF: i16 = 17;
pub const UTF8PROC_CATEGORY_PO: i16 = 18;
pub const UTF8PROC_CATEGORY_SM: i16 = 19;
pub const UTF8PROC_CATEGORY_SC: i16 = 20;
pub const UTF8PROC_CATEGORY_SK: i16 = 21;
pub const UTF8PROC_CATEGORY_SO: i16 = 22;
pub const UTF8PROC_CATEGORY_ZS: i16 = 23;
pub const UTF8PROC_CATEGORY_ZL: i16 = 24;
pub const UTF8PROC_CATEGORY_ZP: i16 = 25;
pub const UTF8PROC_CATEGORY_CC: i16 = 26;
pub const UTF8PROC_CATEGORY_CF: i16 = 27;
pub const UTF8PROC_CATEGORY_CS: i16 = 28;
pub const UTF8PROC_CATEGORY_CO: i16 = 29;

// Grapheme boundclasses (utf8proc_boundclass_t), in `boundclass` field
// order.
pub const UTF8PROC_BOUNDCLASS_START: i32 = 0;
pub const UTF8PROC_BOUNDCLASS_OTHER: i32 = 1;
pub const UTF8PROC_BOUNDCLASS_CR: i32 = 2;
pub const UTF8PROC_BOUNDCLASS_LF: i32 = 3;
pub const UTF8PROC_BOUNDCLASS_CONTROL: i32 = 4;
pub const UTF8PROC_BOUNDCLASS_EXTEND: i32 = 5;
pub const UTF8PROC_BOUNDCLASS_L: i32 = 6;
pub const UTF8PROC_BOUNDCLASS_V: i32 = 7;
pub const UTF8PROC_BOUNDCLASS_T: i32 = 8;
pub const UTF8PROC_BOUNDCLASS_LV: i32 = 9;
pub const UTF8PROC_BOUNDCLASS_LVT: i32 = 10;
pub const UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR: i32 = 11;
pub const UTF8PROC_BOUNDCLASS_SPACINGMARK: i32 = 12;
pub const UTF8PROC_BOUNDCLASS_PREPEND: i32 = 13;
pub const UTF8PROC_BOUNDCLASS_ZWJ: i32 = 14;
pub const UTF8PROC_BOUNDCLASS_E_BASE: i32 = 15;
pub const UTF8PROC_BOUNDCLASS_E_MODIFIER: i32 = 16;
pub const UTF8PROC_BOUNDCLASS_GLUE_AFTER_ZWJ: i32 = 17;
pub const UTF8PROC_BOUNDCLASS_E_BASE_GAZ: i32 = 18;
pub const UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC: i32 = 19;
pub const UTF8PROC_BOUNDCLASS_E_ZWG: i32 = 20;

// Indic_Conjunct_Break property values (utf8proc_indic_conjunct_break_t),
// in `indic_conjunct_break` field order.
pub const UTF8PROC_INDIC_CONJUNCT_BREAK_NONE: i32 = 0;
pub const UTF8PROC_INDIC_CONJUNCT_BREAK_LINKER: i32 = 1;
pub const UTF8PROC_INDIC_CONJUNCT_BREAK_CONSONANT: i32 = 2;
pub const UTF8PROC_INDIC_CONJUNCT_BREAK_EXTEND: i32 = 3;

// Option bits accepted by utf8proc_decompose_char.
pub type utf8proc_option_t = u32;
pub const UTF8PROC_NULLTERM: utf8proc_option_t = 1 << 0;
pub const UTF8PROC_STABLE: utf8proc_option_t = 1 << 1;
pub const UTF8PROC_COMPAT: utf8proc_option_t = 1 << 2;
pub const UTF8PROC_COMPOSE: utf8proc_option_t = 1 << 3;
pub const UTF8PROC_DECOMPOSE: utf8proc_option_t = 1 << 4;
pub const UTF8PROC_IGNORE: utf8proc_option_t = 1 << 5;
pub const UTF8PROC_REJECTNA: utf8proc_option_t = 1 << 6;
pub const UTF8PROC_NLF2LS: utf8proc_option_t = 1 << 7;
pub const UTF8PROC_NLF2PS: utf8proc_option_t = 1 << 8;
pub const UTF8PROC_NLF2LF: utf8proc_option_t = UTF8PROC_NLF2LS | UTF8PROC_NLF2PS;
pub const UTF8PROC_STRIPCC: utf8proc_option_t = 1 << 9;
pub const UTF8PROC_CASEFOLD: utf8proc_option_t = 1 << 10;
pub const UTF8PROC_CHARBOUND: utf8proc_option_t = 1 << 11;
pub const UTF8PROC_LUMP: utf8proc_option_t = 1 << 12;
pub const UTF8PROC_STRIPMARK: utf8proc_option_t = 1 << 13;
pub const UTF8PROC_STRIPNA: utf8proc_option_t = 1 << 14;

// Error returns of utf8proc_decompose_char.
pub const UTF8PROC_ERROR_OVERFLOW: isize = -2;
pub const UTF8PROC_ERROR_NOTASSIGNED: isize = -4;

// Hangul syllable decomposition (Unicode chapter 3.12).
const HANGUL_SBASE: i32 = 0xAC00;
const HANGUL_LBASE: i32 = 0x1100;
const HANGUL_VBASE: i32 = 0x1161;
const HANGUL_TBASE: i32 = 0x11A7;
const HANGUL_TCOUNT: i32 = 28;
const HANGUL_NCOUNT: i32 = 588;
const HANGUL_SCOUNT: i32 = 11172;

/// C `unsafe_get_property`: the two-stage table walk, valid for
/// `0 <= uc < 0x110000`.
#[inline]
fn property_unchecked(uc: i32) -> &'static utf8proc_property_t {
    &PROPERTIES[STAGE2[STAGE1[(uc >> 8) as usize] as usize + (uc & 0xFF) as usize] as usize]
}

/// Looks up the properties of `uc`. Out-of-range codepoints (including
/// negative ones) get `PROPERTIES[0]`, the all-defaults entry, exactly as
/// in C.
#[inline]
pub fn utf8proc_get_property(uc: i32) -> &'static utf8proc_property_t {
    if (0..0x110000).contains(&uc) {
        property_unchecked(uc)
    } else {
        &PROPERTIES[0]
    }
}

/// Whether an extended grapheme cluster break is permitted between
/// boundclasses `lbc` and `tbc`, per TR29 rules GB1-GB999. GB9c, GB10, and
/// GB12/13 need cross-character state and are handled by
/// [`grapheme_break_extended`].
fn grapheme_break_simple(lbc: i32, tbc: i32) -> bool {
    if lbc == UTF8PROC_BOUNDCLASS_START {
        return true; // GB1
    }
    if lbc == UTF8PROC_BOUNDCLASS_CR && tbc == UTF8PROC_BOUNDCLASS_LF {
        return false; // GB3
    }
    if (UTF8PROC_BOUNDCLASS_CR..=UTF8PROC_BOUNDCLASS_CONTROL).contains(&lbc) {
        return true; // GB4
    }
    if (UTF8PROC_BOUNDCLASS_CR..=UTF8PROC_BOUNDCLASS_CONTROL).contains(&tbc) {
        return true; // GB5
    }
    if lbc == UTF8PROC_BOUNDCLASS_L
        && (tbc == UTF8PROC_BOUNDCLASS_L
            || tbc == UTF8PROC_BOUNDCLASS_V
            || tbc == UTF8PROC_BOUNDCLASS_LV
            || tbc == UTF8PROC_BOUNDCLASS_LVT)
    {
        return false; // GB6
    }
    if (lbc == UTF8PROC_BOUNDCLASS_LV || lbc == UTF8PROC_BOUNDCLASS_V)
        && (tbc == UTF8PROC_BOUNDCLASS_V || tbc == UTF8PROC_BOUNDCLASS_T)
    {
        return false; // GB7
    }
    if (lbc == UTF8PROC_BOUNDCLASS_LVT || lbc == UTF8PROC_BOUNDCLASS_T)
        && tbc == UTF8PROC_BOUNDCLASS_T
    {
        return false; // GB8
    }
    if tbc == UTF8PROC_BOUNDCLASS_EXTEND // GB9
        || tbc == UTF8PROC_BOUNDCLASS_ZWJ
        || tbc == UTF8PROC_BOUNDCLASS_SPACINGMARK // GB9a
        || lbc == UTF8PROC_BOUNDCLASS_PREPEND
    {
        return false; // GB9b
    }
    if lbc == UTF8PROC_BOUNDCLASS_E_ZWG && tbc == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC {
        return false; // GB11
    }
    if lbc == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR
        && tbc == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR
    {
        return false; // GB12/13
    }
    true // GB999
}

/// The stateful break decision. `*state == 0` means uninitialized; after a
/// call, byte 0 of `*state` holds the effective boundclass and byte 1 the
/// Indic_Conjunct_Break state, exactly matching the C encoding (callers
/// persist this value across calls).
fn grapheme_break_extended(
    lbc: i32,
    tbc: i32,
    licb: i32,
    ticb: i32,
    state: Option<&mut i32>,
) -> bool {
    let Some(state) = state else {
        return grapheme_break_simple(lbc, tbc);
    };
    let mut state_bc;
    let mut state_icb;
    if *state == 0 {
        state_bc = lbc;
        state_icb = if licb == UTF8PROC_INDIC_CONJUNCT_BREAK_CONSONANT {
            licb
        } else {
            UTF8PROC_INDIC_CONJUNCT_BREAK_NONE
        };
    } else {
        state_bc = *state & 0xff;
        state_icb = *state >> 8;
    }

    let break_permitted = grapheme_break_simple(state_bc, tbc)
        && !(state_icb == UTF8PROC_INDIC_CONJUNCT_BREAK_LINKER
            && ticb == UTF8PROC_INDIC_CONJUNCT_BREAK_CONSONANT); // GB9c

    // GB9c bookkeeping: don't break between two consonants separated by 1+
    // linkers and 0+ extends in any order; LINKER state is entered after at
    // least one linker following a consonant.
    if ticb == UTF8PROC_INDIC_CONJUNCT_BREAK_CONSONANT
        || state_icb == UTF8PROC_INDIC_CONJUNCT_BREAK_CONSONANT
        || state_icb == UTF8PROC_INDIC_CONJUNCT_BREAK_EXTEND
    {
        state_icb = ticb;
    } else if state_icb == UTF8PROC_INDIC_CONJUNCT_BREAK_LINKER {
        state_icb = if ticb == UTF8PROC_INDIC_CONJUNCT_BREAK_EXTEND {
            UTF8PROC_INDIC_CONJUNCT_BREAK_LINKER
        } else {
            ticb
        };
    }

    // GB12/13: after two regional indicators, reset the second one's class
    // to OTHER so GB999 forces a break after it. GB11: track emoji + zwj
    // combos through EXTEND folds.
    if state_bc == tbc && tbc == UTF8PROC_BOUNDCLASS_REGIONAL_INDICATOR {
        state_bc = UTF8PROC_BOUNDCLASS_OTHER;
    } else if state_bc == UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC {
        state_bc = if tbc == UTF8PROC_BOUNDCLASS_EXTEND {
            UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC
        } else if tbc == UTF8PROC_BOUNDCLASS_ZWJ {
            UTF8PROC_BOUNDCLASS_E_ZWG
        } else {
            tbc
        };
    } else {
        state_bc = tbc;
    }

    *state = state_bc + (state_icb << 8);
    break_permitted
}

/// Whether a grapheme break is permitted between `c1` and `c2`, carrying
/// the GB9c/GB11/GB12/13 lookahead in `*state` (pass `Some(&mut 0)` at the
/// start of a sequence and keep the value across calls).
pub fn utf8proc_grapheme_break_stateful(c1: i32, c2: i32, state: Option<&mut i32>) -> bool {
    let p1 = utf8proc_get_property(c1);
    let p2 = utf8proc_get_property(c2);
    grapheme_break_extended(
        p1.boundclass as i32,
        p2.boundclass as i32,
        p1.indic_conjunct_break as i32,
        p2.indic_conjunct_break as i32,
        state,
    )
}

/// Stateless break check; may miss GB9c/GB10/GB12/13 breaks that need
/// context beyond the pair.
pub fn utf8proc_grapheme_break(c1: i32, c2: i32) -> bool {
    utf8proc_grapheme_break_stateful(c1, c2, None)
}

/// Decodes the codepoint starting at `SEQUENCES[*i]`, advancing `*i` past
/// the extra unit when it is stored as a UTF-16 surrogate pair.
fn seqindex_decode_entry(i: &mut usize) -> i32 {
    let mut entry_cp = SEQUENCES[*i] as i32;
    if (entry_cp & 0xF800) == 0xD800 {
        *i += 1;
        entry_cp = ((entry_cp & 0x03FF) << 10) | (SEQUENCES[*i] as i32 & 0x03FF);
        entry_cp += 0x10000;
    }
    entry_cp
}

fn seqindex_decode_index(seqindex: u16) -> i32 {
    seqindex_decode_entry(&mut (seqindex as usize))
}

/// Writes the (recursively decomposed) sequence addressed by `seqindex`:
/// bits 0..14 index [`SEQUENCES`], bits 14..16 encode the length (3 = "the
/// length is the first table entry").
fn seqindex_write_char_decomposed(
    seqindex: u16,
    dst: &mut [i32],
    options: utf8proc_option_t,
    mut last_boundclass: Option<&mut i32>,
) -> isize {
    let mut written: isize = 0;
    let mut i = (seqindex & 0x3FFF) as usize;
    let mut len = (seqindex >> 14) as i32;
    if len >= 3 {
        len = SEQUENCES[i] as i32;
        i += 1;
    }
    while len >= 0 {
        let entry_cp = seqindex_decode_entry(&mut i);
        let sub = if (written as usize) < dst.len() {
            &mut dst[written as usize..]
        } else {
            &mut [][..]
        };
        written += utf8proc_decompose_char(entry_cp, sub, options, last_boundclass.as_deref_mut());
        if written < 0 {
            return UTF8PROC_ERROR_OVERFLOW;
        }
        i += 1;
        len -= 1;
    }
    written
}

/// Lowercases `c`, or returns it unchanged if it has no lowercase mapping.
pub fn utf8proc_tolower(c: i32) -> i32 {
    let cl = utf8proc_get_property(c).lowercase_seqindex;
    if cl != SEQINDEX_NONE {
        seqindex_decode_index(cl)
    } else {
        c
    }
}

/// Uppercases `c`, or returns it unchanged if it has no uppercase mapping.
pub fn utf8proc_toupper(c: i32) -> i32 {
    let cu = utf8proc_get_property(c).uppercase_seqindex;
    if cu != SEQINDEX_NONE {
        seqindex_decode_index(cu)
    } else {
        c
    }
}

/// The ordered `UTF8PROC_LUMP` replacement chain of C
/// `utf8proc_decompose_char`, returning the first matching lump target.
fn lump_replacement(uc: i32, category: i16, options: utf8proc_option_t) -> Option<i32> {
    if category == UTF8PROC_CATEGORY_ZS {
        return Some(0x0020);
    }
    if uc == 0x2018 || uc == 0x2019 || uc == 0x02BC || uc == 0x02C8 {
        return Some(0x0027);
    }
    if category == UTF8PROC_CATEGORY_PD || uc == 0x2212 {
        return Some(0x002D);
    }
    if uc == 0x2044 || uc == 0x2215 {
        return Some(0x002F);
    }
    if uc == 0x2236 {
        return Some(0x003A);
    }
    if uc == 0x2039 || uc == 0x2329 || uc == 0x3008 {
        return Some(0x003C);
    }
    if uc == 0x203A || uc == 0x232A || uc == 0x3009 {
        return Some(0x003E);
    }
    if uc == 0x2216 {
        return Some(0x005C);
    }
    if uc == 0x02C4 || uc == 0x02C6 || uc == 0x2038 || uc == 0x2303 {
        return Some(0x005E);
    }
    if category == UTF8PROC_CATEGORY_PC || uc == 0x02CD {
        return Some(0x005F);
    }
    if uc == 0x02CB {
        return Some(0x0060);
    }
    if uc == 0x2223 {
        return Some(0x007C);
    }
    if uc == 0x223C {
        return Some(0x007E);
    }
    if (options & UTF8PROC_NLF2LS != 0 && options & UTF8PROC_NLF2PS != 0)
        && (category == UTF8PROC_CATEGORY_ZL || category == UTF8PROC_CATEGORY_ZP)
    {
        return Some(0x000A);
    }
    None
}

/// Writes the decomposition of `uc` into the front of `dst` (C passed a
/// pointer + `bufsize`; here `dst.len()` is the buffer size, and an empty
/// slice sizes the output without writing). Returns the number of
/// codepoints the full decomposition needs — which may exceed `dst.len()`,
/// in which case only the prefix was written — or a negative
/// `UTF8PROC_ERROR_*` code.
pub fn utf8proc_decompose_char(
    uc: i32,
    dst: &mut [i32],
    options: utf8proc_option_t,
    last_boundclass: Option<&mut i32>,
) -> isize {
    if !(0..0x110000).contains(&uc) {
        return UTF8PROC_ERROR_NOTASSIGNED;
    }
    let property = property_unchecked(uc);
    let category = property.category;
    let hangul_sindex = uc - HANGUL_SBASE;
    if options & (UTF8PROC_COMPOSE | UTF8PROC_DECOMPOSE) != 0
        && (0..HANGUL_SCOUNT).contains(&hangul_sindex)
    {
        if !dst.is_empty() {
            dst[0] = HANGUL_LBASE + hangul_sindex / HANGUL_NCOUNT;
            if dst.len() >= 2 {
                dst[1] = HANGUL_VBASE + (hangul_sindex % HANGUL_NCOUNT) / HANGUL_TCOUNT;
            }
        }
        let hangul_tindex = hangul_sindex % HANGUL_TCOUNT;
        if hangul_tindex == 0 {
            return 2;
        }
        if dst.len() >= 3 {
            dst[2] = HANGUL_TBASE + hangul_tindex;
        }
        return 3;
    }
    if options & UTF8PROC_REJECTNA != 0 && category == 0 {
        return UTF8PROC_ERROR_NOTASSIGNED;
    }
    if options & UTF8PROC_IGNORE != 0 && property.ignorable {
        return 0;
    }
    if options & UTF8PROC_STRIPNA != 0 && category == 0 {
        return 0;
    }
    if options & UTF8PROC_LUMP != 0 {
        if let Some(replacement) = lump_replacement(uc, category, options) {
            return utf8proc_decompose_char(
                replacement,
                dst,
                options & !UTF8PROC_LUMP,
                last_boundclass,
            );
        }
    }
    if options & UTF8PROC_STRIPMARK != 0
        && (category == UTF8PROC_CATEGORY_MN
            || category == UTF8PROC_CATEGORY_MC
            || category == UTF8PROC_CATEGORY_ME)
    {
        return 0;
    }
    if options & UTF8PROC_CASEFOLD != 0 && property.casefold_seqindex != SEQINDEX_NONE {
        return seqindex_write_char_decomposed(
            property.casefold_seqindex,
            dst,
            options,
            last_boundclass,
        );
    }
    if options & (UTF8PROC_COMPOSE | UTF8PROC_DECOMPOSE) != 0
        && property.decomp_seqindex != SEQINDEX_NONE
        && (property.decomp_type == 0 || options & UTF8PROC_COMPAT != 0)
    {
        return seqindex_write_char_decomposed(
            property.decomp_seqindex,
            dst,
            options,
            last_boundclass,
        );
    }
    if options & UTF8PROC_CHARBOUND != 0 {
        let boundary = grapheme_break_extended(
            0,
            property.boundclass as i32,
            0,
            property.indic_conjunct_break as i32,
            last_boundclass,
        );
        if boundary {
            if !dst.is_empty() {
                dst[0] = -1; // sentinel value for grapheme break
            }
            if dst.len() >= 2 {
                dst[1] = uc;
            }
            return 2;
        }
    }
    if !dst.is_empty() {
        dst[0] = uc;
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_lookup() {
        assert_eq!(
            utf8proc_get_property('A' as i32).category,
            UTF8PROC_CATEGORY_LU
        );
        assert_eq!(utf8proc_get_property(0x3042).charwidth, 2); // あ
        assert_eq!(utf8proc_get_property(0x0301).combining_class, 230); // combining acute
        assert!(utf8proc_get_property(0x00A7).ambiguous_width); // § (East Asian A)
        assert_eq!(
            utf8proc_get_property(0x1F600).boundclass as i32, // 😀
            UTF8PROC_BOUNDCLASS_EXTENDED_PICTOGRAPHIC
        );
        // Out-of-range codepoints resolve to the all-defaults entry.
        assert_eq!(utf8proc_get_property(-1).category, UTF8PROC_CATEGORY_CN);
        assert_eq!(
            utf8proc_get_property(0x110000).boundclass as i32,
            UTF8PROC_BOUNDCLASS_OTHER
        );
    }

    #[test]
    fn case_mapping() {
        assert_eq!(utf8proc_tolower('A' as i32), 'a' as i32);
        assert_eq!(utf8proc_toupper(0x00E9), 0x00C9); // é -> É
        assert_eq!(utf8proc_tolower(0x0130), 0x0069); // İ -> i
        assert_eq!(utf8proc_toupper(0x00DF), 0x1E9E); // ß -> ẞ
        assert_eq!(utf8proc_tolower(0x4E2D), 0x4E2D); // no case
                                                      // Deseret is the classic astral-plane (surrogate-encoded) case pair.
        assert_eq!(utf8proc_tolower(0x10400), 0x10428);
        assert_eq!(utf8proc_toupper(0x10428), 0x10400);
    }

    #[test]
    fn casefold_decompose() {
        let mut buf = [0i32; 4];
        // ß casefolds to "ss": needs 2 slots, only the prefix fits in 1.
        assert_eq!(
            utf8proc_decompose_char(0x00DF, &mut buf, UTF8PROC_CASEFOLD, None),
            2
        );
        assert_eq!(&buf[..2], &[0x73, 0x73]);
        assert_eq!(
            utf8proc_decompose_char(0x00DF, &mut buf[..1], UTF8PROC_CASEFOLD, None),
            2
        );
        // ﬃ ligature casefolds to "ffi".
        assert_eq!(
            utf8proc_decompose_char(0xFB03, &mut buf, UTF8PROC_CASEFOLD, None),
            3
        );
        assert_eq!(&buf[..3], &[0x66, 0x66, 0x69]);
        // Hangul syllable 한 decomposes to L+V+T jamo.
        assert_eq!(
            utf8proc_decompose_char(0xD55C, &mut buf, UTF8PROC_DECOMPOSE, None),
            3
        );
        assert_eq!(&buf[..3], &[0x1112, 0x1161, 0x11AB]);
        assert_eq!(
            utf8proc_decompose_char(-1, &mut buf, UTF8PROC_CASEFOLD, None),
            UTF8PROC_ERROR_NOTASSIGNED
        );
    }

    #[test]
    fn grapheme_breaks() {
        // a|b breaks; e + combining acute does not.
        assert!(utf8proc_grapheme_break('a' as i32, 'b' as i32));
        assert!(!utf8proc_grapheme_break('e' as i32, 0x0301));
        // CRLF is one cluster, LF CR is two.
        assert!(!utf8proc_grapheme_break(0x0D, 0x0A));
        assert!(utf8proc_grapheme_break(0x0A, 0x0D));
        // Regional-indicator pairing needs state: RI RI | RI RI.
        let mut state = 0;
        assert!(!utf8proc_grapheme_break_stateful(
            0x1F1E6,
            0x1F1E7,
            Some(&mut state)
        ));
        assert!(utf8proc_grapheme_break_stateful(
            0x1F1E7,
            0x1F1E6,
            Some(&mut state)
        ));
        // GB11: emoji zwj emoji stays joined, but only via the stateful API.
        let mut state = 0;
        assert!(!utf8proc_grapheme_break_stateful(
            0x1F469,
            0x200D,
            Some(&mut state)
        ));
        assert!(!utf8proc_grapheme_break_stateful(
            0x200D,
            0x1F469,
            Some(&mut state)
        ));
    }
}
