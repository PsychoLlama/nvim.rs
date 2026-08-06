//! `vgetc` and the peek variants: one whole key out of the typeahead.
//!
//! [`vgetc`] is what the rest of the editor calls. It asks
//! [`crate::src::nvim::getchar::vgetorpeek`] for bytes and reassembles them
//! into a single key: a `K_SPECIAL` escape back into its key code, a modifier
//! prefix into `mod_mask`, a UTF-8 sequence into a codepoint.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::{
    K_C_END, K_C_HOME, K_COMMAND, K_IGNORE, K_LUA, K_MOUSEMOVE, key_unescape,
};
use crate::src::nvim::types::MB_MAXBYTES;
use core::ffi::c_int;
use core::ptr;

/// Fold `modifiers` into `c` where a single code stands for the combination.
///
/// Only Ctrl has such codes: `CTRL-A` is 0x01 rather than `<C-> A`. The
/// modifier is cleared out of `*modifiers` when it was folded in.
///
/// # Safety
/// `modifiers` must point at a writable `c_int`.
pub unsafe fn merge_modifiers(c_arg: c_int, modifiers: *mut c_int) -> c_int {
    unsafe {
        let mut c = c_arg;
        if *modifiers & MOD_MASK_CTRL != 0 {
            if c >= '@' as c_int && c <= 0x7f {
                c &= 0x1f;
                if c == NUL {
                    c = K_ZERO;
                }
            } else if c == '6' as c_int {
                // CTRL-6 is equivalent to CTRL-^.
                c = 0x1e;
            }
            if c != c_arg {
                *modifiers &= !MOD_MASK_CTRL;
            }
        }
        c
    }
}

/// The next input character, which may be a special key or a multibyte one.
///
/// Answers `NUL` when called recursively — use [`safe_vgetc`] when that is
/// not wanted. Escaped `K_SPECIAL` bytes are translated back into their key
/// code and the bytes of a multibyte character are collected into the whole
/// character; the modifiers come back in the global `mod_mask`.
///
/// # Safety
/// Callable at any time; may block waiting for input.
pub unsafe fn vgetc() -> c_int {
    unsafe {
        // Garbage collection was requested by a previous `garbagecollect()`
        // and we are back at the top level.
        if may_garbage_collect.get() && want_garbage_collect.get() {
            garbage_collect(false);
        }

        let mut c;
        if can_get_old_char() {
            // A character `vungetc` put back has already been processed.
            c = old_char.get();
            old_char.set(-1);
            mod_mask.set(old_mod_mask.get());
            mouse_grid.set(old_mouse_grid.get());
            mouse_row.set(old_mouse_row.get());
            mouse_col.set(old_mouse_col.get());
        } else {
            c = vgetc_from_typeahead();
        }

        // The main loop sets `may_garbage_collect` so that the next `vgetc`
        // collects; it is disabled again here so that Lists and Dicts in use
        // internally are not freed under a caller.
        may_garbage_collect.set(false);

        // Hand the key's bytes to the Lua on_key callbacks. The buffer is
        // reached through `ptr()` rather than `with_mut` throughout: a
        // callback can re-enter `vgetc`, and a live borrow across the call
        // would be exactly the overlap `GlobalCell` forbids.
        (*on_key_buf.ptr()).push(0);
        let discarded = nlua_execute_on_key(c, (*on_key_buf.ptr()).as_mut_ptr().cast());
        if discarded {
            // Keys following K_COMMAND/K_LUA/K_PASTE_START are not normally
            // seen by vim.on_key() callbacks, so drop them with this one.
            if c == K_COMMAND {
                xfree(getcmdkeycmd(NUL, ptr::null_mut(), 0, false).cast());
            } else if c == K_LUA {
                map_execute_lua(false, true);
            } else if c == K_PASTE_START {
                paste_repeat(0);
            }
            c = K_IGNORE;
        }
        (*on_key_buf.ptr()).clear();

        // The character has to be processed before anything else is safe.
        if c != K_IGNORE {
            state_no_longer_safe(c"key typed".as_ptr());
        }

        c
    }
}

