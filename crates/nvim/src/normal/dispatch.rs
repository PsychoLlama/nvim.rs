//! Turning a keystroke into a command: the table lookup, the count and
//! register that may precede it, the extra character some commands ask for,
//! and the bookkeeping that follows the handler.
//!
//! [`normal_execute`] is the whole of it, once per key. `find_command`
//! resolves a character to a row of `NV_CMD_TABLE`, `normal_get_command_count`
//! stacks the digits before it, and `normal_get_additional_char` reads the
//! second character a row asks for.
//!
//! Per-key path: `GlobalCell` through `get`/`set`/`ptr`, never `with`, and no
//! iterator adaptors or closures-as-helpers -- at opt-level 0 nothing inlines
//! but `#[inline(always)]`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::autocmd::state::DID_CURSORHOLD;
use crate::charset::vim_isprintc;
use crate::cstr;
use crate::digraph::get_digraph;
use crate::drawscreen::showmode;
use crate::drawscreen::state::{CLEAR_CMDLINE, MODE_DISPLAYED};
use crate::edit::edit;
use crate::eval::vars::{set_reg_var, set_vcount};
use crate::ex_docmd::do_sleep;
use crate::fold::fold_open_cursor;
use crate::getchar::state::{
    KEY_STUFFED, KEY_TYPED, LANGMAP_MAPCHAR, MOD_MASK, VGETC_BUSY, VGETC_CHAR, VGETC_MOD_MASK,
};
use crate::getchar::{
    beep_flush, gotchars_ignore, ins_char_typebuf, plain_vgetc, readbuf1_empty, stuff_empty,
    typeahead, ungetchars, vpeekc, vungetc,
};
use crate::guard::{Allow, Keys, Suppress};
use crate::keycodes::ModMask;
use crate::keycodes::{SpecialKey, simplify_mod_mask};
use crate::mapping::langmap_adjust_mb;
use crate::mark::checkpcmark;
use crate::mbyte::GraphemeState;
use crate::mbyte::{encode_char, iscomposing, utf8len_tab};
use crate::message::state::{MSG_COL, MSG_DIDOUT, MSG_NOWAIT};
use crate::normal::defs::{CmdArg, Outcome};
use crate::normal::{
    B_IMODE_LMAP, GRAPHEME_STATE_INIT, NV_CMD_TABLE, NV_CMDS, NV_CMDS_SIZE, NormalState, NvFlags,
    add_to_showcmd, check_text_or_curbuf_locked, clear_showcmd, del_from_showcmd,
    do_check_scrollbind, normal_handle_special_visual_command, normal_need_additional_char,
    normal_need_redraw_mode_message, normal_redraw_mode_message, set_vcount_ca, set_visual_select,
    start_selection, visual_active, visual_select,
};
use crate::ops::OpType;
use crate::ops::{Op, do_pending_operator, get_op_type};
use crate::option::defs::CpoFlag;
use crate::option::vars::{FDO_FLAGS, P_LANGMAP, p_lrm, p_tm, p_ttm};
use crate::os::libc::int64_t;
use crate::register::get_default_register_name;
use crate::state::Step;
use crate::state::mode::{
    FINISH_OP, KM_STARTSEL, MOTION_FORCE, OPCOUNT, RESTART_EDIT, RESTART_V_ISUAL_SELECT, STATE,
    V_ISUAL_SELECT_REG,
};
use crate::state::{
    MODE_LANGMAP, MODE_LREPLACE, MODE_NORMAL, MODE_NORMAL_BUSY, MODE_REPLACE, MODE_SELECT,
    get_real_state, may_trigger_modechanged,
};
use crate::types::Key;
use crate::types::OptInt;
use crate::ui::{ui_cursor_shape, ui_cursor_shape_no_check_conceal, ui_flush};
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_uint};

use crate::getchar::{
    append_to_redobuff, append_to_redobuff_char, append_to_redobuff_number, reset_redobuff,
};
use crate::r#move::{do_check_cursorbind, validate_cursor};
use crate::option::cpo_has;

/// Where a row sorts: the magnitude of its character, so that a special key
/// (stored negative) sorts after every character and `find_command` can
/// search for `-cmdchar`.
const fn rank(key: Key) -> usize {
    key.code().unsigned_abs() as usize
}

