#![deny(unsafe_op_in_unsafe_fn)]

use core::cmp::Ordering;
use core::ffi::CStr;
use core::slice;

use crate::api::private::converter::object_to_vim;
use crate::api::private::helpers::{
    api_clear_error, api_free_object, api_free_string, copy_object, cstr_as_string, cstr_to_string,
    find_buffer_by_handle,
};
use crate::ascii::ascii_iswhite;
use crate::buffer::{bt_prompt, buflist_findnr, bufref_valid, do_modelines, set_bufref};
use crate::charset::{skipdigits, skipwhite};
use crate::cursor::{check_cursor, check_pos};
use crate::eval::typval::{
    TV_INITIAL_VALUE, callback_copy, callback_free, callback_to_string, kCallbackLua,
    kCallbackNone, tv_clear, tv_dict_add_nr, tv_dict_add_tv, tv_dict_set_keys_readonly,
};
use crate::eval::userfunc::{restore_funccal, save_funccal};
use crate::eval::vars::{get_vim_var_nr, get_vim_var_str, set_cmdarg, set_vim_var_nr, vars_clear};
use crate::eval::{callback_call, get_v_event, last_set_msg, restore_v_event};
use crate::event::multiqueue::{multiqueue_new_child, multiqueue_put_event};
use crate::ex_docmd::{do_cmdline, ends_excmd, expand_sfile, get_pressedreturn, set_pressedreturn};
use crate::ex_eval::{aborting, should_abort};
use crate::fileio::{check_timestamps, file_pat_to_reg_pat, match_file_pat};
use crate::getchar::{restoreRedobuff, saveRedobuff};
use crate::global_cell::GlobalCell;
use crate::grid::grid_free;
use crate::hashtab::hash_init;
use crate::highlight_group::{HLF_8, HLF_E, HLF_T};
use crate::insexpand::ins_compl_active;
use crate::lua::executor::{nlua_call_ref, nlua_set_sctx};
use crate::main::{
    KeyTyped, RedrawingDisabled, VIsual, VIsual_active, au_pending_free_buf, au_pending_free_win,
    aucmd_win_vec, autocmd_bufnr, autocmd_busy, autocmd_fname, autocmd_fname_full, autocmd_match,
    autocmd_no_enter, autocmd_no_leave, curbuf, current_sctx, curtab, curwin, deferred_events,
    did_cursorhold, did_emsg, do_profiling, e_argreq, e_cannot_define_autocommands_for_all_events,
    e_duparg2, first_tabpage, firstbuf, firstwin, globaldir, got_int, last_cursormoved,
    last_cursormoved_win, last_mode, lastwin, main_loop, msg_col, need_maketitle, p_acd, p_ei,
    p_verbose, prevwin, reg_recording, secure, starting, typebuf, window_handles,
};
use crate::map::{
    map_del_String_int, map_del_int_String, map_del_int_ptr_t, map_put_ref_String_int,
    map_put_ref_int_String, map_put_ref_int_ptr_t, mh_get_String, mh_get_int,
};
use crate::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup};
use crate::message::{
    emsg, give_warning, msg_advance, msg_clr_eos, msg_end, msg_ext_set_kind, msg_outtrans,
    msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, msg_start, verbose_enter,
    verbose_enter_scroll, verbose_leave, verbose_leave_scroll,
};
use crate::option::set_option_direct;
use crate::options::kOptEventignore;
use crate::os::cshim::{gettext, snprintf, strchr, strncasecmp, strncmp};
use crate::os::env::expand_env_save;
use crate::os::input::line_breakcheck;
use crate::os::time::os_now;
use crate::path::{FullName_save, path_fnamecmp, path_tail};
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::runtime::{estack_pop, estack_push, exestack};
use crate::search::{restore_search_patterns, save_search_patterns};
use crate::state::{MODE_INSERT, MODE_NORMAL_BUSY, get_mode, get_real_state};
use crate::strings::{vim_strchr, xstrnsave};
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::{
    AutoCmd, AutoCmdVec, AutoPat, AutoPatCmd, AutoPatCmd_S, Buffer, Callback,
    Callback_data as C2Rust_Unnamed_5, Error, Event, Integer, LuaRetMode, Map_String_int,
    Map_int_String, Map_int_ptr_t, MapHash, Object, OptVal, OptValData, OptValType, Set_String,
    Set_int, String_0, Timestamp, TriState, VV_CMDBANG, VV_TERMRESPONSE, aco_save_T, aucmdwin_T,
    auto_event, buf_T, buffblock_T, buffheader_T, bufref_T, estack_T, etype_T, event_T, exarg_T,
    expand_T, funccal_entry_T, int64_t, kErrorTypeNone, kFalse, kNone, kObjectTypeBoolean,
    kObjectTypeDict, kTrue, proftime_T, ptr_t, save_redo_T, save_v_event_T, sctx_T, size_t,
    uint32_t, uint64_t, varnumber_T, win_T,
};
use crate::ui::ui_call_win_hide;
use crate::ui_compositor::ui_comp_remove_grid;
use crate::window::{
    check_lnums, check_lnums_nested, close_tabpage, entering_window, goto_tabpage_tp, reset_lnums,
    snapshot_windows_scroll_size, unuse_tabpage, use_tabpage, valid_tabpage_win,
    win_alloc_aucmd_win, win_append, win_enter, win_find_by_handle, win_fix_current_dir, win_goto,
    win_init_empty, win_remove,
};
use crate::winfloat::win_config_float;
use ::libc::{abort, atoi, strcasecmp, strcpy, strlen};

