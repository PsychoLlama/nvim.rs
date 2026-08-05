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

use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;

use crate::src::nvim::cmdhist::{HIST_DEBUG, HIST_EXPR, HIST_INPUT};

use super::*;
use crate::src::nvim::types::{
    VAR_FLAVOUR_DEFAULT, VAR_FLAVOUR_SESSION, VAR_FLAVOUR_SHADA, VAR_FUNC, VAR_PARTIAL,
};

/// Whether a buffer's marks are not worth remembering: it has no file name,
/// it was unlisted on purpose, it is a quickfix or terminal buffer, or its
/// file is on removable media.
pub(crate) unsafe fn ignore_buf(buf: *const buf_T, removable_bufs: *mut Set_ptr_t) -> bool {
    unsafe {
        buf.is_null()
            || (*buf).b_ffname.is_null()
            || ((*buf).b_p_bl == 0 && (*buf).b_p_initialized)
            || bt_quickfix(buf)
            || bt_terminal(buf)
            || set_has_ptr_t(removable_bufs, buf as ptr_t)
    }
}

/// Collect the buffers whose files are on removable media.
pub(crate) unsafe fn find_removable_bufs(removable_bufs: *mut Set_ptr_t) {
    unsafe {
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if !(*buf).b_ffname.is_null() && shada_removable((*buf).b_ffname) {
                set_put_ptr_t(removable_bufs, buf as ptr_t, core::ptr::null_mut());
            }
            buf = (*buf).b_next;
        }
    }
}

/// The buffer list, as one entry: every listed buffer worth remembering,
/// with the cursor position it was last left at.
///
/// `'shada'`'s `%` entry caps how many are kept; a negative cap means all
/// of them.
pub(crate) unsafe fn shada_get_buflist(removable_bufs: *mut Set_ptr_t) -> ShadaEntry {
    unsafe {
        let max_bufs = get_shada_parameter('%' as c_int);
        let mut wanted = Vec::new();
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            if !ignore_buf(buf, removable_bufs)
                && (*buf).b_p_bl != 0
                && (max_bufs < 0 || wanted.len() < max_bufs as usize)
            {
                wanted.push(buffer_list_buffer {
                    pos: (*buf).b_last_cursor.mark,
                    fname: (*buf).b_ffname,
                    additional_data: (*buf).additional_data,
                });
            }
            buf = (*buf).b_next;
        }

        // The array is `xmalloc`ed because the caller releases it with
        // `xfree`, as it does for a buffer list that came off the wire.
        let buffers = xmalloc(size_of_val(&wanted[..])).cast::<buffer_list_buffer>();
        buffers.copy_from_nonoverlapping(wanted.as_ptr(), wanted.len());
        ShadaEntry {
            type_0: kSDItemBufferList,
            can_free_entry: false,
            timestamp: os_time(),
            data: C2Rust_Unnamed_22 {
                buffer_list: buffer_list {
                    size: wanted.len(),
                    buffers,
                },
            },
            additional_data: core::ptr::null_mut(),
        }
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
    unsafe {
        let defaults = (*sd_default_values.ptr())[kSDItemSearchPattern as usize]
            .data
            .search_pattern;
        let mut pat: SearchPattern = core::mem::zeroed();
        get_pattern.expect("non-null function pointer")(&raw mut pat);
        if pat.pat.is_null() {
            return;
        }
        let last_used = is_substitute_pattern != search_last_used;
        *ret_pse = ShadaEntry {
            type_0: kSDItemSearchPattern,
            can_free_entry: false,
            timestamp: pat.timestamp,
            data: C2Rust_Unnamed_22 {
                search_pattern: KeyDict__shada_search_pat {
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
                    pat: cstr_as_string(pat.pat),
                },
            },
            additional_data: pat.additional_data,
        };
        // The two substitute-pattern defaults are `false`, which is what the
        // `!is_substitute_pattern &&` above produces; assert it rather than
        // spelling the branch out twice.
        debug_assert!(
            !defaults.has_line_offset && !defaults.place_cursor_at_end,
            "shada: a search pattern's offset defaults are not false"
        );
    }
}