/// `NV_CMD_TABLE` in ascending `|cmd_char|` order, and how far a direct index
/// works.
///
/// Upstream builds this with `qsort` at startup, and then measures the
/// leading run where the `i`th smallest character *is* `i` — the control
/// characters are dense and start at NUL — so that `find_command` can index
/// straight in below that and only binary-search above it. The table is a
/// `const`, so both answers are: an insertion sort over 188 rows is nothing
/// to a `const fn`, and there is no startup pass and no `qsort` comparator
/// left to keep an `extern "C"` signature for.
const fn sorted_nv_cmds() -> ([usize; NV_CMDS_SIZE], usize) {
    let mut idx = [0; NV_CMDS_SIZE];
    let mut i = 0;
    while i < NV_CMDS_SIZE {
        idx[i] = i;
        i += 1;
    }
    // Insertion sort: `qsort` is not stable, so this leans on no two rows
    // sharing a character, which the assertion below holds them to.
    let mut i = 1;
    while i < NV_CMDS_SIZE {
        let mut j = i;
        while j > 0 && rank(NV_CMDS[idx[j - 1]].cmd_char) > rank(NV_CMDS[idx[j]].cmd_char) {
            let swap = idx[j - 1];
            idx[j - 1] = idx[j];
            idx[j] = swap;
            j -= 1;
        }
        i += 1;
    }
    // The run where the `i`th row answers to the character `i` (NUL first,
    // so there is always one).
    let mut linear = 0;
    while linear < NV_CMDS_SIZE
        && !NV_CMDS[idx[linear]].cmd_char.is_special()
        && linear == rank(NV_CMDS[idx[linear]].cmd_char)
    {
        linear += 1;
    }
    (idx, linear - 1)
}

/// [`sorted_nv_cmds`]'s two answers.
const NV_SORTED: ([usize; NV_CMDS_SIZE], usize) = sorted_nv_cmds();
const NV_CMD_IDX: [usize; NV_CMDS_SIZE] = NV_SORTED.0;
const NV_MAX_LINEAR: usize = NV_SORTED.1;

/// No two rows share a character: the sort above is stable and `find_command`
/// answers with the first match, so a duplicate would make which row wins
/// depend on the table's source order.
const _: () = {
    let mut i = 1;
    while i < NV_CMDS_SIZE {
        assert!(
            rank(NV_CMDS[NV_CMD_IDX[i - 1]].cmd_char) < rank(NV_CMDS[NV_CMD_IDX[i]].cmd_char),
            "two nv_cmds rows share a command character"
        );
        i += 1;
    }
};

/// The row of `NV_CMD_TABLE` that answers to `cmdchar`, if there is one.
pub(crate) fn find_command(cmdchar: Key) -> Option<usize> {
    if cmdchar.code() >= 0x100 {
        return None;
    }
    // A special key is stored negative and searched for by magnitude.
    let code = rank(cmdchar);
    const _: () = assert!(NV_MAX_LINEAR < NV_CMDS_SIZE);

    if code <= NV_MAX_LINEAR {
        return Some(NV_CMD_IDX[code]);
    }
    // `bot` starts past the linear run, so `i` is never 0.
    let mut bot = NV_MAX_LINEAR + 1;
    let mut top = NV_CMDS_SIZE - 1;
    while bot <= top {
        let i = (top + bot) / 2;
        let c = rank(NV_CMDS[NV_CMD_IDX[i]].cmd_char);
        if code == c {
            return Some(NV_CMD_IDX[i]);
        } else if code > c {
            bot = i + 1;
        } else {
            top = i - 1;
        }
    }
    None
}

/// Whether a key just read should be put through 'langmap'.
///
/// The three call sites differ only in the extra condition they add, and all
/// three of those conditions are pure reads, so passing one in as a `bool`
/// evaluates it where the C's `&&` chain would have short-circuited past it
/// without changing anything.
#[inline(always)]
fn langmap_wanted(condition: bool) -> bool {
    let have_langmap = P_LANGMAP.first_byte() != 0;
    let from_a_map = if VGETC_BUSY.get() != 0 {
        typeahead().maplen() == 0
    } else {
        KEY_TYPED.get()
    };
    have_langmap && condition && (p_lrm() || from_a_map) && !KEY_STUFFED.get()
}

