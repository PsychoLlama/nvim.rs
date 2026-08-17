#![forbid(unsafe_code)]

//! The CSI driver's fixed key tables.
//!
//! Upstream filled four mutable globals at first use through a `register_keys`
//! function of some 380 `register_*` calls, guarded by an "initialised" flag.
//! Nothing ever registered a key at runtime, so they are constants here.
//!
//! Ported from libtermkey, Copyright (c) 2007-2011 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libtermkey-LICENSE.txt.

use crate::tui::termkey::termkey::{
    TERMKEY_KEYMOD_SHIFT, TERMKEY_SYM_BEGIN, TERMKEY_SYM_DELETE, TERMKEY_SYM_DOWN, TERMKEY_SYM_END,
    TERMKEY_SYM_FIND, TERMKEY_SYM_HOME, TERMKEY_SYM_INSERT, TERMKEY_SYM_KP0, TERMKEY_SYM_KP1,
    TERMKEY_SYM_KP2, TERMKEY_SYM_KP3, TERMKEY_SYM_KP4, TERMKEY_SYM_KP5, TERMKEY_SYM_KP6,
    TERMKEY_SYM_KP7, TERMKEY_SYM_KP8, TERMKEY_SYM_KP9, TERMKEY_SYM_KPCOMMA, TERMKEY_SYM_KPDIV,
    TERMKEY_SYM_KPENTER, TERMKEY_SYM_KPEQUALS, TERMKEY_SYM_KPMINUS, TERMKEY_SYM_KPMULT,
    TERMKEY_SYM_KPPERIOD, TERMKEY_SYM_KPPLUS, TERMKEY_SYM_LEFT, TERMKEY_SYM_PAGEDOWN,
    TERMKEY_SYM_PAGEUP, TERMKEY_SYM_RIGHT, TERMKEY_SYM_SELECT, TERMKEY_SYM_TAB,
    TERMKEY_SYM_UNKNOWN, TERMKEY_SYM_UP, TERMKEY_TYPE_FUNCTION, TERMKEY_TYPE_KEYSYM,
    TERMKEY_TYPE_UNICODE,
};
use crate::types::{TermKeySym, keyinfo};
use core::ffi::{c_char, c_int};

/// Final bytes and SS3 commands live in 0x40..0x80, so the tables are indexed
/// by the byte less this.
pub const CSI_FINAL_BASE: u8 = 0x40;

/// How many `CSI N ~` function numbers are recognised.
pub const CSI_FUNC_COUNT: usize = 35;

/// No key registered for this slot.
const UNSET: keyinfo = keyinfo {
    type_0: TERMKEY_TYPE_UNICODE,
    sym: TERMKEY_SYM_UNKNOWN,
    modifier_mask: 0,
    modifier_set: 0,
};

const fn sym(sym: TermKeySym) -> keyinfo {
    keyinfo {
        type_0: TERMKEY_TYPE_KEYSYM,
        sym,
        modifier_mask: 0,
        modifier_set: 0,
    }
}

/// A numbered function key. `keyinfo::sym` carries the number, not a symbol,
/// when the type is TERMKEY_TYPE_FUNCTION.
const fn func(number: c_int) -> keyinfo {
    keyinfo {
        type_0: TERMKEY_TYPE_FUNCTION,
        sym: number,
        modifier_mask: 0,
        modifier_set: 0,
    }
}

const fn slot(final_byte: u8) -> usize {
    (final_byte - CSI_FINAL_BASE) as usize
}

/// Keys named by a CSI final byte (`CSI A`), which double as SS3 commands
/// (`SS3 A`). Also reachable with modifiers as `CSI 1;<mod> A`.
pub static CSI_SS3: [keyinfo; 64] = {
    let mut table = [UNSET; 64];
    table[slot(b'A')] = sym(TERMKEY_SYM_UP);
    table[slot(b'B')] = sym(TERMKEY_SYM_DOWN);
    table[slot(b'C')] = sym(TERMKEY_SYM_RIGHT);
    table[slot(b'D')] = sym(TERMKEY_SYM_LEFT);
    table[slot(b'E')] = sym(TERMKEY_SYM_BEGIN);
    table[slot(b'F')] = sym(TERMKEY_SYM_END);
    table[slot(b'H')] = sym(TERMKEY_SYM_HOME);
    table[slot(b'P')] = func(1);
    table[slot(b'Q')] = func(2);
    table[slot(b'R')] = func(3);
    table[slot(b'S')] = func(4);
    table[slot(b'Z')] = keyinfo {
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_TAB,
        modifier_mask: TERMKEY_KEYMOD_SHIFT as c_int,
        modifier_set: TERMKEY_KEYMOD_SHIFT as c_int,
    };
    table
};

