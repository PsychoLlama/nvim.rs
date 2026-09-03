//! Gathering the editor's state into [`ShadaEntry`]s.
//!
//! One function per kind of thing that gets remembered: the buffer list, the
//! last search and substitute patterns, the registers, the numbered file
//! marks, the jump list. [`find_removable_bufs`] collects the buffers whose
//! marks are not to be written at all — the ones on removable media, per
//! `'shada'`'s `r` entries — which several of the others consult.
//!
//! Nothing here owns what it gathers: an entry built by this file points
//! straight into the editor's own structures, and says so with
//! `can_free_entry: false`.
//!
//! The `shada_encode_*` entry points at the end pack one kind into a string
//! rather than into a file; the msgpack API uses them.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::msg_addr;
use crate::siemsg;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;

use crate::cmdhist::{HIST_DEBUG, HIST_EXPR, HIST_INPUT};

use super::*;
use crate::types::{
    NUL, VAR_FLAVOUR_DEFAULT, VAR_FLAVOUR_SESSION, VAR_FLAVOUR_SHADA, VAR_FUNC, VAR_PARTIAL,
};

/// Whether a buffer's marks are not worth remembering: it has no file name,
/// it was unlisted on purpose, it is a quickfix or terminal buffer, or its
/// file is on removable media.
pub(crate) unsafe fn ignore_buf(buf: *const buf_T, removable_bufs: &RemovableBufs) -> bool {
    // SAFETY: the caller's promise -- null or a live buffer.
    let Some(b) = (unsafe { Buf::from_raw(buf.cast_mut()) }) else {
        return true;
    };
    b.b_ffname.is_null()
        || (b.b_p_bl == 0 && b.b_p_initialized)
        || buf_is_quickfix(Some(b))
        || buf_is_terminal(Some(b))
        // SAFETY: the caller's set, and the buffer is only compared.
        || removable_bufs.contains(&buf)
}

/// Collect the buffers whose files are on removable media.
pub(crate) unsafe fn find_removable_bufs(removable_bufs: &mut RemovableBufs) {
    for buf in buffers() {
        // SAFETY: a live buffer from the editor's own list, whose name is
        // only read, and the caller's set.
        if !buf.b_ffname.is_null() && unsafe { shada_removable(buf.b_ffname) } {
            removable_bufs.insert(buf.raw().cast_const());
        }
    }
}

/// The buffer list, as one entry: every listed buffer worth remembering,
/// with the cursor position it was last left at.
///
/// `'shada'`'s `%` entry caps how many are kept; a negative cap means all
/// of them.
pub(crate) unsafe fn shada_get_buflist(removable_bufs: &RemovableBufs) -> ShadaEntry {
    let max_bufs = unsafe { get_shada_parameter('%' as c_int) };
    let mut wanted = Vec::new();
    for buf in buffers() {
        if !unsafe { ignore_buf(buf.raw(), removable_bufs) }
            && buf.b_p_bl != 0
            && (max_bufs < 0 || wanted.len() < max_bufs as usize)
        {
            wanted.push(buffer_list_buffer {
                pos: buf.b_last_cursor.mark,
                fname: buf.b_ffname,
                additional_data: buf.additional_data,
            });
        }
    }

    // The array is `xmalloc`ed because the caller releases it with
    // `xfree`, as it does for a buffer list that came off the wire.
    let buffers = unsafe { xmalloc(size_of_val(&wanted[..])) }.cast::<buffer_list_buffer>();
    unsafe { buffers.copy_from_nonoverlapping(wanted.as_ptr(), wanted.len()) };
    ShadaEntry {
        can_free_entry: false,
        timestamp: os_time(),
        data: ShadaEntryData::BufferList(buffer_list {
            size: wanted.len(),
            buffers,
        }),
        additional_data: core::ptr::null_mut(),
    }
}