/// Translate one key through 'langmap', if this key and this moment call for
/// it.
#[inline(always)]
pub(crate) fn langmap_adjust(c: &mut Key, condition: bool) {
    if !c.is_special() && langmap_wanted(condition) {
        *c = match c.as_byte() {
            Some(b) => Key::byte(LANGMAP_MAPCHAR.with(|map| map[usize::from(b)])),
            None => langmap_adjust_mb(*c),
        };
    }
}

/// Where `normal_get_additional_char` puts the character it reads.
///
/// The transpiled form is a `*mut c_int` aimed at one of two fields of `ca`,
/// and later compares that pointer against `&ca.extra_char` to decide what to
/// do next. Naming the two slots makes that test a `match` rather than a
/// pointer identity.
#[derive(PartialEq, Clone, Copy, Debug)]
enum Slot {
    /// Nothing more to read.
    None,
    /// `ca.nchar`: the ordinary "second character" of a command.
    NChar,
    /// `ca.extra_char`: the third, for `gr`, `g'`, `` g` `` and `g CTRL-\`.
    Extra,
}

/// Which slot, and in which mode, this command's extra character is read.
///
/// Answers `(slot, literal, replace)`. `literal` suppresses digraphs and
/// 'langmap'; `replace` puts the editor in Replace mode while waiting, so the
/// cursor shape says what is about to happen.
fn additional_char_slot(ns: &mut NormalState) -> (Slot, bool, bool) {
    if !ns.ca.cmdchar.is('g') {
        return (Slot::NChar, false, ns.ca.cmdchar.is('r'));
    }
    // `g` reads its own second character first, and only some of them
    // take a third.
    ns.ca.nchar = plain_vgetc();
    langmap_adjust(&mut ns.ca.nchar, true);
    ns.need_flushbuf |= add_to_showcmd(ns.ca.nchar);
    match ns.ca.nchar {
        c if c.is('r') => (Slot::Extra, false, true),
        c if c == Key::ascii(b'\'') || c.is('`') || c == Key::CTRL_BSL => {
            (Slot::Extra, true, false)
        }
        _ => (Slot::None, false, false),
    }
}

/// Read a `CTRL-\` follow-up, waiting up to 'ttimeoutlen' for it.
///
/// `CTRL-\ CTRL-N` and `CTRL-\ CTRL-G` are commands of their own; anything
/// else is put back for the next command to read.
fn resolve_ctrl_backslash(ns: &mut NormalState) {
    let mut towait = if p_ttm() >= 0 { p_ttm() } else { p_tm() };
    loop {
        ns.c = vpeekc();
        if ns.c.is_char() || towait <= 0 {
            break;
        }
        do_sleep(towait.min(50), false);
        towait -= 50;
    }
    if ns.c.is_char() {
        ns.c = plain_vgetc();
        if ns.c != Key::CTRL_N && ns.c != Key::CTRL_G {
            vungetc(ns.c);
        } else {
            ns.ca.cmdchar = Key::CTRL_BSL;
            ns.ca.nchar = ns.c;
            ns.idx = find_command(ns.ca.cmdchar);
            debug_assert!(ns.idx.is_some());
        }
    }
}

/// Collect the combining characters that belong with the character just read.
///
/// Only for a command flagged `NV_LANG` -- `f`, `t`, `r` and friends, where
/// the argument is a real character rather than a command key.
fn read_composing_tail(ns: &mut NormalState) {
    let mapped = Allow::mapping();
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    let mut prev_code = ns.ca.nchar.code();
    loop {
        ns.c = vpeekc();
        // Only a character that is not a lone byte can be composing.
        let multibyte = ns.c.code() >= 0x100
            || ns
                .c
                .as_byte()
                .is_some_and(|b| b != 0 && utf8len_tab[usize::from(b)] > 1);
        if !multibyte {
            break;
        }
        ns.c = plain_vgetc();
        if !iscomposing(prev_code, ns.c.code(), Some(&mut state)) {
            vungetc(ns.c);
            break;
        }
        // The base character is only encoded once a tail turns up.
        if ns.ca.nchar_len == 0 {
            ns.ca.nchar_len = encode_char(ns.ca.nchar.code(), &mut ns.ca.nchar_composing);
        }
        if ns.ca.nchar_len + ns.c.utf8_len() < ns.ca.nchar_composing.len() {
            let at = ns.ca.nchar_len;
            ns.ca.nchar_len += encode_char(ns.c.code(), &mut ns.ca.nchar_composing[at..]);
        }
        prev_code = ns.c.code();
    }
    let at = ns.ca.nchar_len;
    ns.ca.nchar_composing[at] = 0;
    drop(mapped);
    // The keys are recorded for a redo, not fed through undo syncing.
    let _no_sync = Suppress::undo_sync();
    gotchars_ignore();
}