/// One whole key from the typeahead, the half of [`vgetc`] that reads.
///
/// # Safety
/// Callable at any time; may block waiting for input.
unsafe fn vgetc_from_typeahead() -> c_int {
    unsafe {
        /// How many characters the last `vgetc` recorded. Peeking can record
        /// more, so `last_recorded_len` may have grown past it since.
        static last_vgetc_recorded_len: GlobalCell<usize> = GlobalCell::new(0);

        mod_mask.set(0);
        vgetc_mod_mask.set(0);
        vgetc_char.set(0);
        // `wrapping_sub` and not `-`: upstream's comment says
        // `last_recorded_len` can be *larger* than what the last `vgetc`
        // recorded, but it can also be smaller -- `ungetchars` shrinks it --
        // and C's defined unsigned wrap is what happens then. A huge value
        // makes `get_recorded`'s `len >= last_recorded_len` fail, so nothing
        // is trimmed off the recording, which is the upstream behaviour.
        // `test_registers`' Test_recording_with_select_mode reaches it.
        last_recorded_len.set(
            last_recorded_len
                .get()
                .wrapping_sub(last_vgetc_recorded_len.get()),
        );

        let c = loop {
            // No mapping once a modifier has been read.
            let did_inc = mod_mask.get() != 0;
            if did_inc {
                *no_mapping.ptr() += 1;
                *allow_keys.ptr() += 1;
            }
            let mut c = vgetorpeek(true);
            if did_inc {
                *no_mapping.ptr() -= 1;
                *allow_keys.ptr() -= 1;
            }

            // A special key is three bytes; get the other two.
            if c == K_SPECIAL {
                let save_allow_keys = allow_keys.get();
                *no_mapping.ptr() += 1;
                allow_keys.set(0); // make sure BS is not found
                let second = vgetorpeek(true);
                c = vgetorpeek(true);
                *no_mapping.ptr() -= 1;
                allow_keys.set(save_allow_keys);
                if second == KS_MODIFIER {
                    mod_mask.set(c);
                    continue;
                }
                c = key_unescape(second as u8, c as u8);
            }

            // A multibyte character is as many bytes as its lead byte says.
            // This loops until every one of them has arrived.
            let n = mb_byte2len_check(c);
            if n > 1 {
                *no_mapping.ptr() += 1;
                let mut buf = [0u8; MB_MAXBYTES + 1];
                buf[0] = c as u8;
                for byte in &mut buf[1..n] {
                    *byte = vgetorpeek(true) as u8;
                    if c_int::from(*byte) == K_SPECIAL {
                        // Must be K_SPECIAL KS_SPECIAL KE_FILLER, which
                        // stands for a literal 0x80.
                        vgetorpeek(true); // skip KS_SPECIAL
                        vgetorpeek(true); // skip KE_FILLER
                    }
                }
                *no_mapping.ptr() -= 1;
                c = utf_ptr2char(buf.as_ptr().cast());
            }

            // When mappings are enabled (so not after i_CTRL-V) and the user
            // typed an unmapped <M-x>, read it as <Esc>x instead. #8213
            // Not in Terminal mode (#16202, #16220), and not for mouse keys:
            // terminals encode those as CSI sequences where MOD_MASK_ALT
            // means something even unmapped.
            if no_mapping.get() == 0
                && KeyTyped.get()
                && mod_mask.get() == MOD_MASK_ALT
                && State.get() & MODE_TERMINAL == 0
                && !is_mouse_key(c)
            {
                mod_mask.set(0);
                let len = ins_char_typebuf(c, 0, false);
                ins_char_typebuf(ESC, 0, false);
                // K_SPECIAL KS_MODIFIER MOD_MASK_ALT takes three more bytes.
                let old_len = len + 3;
                ungetchars(old_len);
                let keys = on_key_buf.ptr();
                if (*keys).len() >= old_len as usize {
                    let kept = (*keys).len() - old_len as usize;
                    (*keys).truncate(kept);
                }
                continue;
            }

            if vgetc_char.get() == 0 {
                vgetc_mod_mask.set(mod_mask.get());
                vgetc_char.set(c);
            }

            break keypad_equivalent(c);
        };

        last_vgetc_recorded_len.set(last_recorded_len.get());
        c
    }
}

/// The ASCII or plain-key equivalent of an unmapped keypad or special
/// function key.
///
/// Reads and may clear the global `mod_mask`: `<S-Home>` has its own code, so
/// the shift is folded into the key rather than left in the mask.
///
/// # Safety
/// Callable at any time.
unsafe fn keypad_equivalent(c: c_int) -> c_int {
    match c {
        K_KPLUS => '+' as c_int,
        K_KMINUS => '-' as c_int,
        K_KDIVIDE => '/' as c_int,
        K_KMULTIPLY => '*' as c_int,
        K_KENTER => CAR,
        K_KPOINT => '.' as c_int,
        K_KCOMMA => ',' as c_int,
        K_KEQUAL => '=' as c_int,
        K_K0 => '0' as c_int,
        K_K1 => '1' as c_int,
        K_K2 => '2' as c_int,
        K_K3 => '3' as c_int,
        K_K4 => '4' as c_int,
        K_K5 => '5' as c_int,
        K_K6 => '6' as c_int,
        K_K7 => '7' as c_int,
        K_K8 => '8' as c_int,
        K_K9 => '9' as c_int,
        K_XHOME | K_ZHOME => unsafe { modified_key(K_S_HOME, K_C_HOME, K_HOME) },
        K_XEND | K_ZEND => unsafe { modified_key(K_S_END, K_C_END, K_END) },
        K_KUP | K_XUP => K_UP,
        K_KDOWN | K_XDOWN => K_DOWN,
        K_KLEFT | K_XLEFT => K_LEFT,
        K_KRIGHT | K_XRIGHT => K_RIGHT,
        _ => c,
    }
}

