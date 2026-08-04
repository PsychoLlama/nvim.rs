use crate::src::nvim::api::extmark::{parse_virt_text, virt_text_to_array};
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_free_object, api_set_error, api_typename, arena_array,
    cstr_as_string, cstr_to_string, cstrn_as_string, find_buffer_by_handle, find_window_by_handle,
    object_to_hl_id, try_enter, try_leave,
};
use crate::src::nvim::api::private::validate::{
    api_err_conflict, api_err_exp, api_err_invalid, api_err_required,
};
use crate::src::nvim::autocmd::{
    EVENT_WINNEW, apply_autocmds, block_autocmds, is_aucmd_win, unblock_autocmds,
};
use crate::src::nvim::buffer::{bufref_valid, set_bufref};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_later, set_must_redraw};
use crate::src::nvim::eval::window::{
    restore_win, restore_win_noblock, switch_win, switch_win_noblock,
};
use crate::src::nvim::ex_docmd::expr_map_locked;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{syn_check_group, syn_id2name};
use crate::src::nvim::main::{
    autocmd_no_enter, autocmd_no_leave, cmdline_win, cmdwin_buf, cmdwin_old_curwin, cmdwin_type,
    cmdwin_win, curbuf, curtab, curwin, e_cmdwin, e_textlock, float_anchor_str, p_sb, p_spr,
    p_winborder, textlock,
};
use crate::src::nvim::mbyte::{mb_string2cells, mb_string2cells_len};
use crate::src::nvim::memory::{strequal, xrealloc, xstrdup};
use crate::src::nvim::r#move::changed_window_setting;
use crate::src::nvim::option::{copy_option_part, didset_window_options};
use crate::src::nvim::options::opt_winborder_values;
use crate::src::nvim::os::libc::{__assert_fail, memcpy, memset, strchr};
use crate::src::nvim::strings::striequal;
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    AlignTextPos, Arena, Array, BoolVarValue, Boolean, BorderTextType, Buffer, CMD_index, Error,
    Float, FloatAnchor, FloatRelative, Integer, KeyDict_win_config, Object, OptionalKeys,
    ScopeType, SpecialVarValue, String_0, TryState, VarLockStatus, VarType, VirtText,
    VirtTextChunk, WinConfig, WinSplit, WinStyle, Window, buf_T, bufref_T, colnr_T, except_T,
    frame_T, int64_t, kObjectTypeArray, kObjectTypeInteger, kObjectTypeNil, kObjectTypeString,
    linenr_T, lpos_T, msglist_T, object, object_data as C2Rust_Unnamed, size_t, switchwin_T,
    tabpage_T, win_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::ui_compositor::ui_comp_remove_grid;