/// Read the second (and sometimes third) character of a command.
pub(crate) fn normal_get_additional_char(ns: &mut NormalState) {
    let _raw_key = Keys::unmapped_with_codes();
    DID_CURSORHOLD.set(true);

    let (slot, lit, repl) = additional_char_slot(ns);
    let lang = repl || ns.row().cmd_flags.has(NvFlags::LANG);

    if slot != Slot::None {
        if repl {
            STATE.set(MODE_REPLACE);
            ui_cursor_shape_no_check_conceal();
        }
        // A language-mapped argument is read *with* mappings on, which is
        // the whole point of 'iminsert' being lmap.
        let langmap_active = lang && Buf::current().b_p_iminsert == OptInt::from(B_IMODE_LMAP);
        let mapped = langmap_active.then(Allow::mapping_with_codes);
        if langmap_active {
            STATE.set(if repl { MODE_LREPLACE } else { MODE_LANGMAP });
        }
        let mut cp = plain_vgetc();
        drop(mapped);
        STATE.set(MODE_NORMAL_BUSY);
        ns.need_flushbuf |= add_to_showcmd(cp);

        if !lit {
            // CTRL-K starts a digraph, unless 'cpoptions' says otherwise.
            if cp == Key::CTRL_K
                && (ns.row().cmd_flags.has(NvFlags::LANG) || slot == Slot::Extra)
                && !cpo_has(CpoFlag::DIGRAPH)
            {
                ns.c = get_digraph(false);
                if ns.c.is_char() {
                    cp = ns.c;
                    // Take the CTRL-K and its two characters back out of
                    // the echoed command before showing the result.
                    del_from_showcmd(3);
                    ns.need_flushbuf |= add_to_showcmd(cp);
                }
            }
            langmap_adjust(&mut cp, !lang);
        }
        // The character goes into the slot it was read for.
        match slot {
            Slot::NChar => ns.ca.nchar = cp,
            Slot::Extra => ns.ca.extra_char = cp,
            Slot::None => unreachable!(),
        }

        if slot == Slot::Extra
            && ns.ca.nchar == Key::CTRL_BSL
            && (ns.ca.extra_char == Key::CTRL_N || ns.ca.extra_char == Key::CTRL_G)
        {
            // `g CTRL-\ CTRL-N` is really `CTRL-\ CTRL-N`.
            ns.ca.cmdchar = Key::CTRL_BSL;
            ns.ca.nchar = ns.ca.extra_char;
            ns.idx = find_command(ns.ca.cmdchar);
        } else if (ns.ca.nchar.is('n') || ns.ca.nchar.is('N')) && ns.ca.cmdchar.is('g') {
            // `gn`/`gN` take the operator from the character after them.
            ns.ca.op().op_type = get_op_type(cp, Key::NUL);
        } else if cp == Key::CTRL_BSL {
            resolve_ctrl_backslash(ns);
        }

        if lang {
            read_composing_tail(ns);
        }
    }
}

/// Mirror a horizontal command for a right-to-left window.
pub(crate) fn normal_invert_horizontal(ns: &mut NormalState) {
    let c = ns.ca.cmdchar;
    ns.ca.cmdchar = match c.special() {
        Some(SpecialKey::Right) => Key::from(SpecialKey::Left),
        Some(SpecialKey::SRight) => Key::from(SpecialKey::SLeft),
        Some(SpecialKey::CRight) => Key::from(SpecialKey::CLeft),
        Some(SpecialKey::Left) => Key::from(SpecialKey::Right),
        Some(SpecialKey::SLeft) => Key::from(SpecialKey::SRight),
        Some(SpecialKey::CLeft) => Key::from(SpecialKey::CRight),
        _ => match c.as_byte() {
            Some(b'l') => Key::ascii(b'h'),
            Some(b'h') => Key::ascii(b'l'),
            Some(b'>') => Key::ascii(b'<'),
            Some(b'<') => Key::ascii(b'>'),
            _ => c,
        },
    };
    ns.idx = find_command(ns.ca.cmdchar);
}