/// Pick the shifted, control or plain code for a key that has all three, and
/// take the modifier out of `mod_mask` when one was used.
///
/// # Safety
/// Callable at any time.
unsafe fn modified_key(shifted: c_int, control: c_int, plain: c_int) -> c_int {
    if mod_mask.get() == MOD_MASK_SHIFT {
        mod_mask.set(0);
        shifted
    } else if mod_mask.get() == MOD_MASK_CTRL {
        mod_mask.set(0);
        control
    } else {
        plain
    }
}

/// Like [`vgetc`], but never answers `NUL` when called recursively: it falls
/// back to reading a key straight from the user.
///
/// # Safety
/// Callable at any time; may block waiting for input.
pub unsafe fn safe_vgetc() -> c_int {
    unsafe {
        let c = vgetc();
        if c == NUL {
            get_keystroke(ptr::null_mut())
        } else {
            c
        }
    }
}

/// Like [`safe_vgetc`], but loops past `K_IGNORE` and scrollbar events.
///
/// # Safety
/// Callable at any time; may block waiting for input.
pub unsafe fn plain_vgetc() -> c_int {
    unsafe {
        loop {
            let c = safe_vgetc();
            if c != K_IGNORE && c != K_VER_SCROLLBAR && c != K_HOR_SCROLLBAR && c != K_MOUSEMOVE {
                return c;
            }
        }
    }
}

/// Whether a character is available, so that [`vgetc`] will not block.
///
/// Answers `NUL` when none is. When the next character is a special or
/// multibyte one the answer is only its first byte, which is not a valid key.
///
/// # Safety
/// Callable at any time.
pub unsafe fn vpeekc() -> c_int {
    unsafe {
        if can_get_old_char() {
            old_char.get()
        } else {
            vgetorpeek(false)
        }
    }
}

/// Whether *any* character is available, half an escape sequence included.
///
/// The trick: when no typeahead is found but the typeahead buffer is not
/// empty, what is in it must be an ESC that was taken for the start of a key
/// code.
///
/// # Safety
/// Callable at any time.
pub unsafe fn vpeekc_any() -> c_int {
    unsafe {
        let c = vpeekc();
        if c == NUL && (*typebuf.ptr()).tb_len > 0 {
            ESC
        } else {
            c
        }
    }
}

/// [`vpeekc`] without letting anything be mapped.
///
/// # Safety
/// Callable at any time.
pub unsafe fn char_avail() -> bool {
    unsafe {
        if test_disable_char_avail.get() {
            return false;
        }
        *no_mapping.ptr() += 1;
        let c = vpeekc();
        *no_mapping.ptr() -= 1;
        c != NUL
    }
}

/// Put one character back, to be answered by the next [`vgetc`]. Can only be
/// done once.
///
/// A stuffed character comes back immediately; anything else waits until the
/// stuff buffer is empty.
pub fn vungetc(c: c_int) {
    old_char.set(c);
    old_mod_mask.set(mod_mask.get());
    old_mouse_grid.set(mouse_grid.get());
    old_mouse_row.set(mouse_row.get());
    old_mouse_col.set(mouse_col.get());
    old_KeyStuffed.set(KeyStuffed.get());
}

/// Clear `reg_executing` now, or arrange for it to be cleared.
///
/// While *peeking* the register is not finished with yet, so the flag is only
/// noted; the next advancing read acts on it.
///
/// # Safety
/// Callable at any time.
pub unsafe fn check_end_reg_executing(advance: bool) {
    unsafe {
        if reg_executing.get() != 0
            && ((*typebuf.ptr()).tb_maplen == 0 || pending_end_reg_executing.get())
        {
            if advance {
                reg_executing.set(0);
                pending_end_reg_executing.set(false);
            } else {
                pending_end_reg_executing.set(true);
            }
        }
    }
}
