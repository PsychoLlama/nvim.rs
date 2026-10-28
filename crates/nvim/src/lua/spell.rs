#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! `vim.spell`: the Lua binding over the spell checker.

use core::ffi::{CStr, c_int};

use crate::highlight_group::{HLF_COUNT, HLF_SPB, HLF_SPC, HLF_SPL, HLF_SPR};
use crate::lua::stack::{At, Kind, Raise, Stack};
use crate::lua_functions;
use crate::message::{e_no_spell, emsg};
use crate::optionstr::OptString;
use crate::os::cshim::gettext;
use crate::spell::{parse_spelllang, spell_check};
use crate::types::{Hlf, lua_Integer};
use crate::winlayer::Win;

lua_functions! {
    /// `vim.spell`.
    enum Spell in SPELL_FUNCTIONS {
        0 => c"check": spell_check_text,
    }
}

/// One misspelling `vim.spell.check` reports: the word, its kind, and the
/// 0-based byte offset it starts at.
struct Misspelling {
    word: Vec<u8>,
    kind: &'static CStr,
    at: usize,
}

/// `vim.spell.check(str)`: the misspellings in `str`, each as a
/// `{ word, kind, byte_index }` triple.
///
/// It reads `curwin` and turns 'spell' on for the duration.
fn spell_check_text(stack: &mut Stack) -> Result<c_int, Raise> {
    if stack.top().index() < 1 {
        return Err(Raise::message("Expected 1 argument"));
    }
    if stack.kind(At::arg(1)) != Kind::String {
        return Err(Raise::argument(1, "expected string"));
    }

    // spell.c insists 'spell' is on, so turn it on for the duration.
    let mut win = Win::current();
    let wo_spell_save = win.w_onebuf_opt.wo_spell;
    if wo_spell_save == 0 {
        let _ = parse_spelllang(win);
        win.w_onebuf_opt.wo_spell = 1;
    }
    if win.syn_block().b_p_spl.first_byte() == 0 {
        emsg(gettext(e_no_spell));
        win.w_onebuf_opt.wo_spell = wo_spell_save;
        return Ok(0);
    }

    let found = {
        // The text as the C string it was read as, terminator and all: the
        // argument stays in its slot, so its bytes outlive the walk.
        let text = stack
            .c_string(At::arg(1))
            .unwrap_or_default()
            .to_bytes_with_nul();
        let mut found = Vec::new();
        let mut pos = 0;
        let mut capcol: c_int = -1;
        // `spell_check` never steps past the terminator, so `pos` stays
        // inside the text.
        while text[pos] != 0 {
            let word = &text[pos..];
            let (len, attr): (usize, Hlf) = spell_check(win, word, Some(&mut capcol), false);
            if attr != HLF_COUNT {
                let kind: &CStr = match attr {
                    HLF_SPB => c"bad",
                    HLF_SPR => c"rare",
                    HLF_SPL => c"local",
                    HLF_SPC => c"caps",
                    _ => unreachable!("spell_check reported an unknown attribute"),
                };
                found.push(Misspelling {
                    word: word[..len].to_vec(),
                    kind,
                    at: pos,
                });
            }
            pos += len;
            capcol -= c_int::try_from(len).expect("a spell_check length fits an int");
        }
        found
    };

    stack.new_table(0, 0);
    for (n, misspelling) in (1..).zip(found) {
        // The triple is built on top of the results table and stored into
        // it, so the stack ends level.
        stack.new_table(3, 0);
        stack.push_bytes(&misspelling.word);
        stack.raw_set_index(At::from_top(2), 1);
        stack.push_bytes(misspelling.kind.to_bytes());
        stack.raw_set_index(At::from_top(2), 2);
        // +1 for Lua's 1-based indexing.
        stack.push_integer(lua_Integer::try_from(misspelling.at).unwrap_or(lua_Integer::MAX) + 1);
        stack.raw_set_index(At::from_top(2), 3);
        stack.raw_set_index(At::from_top(2), n);
    }

    // 'spell' goes back to what it was.
    win.w_onebuf_opt.wo_spell = wo_spell_save;
    Ok(1)
}

/// Build the `vim.spell` table and leave it on the stack.
pub(crate) fn push_spell(stack: &mut Stack) {
    stack.new_table(0, 0);
    stack.register_functions(SPELL_FUNCTIONS);
}