/// Read the digits, and possibly the CTRL-W, in front of a command.
///
/// Answers whether a CTRL-W was consumed, in which case the caller loops:
/// `CTRL-W` takes a count of its own after it.
pub(crate) fn normal_get_command_count(ns: &mut NormalState) -> bool {
    // Select mode swallows printable keys as replacement text, digits too.
    if visual_active() && visual_select() {
        return false;
    }
    while ns.c.as_byte().is_some_and(|b| (b'1'..=b'9').contains(&b))
        || (ns.ca.count0 != 0
            && (ns.c == SpecialKey::Del || ns.c == SpecialKey::Kdel || ns.c.is('0')))
    {
        if ns.c == SpecialKey::Del || ns.c == SpecialKey::Kdel {
            ns.ca.count0 /= 10;
            // Four columns: <Del> is echoed as its key name.
            del_from_showcmd(4);
        } else if ns.ca.count0 > 99999999 {
            // Saturate rather than overflow; nine digits is the most a
            // count is ever allowed to be.
            ns.ca.count0 = 999999999;
        } else {
            ns.ca.count0 = ns.ca.count0 * 10 + c_int::from(ns.c.digit().unwrap_or(0));
        }
        if ns.toplevel && readbuf1_empty() {
            set_vcount_ca(&ns.ca, ns.set_prevcount);
            ns.set_prevcount = false;
        }
        let raw_key = ns.ctrl_w.then(Keys::unmapped_with_codes);
        // A '0' here is a count digit, not the "go to column 0" command,
        // so it must not be mapped.
        let count_digit = Suppress::zero_mapping();
        ns.c = plain_vgetc();
        langmap_adjust(&mut ns.c, true);
        drop(count_digit);
        drop(raw_key);
        ns.need_flushbuf |= add_to_showcmd(ns.c);
    }

    // CTRL-W takes the count read so far as its own, then a second count
    // for the window command after it.
    if ns.c == Key::CTRL_W && !ns.ctrl_w && ns.oa.op_type == OpType::Nop {
        ns.ctrl_w = true;
        ns.ca.opcount = ns.ca.count0;
        ns.ca.count0 = 0;
        let raw_key = Keys::unmapped_with_codes();
        ns.c = plain_vgetc();
        langmap_adjust(&mut ns.c, true);
        drop(raw_key);
        ns.need_flushbuf |= add_to_showcmd(ns.c);
        return true;
    }
    false
}

