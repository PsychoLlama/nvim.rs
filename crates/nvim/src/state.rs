use crate::autocmd::{EVENT_MODECHANGED, EVENT_SAFESTATE, apply_autocmds, has_event};
use crate::drawscreen::{setcursor, update_screen};
use crate::eval::typval::{tv_dict_add_str, tv_dict_set_keys_readonly};
use crate::eval::{get_v_event, restore_v_event};
use crate::event::multiqueue::{multiqueue_empty, multiqueue_get};
use crate::ex_getln::{cmdline_overstrike, get_cmdline_info};
use crate::getchar::{
    check_end_reg_executing, may_sync_undo, safe_vgetc, stuff_empty, using_script, vpeekc,
};
use crate::global_cell::GlobalCell;
use crate::insexpand::{ctrl_x_mode_not_defined_yet, ins_compl_active};
use crate::keycodes::{Ctrl_V, KE_EVENT, get_special_key_name};
use crate::log::{LOGLVL_DBG, logmsg_c};
use crate::main::{
    State, VIsual_active, VIsual_mode, VIsual_select, curbuf, debug_mode, exmode_active, finish_op,
    global_busy, got_int, last_mode, main_loop, mod_mask, motion_force, must_redraw,
    need_wait_return, restart_VIsual_select, restart_edit, typebuf, virtual_op,
};
use crate::option::get_ve_flags;
use crate::options::{kOptVeFlagAll, kOptVeFlagBlock, kOptVeFlagInsert};
use crate::os::input::{input_available, input_get, os_breakcheck};
use crate::strings::vim_snprintf;
use crate::types::{
    Direction, Event, ProcType, VimState, dict_T, hashitem_T, hashtab_T, kNone, save_v_event_T,
    size_t, uint8_t, win_T,
};
use crate::ui::ui_flush;
use ::libc::{strcmp, strcpy};
pub const kProcTypePty: ProcType = 1;
pub const kDirectionNotSet: Direction = 0;
/// The editor-mode bitmask `State` carries, and the masks that read it.
pub type ModeFlags = ::core::ffi::c_int;
pub const MODE_SHOWMATCH: ModeFlags = 24592;
pub const MODE_EXTERNCMD: ModeFlags = 20480;
pub const MODE_SETWSIZE: ModeFlags = 16384;
pub const MODE_ASKMORE: ModeFlags = 12288;
pub const MODE_HITRETURN: ModeFlags = 8193;
pub const MODE_NORMAL_BUSY: ModeFlags = 4097;
pub const MODE_LREPLACE: ModeFlags = 288;
pub const MODE_VREPLACE: ModeFlags = 784;
pub const VREPLACE_FLAG: ModeFlags = 512;
pub const MODE_REPLACE: ModeFlags = 272;
pub const REPLACE_FLAG: ModeFlags = 256;
pub const MAP_ALL_MODES: ModeFlags = 255;
pub const MODE_TERMINAL: ModeFlags = 128;
pub const MODE_SELECT: ModeFlags = 64;
pub const MODE_LANGMAP: ModeFlags = 32;
pub const MODE_INSERT: ModeFlags = 16;
pub const MODE_CMDLINE: ModeFlags = 8;
pub const MODE_OP_PENDING: ModeFlags = 4;
pub const MODE_VISUAL: ModeFlags = 2;
pub const MODE_NORMAL: ModeFlags = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub unsafe extern "C" fn state_enter(mut s: *mut VimState) {
    's_132: loop {
        let mut check_result: ::core::ffi::c_int = if (*s).check.is_some() {
            (*s).check.expect("non-null function pointer")(s)
        } else {
            1 as ::core::ffi::c_int
        };
        if check_result == 0 {
            break;
        }
        if check_result == -1 as ::core::ffi::c_int {
            continue;
        }
        let mut key: ::core::ffi::c_int = 0;
        loop {
            if vpeekc() != NUL || (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
                key = safe_vgetc();
            } else if !multiqueue_empty((*main_loop.ptr()).events) {
                ui_flush();
                key = -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            } else {
                if must_redraw.get() != 0 as ::core::ffi::c_int
                    && !need_wait_return.get()
                    && State.get() & MODE_CMDLINE == 0 as ::core::ffi::c_int
                {
                    update_screen();
                    setcursor();
                }
                ui_flush();
                input_get(
                    ::core::ptr::null_mut::<uint8_t>(),
                    0 as ::core::ffi::c_int,
                    -1 as ::core::ffi::c_int,
                    (*typebuf.ptr()).tb_change_cnt,
                    (*main_loop.ptr()).events,
                );
                if !(input_available() == 0 && !multiqueue_empty((*main_loop.ptr()).events)) {
                    continue;
                }
                key = -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            if key
                == -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                check_end_reg_executing(true_0 != 0);
                may_sync_undo();
            }
            let mut keyname: *mut ::core::ffi::c_char = (if key
                == -(253 as ::core::ffi::c_int
                    + ((KE_EVENT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                c"K_EVENT".as_ptr()
            } else {
                get_special_key_name(key, mod_mask.get()) as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            logmsg_c!(
                LOGLVL_DBG,
                ::core::ptr::null::<::core::ffi::c_char>(),
                c"state_enter".as_ptr(),
                97 as ::core::ffi::c_int,
                true_0 != 0,
                c"input: %s".as_ptr(),
                keyname,
            );
            let mut execute_result: ::core::ffi::c_int =
                (*s).execute.expect("non-null function pointer")(s, key);
            if execute_result == 0 {
                break 's_132;
            }
            if execute_result != -1 as ::core::ffi::c_int {
                break;
            }
        }
    }
}
pub unsafe extern "C" fn state_handle_k_event() {
    loop {
        let mut event: Event = multiqueue_get((*main_loop.ptr()).events);
        if let Some(handler) = event.handler {
            handler(&raw mut event.argv as *mut *mut ::core::ffi::c_void);
        }
        if multiqueue_empty((*main_loop.ptr()).events) {
            return;
        }
        os_breakcheck();
        if input_available() != 0 || got_int.get() as ::core::ffi::c_int != 0 {
            return;
        }
    }
}
pub unsafe extern "C" fn virtual_active(mut wp: *mut win_T) -> bool {
    if virtual_op.get() as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
        return virtual_op.get() as u64 != 0;
    }
    if State.get() & MODE_TERMINAL != 0 {
        return true_0 != 0;
    }
    let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(wp);
    return cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
        || cur_ve_flags & kOptVeFlagBlock as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && VIsual_active.get() as ::core::ffi::c_int != 0
            && VIsual_mode.get() == Ctrl_V
        || cur_ve_flags & kOptVeFlagInsert as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && State.get() & MODE_INSERT != 0;
}
pub unsafe extern "C" fn get_real_state() -> ::core::ffi::c_int {
    if State.get() & MODE_NORMAL != 0 {
        if VIsual_active.get() {
            if VIsual_select.get() {
                return MODE_SELECT;
            }
            return MODE_VISUAL;
        } else if finish_op.get() {
            return MODE_OP_PENDING;
        }
    }
    return State.get();
}
pub unsafe extern "C" fn get_mode(mut buf: *mut ::core::ffi::c_char) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if State.get() == MODE_HITRETURN
        || State.get() == MODE_ASKMORE
        || State.get() == MODE_SETWSIZE
        || State.get() & MODE_CMDLINE != 0
            && (*get_cmdline_info()).one_key as ::core::ffi::c_int != 0
    {
        let c2rust_fresh0 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh0 as isize) = 'r' as ::core::ffi::c_char;
        if State.get() == MODE_ASKMORE {
            let c2rust_fresh1 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh1 as isize) = 'm' as ::core::ffi::c_char;
        } else if State.get() & MODE_CMDLINE != 0 {
            let c2rust_fresh2 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh2 as isize) = '?' as ::core::ffi::c_char;
        }
    } else if State.get() == MODE_EXTERNCMD {
        let c2rust_fresh3 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh3 as isize) = '!' as ::core::ffi::c_char;
    } else if State.get() & MODE_INSERT != 0 {
        if State.get() & VREPLACE_FLAG != 0 {
            let c2rust_fresh4 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh4 as isize) = 'R' as ::core::ffi::c_char;
            let c2rust_fresh5 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh5 as isize) = 'v' as ::core::ffi::c_char;
        } else if State.get() & REPLACE_FLAG != 0 {
            let c2rust_fresh6 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh6 as isize) = 'R' as ::core::ffi::c_char;
        } else {
            let c2rust_fresh7 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh7 as isize) = 'i' as ::core::ffi::c_char;
        }
        if ins_compl_active() {
            let c2rust_fresh8 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh8 as isize) = 'c' as ::core::ffi::c_char;
        } else if ctrl_x_mode_not_defined_yet() {
            let c2rust_fresh9 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh9 as isize) = 'x' as ::core::ffi::c_char;
        }
    } else if State.get() & MODE_CMDLINE != 0 || exmode_active.get() as ::core::ffi::c_int != 0 {
        let c2rust_fresh10 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh10 as isize) = 'c' as ::core::ffi::c_char;
        if exmode_active.get() {
            let c2rust_fresh11 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh11 as isize) = 'v' as ::core::ffi::c_char;
        }
        if State.get() & MODE_CMDLINE != 0 && cmdline_overstrike() as ::core::ffi::c_int != 0 {
            let c2rust_fresh12 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh12 as isize) = 'r' as ::core::ffi::c_char;
        }
    } else if State.get() & MODE_TERMINAL != 0 {
        let c2rust_fresh13 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh13 as isize) = 't' as ::core::ffi::c_char;
    } else if VIsual_active.get() {
        if VIsual_select.get() {
            let c2rust_fresh14 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh14 as isize) = (VIsual_mode.get() + 's' as ::core::ffi::c_int
                - 'v' as ::core::ffi::c_int)
                as ::core::ffi::c_char;
        } else {
            let c2rust_fresh15 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh15 as isize) = VIsual_mode.get() as ::core::ffi::c_char;
            if restart_VIsual_select.get() != 0 {
                let c2rust_fresh16 = i;
                i = i + 1;
                *buf.offset(c2rust_fresh16 as isize) = 's' as ::core::ffi::c_char;
            }
        }
    } else {
        let c2rust_fresh17 = i;
        i = i + 1;
        *buf.offset(c2rust_fresh17 as isize) = 'n' as ::core::ffi::c_char;
        if finish_op.get() {
            let c2rust_fresh18 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh18 as isize) = 'o' as ::core::ffi::c_char;
            let c2rust_fresh19 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh19 as isize) = motion_force.get() as ::core::ffi::c_char;
        } else if !(*curbuf.get()).terminal.is_null() {
            let c2rust_fresh20 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh20 as isize) = 't' as ::core::ffi::c_char;
            if restart_edit.get() == 'I' as ::core::ffi::c_int {
                let c2rust_fresh21 = i;
                i = i + 1;
                *buf.offset(c2rust_fresh21 as isize) = 'T' as ::core::ffi::c_char;
            }
        } else if restart_edit.get() == 'I' as ::core::ffi::c_int
            || restart_edit.get() == 'R' as ::core::ffi::c_int
            || restart_edit.get() == 'V' as ::core::ffi::c_int
        {
            let c2rust_fresh22 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh22 as isize) = 'i' as ::core::ffi::c_char;
            let c2rust_fresh23 = i;
            i = i + 1;
            *buf.offset(c2rust_fresh23 as isize) = restart_edit.get() as ::core::ffi::c_char;
        }
    }
    *buf.offset(i as isize) = NUL as ::core::ffi::c_char;
}
pub unsafe extern "C" fn may_trigger_modechanged() {
    if !has_event(EVENT_MODECHANGED) || got_int.get() as ::core::ffi::c_int != 0 {
        return;
    }
    let mut curr_mode: [::core::ffi::c_char; 4] = [0; 4];
    let mut pattern_buf: [::core::ffi::c_char; 8] = [0; 8];
    get_mode(&raw mut curr_mode as *mut ::core::ffi::c_char);
    if strcmp(
        &raw mut curr_mode as *mut ::core::ffi::c_char,
        last_mode.ptr() as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        return;
    }
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
    };
    let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
    tv_dict_add_str(
        v_event,
        c"new_mode".as_ptr(),
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        &raw mut curr_mode as *mut ::core::ffi::c_char,
    );
    tv_dict_add_str(
        v_event,
        c"old_mode".as_ptr(),
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        last_mode.ptr() as *mut ::core::ffi::c_char,
    );
    tv_dict_set_keys_readonly(v_event);
    vim_snprintf(
        &raw mut pattern_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
        c"%s:%s".as_ptr(),
        last_mode.ptr() as *mut ::core::ffi::c_char,
        &raw mut curr_mode as *mut ::core::ffi::c_char,
    );
    apply_autocmds(
        EVENT_MODECHANGED,
        &raw mut pattern_buf as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    strcpy(
        last_mode.ptr() as *mut ::core::ffi::c_char,
        &raw mut curr_mode as *mut ::core::ffi::c_char,
    );
    restore_v_event(v_event, &raw mut save_v_event);
}
static was_safe: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn is_safe_now() -> bool {
    return stuff_empty() as ::core::ffi::c_int != 0
        && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        && using_script() == 0
        && global_busy.get() == 0
        && !debug_mode.get();
}
pub unsafe extern "C" fn may_trigger_safestate(mut safe: bool) {
    let mut is_safe: bool =
        safe as ::core::ffi::c_int != 0 && is_safe_now() as ::core::ffi::c_int != 0;
    if was_safe.get() as ::core::ffi::c_int != is_safe as ::core::ffi::c_int {
        logmsg_c!(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            c"may_trigger_safestate".as_ptr(),
            305 as ::core::ffi::c_int,
            true_0 != 0,
            if is_safe as ::core::ffi::c_int != 0 {
                c"SafeState: Start triggering".as_ptr()
            } else {
                c"SafeState: Stop triggering".as_ptr()
            },
        );
    }
    if is_safe {
        apply_autocmds(
            EVENT_SAFESTATE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
    }
    was_safe.set(is_safe);
}
pub unsafe extern "C" fn state_no_longer_safe(mut reason: *const ::core::ffi::c_char) {
    if was_safe.get() as ::core::ffi::c_int != 0 && !reason.is_null() {
        logmsg_c!(
            LOGLVL_DBG,
            ::core::ptr::null::<::core::ffi::c_char>(),
            c"state_no_longer_safe".as_ptr(),
            319 as ::core::ffi::c_int,
            true_0 != 0,
            c"SafeState reset: %s".as_ptr(),
            reason,
        );
    }
    was_safe.set(false_0 != 0);
}
pub unsafe extern "C" fn get_was_safe_state() -> bool {
    return was_safe.get();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