use crate::src::nvim::window::{
    check_split_disallowed_err, clear_float_config, goto_tabpage_win, last_status,
    lastwin_nofloating, merge_win_config, one_window, win_append, win_comp_pos, win_find_tabpage,
    win_goto, win_locked, win_remove, win_set_buf, win_setheight_win, win_setwidth_win,
    win_split_ins, win_valid, win_valid_any_tab, window_layout_locked_err, winframe_find_altwin,
    winframe_remove, winframe_restore,
};
use crate::src::nvim::winfloat::{
    win_config_float, win_float_find_altwin, win_new_float, win_set_minimal_style,
};
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeLaststatus: FloatRelative = 5;
pub const kFloatRelativeTabline: FloatRelative = 4;
pub const kFloatRelativeMouse: FloatRelative = 3;
pub const kFloatRelativeCursor: FloatRelative = 2;
pub const kFloatRelativeWindow: FloatRelative = 1;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kZIndexFloatDefault: C2Rust_Unnamed_13 = 50;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kFloatAnchorSouth: C2Rust_Unnamed_14 = 2;
pub const kFloatAnchorEast: C2Rust_Unnamed_14 = 1;
pub const kBorderTextFooter: BorderTextType = 1;
pub const kBorderTextTitle: BorderTextType = 0;
pub const WSP_VERT: C2Rust_Unnamed_17 = 2;
pub const WSP_NOENTER: C2Rust_Unnamed_17 = 512;
pub const WSP_BELOW: C2Rust_Unnamed_17 = 64;
pub const WSP_BOT: C2Rust_Unnamed_17 = 16;
pub const WSP_ABOVE: C2Rust_Unnamed_17 = 128;
pub const WSP_TOP: C2Rust_Unnamed_17 = 8;
pub const WSP_HOR: C2Rust_Unnamed_17 = 4;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_15 {
    pub name: *const ::core::ffi::c_char,
    pub chars: [[::core::ffi::c_char; 32]; 8],
    pub shadow_color: bool,
}
pub const CMD_SIZE: CMD_index = 557;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const KEYSET_OPTIDX_win_config__col: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__row: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__win: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__hide: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__width: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__split: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__title: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__mouse: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__fixed: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__style: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__anchor: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__bufpos: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__height: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__zindex: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__footer: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__border: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__external: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__relative: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__vertical: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__focusable: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__noautocmd: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__title_pos: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__footer_pos: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config___cmdline_offset: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYDICT_INIT: KeyDict_win_config = KeyDict_win_config {
    is_set__win_config_: 0 as OptionalKeys,
    external: false,
    fixed: false,
    focusable: false,
    footer: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    footer_pos: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    hide: false,
    height: 0,
    mouse: false,
    relative: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    row: 0.,
    style: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    noautocmd: false,
    vertical: false,
    win: 0,
    width: 0,
    zindex: 0,
    anchor: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    border: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    bufpos: Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    },
    col: 0.,
    split: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    title: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    title_pos: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    _cmdline_offset: 0,
};
pub unsafe extern "C" fn nvim_open_win(
    mut buf: Buffer,
    mut enter: Boolean,
    mut config: *mut KeyDict_win_config,
    mut err: *mut Error,
) -> Window {
    let mut bufref: bufref_T = bufref_T::default();
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return 0 as Window;
    }
    if cmdwin_type.get() != 0 as ::core::ffi::c_int && enter as ::core::ffi::c_int != 0
        || b == cmdwin_buf.get()
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return 0 as Window;
    }
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    if !parse_win_config(
        ::core::ptr::null_mut::<win_T>(),
        config,
        &raw mut fconfig,
        false_0 != 0,
        err,
    ) {
        return 0 as Window;
    }
    let mut is_split: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
        != 0 as ::core::ffi::c_ulonglong
        || (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
            != 0 as ::core::ffi::c_ulonglong;
    let mut rv: Window = 0 as Window;
    if fconfig.noautocmd {
        block_autocmds();
    }
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tp: *mut tabpage_T = curtab.get();
    '_c2rust_label: {
        if !(*curwin.ptr()).is_null() {
        } else {
            __assert_fail(
                b"curwin != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/api/win_config.rs\0".as_ptr() as *const ::core::ffi::c_char,
                229 as ::core::ffi::c_uint,
                b"Window nvim_open_win(Buffer, Boolean, KeyDict_win_config *, Error *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut parent: *mut win_T = if (*config).win == 0 as ::core::ffi::c_int {
        curwin.get()
    } else {
        ::core::ptr::null_mut::<win_T>()
    };
    '_cleanup: {
        if (*config).win > 0 as ::core::ffi::c_int {
            parent = find_window_by_handle(fconfig.window, err);
            if parent.is_null() {
                break '_cleanup;
            } else if is_split as ::core::ffi::c_int != 0
                && (*parent).w_floating as ::core::ffi::c_int != 0
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Cannot split a floating window\0".as_ptr() as *const ::core::ffi::c_char,
                );
                break '_cleanup;
            } else {
                tp = win_find_tabpage(parent);
            }
        }
        if is_split {
            if !check_split_disallowed_err(
                if !parent.is_null() {
                    parent
                } else {
                    curwin.get()
                },
                err,
            ) {
                break '_cleanup;
            } else {
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
                    != 0 as ::core::ffi::c_ulonglong
                    && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
                        != 0 as ::core::ffi::c_ulonglong)
                {
                    if (*config).vertical {
                        fconfig.split = (if p_spr.get() != 0 {
                            kWinSplitRight as ::core::ffi::c_int
                        } else {
                            kWinSplitLeft as ::core::ffi::c_int
                        }) as WinSplit;
                    } else {
                        fconfig.split = (if p_sb.get() != 0 {
                            kWinSplitBelow as ::core::ffi::c_int
                        } else {
                            kWinSplitAbove as ::core::ffi::c_int
                        }) as WinSplit;
                    }
                }
                let mut flags: ::core::ffi::c_int =
                    win_split_flags(fconfig.split, parent.is_null())
                        | WSP_NOENTER as ::core::ffi::c_int;
                let mut size: ::core::ffi::c_int = if flags & WSP_VERT as ::core::ffi::c_int != 0 {
                    fconfig.width
                } else {
                    fconfig.height
                };
                let mut tstate: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate);
                if parent.is_null() || parent == curwin.get() {
                    wp = win_split_ins(
                        size,
                        flags,
                        ::core::ptr::null_mut::<win_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<frame_T>(),
                    );
                } else {
                    let mut switchwin: switchwin_T = switchwin_T {
                        sw_curwin: ::core::ptr::null_mut::<win_T>(),
                        sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                        sw_same_win: false,
                        sw_visual_active: false,
                    };
                    let result: ::core::ffi::c_int =
                        switch_win(&raw mut switchwin, parent, tp, true);
                    '_c2rust_label_0: {
                        if result == 1 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                264 as ::core::ffi::c_uint,
                                b"Window nvim_open_win(Buffer, Boolean, KeyDict_win_config *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    wp = win_split_ins(
                        size,
                        flags,
                        ::core::ptr::null_mut::<win_T>(),
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<frame_T>(),
                    );
                    restore_win(&raw mut switchwin, true);
                }
                try_leave(&raw mut tstate, err);
                if !wp.is_null() {
                    (*wp).w_config = fconfig;
                    if size > 0 as ::core::ffi::c_int {
                        if flags & WSP_VERT as ::core::ffi::c_int != 0 && (*wp).w_width != size {
                            win_setwidth_win(size, wp);
                        } else if flags & WSP_VERT as ::core::ffi::c_int == 0
                            && (*wp).w_height != size
                        {
                            win_setheight_win(size, wp);
                        }
                    }
                }
            }
        } else if (*(*curwin.get()).w_buffer).b_locked_split != 0 {
            api_set_error(
                err,
                kErrorTypeException,
                b"E1159: Cannot open a float when closing the buffer\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            break '_cleanup;
        } else {
            wp = win_new_float(::core::ptr::null_mut::<win_T>(), false_0 != 0, fconfig, err);
        }
        if wp.is_null() {
            if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Failed to create window\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        } else {
            if fconfig._cmdline_offset < INT_MAX {
                cmdline_win.set(wp);
            }
            bufref = bufref_T::default();
            set_bufref(&raw mut bufref, b);
            if !fconfig.noautocmd {
                let mut switchwin_0: switchwin_T = switchwin_T {
                    sw_curwin: ::core::ptr::null_mut::<win_T>(),
                    sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                    sw_same_win: false,
                    sw_visual_active: false,
                };
                let result_0: ::core::ffi::c_int =
                    switch_win_noblock(&raw mut switchwin_0, wp, tp, true_0 != 0);
                '_c2rust_label_1: {
                    if result_0 == 1 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/api/win_config.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            311 as ::core::ffi::c_uint,
                            b"Window nvim_open_win(Buffer, Boolean, KeyDict_win_config *, Error *)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                if apply_autocmds(
                    EVENT_WINNEW,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    false_0 != 0,
                    curbuf.get(),
                ) {
                    tp = win_find_tabpage(wp);
                }
                restore_win_noblock(&raw mut switchwin_0, true_0 != 0);
            }
            if !tp.is_null() && enter as ::core::ffi::c_int != 0 {
                goto_tabpage_win(tp, wp);
                tp = win_find_tabpage(wp);
            }
            if !tp.is_null()
                && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && b != (*wp).w_buffer
            {
                let au_no_enter_leave: bool = curwin.get() != wp && !fconfig.noautocmd;
                if au_no_enter_leave {
                    (*autocmd_no_enter.ptr()) += 1;
                    (*autocmd_no_leave.ptr()) += 1;
                }
                win_set_buf(wp, b, err);
                if !fconfig.noautocmd {
                    tp = win_find_tabpage(wp);
                }
                if au_no_enter_leave {
                    (*autocmd_no_enter.ptr()) -= 1;
                    (*autocmd_no_leave.ptr()) -= 1;
                }
            }
            if tp.is_null() {
                api_clear_error(err);
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Window was closed immediately\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if fconfig.style as ::core::ffi::c_uint
                    == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    win_set_minimal_style(wp);
                    didset_window_options(wp, true_0 != 0);
                    changed_window_setting(wp);
                }
                rv = (*wp).handle as Window;
            }
        }
    }
    if fconfig.noautocmd {
        unblock_autocmds();
    }
    return rv;
}
unsafe extern "C" fn win_split_dir(mut win: *mut win_T) -> WinSplit {
    if (*win).w_frame.is_null() || (*(*win).w_frame).fr_parent.is_null() {
        return kWinSplitLeft;
    }
    let mut layout: ::core::ffi::c_char = (*(*(*win).w_frame).fr_parent).fr_layout;
    if layout as ::core::ffi::c_int == FR_COL {
        return (if !(*(*win).w_frame).fr_next.is_null() {
            kWinSplitAbove as ::core::ffi::c_int
        } else {
            kWinSplitBelow as ::core::ffi::c_int
        }) as WinSplit;
    } else {
        return (if !(*(*win).w_frame).fr_next.is_null() {
            kWinSplitLeft as ::core::ffi::c_int
        } else {
            kWinSplitRight as ::core::ffi::c_int
        }) as WinSplit;
    };
}
unsafe extern "C" fn win_split_flags(
    mut split: WinSplit,
    mut toplevel: bool,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= WSP_HOR as ::core::ffi::c_int;
    } else {
        flags |= WSP_VERT as ::core::ffi::c_int;
    }
    if split as ::core::ffi::c_uint == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
        || split as ::core::ffi::c_uint
            == kWinSplitLeft as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_TOP as ::core::ffi::c_int
        } else {
            WSP_ABOVE as ::core::ffi::c_int
        };
    } else {
        flags |= if toplevel as ::core::ffi::c_int != 0 {
            WSP_BOT as ::core::ffi::c_int
        } else {
            WSP_BELOW as ::core::ffi::c_int
        };
    }
    return flags;
}
unsafe extern "C" fn win_can_move_tp(
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
    mut err: *mut Error,
) -> bool {
    if one_window(
        wp,
        if tp == curtab.get() {
            ::core::ptr::null_mut::<tabpage_T>()
        } else {
            tp
        },
    ) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Cannot move last non-floating window\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if win_locked(wp) != 0 {
        api_set_error(
            err,
            kErrorTypeException,
            b"Cannot move window to another tabpage whilst in use\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if window_layout_locked_err(CMD_SIZE, err) {
        return false_0 != 0;
    }
    if textlock.get() != 0 || expr_map_locked() as ::core::ffi::c_int != 0 {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_textlock as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if is_aucmd_win(wp) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Cannot move autocmd window to another tabpage\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    if wp == cmdwin_win.get() || wp == cmdwin_old_curwin.get() {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn win_find_altwin(mut win: *mut win_T, mut tp: *mut tabpage_T) -> *mut win_T {
    if (*win).w_floating {
        return win_float_find_altwin(
            win,
            if tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                tp
            },
        );
    } else {
        let mut dir: ::core::ffi::c_int = 0;
        return winframe_find_altwin(
            win,
            &raw mut dir,
            if tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                tp
            },
            ::core::ptr::null_mut::<*mut frame_T>(),
        );
    };
}
unsafe extern "C" fn win_config_split(
    mut win: *mut win_T,
    mut config: *const KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) -> bool {
    let mut dir: ::core::ffi::c_int = 0;
    let mut unflat_altfr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
    let mut altwin_0: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut parent: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut parent_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut win_tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut to_split_ok: bool = false;
    let mut curwin_moving_tp: bool = false;
    let mut was_split: bool = !(*win).w_floating;
    let mut has_split: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
        != 0 as ::core::ffi::c_ulonglong;
    let mut has_vertical: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
        != 0 as ::core::ffi::c_ulonglong;
    let mut old_split: WinSplit = win_split_dir(win);
    if has_vertical as ::core::ffi::c_int != 0 && !has_split {
        if (*config).vertical {
            (*fconfig).split = (if old_split as ::core::ffi::c_uint
                == kWinSplitRight as ::core::ffi::c_int as ::core::ffi::c_uint
                || p_spr.get() != 0
            {
                kWinSplitRight as ::core::ffi::c_int
            } else {
                kWinSplitLeft as ::core::ffi::c_int
            }) as WinSplit;
        } else {
            (*fconfig).split = (if old_split as ::core::ffi::c_uint
                == kWinSplitBelow as ::core::ffi::c_int as ::core::ffi::c_uint
                || p_sb.get() != 0
            {
                kWinSplitBelow as ::core::ffi::c_int
            } else {
                kWinSplitAbove as ::core::ffi::c_int
            }) as WinSplit;
        }
    }
    '_resize: {
        if !(!has_vertical && !has_split
            || was_split as ::core::ffi::c_int != 0
                && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
                    != 0 as ::core::ffi::c_ulonglong)
                && old_split as ::core::ffi::c_uint == (*fconfig).split as ::core::ffi::c_uint)
        {
            parent = ::core::ptr::null_mut::<win_T>();
            parent_tp = ::core::ptr::null_mut::<tabpage_T>();
            if (*config).win == 0 as ::core::ffi::c_int {
                parent = curwin.get();
                parent_tp = curtab.get();
            } else if (*config).win > 0 as ::core::ffi::c_int {
                parent = find_window_by_handle((*fconfig).window, err);
                if parent.is_null() {
                    return false_0 != 0;
                }
                parent_tp = win_find_tabpage(parent);
            }
            win_tp = win_find_tabpage(win);
            if !parent.is_null() {
                if (*parent).w_floating {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Cannot split a floating window\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return false_0 != 0;
                }
                if win_tp != parent_tp && !win_can_move_tp(win, win_tp, err) {
                    return false_0 != 0;
                }
            }
            if !check_split_disallowed_err(win, err) {
                return false_0 != 0;
            }
            to_split_ok = false_0 != 0;
            curwin_moving_tp = win == curwin.get() && !parent.is_null() && win_tp != parent_tp;
            '_restore_curwin: {
                if curwin_moving_tp {
                    let mut altwin: *mut win_T = win_find_altwin(win, win_tp);
                    '_c2rust_label: {
                        if !altwin.is_null() {
                        } else {
                            __assert_fail(
                                b"altwin\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                492 as ::core::ffi::c_uint,
                                b"_Bool win_config_split(win_T *, const KeyDict_win_config *, WinConfig *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    win_goto(altwin);
                    if curwin.get() == win {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to switch away from window %d\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            (*win).handle,
                        );
                        return false_0 != 0;
                    }
                    win_tp = win_find_tabpage(win);
                    if win_tp.is_null() || !win_valid_any_tab(parent) {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Windows to split were closed\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_restore_curwin;
                    } else if was_split as ::core::ffi::c_int
                        == (*win).w_floating as ::core::ffi::c_int
                        || (*parent).w_floating as ::core::ffi::c_int != 0
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Floating state of windows to split changed\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_restore_curwin;
                    }
                }
                dir = 0 as ::core::ffi::c_int;
                unflat_altfr = ::core::ptr::null_mut::<frame_T>();
                altwin_0 = ::core::ptr::null_mut::<win_T>();
                if was_split {
                    if (*(*win).w_frame).fr_parent.is_null() {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Cannot move last non-floating window\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_restore_curwin;
                    } else if !parent.is_null() && (*parent).handle == (*win).handle {
                        let mut n_frames: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut fr: *mut frame_T = (*(*(*win).w_frame).fr_parent).fr_child;
                        while !fr.is_null() {
                            n_frames += 1;
                            fr = (*fr).fr_next;
                        }
                        let mut neighbor: *mut win_T = ::core::ptr::null_mut::<win_T>();
                        if n_frames > 2 as ::core::ffi::c_int {
                            let mut frame: *mut frame_T = (*(*win).w_frame).fr_parent;
                            if !(*frame).fr_parent.is_null() {
                                if (*fconfig).split as ::core::ffi::c_uint
                                    == kWinSplitAbove as ::core::ffi::c_int as ::core::ffi::c_uint
                                    || (*fconfig).split as ::core::ffi::c_uint
                                        == kWinSplitLeft as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                {
                                    neighbor = (*win).w_next;
                                } else {
                                    neighbor = (*win).w_prev;
                                }
                            }
                            altwin_0 = winframe_remove(
                                win,
                                &raw mut dir,
                                if win_tp == curtab.get() {
                                    ::core::ptr::null_mut::<tabpage_T>()
                                } else {
                                    win_tp
                                },
                                &raw mut unflat_altfr,
                            );
                        } else if n_frames == 2 as ::core::ffi::c_int {
                            altwin_0 = winframe_remove(
                                win,
                                &raw mut dir,
                                if win_tp == curtab.get() {
                                    ::core::ptr::null_mut::<tabpage_T>()
                                } else {
                                    win_tp
                                },
                                &raw mut unflat_altfr,
                            );
                            neighbor = altwin_0;
                        } else {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                b"Cannot split window into itself\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_restore_curwin;
                        }
                        parent = neighbor;
                    } else {
                        altwin_0 = winframe_remove(
                            win,
                            &raw mut dir,
                            if win_tp == curtab.get() {
                                ::core::ptr::null_mut::<tabpage_T>()
                            } else {
                                win_tp
                            },
                            &raw mut unflat_altfr,
                        );
                    }
                } else {
                    altwin_0 = win_float_find_altwin(
                        win,
                        if win_tp == curtab.get() {
                            ::core::ptr::null_mut::<tabpage_T>()
                        } else {
                            win_tp
                        },
                    );
                }
                win_remove(
                    win,
                    if win_tp == curtab.get() {
                        ::core::ptr::null_mut::<tabpage_T>()
                    } else {
                        win_tp
                    },
                );
                if win_tp == curtab.get() {
                    last_status(false_0 != 0);
                    win_comp_pos();
                }
                flags = win_split_flags((*fconfig).split, parent.is_null())
                    | WSP_NOENTER as ::core::ffi::c_int;
                parent_tp = if !parent.is_null() {
                    win_find_tabpage(parent)
                } else {
                    curtab.get()
                };
                let mut tstate: TryState = TryState {
                    current_exception: ::core::ptr::null_mut::<except_T>(),
                    private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
                    msg_list: ::core::ptr::null::<*const msglist_T>(),
                    got_int: 0,
                    did_throw: false,
                    need_rethrow: 0,
                    did_emsg: 0,
                };
                try_enter(&raw mut tstate);
                let need_switch: bool = !parent.is_null() && parent != curwin.get();
                let mut switchwin: switchwin_T = switchwin_T {
                    sw_curwin: ::core::ptr::null_mut::<win_T>(),
                    sw_curtab: ::core::ptr::null_mut::<tabpage_T>(),
                    sw_same_win: false,
                    sw_visual_active: false,
                };
                if need_switch {
                    let result: ::core::ffi::c_int =
                        switch_win(&raw mut switchwin, parent, parent_tp, true);
                    '_c2rust_label_0: {
                        if result == 1 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"result == OK\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                594 as ::core::ffi::c_uint,
                                b"_Bool win_config_split(win_T *, const KeyDict_win_config *, WinConfig *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
                to_split_ok = !win_split_ins(
                    0 as ::core::ffi::c_int,
                    flags,
                    win,
                    0 as ::core::ffi::c_int,
                    unflat_altfr,
                )
                .is_null();
                if !to_split_ok {
                    win_append(
                        (*win).w_prev,
                        win,
                        if win_tp == curtab.get() {
                            ::core::ptr::null_mut::<tabpage_T>()
                        } else {
                            win_tp
                        },
                    );
                }
                if need_switch {
                    restore_win(&raw mut switchwin, true);
                }
                try_leave(&raw mut tstate, err);
                if !to_split_ok {
                    if was_split {
                        winframe_restore(win, dir, unflat_altfr);
                    }
                    if !((*err).type_0 as ::core::ffi::c_int
                        != kErrorTypeNone as ::core::ffi::c_int)
                    {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"Failed to move window %d into split\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            (*win).handle,
                        );
                    }
                } else {
                    if win_tp != parent_tp && (*win_tp).tp_curwin == win {
                        (*win_tp).tp_curwin = altwin_0;
                    }
                    break '_resize;
                }
            }
            if curwin_moving_tp as ::core::ffi::c_int != 0
                && win_valid(win) as ::core::ffi::c_int != 0
            {
                win_goto(win);
            }
            return false_0 != 0;
        }
    }
    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width
        != 0 as ::core::ffi::c_ulonglong
    {
        win_setwidth_win((*fconfig).width, win);
    }
    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height
        != 0 as ::core::ffi::c_ulonglong
    {
        win_setheight_win((*fconfig).height, win);
    }
    if !was_split {
        clear_float_config(fconfig, false_0 != 0);
    }
    merge_win_config(&raw mut (*win).w_config, *fconfig);
    return true_0 != 0;
}
unsafe extern "C" fn win_config_float_tp(
    mut win: *mut win_T,
    mut config: *const KeyDict_win_config,
    mut fconfig: *const WinConfig,
    mut err: *mut Error,
) -> bool {
    let mut win_tp: *mut tabpage_T = win_find_tabpage(win);
    let mut parent: *mut win_T = win;
    let mut parent_tp: *mut tabpage_T = win_tp;
    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
        != 0 as ::core::ffi::c_ulonglong
    {
        parent = find_window_by_handle((*fconfig).window, err);
        if parent.is_null() {
            return false_0 != 0;
        }
        parent_tp = win_find_tabpage(parent);
    }
    let mut curwin_moving_tp: bool = false_0 != 0;
    let mut altwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
    '_restore_curwin: {
        if win_tp != parent_tp {
            if !win_can_move_tp(win, win_tp, err) {
                return false_0 != 0;
            }
            altwin = win_find_altwin(win, win_tp);
            '_c2rust_label: {
                if !altwin.is_null() {
                } else {
                    __assert_fail(
                        b"altwin\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/api/win_config.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        671 as ::core::ffi::c_uint,
                        b"_Bool win_config_float_tp(win_T *, const KeyDict_win_config *, const WinConfig *, Error *)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if curwin.get() == win {
                curwin_moving_tp = true_0 != 0;
                win_goto(altwin);
                if curwin.get() == win {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Failed to switch away from window %d\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        (*win).handle,
                    );
                    return false_0 != 0;
                }
                win_tp = win_find_tabpage(win);
                parent_tp = win_find_tabpage(parent);
                if win_tp.is_null() || parent_tp.is_null() {
                    api_set_error(
                        err,
                        kErrorTypeException,
                        b"Target windows were closed\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_restore_curwin;
                } else if win_tp != parent_tp && !win_can_move_tp(win, win_tp, err) {
                    break '_restore_curwin;
                } else {
                    altwin = win_find_altwin(win, win_tp);
                    '_c2rust_label_0: {
                        if !altwin.is_null() {
                        } else {
                            __assert_fail(
                                b"altwin\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/api/win_config.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                696 as ::core::ffi::c_uint,
                                b"_Bool win_config_float_tp(win_T *, const KeyDict_win_config *, const WinConfig *, Error *)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                }
            }
        }
        if !(*win).w_floating {
            if win_new_float(win, false_0 != 0, *fconfig, err).is_null() {
                break '_restore_curwin;
            } else {
                redraw_later(win, UPD_NOT_VALID);
            }
        }
        if win_tp != parent_tp {
            win_remove(
                win,
                if win_tp == curtab.get() {
                    ::core::ptr::null_mut::<tabpage_T>()
                } else {
                    win_tp
                },
            );
            let mut append_tp: *mut tabpage_T = if parent_tp == curtab.get() {
                ::core::ptr::null_mut::<tabpage_T>()
            } else {
                parent_tp
            };
            win_append(lastwin_nofloating(append_tp), win, append_tp);
            if win_tp != curtab.get() && (*win_tp).tp_curwin == win {
                (*win_tp).tp_curwin = altwin;
            }
            ui_comp_remove_grid(&raw mut (*win).w_grid_alloc);
            redraw_later(win, UPD_NOT_VALID);
            set_must_redraw(UPD_NOT_VALID);
        }
        win_config_float(win, *fconfig);
        return true_0 != 0;
    }
    if curwin_moving_tp as ::core::ffi::c_int != 0 && win_valid(win) as ::core::ffi::c_int != 0 {
        win_goto(win);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn nvim_win_set_config(
    mut win: Window,
    mut config: *mut KeyDict_win_config,
    mut err: *mut Error,
) {
    let mut w: *mut win_T = find_window_by_handle(win, err);
    if w.is_null() {
        return;
    }
    let mut was_split: bool = !(*w).w_floating;
    let mut has_split: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
        != 0 as ::core::ffi::c_ulonglong;
    let mut has_vertical: bool = (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
        != 0 as ::core::ffi::c_ulonglong;
    let mut old_style: WinStyle = (*w).w_config.style;
    let mut fconfig: WinConfig = (*w).w_config;
    let mut to_split: bool = (*config).relative.size == 0 as size_t
        && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external
            != 0 as ::core::ffi::c_ulonglong
            && (*config).external as ::core::ffi::c_int != 0)
        && (has_split as ::core::ffi::c_int != 0
            || has_vertical as ::core::ffi::c_int != 0
            || was_split as ::core::ffi::c_int != 0);
    if !parse_win_config(
        w,
        config,
        &raw mut fconfig,
        !was_split || to_split as ::core::ffi::c_int != 0,
        err,
    ) {
        return;
    }
    if to_split {
        if !win_config_split(w, config, &raw mut fconfig, err) {
            return;
        }
    } else if !win_config_float_tp(w, config, &raw mut fconfig, err) {
        return;
    }
    if fconfig.style as ::core::ffi::c_uint
        == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
        && old_style as ::core::ffi::c_uint != fconfig.style as ::core::ffi::c_uint
    {
        win_set_minimal_style(w);
        didset_window_options(w, true_0 != 0);
        changed_window_setting(w);
    }
    if fconfig._cmdline_offset < INT_MAX {
        cmdline_win.set(w);
    } else if w == cmdline_win.get() && fconfig._cmdline_offset == INT_MAX {
        cmdline_win.set(::core::ptr::null_mut::<win_T>());
    }
}
unsafe extern "C" fn config_put_bordertext(
    mut config: *mut KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut bordertext_type: BorderTextType,
    mut arena: *mut Arena,
) {
    let mut vt: VirtText = VirtText {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<VirtTextChunk>(),
    };
    let mut align: AlignTextPos = kAlignLeft;
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            vt = (*fconfig).title_chunks;
            align = (*fconfig).title_pos;
        }
        1 => {
            vt = (*fconfig).footer_chunks;
            align = (*fconfig).footer_pos;
        }
        _ => {}
    }
    let mut bordertext: Array = virt_text_to_array(vt, true_0 != 0, arena);
    let mut pos: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    match align as ::core::ffi::c_uint {
        0 => {
            pos = b"left\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        1 => {
            pos = b"center\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        2 => {
            pos = b"right\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        _ => {}
    }
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title)
                as OptionalKeys;
            (*config).title = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: bordertext },
            };
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title_pos)
                as OptionalKeys;
            (*config).title_pos = cstr_as_string(pos);
        }
        1 => {
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer)
                as OptionalKeys;
            (*config).footer = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: bordertext },
            };
            (*config).is_set__win_config_ = ((*config).is_set__win_config_
                as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer_pos)
                as OptionalKeys;
            (*config).footer_pos = cstr_as_string(pos);
        }
        _ => {}
    };
}
pub unsafe extern "C" fn nvim_win_get_config(
    mut win: Window,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> KeyDict_win_config {
    static float_relative_str: GlobalCell<[*const ::core::ffi::c_char; 6]> = GlobalCell::new([
        b"editor\0".as_ptr() as *const ::core::ffi::c_char,
        b"win\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"mouse\0".as_ptr() as *const ::core::ffi::c_char,
        b"tabline\0".as_ptr() as *const ::core::ffi::c_char,
        b"laststatus\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    static win_split_str: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
        b"left\0".as_ptr() as *const ::core::ffi::c_char,
        b"right\0".as_ptr() as *const ::core::ffi::c_char,
        b"above\0".as_ptr() as *const ::core::ffi::c_char,
        b"below\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    static win_style_str: GlobalCell<[*const ::core::ffi::c_char; 2]> = GlobalCell::new([
        b"\0".as_ptr() as *const ::core::ffi::c_char,
        b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut rv: KeyDict_win_config = KEYDICT_INIT;
    let mut wp: *mut win_T = find_window_by_handle(win, err);
    if wp.is_null() {
        return rv;
    }
    let mut config: *mut WinConfig = &raw mut (*wp).w_config;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__focusable)
        as OptionalKeys;
    rv.focusable = (*config).focusable as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external)
        as OptionalKeys;
    rv.external = (*config).external as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__hide)
        as OptionalKeys;
    rv.hide = (*config).hide as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__mouse)
        as OptionalKeys;
    rv.mouse = (*config).mouse as Boolean;
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__style)
        as OptionalKeys;
    rv.style = cstr_as_string((*win_style_str.ptr())[(*config).style as usize]);
    if (*wp).w_floating {
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width)
            as OptionalKeys;
        rv.width = (*config).width as Integer;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height)
            as OptionalKeys;
        rv.height = (*config).height as Integer;
        if !(*config).external {
            if (*config).relative as ::core::ffi::c_uint
                == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                    | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win)
                    as OptionalKeys;
                rv.win = (*config).window;
                if (*config).bufpos.lnum >= 0 as linenr_T {
                    let mut pos: Array = arena_array(arena, 2 as size_t);
                    let c2rust_fresh2 = pos.size;
                    pos.size = pos.size.wrapping_add(1);
                    *pos.items.offset(c2rust_fresh2 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*config).bufpos.lnum as Integer,
                        },
                    };
                    let c2rust_fresh3 = pos.size;
                    pos.size = pos.size.wrapping_add(1);
                    *pos.items.offset(c2rust_fresh3 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*config).bufpos.col as Integer,
                        },
                    };
                    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__bufpos)
                        as OptionalKeys;
                    rv.bufpos = pos;
                }
            }
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__anchor)
                as OptionalKeys;
            rv.anchor = cstr_as_string(
                *(&raw const float_anchor_str as *const *const ::core::ffi::c_char)
                    .offset((*config).anchor as isize),
            );
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row)
                as OptionalKeys;
            rv.row = (*config).row as Float;
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col)
                as OptionalKeys;
            rv.col = (*config).col as Float;
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__zindex)
                as OptionalKeys;
            rv.zindex = (*config).zindex as Integer;
        }
        if (*config).border {
            let mut border: Array = arena_array(arena, 8 as size_t);
            let mut i: size_t = 0 as size_t;
            while i < 8 as size_t {
                let mut s: String_0 = cstrn_as_string(
                    &raw mut *(&raw mut (*config).border_chars as *mut [::core::ffi::c_char; 32])
                        .offset(i as isize) as *mut ::core::ffi::c_char,
                    MAX_SCHAR_SIZE as size_t,
                );
                let mut hi_id: ::core::ffi::c_int = (*config).border_hl_ids[i as usize];
                let mut hi_name: *mut ::core::ffi::c_char = syn_id2name(hi_id);
                if *hi_name.offset(0 as ::core::ffi::c_int as isize) != 0 {
                    let mut tuple: Array = arena_array(arena, 2 as size_t);
                    let c2rust_fresh4 = tuple.size;
                    tuple.size = tuple.size.wrapping_add(1);
                    *tuple.items.offset(c2rust_fresh4 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed { string: s },
                    };
                    let c2rust_fresh5 = tuple.size;
                    tuple.size = tuple.size.wrapping_add(1);
                    *tuple.items.offset(c2rust_fresh5 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstr_as_string(hi_name),
                        },
                    };
                    let c2rust_fresh6 = border.size;
                    border.size = border.size.wrapping_add(1);
                    *border.items.offset(c2rust_fresh6 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed { array: tuple },
                    };
                } else {
                    let c2rust_fresh7 = border.size;
                    border.size = border.size.wrapping_add(1);
                    *border.items.offset(c2rust_fresh7 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed { string: s },
                    };
                }
                i = i.wrapping_add(1);
            }
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border)
                as OptionalKeys;
            rv.border = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed { array: border },
            };
            if (*config).title {
                config_put_bordertext(&raw mut rv, config, kBorderTextTitle, arena);
            }
            if (*config).footer {
                config_put_bordertext(&raw mut rv, config, kBorderTextFooter, arena);
            }
        } else {
            rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
                | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border)
                as OptionalKeys;
            rv.border = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(b"none\0".as_ptr() as *const ::core::ffi::c_char),
                },
            };
        }
    } else if !(*config).external {
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width)
            as OptionalKeys;
        rv.width = (*wp).w_width as Integer;
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height)
            as OptionalKeys;
        rv.height = (*wp).w_height as Integer;
        let mut split: WinSplit = win_split_dir(wp);
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split)
            as OptionalKeys;
        rv.split = cstr_as_string((*win_split_str.ptr())[split as usize]);
    }
    let mut rel: *const ::core::ffi::c_char =
        if (*wp).w_floating as ::core::ffi::c_int != 0 && !(*config).external {
            (*float_relative_str.ptr())[(*config).relative as usize]
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        };
    rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
        | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__relative)
        as OptionalKeys;
    rv.relative = cstr_as_string(rel);
    if (*config)._cmdline_offset < INT_MAX {
        rv.is_set__win_config_ = (rv.is_set__win_config_ as ::core::ffi::c_ulonglong
            | (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config___cmdline_offset)
            as OptionalKeys;
        rv._cmdline_offset = (*config)._cmdline_offset as Integer;
    }
    return rv;
}
unsafe extern "C" fn parse_float_anchor(mut anchor: String_0, mut out: *mut FloatAnchor) -> bool {
    if anchor.size == 0 as size_t {
        *out = 0 as ::core::ffi::c_int;
    }
    let mut str: *mut ::core::ffi::c_char = anchor.data;
    if striequal(str, b"NW\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = 0 as ::core::ffi::c_int as FloatAnchor;
    } else if striequal(str, b"NE\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatAnchorEast as ::core::ffi::c_int as FloatAnchor;
    } else if striequal(str, b"SW\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatAnchorSouth as ::core::ffi::c_int as FloatAnchor;
    } else if striequal(str, b"SE\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = (kFloatAnchorSouth as ::core::ffi::c_int | kFloatAnchorEast as ::core::ffi::c_int)
            as FloatAnchor;
    } else {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn parse_float_relative(
    mut relative: String_0,
    mut out: *mut FloatRelative,
) -> bool {
    let mut str: *mut ::core::ffi::c_char = relative.data;
    if striequal(str, b"editor\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeEditor;
    } else if striequal(str, b"win\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeWindow;
    } else if striequal(str, b"cursor\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeCursor;
    } else if striequal(str, b"mouse\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeMouse;
    } else if striequal(str, b"tabline\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeTabline;
    } else if striequal(str, b"laststatus\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kFloatRelativeLaststatus;
    } else {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn parse_config_split(mut split: String_0, mut out: *mut WinSplit) -> bool {
    let mut str: *mut ::core::ffi::c_char = split.data;
    if striequal(str, b"left\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitLeft;
    } else if striequal(str, b"right\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitRight;
    } else if striequal(str, b"above\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitAbove;
    } else if striequal(str, b"below\0".as_ptr() as *const ::core::ffi::c_char) {
        *out = kWinSplitBelow;
    } else {
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C" fn parse_float_bufpos(mut bufpos: Array, mut out: *mut lpos_T) -> bool {
    if bufpos.size != 2 as size_t
        || (*bufpos.items.offset(0 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*bufpos.items.offset(1 as ::core::ffi::c_int as isize)).type_0 as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return false_0 != 0;
    }
    (*out).lnum = (*bufpos.items.offset(0 as ::core::ffi::c_int as isize))
        .data
        .integer as linenr_T;
    (*out).col = (*bufpos.items.offset(1 as ::core::ffi::c_int as isize))
        .data
        .integer as colnr_T;
    return true_0 != 0;
}
unsafe extern "C" fn parse_bordertext(
    mut bordertext: Object,
    mut bordertext_type: BorderTextType,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) {
    if bordertext.type_0 as ::core::ffi::c_uint
        != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        && bordertext.type_0 as ::core::ffi::c_uint
            != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        api_err_exp(
            err,
            b"title/footer\0".as_ptr() as *const ::core::ffi::c_char,
            b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
            api_typename(bordertext.type_0),
        );
        return;
    }
    if bordertext.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        && bordertext.data.array.size == 0 as size_t
    {
        api_err_exp(
            err,
            b"title/footer\0".as_ptr() as *const ::core::ffi::c_char,
            b"non-empty Array\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        return;
    }
    let mut is_present: *mut bool = ::core::ptr::null_mut::<bool>();
    let mut chunks: *mut VirtText = ::core::ptr::null_mut::<VirtText>();
    let mut width: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            is_present = &raw mut (*fconfig).title;
            chunks = &raw mut (*fconfig).title_chunks;
            width = &raw mut (*fconfig).title_width;
        }
        1 => {
            is_present = &raw mut (*fconfig).footer;
            chunks = &raw mut (*fconfig).footer_chunks;
            width = &raw mut (*fconfig).footer_width;
        }
        _ => {}
    }
    if bordertext.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if bordertext.data.string.size == 0 as size_t {
            *is_present = false_0 != 0;
            return;
        }
        (*chunks).capacity = 0 as size_t;
        (*chunks).size = (*chunks).capacity;
        (*chunks).items = ::core::ptr::null_mut::<VirtTextChunk>();
        if (*chunks).size == (*chunks).capacity {
            (*chunks).capacity = if (*chunks).capacity != 0 {
                (*chunks).capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*chunks).items = xrealloc(
                (*chunks).items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<VirtTextChunk>().wrapping_mul((*chunks).capacity),
            ) as *mut VirtTextChunk;
        } else {
        };
        let c2rust_fresh1 = (*chunks).size;
        (*chunks).size = (*chunks).size.wrapping_add(1);
        *(*chunks).items.offset(c2rust_fresh1 as isize) = VirtTextChunk {
            text: xstrdup(bordertext.data.string.data),
            hl_id: -1 as ::core::ffi::c_int,
        };
        *width = mb_string2cells(bordertext.data.string.data) as ::core::ffi::c_int;
        *is_present = true_0 != 0;
        return;
    }
    *width = 0 as ::core::ffi::c_int;
    *chunks = parse_virt_text(bordertext.data.array, err, width);
    *is_present = true_0 != 0;
}
unsafe extern "C" fn parse_bordertext_pos(
    mut wp: *mut win_T,
    mut bordertext_pos: String_0,
    mut bordertext_type: BorderTextType,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) -> bool {
    let mut align: *mut AlignTextPos = ::core::ptr::null_mut::<AlignTextPos>();
    match bordertext_type as ::core::ffi::c_uint {
        0 => {
            align = &raw mut (*fconfig).title_pos;
        }
        1 => {
            align = &raw mut (*fconfig).footer_pos;
        }
        _ => {}
    }
    if bordertext_pos.size == 0 as size_t {
        if wp.is_null() {
            *align = kAlignLeft;
        }
        return true_0 != 0;
    }
    let mut pos: *mut ::core::ffi::c_char = bordertext_pos.data;
    if strequal(pos, b"left\0".as_ptr() as *const ::core::ffi::c_char) {
        *align = kAlignLeft;
    } else if strequal(pos, b"center\0".as_ptr() as *const ::core::ffi::c_char) {
        *align = kAlignCenter;
    } else if strequal(pos, b"right\0".as_ptr() as *const ::core::ffi::c_char) {
        *align = kAlignRight;
    } else if true {
        api_err_invalid(
            err,
            if bordertext_type as ::core::ffi::c_uint
                == kBorderTextTitle as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"title_pos\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"footer_pos\0".as_ptr() as *const ::core::ffi::c_char
            },
            pos,
            0 as int64_t,
            true_0 != 0,
        );
        return false;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn parse_border_style(
    mut style: Object,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) {
    let mut defaults: [C2Rust_Unnamed_15; 7] = [
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[1 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x9D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x9A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[2 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x8C\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x98\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[3 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: true_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[4 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xAD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xAE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xAF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x95\xB0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[5 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: (*opt_winborder_values.ptr())[6 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
            chars: [
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x8F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x93\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x9B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
                ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                    *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                ),
            ],
            shadow_color: false_0 != 0,
        },
        C2Rust_Unnamed_15 {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            chars: [
                [
                    NUL as ::core::ffi::c_char,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
                [0; 32],
            ],
            shadow_color: false_0 != 0,
        },
    ];
    let mut chars: *mut [::core::ffi::c_char; 32] =
        &raw mut (*fconfig).border_chars as *mut [::core::ffi::c_char; 32];
    let mut hl_ids: *mut ::core::ffi::c_int =
        &raw mut (*fconfig).border_hl_ids as *mut ::core::ffi::c_int;
    (*fconfig).border = true_0 != 0;
    if style.type_0 as ::core::ffi::c_uint
        == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut arr: Array = style.data.array;
        let mut size: size_t = arr.size;
        if size == 0 || size > 8 as size_t || size & size.wrapping_sub(1 as size_t) != 0 {
            api_err_exp(
                err,
                b"border\0".as_ptr() as *const ::core::ffi::c_char,
                b"1, 2, 4, or 8 chars\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return;
        }
        let mut i: size_t = 0 as size_t;
        while i < size {
            let mut iytem: Object = *arr.items.offset(i as isize);
            let mut string: String_0 = String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            };
            let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if iytem.type_0 as ::core::ffi::c_uint
                == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut iarr: Array = iytem.data.array;
                if iarr.size == 0 || iarr.size > 2 as size_t {
                    api_err_exp(
                        err,
                        b"border\0".as_ptr() as *const ::core::ffi::c_char,
                        b"1 or 2-item Array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return;
                }
                if !((*iarr.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint)
                {
                    api_err_exp(
                        err,
                        b"border\0".as_ptr() as *const ::core::ffi::c_char,
                        b"Array of Strings\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return;
                }
                string = (*iarr.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .string;
                if iarr.size == 2 as size_t {
                    hl_id = object_to_hl_id(
                        *iarr.items.offset(1 as ::core::ffi::c_int as isize),
                        b"border char highlight\0".as_ptr() as *const ::core::ffi::c_char,
                        err,
                    );
                    if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                        return;
                    }
                }
            } else if iytem.type_0 as ::core::ffi::c_uint
                == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                string = iytem.data.string;
            } else if true {
                api_err_exp(
                    err,
                    b"border\0".as_ptr() as *const ::core::ffi::c_char,
                    b"String or Array\0".as_ptr() as *const ::core::ffi::c_char,
                    api_typename(iytem.type_0),
                );
                return;
            }
            if string.size != 0 && mb_string2cells_len(string.data, string.size) > 1 as size_t {
                api_err_exp(
                    err,
                    b"border\0".as_ptr() as *const ::core::ffi::c_char,
                    b"only one-cell chars\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return;
            }
            let mut len: size_t = if string.size
                < ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as usize)
            {
                string.size
            } else {
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as size_t)
            };
            if len != 0 {
                memcpy(
                    &raw mut *chars.offset(i as isize) as *mut ::core::ffi::c_char
                        as *mut ::core::ffi::c_void,
                    string.data as *const ::core::ffi::c_void,
                    len,
                );
            }
            (*chars.offset(i as isize))[len as usize] = NUL as ::core::ffi::c_char;
            *hl_ids.offset(i as isize) = hl_id;
            i = i.wrapping_add(1);
        }
        while size < 8 as size_t {
            memcpy(
                chars.offset(size as isize) as *mut ::core::ffi::c_void,
                chars as *const ::core::ffi::c_void,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_mul(size),
            );
            memcpy(
                hl_ids.offset(size as isize) as *mut ::core::ffi::c_void,
                hl_ids as *const ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(size),
            );
            size <<= 1 as ::core::ffi::c_int;
        }
        if (*chars.offset(7 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            != 0
            && (*chars.offset(1 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
            && (*chars.offset(0 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                == 0
            || (*chars.offset(1 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(3 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(2 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
            || (*chars.offset(3 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(5 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(4 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
            || (*chars.offset(5 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(7 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(6 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
        {
            api_err_exp(
                err,
                b"border\0".as_ptr() as *const ::core::ffi::c_char,
                b"corner char between edge chars\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return;
        }
    } else if style.type_0 as ::core::ffi::c_uint
        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut str: String_0 = style.data.string;
        if str.size == 0 as size_t
            || strequal(str.data, b"none\0".as_ptr() as *const ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
        {
            (*fconfig).border = false_0 != 0;
            (*fconfig).title = false_0 != 0;
            (*fconfig).footer = false_0 != 0;
            return;
        }
        let mut i_0: size_t = 0 as size_t;
        while !defaults[i_0 as usize].name.is_null() {
            if strequal(str.data, defaults[i_0 as usize].name) {
                memcpy(
                    chars as *mut ::core::ffi::c_void,
                    &raw mut (*(&raw mut defaults as *mut C2Rust_Unnamed_15).offset(i_0 as isize))
                        .chars as *mut [::core::ffi::c_char; 32]
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[[::core::ffi::c_char; 32]; 8]>(),
                );
                memset(
                    hl_ids as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    (8 as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
                );
                if defaults[i_0 as usize].shadow_color {
                    let mut hl_blend: ::core::ffi::c_int = syn_check_group(
                        b"FloatShadow\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 12]>()
                            .wrapping_sub(1 as size_t),
                    );
                    let mut hl_through: ::core::ffi::c_int = syn_check_group(
                        b"FloatShadowThrough\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 19]>()
                            .wrapping_sub(1 as size_t),
                    );
                    *hl_ids.offset(2 as ::core::ffi::c_int as isize) = hl_through;
                    *hl_ids.offset(3 as ::core::ffi::c_int as isize) = hl_blend;
                    *hl_ids.offset(4 as ::core::ffi::c_int as isize) = hl_blend;
                    *hl_ids.offset(5 as ::core::ffi::c_int as isize) = hl_blend;
                    *hl_ids.offset(6 as ::core::ffi::c_int as isize) = hl_through;
                }
                return;
            }
            i_0 = i_0.wrapping_add(1);
        }
        if true {
            api_err_invalid(
                err,
                b"border\0".as_ptr() as *const ::core::ffi::c_char,
                str.data,
                0 as int64_t,
                true_0 != 0,
            );
            return;
        }
    }
}
unsafe extern "C" fn generate_api_error(
    mut wp: *mut win_T,
    mut attribute: *const ::core::ffi::c_char,
    mut err: *mut Error,
) {
    if !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Required: 'relative' when reconfiguring floating window %d\0".as_ptr()
                as *const ::core::ffi::c_char,
            (*wp).handle,
        );
    } else if true {
        api_err_conflict(
            err,
            attribute,
            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}
pub unsafe extern "C" fn parse_winborder(
    mut fconfig: *mut WinConfig,
    mut border_opt: *mut ::core::ffi::c_char,
    mut err: *mut Error,
) -> bool {
    if fconfig.is_null() {
        return false_0 != 0;
    }
    let mut style: Object = object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    if !strchr(border_opt, ',' as ::core::ffi::c_int).is_null() {
        let mut border_chars: Array = ARRAY_DICT_INIT;
        let mut p: *mut ::core::ffi::c_char = border_opt;
        let mut part: [::core::ffi::c_char; 32] = [
            0 as ::core::ffi::c_char,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != NUL {
            if count >= 8 as ::core::ffi::c_int {
                api_free_array(border_chars);
                return false_0 != 0;
            }
            let mut part_len: size_t = copy_option_part(
                &raw mut p,
                &raw mut part as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if part_len == 0 as size_t
                || part[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
            {
                api_free_array(border_chars);
                return false_0 != 0;
            }
            let mut str: String_0 = cstr_to_string(&raw mut part as *mut ::core::ffi::c_char);
            if border_chars.size == border_chars.capacity {
                border_chars.capacity = if border_chars.capacity != 0 {
                    border_chars.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                border_chars.items = xrealloc(
                    border_chars.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(border_chars.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh0 = border_chars.size;
            border_chars.size = border_chars.size.wrapping_add(1);
            *border_chars.items.offset(c2rust_fresh0 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed { string: str },
            };
            count += 1;
        }
        if count != 8 as ::core::ffi::c_int {
            api_free_array(border_chars);
            return false_0 != 0;
        }
        style = object {
            type_0: kObjectTypeArray,
            data: C2Rust_Unnamed {
                array: border_chars,
            },
        };
    } else {
        style = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed {
                string: cstr_to_string(border_opt),
            },
        };
    }
    parse_border_style(style, fconfig, err);
    api_free_object(style);
    return !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int);
}
unsafe extern "C" fn parse_win_config(
    mut wp: *mut win_T,
    mut config: *mut KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut reconf: bool,
    mut err: *mut Error,
) -> bool {
    let mut border_style: Object = Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    };
    let mut has_relative: bool = false_0 != 0;
    let mut relative_is_win: bool = false_0 != 0;
    let mut is_split: bool = false_0 != 0;
    '_fail: {
        if (*config).relative.size > 0 as size_t {
            if !parse_float_relative((*config).relative, &raw mut (*fconfig).relative) {
                api_err_invalid(
                    err,
                    b"relative\0".as_ptr() as *const ::core::ffi::c_char,
                    (*config).relative.data,
                    0 as int64_t,
                    true_0 != 0,
                );
                break '_fail;
            } else if (*config).relative.size > 0 as size_t
                && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 2 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                    && (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << 1 as ::core::ffi::c_int
                        != 0 as ::core::ffi::c_ulonglong)
                && !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 12 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong)
            {
                api_err_required(
                    err,
                    b"'relative' requires 'row'/'col' or 'bufpos'\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                break '_fail;
            } else {
                has_relative = true_0 != 0;
                (*fconfig).external = false_0 != 0;
                if (*fconfig).relative as ::core::ffi::c_uint
                    == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    relative_is_win = true_0 != 0;
                    (*fconfig).bufpos.lnum = -1 as ::core::ffi::c_int as linenr_T;
                }
            }
        } else if !(*config).external {
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__vertical
                != 0 as ::core::ffi::c_ulonglong
                || (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
                    != 0 as ::core::ffi::c_ulonglong
            {
                is_split = true_0 != 0;
                (*fconfig).external = false_0 != 0;
            } else if wp.is_null() {
                if true {
                    api_err_required(
                        err,
                        b"'relative' or 'external' when creating a float\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
            }
        }
        if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 19 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
            && !is_split
        {
            api_err_conflict(
                err,
                b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
                b"floating windows\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << 6 as ::core::ffi::c_int
            != 0 as ::core::ffi::c_ulonglong
            && !is_split
        {
            api_err_conflict(
                err,
                b"split\0".as_ptr() as *const ::core::ffi::c_char,
                b"floating windows\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__split
                != 0 as ::core::ffi::c_ulonglong
            {
                if !is_split {
                    api_err_conflict(
                        err,
                        b"split\0".as_ptr() as *const ::core::ffi::c_char,
                        b"floating windows\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                } else if !parse_config_split((*config).split, &raw mut (*fconfig).split) {
                    api_err_invalid(
                        err,
                        b"split\0".as_ptr() as *const ::core::ffi::c_char,
                        (*config).split.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__anchor
                != 0 as ::core::ffi::c_ulonglong
            {
                if !parse_float_anchor((*config).anchor, &raw mut (*fconfig).anchor) {
                    api_err_invalid(
                        err,
                        b"anchor\0".as_ptr() as *const ::core::ffi::c_char,
                        (*config).anchor.data,
                        0 as int64_t,
                        true_0 != 0,
                    );
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row
                != 0 as ::core::ffi::c_ulonglong
            {
                if !has_relative || is_split as ::core::ffi::c_int != 0 {
                    generate_api_error(wp, b"row\0".as_ptr() as *const ::core::ffi::c_char, err);
                    break '_fail;
                } else {
                    (*fconfig).row = (*config).row as ::core::ffi::c_double;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col
                != 0 as ::core::ffi::c_ulonglong
            {
                if !has_relative || is_split as ::core::ffi::c_int != 0 {
                    generate_api_error(wp, b"col\0".as_ptr() as *const ::core::ffi::c_char, err);
                    break '_fail;
                } else {
                    (*fconfig).col = (*config).col as ::core::ffi::c_double;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__bufpos
                != 0 as ::core::ffi::c_ulonglong
            {
                if !has_relative || is_split as ::core::ffi::c_int != 0 {
                    generate_api_error(wp, b"bufpos\0".as_ptr() as *const ::core::ffi::c_char, err);
                    break '_fail;
                } else if !parse_float_bufpos((*config).bufpos, &raw mut (*fconfig).bufpos) {
                    api_err_exp(
                        err,
                        b"bufpos\0".as_ptr() as *const ::core::ffi::c_char,
                        b"[row, col] array\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_fail;
                } else {
                    if !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__row
                        != 0 as ::core::ffi::c_ulonglong)
                    {
                        (*fconfig).row = (if (*fconfig).anchor as ::core::ffi::c_int
                            & kFloatAnchorSouth as ::core::ffi::c_int
                            != 0
                        {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        }) as ::core::ffi::c_double;
                    }
                    if !((*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__col
                        != 0 as ::core::ffi::c_ulonglong)
                    {
                        (*fconfig).col = 0 as ::core::ffi::c_int as ::core::ffi::c_double;
                    }
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__width
                != 0 as ::core::ffi::c_ulonglong
            {
                if !((*config).width > 0 as Integer) {
                    api_err_exp(
                        err,
                        b"width\0".as_ptr() as *const ::core::ffi::c_char,
                        b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_fail;
                } else {
                    (*fconfig).width = (*config).width as ::core::ffi::c_int;
                }
            } else if !reconf && !is_split {
                if true {
                    api_err_required(err, b"width\0".as_ptr() as *const ::core::ffi::c_char);
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__height
                != 0 as ::core::ffi::c_ulonglong
            {
                if !((*config).height > 0 as Integer) {
                    api_err_exp(
                        err,
                        b"height\0".as_ptr() as *const ::core::ffi::c_char,
                        b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    break '_fail;
                } else {
                    (*fconfig).height = (*config).height as ::core::ffi::c_int;
                }
            } else if !reconf && !is_split {
                if true {
                    api_err_required(err, b"height\0".as_ptr() as *const ::core::ffi::c_char);
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__external
                != 0 as ::core::ffi::c_ulonglong
            {
                (*fconfig).external = (*config).external as bool;
                if has_relative as ::core::ffi::c_int != 0
                    && (*fconfig).external as ::core::ffi::c_int != 0
                {
                    api_err_conflict(
                        err,
                        b"relative\0".as_ptr() as *const ::core::ffi::c_char,
                        b"external\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                } else if (*fconfig).external as ::core::ffi::c_int != 0 && !ui_has(kUIMultigrid) {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"UI doesn't support external windows\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
            }
            if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 3 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong
                && (*fconfig).external as ::core::ffi::c_int != 0
            {
                api_err_conflict(
                    err,
                    b"win\0".as_ptr() as *const ::core::ffi::c_char,
                    b"external window\0".as_ptr() as *const ::core::ffi::c_char,
                );
            } else {
                if relative_is_win as ::core::ffi::c_int != 0
                    || (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
                        != 0 as ::core::ffi::c_ulonglong
                        && !is_split
                        && !wp.is_null()
                        && (*wp).w_floating as ::core::ffi::c_int != 0
                        && (*fconfig).relative as ::core::ffi::c_uint
                            == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut target_win: *mut win_T = find_window_by_handle((*config).win, err);
                    if target_win.is_null() {
                        break '_fail;
                    } else if target_win == wp {
                        api_set_error(
                            err,
                            kErrorTypeException,
                            b"floating window cannot be relative to itself\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        (*fconfig).window = (*target_win).handle as Window;
                    }
                } else {
                    if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__win
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if !is_split && !has_relative && (wp.is_null() || !(*wp).w_floating) {
                            api_err_required(
                                err,
                                b"non-float with 'win' requires 'split' or 'vertical'\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                            break '_fail;
                        } else {
                            (*fconfig).window = (*config).win;
                        }
                    }
                    if (*fconfig).window == 0 as ::core::ffi::c_int {
                        (*fconfig).window = (*curwin.get()).handle as Window;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__focusable
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).focusable = (*config).focusable as bool;
                    (*fconfig).mouse = (*config).focusable as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__mouse
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).mouse = (*config).mouse as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__zindex
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"zindex\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else if !((*config).zindex > 0 as Integer) {
                        api_err_exp(
                            err,
                            b"zindex\0".as_ptr() as *const ::core::ffi::c_char,
                            b"positive Integer\0".as_ptr() as *const ::core::ffi::c_char,
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_fail;
                    } else {
                        (*fconfig).zindex = (*config).zindex as ::core::ffi::c_int;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__title
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"title\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        parse_bordertext((*config).title, kBorderTextTitle, fconfig, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_fail;
                        } else if !parse_bordertext_pos(
                            wp,
                            (*config).title_pos,
                            kBorderTextTitle,
                            fconfig,
                            err,
                        ) {
                            break '_fail;
                        }
                    }
                } else if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 22 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                {
                    api_err_required(
                        err,
                        b"'title' requires 'title_pos'\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__footer
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"footer\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        parse_bordertext((*config).footer, kBorderTextFooter, fconfig, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_fail;
                        } else if !parse_bordertext_pos(
                            wp,
                            (*config).footer_pos,
                            kBorderTextFooter,
                            fconfig,
                            err,
                        ) {
                            break '_fail;
                        }
                    }
                } else if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 23 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong
                {
                    api_err_required(
                        err,
                        b"'footer' requires 'footer_pos'\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_fail;
                }
                border_style = object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                };
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__border
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if is_split {
                        api_err_conflict(
                            err,
                            b"border\0".as_ptr() as *const ::core::ffi::c_char,
                            b"non-float window\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        border_style = (*config).border;
                        if border_style.type_0 as ::core::ffi::c_uint
                            != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            parse_border_style(border_style, fconfig, err);
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_fail;
                            }
                        }
                    }
                } else if *p_winborder.get() as ::core::ffi::c_int != NUL
                    && (wp.is_null() || !(*wp).w_floating)
                    && !parse_winborder(fconfig, p_winborder.get(), err)
                {
                    break '_fail;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__style
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if *(*config)
                        .style
                        .data
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == NUL
                    {
                        (*fconfig).style = kWinStyleUnused;
                    } else if striequal(
                        (*config).style.data,
                        b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        (*fconfig).style = kWinStyleMinimal;
                    } else if true {
                        api_err_invalid(
                            err,
                            b"style\0".as_ptr() as *const ::core::ffi::c_char,
                            (*config).style.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_fail;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__noautocmd
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if !wp.is_null()
                        && (*config).noautocmd as ::core::ffi::c_int
                            != (*fconfig).noautocmd as ::core::ffi::c_int
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            b"'noautocmd' cannot be changed on existing window\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                        break '_fail;
                    } else {
                        (*fconfig).noautocmd = (*config).noautocmd as bool;
                    }
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__fixed
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).fixed = (*config).fixed as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config__hide
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig).hide = (*config).hide as bool;
                }
                if (*config).is_set__win_config_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_win_config___cmdline_offset
                    != 0 as ::core::ffi::c_ulonglong
                {
                    (*fconfig)._cmdline_offset = (*config)._cmdline_offset as ::core::ffi::c_int;
                }
                return true_0 != 0;
            }
        }
    }
    merge_win_config(
        fconfig,
        if !wp.is_null() {
            (*wp).w_config
        } else {
            WinConfig {
                window: 0,
                bufpos: lpos_T {
                    lnum: -1 as linenr_T,
                    col: 0 as colnr_T,
                },
                height: 0 as ::core::ffi::c_int,
                width: 0 as ::core::ffi::c_int,
                row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                anchor: 0 as FloatAnchor,
                relative: kFloatRelativeEditor,
                external: false_0 != 0,
                focusable: true_0 != 0,
                mouse: true_0 != 0,
                split: kWinSplitLeft,
                zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                style: kWinStyleUnused,
                border: false,
                shadow: false,
                border_chars: [[0; 32]; 8],
                border_hl_ids: [0; 8],
                border_attr: [0; 8],
                title: false,
                title_pos: kAlignLeft,
                title_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                title_width: 0,
                footer: false,
                footer_pos: kAlignLeft,
                footer_chunks: VirtText {
                    size: 0,
                    capacity: 0,
                    items: ::core::ptr::null_mut::<VirtTextChunk>(),
                },
                footer_width: 0,
                noautocmd: false_0 != 0,
                fixed: false_0 != 0,
                hide: false_0 != 0,
                _cmdline_offset: INT_MAX,
            }
        },
    );
    return false_0 != 0;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
