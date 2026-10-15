//! Gathering the editor's state into [`ShadaEntry`]s.
//!
//! One function per kind of thing that gets remembered: the buffer list, the
//! last search and substitute patterns, the registers, the numbered file
//! marks, the jump list. [`find_removable_bufs`] collects the buffers whose
//! marks are not to be written at all — the ones on removable media, per
//! `'shada'`'s `r` entries — which several of the others consult.
//!
//! Every entry built here holds copies of what it gathers, so the merge may
//! drop or replace any of them.
//!
//! The `shada_encode_*` entry points at the end pack one kind into a string
//! rather than into a file; the context stack uses them.

#![forbid(unsafe_code)]

use crate::buffer::{buf_is_quickfix, buf_is_terminal, find_buf};
use crate::cmdhist::{HIST_DEBUG, HIST_EXPR, HIST_INPUT};
use crate::eval::typval::{TV_INITIAL_VALUE, tv_copy};
use crate::eval::{var_flavour, vars::globvar_dict};
use crate::mark::{ShadaMark, cleanup_jumplist, setpcmark, shada_jumps};
use crate::memory::XString;
use crate::message_fmt::msg_addr;
use crate::os::time::os_time;
use crate::register::{op_reg_index, shada_registers};
use crate::search::shada_pattern;
use crate::siemsg;
use crate::types::{
    KeyDict__shada_search_pat, String_0, TypVal, VAR_FLAVOUR_DEFAULT, VAR_FLAVOUR_SESSION,
    VAR_FLAVOUR_SHADA, VAR_FUNC, VAR_PARTIAL, VarFlavour,
};
use crate::winlayer::{Buf, Win, buffers};
use core::ffi::{c_char, c_int};

use super::file::{get_shada_parameter, shada_removable};
use super::pack::{Packer, pack_entry};
use super::{
    DEFAULT_SEARCH_PATTERN, JUMPLISTSIZE, MOTION_BLOCKWISE, RemovableBufs, ShadaBufferListItem,
    ShadaEntry, ShadaEntryData, ShadaFileMark, ShadaGlobalVar, ShadaRegister, WriteMergerState,
    WriteStatus,
};
use crate::cmdhist::{HIST_CMD, HIST_SEARCH};

/// Whether a buffer's marks are not worth remembering: it has no file name,
/// it was unlisted on purpose, it is a quickfix or terminal buffer, or its
/// file is on removable media.
pub(crate) fn ignore_buf(buffer: Option<Buf>, removable_bufs: &RemovableBufs) -> bool {
    let Some(b) = buffer else {
        return true;
    };
    b.name.full().is_none()
        || (b.b_p_bl == 0 && b.b_p_initialized)
        || buf_is_quickfix(Some(b))
        || buf_is_terminal(Some(b))
        || removable_bufs.contains(&b.raw().cast_const())
}

/// Collect the buffers whose files are on removable media.
pub(crate) fn find_removable_bufs(removable_bufs: &mut RemovableBufs) {
    for buf in buffers() {
        if buf.name.full().is_some_and(shada_removable) {
            removable_bufs.insert(buf.raw().cast_const());
        }
    }
}

/// The buffer list, as one entry: every listed buffer worth remembering,
/// with the cursor position it was last left at.
///
/// `'shada'`'s `%` entry caps how many are kept; a negative cap means all
/// of them.
pub(crate) fn shada_get_buflist(removable_bufs: &RemovableBufs) -> ShadaEntry {
    let max_bufs = get_shada_parameter(c_int::from(b'%'));
    let mut wanted = Vec::new();
    for buf in buffers() {
        if !ignore_buf(Some(buf), removable_bufs)
            && buf.b_p_bl != 0
            && (max_bufs < 0 || wanted.len() < max_bufs as usize)
            && let Some(fname) = buf.name.full()
        {
            wanted.push(ShadaBufferListItem {
                pos: buf.b_last_cursor.mark,
                fname: XString::from_cstr(fname),
                extra: buf.additional_data.clone(),
            });
        }
    }
    ShadaEntry::new(os_time(), ShadaEntryData::BufferList(wanted))
}

