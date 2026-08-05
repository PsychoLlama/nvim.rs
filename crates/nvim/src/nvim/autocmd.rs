use crate::src::nvim::api::private::converter::object_to_vim;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_object, api_free_string, copy_object, cstr_as_string, cstr_to_string,
    find_buffer_by_handle,
};
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::buffer::{bt_prompt, buflist_findnr, bufref_valid, do_modelines, set_bufref};
use crate::src::nvim::charset::{skipdigits, skipwhite};
use crate::src::nvim::cursor::{check_cursor, check_pos};
use crate::src::nvim::eval::typval::{
    callback_copy, callback_free, callback_to_string, kCallbackLua, kCallbackNone, tv_clear,
    tv_dict_add_nr, tv_dict_add_tv, tv_dict_set_keys_readonly,
};
use crate::src::nvim::eval::userfunc::{restore_funccal, save_funccal};
use crate::src::nvim::eval::vars::{
    get_vim_var_nr, get_vim_var_str, set_cmdarg, set_vim_var_nr, vars_clear,
};
use crate::src::nvim::eval::{callback_call, get_v_event, last_set_msg, restore_v_event};
use crate::src::nvim::event::multiqueue::{multiqueue_new_child, multiqueue_put_event};
use crate::src::nvim::ex_docmd::{
    do_cmdline, ends_excmd, expand_sfile, get_pressedreturn, set_pressedreturn,
};
use crate::src::nvim::ex_eval::{aborting, should_abort};
use crate::src::nvim::fileio::{check_timestamps, file_pat_to_reg_pat, match_file_pat};
use crate::src::nvim::getchar::{restoreRedobuff, saveRedobuff};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::grid_free;
use crate::src::nvim::hashtab::hash_init;
use crate::src::nvim::highlight_group::{HLF_8, HLF_E, HLF_T};
use crate::src::nvim::insexpand::ins_compl_active;
use crate::src::nvim::lua::executor::{nlua_call_ref, nlua_set_sctx};
use crate::src::nvim::main::aucmd_win_vec;
use crate::src::nvim::main::{
    KeyTyped, RedrawingDisabled, VIsual, VIsual_active, au_pending_free_buf, au_pending_free_win,
    autocmd_bufnr, autocmd_busy, autocmd_fname, autocmd_fname_full, autocmd_match,
    autocmd_no_enter, autocmd_no_leave, curbuf, current_sctx, curtab, curwin, deferred_events,
    did_cursorhold, did_emsg, do_profiling, e_argreq, e_cannot_define_autocommands_for_all_events,
    e_duparg2, first_tabpage, firstbuf, firstwin, globaldir, got_int, last_cursormoved,
    last_cursormoved_win, last_mode, lastwin, main_loop, msg_col, need_maketitle, p_acd, p_ei,
    p_verbose, prevwin, reg_recording, secure, starting, typebuf, window_handles,
};
use crate::src::nvim::map::{
    map_del_String_int, map_del_int_String, map_del_int_ptr_t, map_put_ref_String_int,
    map_put_ref_int_String, map_put_ref_int_ptr_t, mh_get_String, mh_get_int,
};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup};
use crate::src::nvim::message::{
    emsg, give_warning, msg_advance, msg_clr_eos, msg_end, msg_ext_set_kind, msg_outtrans,
    msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, msg_start, semsg, smsg, verbose_enter,
    verbose_enter_scroll, verbose_leave, verbose_leave_scroll,
};
use crate::src::nvim::option::set_option_direct;
use crate::src::nvim::options::kOptEventignore;
use crate::src::nvim::os::env::expand_env_save;
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, abs, atoi, gettext, snprintf, strcasecmp, strchr, strcpy, strlen,
    strncasecmp, strncmp,
};
use crate::src::nvim::os::time::os_now;
use crate::src::nvim::path::{FullName_save, path_fnamecmp, path_tail};
use crate::src::nvim::profile::{prof_child_enter, prof_child_exit};
use crate::src::nvim::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::src::nvim::runtime::{estack_pop, estack_push, exestack};
use crate::src::nvim::search::{restore_search_patterns, save_search_patterns};
use crate::src::nvim::state::{MODE_INSERT, MODE_NORMAL_BUSY, get_mode, get_real_state};
use crate::src::nvim::strings::{vim_strchr, vim_strnicmp_asc, xstrnsave};
use crate::src::nvim::types::{
    Arena, Array, AutoCmd, AutoCmdVec, AutoPat, AutoPatCmd, AutoPatCmd_S, Buffer, Callback,
    Callback_data as C2Rust_Unnamed_5, Dict, Error, Event, Integer, KeyValuePair, LuaRetMode,
    Map_String_int, Map_int_String, Map_int_ptr_t, MapHash, Object, OptInt, OptVal, OptValData,
    OptValType, Set_String, Set_int, String_0, Timestamp, TriState, VAR_UNKNOWN, VAR_UNLOCKED,
    VV_CMDBANG, VV_TERMRESPONSE, aco_save_T, aucmdwin_T, auto_event, buf_T, buffblock, buffblock_T,
    buffheader_T, bufref_T, dict_T, estack_T, etype_T, event_T, exarg_T, expand_T, funccal_entry_T,
    handle_T, hashitem_T, hashtab_T, int64_t, kErrorTypeNone, kFalse, kNone, kObjectTypeBoolean,
    kObjectTypeDict, kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, kTrue, key_value_pair,
    linenr_T, object, object_data as C2Rust_Unnamed, proftime_T, ptr_t, ptrdiff_t, regprog_T,
    save_redo_T, save_v_event_T, sctx_T, size_t, tabpage_T, typval_T, typval_vval_union, uint32_t,
    uint64_t, varnumber_T, win_T,
};
use crate::src::nvim::ui::ui_call_win_hide;
use crate::src::nvim::ui_compositor::ui_comp_remove_grid;
use crate::src::nvim::window::{
    check_lnums, check_lnums_nested, close_tabpage, entering_window, goto_tabpage_tp, reset_lnums,
    snapshot_windows_scroll_size, unuse_tabpage, use_tabpage, valid_tabpage_win,
    win_alloc_aucmd_win, win_append, win_enter, win_find_by_handle, win_fix_current_dir, win_goto,
    win_init_empty, win_remove,
};
use crate::src::nvim::winfloat::win_config_float;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const EXPAND_EVENTS: C2Rust_Unnamed_28 = 10;
pub const EXPAND_FILES: C2Rust_Unnamed_28 = 2;
pub const EXPAND_NOTHING: C2Rust_Unnamed_28 = 0;
pub const kOptValTypeString: OptValType = 2;
pub const NUM_EVENTS: auto_event = 145;
pub const EVENT_WINSCROLLED: auto_event = 144;
pub const EVENT_WINRESIZED: auto_event = 143;
pub const EVENT_WINNEWPRE: auto_event = 142;
pub const EVENT_WINNEW: auto_event = 141;
pub const EVENT_WINLEAVE: auto_event = 140;
pub const EVENT_WINENTER: auto_event = 139;
pub const EVENT_WINCLOSED: auto_event = 138;
pub const EVENT_VIMSUSPEND: auto_event = 137;
pub const EVENT_VIMRESUME: auto_event = 136;
pub const EVENT_VIMRESIZED: auto_event = 135;
pub const EVENT_VIMLEAVEPRE: auto_event = 134;
pub const EVENT_VIMLEAVE: auto_event = 133;
pub const EVENT_VIMENTER: auto_event = 132;
pub const EVENT_USER: auto_event = 131;
pub const EVENT_UILEAVE: auto_event = 130;
pub const EVENT_UIENTER: auto_event = 129;
pub const EVENT_TEXTYANKPOST: auto_event = 128;
pub const EVENT_TEXTCHANGEDT: auto_event = 127;
pub const EVENT_TEXTCHANGEDP: auto_event = 126;
pub const EVENT_TEXTCHANGEDI: auto_event = 125;
pub const EVENT_TEXTCHANGED: auto_event = 124;
pub const EVENT_TERMRESPONSE: auto_event = 123;
pub const EVENT_TERMREQUEST: auto_event = 122;
pub const EVENT_TERMOPEN: auto_event = 121;
pub const EVENT_TERMLEAVE: auto_event = 120;
pub const EVENT_TERMENTER: auto_event = 119;
pub const EVENT_TERMCLOSE: auto_event = 118;
pub const EVENT_TERMCHANGED: auto_event = 117;
pub const EVENT_TABNEWENTERED: auto_event = 116;
pub const EVENT_TABNEW: auto_event = 115;
pub const EVENT_TABLEAVE: auto_event = 114;
pub const EVENT_TABENTER: auto_event = 113;
pub const EVENT_TABCLOSEDPRE: auto_event = 112;
pub const EVENT_TABCLOSED: auto_event = 111;
pub const EVENT_SYNTAX: auto_event = 110;
pub const EVENT_SWAPEXISTS: auto_event = 109;
pub const EVENT_STDINREADPRE: auto_event = 108;
pub const EVENT_STDINREADPOST: auto_event = 107;
pub const EVENT_SPELLFILEMISSING: auto_event = 106;
pub const EVENT_SOURCEPRE: auto_event = 105;
pub const EVENT_SOURCEPOST: auto_event = 104;
pub const EVENT_SOURCECMD: auto_event = 103;
pub const EVENT_SIGNAL: auto_event = 102;
pub const EVENT_SHELLFILTERPOST: auto_event = 101;
pub const EVENT_SHELLCMDPOST: auto_event = 100;
pub const EVENT_SESSIONWRITEPOST: auto_event = 99;
pub const EVENT_SESSIONLOADPRE: auto_event = 98;
pub const EVENT_SESSIONLOADPOST: auto_event = 97;
pub const EVENT_SEARCHWRAPPED: auto_event = 96;
pub const EVENT_SAFESTATE: auto_event = 95;
pub const EVENT_REMOTEREPLY: auto_event = 94;
pub const EVENT_RECORDINGLEAVE: auto_event = 93;
pub const EVENT_RECORDINGENTER: auto_event = 92;
pub const EVENT_QUITPRE: auto_event = 91;
pub const EVENT_QUICKFIXCMDPRE: auto_event = 90;
pub const EVENT_QUICKFIXCMDPOST: auto_event = 89;
pub const EVENT_PROGRESS: auto_event = 88;
pub const EVENT_PACKCHANGEDPRE: auto_event = 87;
pub const EVENT_PACKCHANGED: auto_event = 86;
pub const EVENT_OPTIONSET: auto_event = 85;
pub const EVENT_MODECHANGED: auto_event = 84;
pub const EVENT_MENUPOPUP: auto_event = 83;
pub const EVENT_MARKSET: auto_event = 82;
pub const EVENT_LSPTOKENUPDATE: auto_event = 81;
pub const EVENT_LSPREQUEST: auto_event = 80;
pub const EVENT_LSPPROGRESS: auto_event = 79;
pub const EVENT_LSPNOTIFY: auto_event = 78;
pub const EVENT_LSPDETACH: auto_event = 77;
pub const EVENT_LSPATTACH: auto_event = 76;
pub const EVENT_INSERTLEAVEPRE: auto_event = 75;
pub const EVENT_INSERTLEAVE: auto_event = 74;
pub const EVENT_INSERTENTER: auto_event = 73;
pub const EVENT_INSERTCHARPRE: auto_event = 72;
pub const EVENT_INSERTCHANGE: auto_event = 71;
pub const EVENT_GUIFAILED: auto_event = 70;
pub const EVENT_GUIENTER: auto_event = 69;
pub const EVENT_FUNCUNDEFINED: auto_event = 68;
pub const EVENT_FOCUSLOST: auto_event = 67;
pub const EVENT_FOCUSGAINED: auto_event = 66;
pub const EVENT_FILTERWRITEPRE: auto_event = 65;
pub const EVENT_FILTERWRITEPOST: auto_event = 64;
pub const EVENT_FILTERREADPRE: auto_event = 63;
pub const EVENT_FILTERREADPOST: auto_event = 62;
pub const EVENT_FILEWRITEPRE: auto_event = 61;
pub const EVENT_FILEWRITEPOST: auto_event = 60;
pub const EVENT_FILEWRITECMD: auto_event = 59;
pub const EVENT_FILETYPE: auto_event = 58;
pub const EVENT_FILEREADPRE: auto_event = 57;
pub const EVENT_FILEREADPOST: auto_event = 56;
pub const EVENT_FILEREADCMD: auto_event = 55;
pub const EVENT_FILEENCODING: auto_event = 54;
pub const EVENT_FILECHANGEDSHELLPOST: auto_event = 53;
pub const EVENT_FILECHANGEDSHELL: auto_event = 52;
pub const EVENT_FILECHANGEDRO: auto_event = 51;
pub const EVENT_FILEAPPENDPRE: auto_event = 50;
pub const EVENT_FILEAPPENDPOST: auto_event = 49;
pub const EVENT_FILEAPPENDCMD: auto_event = 48;
pub const EVENT_EXITPRE: auto_event = 47;
pub const EVENT_ENCODINGCHANGED: auto_event = 46;
pub const EVENT_DIRCHANGEDPRE: auto_event = 45;
pub const EVENT_DIRCHANGED: auto_event = 44;
pub const EVENT_DIFFUPDATED: auto_event = 43;
pub const EVENT_DIAGNOSTICCHANGED: auto_event = 42;
pub const EVENT_CURSORMOVEDI: auto_event = 41;
pub const EVENT_CURSORMOVEDC: auto_event = 40;
pub const EVENT_CURSORMOVED: auto_event = 39;
pub const EVENT_CURSORHOLDI: auto_event = 38;
pub const EVENT_CURSORHOLD: auto_event = 37;
pub const EVENT_COMPLETEDONEPRE: auto_event = 36;
pub const EVENT_COMPLETEDONE: auto_event = 35;
pub const EVENT_COMPLETECHANGED: auto_event = 34;
pub const EVENT_COLORSCHEMEPRE: auto_event = 33;
pub const EVENT_COLORSCHEME: auto_event = 32;
pub const EVENT_CMDWINLEAVE: auto_event = 31;
pub const EVENT_CMDWINENTER: auto_event = 30;
pub const EVENT_CMDUNDEFINED: auto_event = 29;
pub const EVENT_CMDLINELEAVEPRE: auto_event = 28;
pub const EVENT_CMDLINELEAVE: auto_event = 27;
pub const EVENT_CMDLINEENTER: auto_event = 26;
pub const EVENT_CMDLINECHANGED: auto_event = 25;
pub const EVENT_CHANOPEN: auto_event = 24;
pub const EVENT_CHANINFO: auto_event = 23;
pub const EVENT_BUFWRITEPRE: auto_event = 22;
pub const EVENT_BUFWRITEPOST: auto_event = 21;
pub const EVENT_BUFWRITECMD: auto_event = 20;
pub const EVENT_BUFWRITE: auto_event = 19;
pub const EVENT_BUFWIPEOUT: auto_event = 18;
pub const EVENT_BUFWINLEAVE: auto_event = 17;
pub const EVENT_BUFWINENTER: auto_event = 16;
pub const EVENT_BUFUNLOAD: auto_event = 15;
pub const EVENT_BUFREADPRE: auto_event = 14;
pub const EVENT_BUFREADPOST: auto_event = 13;
pub const EVENT_BUFREADCMD: auto_event = 12;
pub const EVENT_BUFREAD: auto_event = 11;
pub const EVENT_BUFNEWFILE: auto_event = 10;
pub const EVENT_BUFNEW: auto_event = 9;
pub const EVENT_BUFMODIFIEDSET: auto_event = 8;
pub const EVENT_BUFLEAVE: auto_event = 7;
pub const EVENT_BUFHIDDEN: auto_event = 6;
pub const EVENT_BUFFILEPRE: auto_event = 5;
pub const EVENT_BUFFILEPOST: auto_event = 4;
pub const EVENT_BUFENTER: auto_event = 3;
pub const EVENT_BUFDELETE: auto_event = 2;
pub const EVENT_BUFCREATE: auto_event = 1;
pub const EVENT_BUFADD: auto_event = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoCmdEvent {
    pub event: event_T,
    pub fname: *mut ::core::ffi::c_char,
    pub fname_io: *mut ::core::ffi::c_char,
    pub buf: Buffer,
    pub group: ::core::ffi::c_int,
    pub eap: *mut exarg_T,
    pub data: *mut Object,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_30 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub type C2Rust_Unnamed_31 = ::core::ffi::c_int;
pub const AUGROUP_DELETED: C2Rust_Unnamed_31 = -4;
pub const AUGROUP_ALL: C2Rust_Unnamed_31 = -3;
pub const AUGROUP_ERROR: C2Rust_Unnamed_31 = -2;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_31 = -1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const BUFLOCAL_PAT_LEN: C2Rust_Unnamed_32 = 25;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct event_name {
    pub len: size_t,
    pub name: *mut ::core::ffi::c_char,
    pub event: ::core::ffi::c_int,
}
pub const ETYPE_AUCMD: etype_T = 3;
pub const DOCMD_REPEAT: C2Rust_Unnamed_34 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_34 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_34 = 2;
pub const kRetNilBool: LuaRetMode = 1;
pub const OPT_NOWIN: C2Rust_Unnamed_36 = 16;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_36 = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_String: GlobalCell<String_0> = GlobalCell::new(STRING_INIT);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_put_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = map_put_ref_String_int(
        map,
        key,
        ::core::ptr::null_mut::<*mut String_0>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
#[inline]
unsafe extern "C" fn map_put_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
    mut value: String_0,
) {
    let mut val: *mut String_0 = map_put_ref_int_String(
        map,
        key,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
) -> String_0 {
    let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_String.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static e_autocommand_nesting_too_deep: GlobalCell<[::core::ffi::c_char; 35]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 35], [::core::ffi::c_char; 35]>(
            *b"E218: Autocommand nesting too deep\0",
        )
    });
