//! `vgetc` and the peek variants: one whole key out of the typeahead.
//!
//! [`vgetc`] is what the rest of the editor calls. It asks
//! [`crate::getchar::vgetorpeek`] for bytes and reassembles them
//! into a single key: a `K_SPECIAL` escape back into its key code, a modifier
//! prefix into `MOD_MASK`, a UTF-8 sequence into a codepoint.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::guard::{Allow, Keys};
use crate::keycodes::ModMask;
use crate::keycodes::{SpecialKey, key_unescape};
use crate::mbyte::MB_MAXBYTES;
use crate::mbyte::char_at;
use crate::register::RegName;
use crate::types::Key;

/// Fold `modifiers` into `c` where a single code stands for the combination.
///
/// Only Ctrl has such codes: `CTRL-A` is 0x01 rather than `<C-> A`. The
/// modifier is cleared out of `*modifiers` when it was folded in.
pub fn merge_modifiers(key: Key, modifiers: &mut ModMask) -> Key {
    if !modifiers.has(ModMask::CTRL) {
        return key;
    }
    // On the code rather than through `as_byte`: the mapping walk asks this
    // for every typeahead byte it compares under CTRL.
    let c = key.raw();
    let merged = if (Key::ascii(b'@')..=Key::DEL).contains(&key) {
        match c & 0x1f {
            0 => SpecialKey::Zero.code(),
            ctrl => ctrl,
        }
    } else if key.is('6') {
        // CTRL-6 is equivalent to CTRL-^.
        Key::CTRL_HAT.raw()
    } else {
        c
    };
    if merged != c {
        modifiers.clear(ModMask::CTRL);
    }
    Key::from_raw(merged)
}

/// [`merge_modifiers`] applied to the global `MOD_MASK`, which is where the
/// editor's own key handling keeps the pending modifiers.
pub(crate) fn merge_mod_mask(c: Key) -> Key {
    let mut modifiers = MOD_MASK.get();
    let merged = merge_modifiers(c, &mut modifiers);
    MOD_MASK.set(modifiers);
    merged
}

/// The next input character, which may be a special key or a multibyte one.
///
/// Answers `NUL` when called recursively — use [`safe_vgetc`] when that is
/// not wanted. Escaped `K_SPECIAL` bytes are translated back into their key
/// code and the bytes of a multibyte character are collected into the whole
/// character; the modifiers come back in the global `MOD_MASK`.
///
/// Safe: it only reads the editor's own globals, and may block waiting for
/// input. Re-entry into mappings and Lua callbacks is what the function is
/// for, so it guards against that itself rather than asking the caller to.
pub fn vgetc() -> Key {
    // Garbage collection was requested by a previous `garbagecollect()`
    // and we are back at the top level.
    if MAY_GARBAGE_COLLECT.get() && WANT_GARBAGE_COLLECT.get() {
        garbage_collect(false);
    }

    let mut c;
    if ungot_key_ready()
        && let Some(key) = UNGOT.take()
    {
        // A character `vungetc` put back has already been processed.
        c = key.c;
        MOD_MASK.set(key.mod_mask);
        MOUSE_GRID.set(key.mouse_grid);
        MOUSE_ROW.set(key.mouse_row);
        MOUSE_COL.set(key.mouse_col);
    } else {
        c = vgetc_from_typeahead();
    }

    // The main loop sets `MAY_GARBAGE_COLLECT` so that the next `vgetc`
    // collects; it is disabled again here so that Lists and Dicts in use
    // internally are not freed under a caller.
    MAY_GARBAGE_COLLECT.set(false);

    // Hand the key's bytes to the Lua on_key callbacks. The buffer is moved
    // out for the call: a callback can read keys, which append to it, and
    // all of that is cleared below anyway.
    let mut typed = ON_KEY_BUF.take();
    let discarded = nlua_execute_on_key(c, &mut typed);
    typed.clear();
    ON_KEY_BUF.set(typed);
    if discarded {
        // Keys following K_COMMAND/K_LUA/K_PASTE_START are not normally
        // seen by vim.on_key() callbacks, so drop them with this one.
        if c == SpecialKey::Command {
            drop(read_cmd_key_line());
        } else if c == SpecialKey::Lua {
            map_execute_lua(false, true);
        } else if c == SpecialKey::PasteStart {
            paste_repeat(0);
        }
        c = SpecialKey::Ignore.into();
    }
    ON_KEY_BUF.with_mut(Vec::clear);

    // The character has to be processed before anything else is safe.
    if c != SpecialKey::Ignore {
        // SAFETY: a NUL-terminated string literal.
        state_no_longer_safe(Some(c"key typed"));
    }

    c
}

