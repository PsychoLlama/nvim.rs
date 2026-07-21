//! Arabic contextual shaping: pick the presentation form (isolated /
//! initial / medial / final) of each Arabic codepoint from its neighbors,
//! including the lam-alef ligatures. Encoding-independent; operates on
//! Unicode codepoints. Active when 'arabicshape' is set and 'termbidi'
//! is not.

use core::ffi::c_int;

use crate::src::nvim::main::{p_arshape, p_tbidi};

const A_HAMZA: c_int = 0x0621;
const A_ALEF_MADDA: c_int = 0x0622;
const A_ALEF_HAMZA_ABOVE: c_int = 0x0623;
const A_ALEF_HAMZA_BELOW: c_int = 0x0625;
const A_ALEF: c_int = 0x0627;
const A_LAM: c_int = 0x0644;
const A_BYTE_ORDER_MARK: c_int = 0xfeff;

// Isolated and final presentation forms of the four lam-alef ligatures.
const A_S_LAM_ALEF_MADDA_ABOVE: c_int = 0xfef5;
const A_F_LAM_ALEF_MADDA_ABOVE: c_int = 0xfef6;
const A_S_LAM_ALEF_HAMZA_ABOVE: c_int = 0xfef7;
const A_F_LAM_ALEF_HAMZA_ABOVE: c_int = 0xfef8;
const A_S_LAM_ALEF_HAMZA_BELOW: c_int = 0xfef9;
const A_F_LAM_ALEF_HAMZA_BELOW: c_int = 0xfefa;
const A_S_LAM_ALEF: c_int = 0xfefb;
const A_F_LAM_ALEF: c_int = 0xfefc;

/// One shapable codepoint and its presentation forms (0 = no such form).
struct Achar {
    c: u32,
    isolated: u32,
    initial: u32,
    medial: u32,
    final_: u32,
}

const fn a(c: u32, isolated: u32, initial: u32, medial: u32, final_: u32) -> Achar {
    Achar {
        c,
        isolated,
        initial,
        medial,
        final_,
    }
}