/// Everything that happens after the handler has run.
pub(crate) fn normal_finish_command(ns: &mut NormalState) {
    let mut did_visual_op = false;
    if !ns.command_finished {
        // A command that is not itself an operator, and does not claim
        // releases the register it was given.
        if !FINISH_OP.get()
            && ns.oa.op_type == OpType::Nop
            && ns
                .idx
                .is_none_or(|i| !NV_CMD_TABLE[i].cmd_flags.has(NvFlags::KEEPREG))
        {
            clear_op(ns.op());
            set_reg_var(get_default_register_name());
        }
        if ns.old_mapped_len > 0 {
            ns.old_mapped_len = typeahead().maplen();
        }
        if ns.ca.cmdchar != SpecialKey::Ignore && ns.ca.cmdchar != SpecialKey::Mousemove {
            did_visual_op =
                visual_active() && ns.oa.op_type != OpType::Nop && ns.oa.op_type != OpType::Colon;
            let old_col = ns.old_col;
            do_pending_operator(&mut ns.ca, old_col, false);
        }
        if normal_need_redraw_mode_message(ns) {
            normal_redraw_mode_message();
        }
    }
    MSG_NOWAIT.set(false);

    if FINISH_OP.get() || did_visual_op {
        set_reg_var(get_default_register_name());
    }
    let prev_finish_op = FINISH_OP.get();
    if ns.oa.op_type == OpType::Nop {
        FINISH_OP.set(false);
        may_trigger_modechanged();
    }
    // The cursor shape says whether an operator is pending, and `r`/`gr`
    // change it while they wait.
    if prev_finish_op || ns.ca.cmdchar.is('r') || (ns.ca.cmdchar.is('g') && ns.ca.nchar.is('r')) {
        ui_cursor_shape();
    }
    if ns.oa.op_type == OpType::Nop && ns.oa.regname == 0 && ns.ca.cmdchar != SpecialKey::Event {
        clear_showcmd();
    }
    checkpcmark();
    ns.ca.searchbuf = None;
    Win::current().snap_cursor_to_char();

    if Win::current().w_onebuf_opt.wo_scb != 0 && ns.toplevel {
        validate_cursor(Win::current());
        do_check_scrollbind(true);
    }
    if Win::current().w_onebuf_opt.wo_crb != 0 && ns.toplevel {
        validate_cursor(Win::current());
        do_check_cursorbind();
    }

    // A command may have asked for insert mode or for Select mode to be
    // resumed; neither happens until the command is completely done.
    let want_insert = !RESTART_EDIT.get().is_nul() && !visual_active() && ns.old_mapped_len == 0;
    if ns.oa.op_type == OpType::Nop
        && (want_insert || RESTART_V_ISUAL_SELECT.get() == 1)
        && !ns.ca.outcome.has(Outcome::COMMAND_BUSY)
        && stuff_empty()
        && ns.oa.regname == 0
    {
        if RESTART_V_ISUAL_SELECT.get() == 1 {
            set_visual_select(true);
            V_ISUAL_SELECT_REG.set(0);
            may_trigger_modechanged();
            showmode();
            RESTART_V_ISUAL_SELECT.set(0);
        }
        if want_insert {
            let _ = edit(RESTART_EDIT.get(), false, 1);
        }
    }
    // 2 means "next command", 1 means "this one"; the countdown is here.
    if RESTART_V_ISUAL_SELECT.get() == 2 {
        RESTART_V_ISUAL_SELECT.set(1);
    }
    OPCOUNT.set(ns.ca.opcount);
}