/// Keypad keys, reachable only through SS3 (`SS3 p` is keypad 0). Kept apart
/// from `CSI_SS3` because the same bytes mean different keys there.
pub static SS3: [keyinfo; 64] = {
    let mut table = [UNSET; 64];
    table[slot(b'M')] = sym(TERMKEY_SYM_KPENTER);
    table[slot(b'X')] = sym(TERMKEY_SYM_KPEQUALS);
    table[slot(b'j')] = sym(TERMKEY_SYM_KPMULT);
    table[slot(b'k')] = sym(TERMKEY_SYM_KPPLUS);
    table[slot(b'l')] = sym(TERMKEY_SYM_KPCOMMA);
    table[slot(b'm')] = sym(TERMKEY_SYM_KPMINUS);
    table[slot(b'n')] = sym(TERMKEY_SYM_KPPERIOD);
    table[slot(b'o')] = sym(TERMKEY_SYM_KPDIV);
    table[slot(b'p')] = sym(TERMKEY_SYM_KP0);
    table[slot(b'q')] = sym(TERMKEY_SYM_KP1);
    table[slot(b'r')] = sym(TERMKEY_SYM_KP2);
    table[slot(b's')] = sym(TERMKEY_SYM_KP3);
    table[slot(b't')] = sym(TERMKEY_SYM_KP4);
    table[slot(b'u')] = sym(TERMKEY_SYM_KP5);
    table[slot(b'v')] = sym(TERMKEY_SYM_KP6);
    table[slot(b'w')] = sym(TERMKEY_SYM_KP7);
    table[slot(b'x')] = sym(TERMKEY_SYM_KP8);
    table[slot(b'y')] = sym(TERMKEY_SYM_KP9);
    table
};

/// The plain character each keypad key stands for, reported instead of the
/// keypad symbol under TERMKEY_FLAG_CONVERTKP. Zero where there is none.
pub static SS3_KEYPAD_ALT: [c_char; 64] = {
    let mut table = [0; 64];
    table[slot(b'X')] = b'=' as c_char;
    table[slot(b'j')] = b'*' as c_char;
    table[slot(b'k')] = b'+' as c_char;
    table[slot(b'l')] = b',' as c_char;
    table[slot(b'm')] = b'-' as c_char;
    table[slot(b'n')] = b'.' as c_char;
    table[slot(b'o')] = b'/' as c_char;
    table[slot(b'p')] = b'0' as c_char;
    table[slot(b'q')] = b'1' as c_char;
    table[slot(b'r')] = b'2' as c_char;
    table[slot(b's')] = b'3' as c_char;
    table[slot(b't')] = b'4' as c_char;
    table[slot(b'u')] = b'5' as c_char;
    table[slot(b'v')] = b'6' as c_char;
    table[slot(b'w')] = b'7' as c_char;
    table[slot(b'x')] = b'8' as c_char;
    table[slot(b'y')] = b'9' as c_char;
    table
};

/// Keys named by number in `CSI N ~`.
pub static CSI_FUNCS: [keyinfo; CSI_FUNC_COUNT] = {
    let mut table = [UNSET; CSI_FUNC_COUNT];
    table[1] = sym(TERMKEY_SYM_FIND);
    table[2] = sym(TERMKEY_SYM_INSERT);
    table[3] = sym(TERMKEY_SYM_DELETE);
    table[4] = sym(TERMKEY_SYM_SELECT);
    table[5] = sym(TERMKEY_SYM_PAGEUP);
    table[6] = sym(TERMKEY_SYM_PAGEDOWN);
    table[7] = sym(TERMKEY_SYM_HOME);
    table[8] = sym(TERMKEY_SYM_END);
    // 11..34 are the function keys, with three gaps (9, 10, 16, 22, 27, 30)
    // where historical terminals disagreed; upstream leaves those unclaimed.
    table[11] = func(1);
    table[12] = func(2);
    table[13] = func(3);
    table[14] = func(4);
    table[15] = func(5);
    table[17] = func(6);
    table[18] = func(7);
    table[19] = func(8);
    table[20] = func(9);
    table[21] = func(10);
    table[23] = func(11);
    table[24] = func(12);
    table[25] = func(13);
    table[26] = func(14);
    table[28] = func(15);
    table[29] = func(16);
    table[31] = func(17);
    table[32] = func(18);
    table[33] = func(19);
    table[34] = func(20);
    table
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csi_finals_and_ss3_commands_name_different_keys() {
        assert_eq!(CSI_SS3[slot(b'A')].sym, TERMKEY_SYM_UP);
        assert_eq!(CSI_SS3[slot(b'u')], UNSET);
        assert_eq!(SS3[slot(b'u')].sym, TERMKEY_SYM_KP5);
        assert_eq!(SS3[slot(b'A')], UNSET);
    }

    #[test]
    fn shift_tab_is_the_only_final_byte_carrying_a_modifier() {
        let with_mods: Vec<usize> = (0..64).filter(|&i| CSI_SS3[i].modifier_set != 0).collect();
        assert_eq!(with_mods, [slot(b'Z')]);
        assert_eq!(CSI_SS3[slot(b'Z')].sym, TERMKEY_SYM_TAB);
    }

    #[test]
    fn function_keys_carry_their_number_in_the_sym_field() {
        assert_eq!(CSI_SS3[slot(b'P')].type_0, TERMKEY_TYPE_FUNCTION);
        assert_eq!(CSI_SS3[slot(b'P')].sym, 1);
        assert_eq!(CSI_FUNCS[34], func(20));
    }

    #[test]
    fn the_gaps_in_the_numbered_keys_stay_unclaimed() {
        for gap in [0, 9, 10, 16, 22, 27, 30] {
            assert_eq!(CSI_FUNCS[gap], UNSET, "CSI {gap} ~");
        }
    }

    #[test]
    fn keypad_enter_has_no_plain_character_but_the_digits_do() {
        assert_eq!(SS3_KEYPAD_ALT[slot(b'M')], 0);
        assert_eq!(SS3_KEYPAD_ALT[slot(b'p')], b'0' as c_char);
        assert_eq!(SS3_KEYPAD_ALT[slot(b'y')], b'9' as c_char);
    }
}