/// One search pattern, as an entry, or `None` when there is none.
///
/// The same function serves the last search pattern and the last
/// `:substitute` pattern; `is_substitute_pattern` picks which, and decides
/// which fields are meaningful — a substitute pattern has no search offset,
/// so those fields keep their defaults and are not written at all.
///
/// `search_last_used` says which of the two was used most recently, so
/// exactly one of the pair is written with `is_last_used` set.
pub(crate) fn search_pattern_entry(
    is_substitute_pattern: bool,
    search_last_used: bool,
    search_highlighted: bool,
) -> Option<ShadaEntry> {
    let defaults = DEFAULT_SEARCH_PATTERN;
    let pat = shada_pattern(is_substitute_pattern)?;
    let last_used = is_substitute_pattern != search_last_used;
    // Every key is named here, so the entry a collector builds reads the
    // same way as one the parser filled in over its defaults.
    let data = ShadaEntryData::SearchPattern(KeyDict__shada_search_pat {
        magic: Some(pat.magic),
        smartcase: Some(!pat.no_scs),
        has_line_offset: Some(!is_substitute_pattern && pat.off.line),
        place_cursor_at_end: Some(!is_substitute_pattern && pat.off.end),
        is_last_used: Some(last_used),
        is_substitute_pattern: Some(is_substitute_pattern),
        highlighted: Some(last_used && search_highlighted),
        search_backward: Some(!is_substitute_pattern && pat.off.dir == b'?' as c_char),
        offset: if is_substitute_pattern {
            defaults.offset
        } else {
            Some(pat.off.off)
        },
        pat: Some(pat.pat),
    });
    Some(ShadaEntry {
        timestamp: pat.timestamp,
        data,
        extra: pat.extra,
    })
}

/// Every global register that has anything in it, into `wms`.
///
/// `max_reg_lines` is `'shada'`'s `<` (or `"`) entry: a register with more
/// lines than that is not remembered at all. A negative value means no
/// limit.
pub(crate) fn shada_initialize_registers(wms: &mut WriteMergerState, max_reg_lines: c_int) {
    let limit = usize::try_from(max_reg_lines).ok();
    for (name, is_unnamed, yank) in shada_registers(limit) {
        let entry = ShadaEntry {
            timestamp: yank.timestamp,
            data: ShadaEntryData::Register(ShadaRegister {
                name,
                motion: yank.motion,
                contents: yank.contents,
                is_unnamed,
                // Only a blockwise register has a width.
                width: if yank.motion == MOTION_BLOCKWISE {
                    yank.width as usize
                } else {
                    0
                },
            }),
            extra: yank.extra,
        };
        let idx = usize::try_from(op_reg_index(c_int::from(name)))
            .expect("a register the store answered has a slot");
        wms.registers[idx] = entry;
    }
}

/// Put a numbered mark at `idx`, pushing the ones from there down and
/// dropping the last.
///
/// The ten numbered marks are just the ten most recent, so their *names*
/// are their positions in the list: moving one renames all of them.
pub(crate) fn replace_numbered_mark(wms: &mut WriteMergerState, idx: usize, mut entry: ShadaEntry) {
    let marks = &mut wms.numbered_marks;
    let last = marks.len() - 1;
    for (i, mark) in marks.iter_mut().enumerate().take(last).skip(idx) {
        if let ShadaEntryData::GlobalMark(mark) = &mut mark.data {
            mark.name = (b'0' + i as u8 + 1) as c_char;
        }
    }
    super::merge::shift_within(marks, idx..last, idx + 1);
    entry.data.filemark_mut().name = (b'0' + idx as u8) as c_char;
    marks[idx] = entry;
}

/// The character `:history` names a history type by.
pub(crate) fn hist_type2char(type_0: c_int) -> c_int {
    let c = match type_0 {
        HIST_CMD => b':',
        HIST_SEARCH => b'/',
        HIST_EXPR => b'=',
        HIST_INPUT => b'@',
        HIST_DEBUG => b'>',
        _ => unreachable!("shada: history type {type_0} has no character"),
    };
    c_int::from(c)
}

/// The global variables whose names `'shada'` says to remember, walked in
/// the table's slot order.
///
/// A slot *index* rather than an item: the walk's position comes round again
/// between steps, and a pointer would not survive a mutation of the table.
pub(crate) struct VarWalk {
    next: usize,
    flavour: VarFlavour,
}

impl VarWalk {
    pub(crate) fn new(flavour: VarFlavour) -> VarWalk {
        VarWalk { next: 0, flavour }
    }
}

impl Iterator for VarWalk {
    /// The variable's name and a copy of its value.
    type Item = (XString, TypVal);

    fn next(&mut self) -> Option<(XString, TypVal)> {
        let globals = globvar_dict();
        let count = globals.dv_hashtab.size();
        while self.next < count {
            let idx = self.next;
            self.next += 1;
            let Some(item) = globals.item_at(idx) else {
                continue;
            };
            if var_flavour(item.di_key.bytes()) & self.flavour == 0 {
                continue;
            }
            let mut value = TV_INITIAL_VALUE;
            tv_copy(&item.di_tv, &mut value);
            return Some((XString::from_bytes(item.di_key.bytes()), value));
        }
        None
    }
}