/// One search pattern, as an entry.
///
/// The same function serves the last search pattern and the last
/// `:substitute` pattern; `is_substitute_pattern` picks which, and decides
/// which fields are meaningful — a substitute pattern has no search offset,
/// so those fields keep their defaults and are not written at all.
///
/// `search_last_used` says which of the two was used most recently, so
/// exactly one of the pair is written with `is_last_used` set.
pub(crate) unsafe fn add_search_pattern(
    ret_pse: *mut ShadaEntry,
    get_pattern: SearchPatternGetter,
    is_substitute_pattern: bool,
    search_last_used: bool,
    search_highlighted: bool,
) {
    let defaults = DEFAULT_SEARCH_PATTERN;
    let mut pat: SearchPattern = unsafe { core::mem::zeroed() };
    unsafe { get_pattern.expect("non-null function pointer")(&raw mut pat) };
    if pat.pat.is_null() {
        return;
    }
    let last_used = is_substitute_pattern != search_last_used;
    let entry = ShadaEntry {
        can_free_entry: false,
        timestamp: pat.timestamp,
        data: ShadaEntryData::SearchPattern(KeyDict__shada_search_pat {
            is_set___shada_search_pat_: 0,
            magic: pat.magic,
            smartcase: !pat.no_scs,
            has_line_offset: !is_substitute_pattern && pat.off.line,
            place_cursor_at_end: !is_substitute_pattern && pat.off.end,
            is_last_used: last_used,
            is_substitute_pattern,
            highlighted: last_used && search_highlighted,
            search_backward: !is_substitute_pattern && pat.off.dir as c_int == '?' as c_int,
            offset: if is_substitute_pattern {
                defaults.offset
            } else {
                pat.off.off as Integer
            },
            pat: unsafe { cstr_as_string(pat.pat) },
        }),
        additional_data: pat.additional_data,
    };
    unsafe { *ret_pse = entry };
    // The two substitute-pattern defaults are `false`, which is what the
    // `!is_substitute_pattern &&` above produces; assert it rather than
    // spelling the branch out twice.
    debug_assert!(
        !defaults.has_line_offset && !defaults.place_cursor_at_end,
        "shada: a search pattern's offset defaults are not false"
    );
}

/// Every global register that has anything in it, as entries.
///
/// `max_reg_lines` is `'shada'`'s `<` (or `"`) entry: a register with more
/// lines than that is not remembered at all. A negative value means no
/// limit.
pub(crate) unsafe fn shada_initialize_registers(wms: *mut WriteMergerState, max_reg_lines: c_int) {
    let mut reg_iter = core::ptr::null::<c_void>();
    loop {
        let mut reg: yankreg_T = unsafe { core::mem::zeroed() };
        let mut name: c_char = NUL as c_char;
        let mut is_unnamed = false;
        reg_iter = unsafe {
            op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed)
        };
        if name as c_int == NUL {
            return;
        }
        let too_long = max_reg_lines >= 0 && reg.y_size > max_reg_lines as size_t;
        if !too_long {
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp: reg.timestamp,
                data: ShadaEntryData::Register(reg {
                    name,
                    type_0: reg.y_type,
                    contents: reg.y_array,
                    is_unnamed,
                    contents_size: reg.y_size,
                    // Only a blockwise register has a width.
                    width: if reg.y_type == kMTBlockWise {
                        reg.y_width as size_t
                    } else {
                        0
                    },
                }),
                additional_data: reg.additional_data,
            };
            unsafe { (*wms).registers[op_reg_index(name as c_int) as usize] = entry };
        }
        if reg_iter.is_null() {
            return;
        }
    }
}

/// Put a numbered mark at `idx`, pushing the ones from there down and
/// dropping the last.
///
/// The ten numbered marks are just the ten most recent, so their *names*
/// are their positions in the list: moving one renames all of them.
pub(crate) unsafe fn replace_numbered_mark(
    wms: *mut WriteMergerState,
    idx: size_t,
    entry: ShadaEntry,
) {
    let marks = unsafe { &mut (*wms).numbered_marks };
    let last = marks.len() - 1;
    unsafe { shada_free_shada_entry(&raw mut marks[last]) };
    for (i, mark) in marks.iter_mut().enumerate().take(last).skip(idx) {
        if let ShadaEntryData::GlobalMark(mark) = &mut mark.data {
            mark.name = (b'0' + i as u8 + 1) as c_char;
        }
    }
    marks.copy_within(idx..last, idx + 1);
    marks[idx] = entry;
    marks[idx].data.filemark_mut().name = (b'0' + idx as u8) as c_char;
}