// The carve of the transpiled module; see each child's docs.
mod aucmdwin;
mod cleanup;
mod define;
mod events;
mod fire;
mod groups;
mod listing;
mod trigger;
mod walk;

pub use self::aucmdwin::*;
pub use self::cleanup::*;
pub use self::define::*;
pub use self::events::*;
pub use self::fire::*;
pub use self::groups::*;
pub use self::listing::*;
pub use self::trigger::*;
pub use self::walk::*;
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
unsafe fn map_put_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
    mut value: ptr_t,
) {
    unsafe {
        let mut val: *mut ptr_t = map_put_ref_int_ptr_t(
            map,
            key,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}
#[inline]
unsafe fn map_put_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    unsafe {
        let mut val: *mut ::core::ffi::c_int = map_put_ref_String_int(
            map,
            key,
            ::core::ptr::null_mut::<*mut String_0>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}
#[inline]
unsafe fn map_get_String_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    unsafe {
        let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_int.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
#[inline]
unsafe fn map_put_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
    mut value: String_0,
) {
    unsafe {
        let mut val: *mut String_0 = map_put_ref_int_String(
            map,
            key,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
        *val = value;
    }
}
#[inline]
unsafe fn map_get_int_String(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
) -> String_0 {
    unsafe {
        let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_String.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
/// `e_autocommand_nesting_too_deep`.  A `GlobalCell` holding a transmuted
/// byte array upstream, because c2rust has no `CStr`; nothing writes it.
const E_AUTOCOMMAND_NESTING_TOO_DEEP: &CStr = c"E218: Autocommand nesting too deep";
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
pub unsafe fn autocmd_init() {
    unsafe {
        deferred_events.set(multiqueue_new_child((*main_loop.ptr()).events));
    }
}
static pending_vimresume: GlobalCell<TriState> = GlobalCell::new(kFalse);
pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
/// One row of [`EVENT_NAMES`]: a spelling of an event, and which event it
/// is.  There is one row -- and one `EVENT_*` constant, at the same index
/// -- per *name*, so the four aliases have a constant of their own that no
/// event ever takes: row `EVENT_BUFCREATE` is `BufCreate`, whose `event` is
/// `EVENT_BUFADD`.  `event_nr2name` reads the row at the index, the name
/// lookup answers the row's `event`, and the two agree everywhere a real
/// event value can reach.
///
/// The sign of `event` is upstream's window-local flag -- `gen_events.lua`
/// negates the event of every name whose `auevents.lua` entry is `true` --
/// and it is read back as `event <= 0`, which is why `BufAdd`, event 0,
/// counts as window-local by having no sign at all.
pub struct EventName {
    pub name: &'static CStr,
    pub event: ::core::ffi::c_int,
}

/// A row of [`EVENT_NAMES`] for an event that is not window-local.
const fn named(name: &'static CStr, event: auto_event) -> EventName {
    EventName {
        name,
        event: event as ::core::ffi::c_int,
    }
}

/// [`named`] for an event that is window-local, so it may be listed in
/// 'eventignorewin' as well as 'eventignore'.
const fn named_win_local(name: &'static CStr, event: auto_event) -> EventName {
    EventName {
        name,
        event: -(event as ::core::ffi::c_int),
    }
}

/// Every autocommand event's name, sorted by lower-cased name -- the order
/// `gen_events.lua` numbers the `auto_event` enum in, so a row's index *is*
/// its event number, and the order the name lookup binary searches.
pub static EVENT_NAMES: [EventName; 145] = [
    named_win_local(c"BufAdd", EVENT_BUFADD),
    named_win_local(c"BufCreate", EVENT_BUFADD),
    named_win_local(c"BufDelete", EVENT_BUFDELETE),
    named_win_local(c"BufEnter", EVENT_BUFENTER),
    named_win_local(c"BufFilePost", EVENT_BUFFILEPOST),
    named_win_local(c"BufFilePre", EVENT_BUFFILEPRE),
    named_win_local(c"BufHidden", EVENT_BUFHIDDEN),
    named_win_local(c"BufLeave", EVENT_BUFLEAVE),
    named_win_local(c"BufModifiedSet", EVENT_BUFMODIFIEDSET),
    named_win_local(c"BufNew", EVENT_BUFNEW),
    named_win_local(c"BufNewFile", EVENT_BUFNEWFILE),
    named_win_local(c"BufRead", EVENT_BUFREADPOST),
    named_win_local(c"BufReadCmd", EVENT_BUFREADCMD),
    named_win_local(c"BufReadPost", EVENT_BUFREADPOST),
    named_win_local(c"BufReadPre", EVENT_BUFREADPRE),
    named_win_local(c"BufUnload", EVENT_BUFUNLOAD),
    named_win_local(c"BufWinEnter", EVENT_BUFWINENTER),
    named_win_local(c"BufWinLeave", EVENT_BUFWINLEAVE),
    named_win_local(c"BufWipeout", EVENT_BUFWIPEOUT),
    named_win_local(c"BufWrite", EVENT_BUFWRITEPRE),
    named_win_local(c"BufWriteCmd", EVENT_BUFWRITECMD),
    named_win_local(c"BufWritePost", EVENT_BUFWRITEPOST),
    named_win_local(c"BufWritePre", EVENT_BUFWRITEPRE),
    named(c"ChanInfo", EVENT_CHANINFO),
    named(c"ChanOpen", EVENT_CHANOPEN),
    named(c"CmdlineChanged", EVENT_CMDLINECHANGED),
    named(c"CmdlineEnter", EVENT_CMDLINEENTER),
    named(c"CmdlineLeave", EVENT_CMDLINELEAVE),
    named(c"CmdlineLeavePre", EVENT_CMDLINELEAVEPRE),
    named(c"CmdUndefined", EVENT_CMDUNDEFINED),
    named(c"CmdwinEnter", EVENT_CMDWINENTER),
    named(c"CmdwinLeave", EVENT_CMDWINLEAVE),
    named(c"ColorScheme", EVENT_COLORSCHEME),
    named(c"ColorSchemePre", EVENT_COLORSCHEMEPRE),
    named(c"CompleteChanged", EVENT_COMPLETECHANGED),
    named(c"CompleteDone", EVENT_COMPLETEDONE),
    named(c"CompleteDonePre", EVENT_COMPLETEDONEPRE),
    named_win_local(c"CursorHold", EVENT_CURSORHOLD),
    named_win_local(c"CursorHoldI", EVENT_CURSORHOLDI),
    named_win_local(c"CursorMoved", EVENT_CURSORMOVED),
    named_win_local(c"CursorMovedC", EVENT_CURSORMOVEDC),
    named_win_local(c"CursorMovedI", EVENT_CURSORMOVEDI),
    named(c"DiagnosticChanged", EVENT_DIAGNOSTICCHANGED),
    named(c"DiffUpdated", EVENT_DIFFUPDATED),
    named(c"DirChanged", EVENT_DIRCHANGED),
    named(c"DirChangedPre", EVENT_DIRCHANGEDPRE),
    named(c"EncodingChanged", EVENT_ENCODINGCHANGED),
    named(c"ExitPre", EVENT_EXITPRE),
    named_win_local(c"FileAppendCmd", EVENT_FILEAPPENDCMD),
    named_win_local(c"FileAppendPost", EVENT_FILEAPPENDPOST),
    named_win_local(c"FileAppendPre", EVENT_FILEAPPENDPRE),
    named_win_local(c"FileChangedRO", EVENT_FILECHANGEDRO),
    named_win_local(c"FileChangedShell", EVENT_FILECHANGEDSHELL),
    named_win_local(c"FileChangedShellPost", EVENT_FILECHANGEDSHELLPOST),
    named(c"FileEncoding", EVENT_ENCODINGCHANGED),
    named_win_local(c"FileReadCmd", EVENT_FILEREADCMD),
    named_win_local(c"FileReadPost", EVENT_FILEREADPOST),
    named_win_local(c"FileReadPre", EVENT_FILEREADPRE),
    named_win_local(c"FileType", EVENT_FILETYPE),
    named_win_local(c"FileWriteCmd", EVENT_FILEWRITECMD),
    named_win_local(c"FileWritePost", EVENT_FILEWRITEPOST),
    named_win_local(c"FileWritePre", EVENT_FILEWRITEPRE),
    named_win_local(c"FilterReadPost", EVENT_FILTERREADPOST),
    named_win_local(c"FilterReadPre", EVENT_FILTERREADPRE),
    named_win_local(c"FilterWritePost", EVENT_FILTERWRITEPOST),
    named_win_local(c"FilterWritePre", EVENT_FILTERWRITEPRE),
    named(c"FocusGained", EVENT_FOCUSGAINED),
    named(c"FocusLost", EVENT_FOCUSLOST),
    named(c"FuncUndefined", EVENT_FUNCUNDEFINED),
    named(c"GUIEnter", EVENT_GUIENTER),
    named(c"GUIFailed", EVENT_GUIFAILED),
    named_win_local(c"InsertChange", EVENT_INSERTCHANGE),
    named_win_local(c"InsertCharPre", EVENT_INSERTCHARPRE),
    named_win_local(c"InsertEnter", EVENT_INSERTENTER),
    named_win_local(c"InsertLeave", EVENT_INSERTLEAVE),
    named_win_local(c"InsertLeavePre", EVENT_INSERTLEAVEPRE),
    named(c"LspAttach", EVENT_LSPATTACH),
    named(c"LspDetach", EVENT_LSPDETACH),
    named(c"LspNotify", EVENT_LSPNOTIFY),
    named(c"LspProgress", EVENT_LSPPROGRESS),
    named(c"LspRequest", EVENT_LSPREQUEST),
    named(c"LspTokenUpdate", EVENT_LSPTOKENUPDATE),
    named(c"MarkSet", EVENT_MARKSET),
    named(c"MenuPopup", EVENT_MENUPOPUP),
    named(c"ModeChanged", EVENT_MODECHANGED),
    named(c"OptionSet", EVENT_OPTIONSET),
    named(c"PackChanged", EVENT_PACKCHANGED),
    named(c"PackChangedPre", EVENT_PACKCHANGEDPRE),
    named(c"Progress", EVENT_PROGRESS),
    named(c"QuickFixCmdPost", EVENT_QUICKFIXCMDPOST),
    named(c"QuickFixCmdPre", EVENT_QUICKFIXCMDPRE),
    named(c"QuitPre", EVENT_QUITPRE),
    named_win_local(c"RecordingEnter", EVENT_RECORDINGENTER),
    named_win_local(c"RecordingLeave", EVENT_RECORDINGLEAVE),
    named(c"RemoteReply", EVENT_REMOTEREPLY),
    named(c"SafeState", EVENT_SAFESTATE),
    named_win_local(c"SearchWrapped", EVENT_SEARCHWRAPPED),
    named(c"SessionLoadPost", EVENT_SESSIONLOADPOST),
    named(c"SessionLoadPre", EVENT_SESSIONLOADPRE),
    named(c"SessionWritePost", EVENT_SESSIONWRITEPOST),
    named(c"ShellCmdPost", EVENT_SHELLCMDPOST),
    named_win_local(c"ShellFilterPost", EVENT_SHELLFILTERPOST),
    named(c"Signal", EVENT_SIGNAL),
    named(c"SourceCmd", EVENT_SOURCECMD),
    named(c"SourcePost", EVENT_SOURCEPOST),
    named(c"SourcePre", EVENT_SOURCEPRE),
    named(c"SpellFileMissing", EVENT_SPELLFILEMISSING),
    named(c"StdinReadPost", EVENT_STDINREADPOST),
    named(c"StdinReadPre", EVENT_STDINREADPRE),
    named(c"SwapExists", EVENT_SWAPEXISTS),
    named(c"Syntax", EVENT_SYNTAX),
    named(c"TabClosed", EVENT_TABCLOSED),
    named(c"TabClosedPre", EVENT_TABCLOSEDPRE),
    named(c"TabEnter", EVENT_TABENTER),
    named(c"TabLeave", EVENT_TABLEAVE),
    named(c"TabNew", EVENT_TABNEW),
    named(c"TabNewEntered", EVENT_TABNEWENTERED),
    named(c"TermChanged", EVENT_TERMCHANGED),
    named(c"TermClose", EVENT_TERMCLOSE),
    named(c"TermEnter", EVENT_TERMENTER),
    named(c"TermLeave", EVENT_TERMLEAVE),
    named(c"TermOpen", EVENT_TERMOPEN),
    named(c"TermRequest", EVENT_TERMREQUEST),
    named(c"TermResponse", EVENT_TERMRESPONSE),
    named_win_local(c"TextChanged", EVENT_TEXTCHANGED),
    named_win_local(c"TextChangedI", EVENT_TEXTCHANGEDI),
    named_win_local(c"TextChangedP", EVENT_TEXTCHANGEDP),
    named_win_local(c"TextChangedT", EVENT_TEXTCHANGEDT),
    named_win_local(c"TextYankPost", EVENT_TEXTYANKPOST),
    named(c"UIEnter", EVENT_UIENTER),
    named(c"UILeave", EVENT_UILEAVE),
    named(c"User", EVENT_USER),
    named(c"VimEnter", EVENT_VIMENTER),
    named(c"VimLeave", EVENT_VIMLEAVE),
    named(c"VimLeavePre", EVENT_VIMLEAVEPRE),
    named(c"VimResized", EVENT_VIMRESIZED),
    named(c"VimResume", EVENT_VIMRESUME),
    named(c"VimSuspend", EVENT_VIMSUSPEND),
    named_win_local(c"WinClosed", EVENT_WINCLOSED),
    named_win_local(c"WinEnter", EVENT_WINENTER),
    named_win_local(c"WinLeave", EVENT_WINLEAVE),
    named(c"WinNew", EVENT_WINNEW),
    named(c"WinNewPre", EVENT_WINNEWPRE),
    named_win_local(c"WinResized", EVENT_WINRESIZED),
    named_win_local(c"WinScrolled", EVENT_WINSCROLLED),
];
/// The state every event's autocommand list starts in: an empty `kvec`
/// with nothing allocated.
const AUTOCMDVEC_INIT: AutoCmdVec = AutoCmdVec {
    size: 0 as size_t,
    capacity: 0,
    items: ::core::ptr::null_mut::<AutoCmd>(),
};

/// The autocommands defined for each event, indexed by event number.
static autocmds: GlobalCell<[AutoCmdVec; 145]> = GlobalCell::new([AUTOCMDVEC_INIT; 145]);

/// The autocommand list of `event`.
///
/// It stays a raw pointer on purpose.  A handler running under the walk
/// can define or delete autocommands, which reallocates the `items` block
/// and marks rows deleted underneath it, so nothing may hold a borrow of
/// one of these across a call into user code.  `wrapping_add` keeps the
/// whole table's provenance and needs no `unsafe`; only reading through
/// the result does.
fn au_event_vec(event: event_T) -> *mut AutoCmdVec {
    autocmds
        .ptr()
        .cast::<AutoCmdVec>()
        .wrapping_add(event as usize)
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