/// One normal-mode command, from its first key to the end of its effects.
pub(crate) fn normal_execute(ns: &mut NormalState, key: Key) -> Step {
    ns.command_finished = false;
    ns.ctrl_w = false;
    ns.old_col = Win::current().w_curswant.get();
    ns.c = key;
    langmap_adjust(&mut ns.c, get_real_state() != MODE_SELECT);

    if RESTART_EDIT.get().is_nul() {
        ns.old_mapped_len = 0;
    } else if ns.old_mapped_len != 0
        || (visual_active() && ns.mapped_len == 0 && typeahead().maplen() > 0)
    {
        ns.old_mapped_len = typeahead().maplen();
    }

    if ns.c.is_nul() {
        ns.c = Key::from(SpecialKey::Zero);
    }

    // In Select mode a printable key replaces the selection: the key is
    // put back for insert mode to read and the command becomes a change.
    if visual_active()
        && visual_select()
        && (vim_isprintc(ns.c.code())
            || ns.c == Key::NL
            || ns.c == Key::CAR
            || ns.c == SpecialKey::Kenter)
    {
        let len = ins_char_typebuf(VGETC_CHAR.get(), VGETC_MOD_MASK.get(), true);
        if KEY_TYPED.get() {
            ungetchars(len);
        }
        ns.c = if !RESTART_EDIT.get().is_nul() {
            Key::ascii(b'd')
        } else {
            Key::ascii(b'c')
        };
        MSG_NOWAIT.set(true);
        ns.old_mapped_len = 0;
    }

    ns.need_flushbuf = add_to_showcmd(ns.c);
    while normal_get_command_count(ns) {}

    if ns.c == SpecialKey::Event {
        // An event is not a command: the count it interrupted is stashed
        // for the real command that follows.
        ns.oa.prev_opcount = ns.ca.opcount;
        ns.oa.prev_count0 = ns.ca.count0;
    } else if ns.ca.opcount != 0 {
        // An operator count and a motion count multiply, saturating.
        if ns.ca.count0 != 0 {
            if ns.ca.opcount >= 999999999 / ns.ca.count0 {
                ns.ca.count0 = 999999999;
            } else {
                ns.ca.count0 *= ns.ca.opcount;
            }
        } else {
            ns.ca.count0 = ns.ca.opcount;
        }
    }
    ns.ca.opcount = ns.ca.count0;
    ns.ca.count1 = if ns.ca.count0 == 0 { 1 } else { ns.ca.count0 };
    if ns.toplevel && readbuf1_empty() {
        let (n0, n1) = (int64_t::from(ns.ca.count0), int64_t::from(ns.ca.count1));
        set_vcount(n0, n1, ns.set_prevcount);
    }

    if ns.ctrl_w {
        ns.ca.nchar = ns.c;
        ns.ca.cmdchar = Key::CTRL_W;
    } else {
        ns.ca.cmdchar = ns.c;
    }
    ns.idx = find_command(ns.ca.cmdchar);

    if ns.idx.is_none() {
        clear_op_beep(ns.op());
        ns.command_finished = true;
    } else if (ns.row().cmd_flags.has(NvFlags::NCW) && check_text_or_curbuf_locked(Some(ns.op())))
        || (visual_active() && normal_handle_special_visual_command(ns))
    {
        ns.command_finished = true;
    } else {
        if Win::current().w_onebuf_opt.wo_rl != 0
            && KEY_TYPED.get()
            && !KEY_STUFFED.get()
            && ns.row().cmd_flags.has(NvFlags::RL)
        {
            normal_invert_horizontal(ns);
        }
        if normal_need_additional_char(ns) {
            normal_get_additional_char(ns);
        }
        if ns.need_flushbuf {
            ui_flush();
        }
        if ns.ca.cmdchar != SpecialKey::Ignore && ns.ca.cmdchar != SpecialKey::Event {
            DID_CURSORHOLD.set(false);
        }
        STATE.set(MODE_NORMAL);

        if ns.ca.nchar == Key::ESC || ns.ca.extra_char == Key::ESC {
            clear_op(ns.op());
            ns.command_finished = true;
        } else {
            if ns.ca.cmdchar != SpecialKey::Ignore {
                MSG_DIDOUT.set(false);
                MSG_COL.set(0);
            }
            ns.old_pos = Win::current().w_cursor;

            // 'keymodel' startsel: a shifted special key starts a
            // selection and then acts as its unshifted self.
            if !visual_active() && KM_STARTSEL.get() {
                let flags = ns.row().cmd_flags;
                if flags.has(NvFlags::SS) {
                    start_selection();
                    unshift_special(&mut ns.ca);
                    ns.idx = find_command(ns.ca.cmdchar);
                    debug_assert!(ns.idx.is_some());
                } else if flags.has(NvFlags::SSS) && MOD_MASK.get().has(ModMask::SHIFT) {
                    start_selection();
                    MOD_MASK.set(MOD_MASK.get().without(ModMask::SHIFT));
                }
            }

            ns.ca.arg = ns.row().cmd_arg;
            (ns.row().cmd_func)(&mut ns.ca);
        }
    }
    normal_finish_command(ns);
    Step::Go
}

/// Record a command for `.`, taking its second character from `cmd_arg`.
pub(crate) fn prep_redo_cmd(cmd_arg: &mut CmdArg) {
    prep_redo(
        cmd_arg.op().regname,
        cmd_arg.count0,
        Key::NUL,
        cmd_arg.cmdchar,
        Key::NUL,
        Key::NUL,
        Key::NUL,
    );
    // A character with a combining tail is replayed as its whole encoding.
    if cmd_arg.nchar_len > 0 {
        append_to_redobuff(cstr::until_nul(&cmd_arg.nchar_composing));
    } else {
        append_to_redobuff_char(cmd_arg.nchar);
    }
}

/// Record a command for `.`: an optional register, a count, and up to five
/// command characters.
pub(crate) fn prep_redo(
    regname: u8,
    num: c_int,
    cmd1: Key,
    cmd2: Key,
    cmd3: Key,
    cmd4: Key,
    cmd5: Key,
) {
    prep_redo_num2(regname, num, cmd1, cmd2, 0, cmd3, cmd4, cmd5);
}