/// The character `:history` names a history type by.
pub(crate) fn hist_type2char(type_0: c_int) -> c_int {
    match type_0 {
        HIST_CMD => ':' as c_int,
        HIST_SEARCH => '/' as c_int,
        HIST_EXPR => '=' as c_int,
        HIST_INPUT => '@' as c_int,
        HIST_DEBUG => '>' as c_int,
        _ => unreachable!("shada: history type {type_0} has no character"),
    }
}

/// Walk the global variables whose names `'shada'` says to remember.
///
/// Answers what to pass in next, or null when the walk is over; `name` is
/// set to null at the end. `rettv` gets a copy of the value, which the
/// caller clears.
///
/// # Safety
/// No global variable may be added or removed while a walk is in progress.
pub(crate) unsafe fn var_shada_iter(
    iter: Option<usize>,
    name: *mut *const c_char,
    rettv: *mut typval_T,
    flavour: var_flavour_T,
) -> Option<usize> {
    let globvarht = get_globvar_ht();
    let count = unsafe { (*globvarht).size() };
    // The walk's position is a slot *index*: it is handed back to the caller
    // and comes round again, and the table's small run lives inside the
    // table, so a pointer would not survive a mutation of it.
    let wanted = |idx: usize| {
        let hi = unsafe { (*globvarht).slot(idx) };
        hi.is_kept() && unsafe { var_flavour(hi.hi_key) } & flavour != 0
    };

    unsafe { *name = core::ptr::null() };
    let mut idx = match iter {
        Some(idx) => idx,
        None => (0..count).find(|&idx| wanted(idx))?,
    };

    let key = unsafe { (*globvarht).slot(idx) }.hi_key;
    let di = unsafe { key.sub(offset_of!(dictitem_T, di_key)) } as *mut dictitem_T;
    unsafe { *name = &raw mut (*di).di_key as *mut c_char };
    unsafe { tv_copy(&raw mut (*di).di_tv, rettv) };

    // Answer where the *next* one is, so the caller knows to stop.
    loop {
        idx += 1;
        if idx >= count {
            return None;
        }
        if wanted(idx) {
            return Some(idx);
        }
    }
}

/// The jump list, as entries, into `jumps`. Answers how many were written.
///
/// The current position is pushed onto the jump list first, so that where
/// Nvim was when it exited is remembered too.
pub(crate) unsafe fn shada_init_jumps(
    jumps: *mut ShadaEntry,
    removable_bufs: &RemovableBufs,
) -> size_t {
    let mut jumps_size: size_t = 0;
    let mut jump_iter = core::ptr::null::<c_void>();
    setpcmark();
    unsafe { cleanup_jumplist(curwin.get(), false) };
    loop {
        let mut fm: xfmark_T = unsafe { core::mem::zeroed() };
        jump_iter = unsafe { mark_jumplist_iter(jump_iter, curwin.get(), &raw mut fm) };

        if let Some(fname) = unsafe { jump_target(&fm, jump_iter, removable_bufs) } {
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp: fm.fmark.timestamp,
                data: ShadaEntryData::Jump(shada_filemark {
                    name: NUL as c_char,
                    mark: fm.fmark.mark,
                    fname: fname.cast_mut(),
                }),
                additional_data: fm.fmark.additional_data,
            };
            unsafe { *jumps.add(jumps_size) = entry };
            jumps_size += 1;
        }
        if jump_iter.is_null() {
            return jumps_size;
        }
    }
}

/// The file name to remember one jump under, or `None` if it is not worth
/// remembering: no line number, a buffer whose marks are ignored, a buffer
/// number that names no buffer, or no file name at all.
unsafe fn jump_target(
    fm: &xfmark_T,
    jump_iter: *const c_void,
    removable_bufs: &RemovableBufs,
) -> Option<*const c_char> {
    if fm.fmark.mark.lnum == 0 {
        // SAFETY: the caller's contract -- `curwin` is the editor's own.
        unsafe {
            siemsg!(
                "ShaDa: mark lnum zero (ji:{}, js:{}, len:{})",
                msg_addr(jump_iter),
                msg_addr(&raw const (*curwin.get()).w_jumplist),
                (*curwin.get()).w_jumplistlen,
            )
        };
        return None;
    }
    if fm.fmark.fnum == 0 {
        // Not in a loaded buffer: the entry carries the name itself.
        return (!fm.fname.is_null()).then_some(fm.fname as *const c_char);
    }
    let buf = find_buf(fm.fmark.fnum).map_or(core::ptr::null_mut(), |mut b| b.raw());
    if buf.is_null()
        || unsafe { ignore_buf(buf, removable_bufs) }
        || unsafe { (*buf).b_ffname.is_null() }
    {
        return None;
    }
    Some(unsafe { (*buf).b_ffname })
}