/// One whole key from the typeahead, the half of [`vgetc`] that reads.
///
/// Safe: everything it touches is the editor's own typeahead state; it may
/// block waiting for input.
fn vgetc_from_typeahead() -> Key {
    /// One byte out of the typeahead.
    fn next_byte() -> u8 {
        vgetorpeek(true)
    }

    MOD_MASK.set(ModMask::NONE);
    VGETC_MOD_MASK.set(ModMask::NONE);
    VGETC_CHAR.set(Key::NUL);
    // `wrapping_sub` and not `-`: upstream's comment says
    // `LAST_RECORDED_LEN` can be *larger* than what the last `vgetc`
    // recorded, but it can also be smaller -- `ungetchars` shrinks it --
    // and C's defined unsigned wrap is what happens then. A huge value
    // makes `get_recorded`'s `len >= LAST_RECORDED_LEN` fail, so nothing
    // is trimmed off the recording, which is the upstream behaviour.
    // `test_registers`' Test_recording_with_select_mode reaches it.
    LAST_RECORDED_LEN.set(
        LAST_RECORDED_LEN
            .get()
            .wrapping_sub(LAST_VGETC_RECORDED_LEN.get()),
    );

    let c = loop {
        // No mapping once a modifier has been read.
        let raw_key = (!MOD_MASK.get().is_empty()).then(Keys::unmapped_with_codes);
        let lead = next_byte();
        drop(raw_key);

        let mut c = if lead == K_SPECIAL {
            // A special key is three bytes; get the other two.
            let unmapped = Keys::unmapped();
            let no_codes = Allow::no_key_codes(); // make sure BS is not found
            let second = next_byte();
            let third = next_byte();
            drop((unmapped, no_codes));
            if second == KS_MODIFIER {
                MOD_MASK.set(ModMask::from_byte(third));
                continue;
            }
            key_unescape(second, third)
        } else {
            Key::byte(lead)
        };

        // A multibyte character is as many bytes as its lead byte says.
        // This loops until every one of them has arrived.
        let n = c
            .as_byte()
            .map_or(1, |lead| usize::from(utf8len_tab[usize::from(lead)]));
        if n > 1 {
            let unmapped = Keys::unmapped();
            let mut buf = [0u8; MB_MAXBYTES + 1];
            buf[0] = lead;
            for byte in &mut buf[1..n] {
                *byte = next_byte();
                if *byte == K_SPECIAL {
                    // Must be K_SPECIAL KS_SPECIAL KE_FILLER, which
                    // stands for a literal 0x80.
                    next_byte(); // skip KS_SPECIAL
                    next_byte(); // skip KE_FILLER
                }
            }
            drop(unmapped);
            // The bytes are followed by the rest of the zeroed buffer.
            c = Key::from_code(char_at(&buf));
        }

        // When mappings are enabled (so not after i_CTRL-V) and the user
        // typed an unmapped <M-x>, read it as <Esc>x instead. #8213
        // Not in Terminal mode (#16202, #16220), and not for mouse keys:
        // terminals encode those as CSI sequences where ModMask::ALT
        // means something even unmapped.
        if NO_MAPPING.get() == 0
            && KEY_TYPED.get()
            && MOD_MASK.get() == ModMask::ALT
            && STATE.get() & MODE_TERMINAL == 0
            && !is_mouse_key(c)
        {
            MOD_MASK.set(ModMask::NONE);
            let len = ins_char_typebuf(c, ModMask::NONE, false);
            ins_char_typebuf(Key::ESC, ModMask::NONE, false);
            // K_SPECIAL KS_MODIFIER ModMask::ALT takes three more bytes.
            let old_len = len + 3;
            ungetchars(old_len);
            unsee_keys(old_len);
            continue;
        }

        if VGETC_CHAR.get().is_nul() {
            VGETC_MOD_MASK.set(MOD_MASK.get());
            VGETC_CHAR.set(c);
        }

        break keypad_equivalent(c);
    };

    LAST_VGETC_RECORDED_LEN.set(LAST_RECORDED_LEN.get());
    c
}