/// As [`prep_redo`], with a second count between the second and third
/// command characters -- which is what `z<n><CR>` and the `[count]` forms of
/// the `Z` commands need.
#[allow(clippy::too_many_arguments)]
pub(crate) fn prep_redo_num2(
    regname: u8,
    num1: c_int,
    cmd1: Key,
    cmd2: Key,
    num2: c_int,
    cmd3: Key,
    cmd4: Key,
    cmd5: Key,
) {
    reset_redobuff();
    if regname != 0 {
        append_to_redobuff_char(Key::ascii(b'"'));
        append_to_redobuff_char(Key::byte(regname));
    }
    if num1 != 0 {
        append_to_redobuff_number(num1);
    }
    for cmd in [cmd1, cmd2] {
        if !cmd.is_nul() {
            append_to_redobuff_char(cmd);
        }
    }
    if num2 != 0 {
        append_to_redobuff_number(num2);
    }
    for cmd in [cmd3, cmd4, cmd5] {
        if !cmd.is_nul() {
            append_to_redobuff_char(cmd);
        }
    }
}

// A live operator is all the `clear*` entry points need, and [`Op`] already
// carries that promise, so they are safe functions. A caller holding the
// state machine's frame asks it for the operator with
// [`NormalState::op`](crate::normal::NormalState::op).

/// Beep and clear the operator if one is pending. Answers whether it was.
pub(crate) fn check_clear_op(op: Op) -> bool {
    if op.op_type == OpType::Nop {
        return false;
    }
    clear_op_beep(op);
    true
}

/// As [`check_clear_op`], and also refuse while a Visual selection is up --
/// for commands that make no sense applied to one.
pub(crate) fn check_clear_op_quit(op: Op) -> bool {
    if op.op_type == OpType::Nop && !visual_active() {
        return false;
    }
    clear_op_beep(op);
    true
}

/// Forget the pending operator, its register and its forced motion kind.
pub(crate) fn clear_op(mut op: Op) {
    op.op_type = OpType::Nop;
    op.regname = 0;
    op.motion_force = Key::NUL;
    op.use_reg_one = false;
    MOTION_FORCE.set(Key::NUL);
}

/// [`clear_op`], and say so.
///
/// The beep also flushes the typeahead, which is what makes a failed command
/// abandon the rest of a mapping or a `:normal` argument.
pub(crate) fn clear_op_beep(op: Op) {
    clear_op(op);
    beep_flush();
}

/// Read one more key for a command that takes several, with mappings and
/// 'langmap' handled the way a command character wants them, and show it in
/// the 'showcmd' area.
pub(crate) fn read_command_char() -> Key {
    let raw_key = Keys::unmapped_with_codes();
    let mut c = plain_vgetc();
    langmap_adjust(&mut c, true);
    drop(raw_key);
    add_to_showcmd(c);
    c
}

/// Open a fold the cursor has landed in, if the 'foldopen' flag for this kind
/// of movement is set, the key was typed rather than mapped, and no operator
/// is waiting for the motion to finish.
pub(crate) fn may_fold_open(cmd_arg: &mut CmdArg, fdo_flag: c_uint) {
    if FDO_FLAGS.get() & fdo_flag != 0 && KEY_TYPED.get() && cmd_arg.op().op_type == OpType::Nop {
        fold_open_cursor();
    }
}

/// Turn a shifted special key into its unshifted self.
pub(crate) fn unshift_special(cmd_arg: &mut CmdArg) {
    let unshifted = match cmd_arg.cmdchar.special() {
        Some(SpecialKey::SRight) => Key::from(SpecialKey::Right),
        Some(SpecialKey::SLeft) => Key::from(SpecialKey::Left),
        Some(SpecialKey::SUp) => Key::from(SpecialKey::Up),
        Some(SpecialKey::SDown) => Key::from(SpecialKey::Down),
        Some(SpecialKey::SHome) => Key::from(SpecialKey::Home),
        Some(SpecialKey::SEnd) => Key::from(SpecialKey::End),
        _ => cmd_arg.cmdchar,
    };
    cmd_arg.cmdchar = simplify_mod_mask(unshifted);
}

/// Make room on the command line for whatever is about to be shown there.
pub(crate) fn may_clear_cmdline() {
    if MODE_DISPLAYED.get() {
        // The mode message is there; let the redraw take it away.
        CLEAR_CMDLINE.set(true);
    } else {
        clear_showcmd();
    }
}