/// Every register, as msgpack.
pub unsafe fn shada_encode_regs() -> String_0 {
    let wms = shada_heap(WriteMergerState::EMPTY);
    unsafe { shada_initialize_registers(wms, -1) };
    let mut packer = packer_string_buffer();
    for i in 0..unsafe { (*wms).registers.len() } {
        if !unsafe { (*wms).registers[i].data.is_missing() } {
            let written =
                unsafe { shada_pack_pfreed_entry(&raw mut packer, (*wms).registers[i], 0) };
            assert!(written != kSDWriteFailed, "shada: cannot pack a register");
        }
    }
    unsafe { xfree(wms.cast::<c_void>()) };
    packer_take_string(&packer)
}

/// The jump list, as msgpack.
pub unsafe fn shada_encode_jumps() -> String_0 {
    let mut removable_bufs = id_set();
    unsafe { find_removable_bufs(&mut removable_bufs) };
    let mut jumps = [ShadaEntry::MISSING; JUMPLISTSIZE as usize];
    let jumps_size = unsafe { shada_init_jumps(jumps.as_mut_ptr(), &removable_bufs) };
    let mut packer = packer_string_buffer();
    for jump in &jumps[..jumps_size] {
        let written = unsafe { shada_pack_pfreed_entry(&raw mut packer, *jump, 0) };
        assert!(written != kSDWriteFailed, "shada: cannot pack a jump");
    }
    packer_take_string(&packer)
}

/// The buffer list, as msgpack.
pub unsafe fn shada_encode_buflist() -> String_0 {
    let mut removable_bufs = id_set();
    unsafe { find_removable_bufs(&mut removable_bufs) };
    let buflist_entry = unsafe { shada_get_buflist(&removable_bufs) };
    let mut packer = packer_string_buffer();
    let written = unsafe { shada_pack_entry(&raw mut packer, buflist_entry, 0) };
    assert!(
        written != kSDWriteFailed,
        "shada: cannot pack the buffer list"
    );
    unsafe { xfree(buflist_entry.data.buffer_list().buffers.cast::<c_void>()) };
    packer_take_string(&packer)
}

/// Every global variable `'shada'` says to remember, as msgpack.
pub unsafe fn shada_encode_gvars() -> String_0 {
    let mut packer = packer_string_buffer();
    let mut var_iter: Option<usize> = None;
    let cur_timestamp = os_time();
    loop {
        let mut vartv: typval_T = unsafe { core::mem::zeroed() };
        let mut name = core::ptr::null::<c_char>();
        var_iter = unsafe {
            var_shada_iter(
                var_iter,
                &raw mut name,
                &raw mut vartv,
                VAR_FLAVOUR_DEFAULT | VAR_FLAVOUR_SESSION | VAR_FLAVOUR_SHADA,
            )
        };
        if name.is_null() {
            return packer_take_string(&packer);
        }
        // A function reference cannot be written to a file.
        if vartv.v_type != VAR_FUNC && vartv.v_type != VAR_PARTIAL {
            let mut tgttv: typval_T = unsafe { core::mem::zeroed() };
            unsafe { tv_copy(&raw mut vartv, &raw mut tgttv) };
            let entry = ShadaEntry {
                can_free_entry: false,
                timestamp: cur_timestamp,
                data: ShadaEntryData::Variable(global_var {
                    name: name.cast_mut(),
                    value: tgttv,
                }),
                additional_data: core::ptr::null_mut(),
            };
            let written = unsafe { shada_pack_entry(&raw mut packer, entry, 0) };
            assert!(written != kSDWriteFailed, "shada: cannot pack a variable");
            unsafe { tv_clear(&raw mut tgttv) };
        }
        unsafe { tv_clear(&raw mut vartv) };
        if var_iter.is_none() {
            return packer_take_string(&packer);
        }
    }
}