/// The ASCII or plain-key equivalent of an unmapped keypad or special
/// function key.
///
/// Reads and may clear the global `MOD_MASK`: `<S-Home>` has its own code, so
/// the shift is folded into the key rather than left in the mask.
///
/// Safe: it only reads and writes the editor's own `MOD_MASK`.
fn keypad_equivalent(c: Key) -> Key {
    let plain = match c.special() {
        Some(SpecialKey::Kplus) => return Key::ascii(b'+'),
        Some(SpecialKey::Kminus) => return Key::ascii(b'-'),
        Some(SpecialKey::Kdivide) => return Key::ascii(b'/'),
        Some(SpecialKey::Kmultiply) => return Key::ascii(b'*'),
        Some(SpecialKey::Kenter) => return Key::CAR,
        Some(SpecialKey::Kpoint) => return Key::ascii(b'.'),
        Some(SpecialKey::Kcomma) => return Key::ascii(b','),
        Some(SpecialKey::Kequal) => return Key::ascii(b'='),
        Some(SpecialKey::K0) => return Key::ascii(b'0'),
        Some(SpecialKey::K1) => return Key::ascii(b'1'),
        Some(SpecialKey::K2) => return Key::ascii(b'2'),
        Some(SpecialKey::K3) => return Key::ascii(b'3'),
        Some(SpecialKey::K4) => return Key::ascii(b'4'),
        Some(SpecialKey::K5) => return Key::ascii(b'5'),
        Some(SpecialKey::K6) => return Key::ascii(b'6'),
        Some(SpecialKey::K7) => return Key::ascii(b'7'),
        Some(SpecialKey::K8) => return Key::ascii(b'8'),
        Some(SpecialKey::K9) => return Key::ascii(b'9'),
        Some(SpecialKey::Xhome | SpecialKey::Zhome) => {
            modified_key(SpecialKey::SHome, SpecialKey::CHome, SpecialKey::Home)
        }
        Some(SpecialKey::Xend | SpecialKey::Zend) => {
            modified_key(SpecialKey::SEnd, SpecialKey::CEnd, SpecialKey::End)
        }
        Some(SpecialKey::Kup | SpecialKey::Xup) => SpecialKey::Up,
        Some(SpecialKey::Kdown | SpecialKey::Xdown) => SpecialKey::Down,
        Some(SpecialKey::Kleft | SpecialKey::Xleft) => SpecialKey::Left,
        Some(SpecialKey::Kright | SpecialKey::Xright) => SpecialKey::Right,
        _ => return c,
    };
    plain.into()
}

/// Pick the shifted, control or plain code for a key that has all three, and
/// take the modifier out of `MOD_MASK` when one was used.
///
/// Safe: it only reads and writes the editor's own `MOD_MASK`.
fn modified_key(shifted: SpecialKey, control: SpecialKey, plain: SpecialKey) -> SpecialKey {
    if MOD_MASK.get() == ModMask::SHIFT {
        MOD_MASK.set(ModMask::NONE);
        shifted
    } else if MOD_MASK.get() == ModMask::CTRL {
        MOD_MASK.set(ModMask::NONE);
        control
    } else {
        plain
    }
}