/// Sorted by `c` for the binary search in [`find_achar`].
const ACHARS: [Achar; 54] = [
    a(0x0621, 0xfe80, 0, 0, 0),                // hamza
    a(0x0622, 0xfe81, 0, 0, 0xfe82),           // alef_madda
    a(0x0623, 0xfe83, 0, 0, 0xfe84),           // alef_hamza_above
    a(0x0624, 0xfe85, 0, 0, 0xfe86),           // waw_hamza
    a(0x0625, 0xfe87, 0, 0, 0xfe88),           // alef_hamza_below
    a(0x0626, 0xfe89, 0xfe8b, 0xfe8c, 0xfe8a), // yeh_hamza
    a(0x0627, 0xfe8d, 0, 0, 0xfe8e),           // alef
    a(0x0628, 0xfe8f, 0xfe91, 0xfe92, 0xfe90), // beh
    a(0x0629, 0xfe93, 0, 0, 0xfe94),           // teh_marbuta
    a(0x062a, 0xfe95, 0xfe97, 0xfe98, 0xfe96), // teh
    a(0x062b, 0xfe99, 0xfe9b, 0xfe9c, 0xfe9a), // theh
    a(0x062c, 0xfe9d, 0xfe9f, 0xfea0, 0xfe9e), // jeem
    a(0x062d, 0xfea1, 0xfea3, 0xfea4, 0xfea2), // hah
    a(0x062e, 0xfea5, 0xfea7, 0xfea8, 0xfea6), // khah
    a(0x062f, 0xfea9, 0, 0, 0xfeaa),           // dal
    a(0x0630, 0xfeab, 0, 0, 0xfeac),           // thal
    a(0x0631, 0xfead, 0, 0, 0xfeae),           // reh
    a(0x0632, 0xfeaf, 0, 0, 0xfeb0),           // zain
    a(0x0633, 0xfeb1, 0xfeb3, 0xfeb4, 0xfeb2), // seen
    a(0x0634, 0xfeb5, 0xfeb7, 0xfeb8, 0xfeb6), // sheen
    a(0x0635, 0xfeb9, 0xfebb, 0xfebc, 0xfeba), // sad
    a(0x0636, 0xfebd, 0xfebf, 0xfec0, 0xfebe), // dad
    a(0x0637, 0xfec1, 0xfec3, 0xfec4, 0xfec2), // tah
    a(0x0638, 0xfec5, 0xfec7, 0xfec8, 0xfec6), // zah
    a(0x0639, 0xfec9, 0xfecb, 0xfecc, 0xfeca), // ain
    a(0x063a, 0xfecd, 0xfecf, 0xfed0, 0xfece), // ghain
    a(0x0640, 0, 0x640, 0x640, 0x640),         // tatweel
    a(0x0641, 0xfed1, 0xfed3, 0xfed4, 0xfed2), // feh
    a(0x0642, 0xfed5, 0xfed7, 0xfed8, 0xfed6), // qaf
    a(0x0643, 0xfed9, 0xfedb, 0xfedc, 0xfeda), // kaf
    a(0x0644, 0xfedd, 0xfedf, 0xfee0, 0xfede), // lam
    a(0x0645, 0xfee1, 0xfee3, 0xfee4, 0xfee2), // meem
    a(0x0646, 0xfee5, 0xfee7, 0xfee8, 0xfee6), // noon
    a(0x0647, 0xfee9, 0xfeeb, 0xfeec, 0xfeea), // heh
    a(0x0648, 0xfeed, 0, 0, 0xfeee),           // waw
    a(0x0649, 0xfeef, 0, 0, 0xfef0),           // alef_maksura
    a(0x064a, 0xfef1, 0xfef3, 0xfef4, 0xfef2), // yeh
    a(0x064b, 0xfe70, 0, 0, 0),                // fathatan
    a(0x064c, 0xfe72, 0, 0, 0),                // dammatan
    a(0x064d, 0xfe74, 0, 0, 0),                // kasratan
    a(0x064e, 0xfe76, 0, 0xfe77, 0),           // fatha
    a(0x064f, 0xfe78, 0, 0xfe79, 0),           // damma
    a(0x0650, 0xfe7a, 0, 0xfe7b, 0),           // kasra
    a(0x0651, 0xfe7c, 0, 0xfe7c, 0),           // shadda
    a(0x0652, 0xfe7e, 0, 0xfe7f, 0),           // sukun
    a(0x0653, 0, 0, 0, 0),                     // madda_above
    a(0x0654, 0, 0, 0, 0),                     // hamza_above
    a(0x0655, 0, 0, 0, 0),                     // hamza_below
    a(0x067e, 0xfb56, 0xfb58, 0xfb59, 0xfb57), // peh
    a(0x0686, 0xfb7a, 0xfb7c, 0xfb7d, 0xfb7b), // tcheh
    a(0x0698, 0xfb8a, 0, 0, 0xfb8b),           // jeh
    a(0x06a9, 0xfb8e, 0xfb90, 0xfb91, 0xfb8f), // fkaf
    a(0x06af, 0xfb92, 0xfb94, 0xfb95, 0xfb93), // gaf
    a(0x06cc, 0xfbfc, 0xfbfe, 0xfbff, 0xfbfd), // fyeh
];

fn find_achar(c: c_int) -> Option<&'static Achar> {
    ACHARS
        .binary_search_by_key(&(c as u32), |a| a.c)
        .ok()
        .map(|i| &ACHARS[i])
}

/// Isolated lam-alef ligature for the alef variant `hid_c`.
fn chg_c_laa2i(hid_c: c_int) -> c_int {
    match hid_c {
        A_ALEF_MADDA => A_S_LAM_ALEF_MADDA_ABOVE,
        A_ALEF_HAMZA_ABOVE => A_S_LAM_ALEF_HAMZA_ABOVE,
        A_ALEF_HAMZA_BELOW => A_S_LAM_ALEF_HAMZA_BELOW,
        A_ALEF => A_S_LAM_ALEF,
        _ => 0,
    }
}