/// The jump list, as entries, into `jumps`. Answers how many were written.
///
/// The current position is pushed onto the jump list first, so that where
/// Nvim was when it exited is remembered too.
pub(crate) fn shada_init_jumps(
    jumps: &mut [ShadaEntry; JUMPLISTSIZE],
    removable_bufs: &RemovableBufs,
) -> usize {
    setpcmark();
    cleanup_jumplist(Win::current(), false);
    let list = shada_jumps(Win::current());
    if list.is_empty() {
        // The iterator answers one unset mark for an empty list.
        mark_lnum_zero(0, 0);
    }
    let mut jumps_size = 0;
    let len = list.len();
    for (i, fm) in list.into_iter().enumerate() {
        if fm.mark.lnum == 0 {
            mark_lnum_zero(i + 1, len);
            continue;
        }
        let Some(fname) = jump_target(&fm, removable_bufs) else {
            continue;
        };
        jumps[jumps_size] = ShadaEntry {
            timestamp: fm.timestamp,
            data: ShadaEntryData::Jump(ShadaFileMark {
                name: 0,
                mark: fm.mark,
                fname,
            }),
            extra: fm.extra,
        };
        jumps_size += 1;
    }
    jumps_size
}

/// The internal error a jump with no line number raises: `next` is where
/// the walk would have resumed, as upstream's iterator token named it.
fn mark_lnum_zero(next: usize, len: usize) {
    let win = Win::current();
    let list = win.w_jumplist.as_ptr();
    let iter = if next < len {
        list.wrapping_add(next)
    } else {
        core::ptr::null()
    };
    siemsg!(
        "ShaDa: mark lnum zero (ji:{}, js:{}, len:{})",
        msg_addr(iter),
        msg_addr(list),
        win.w_jumplistlen,
    );
}

/// The file name to remember one jump under, or `None` if it is not worth
/// remembering: a buffer whose marks are ignored, a buffer number that names
/// no buffer, or no file name at all.
fn jump_target(fm: &ShadaMark, removable_bufs: &RemovableBufs) -> Option<XString> {
    if fm.fnum == 0 {
        // Not in a loaded buffer: the entry carries the name itself.
        return fm.fname.clone();
    }
    let buf = find_buf(fm.fnum);
    if ignore_buf(buf, removable_bufs) {
        return None;
    }
    buf?.name.full().map(XString::from_cstr)
}

/// The packed entries as the API string the context stack keeps.
fn packed_string(out: &Packer) -> String_0 {
    String_0::from_bytes(out.bytes())
}

/// Every register, as msgpack.
pub fn shada_encode_regs() -> String_0 {
    let mut wms = WriteMergerState::new();
    shada_initialize_registers(&mut wms, -1);
    let mut out = Packer::default();
    for reg in wms.registers.iter().filter(|reg| !reg.is_missing()) {
        let written = pack_entry(&mut out, reg, 0);
        assert!(
            written != WriteStatus::Error,
            "shada: cannot pack a register"
        );
    }
    packed_string(&out)
}

/// The jump list, as msgpack.
pub fn shada_encode_jumps() -> String_0 {
    let mut removable_bufs = crate::registry::id_set();
    find_removable_bufs(&mut removable_bufs);
    let mut jumps = [const { ShadaEntry::MISSING }; JUMPLISTSIZE];
    let jumps_size = shada_init_jumps(&mut jumps, &removable_bufs);
    let mut out = Packer::default();
    for jump in &jumps[..jumps_size] {
        let written = pack_entry(&mut out, jump, 0);
        assert!(written != WriteStatus::Error, "shada: cannot pack a jump");
    }
    packed_string(&out)
}

/// The buffer list, as msgpack.
pub fn shada_encode_buflist() -> String_0 {
    let mut removable_bufs = crate::registry::id_set();
    find_removable_bufs(&mut removable_bufs);
    let buflist_entry = shada_get_buflist(&removable_bufs);
    let mut out = Packer::default();
    let written = pack_entry(&mut out, &buflist_entry, 0);
    assert!(
        written != WriteStatus::Error,
        "shada: cannot pack the buffer list"
    );
    packed_string(&out)
}

/// Every global variable `'shada'` says to remember, as msgpack.
pub fn shada_encode_gvars() -> String_0 {
    let mut out = Packer::default();
    let cur_timestamp = os_time();
    let flavour = VAR_FLAVOUR_DEFAULT | VAR_FLAVOUR_SESSION | VAR_FLAVOUR_SHADA;
    for (name, value) in VarWalk::new(flavour) {
        // A function reference cannot be written to a file.
        if value.v_type() == VAR_FUNC || value.v_type() == VAR_PARTIAL {
            continue;
        }
        let entry = ShadaEntry::new(
            cur_timestamp,
            ShadaEntryData::Variable(ShadaGlobalVar { name, value }),
        );
        let written = pack_entry(&mut out, &entry, 0);
        assert!(
            written != WriteStatus::Error,
            "shada: cannot pack a variable"
        );
    }
    packed_string(&out)
}