/// Every global register that has anything in it, as entries.
///
/// `max_reg_lines` is `'shada'`'s `<` (or `"`) entry: a register with more
/// lines than that is not remembered at all. A negative value means no
/// limit.
pub(crate) unsafe fn shada_initialize_registers(wms: *mut WriteMergerState, max_reg_lines: c_int) {
    unsafe {
        let mut reg_iter = core::ptr::null::<c_void>();
        loop {
            let mut reg: yankreg_T = core::mem::zeroed();
            let mut name: c_char = NUL as c_char;
            let mut is_unnamed = false;
            reg_iter =
                op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed);
            if name as c_int == NUL {
                return;
            }
            let too_long = max_reg_lines >= 0 && reg.y_size > max_reg_lines as size_t;
            if !too_long {
                (*wms).registers[op_reg_index(name as c_int) as usize] = ShadaEntry {
                    type_0: kSDItemRegister,
                    can_free_entry: false,
                    timestamp: reg.timestamp,
                    data: C2Rust_Unnamed_22 {
                        reg: reg {
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
                        },
                    },
                    additional_data: reg.additional_data,
                };
            }
            if reg_iter.is_null() {
                return;
            }
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
    unsafe {
        let marks = &mut (*wms).numbered_marks;
        let last = marks.len() - 1;
        shada_free_shada_entry(&raw mut marks[last]);
        for (i, mark) in marks.iter_mut().enumerate().take(last).skip(idx) {
            if mark.type_0 == kSDItemGlobalMark {
                mark.data.filemark.name = (b'0' + i as u8 + 1) as c_char;
            }
        }
        marks.copy_within(idx..last, idx + 1);
        marks[idx] = entry;
        marks[idx].data.filemark.name = (b'0' + idx as u8) as c_char;
    }
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
    iter: *const c_void,
    name: *mut *const c_char,
    rettv: *mut typval_T,
    flavour: var_flavour_T,
) -> *const c_void {
    unsafe {
        let globvarht = get_globvar_ht();
        let first = (*globvarht).ht_array;
        let count = (*globvarht).ht_mask + 1;
        let wanted = |hi: *const hashitem_T| {
            !(*hi).hi_key.is_null()
                && (*hi).hi_key != &raw const hash_removed as *mut c_char
                && var_flavour((*hi).hi_key) & flavour != 0
        };

        *name = core::ptr::null();
        let mut hi = if iter.is_null() {
            let mut hi = first;
            while hi.offset_from_unsigned(first) < count && !wanted(hi) {
                hi = hi.add(1);
            }
            if hi.offset_from_unsigned(first) == count {
                return core::ptr::null();
            }
            hi
        } else {
            iter.cast::<hashitem_T>()
        };

        let di = (*hi).hi_key.sub(offset_of!(dictitem_T, di_key)) as *mut dictitem_T;
        *name = &raw mut (*di).di_key as *mut c_char;
        tv_copy(&raw mut (*di).di_tv, rettv);

        // Answer where the *next* one is, so the caller knows to stop.
        loop {
            hi = hi.add(1);
            if hi.offset_from_unsigned(first) >= count {
                return core::ptr::null();
            }
            if wanted(hi) {
                return hi.cast::<c_void>();
            }
        }
    }
}

/// The jump list, as entries, into `jumps`. Answers how many were written.
///
/// The current position is pushed onto the jump list first, so that where
/// Nvim was when it exited is remembered too.
pub(crate) unsafe fn shada_init_jumps(
    jumps: *mut ShadaEntry,
    removable_bufs: *mut Set_ptr_t,
) -> size_t {
    unsafe {
        let mut jumps_size: size_t = 0;
        let mut jump_iter = core::ptr::null::<c_void>();
        setpcmark();
        cleanup_jumplist(curwin.get(), false);
        loop {
            let mut fm: xfmark_T = core::mem::zeroed();
            jump_iter = mark_jumplist_iter(jump_iter, curwin.get(), &raw mut fm);

            if let Some(fname) = jump_target(&fm, jump_iter, removable_bufs) {
                *jumps.add(jumps_size) = ShadaEntry {
                    type_0: kSDItemJump,
                    can_free_entry: false,
                    timestamp: fm.fmark.timestamp,
                    data: C2Rust_Unnamed_22 {
                        filemark: shada_filemark {
                            name: NUL as c_char,
                            mark: fm.fmark.mark,
                            fname: fname.cast_mut(),
                        },
                    },
                    additional_data: fm.fmark.additional_data,
                };
                jumps_size += 1;
            }
            if jump_iter.is_null() {
                return jumps_size;
            }
        }
    }
}

/// The file name to remember one jump under, or `None` if it is not worth
/// remembering: no line number, a buffer whose marks are ignored, a buffer
/// number that names no buffer, or no file name at all.
unsafe fn jump_target(
    fm: &xfmark_T,
    jump_iter: *const c_void,
    removable_bufs: *mut Set_ptr_t,
) -> Option<*const c_char> {
    unsafe {
        if fm.fmark.mark.lnum == 0 {
            siemsg(
                c"ShaDa: mark lnum zero (ji:%p, js:%p, len:%i)".as_ptr(),
                jump_iter,
                (&raw const (*curwin.get()).w_jumplist).cast::<c_void>(),
                (*curwin.get()).w_jumplistlen,
            );
            return None;
        }
        if fm.fmark.fnum == 0 {
            // Not in a loaded buffer: the entry carries the name itself.
            return (!fm.fname.is_null()).then_some(fm.fname as *const c_char);
        }
        let buf = buflist_findnr(fm.fmark.fnum);
        if buf.is_null() || ignore_buf(buf, removable_bufs) || (*buf).b_ffname.is_null() {
            return None;
        }
        Some((*buf).b_ffname)
    }
}

/// Every register, as msgpack.
pub unsafe fn shada_encode_regs() -> String_0 {
    unsafe {
        let wms = xcalloc(1, size_of::<WriteMergerState>()).cast::<WriteMergerState>();
        shada_initialize_registers(wms, -1);
        let mut packer = packer_string_buffer();
        for i in 0..(*wms).registers.len() {
            if (*wms).registers[i].type_0 == kSDItemRegister {
                let written = shada_pack_pfreed_entry(&raw mut packer, (*wms).registers[i], 0);
                assert!(written != kSDWriteFailed, "shada: cannot pack a register");
            }
        }
        xfree(wms.cast::<c_void>());
        packer_take_string(&packer)
    }
}

/// The jump list, as msgpack.
pub unsafe fn shada_encode_jumps() -> String_0 {
    unsafe {
        let mut removable_bufs: Set_ptr_t = core::mem::zeroed();
        find_removable_bufs(&raw mut removable_bufs);
        let mut jumps: [ShadaEntry; JUMPLISTSIZE as usize] = core::mem::zeroed();
        let jumps_size = shada_init_jumps(jumps.as_mut_ptr(), &raw mut removable_bufs);
        let mut packer = packer_string_buffer();
        for jump in &jumps[..jumps_size] {
            let written = shada_pack_pfreed_entry(&raw mut packer, *jump, 0);
            assert!(written != kSDWriteFailed, "shada: cannot pack a jump");
        }
        packer_take_string(&packer)
    }
}

/// The buffer list, as msgpack.
pub unsafe fn shada_encode_buflist() -> String_0 {
    unsafe {
        let mut removable_bufs: Set_ptr_t = core::mem::zeroed();
        find_removable_bufs(&raw mut removable_bufs);
        let buflist_entry = shada_get_buflist(&raw mut removable_bufs);
        let mut packer = packer_string_buffer();
        let written = shada_pack_entry(&raw mut packer, buflist_entry, 0);
        assert!(
            written != kSDWriteFailed,
            "shada: cannot pack the buffer list"
        );
        xfree(buflist_entry.data.buffer_list.buffers.cast::<c_void>());
        packer_take_string(&packer)
    }
}

/// Every global variable `'shada'` says to remember, as msgpack.
pub unsafe fn shada_encode_gvars() -> String_0 {
    unsafe {
        let mut packer = packer_string_buffer();
        let mut var_iter = core::ptr::null::<c_void>();
        let cur_timestamp = os_time();
        loop {
            let mut vartv: typval_T = core::mem::zeroed();
            let mut name = core::ptr::null::<c_char>();
            var_iter = var_shada_iter(
                var_iter,
                &raw mut name,
                &raw mut vartv,
                VAR_FLAVOUR_DEFAULT | VAR_FLAVOUR_SESSION | VAR_FLAVOUR_SHADA,
            );
            if name.is_null() {
                return packer_take_string(&packer);
            }
            // A function reference cannot be written to a file.
            if vartv.v_type != VAR_FUNC && vartv.v_type != VAR_PARTIAL {
                let mut tgttv: typval_T = core::mem::zeroed();
                tv_copy(&raw mut vartv, &raw mut tgttv);
                let written = shada_pack_entry(
                    &raw mut packer,
                    ShadaEntry {
                        type_0: kSDItemVariable,
                        can_free_entry: false,
                        timestamp: cur_timestamp,
                        data: C2Rust_Unnamed_22 {
                            global_var: global_var {
                                name: name.cast_mut(),
                                value: tgttv,
                            },
                        },
                        additional_data: core::ptr::null_mut(),
                    },
                    0,
                );
                assert!(written != kSDWriteFailed, "shada: cannot pack a variable");
                tv_clear(&raw mut tgttv);
            }
            tv_clear(&raw mut vartv);
            if var_iter.is_null() {
                return packer_take_string(&packer);
            }
        }
    }
}