/// Final lam-alef ligature for the alef variant `hid_c`.
fn chg_c_laa2f(hid_c: c_int) -> c_int {
    match hid_c {
        A_ALEF_MADDA => A_F_LAM_ALEF_MADDA_ABOVE,
        A_ALEF_HAMZA_ABOVE => A_F_LAM_ALEF_HAMZA_ABOVE,
        A_ALEF_HAMZA_BELOW => A_F_LAM_ALEF_HAMZA_BELOW,
        A_ALEF => A_F_LAM_ALEF,
        _ => 0,
    }
}

/// Can `c1` be joined to a following `c2`?
fn can_join(c1: c_int, c2: c_int) -> bool {
    match (find_achar(c1), find_achar(c2)) {
        (Some(a1), Some(a2)) => {
            (a1.initial != 0 || a1.medial != 0) && (a2.final_ != 0 || a2.medial != 0)
        }
        _ => false,
    }
}

fn a_is_ok(c: c_int) -> bool {
    find_achar(c).is_some() || c == A_BYTE_ORDER_MARK
}

fn a_is_valid(c: c_int) -> bool {
    a_is_ok(c) && c != A_HAMZA
}

/// Is `two` an alef variant that combines with a preceding lam?
pub fn arabic_maycombine(two: c_int) -> bool {
    p_arshape.get() != 0
        && p_tbidi.get() == 0
        && matches!(
            two,
            A_ALEF_MADDA | A_ALEF_HAMZA_ABOVE | A_ALEF_HAMZA_BELOW | A_ALEF
        )
}

/// Do `one` and `two` combine into a lam-alef ligature?
pub fn arabic_combine(one: c_int, two: c_int) -> bool {
    one == A_LAM && arabic_maycombine(two)
}

/// Shape codepoint `c` (with combining char `*c1`) based on its neighbors.
/// Returns the shaped codepoint; `*c1` is zeroed when it was absorbed into
/// a lam-alef ligature.
pub fn arabic_shape(
    c: c_int,
    c1: &mut c_int,
    prev_c: c_int,
    prev_c1: c_int,
    next_c: c_int,
) -> c_int {
    if !a_is_ok(c) {
        return c;
    }
    let curr_laa = arabic_combine(c, *c1);
    let prev_laa = arabic_combine(prev_c, prev_c1);
    let curr_c;
    if curr_laa {
        if a_is_valid(prev_c) && can_join(prev_c, A_LAM) && !prev_laa {
            curr_c = chg_c_laa2f(*c1);
        } else {
            curr_c = chg_c_laa2i(*c1);
        }
        *c1 = 0;
    } else {
        // Only the byte-order mark passes a_is_ok without a table entry;
        // upstream C would dereference NULL here. Leave it unshaped.
        let Some(curr_a) = find_achar(c) else {
            return c;
        };
        let backward_combine = !prev_laa && can_join(prev_c, c);
        let forward_combine = can_join(c, next_c);
        curr_c = if backward_combine {
            if forward_combine {
                curr_a.medial as c_int
            } else {
                curr_a.final_ as c_int
            }
        } else if forward_combine {
            curr_a.initial as c_int
        } else {
            curr_a.isolated as c_int
        };
    }
    if curr_c == 0 {
        c
    } else {
        curr_c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn achars_table_is_sorted_and_unique() {
        assert!(ACHARS.windows(2).all(|w| w[0].c < w[1].c));
    }

    #[test]
    fn find_achar_matches_linear_scan() {
        for c in 0..0x10000 {
            let expect = ACHARS.iter().find(|a| a.c == c as u32).map(|a| a.c);
            assert_eq!(find_achar(c).map(|a| a.c), expect);
        }
    }
}