/// Like [`vgetc`], but never answers `NUL` when called recursively: it falls
/// back to reading a key straight from the user.
///
/// Safe: like [`vgetc`], it needs nothing of its caller but a running
/// editor; it may block waiting for input.
pub fn safe_vgetc() -> Key {
    let c = vgetc();
    if c.is_nul() { get_keystroke(None) } else { c }
}

/// Like [`safe_vgetc`], but loops past `K_IGNORE` and scrollbar events.
///
/// Safe: it is a loop around [`safe_vgetc`]; it may block waiting for input.
pub fn plain_vgetc() -> Key {
    loop {
        let c = safe_vgetc();
        if !matches!(
            c.special(),
            Some(
                SpecialKey::Ignore
                    | SpecialKey::VerScrollbar
                    | SpecialKey::HorScrollbar
                    | SpecialKey::Mousemove
            )
        ) {
            return c;
        }
    }
}

/// Whether a character is available, so that [`vgetc`] will not block.
///
/// Answers `NUL` when none is. When the next character is a special or
/// multibyte one the answer is only its first byte, which is not a valid key.
///
/// Safe: it only looks at the editor's own typeahead, and never blocks.
pub fn vpeekc() -> Key {
    match UNGOT.with(|key| key.as_ref().map(|key| key.c)) {
        Some(c) if ungot_key_ready() => c,
        _ => Key::byte(vgetorpeek(false)),
    }
}

/// Whether *any* character is available, half an escape sequence included.
///
/// The trick: when no typeahead is found but the typeahead buffer is not
/// empty, what is in it must be an ESC that was taken for the start of a key
/// code.
///
/// Safe: [`vpeekc`] plus a look at the typeahead buffer's length.
pub fn vpeekc_any() -> Key {
    let c = vpeekc();
    if c.is_nul() && !typeahead().is_empty() {
        Key::ESC
    } else {
        c
    }
}

/// [`vpeekc`] without letting anything be mapped.
///
/// Safe: [`vpeekc`] with mappings turned off for the duration.
pub fn char_avail() -> bool {
    if test_disable_char_avail.get() {
        return false;
    }
    let unmapped = Keys::unmapped();
    let c = vpeekc();
    drop(unmapped);
    !c.is_nul()
}

/// Put one character back, to be answered by the next [`vgetc`]. Can only be
/// done once.
///
/// A stuffed character comes back immediately; anything else waits until the
/// stuff buffer is empty.
pub(crate) fn vungetc(c: Key) {
    UNGOT.set(Some(UngotKey {
        c,
        mod_mask: MOD_MASK.get(),
        mouse_grid: MOUSE_GRID.get(),
        mouse_row: MOUSE_ROW.get(),
        mouse_col: MOUSE_COL.get(),
        stuffed: KEY_STUFFED.get(),
    }));
}

/// Clear `REG_EXECUTING` now, or arrange for it to be cleared.
///
/// While *peeking* the register is not finished with yet, so the flag is only
/// noted; the next advancing read acts on it.
pub fn check_end_reg_executing(advance: bool) {
    if REG_EXECUTING.get() != 0 && (typeahead().maplen() == 0 || PENDING_END_REG_EXECUTING.get()) {
        if advance {
            REG_EXECUTING.set(RegName::UNNAMED);
            PENDING_END_REG_EXECUTING.set(false);
        } else {
            PENDING_END_REG_EXECUTING.set(true);
        }
    }
}

/// Take the last `len` bytes back off the `vim.on_key()` buffer, for keys
/// that were put back into the typeahead rather than acted on.
fn unsee_keys(len: usize) {
    ON_KEY_BUF.with_mut(|buf| {
        if buf.len() >= len {
            let kept = buf.len() - len;
            buf.truncate(kept);
        }
    });
}