static active_apc_list: GlobalCell<*mut AutoPatCmd> =
    GlobalCell::new(::core::ptr::null_mut::<AutoPatCmd>());
static next_augroup_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
static deleted_augroup: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
static current_augroup: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(AUGROUP_DEFAULT as ::core::ffi::c_int);
static au_need_clean: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static autocmd_blocked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static autocmd_nested: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static autocmd_include_groups: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static termresponse_changed: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static map_augroup_name_to_id: GlobalCell<Map_String_int> = GlobalCell::new(Map_String_int {
    set: Set_String {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<String_0>(),
    },
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
});
static map_augroup_id_to_name: GlobalCell<Map_int_String> = GlobalCell::new(Map_int_String {
    set: Set_int {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<::core::ffi::c_int>(),
    },
    values: ::core::ptr::null_mut::<String_0>(),
});
pub unsafe extern "C" fn autocmd_init() {
    deferred_events.set(multiqueue_new_child((*main_loop.ptr()).events));
}
unsafe extern "C" fn augroup_map_del(
    mut id: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
) {
    if !name.is_null() {
        let mut key: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        map_del_String_int(
            map_augroup_name_to_id.ptr(),
            cstr_as_string(name),
            &raw mut key,
        );
        api_free_string(key);
    }
    if id > 0 as ::core::ffi::c_int {
        let mut mapped: String_0 = map_del_int_String(
            map_augroup_id_to_name.ptr(),
            id,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        api_free_string(mapped);
    }
}
#[inline(always)]
unsafe extern "C" fn get_deleted_augroup() -> *const ::core::ffi::c_char {
    if (*deleted_augroup.ptr()).is_null() {
        deleted_augroup.set(gettext(
            b"--Deleted--\0".as_ptr() as *const ::core::ffi::c_char
        ));
    }
    return deleted_augroup.get();
}
unsafe extern "C" fn au_show_for_all_events(
    mut group: ::core::ffi::c_int,
    mut pat: *const ::core::ffi::c_char,
) {
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        au_show_for_event(group, event, pat);
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
}
unsafe extern "C" fn au_show_for_event(
    mut group: ::core::ffi::c_int,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
) {
    let acs: *mut AutoCmdVec =
        (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
    if (*acs).size == 0 as size_t {
        return;
    }
    let mut patlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if *pat as ::core::ffi::c_int != NUL {
        patlen = aucmd_span_pattern(pat, &raw mut pat) as ::core::ffi::c_int;
        if patlen == 0 as ::core::ffi::c_int {
            return;
        }
    }
    let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
    let mut last_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
    let mut last_group_name: *const ::core::ffi::c_char =
        ::core::ptr::null::<::core::ffi::c_char>();
    loop {
        let mut last_ap: *mut AutoPat = ::core::ptr::null_mut::<AutoPat>();
        let mut endpat: *const ::core::ffi::c_char = pat.offset(patlen as isize);
        if aupat_is_buflocal(pat, patlen) {
            aupat_normalize_buflocal_pat(
                &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                pat,
                patlen,
                aupat_get_buflocal_nr(pat, patlen),
            );
            pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
            patlen =
                strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        }
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if !(*ac).pat.is_null() {
                if !(group != AUGROUP_ALL as ::core::ffi::c_int && (*(*ac).pat).group != group
                    || patlen != 0
                        && ((*(*ac).pat).patlen != patlen
                            || strncmp(pat, (*(*ac).pat).pat, patlen as size_t)
                                != 0 as ::core::ffi::c_int))
                {
                    if (*(*ac).pat).group != last_group {
                        last_group = (*(*ac).pat).group;
                        last_group_name = augroup_name((*(*ac).pat).group);
                        if got_int.get() {
                            return;
                        }
                        msg_putchar('\n' as ::core::ffi::c_int);
                        if got_int.get() {
                            return;
                        }
                        if (*(*ac).pat).group != AUGROUP_DEFAULT as ::core::ffi::c_int {
                            if last_group_name.is_null() {
                                msg_puts_hl(get_deleted_augroup(), HLF_E, false_0 != 0);
                            } else {
                                msg_puts_hl(last_group_name, HLF_T, false_0 != 0);
                            }
                            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                        }
                        msg_puts_hl(event_nr2name(event), HLF_T, false_0 != 0);
                    }
                    if last_ap != (*ac).pat {
                        last_ap = (*ac).pat;
                        msg_putchar('\n' as ::core::ffi::c_int);
                        if got_int.get() {
                            return;
                        }
                        msg_advance(4 as ::core::ffi::c_int);
                        msg_outtrans((*(*ac).pat).pat, 0 as ::core::ffi::c_int, false_0 != 0);
                    }
                    if got_int.get() {
                        return;
                    }
                    if msg_col.get() >= 14 as ::core::ffi::c_int {
                        msg_putchar('\n' as ::core::ffi::c_int);
                    }
                    msg_advance(14 as ::core::ffi::c_int);
                    if got_int.get() {
                        return;
                    }
                    let mut handler_str: *mut ::core::ffi::c_char = aucmd_handler_to_string(ac);
                    if !(*ac).desc.is_null() {
                        let mut msglen: size_t = 100 as size_t;
                        let mut msg: *mut ::core::ffi::c_char =
                            xmallocz(msglen) as *mut ::core::ffi::c_char;
                        if !(*ac).handler_cmd.is_null() {
                            snprintf(
                                msg,
                                msglen,
                                b"%s [%s]\0".as_ptr() as *const ::core::ffi::c_char,
                                handler_str,
                                (*ac).desc,
                            );
                        } else {
                            msg_puts_hl(handler_str, HLF_8, false_0 != 0);
                            snprintf(
                                msg,
                                msglen,
                                b" [%s]\0".as_ptr() as *const ::core::ffi::c_char,
                                (*ac).desc,
                            );
                        }
                        msg_outtrans(msg, 0 as ::core::ffi::c_int, false_0 != 0);
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut msg as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        let _ = *ptr_;
                    } else if !(*ac).handler_cmd.is_null() {
                        msg_outtrans(handler_str, 0 as ::core::ffi::c_int, false_0 != 0);
                    } else {
                        msg_puts_hl(handler_str, HLF_8, false_0 != 0);
                    }
                    let mut ptr__0: *mut *mut ::core::ffi::c_void =
                        &raw mut handler_str as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL_0;
                    let _ = *ptr__0;
                    if p_verbose.get() > 0 as OptInt {
                        last_set_msg((*ac).script_ctx);
                    }
                    if got_int.get() {
                        return;
                    }
                }
            }
            i = i.wrapping_add(1);
        }
        patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
        if patlen == 0 {
            break;
        }
    }
}
unsafe extern "C" fn aucmd_del(mut ac: *mut AutoCmd) {
    if !(*ac).pat.is_null() && {
        (*(*ac).pat).refcount = (*(*ac).pat).refcount.wrapping_sub(1);
        (*(*ac).pat).refcount == 0 as size_t
    } {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*(*ac).pat).pat as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        vim_regfree((*(*ac).pat).reg_prog);
        xfree((*ac).pat as *mut ::core::ffi::c_void);
    }
    (*ac).pat = ::core::ptr::null_mut::<AutoPat>();
    if !(*ac).handler_cmd.is_null() {
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*ac).handler_cmd as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
    } else {
        callback_free(&raw mut (*ac).handler_fn);
    }
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*ac).desc as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    let _ = *ptr__1;
    au_need_clean.set(true_0 != 0);
}
pub unsafe extern "C" fn aucmd_del_for_event_and_group(
    mut event: event_T,
    mut group: ::core::ffi::c_int,
) {
    let acs: *mut AutoCmdVec =
        (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
    let mut i: size_t = 0 as size_t;
    while i < (*acs).size {
        let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
        if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
            aucmd_del(ac);
        }
        i = i.wrapping_add(1);
    }
    au_cleanup();
}
unsafe extern "C" fn au_cleanup() {
    if autocmd_busy.get() as ::core::ffi::c_int != 0 || !au_need_clean.get() {
        return;
    }
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        let mut nsize: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if nsize != i {
                *(*acs).items.offset(nsize as isize) = *ac;
            }
            if !(*ac).pat.is_null() {
                nsize = nsize.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        if nsize == 0 as size_t {
            xfree((*acs).items as *mut ::core::ffi::c_void);
            (*acs).capacity = 0 as size_t;
            (*acs).size = (*acs).capacity;
            (*acs).items = ::core::ptr::null_mut::<AutoCmd>();
        } else {
            (*acs).size = nsize;
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
    au_need_clean.set(false_0 != 0);
}
pub unsafe extern "C" fn au_get_autocmds_for_event(mut event: event_T) -> *mut AutoCmdVec {
    return (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
}
pub unsafe extern "C" fn aubuflocal_remove(mut buf: *mut buf_T) {
    let mut apc: *mut AutoPatCmd = active_apc_list.get();
    while !apc.is_null() {
        if (*buf).handle == (*apc).arg_bufnr {
            (*apc).arg_bufnr = 0 as ::core::ffi::c_int;
        }
        apc = (*apc).next;
    }
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if !((*ac).pat.is_null() || (*(*ac).pat).buflocal_nr != (*buf).handle) {
                aucmd_del(ac);
                if p_verbose.get() >= 6 as OptInt {
                    verbose_enter();
                    smsg(
                        0 as ::core::ffi::c_int,
                        gettext(b"auto-removing autocommand: %s <buffer=%d>\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        event_nr2name(event),
                        (*buf).handle,
                    );
                    verbose_leave();
                }
            }
            i = i.wrapping_add(1);
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
    au_cleanup();
}
pub unsafe extern "C" fn augroup_add(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) != 0 as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"STRICMP(name, \"end\") != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                400 as ::core::ffi::c_uint,
                b"int augroup_add(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut existing_id: ::core::ffi::c_int = augroup_find(name);
    if existing_id > 0 as ::core::ffi::c_int {
        '_c2rust_label_0: {
            if existing_id != AUGROUP_DELETED as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"existing_id != AUGROUP_DELETED\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    404 as ::core::ffi::c_uint,
                    b"int augroup_add(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        return existing_id;
    }
    if existing_id == AUGROUP_DELETED as ::core::ffi::c_int {
        augroup_map_del(existing_id, name);
    }
    let c2rust_fresh0 = next_augroup_id.get();
    next_augroup_id.set(next_augroup_id.get() + 1);
    let mut next_id: ::core::ffi::c_int = c2rust_fresh0;
    let mut name_key: String_0 = cstr_to_string(name);
    let mut name_val: String_0 = cstr_to_string(name);
    map_put_String_int(map_augroup_name_to_id.ptr(), name_key, next_id);
    map_put_int_String(map_augroup_id_to_name.ptr(), next_id, name_val);
    return next_id;
}
pub unsafe extern "C" fn augroup_del(
    mut name: *mut ::core::ffi::c_char,
    mut stupid_legacy_mode: bool,
) {
    let mut group: ::core::ffi::c_int = augroup_find(name);
    if group == AUGROUP_ERROR as ::core::ffi::c_int {
        semsg(
            gettext(b"E367: No such group: \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
            name,
        );
        return;
    } else if group == current_augroup.get() {
        emsg(gettext(
            b"E936: Cannot delete the current group\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    if stupid_legacy_mode {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs: *mut AutoCmdVec =
                (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
                if !ap.is_null() && (*ap).group == group {
                    give_warning(
                        gettext(b"W19: Deleting augroup that is still in use\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        true_0 != 0,
                        true_0 != 0,
                    );
                    map_put_String_int(
                        map_augroup_name_to_id.ptr(),
                        cstr_as_string(name),
                        AUGROUP_DELETED as ::core::ffi::c_int,
                    );
                    augroup_map_del((*ap).group, ::core::ptr::null::<::core::ffi::c_char>());
                    return;
                }
                i = i.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    } else {
        let mut event_0: event_T = EVENT_BUFADD;
        while (event_0 as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs_0: *mut AutoCmdVec =
                (autocmds.ptr() as *mut AutoCmdVec).offset(event_0 as ::core::ffi::c_int as isize);
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*acs_0).size {
                let ac: *mut AutoCmd = (*acs_0).items.offset(i_0 as isize);
                if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
                    aucmd_del(ac);
                }
                i_0 = i_0.wrapping_add(1);
            }
            event_0 = (event_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    }
    augroup_map_del(group, name);
    au_cleanup();
}
pub unsafe extern "C" fn augroup_find(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut existing_id: ::core::ffi::c_int =
        map_get_String_int(map_augroup_name_to_id.ptr(), cstr_as_string(name));
    if existing_id == AUGROUP_DELETED as ::core::ffi::c_int {
        return existing_id;
    }
    if existing_id > 0 as ::core::ffi::c_int {
        return existing_id;
    }
    return AUGROUP_ERROR as ::core::ffi::c_int;
}
pub unsafe extern "C" fn augroup_name(mut group: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if group != 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"group != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                496 as ::core::ffi::c_uint,
                b"char *augroup_name(int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if group == AUGROUP_DELETED as ::core::ffi::c_int {
        return get_deleted_augroup() as *mut ::core::ffi::c_char;
    }
    if group == AUGROUP_ALL as ::core::ffi::c_int {
        group = current_augroup.get();
    }
    if group == next_augroup_id.get() {
        return b"END\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if group > next_augroup_id.get() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut key: String_0 = map_get_int_String(map_augroup_id_to_name.ptr(), group);
    if !key.data.is_null() {
        return key.data;
    }
    return get_deleted_augroup() as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn augroup_exists(mut name: *const ::core::ffi::c_char) -> bool {
    return augroup_find(name) > 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn do_augroup(mut arg: *mut ::core::ffi::c_char, mut del_group: bool) {
    if del_group {
        if *arg as ::core::ffi::c_int == NUL {
            emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        } else {
            augroup_del(arg, true_0 != 0);
        }
    } else if strcasecmp(
        arg,
        b"end\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        current_augroup.set(AUGROUP_DEFAULT as ::core::ffi::c_int);
    } else if *arg != 0 {
        current_augroup.set(augroup_add(arg));
    } else {
        msg_start();
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        let mut name: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        };
        let mut value: ::core::ffi::c_int = 0;
        let mut __i: uint32_t = 0;
        __i = 0 as uint32_t;
        while __i < (*map_augroup_name_to_id.ptr()).set.h.n_keys {
            name = *(*map_augroup_name_to_id.ptr())
                .set
                .keys
                .offset(__i as isize);
            value = *(*map_augroup_name_to_id.ptr()).values.offset(__i as isize);
            if value > 0 as ::core::ffi::c_int {
                msg_puts(name.data);
            } else {
                msg_puts(augroup_name(value));
            }
            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
            __i = __i.wrapping_add(1);
        }
        msg_clr_eos();
        msg_end();
    };
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn is_aucmd_win(mut win: *mut win_T) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int {
        if (*(*aucmd_win_vec.ptr()).items.offset(i as isize)).auc_win_used as ::core::ffi::c_int
            != 0
            && (*(*aucmd_win_vec.ptr()).items.offset(i as isize)).auc_win == win
        {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn event_name2nr(
    mut start: *const ::core::ffi::c_char,
    mut end: *mut *mut ::core::ffi::c_char,
) -> event_T {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    p = start;
    while *p as ::core::ffi::c_int != 0
        && !ascii_iswhite(*p as ::core::ffi::c_int)
        && *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
        && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    let mut hash_idx: ::core::ffi::c_int =
        event_name2nr_hash(start, p.offset_from(start) as size_t);
    if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    *end = p as *mut ::core::ffi::c_char;
    if hash_idx < 0 as ::core::ffi::c_int {
        return NUM_EVENTS;
    }
    return abs((*event_names.ptr())[(*event_hash.ptr())[hash_idx as usize] as usize].event)
        as event_T;
}
pub unsafe extern "C" fn event_name2nr_str(mut str: String_0) -> event_T {
    let mut hash_idx: ::core::ffi::c_int = event_name2nr_hash(str.data, str.size);
    if hash_idx < 0 as ::core::ffi::c_int {
        return NUM_EVENTS;
    }
    return abs((*event_names.ptr())[(*event_hash.ptr())[hash_idx as usize] as usize].event)
        as event_T;
}
pub unsafe extern "C" fn event_nr2name(mut event: event_T) -> *const ::core::ffi::c_char {
    return if event as ::core::ffi::c_uint >= 0 as ::core::ffi::c_uint
        && (event as ::core::ffi::c_uint) < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*event_names.ptr())[event as usize].name as *const ::core::ffi::c_char
    } else {
        b"Unknown\0".as_ptr() as *const ::core::ffi::c_char
    };
}
pub unsafe extern "C" fn event_ignored(
    mut event: event_T,
    mut ei: *mut ::core::ffi::c_char,
) -> bool {
    let mut ignored: bool = false_0 != 0;
    while *ei as ::core::ffi::c_int != NUL {
        let mut unignore: bool = *ei as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
        ei = ei.offset(unignore as ::core::ffi::c_int as isize);
        if strncasecmp(
            ei,
            b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            3 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ',' as ::core::ffi::c_int)
        {
            ignored = ei == p_ei.get()
                || (*event_names.ptr())[event as usize].event <= 0 as ::core::ffi::c_int;
            ei = ei.offset(
                (3 as ::core::ffi::c_int
                    + (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int) as ::core::ffi::c_int)
                    as isize,
            );
        } else if event_name2nr(ei, &raw mut ei) as ::core::ffi::c_uint
            == event as ::core::ffi::c_uint
        {
            if unignore {
                return false_0 != 0;
            }
            ignored = true_0 != 0;
        }
    }
    return ignored;
}
pub unsafe extern "C" fn check_ei(mut ei: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut win: bool = ei != p_ei.get();
    while *ei != 0 {
        if strncasecmp(
            ei,
            b"all\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            3 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || *ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ',' as ::core::ffi::c_int)
        {
            ei = ei.offset(
                (3 as ::core::ffi::c_int
                    + (*ei.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int) as ::core::ffi::c_int)
                    as isize,
            );
        } else {
            ei = ei.offset(
                (*ei as ::core::ffi::c_int == '-' as ::core::ffi::c_int) as ::core::ffi::c_int
                    as isize,
            );
            let mut event: event_T = event_name2nr(ei, &raw mut ei);
            if event as ::core::ffi::c_uint
                == NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                || win as ::core::ffi::c_int != 0
                    && (*event_names.ptr())[event as usize].event > 0 as ::core::ffi::c_int
            {
                return FAIL;
            }
        }
    }
    return OK;
}
pub unsafe extern "C" fn au_event_disable(
    mut what: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut p_ei_len: size_t = strlen(p_ei.get());
    let mut save_ei: *mut ::core::ffi::c_char =
        xmemdupz(p_ei.get() as *const ::core::ffi::c_void, p_ei_len) as *mut ::core::ffi::c_char;
    let mut new_ei: *mut ::core::ffi::c_char =
        xstrnsave(p_ei.get(), p_ei_len.wrapping_add(strlen(what)));
    if *what as ::core::ffi::c_int == ',' as ::core::ffi::c_int
        && *p_ei.get() as ::core::ffi::c_int == NUL
    {
        strcpy(new_ei, what.offset(1 as ::core::ffi::c_int as isize));
    } else {
        strcpy(new_ei.offset(p_ei_len as isize), what);
    }
    set_option_direct(
        kOptEventignore,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: cstr_as_string(new_ei),
            },
        },
        0 as ::core::ffi::c_int,
        SID_NONE,
    );
    xfree(new_ei as *mut ::core::ffi::c_void);
    return save_ei;
}
pub unsafe extern "C" fn au_event_restore(mut old_ei: *mut ::core::ffi::c_char) {
    if !old_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(old_ei),
                },
            },
            0 as ::core::ffi::c_int,
            SID_NONE,
        );
        xfree(old_ei as *mut ::core::ffi::c_void);
    }
}
pub unsafe extern "C" fn do_autocmd(
    mut eap: *mut exarg_T,
    mut arg_in: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
) {
    let mut arg: *mut ::core::ffi::c_char = arg_in;
    let mut envpat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut need_free: bool = false_0 != 0;
    let mut nested: bool = false_0 != 0;
    let mut once: bool = false_0 != 0;
    let mut group: ::core::ffi::c_int = 0;
    if *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
        (*eap).nextcmd = arg.offset(1 as ::core::ffi::c_int as isize);
        arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        group = AUGROUP_ALL as ::core::ffi::c_int;
    } else {
        group = arg_augroup_get(&raw mut arg);
    }
    let mut pat: *mut ::core::ffi::c_char =
        arg_event_skip(arg, group != AUGROUP_ALL as ::core::ffi::c_int);
    if pat.is_null() {
        return;
    }
    pat = skipwhite(pat);
    if *pat as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
        (*eap).nextcmd = pat.offset(1 as ::core::ffi::c_int as isize);
        pat = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        cmd = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        cmd = pat;
        while *cmd as ::core::ffi::c_int != 0
            && (!ascii_iswhite(*cmd as ::core::ffi::c_int)
                || *cmd.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
        {
            cmd = cmd.offset(1);
        }
        if *cmd != 0 {
            let c2rust_fresh1 = cmd;
            cmd = cmd.offset(1);
            *c2rust_fresh1 = NUL as ::core::ffi::c_char;
        }
        if !vim_strchr(pat, '$' as ::core::ffi::c_int).is_null()
            || !vim_strchr(pat, '~' as ::core::ffi::c_int).is_null()
        {
            envpat = expand_env_save(pat);
            if !envpat.is_null() {
                pat = envpat;
            }
        }
        cmd = skipwhite(cmd);
        let mut invalid_flags: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < 2 as size_t {
            if *cmd as ::core::ffi::c_int != NUL {
                invalid_flags = invalid_flags as ::core::ffi::c_int
                    | arg_autocmd_flag_get(
                        &raw mut once,
                        &raw mut cmd,
                        b"++once\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        6 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                    != 0;
                invalid_flags = invalid_flags as ::core::ffi::c_int
                    | arg_autocmd_flag_get(
                        &raw mut nested,
                        &raw mut cmd,
                        b"++nested\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        8 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                    != 0;
                invalid_flags = invalid_flags as ::core::ffi::c_int
                    | arg_autocmd_flag_get(
                        &raw mut nested,
                        &raw mut cmd,
                        b"nested\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                        6 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                    != 0;
            }
            i = i.wrapping_add(1);
        }
        if invalid_flags {
            return;
        }
        if *cmd as ::core::ffi::c_int != NUL {
            cmd = expand_sfile(cmd);
            if cmd.is_null() {
                return;
            }
            need_free = true_0 != 0;
        }
    }
    let is_showing: bool = forceit == 0 && *cmd as ::core::ffi::c_int == NUL;
    if is_showing {
        msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
        msg_puts_title(gettext(
            b"\n--- Autocommands ---\0".as_ptr() as *const ::core::ffi::c_char
        ));
        if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == NUL
        {
            au_show_for_all_events(group, pat);
        } else {
            let mut event: event_T = event_name2nr(arg, &raw mut arg);
            '_c2rust_label: {
                if (event as ::core::ffi::c_uint)
                    < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"event < NUM_EVENTS\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        860 as ::core::ffi::c_uint,
                        b"void do_autocmd(exarg_T *, char *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            au_show_for_event(group, event, pat);
        }
    } else if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int
        || *arg as ::core::ffi::c_int == NUL
        || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
    {
        if *cmd as ::core::ffi::c_int != NUL {
            emsg(gettext(
                &raw const e_cannot_define_autocommands_for_all_events
                    as *const ::core::ffi::c_char,
            ));
        } else {
            do_all_autocmd_events(
                pat,
                once,
                nested as ::core::ffi::c_int,
                cmd,
                forceit != 0,
                group,
            );
        }
    } else {
        while *arg as ::core::ffi::c_int != 0
            && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
            && !ascii_iswhite(*arg as ::core::ffi::c_int)
        {
            let mut event_0: event_T = event_name2nr(arg, &raw mut arg);
            '_c2rust_label_0: {
                if (event_0 as ::core::ffi::c_uint)
                    < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"event < NUM_EVENTS\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        873 as ::core::ffi::c_uint,
                        b"void do_autocmd(exarg_T *, char *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if do_autocmd_event(
                event_0,
                pat,
                once,
                nested as ::core::ffi::c_int,
                cmd,
                forceit != 0,
                group,
            ) == FAIL
            {
                break;
            }
        }
    }
    if need_free {
        xfree(cmd as *mut ::core::ffi::c_void);
    }
    xfree(envpat as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn do_all_autocmd_events(
    mut pat: *const ::core::ffi::c_char,
    mut once: bool,
    mut nested: ::core::ffi::c_int,
    mut cmd: *mut ::core::ffi::c_char,
    mut del: bool,
    mut group: ::core::ffi::c_int,
) {
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        if do_autocmd_event(event, pat, once, nested, cmd, del, group) == FAIL {
            return;
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
}
pub unsafe extern "C" fn do_autocmd_event(
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut once: bool,
    mut nested: ::core::ffi::c_int,
    mut cmd: *const ::core::ffi::c_char,
    mut del: bool,
    mut group: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if *pat as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
            || del as ::core::ffi::c_int != 0
        {
        } else {
            __assert_fail(
                b"*pat != NUL || del\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                908 as ::core::ffi::c_uint,
                b"int do_autocmd_event(event_T, const char *, _Bool, int, const char *, _Bool, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
    let mut is_adding_cmd: bool = *cmd as ::core::ffi::c_int != NUL;
    let findgroup: ::core::ffi::c_int = if group == AUGROUP_ALL as ::core::ffi::c_int {
        current_augroup.get()
    } else {
        group
    };
    if *pat as ::core::ffi::c_int == NUL && del as ::core::ffi::c_int != 0 {
        aucmd_del_for_event_and_group(event, findgroup);
        return OK;
    }
    let mut patlen: ::core::ffi::c_int =
        aucmd_span_pattern(pat, &raw mut pat) as ::core::ffi::c_int;
    while patlen != 0 {
        let mut endpat: *const ::core::ffi::c_char = pat.offset(patlen as isize);
        let mut is_buflocal: bool = aupat_is_buflocal(pat, patlen);
        if is_buflocal {
            let buflocal_nr: ::core::ffi::c_int = aupat_get_buflocal_nr(pat, patlen);
            aupat_normalize_buflocal_pat(
                &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                pat,
                patlen,
                buflocal_nr,
            );
            pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
            patlen =
                strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        }
        if del {
            '_c2rust_label_0: {
                if *pat as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"*pat != NUL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/autocmd.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        939 as ::core::ffi::c_uint,
                        b"int do_autocmd_event(event_T, const char *, _Bool, int, const char *, _Bool, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let acs: *mut AutoCmdVec =
                (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                let ap: *mut AutoPat = (*ac).pat;
                if !ap.is_null()
                    && (*ap).group == findgroup
                    && (*ap).patlen == patlen
                    && strncmp(pat, (*ap).pat, patlen as size_t) == 0 as ::core::ffi::c_int
                {
                    aucmd_del(ac);
                }
                i = i.wrapping_add(1);
            }
        }
        if is_adding_cmd {
            let mut handler_fn: Callback = Callback {
                data: C2Rust_Unnamed_5 {
                    funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                },
                type_0: kCallbackNone,
            };
            autocmd_register(
                0 as int64_t,
                event,
                pat,
                patlen,
                group,
                once,
                nested != 0,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                cmd,
                &raw mut handler_fn,
            );
        }
        patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
    }
    au_cleanup();
    return OK;
}
pub unsafe extern "C" fn autocmd_register(
    mut id: int64_t,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    mut group: ::core::ffi::c_int,
    mut once: bool,
    mut nested: bool,
    mut desc: *mut ::core::ffi::c_char,
    mut handler_cmd: *const ::core::ffi::c_char,
    mut handler_fn: *mut Callback,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if group != 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"group != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                984 as ::core::ffi::c_uint,
                b"int autocmd_register(int64_t, event_T, const char *, int, int, _Bool, _Bool, char *, const char *, Callback *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if patlen > strlen(pat) as ::core::ffi::c_int {
        return FAIL;
    }
    let findgroup: ::core::ffi::c_int = if group == AUGROUP_ALL as ::core::ffi::c_int {
        current_augroup.get()
    } else {
        group
    };
    let is_buflocal: bool = aupat_is_buflocal(pat, patlen);
    let mut buflocal_nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
    if is_buflocal {
        buflocal_nr = aupat_get_buflocal_nr(pat, patlen);
        aupat_normalize_buflocal_pat(
            &raw mut buflocal_pat as *mut ::core::ffi::c_char,
            pat,
            patlen,
            buflocal_nr,
        );
        pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
        patlen = strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
    }
    let mut ap: *mut AutoPat = ::core::ptr::null_mut::<AutoPat>();
    let acs: *mut AutoCmdVec =
        (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
    let mut i: ptrdiff_t = (*acs).size as ptrdiff_t - 1 as ptrdiff_t;
    while i >= 0 as ptrdiff_t {
        ap = (*(*acs).items.offset(i as isize)).pat;
        if ap.is_null() {
            i -= 1;
        } else {
            if (*ap).group != findgroup
                || (*ap).patlen != patlen
                || strncmp(pat, (*ap).pat, patlen as size_t) != 0 as ::core::ffi::c_int
            {
                ap = ::core::ptr::null_mut::<AutoPat>();
            }
            break;
        }
    }
    if ap.is_null() {
        if is_buflocal as ::core::ffi::c_int != 0
            && (buflocal_nr == 0 as ::core::ffi::c_int || buflist_findnr(buflocal_nr).is_null())
        {
            semsg(
                gettext(b"E680: <buffer=%d>: invalid buffer number \0".as_ptr()
                    as *const ::core::ffi::c_char),
                buflocal_nr,
            );
            return FAIL;
        }
        ap = xmalloc(::core::mem::size_of::<AutoPat>()) as *mut AutoPat;
        if is_buflocal {
            (*ap).buflocal_nr = buflocal_nr;
            (*ap).reg_prog = ::core::ptr::null_mut::<regprog_T>();
        } else {
            (*ap).buflocal_nr = 0 as ::core::ffi::c_int;
            let mut reg_pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
                pat,
                pat.offset(patlen as isize),
                &raw mut (*ap).allow_dirs,
                true_0,
            );
            if !reg_pat.is_null() {
                (*ap).reg_prog = vim_regcomp(reg_pat, RE_MAGIC);
            }
            xfree(reg_pat as *mut ::core::ffi::c_void);
            if reg_pat.is_null() || (*ap).reg_prog.is_null() {
                xfree(ap as *mut ::core::ffi::c_void);
                return FAIL;
            }
        }
        (*ap).refcount = 0 as size_t;
        (*ap).pat = xmemdupz(pat as *const ::core::ffi::c_void, patlen as size_t)
            as *mut ::core::ffi::c_char;
        (*ap).patlen = patlen;
        if event as ::core::ffi::c_uint
            == EVENT_MODECHANGED as ::core::ffi::c_int as ::core::ffi::c_uint
            && !has_event(EVENT_MODECHANGED)
        {
            get_mode(last_mode.ptr() as *mut ::core::ffi::c_char);
        }
        if event as ::core::ffi::c_uint
            == EVENT_CURSORMOVED as ::core::ffi::c_int as ::core::ffi::c_uint
            && !has_event(EVENT_CURSORMOVED)
            || event as ::core::ffi::c_uint
                == EVENT_CURSORMOVEDI as ::core::ffi::c_int as ::core::ffi::c_uint
                && !has_event(EVENT_CURSORMOVEDI)
        {
            last_cursormoved_win.set(curwin.get());
            last_cursormoved.set((*curwin.get()).w_cursor);
        }
        if (event as ::core::ffi::c_uint
            == EVENT_WINSCROLLED as ::core::ffi::c_int as ::core::ffi::c_uint
            || event as ::core::ffi::c_uint
                == EVENT_WINRESIZED as ::core::ffi::c_int as ::core::ffi::c_uint)
            && !(has_event(EVENT_WINSCROLLED) as ::core::ffi::c_int != 0
                || has_event(EVENT_WINRESIZED) as ::core::ffi::c_int != 0)
        {
            let mut save_curtab: *mut tabpage_T = curtab.get();
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                unuse_tabpage(curtab.get());
                use_tabpage(tp as *mut tabpage_T);
                snapshot_windows_scroll_size();
                tp = (*tp).tp_next as *mut tabpage_T;
            }
            unuse_tabpage(curtab.get());
            use_tabpage(save_curtab);
        }
        (*ap).group = if group == AUGROUP_ALL as ::core::ffi::c_int {
            current_augroup.get()
        } else {
            group
        };
    }
    (*ap).refcount = (*ap).refcount.wrapping_add(1);
    if (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size
        == (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity
    {
        (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity =
            if (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity != 0 {
                (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity
                    << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
        (*autocmds.ptr())[event as ::core::ffi::c_int as usize].items = xrealloc(
            (*autocmds.ptr())[event as ::core::ffi::c_int as usize].items
                as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<AutoCmd>()
                .wrapping_mul((*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity),
        ) as *mut AutoCmd;
    } else {
    };
    let c2rust_fresh2 = (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size;
    (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size = (*autocmds.ptr())
        [event as ::core::ffi::c_int as usize]
        .size
        .wrapping_add(1);
    let mut ac: *mut AutoCmd = (*autocmds.ptr())[event as ::core::ffi::c_int as usize]
        .items
        .offset(c2rust_fresh2 as isize);
    (*ac).pat = ap;
    (*ac).id = id;
    if !handler_cmd.is_null() {
        (*ac).handler_cmd = xstrdup(handler_cmd);
    } else {
        (*ac).handler_cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
        callback_copy(&raw mut (*ac).handler_fn, handler_fn);
    }
    (*ac).script_ctx = current_sctx.get();
    (*ac).script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum;
    nlua_set_sctx(&raw mut (*ac).script_ctx);
    (*ac).once = once;
    (*ac).nested = nested;
    (*ac).desc = if desc.is_null() {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    } else {
        xstrdup(desc)
    };
    return OK;
}
pub unsafe extern "C" fn aucmd_span_pattern(
    mut pat: *const ::core::ffi::c_char,
    mut start: *mut *const ::core::ffi::c_char,
) -> size_t {
    while *pat as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        pat = pat.offset(1);
    }
    let mut p: *const ::core::ffi::c_char = pat;
    let mut brace_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != 0
        && (*p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
            || brace_level != 0
            || p > pat
                && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
    {
        if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
            brace_level += 1;
        } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
            brace_level -= 1;
        }
        p = p.offset(1);
    }
    *start = pat;
    return p.offset_from(pat) as size_t;
}
pub unsafe extern "C" fn do_doautocmd(
    mut arg_start: *mut ::core::ffi::c_char,
    mut do_msg: bool,
    mut did_something: *mut bool,
) -> ::core::ffi::c_int {
    let mut arg: *mut ::core::ffi::c_char = arg_start;
    let mut nothing_done: ::core::ffi::c_int = true_0;
    if !did_something.is_null() {
        *did_something = false_0 != 0;
    }
    let mut group: ::core::ffi::c_int = arg_augroup_get(&raw mut arg);
    if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        emsg(gettext(
            b"E217: Can't execute autocommands for ALL events\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    let mut fname: *mut ::core::ffi::c_char =
        arg_event_skip(arg, group != AUGROUP_ALL as ::core::ffi::c_int);
    if fname.is_null() {
        return FAIL;
    }
    fname = skipwhite(fname);
    while *arg as ::core::ffi::c_int != 0
        && ends_excmd(*arg as ::core::ffi::c_int) == 0
        && !ascii_iswhite(*arg as ::core::ffi::c_int)
    {
        if apply_autocmds_group(
            event_name2nr(arg, &raw mut arg),
            fname,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            true_0 != 0,
            group,
            curbuf.get(),
            ::core::ptr::null_mut::<exarg_T>(),
            ::core::ptr::null_mut::<Object>(),
        ) {
            nothing_done = false_0;
        }
    }
    if nothing_done != 0 && do_msg as ::core::ffi::c_int != 0 && !aborting() {
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"No matching autocommands: %s\0".as_ptr() as *const ::core::ffi::c_char),
            arg_start,
        );
    }
    if !did_something.is_null() {
        *did_something = nothing_done == 0;
    }
    return if aborting() as ::core::ffi::c_int != 0 {
        FAIL
    } else {
        OK
    };
}
pub unsafe fn ex_doautoall(mut eap: *mut exarg_T) {
    let mut retval: ::core::ffi::c_int = OK;
    let mut aco: aco_save_T = aco_save_T::default();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut call_do_modelines: ::core::ffi::c_int =
        check_nomodeline(&raw mut arg) as ::core::ffi::c_int;
    let mut bufref: bufref_T = bufref_T::default();
    let mut did_aucmd: bool = false;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        if !((*buf).b_ml.ml_mfp.is_null() || buf == curbuf.get()) {
            aucmd_prepbuf(&raw mut aco, buf);
            set_bufref(&raw mut bufref, buf);
            retval = do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
            if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
                do_modelines(if is_aucmd_win(curwin.get()) as ::core::ffi::c_int != 0 {
                    OPT_NOWIN as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                });
            }
            aucmd_restbuf(&raw mut aco);
            if retval == FAIL || !bufref_valid(&raw mut bufref) {
                retval = FAIL;
                break;
            }
        }
        buf = (*buf).b_next;
    }
    if retval == OK {
        do_doautocmd(arg, false_0 != 0, &raw mut did_aucmd);
        if call_do_modelines != 0 && did_aucmd as ::core::ffi::c_int != 0 {
            do_modelines(0 as ::core::ffi::c_int);
        }
    }
}
pub unsafe extern "C" fn check_nomodeline(mut argp: *mut *mut ::core::ffi::c_char) -> bool {
    if strncmp(
        *argp,
        b"<nomodeline>\0".as_ptr() as *const ::core::ffi::c_char,
        12 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        *argp = skipwhite((*argp).offset(12 as ::core::ffi::c_int as isize));
        return false_0 != 0;
    }
    return true_0 != 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aucmd_prepbuf(mut aco: *mut aco_save_T, mut buf: *mut buf_T) {
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut need_append: bool = true_0 != 0;
    let same_buffer: bool = buf == curbuf.get();
    if same_buffer {
        win = curwin.get();
    } else {
        win = ::core::ptr::null_mut::<win_T>();
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                win = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
    }
    let mut auc_win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut auc_idx: ::core::ffi::c_int = (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int;
    if win.is_null() {
        auc_idx = 0 as ::core::ffi::c_int;
        while auc_idx < (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int {
            if !(*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize)).auc_win_used {
                break;
            }
            auc_idx += 1;
        }
        if auc_idx == (*aucmd_win_vec.ptr()).size as ::core::ffi::c_int {
            if (*aucmd_win_vec.ptr()).size == (*aucmd_win_vec.ptr()).capacity {
                (*aucmd_win_vec.ptr()).capacity = if (*aucmd_win_vec.ptr()).capacity != 0 {
                    (*aucmd_win_vec.ptr()).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*aucmd_win_vec.ptr()).items = xrealloc(
                    (*aucmd_win_vec.ptr()).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<aucmdwin_T>()
                        .wrapping_mul((*aucmd_win_vec.ptr()).capacity),
                ) as *mut aucmdwin_T;
            } else {
            };
            let c2rust_fresh12 = (*aucmd_win_vec.ptr()).size;
            (*aucmd_win_vec.ptr()).size = (*aucmd_win_vec.ptr()).size.wrapping_add(1);
            *(*aucmd_win_vec.ptr()).items.offset(c2rust_fresh12 as isize) = aucmdwin_T {
                auc_win: ::core::ptr::null_mut::<win_T>(),
                auc_win_used: false,
            };
        }
        if (*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize))
            .auc_win
            .is_null()
        {
            win_alloc_aucmd_win(auc_idx);
            need_append = false_0 != 0;
        }
        auc_win = (*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize)).auc_win;
        (*(*aucmd_win_vec.ptr()).items.offset(auc_idx as isize)).auc_win_used = true_0 != 0;
    }
    (*aco).save_curwin_handle = (*curwin.get()).handle;
    (*aco).save_prevwin_handle = (if (*prevwin.ptr()).is_null() {
        0 as ::core::ffi::c_int
    } else {
        (*prevwin.get()).handle as ::core::ffi::c_int
    }) as handle_T;
    if bt_prompt(curbuf.get()) {
        (*aco).save_prompt_insert = (*curbuf.get()).b_prompt_insert;
    }
    if !win.is_null() {
        (*aco).use_aucmd_win_idx = -1 as ::core::ffi::c_int;
        curwin.set(win);
    } else {
        (*aco).use_aucmd_win_idx = auc_idx;
        (*auc_win).w_buffer = buf;
        (*auc_win).w_s = &raw mut (*buf).b_s;
        (*buf).b_nwindows += 1;
        win_init_empty(auc_win);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*auc_win).w_localdir as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        (*aco).tp_localdir = (*curtab.get()).tp_localdir;
        (*curtab.get()).tp_localdir = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*aco).globaldir = globaldir.get();
        globaldir.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        block_autocmds();
        if need_append {
            win_append(lastwin.get(), auc_win, ::core::ptr::null_mut::<tabpage_T>());
            map_put_int_ptr_t(
                window_handles.ptr(),
                (*auc_win).handle as ::core::ffi::c_int,
                auc_win as ptr_t,
            );
            win_config_float(auc_win, (*auc_win).w_config);
        }
        let save_acd: ::core::ffi::c_int = p_acd.get();
        p_acd.set(false_0);
        (*RedrawingDisabled.ptr()) += 1;
        win_enter(auc_win, false_0 != 0);
        (*RedrawingDisabled.ptr()) -= 1;
        p_acd.set(save_acd);
        unblock_autocmds();
        curwin.set(auc_win);
    }
    curbuf.set(buf);
    (*aco).new_curwin_handle = (*curwin.get()).handle;
    set_bufref(&raw mut (*aco).new_curbuf, curbuf.get());
    (*aco).save_VIsual_active = VIsual_active.get();
    if !same_buffer {
        VIsual_active.set(false_0 != 0);
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aucmd_restbuf(mut aco: *mut aco_save_T) {
    if (*aco).use_aucmd_win_idx >= 0 as ::core::ffi::c_int {
        let mut awp: *mut win_T = (*(*aucmd_win_vec.ptr())
            .items
            .offset((*aco).use_aucmd_win_idx as isize))
        .auc_win;
        block_autocmds();
        '_win_found: {
            if curwin.get() != awp {
                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                loop {
                    if tp.is_null() {
                        break '_win_found;
                    }
                    let mut wp: *mut win_T = if tp == curtab.get() {
                        firstwin.get()
                    } else {
                        (*tp).tp_firstwin
                    };
                    while !wp.is_null() {
                        if wp == awp {
                            if tp != curtab.get() {
                                goto_tabpage_tp(tp as *mut tabpage_T, true_0 != 0, true_0 != 0);
                            }
                            win_goto(awp);
                            break '_win_found;
                        } else {
                            wp = (*wp).w_next;
                        }
                    }
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
            }
        }
        (*curbuf.get()).b_nwindows -= 1;
        win_remove(curwin.get(), ::core::ptr::null_mut::<tabpage_T>());
        map_del_int_ptr_t(
            window_handles.ptr(),
            (*curwin.get()).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if !(*curwin.get()).w_grid_alloc.chars.is_null() {
            ui_comp_remove_grid(&raw mut (*curwin.get()).w_grid_alloc);
            ui_call_win_hide((*curwin.get()).w_grid_alloc.handle as Integer);
            grid_free(&raw mut (*curwin.get()).w_grid_alloc);
        }
        (*(*aucmd_win_vec.ptr())
            .items
            .offset((*aco).use_aucmd_win_idx as isize))
        .auc_win_used = false_0 != 0;
        if valid_tabpage_win(curtab.get()) == 0 {
            close_tabpage(curtab.get());
        }
        unblock_autocmds();
        let save_curwin: *mut win_T = win_find_by_handle((*aco).save_curwin_handle);
        if !save_curwin.is_null() {
            curwin.set(save_curwin);
        } else {
            curwin.set(firstwin.get());
        }
        curbuf.set((*curwin.get()).w_buffer);
        entering_window(curwin.get());
        if bt_prompt(curbuf.get()) {
            (*curbuf.get()).b_prompt_insert = (*aco).save_prompt_insert;
        }
        prevwin.set(win_find_by_handle((*aco).save_prevwin_handle));
        vars_clear(&raw mut (*(*awp).w_vars).dv_hashtab);
        hash_init(&raw mut (*(*awp).w_vars).dv_hashtab);
        if !(*awp).w_localdir.is_null() {
            win_fix_current_dir();
        }
        xfree((*curtab.get()).tp_localdir as *mut ::core::ffi::c_void);
        (*curtab.get()).tp_localdir = (*aco).tp_localdir;
        xfree(globaldir.get() as *mut ::core::ffi::c_void);
        globaldir.set((*aco).globaldir);
        VIsual_active.set((*aco).save_VIsual_active);
        check_cursor(curwin.get());
        if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
            (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
            (*curwin.get()).w_topfill = 0 as ::core::ffi::c_int;
        }
    } else {
        let save_curwin_0: *mut win_T = win_find_by_handle((*aco).save_curwin_handle);
        if !save_curwin_0.is_null() {
            if (*curwin.get()).handle == (*aco).new_curwin_handle
                && curbuf.get() != (*aco).new_curbuf.br_buf
                && bufref_valid(&raw mut (*aco).new_curbuf) as ::core::ffi::c_int != 0
                && !(*(*aco).new_curbuf.br_buf).b_ml.ml_mfp.is_null()
            {
                if (*curwin.get()).w_s == &raw mut (*curbuf.get()).b_s {
                    (*curwin.get()).w_s = &raw mut (*(*aco).new_curbuf.br_buf).b_s;
                }
                (*curbuf.get()).b_nwindows -= 1;
                curbuf.set((*aco).new_curbuf.br_buf);
                (*curwin.get()).w_buffer = curbuf.get();
                (*curbuf.get()).b_nwindows += 1;
            }
            curwin.set(save_curwin_0);
            curbuf.set((*curwin.get()).w_buffer);
            prevwin.set(win_find_by_handle((*aco).save_prevwin_handle));
            VIsual_active.set((*aco).save_VIsual_active);
            check_cursor(curwin.get());
        }
    }
    VIsual_active.set((*aco).save_VIsual_active);
    check_cursor(curwin.get());
    if VIsual_active.get() {
        check_pos(curbuf.get(), VIsual.ptr());
    }
}
pub unsafe extern "C" fn aucmd_defer(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut group: ::core::ffi::c_int,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
    mut data: *mut Object,
) {
    let mut evdata: *mut AutoCmdEvent =
        xmalloc(::core::mem::size_of::<AutoCmdEvent>()) as *mut AutoCmdEvent;
    (*evdata).event = event;
    (*evdata).fname = if !fname.is_null() {
        xstrdup(fname)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*evdata).fname_io = if !fname_io.is_null() {
        xstrdup(fname_io)
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    (*evdata).group = group;
    (*evdata).buf = (*buf).handle as Buffer;
    (*evdata).eap = eap;
    if !data.is_null() {
        (*evdata).data = xmalloc(::core::mem::size_of::<Object>()) as *mut Object;
        *(*evdata).data = copy_object(*data, ::core::ptr::null_mut::<Arena>());
    } else {
        (*evdata).data = ::core::ptr::null_mut::<Object>();
    }
    multiqueue_put_event(
        deferred_events.get(),
        Event {
            handler: Some(
                deferred_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                evdata as *mut ::core::ffi::c_void,
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
}
unsafe extern "C" fn deferred_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut e: *mut AutoCmdEvent =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut AutoCmdEvent;
    let mut event: event_T = (*e).event;
    let mut fname: *mut ::core::ffi::c_char = (*e).fname;
    let mut fname_io: *mut ::core::ffi::c_char = (*e).fname_io;
    let mut group: ::core::ffi::c_int = (*e).group;
    let mut eap: *mut exarg_T = (*e).eap;
    let mut data: *mut Object = (*e).data;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut buf: *mut buf_T = find_buffer_by_handle((*e).buf, &raw mut err);
    if !buf.is_null() {
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
        if !data.is_null()
            && (*data).type_0 as ::core::ffi::c_uint
                == kObjectTypeDict as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut i: size_t = 0 as size_t;
            while i < (*data).data.dict.size {
                let mut item: KeyValuePair = *(*data).data.dict.items.offset(i as isize);
                let mut tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                object_to_vim(item.value, &raw mut tv, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    api_clear_error(&raw mut err);
                } else {
                    tv_dict_add_tv(v_event, item.key.data, item.key.size, &raw mut tv);
                    tv_clear(&raw mut tv);
                }
                i = i.wrapping_add(1);
            }
        }
        tv_dict_set_keys_readonly(v_event);
        let mut aco: aco_save_T = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, buf);
        apply_autocmds_group(event, fname, fname_io, false_0 != 0, group, buf, eap, data);
        aucmd_restbuf(&raw mut aco);
        restore_v_event(v_event, &raw mut save_v_event);
    }
    xfree(fname as *mut ::core::ffi::c_void);
    xfree(fname_io as *mut ::core::ffi::c_void);
    if !data.is_null() {
        api_free_object(*data);
        xfree(data as *mut ::core::ffi::c_void);
    }
    xfree(e as *mut ::core::ffi::c_void);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn apply_autocmds(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
) -> bool {
    return apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<exarg_T>(),
        ::core::ptr::null_mut::<Object>(),
    );
}
pub unsafe extern "C" fn apply_autocmds_exarg(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
) -> bool {
    return apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        eap,
        ::core::ptr::null_mut::<Object>(),
    );
}
pub unsafe extern "C" fn apply_autocmds_retval(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut buf: *mut buf_T,
    mut retval: *mut ::core::ffi::c_int,
) -> bool {
    if should_abort(*retval) {
        return false_0 != 0;
    }
    let mut did_cmd: bool = apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        AUGROUP_ALL as ::core::ffi::c_int,
        buf,
        ::core::ptr::null_mut::<exarg_T>(),
        ::core::ptr::null_mut::<Object>(),
    );
    if did_cmd as ::core::ffi::c_int != 0 && aborting() as ::core::ffi::c_int != 0 {
        *retval = FAIL;
    }
    return did_cmd;
}
pub unsafe extern "C" fn has_event(mut event: event_T) -> bool {
    return (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size != 0 as size_t;
}
unsafe extern "C" fn has_cursorhold() -> bool {
    return has_event(
        (if get_real_state() == MODE_NORMAL_BUSY {
            EVENT_CURSORHOLD as ::core::ffi::c_int
        } else {
            EVENT_CURSORHOLDI as ::core::ffi::c_int
        }) as event_T,
    );
}
pub unsafe extern "C" fn trigger_cursorhold() -> bool {
    if !did_cursorhold.get()
        && has_cursorhold() as ::core::ffi::c_int != 0
        && reg_recording.get() == 0 as ::core::ffi::c_int
        && (*typebuf.ptr()).tb_len == 0 as ::core::ffi::c_int
        && !ins_compl_active()
    {
        let mut state: ::core::ffi::c_int = get_real_state();
        if state == MODE_NORMAL_BUSY || state & MODE_INSERT != 0 as ::core::ffi::c_int {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn apply_autocmds_group(
    mut event: event_T,
    mut fname: *mut ::core::ffi::c_char,
    mut fname_io: *mut ::core::ffi::c_char,
    mut force: bool,
    mut group: ::core::ffi::c_int,
    mut buf: *mut buf_T,
    mut eap: *mut exarg_T,
    mut data: *mut Object,
) -> bool {
    let mut win_ignore: bool = false;
    let mut save_autocmd_fname: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_autocmd_fname_full: bool = false;
    let mut save_autocmd_bufnr: ::core::ffi::c_int = 0;
    let mut save_autocmd_match: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_autocmd_busy: ::core::ffi::c_int = 0;
    let mut save_autocmd_nested: ::core::ffi::c_int = 0;
    let mut save_changed: bool = false;
    let mut old_curbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut afile_orig: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut funccal_entry: funccal_entry_T = funccal_entry_T {
        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        next: ::core::ptr::null_mut::<funccal_entry_T>(),
    };
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut patcmd: AutoPatCmd = AutoPatCmd {
        lastpat: ::core::ptr::null_mut::<AutoPat>(),
        auidx: 0,
        ausize: 0,
        afile_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        sfname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        tail: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        group: 0,
        event: EVENT_BUFADD,
        script_ctx: sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        },
        arg_bufnr: 0,
        data: ::core::ptr::null_mut::<Object>(),
        next: ::core::ptr::null_mut::<AutoPatCmd>(),
    };
    let mut sfname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: bool = false_0 != 0;
    static nesting: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut save_cmdarg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static filechangeshell_busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    let mut wait_time: proftime_T = 0;
    let mut did_save_redobuff: bool = false_0 != 0;
    let mut save_redo: save_redo_T = save_redo_T {
        sr_redobuff: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        sr_old_redobuff: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
    };
    let save_KeyTyped: bool = KeyTyped.get();
    if !(event as ::core::ffi::c_uint == NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size == 0 as size_t
        || is_autocmd_blocked() as ::core::ffi::c_int != 0)
    {
        if !(autocmd_busy.get() as ::core::ffi::c_int != 0
            && !(force as ::core::ffi::c_int != 0
                || autocmd_nested.get() as ::core::ffi::c_int != 0))
        {
            if !aborting() {
                if !(filechangeshell_busy.get() as ::core::ffi::c_int != 0
                    && (event as ::core::ffi::c_uint
                        == EVENT_FILECHANGEDSHELL as ::core::ffi::c_int as ::core::ffi::c_uint
                        || event as ::core::ffi::c_uint
                            == EVENT_FILECHANGEDSHELLPOST as ::core::ffi::c_int
                                as ::core::ffi::c_uint))
                {
                    if !event_ignored(event, p_ei.get()) {
                        win_ignore = false_0 != 0;
                        if buf == curbuf.get()
                            && (*event_names.ptr())[event as usize].event <= 0 as ::core::ffi::c_int
                        {
                            win_ignore = event_ignored(event, (*curwin.get()).w_onebuf_opt.wo_eiw);
                        } else if !buf.is_null()
                            && (*event_names.ptr())[event as usize].event <= 0 as ::core::ffi::c_int
                            && (*buf).b_nwindows > 0 as ::core::ffi::c_int
                        {
                            win_ignore = true_0 != 0;
                            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                            while !tp.is_null() {
                                let mut wp: *mut win_T = if tp == curtab.get() {
                                    firstwin.get()
                                } else {
                                    (*tp).tp_firstwin
                                };
                                while !wp.is_null() {
                                    if (*wp).w_buffer == buf
                                        && !event_ignored(event, (*wp).w_onebuf_opt.wo_eiw)
                                    {
                                        win_ignore = false_0 != 0;
                                        break;
                                    } else {
                                        wp = (*wp).w_next;
                                    }
                                }
                                tp = (*tp).tp_next as *mut tabpage_T;
                            }
                        }
                        if !win_ignore {
                            if nesting.get() == 10 as ::core::ffi::c_int {
                                emsg(gettext(
                                    (e_autocommand_nesting_too_deep.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ));
                            } else if !(autocmd_no_enter.get() != 0
                                && (event as ::core::ffi::c_uint
                                    == EVENT_WINENTER as ::core::ffi::c_int as ::core::ffi::c_uint
                                    || event as ::core::ffi::c_uint
                                        == EVENT_BUFENTER as ::core::ffi::c_int
                                            as ::core::ffi::c_uint)
                                || autocmd_no_leave.get() != 0
                                    && (event as ::core::ffi::c_uint
                                        == EVENT_WINLEAVE as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_BUFLEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint))
                            {
                                save_autocmd_fname = autocmd_fname.get();
                                save_autocmd_fname_full = autocmd_fname_full.get();
                                save_autocmd_bufnr = autocmd_bufnr.get();
                                save_autocmd_match = autocmd_match.get();
                                save_autocmd_busy = autocmd_busy.get() as ::core::ffi::c_int;
                                save_autocmd_nested = autocmd_nested.get() as ::core::ffi::c_int;
                                save_changed = (*curbuf.get()).b_changed != 0;
                                old_curbuf = curbuf.get();
                                if fname_io.is_null() {
                                    if event as ::core::ffi::c_uint
                                        == EVENT_COLORSCHEME as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEMEPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_OPTIONSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MODECHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MARKSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        autocmd_fname
                                            .set(::core::ptr::null_mut::<::core::ffi::c_char>());
                                    } else if !fname.is_null()
                                        && ends_excmd(*fname as ::core::ffi::c_int) == 0
                                    {
                                        autocmd_fname.set(fname);
                                    } else if !buf.is_null() {
                                        autocmd_fname.set((*buf).b_ffname);
                                    } else {
                                        autocmd_fname
                                            .set(::core::ptr::null_mut::<::core::ffi::c_char>());
                                    }
                                } else {
                                    autocmd_fname.set(fname_io);
                                }
                                afile_orig = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                if !(*autocmd_fname.ptr()).is_null() {
                                    afile_orig = xstrdup(autocmd_fname.get());
                                    autocmd_fname
                                        .set(xstrnsave(autocmd_fname.get(), MAXPATHL as size_t));
                                }
                                autocmd_fname_full.set(false_0 != 0);
                                autocmd_bufnr.set(if buf.is_null() {
                                    0 as ::core::ffi::c_int
                                } else {
                                    (*buf).handle as ::core::ffi::c_int
                                });
                                if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
                                    if buf.is_null() {
                                        fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    } else if event as ::core::ffi::c_uint
                                        == EVENT_SYNTAX as ::core::ffi::c_int as ::core::ffi::c_uint
                                    {
                                        fname = (*buf).b_p_syn;
                                    } else if event as ::core::ffi::c_uint
                                        == EVENT_FILETYPE as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        fname = (*buf).b_p_ft;
                                    } else {
                                        if !(*buf).b_sfname.is_null() {
                                            sfname = xstrdup((*buf).b_sfname);
                                        }
                                        fname = (*buf).b_ffname;
                                    }
                                    if fname.is_null() {
                                        fname = b"\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char;
                                    }
                                    fname = xstrdup(fname);
                                } else {
                                    sfname = xstrdup(fname);
                                    if event as ::core::ffi::c_uint
                                        == EVENT_CMDLINECHANGED as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDLINEENTER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDLINELEAVEPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDLINELEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDUNDEFINED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CURSORMOVEDC as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDWINENTER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_CMDWINLEAVE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEME as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_COLORSCHEMEPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_DIRCHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_DIRCHANGEDPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_FILETYPE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_FUNCUNDEFINED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MARKSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MENUPOPUP as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_MODECHANGED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_OPTIONSET as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_PROGRESS as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_QUICKFIXCMDPOST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_QUICKFIXCMDPRE as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_REMOTEREPLY as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_SIGNAL as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_SPELLFILEMISSING as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_SYNTAX as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_TABCLOSED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_USER as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_WINCLOSED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_WINRESIZED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                        || event as ::core::ffi::c_uint
                                            == EVENT_WINSCROLLED as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                    {
                                        fname = xstrdup(fname);
                                        autocmd_fname_full.set(true_0 != 0);
                                    } else {
                                        fname = FullName_save(fname, false_0 != 0);
                                    }
                                }
                                if fname.is_null() {
                                    xfree(sfname as *mut ::core::ffi::c_void);
                                    retval = false_0 != 0;
                                } else {
                                    autocmd_match.set(fname);
                                    (*RedrawingDisabled.ptr()) += 1;
                                    estack_push(
                                        ETYPE_AUCMD,
                                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                        0 as linenr_T,
                                    );
                                    save_current_sctx = current_sctx.get();
                                    if do_profiling.get() == PROF_YES {
                                        wait_time = prof_child_enter();
                                    }
                                    funccal_entry = funccal_entry_T {
                                        top_funccal: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                                        next: ::core::ptr::null_mut::<funccal_entry_T>(),
                                    };
                                    save_funccal(&raw mut funccal_entry);
                                    if !autocmd_busy.get() {
                                        save_search_patterns();
                                        if !ins_compl_active() {
                                            saveRedobuff(&raw mut save_redo);
                                            did_save_redobuff = true_0 != 0;
                                        }
                                        (*curbuf.get()).b_did_filetype =
                                            (*curbuf.get()).b_keep_filetype;
                                    }
                                    autocmd_busy.set(true_0 != 0);
                                    filechangeshell_busy.set(
                                        event as ::core::ffi::c_uint
                                            == EVENT_FILECHANGEDSHELL as ::core::ffi::c_int
                                                as ::core::ffi::c_uint,
                                    );
                                    (*nesting.ptr()) += 1;
                                    if event as ::core::ffi::c_uint
                                        == EVENT_FILETYPE as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        (*curbuf.get()).b_did_filetype = true_0 != 0;
                                    }
                                    tail = path_tail(fname);
                                    patcmd = AutoPatCmd_S {
                                        lastpat: ::core::ptr::null_mut::<AutoPat>(),
                                        auidx: 0 as size_t,
                                        ausize: (*autocmds.ptr())
                                            [event as ::core::ffi::c_int as usize]
                                            .size,
                                        afile_orig: afile_orig,
                                        fname: fname,
                                        sfname: sfname,
                                        tail: tail,
                                        group: group,
                                        event: event,
                                        script_ctx: sctx_T {
                                            sc_sid: 0,
                                            sc_seq: 0,
                                            sc_lnum: 0,
                                            sc_chan: 0,
                                        },
                                        arg_bufnr: autocmd_bufnr.get(),
                                        data: ::core::ptr::null_mut::<Object>(),
                                        next: ::core::ptr::null_mut::<AutoPatCmd>(),
                                    };
                                    aucmd_next(&raw mut patcmd);
                                    if !patcmd.lastpat.is_null() {
                                        patcmd.next = active_apc_list.get();
                                        active_apc_list.set(&raw mut patcmd);
                                        patcmd.data = data;
                                        let mut save_cmdbang: varnumber_T =
                                            get_vim_var_nr(VV_CMDBANG);
                                        if !eap.is_null() {
                                            save_cmdarg = set_cmdarg(
                                                eap,
                                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            );
                                            set_vim_var_nr(
                                                VV_CMDBANG,
                                                (*eap).forceit as varnumber_T,
                                            );
                                        } else {
                                            save_cmdarg =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        }
                                        retval = true_0 != 0;
                                        if nesting.get() == 1 as ::core::ffi::c_int {
                                            check_lnums(true_0 != 0);
                                        } else {
                                            check_lnums_nested(true_0 != 0);
                                        }
                                        let save_did_emsg: ::core::ffi::c_int = did_emsg.get();
                                        let save_ex_pressedreturn: bool = get_pressedreturn();
                                        do_cmdline(
                                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                            Some(
                                                getnextac
                                                    as unsafe extern "C" fn(
                                                        ::core::ffi::c_int,
                                                        *mut ::core::ffi::c_void,
                                                        ::core::ffi::c_int,
                                                        bool,
                                                    ) -> *mut ::core::ffi::c_char,
                                            ),
                                            &raw mut patcmd as *mut ::core::ffi::c_void,
                                            DOCMD_NOWAIT as ::core::ffi::c_int
                                                | DOCMD_VERBOSE as ::core::ffi::c_int
                                                | DOCMD_REPEAT as ::core::ffi::c_int,
                                        );
                                        (*did_emsg.ptr()) += save_did_emsg;
                                        set_pressedreturn(save_ex_pressedreturn);
                                        if nesting.get() == 1 as ::core::ffi::c_int {
                                            reset_lnums();
                                        }
                                        if !eap.is_null() {
                                            set_cmdarg(
                                                ::core::ptr::null_mut::<exarg_T>(),
                                                save_cmdarg,
                                            );
                                            set_vim_var_nr(VV_CMDBANG, save_cmdbang);
                                        }
                                        if active_apc_list.get() == &raw mut patcmd {
                                            active_apc_list.set(patcmd.next);
                                        }
                                    }
                                    (*RedrawingDisabled.ptr()) -= 1;
                                    autocmd_busy.set(save_autocmd_busy != 0);
                                    filechangeshell_busy.set(false_0 != 0);
                                    autocmd_nested.set(save_autocmd_nested != 0);
                                    xfree(
                                        (*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                            ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int)
                                                as isize,
                                        ))
                                        .es_name
                                            as *mut ::core::ffi::c_void,
                                    );
                                    estack_pop();
                                    xfree(afile_orig as *mut ::core::ffi::c_void);
                                    xfree(autocmd_fname.get() as *mut ::core::ffi::c_void);
                                    autocmd_fname.set(save_autocmd_fname);
                                    autocmd_fname_full.set(save_autocmd_fname_full);
                                    autocmd_bufnr.set(save_autocmd_bufnr);
                                    autocmd_match.set(save_autocmd_match);
                                    current_sctx.set(save_current_sctx);
                                    restore_funccal();
                                    if do_profiling.get() == PROF_YES {
                                        prof_child_exit(wait_time);
                                    }
                                    KeyTyped.set(save_KeyTyped);
                                    xfree(fname as *mut ::core::ffi::c_void);
                                    xfree(sfname as *mut ::core::ffi::c_void);
                                    (*nesting.ptr()) -= 1;
                                    if !autocmd_busy.get() {
                                        restore_search_patterns();
                                        if did_save_redobuff {
                                            restoreRedobuff(&raw mut save_redo);
                                        }
                                        (*curbuf.get()).b_did_filetype = false_0 != 0;
                                        while !(*au_pending_free_buf.ptr()).is_null() {
                                            let mut b: *mut buf_T =
                                                (*au_pending_free_buf.get()).b_next;
                                            xfree(au_pending_free_buf.get()
                                                as *mut ::core::ffi::c_void);
                                            au_pending_free_buf.set(b);
                                        }
                                        while !(*au_pending_free_win.ptr()).is_null() {
                                            let mut w: *mut win_T =
                                                (*au_pending_free_win.get()).w_next;
                                            xfree(au_pending_free_win.get()
                                                as *mut ::core::ffi::c_void);
                                            au_pending_free_win.set(w);
                                        }
                                    }
                                    if curbuf.get() == old_curbuf
                                        && (event as ::core::ffi::c_uint
                                            == EVENT_BUFREADPOST as ::core::ffi::c_int
                                                as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_BUFWRITEPOST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_FILEAPPENDPOST as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_VIMLEAVE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            || event as ::core::ffi::c_uint
                                                == EVENT_VIMLEAVEPRE as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                    {
                                        if (*curbuf.get()).b_changed
                                            != save_changed as ::core::ffi::c_int
                                        {
                                            need_maketitle.set(true_0 != 0);
                                        }
                                        (*curbuf.get()).b_changed =
                                            save_changed as ::core::ffi::c_int;
                                    }
                                    au_cleanup();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if event as ::core::ffi::c_uint == EVENT_BUFWIPEOUT as ::core::ffi::c_int as ::core::ffi::c_uint
        && !buf.is_null()
    {
        aubuflocal_remove(buf);
    }
    if retval as ::core::ffi::c_int == OK
        && event as ::core::ffi::c_uint
            == EVENT_FILETYPE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*curbuf.get()).b_au_did_filetype = true_0 != 0;
    }
    return retval;
}
pub unsafe extern "C" fn do_termresponse_autocmd(sequence: String_0) {
    let mut data: Dict = Dict {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<KeyValuePair>(),
    };
    let mut data__items: [KeyValuePair; 1] = [KeyValuePair {
        key: String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0,
        },
        value: Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
    }; 1];
    data.capacity = 1 as size_t;
    data.items = &raw mut data__items as *mut KeyValuePair;
    let c2rust_fresh11 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh11 as isize) = key_value_pair {
        key: cstr_as_string(b"sequence\0".as_ptr() as *const ::core::ffi::c_char),
        value: object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed { string: sequence },
        },
    };
    let mut c2rust_lvalue: Object = object {
        type_0: kObjectTypeDict,
        data: C2Rust_Unnamed { dict: data },
    };
    apply_autocmds_group(
        EVENT_TERMRESPONSE,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        true_0 != 0,
        AUGROUP_ALL as ::core::ffi::c_int,
        ::core::ptr::null_mut::<buf_T>(),
        ::core::ptr::null_mut::<exarg_T>(),
        &raw mut c2rust_lvalue,
    );
    termresponse_changed.set(true_0 != 0);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn block_autocmds() {
    if !is_autocmd_blocked() {
        termresponse_changed.set(false_0 != 0);
    }
    (*autocmd_blocked.ptr()) += 1;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn unblock_autocmds() {
    (*autocmd_blocked.ptr()) -= 1;
    if !is_autocmd_blocked()
        && termresponse_changed.get() as ::core::ffi::c_int != 0
        && has_event(EVENT_TERMRESPONSE) as ::core::ffi::c_int != 0
    {
        let sequence: String_0 = cstr_to_string(get_vim_var_str(VV_TERMRESPONSE));
        do_termresponse_autocmd(sequence);
        api_free_string(sequence);
    }
}
pub unsafe extern "C" fn is_autocmd_blocked() -> bool {
    return autocmd_blocked.get() != 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn aucmd_next(mut apc: *mut AutoPatCmd) {
    let entry: *mut estack_T = ((*exestack.ptr()).ga_data as *mut estack_T)
        .offset((*exestack.ptr()).ga_len as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    let acs: *mut AutoCmdVec =
        (autocmds.ptr() as *mut AutoCmdVec).offset((*apc).event as ::core::ffi::c_int as isize);
    '_c2rust_label: {
        if (*apc).ausize <= (*acs).size {
        } else {
            __assert_fail(
                b"apc->ausize <= kv_size(*acs)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2077 as ::core::ffi::c_uint,
                b"void aucmd_next(AutoPatCmd *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut i: size_t = (*apc).auidx;
    while i < (*apc).ausize && !got_int.get() {
        let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
        let ap: *mut AutoPat = (*ac).pat;
        's_11: {
            if !ap.is_null() {
                if ap != (*apc).lastpat {
                    if (*apc).group != AUGROUP_ALL as ::core::ffi::c_int
                        && (*apc).group != (*ap).group
                    {
                        break 's_11;
                    } else if if (*ap).buflocal_nr == 0 as ::core::ffi::c_int {
                        !match_file_pat(
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut (*ap).reg_prog,
                            (*apc).fname,
                            (*apc).sfname,
                            (*apc).tail,
                            (*ap).allow_dirs as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                    } else {
                        ((*ap).buflocal_nr != (*apc).arg_bufnr) as ::core::ffi::c_int
                    } != 0
                    {
                        break 's_11;
                    } else {
                        let name: *const ::core::ffi::c_char = event_nr2name((*apc).event);
                        let s: *const ::core::ffi::c_char =
                            gettext(b"%s Autocommands for \"%s\"\0".as_ptr()
                                as *const ::core::ffi::c_char);
                        let sourcing_name_len: size_t = strlen(s)
                            .wrapping_add(strlen(name))
                            .wrapping_add((*ap).patlen as size_t)
                            .wrapping_add(1 as size_t);
                        let namep: *mut ::core::ffi::c_char =
                            xmalloc(sourcing_name_len) as *mut ::core::ffi::c_char;
                        snprintf(namep, sourcing_name_len, s, name, (*ap).pat);
                        if p_verbose.get() >= 8 as OptInt {
                            verbose_enter();
                            smsg(
                                0 as ::core::ffi::c_int,
                                gettext(b"Executing %s\0".as_ptr() as *const ::core::ffi::c_char),
                                namep,
                            );
                            verbose_leave();
                        }
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            &raw mut (*entry).es_name as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL_0;
                        let _ = *ptr_;
                        (*entry).es_name = namep;
                        (*entry).es_info.aucmd = apc;
                    }
                }
                (*apc).lastpat = ap;
                (*apc).auidx = i;
                line_breakcheck();
                return;
            }
        }
        i = i.wrapping_add(1);
    }
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*entry).es_name as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
    (*entry).es_info.aucmd = ::core::ptr::null_mut::<AutoPatCmd>();
    (*apc).lastpat = ::core::ptr::null_mut::<AutoPat>();
    (*apc).auidx = SIZE_MAX as size_t;
}
unsafe extern "C" fn au_callback(mut ac: *const AutoCmd, mut apc: *const AutoPatCmd) -> bool {
    let mut callback: Callback = (*ac).handler_fn;
    if callback.type_0 as ::core::ffi::c_uint
        == kCallbackLua as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut data: Dict = Dict {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        };
        let mut data__items: [KeyValuePair; 7] = [KeyValuePair {
            key: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            value: Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
        }; 7];
        data.capacity = 7 as size_t;
        data.items = &raw mut data__items as *mut KeyValuePair;
        let c2rust_fresh3 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh3 as isize) = key_value_pair {
            key: cstr_as_string(b"id\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed { integer: (*ac).id },
            },
        };
        let c2rust_fresh4 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh4 as isize) = key_value_pair {
            key: cstr_as_string(b"event\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(event_nr2name((*apc).event)),
                },
            },
        };
        let c2rust_fresh5 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh5 as isize) = key_value_pair {
            key: cstr_as_string(b"file\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*apc).afile_orig),
                },
            },
        };
        let c2rust_fresh6 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh6 as isize) = key_value_pair {
            key: cstr_as_string(b"match\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string(autocmd_match.get()),
                },
            },
        };
        let c2rust_fresh7 = data.size;
        data.size = data.size.wrapping_add(1);
        *data.items.offset(c2rust_fresh7 as isize) = key_value_pair {
            key: cstr_as_string(b"buf\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: autocmd_bufnr.get() as Integer,
                },
            },
        };
        if !(*apc).data.is_null() {
            let c2rust_fresh8 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh8 as isize) = key_value_pair {
                key: cstr_as_string(b"data\0".as_ptr() as *const ::core::ffi::c_char),
                value: *(*apc).data,
            };
        }
        let mut group: ::core::ffi::c_int = (*(*ac).pat).group;
        match group {
            -2 => {
                abort();
            }
            -1 | -3 | -4 => {}
            _ => {
                let c2rust_fresh9 = data.size;
                data.size = data.size.wrapping_add(1);
                *data.items.offset(c2rust_fresh9 as isize) = key_value_pair {
                    key: cstr_as_string(b"group\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: group as Integer,
                        },
                    },
                };
            }
        }
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 1] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 1];
        args.capacity = 1 as size_t;
        args.items = &raw mut args__items as *mut Object;
        let c2rust_fresh10 = args.size;
        args.size = args.size.wrapping_add(1);
        *args.items.offset(c2rust_fresh10 as isize) = object {
            type_0: kObjectTypeDict,
            data: C2Rust_Unnamed { dict: data },
        };
        let mut result: Object = nlua_call_ref(
            callback.data.luaref,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            ::core::ptr::null_mut::<Error>(),
        );
        return result.type_0 as ::core::ffi::c_uint
            == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            && result.data.boolean as ::core::ffi::c_int == true_0;
    } else {
        let mut argsin: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        callback_call(
            &raw mut callback,
            0 as ::core::ffi::c_int,
            &raw mut argsin,
            &raw mut rettv,
        );
        return false_0 != 0;
    };
}
pub unsafe extern "C" fn getnextac(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let apc: *mut AutoPatCmd = cookie as *mut AutoPatCmd;
    let acs: *mut AutoCmdVec =
        (autocmds.ptr() as *mut AutoCmdVec).offset((*apc).event as ::core::ffi::c_int as isize);
    aucmd_next(apc);
    if (*apc).lastpat.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    '_c2rust_label: {
        if (*apc).auidx < (*acs).size {
        } else {
            __assert_fail(
                b"apc->auidx < kv_size(*acs)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2193 as ::core::ffi::c_uint,
                b"char *getnextac(int, void *, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let ac: *mut AutoCmd = (*acs).items.offset((*apc).auidx as isize);
    '_c2rust_label_0: {
        if !(*ac).pat.is_null() {
        } else {
            __assert_fail(
                b"ac->pat != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2195 as ::core::ffi::c_uint,
                b"char *getnextac(int, void *, int, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut oneshot: bool = (*ac).once;
    if p_verbose.get() >= 9 as OptInt {
        verbose_enter_scroll();
        let mut handler_str: *mut ::core::ffi::c_char = aucmd_handler_to_string(ac);
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"autocommand %s\0".as_ptr() as *const ::core::ffi::c_char),
            handler_str,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut handler_str as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        verbose_leave_scroll();
    }
    autocmd_nested.set((*ac).nested);
    current_sctx.set((*ac).script_ctx);
    (*apc).script_ctx = current_sctx.get();
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !(*ac).handler_cmd.is_null() {
        retval = xstrdup((*ac).handler_cmd);
    } else {
        let mut ac_copy: AutoCmd = *ac;
        (*ac).pat = if oneshot as ::core::ffi::c_int != 0 {
            ::core::ptr::null_mut::<AutoPat>()
        } else {
            (*ac).pat
        };
        let mut rv: bool = au_callback(&raw mut ac_copy, apc);
        if oneshot {
            (*(*acs).items.offset((*apc).auidx as isize)).pat = ac_copy.pat;
        }
        oneshot = oneshot as ::core::ffi::c_int != 0 || rv as ::core::ffi::c_int != 0;
        retval = xcalloc(1 as size_t, 1 as size_t) as *mut ::core::ffi::c_char;
    }
    if oneshot {
        aucmd_del((*acs).items.offset((*apc).auidx as isize));
    }
    if (*apc).auidx < (*apc).ausize {
        (*apc).auidx = (*apc).auidx.wrapping_add(1);
    } else {
        (*apc).auidx = SIZE_MAX as size_t;
    }
    return retval;
}
pub unsafe extern "C" fn has_autocmd(
    mut event: event_T,
    mut sfname: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
) -> bool {
    let mut tail: *mut ::core::ffi::c_char = path_tail(sfname);
    let mut retval: bool = false_0 != 0;
    let mut fname: *mut ::core::ffi::c_char = FullName_save(sfname, false_0 != 0);
    if fname.is_null() {
        return false_0 != 0;
    }
    let acs: *mut AutoCmdVec =
        (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
    let mut i: size_t = 0 as size_t;
    while i < (*acs).size {
        let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
        if !ap.is_null()
            && (if (*ap).buflocal_nr == 0 as ::core::ffi::c_int {
                match_file_pat(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    &raw mut (*ap).reg_prog,
                    fname,
                    sfname,
                    tail,
                    (*ap).allow_dirs as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
            } else {
                (!buf.is_null() && (*ap).buflocal_nr == (*buf).handle) as ::core::ffi::c_int
            }) != 0
        {
            retval = true_0 != 0;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }
    xfree(fname as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn expand_get_augroup_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return augroup_name(idx + 1 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn set_context_in_autocmd(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
    mut doautocmd: bool,
) -> *mut ::core::ffi::c_char {
    autocmd_include_groups.set(false_0 != 0);
    let mut p: *mut ::core::ffi::c_char = arg;
    let mut group: ::core::ffi::c_int = arg_augroup_get(&raw mut arg);
    if *arg as ::core::ffi::c_int == NUL
        && group != AUGROUP_ALL as ::core::ffi::c_int
        && !ascii_iswhite(*arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
    {
        arg = p;
        group = AUGROUP_ALL as ::core::ffi::c_int;
    }
    p = arg;
    while *p as ::core::ffi::c_int != NUL && !ascii_iswhite(*p as ::core::ffi::c_int) {
        if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            arg = p.offset(1 as ::core::ffi::c_int as isize);
        }
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == NUL {
        if group == AUGROUP_ALL as ::core::ffi::c_int {
            autocmd_include_groups.set(true_0 != 0);
        }
        (*xp).xp_context = EXPAND_EVENTS as ::core::ffi::c_int;
        (*xp).xp_pattern = arg;
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    arg = skipwhite(p);
    while *arg as ::core::ffi::c_int != 0
        && (!ascii_iswhite(*arg as ::core::ffi::c_int)
            || *arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int)
    {
        arg = arg.offset(1);
    }
    if *arg != 0 {
        return arg;
    }
    if doautocmd {
        (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn expand_get_event_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut name: *mut ::core::ffi::c_char = augroup_name(idx + 1 as ::core::ffi::c_int);
    if !name.is_null() {
        if !autocmd_include_groups.get()
            || name == get_deleted_augroup() as *mut ::core::ffi::c_char
        {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        return name;
    }
    let mut i: ::core::ffi::c_int = idx - next_augroup_id.get();
    if i < 0 as ::core::ffi::c_int || i >= NUM_EVENTS as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return (*event_names.ptr())[i as usize].name;
}
pub unsafe extern "C" fn get_event_name_no_group(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
    mut win: bool,
) -> *mut ::core::ffi::c_char {
    if idx < 0 as ::core::ffi::c_int || idx >= NUM_EVENTS as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !win {
        return (*event_names.ptr())[idx as usize].name;
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < NUM_EVENTS as ::core::ffi::c_int {
        j += ((*event_names.ptr())[i as usize].event <= 0 as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if j == idx + 1 as ::core::ffi::c_int {
            return (*event_names.ptr())[i as usize].name;
        }
        i += 1;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn autocmd_supported(event: *const ::core::ffi::c_char) -> bool {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return event_name2nr(event, &raw mut p) as ::core::ffi::c_uint
        != NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint;
}
pub unsafe extern "C" fn au_exists(arg: *const ::core::ffi::c_char) -> bool {
    let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut event: event_T = EVENT_BUFADD;
    let mut acs: *mut AutoCmdVec = ::core::ptr::null_mut::<AutoCmdVec>();
    let mut buflocal_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut retval: bool = false_0 != 0;
    let arg_save: *mut ::core::ffi::c_char = xstrdup(arg);
    let mut p: *mut ::core::ffi::c_char = strchr(arg_save, '#' as ::core::ffi::c_int);
    if !p.is_null() {
        let c2rust_fresh13 = p;
        p = p.offset(1);
        *c2rust_fresh13 = NUL as ::core::ffi::c_char;
    }
    let mut group: ::core::ffi::c_int = augroup_find(arg_save);
    let mut event_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_theend: {
        if group == AUGROUP_ERROR as ::core::ffi::c_int {
            group = AUGROUP_ALL as ::core::ffi::c_int;
            event_name = arg_save;
        } else if p.is_null() {
            retval = true_0 != 0;
            break '_theend;
        } else {
            event_name = p;
            p = strchr(event_name, '#' as ::core::ffi::c_int);
            if !p.is_null() {
                let c2rust_fresh14 = p;
                p = p.offset(1);
                *c2rust_fresh14 = NUL as ::core::ffi::c_char;
            }
        }
        pattern = p;
        event = event_name2nr(event_name, &raw mut p);
        if event as ::core::ffi::c_uint != NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint {
            acs = (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
            if (*acs).size != 0 as size_t {
                if !pattern.is_null()
                    && strcasecmp(
                        pattern,
                        b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                {
                    buflocal_buf = curbuf.get();
                }
                let mut i: size_t = 0 as size_t;
                while i < (*acs).size {
                    let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
                    if !ap.is_null()
                        && (group == AUGROUP_ALL as ::core::ffi::c_int || (*ap).group == group)
                        && (pattern.is_null()
                            || (if buflocal_buf.is_null() {
                                (path_fnamecmp((*ap).pat, pattern) == 0 as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            } else {
                                ((*ap).buflocal_nr == (*buflocal_buf).handle) as ::core::ffi::c_int
                            }) != 0)
                    {
                        retval = true_0 != 0;
                        break;
                    } else {
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    }
    xfree(arg_save as *mut ::core::ffi::c_void);
    return retval;
}
pub unsafe extern "C" fn aupat_is_buflocal(
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
) -> bool {
    return patlen >= 8 as ::core::ffi::c_int
        && strncmp(
            pat,
            b"<buffer\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
        && *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
            == '>' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn aupat_get_buflocal_nr(
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    '_c2rust_label: {
        if aupat_is_buflocal(pat, patlen) {
        } else {
            __assert_fail(
                b"aupat_is_buflocal(pat, patlen)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2514 as ::core::ffi::c_uint,
                b"int aupat_get_buflocal_nr(const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if patlen == 8 as ::core::ffi::c_int {
        return (*curbuf.get()).handle as ::core::ffi::c_int;
    }
    if patlen > 9 as ::core::ffi::c_int
        && *pat.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
    {
        if patlen == 13 as ::core::ffi::c_int
            && strncasecmp(
                pat as *mut ::core::ffi::c_char,
                b"<buffer=abuf>\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                13 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            return autocmd_bufnr.get();
        }
        if skipdigits(pat.offset(8 as ::core::ffi::c_int as isize))
            == pat
                .offset(patlen as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_char
        {
            return atoi(pat.offset(8 as ::core::ffi::c_int as isize));
        }
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn aupat_normalize_buflocal_pat(
    mut dest: *mut ::core::ffi::c_char,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    mut buflocal_nr: ::core::ffi::c_int,
) {
    '_c2rust_label: {
        if aupat_is_buflocal(pat, patlen) {
        } else {
            __assert_fail(
                b"aupat_is_buflocal(pat, patlen)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2539 as ::core::ffi::c_uint,
                b"void aupat_normalize_buflocal_pat(char *, const char *, int, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if buflocal_nr == 0 as ::core::ffi::c_int {
        buflocal_nr = (*curbuf.get()).handle as ::core::ffi::c_int;
    }
    snprintf(
        dest,
        BUFLOCAL_PAT_LEN as ::core::ffi::c_int as size_t,
        b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
        buflocal_nr,
    );
}
pub unsafe extern "C" fn autocmd_delete_id(mut id: int64_t) -> bool {
    '_c2rust_label: {
        if id > 0 as int64_t {
        } else {
            __assert_fail(
                b"id > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2560 as ::core::ffi::c_uint,
                b"_Bool autocmd_delete_id(int64_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut success: bool = false_0 != 0;
    let mut event: event_T = EVENT_BUFADD;
    while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if (*ac).id == id {
                aucmd_del(ac);
                success = true_0 != 0;
            }
            i = i.wrapping_add(1);
        }
        event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
    }
    return success;
}
pub unsafe extern "C" fn aucmd_handler_to_string(mut ac: *mut AutoCmd) -> *mut ::core::ffi::c_char {
    if !(*ac).handler_cmd.is_null() {
        return xstrdup((*ac).handler_cmd);
    }
    return callback_to_string(&raw mut (*ac).handler_fn, ::core::ptr::null_mut::<Arena>());
}
unsafe extern "C" fn arg_event_skip(
    mut arg: *mut ::core::ffi::c_char,
    mut have_group: bool,
) -> *mut ::core::ffi::c_char {
    let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
        if *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
            && !ascii_iswhite(*arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            semsg(
                gettext(
                    b"E215: Illegal character after *: %s\0".as_ptr() as *const ::core::ffi::c_char
                ),
                arg,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        pat = arg.offset(1 as ::core::ffi::c_int as isize);
    } else {
        pat = arg;
        while *pat as ::core::ffi::c_int != 0
            && *pat as ::core::ffi::c_int != '|' as ::core::ffi::c_int
            && !ascii_iswhite(*pat as ::core::ffi::c_int)
        {
            if event_name2nr(pat, &raw mut p) as ::core::ffi::c_int
                >= NUM_EVENTS as ::core::ffi::c_int
            {
                if have_group {
                    semsg(
                        gettext(b"E216: No such event: %s\0".as_ptr() as *const ::core::ffi::c_char),
                        pat,
                    );
                } else {
                    semsg(
                        gettext(b"E216: No such group or event: %s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        pat,
                    );
                }
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            pat = p;
        }
    }
    return pat;
}
unsafe extern "C" fn arg_augroup_get(
    mut argp: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = *argp;
    p = arg;
    while *p as ::core::ffi::c_int != 0
        && !ascii_iswhite(*p as ::core::ffi::c_int)
        && *p as ::core::ffi::c_int != '|' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    if p <= arg {
        return AUGROUP_ALL as ::core::ffi::c_int;
    }
    let mut group_name: *mut ::core::ffi::c_char = xmemdupz(
        arg as *const ::core::ffi::c_void,
        p.offset_from(arg) as size_t,
    ) as *mut ::core::ffi::c_char;
    let mut group: ::core::ffi::c_int = augroup_find(group_name);
    if group == AUGROUP_ERROR as ::core::ffi::c_int {
        group = AUGROUP_ALL as ::core::ffi::c_int;
    } else {
        *argp = skipwhite(p);
    }
    xfree(group_name as *mut ::core::ffi::c_void);
    return group;
}
unsafe extern "C" fn arg_autocmd_flag_get(
    mut flag: *mut bool,
    mut cmd_ptr: *mut *mut ::core::ffi::c_char,
    mut pattern: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    if strncmp(*cmd_ptr, pattern, len as size_t) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*(*cmd_ptr).offset(len as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        if *flag {
            semsg(
                gettext(&raw const e_duparg2 as *const ::core::ffi::c_char),
                pattern,
            );
            return true_0 != 0;
        }
        *flag = true_0 != 0;
        *cmd_ptr = skipwhite((*cmd_ptr).offset(len as isize));
    }
    return false_0 != 0;
}
static pending_vimresume: GlobalCell<TriState> = GlobalCell::new(kFalse);
unsafe extern "C" fn vimresume_event(mut _argv: *mut *mut ::core::ffi::c_void) {
    apply_autocmds(
        EVENT_VIMRESUME,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        ::core::ptr::null_mut::<buf_T>(),
    );
    pending_vimresume.set(kFalse);
}
pub unsafe extern "C" fn may_trigger_vim_suspend_resume(mut suspend: bool) {
    if suspend as ::core::ffi::c_int != 0
        && pending_vimresume.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int
    {
        pending_vimresume.set(kNone);
        apply_autocmds(
            EVENT_VIMSUSPEND,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<buf_T>(),
        );
        pending_vimresume.set(kTrue);
    } else if !suspend
        && pending_vimresume.get() as ::core::ffi::c_int == kTrue as ::core::ffi::c_int
    {
        pending_vimresume.set(kNone);
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    vimresume_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}
pub unsafe extern "C" fn do_autocmd_uienter(mut chanid: uint64_t, mut attached: bool) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if starting.get() == NO_SCREEN {
        return;
    }
    if recursive.get() {
        return;
    }
    recursive.set(true_0 != 0);
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
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    '_c2rust_label: {
        if chanid < 9223372036854775807 as uint64_t {
        } else {
            __assert_fail(
                b"chanid < VARNUMBER_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2697 as ::core::ffi::c_uint,
                b"void do_autocmd_uienter(uint64_t, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    tv_dict_add_nr(
        dict,
        b"chan\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        chanid as varnumber_T,
    );
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(
        (if attached as ::core::ffi::c_int != 0 {
            EVENT_UIENTER as ::core::ffi::c_int
        } else {
            EVENT_UILEAVE as ::core::ffi::c_int
        }) as event_T,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    restore_v_event(dict, &raw mut save_v_event);
    recursive.set(false_0 != 0);
}
pub unsafe extern "C" fn do_autocmd_focusgained(mut gained: bool) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    static last_time: GlobalCell<Timestamp> = GlobalCell::new(0 as Timestamp);
    if recursive.get() {
        return;
    }
    recursive.set(true_0 != 0);
    apply_autocmds(
        (if gained as ::core::ffi::c_int != 0 {
            EVENT_FOCUSGAINED as ::core::ffi::c_int
        } else {
            EVENT_FOCUSLOST as ::core::ffi::c_int
        }) as event_T,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    if gained as ::core::ffi::c_int != 0
        && (*last_time.ptr()).wrapping_add(2000 as ::core::ffi::c_int as Timestamp) < os_now()
    {
        check_timestamps(true_0);
        last_time.set(os_now() as Timestamp);
    }
    recursive.set(false_0 != 0);
}
pub unsafe extern "C" fn do_filetype_autocmd(mut buf: *mut buf_T, mut force: bool) -> bool {
    static ft_recursive: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if ft_recursive.get() > 0 as ::core::ffi::c_int && !force {
        return false_0 != 0;
    }
    let mut secure_save: ::core::ffi::c_int = secure.get();
    secure.set(0 as ::core::ffi::c_int);
    (*ft_recursive.ptr()) += 1;
    (*buf).b_did_filetype = true_0 != 0;
    let mut ret: bool = apply_autocmds(
        EVENT_FILETYPE,
        (*buf).b_p_ft,
        (*buf).b_fname,
        force as ::core::ffi::c_int != 0 || ft_recursive.get() == 1 as ::core::ffi::c_int,
        buf,
    );
    (*ft_recursive.ptr()) -= 1;
    secure.set(secure_save);
    return ret;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
static event_names: GlobalCell<[event_name; 145]> = GlobalCell::new([
    event_name {
        len: 6 as size_t,
        name: b"BufAdd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFADD as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufCreate\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFADD as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufDelete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFDELETE as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"BufEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufFilePost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFFILEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufFilePre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFFILEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufHidden\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFHIDDEN as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"BufLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"BufModifiedSet\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFMODIFIEDSET as ::core::ffi::c_int),
    },
    event_name {
        len: 6 as size_t,
        name: b"BufNew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFNEW as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufNewFile\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFNEWFILE as ::core::ffi::c_int),
    },
    event_name {
        len: 7 as size_t,
        name: b"BufRead\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufReadCmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADCMD as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufReadPost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufReadPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFREADPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"BufUnload\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFUNLOAD as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWinEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWINENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWinLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWINLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 10 as size_t,
        name: b"BufWipeout\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWIPEOUT as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"BufWrite\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWriteCmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITECMD as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"BufWritePost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"BufWritePre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_BUFWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"ChanInfo\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CHANINFO as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"ChanOpen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CHANOPEN as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"CmdlineChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINECHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CmdlineEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINEENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CmdlineLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINELEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"CmdlineLeavePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_CMDLINELEAVEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CmdUndefined\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CMDUNDEFINED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"CmdwinEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CMDWINENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"CmdwinLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_CMDWINLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"ColorScheme\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_COLORSCHEME as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"ColorSchemePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COLORSCHEMEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"CompleteChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COMPLETECHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"CompleteDone\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_COMPLETEDONE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"CompleteDonePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_COMPLETEDONEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"CursorHold\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORHOLD as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"CursorHoldI\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORHOLDI as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"CursorMoved\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORMOVED as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"CursorMovedC\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORMOVEDC as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"CursorMovedI\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_CURSORMOVEDI as ::core::ffi::c_int),
    },
    event_name {
        len: 17 as size_t,
        name: b"DiagnosticChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_DIAGNOSTICCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"DiffUpdated\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_DIFFUPDATED as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"DirChanged\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_DIRCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"DirChangedPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_DIRCHANGEDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"EncodingChanged\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_ENCODINGCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"ExitPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_EXITPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"FileAppendCmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEAPPENDCMD as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"FileAppendPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEAPPENDPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FileAppendPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEAPPENDPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FileChangedRO\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILECHANGEDRO as ::core::ffi::c_int),
    },
    event_name {
        len: 16 as size_t,
        name: b"FileChangedShell\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILECHANGEDSHELL as ::core::ffi::c_int),
    },
    event_name {
        len: 20 as size_t,
        name: b"FileChangedShellPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILECHANGEDSHELLPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileEncoding\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_ENCODINGCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"FileReadCmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEREADCMD as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileReadPost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"FileReadPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEREADPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"FileType\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILETYPE as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileWriteCmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEWRITECMD as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FileWritePost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEWRITEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"FileWritePre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILEWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"FilterReadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERREADPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"FilterReadPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERREADPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 15 as size_t,
        name: b"FilterWritePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERWRITEPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"FilterWritePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_FILTERWRITEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"FocusGained\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_FOCUSGAINED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"FocusLost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_FOCUSLOST as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"FuncUndefined\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_FUNCUNDEFINED as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"GUIEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_GUIENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"GUIFailed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_GUIFAILED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"InsertChange\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTCHANGE as ::core::ffi::c_int),
    },
    event_name {
        len: 13 as size_t,
        name: b"InsertCharPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTCHARPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"InsertEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"InsertLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"InsertLeavePre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_INSERTLEAVEPRE as ::core::ffi::c_int),
    },
    event_name {
        len: 9 as size_t,
        name: b"LspAttach\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_LSPATTACH as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"LspDetach\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_LSPDETACH as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"LspNotify\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_LSPNOTIFY as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"LspProgress\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_LSPPROGRESS as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"LspRequest\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_LSPREQUEST as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"LspTokenUpdate\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_LSPTOKENUPDATE as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"MarkSet\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_MARKSET as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"MenuPopup\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_MENUPOPUP as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"ModeChanged\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_MODECHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"OptionSet\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_OPTIONSET as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"PackChanged\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_PACKCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"PackChangedPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_PACKCHANGEDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"Progress\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_PROGRESS as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"QuickFixCmdPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_QUICKFIXCMDPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"QuickFixCmdPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_QUICKFIXCMDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"QuitPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_QUITPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"RecordingEnter\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_RECORDINGENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 14 as size_t,
        name: b"RecordingLeave\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_RECORDINGLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"RemoteReply\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_REMOTEREPLY as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"SafeState\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SAFESTATE as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"SearchWrapped\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_SEARCHWRAPPED as ::core::ffi::c_int),
    },
    event_name {
        len: 15 as size_t,
        name: b"SessionLoadPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SESSIONLOADPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 14 as size_t,
        name: b"SessionLoadPre\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SESSIONLOADPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 16 as size_t,
        name: b"SessionWritePost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SESSIONWRITEPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"ShellCmdPost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SHELLCMDPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 15 as size_t,
        name: b"ShellFilterPost\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: -(EVENT_SHELLFILTERPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 6 as size_t,
        name: b"Signal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SIGNAL as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"SourceCmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SOURCECMD as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"SourcePost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SOURCEPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"SourcePre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SOURCEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 16 as size_t,
        name: b"SpellFileMissing\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        event: EVENT_SPELLFILEMISSING as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"StdinReadPost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_STDINREADPOST as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"StdinReadPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_STDINREADPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"SwapExists\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SWAPEXISTS as ::core::ffi::c_int,
    },
    event_name {
        len: 6 as size_t,
        name: b"Syntax\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_SYNTAX as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TabClosed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TABCLOSED as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"TabClosedPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TABCLOSEDPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"TabEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TABENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"TabLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TABLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 6 as size_t,
        name: b"TabNew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TABNEW as ::core::ffi::c_int,
    },
    event_name {
        len: 13 as size_t,
        name: b"TabNewEntered\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TABNEWENTERED as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"TermChanged\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMCHANGED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TermClose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMCLOSE as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TermEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"TermLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"TermOpen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMOPEN as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"TermRequest\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMREQUEST as ::core::ffi::c_int,
    },
    event_name {
        len: 12 as size_t,
        name: b"TermResponse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_TERMRESPONSE as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"TextChanged\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGED as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextChangedI\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGEDI as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextChangedP\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGEDP as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextChangedT\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTCHANGEDT as ::core::ffi::c_int),
    },
    event_name {
        len: 12 as size_t,
        name: b"TextYankPost\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_TEXTYANKPOST as ::core::ffi::c_int),
    },
    event_name {
        len: 7 as size_t,
        name: b"UIEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_UIENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 7 as size_t,
        name: b"UILeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_UILEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 4 as size_t,
        name: b"User\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_USER as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"VimEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_VIMENTER as ::core::ffi::c_int,
    },
    event_name {
        len: 8 as size_t,
        name: b"VimLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_VIMLEAVE as ::core::ffi::c_int,
    },
    event_name {
        len: 11 as size_t,
        name: b"VimLeavePre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_VIMLEAVEPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"VimResized\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_VIMRESIZED as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"VimResume\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_VIMRESUME as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"VimSuspend\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_VIMSUSPEND as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"WinClosed\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_WINCLOSED as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"WinEnter\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_WINENTER as ::core::ffi::c_int),
    },
    event_name {
        len: 8 as size_t,
        name: b"WinLeave\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_WINLEAVE as ::core::ffi::c_int),
    },
    event_name {
        len: 6 as size_t,
        name: b"WinNew\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_WINNEW as ::core::ffi::c_int,
    },
    event_name {
        len: 9 as size_t,
        name: b"WinNewPre\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: EVENT_WINNEWPRE as ::core::ffi::c_int,
    },
    event_name {
        len: 10 as size_t,
        name: b"WinResized\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_WINRESIZED as ::core::ffi::c_int),
    },
    event_name {
        len: 11 as size_t,
        name: b"WinScrolled\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        event: -(EVENT_WINSCROLLED as ::core::ffi::c_int),
    },
]);
static autocmds: GlobalCell<[AutoCmdVec; 145]> = GlobalCell::new([
    AutoCmdVec {
        size: 0 as size_t,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
    AutoCmdVec {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<AutoCmd>(),
    },
]);
static event_hash: GlobalCell<[event_T; 145]> = GlobalCell::new([
    EVENT_USER,
    EVENT_BUFADD,
    EVENT_BUFNEW,
    EVENT_SIGNAL,
    EVENT_SYNTAX,
    EVENT_TABNEW,
    EVENT_WINNEW,
    EVENT_BUFREAD,
    EVENT_EXITPRE,
    EVENT_MARKSET,
    EVENT_QUITPRE,
    EVENT_UIENTER,
    EVENT_UILEAVE,
    EVENT_BUFENTER,
    EVENT_BUFLEAVE,
    EVENT_BUFWRITE,
    EVENT_CHANINFO,
    EVENT_CHANOPEN,
    EVENT_FILETYPE,
    EVENT_GUIENTER,
    EVENT_PROGRESS,
    EVENT_TABENTER,
    EVENT_TABLEAVE,
    EVENT_TERMOPEN,
    EVENT_VIMENTER,
    EVENT_VIMLEAVE,
    EVENT_WINENTER,
    EVENT_WINLEAVE,
    EVENT_LSPATTACH,
    EVENT_BUFCREATE,
    EVENT_TABCLOSED,
    EVENT_WINCLOSED,
    EVENT_BUFDELETE,
    EVENT_LSPDETACH,
    EVENT_SAFESTATE,
    EVENT_GUIFAILED,
    EVENT_BUFHIDDEN,
    EVENT_OPTIONSET,
    EVENT_TERMCLOSE,
    EVENT_TERMENTER,
    EVENT_TERMLEAVE,
    EVENT_LSPNOTIFY,
    EVENT_WINNEWPRE,
    EVENT_SOURCECMD,
    EVENT_SOURCEPRE,
    EVENT_VIMRESUME,
    EVENT_BUFUNLOAD,
    EVENT_FOCUSLOST,
    EVENT_MENUPOPUP,
    EVENT_BUFREADCMD,
    EVENT_BUFREADPRE,
    EVENT_DIRCHANGED,
    EVENT_SOURCEPOST,
    EVENT_BUFFILEPRE,
    EVENT_BUFWIPEOUT,
    EVENT_LSPREQUEST,
    EVENT_CURSORHOLD,
    EVENT_VIMRESIZED,
    EVENT_VIMSUSPEND,
    EVENT_WINRESIZED,
    EVENT_BUFNEWFILE,
    EVENT_SWAPEXISTS,
    EVENT_BUFREADPOST,
    EVENT_VIMLEAVEPRE,
    EVENT_FILEREADCMD,
    EVENT_FILEREADPRE,
    EVENT_REMOTEREPLY,
    EVENT_TERMREQUEST,
    EVENT_FOCUSGAINED,
    EVENT_MODECHANGED,
    EVENT_PACKCHANGED,
    EVENT_TERMCHANGED,
    EVENT_TEXTCHANGED,
    EVENT_BUFWRITECMD,
    EVENT_BUFWRITEPRE,
    EVENT_BUFFILEPOST,
    EVENT_BUFWINENTER,
    EVENT_BUFWINLEAVE,
    EVENT_CMDWINENTER,
    EVENT_CMDWINLEAVE,
    EVENT_LSPPROGRESS,
    EVENT_DIFFUPDATED,
    EVENT_CURSORHOLDI,
    EVENT_CURSORMOVED,
    EVENT_WINSCROLLED,
    EVENT_COLORSCHEME,
    EVENT_INSERTENTER,
    EVENT_INSERTLEAVE,
    EVENT_TABCLOSEDPRE,
    EVENT_CMDLINEENTER,
    EVENT_CMDLINELEAVE,
    EVENT_CMDUNDEFINED,
    EVENT_STDINREADPRE,
    EVENT_SHELLCMDPOST,
    EVENT_BUFWRITEPOST,
    EVENT_FILEENCODING,
    EVENT_FILEREADPOST,
    EVENT_FILEWRITECMD,
    EVENT_FILEWRITEPRE,
    EVENT_COMPLETEDONE,
    EVENT_CURSORMOVEDC,
    EVENT_CURSORMOVEDI,
    EVENT_TERMRESPONSE,
    EVENT_INSERTCHANGE,
    EVENT_TEXTCHANGEDI,
    EVENT_TEXTCHANGEDP,
    EVENT_TEXTCHANGEDT,
    EVENT_TEXTYANKPOST,
    EVENT_FILEAPPENDCMD,
    EVENT_FILEAPPENDPRE,
    EVENT_FILECHANGEDRO,
    EVENT_SEARCHWRAPPED,
    EVENT_FILTERREADPRE,
    EVENT_TABNEWENTERED,
    EVENT_DIRCHANGEDPRE,
    EVENT_STDINREADPOST,
    EVENT_INSERTCHARPRE,
    EVENT_FUNCUNDEFINED,
    EVENT_FILEWRITEPOST,
    EVENT_BUFMODIFIEDSET,
    EVENT_CMDLINECHANGED,
    EVENT_COLORSCHEMEPRE,
    EVENT_FILEAPPENDPOST,
    EVENT_FILTERREADPOST,
    EVENT_FILTERWRITEPRE,
    EVENT_INSERTLEAVEPRE,
    EVENT_LSPTOKENUPDATE,
    EVENT_PACKCHANGEDPRE,
    EVENT_QUICKFIXCMDPRE,
    EVENT_RECORDINGENTER,
    EVENT_RECORDINGLEAVE,
    EVENT_SESSIONLOADPRE,
    EVENT_SESSIONLOADPOST,
    EVENT_SHELLFILTERPOST,
    EVENT_FILTERWRITEPOST,
    EVENT_CMDLINELEAVEPRE,
    EVENT_ENCODINGCHANGED,
    EVENT_COMPLETECHANGED,
    EVENT_COMPLETEDONEPRE,
    EVENT_QUICKFIXCMDPOST,
    EVENT_SESSIONWRITEPOST,
    EVENT_FILECHANGEDSHELL,
    EVENT_SPELLFILEMISSING,
    EVENT_DIAGNOSTICCHANGED,
    EVENT_FILECHANGEDSHELLPOST,
]);
unsafe extern "C" fn event_name2nr_hash(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    let mut low: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut high: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    match len {
        4 => {
            low = 0 as ::core::ffi::c_int;
            high = 1 as ::core::ffi::c_int;
        }
        6 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            66 | 98 => {
                low = 1 as ::core::ffi::c_int;
                high = 3 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 3 as ::core::ffi::c_int;
                high = 5 as ::core::ffi::c_int;
            }
            84 | 116 => {
                low = 5 as ::core::ffi::c_int;
                high = 6 as ::core::ffi::c_int;
            }
            87 | 119 => {
                low = 6 as ::core::ffi::c_int;
                high = 7 as ::core::ffi::c_int;
            }
            _ => {}
        },
        7 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            66 | 98 => {
                low = 7 as ::core::ffi::c_int;
                high = 8 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 8 as ::core::ffi::c_int;
                high = 9 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 9 as ::core::ffi::c_int;
                high = 10 as ::core::ffi::c_int;
            }
            81 | 113 => {
                low = 10 as ::core::ffi::c_int;
                high = 11 as ::core::ffi::c_int;
            }
            85 | 117 => {
                low = 11 as ::core::ffi::c_int;
                high = 13 as ::core::ffi::c_int;
            }
            _ => {}
        },
        8 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            66 | 98 => {
                low = 13 as ::core::ffi::c_int;
                high = 16 as ::core::ffi::c_int;
            }
            67 | 99 => {
                low = 16 as ::core::ffi::c_int;
                high = 18 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 18 as ::core::ffi::c_int;
                high = 19 as ::core::ffi::c_int;
            }
            71 | 103 => {
                low = 19 as ::core::ffi::c_int;
                high = 20 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 20 as ::core::ffi::c_int;
                high = 21 as ::core::ffi::c_int;
            }
            84 | 116 => {
                low = 21 as ::core::ffi::c_int;
                high = 24 as ::core::ffi::c_int;
            }
            86 | 118 => {
                low = 24 as ::core::ffi::c_int;
                high = 26 as ::core::ffi::c_int;
            }
            87 | 119 => {
                low = 26 as ::core::ffi::c_int;
                high = 28 as ::core::ffi::c_int;
            }
            _ => {}
        },
        9 => match *str.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            65 | 97 => {
                low = 28 as ::core::ffi::c_int;
                high = 29 as ::core::ffi::c_int;
            }
            67 | 99 => {
                low = 29 as ::core::ffi::c_int;
                high = 32 as ::core::ffi::c_int;
            }
            68 | 100 => {
                low = 32 as ::core::ffi::c_int;
                high = 34 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 34 as ::core::ffi::c_int;
                high = 35 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 35 as ::core::ffi::c_int;
                high = 36 as ::core::ffi::c_int;
            }
            72 | 104 => {
                low = 36 as ::core::ffi::c_int;
                high = 37 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 37 as ::core::ffi::c_int;
                high = 38 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 38 as ::core::ffi::c_int;
                high = 41 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 41 as ::core::ffi::c_int;
                high = 43 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 43 as ::core::ffi::c_int;
                high = 46 as ::core::ffi::c_int;
            }
            85 | 117 => {
                low = 46 as ::core::ffi::c_int;
                high = 49 as ::core::ffi::c_int;
            }
            _ => {}
        },
        10 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            65 | 97 => {
                low = 49 as ::core::ffi::c_int;
                high = 52 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 52 as ::core::ffi::c_int;
                high = 53 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 53 as ::core::ffi::c_int;
                high = 54 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 54 as ::core::ffi::c_int;
                high = 55 as ::core::ffi::c_int;
            }
            81 | 113 => {
                low = 55 as ::core::ffi::c_int;
                high = 56 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 56 as ::core::ffi::c_int;
                high = 57 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 57 as ::core::ffi::c_int;
                high = 60 as ::core::ffi::c_int;
            }
            87 | 119 => {
                low = 60 as ::core::ffi::c_int;
                high = 61 as ::core::ffi::c_int;
            }
            88 | 120 => {
                low = 61 as ::core::ffi::c_int;
                high = 62 as ::core::ffi::c_int;
            }
            _ => {}
        },
        11 => match *str.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            65 | 97 => {
                low = 62 as ::core::ffi::c_int;
                high = 64 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 64 as ::core::ffi::c_int;
                high = 68 as ::core::ffi::c_int;
            }
            71 | 103 => {
                low = 68 as ::core::ffi::c_int;
                high = 69 as ::core::ffi::c_int;
            }
            72 | 104 => {
                low = 69 as ::core::ffi::c_int;
                high = 73 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 73 as ::core::ffi::c_int;
                high = 75 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 75 as ::core::ffi::c_int;
                high = 76 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 76 as ::core::ffi::c_int;
                high = 80 as ::core::ffi::c_int;
            }
            79 | 111 => {
                low = 80 as ::core::ffi::c_int;
                high = 81 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 81 as ::core::ffi::c_int;
                high = 82 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 82 as ::core::ffi::c_int;
                high = 85 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 85 as ::core::ffi::c_int;
                high = 86 as ::core::ffi::c_int;
            }
            84 | 116 => {
                low = 86 as ::core::ffi::c_int;
                high = 88 as ::core::ffi::c_int;
            }
            _ => {}
        },
        12 => match *str.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            66 | 98 => {
                low = 88 as ::core::ffi::c_int;
                high = 89 as ::core::ffi::c_int;
            }
            68 | 100 => {
                low = 89 as ::core::ffi::c_int;
                high = 93 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 93 as ::core::ffi::c_int;
                high = 94 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 94 as ::core::ffi::c_int;
                high = 95 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 95 as ::core::ffi::c_int;
                high = 99 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 99 as ::core::ffi::c_int;
                high = 100 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 100 as ::core::ffi::c_int;
                high = 103 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 103 as ::core::ffi::c_int;
                high = 104 as ::core::ffi::c_int;
            }
            88 | 120 => {
                low = 104 as ::core::ffi::c_int;
                high = 108 as ::core::ffi::c_int;
            }
            _ => {}
        },
        13 => match *str.offset(4 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            65 | 97 => {
                low = 108 as ::core::ffi::c_int;
                high = 110 as ::core::ffi::c_int;
            }
            67 | 99 => {
                low = 110 as ::core::ffi::c_int;
                high = 112 as ::core::ffi::c_int;
            }
            69 | 101 => {
                low = 112 as ::core::ffi::c_int;
                high = 114 as ::core::ffi::c_int;
            }
            72 | 104 => {
                low = 114 as ::core::ffi::c_int;
                high = 115 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 115 as ::core::ffi::c_int;
                high = 116 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 116 as ::core::ffi::c_int;
                high = 117 as ::core::ffi::c_int;
            }
            85 | 117 => {
                low = 117 as ::core::ffi::c_int;
                high = 118 as ::core::ffi::c_int;
            }
            87 | 119 => {
                low = 118 as ::core::ffi::c_int;
                high = 119 as ::core::ffi::c_int;
            }
            _ => {}
        },
        14 => match *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            66 | 98 => {
                low = 119 as ::core::ffi::c_int;
                high = 120 as ::core::ffi::c_int;
            }
            67 | 99 => {
                low = 120 as ::core::ffi::c_int;
                high = 122 as ::core::ffi::c_int;
            }
            70 | 102 => {
                low = 122 as ::core::ffi::c_int;
                high = 125 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 125 as ::core::ffi::c_int;
                high = 126 as ::core::ffi::c_int;
            }
            76 | 108 => {
                low = 126 as ::core::ffi::c_int;
                high = 127 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 127 as ::core::ffi::c_int;
                high = 128 as ::core::ffi::c_int;
            }
            81 | 113 => {
                low = 128 as ::core::ffi::c_int;
                high = 129 as ::core::ffi::c_int;
            }
            82 | 114 => {
                low = 129 as ::core::ffi::c_int;
                high = 131 as ::core::ffi::c_int;
            }
            83 | 115 => {
                low = 131 as ::core::ffi::c_int;
                high = 132 as ::core::ffi::c_int;
            }
            _ => {}
        },
        15 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            69 | 101 => {
                low = 132 as ::core::ffi::c_int;
                high = 133 as ::core::ffi::c_int;
            }
            72 | 104 => {
                low = 133 as ::core::ffi::c_int;
                high = 134 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 134 as ::core::ffi::c_int;
                high = 135 as ::core::ffi::c_int;
            }
            77 | 109 => {
                low = 135 as ::core::ffi::c_int;
                high = 136 as ::core::ffi::c_int;
            }
            78 | 110 => {
                low = 136 as ::core::ffi::c_int;
                high = 137 as ::core::ffi::c_int;
            }
            79 | 111 => {
                low = 137 as ::core::ffi::c_int;
                high = 139 as ::core::ffi::c_int;
            }
            85 | 117 => {
                low = 139 as ::core::ffi::c_int;
                high = 140 as ::core::ffi::c_int;
            }
            _ => {}
        },
        16 => match *str.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            69 | 101 => {
                low = 140 as ::core::ffi::c_int;
                high = 141 as ::core::ffi::c_int;
            }
            73 | 105 => {
                low = 141 as ::core::ffi::c_int;
                high = 142 as ::core::ffi::c_int;
            }
            80 | 112 => {
                low = 142 as ::core::ffi::c_int;
                high = 143 as ::core::ffi::c_int;
            }
            _ => {}
        },
        17 => {
            low = 143 as ::core::ffi::c_int;
            high = 144 as ::core::ffi::c_int;
        }
        20 => {
            low = 144 as ::core::ffi::c_int;
            high = 145 as ::core::ffi::c_int;
        }
        _ => {}
    }
    let mut i: ::core::ffi::c_int = low;
    while i < high {
        if vim_strnicmp_asc(
            str,
            (*event_names.ptr())[(*event_hash.ptr())[i as usize] as usize].name,
            len,
        ) == 0
        {
            return i;
        }
        i += 1;
    }
    return -1 as ::core::ffi::c_int;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
