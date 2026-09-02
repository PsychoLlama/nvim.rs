#![deny(unsafe_op_in_unsafe_fn)]

use crate::types::AutoEvent;
use core::cmp::Ordering;
use core::ffi::CStr;

use crate::api::private::converter::object_to_vim;
use crate::api::private::helpers::{
    api_free_object, api_free_string, copy_object, cstr_as_string, cstr_to_string,
    find_buffer_by_handle,
};
use crate::ascii::ascii_iswhite;
use crate::buffer::{buf_is_prompt, current_buf, do_modelines, find_buf};
use crate::charset::{skipdigits, skipwhite};
use crate::cursor::{check_cursor, check_pos};
use crate::eval::typval::{
    TV_INITIAL_VALUE, callback_copy, callback_free, callback_to_string, tv_clear, tv_dict_add_nr,
    tv_dict_add_tv, tv_dict_set_keys_readonly,
};
use crate::eval::userfunc::{restore_funccal, save_funccal};
use crate::eval::vars::{get_vim_var_nr, get_vim_var_str, set_cmdarg, set_vim_var_nr, vars_clear};
use crate::eval::{callback_call, get_v_event, last_set_msg, restore_v_event};
use crate::event::multiqueue::{multiqueue_new_child, multiqueue_put_event};
use crate::ex_docmd::{do_cmdline, ends_excmd, expand_sfile, get_pressedreturn, set_pressedreturn};
use crate::ex_eval::{aborting, should_abort};
use crate::fileio::{check_timestamps, file_pat_to_reg_pat, match_file_pat};
use crate::getchar::{restore_redobuff, save_redobuff};
use crate::global_cell::GlobalCell;
use crate::hashtab::hash_init;
use crate::highlight_group::{HLF_8, HLF_E, HLF_T};
use crate::insexpand::ins_compl_active;
use crate::lua::executor::nlua_set_sctx;
use crate::main::{
    KeyTyped, aucmd_win_vec, autocmd_bufnr, autocmd_busy, autocmd_fname, autocmd_fname_full,
    autocmd_match, autocmd_no_enter, autocmd_no_leave, curbuf, current_sctx, curtab, curwin,
    deferred_events, did_cursorhold, did_emsg, do_profiling, e_argreq,
    e_cannot_define_autocommands_for_all_events, globaldir, got_int, last_cursormoved,
    last_cursormoved_win, last_mode, main_loop, msg_col, need_maketitle, p_acd, p_ei, p_verbose,
    prevwin, reg_recording, secure, starting,
};
use crate::map::{
    map_del_int_string, map_del_string_int, map_put_ref_int_string, map_put_ref_string_int,
    mh_get_int, mh_get_string,
};
use crate::memory::{xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup};
use crate::message::{
    emsg, give_warning, msg_advance, msg_clr_eos, msg_end, msg_ext_set_kind, msg_outtrans,
    msg_putchar, msg_puts, msg_puts_hl, msg_puts_title, msg_start, verbose_enter,
    verbose_enter_scroll, verbose_leave, verbose_leave_scroll,
};
use crate::option::set_option_direct;
use crate::options::kOptEventignore;
use crate::os::cshim::{gettext, snprintf, strchr, strncasecmp};
use crate::os::env::expand_env_save;
use crate::os::input::line_breakcheck;
use crate::os::time::os_now;
use crate::path::{full_name_save, path_fnamecmp, path_tail};
use crate::profile::{prof_child_enter, prof_child_exit};
use crate::regexp::{RE_MAGIC, vim_regcomp, vim_regfree};
use crate::runtime::{estack_pop, estack_push};
use crate::search::{restore_search_patterns, save_search_patterns};
use crate::state::{MODE_INSERT, MODE_NORMAL_BUSY, get_mode, get_real_state};
use crate::strings::{vim_strchr, xstrnsave};
use crate::types::builders::{ArrayBuf, DictBuf};
use crate::types::{
    AutoCmd, AutoCmdVec, AutoPat, AutoPatCmd, AutoPatCmd_S, Buffer, Callback, Error, Event,
    Integer, LuaRetMode, Map_String_int, Map_int_String, MapHash, Object, OptVal, Set_String,
    Set_int, String_0, Timestamp, Vv, aco_save_T, aucmdwin_T, buf_T, etype_T, exarg_T, expand_T,
    funccal_entry_T, int64_t, proftime_T, save_redo_T, save_v_event_T, sctx_T, size_t, uint32_t,
    uint64_t, varnumber_T, win_T,
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
use crate::winlayer::{forget_window, free_deferred, register_window};
use ::libc::{abort, atoi, strcasecmp, strcpy};

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
pub struct AutoCmdEvent {
    pub event: AutoEvent,
    pub fname: *mut ::core::ffi::c_char,
    pub fname_io: *mut ::core::ffi::c_char,
    pub buf: Buffer,
    pub group: ::core::ffi::c_int,
    pub eap: *mut exarg_T,
    pub data: *mut Object,
}
pub const AUGROUP_DELETED: ::core::ffi::c_int = -4;
pub const AUGROUP_ALL: ::core::ffi::c_int = -3;
pub const AUGROUP_ERROR: ::core::ffi::c_int = -2;
pub const AUGROUP_DEFAULT: ::core::ffi::c_int = -1;
pub const BUFLOCAL_PAT_LEN: ::core::ffi::c_uint = 25;
pub const ETYPE_AUCMD: etype_T = 3;
pub const kRetNilBool: LuaRetMode = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
static value_init_int: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static value_init_String: GlobalCell<String_0> = GlobalCell::new(String_0::NULL);
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
unsafe fn map_put_string_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    // The two out-parameters `map_put_ref` offers -- the key it took
    // ownership of, and whether the entry is new -- are both unwanted here.
    let no_key_out = ::core::ptr::null_mut::<*mut String_0>();
    let no_new_item_out = ::core::ptr::null_mut::<bool>();
    // SAFETY: `map` points at a live `Map_String_int` and `key` is a string
    // it may take; null out-parameters are how they are declined.
    let val = unsafe { map_put_ref_string_int(map, key, no_key_out, no_new_item_out) };
    // SAFETY: `val` is the value slot the map just handed back, which stays
    // valid until the map is next changed -- and nothing changes it here.
    unsafe { *val = value };
}
#[inline]
unsafe fn map_get_string_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    // SAFETY: `map` points at a live `Map_String_int`, so `&raw mut` on its
    // `set` field is in bounds; `key` is only borrowed for the lookup.
    let k: uint32_t = unsafe { mh_get_string(&raw mut (*map).set, key) };
    if k == MH_TOMBSTONE as uint32_t {
        value_init_int.get()
    } else {
        // SAFETY: not a tombstone, so `k` is the index into `values` that
        // the lookup just answered.
        unsafe { *(*map).values.offset(k as isize) }
    }
}
#[inline]
unsafe fn map_put_int_string(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
    mut value: String_0,
) {
    // Both of `map_put_ref`'s out-parameters are unwanted here; see
    // [`map_put_string_int`].
    let no_key_out = ::core::ptr::null_mut::<*mut ::core::ffi::c_int>();
    let no_new_item_out = ::core::ptr::null_mut::<bool>();
    // SAFETY: `map` points at a live `Map_int_String`; null out-parameters
    // are how they are declined.
    let val = unsafe { map_put_ref_int_string(map, key, no_key_out, no_new_item_out) };
    // SAFETY: `val` is the value slot the map just handed back, which stays
    // valid until the map is next changed -- and nothing changes it here.
    unsafe { *val = value };
}
#[inline]
unsafe fn map_get_int_string(
    mut map: *mut Map_int_String,
    mut key: ::core::ffi::c_int,
) -> String_0 {
    // SAFETY: `map` points at a live `Map_int_String`, so `&raw mut` on its
    // `set` field is in bounds.
    let k: uint32_t = unsafe { mh_get_int(&raw mut (*map).set, key) };
    if k == MH_TOMBSTONE as uint32_t {
        value_init_String.get()
    } else {
        // SAFETY: not a tombstone, so `k` is the index into `values` that
        // the lookup just answered.
        unsafe { *(*map).values.offset(k as isize) }
    }
}
/// `e_autocommand_nesting_too_deep`.  A `GlobalCell` holding a transmuted
/// byte array upstream, because c2rust had no `CStr`; nothing writes it.
const E_AUTOCOMMAND_NESTING_TOO_DEEP: &CStr = c"E218: Autocommand nesting too deep";
static active_apc_list: GlobalCell<*mut AutoPatCmd> =
    GlobalCell::new(::core::ptr::null_mut::<AutoPatCmd>());
static next_augroup_id: GlobalCell<::core::ffi::c_int> = GlobalCell::new(1 as ::core::ffi::c_int);
static deleted_augroup: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null::<::core::ffi::c_char>());
static current_augroup: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(AUGROUP_DEFAULT as ::core::ffi::c_int);
static au_need_clean: GlobalCell<bool> = GlobalCell::new(false);
static autocmd_blocked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static autocmd_nested: GlobalCell<bool> = GlobalCell::new(false);
static autocmd_include_groups: GlobalCell<bool> = GlobalCell::new(false);
static termresponse_changed: GlobalCell<bool> = GlobalCell::new(false);
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
/// Safe: it takes nothing, and the event loop it hangs the deferred-event
/// queue off is live from startup to exit.
pub fn autocmd_init() {
    // SAFETY: `main_loop` is initialised before this is reached, so reading
    // its `events` queue and giving it a child are both on a live loop.
    let child = unsafe { multiqueue_new_child((*main_loop.ptr()).events) };
    deferred_events.set(child);
}
/// Where the `VimSuspend`/`VimResume` pair has got to.
///
/// Three states, and the middle one is not "unknown": it is "an autocommand
/// for this pair is running right now", which is what keeps the pair from
/// firing twice for one suspension.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum SuspendLatch {
    /// Nothing owed.
    Idle,
    /// One of the two events is being fired at this moment.
    Firing,
    /// A `VimResume` is owed for a suspension that already fired.
    ResumeOwed,
}

static pending_vimresume: GlobalCell<SuspendLatch> = GlobalCell::new(SuspendLatch::Idle);
pub const NO_SCREEN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_NONE: ::core::ffi::c_int = -6 as ::core::ffi::c_int;
/// One row of [`EVENT_NAMES`]: a spelling of an event, and which event it
/// is.  There is one row -- and one [`AutoEvent`] variant, at the same
/// index -- per *name*, so the four aliases have a variant of their own
/// that no event ever takes: row `AutoEvent::BufCreate` is `BufCreate`,
/// whose `event` is `AutoEvent::BufAdd`.  `event_nr2name` reads the row at
/// the index, the name lookup answers the row's `event`, and the two agree
/// everywhere a real event value can reach.
///
/// `win_local` is upstream's window-local flag, which it packed into the
/// *sign* of the event number -- `gen_events.lua` negates the event of
/// every name whose `auevents.lua` entry is `true`, and it was read back as
/// `event <= 0`, which made `BufAdd`, event 0, window-local by having no
/// sign at all.  A flag in a field of its own needs no such coincidence.
pub struct EventName {
    pub name: &'static CStr,
    pub event: AutoEvent,
    /// May this name be listed in 'eventignorewin' as well as
    /// 'eventignore'?
    pub win_local: bool,
}

/// A row of [`EVENT_NAMES`] for an event that is not window-local.
const fn named(name: &'static CStr, event: AutoEvent) -> EventName {
    EventName {
        name,
        event,
        win_local: false,
    }
}

/// [`named`] for an event that is window-local.
const fn named_win_local(name: &'static CStr, event: AutoEvent) -> EventName {
    EventName {
        name,
        event,
        win_local: true,
    }
}

/// Every autocommand event's name, sorted by lower-cased name -- the order
/// `gen_events.lua` numbers the `AutoEvent` enum in, so a row's index *is*
/// its event number, and the order the name lookup binary searches.
pub static EVENT_NAMES: [EventName; 145] = [
    named_win_local(c"BufAdd", AutoEvent::BufAdd),
    named_win_local(c"BufCreate", AutoEvent::BufAdd),
    named_win_local(c"BufDelete", AutoEvent::BufDelete),
    named_win_local(c"BufEnter", AutoEvent::BufEnter),
    named_win_local(c"BufFilePost", AutoEvent::BufFilePost),
    named_win_local(c"BufFilePre", AutoEvent::BufFilePre),
    named_win_local(c"BufHidden", AutoEvent::BufHidden),
    named_win_local(c"BufLeave", AutoEvent::BufLeave),
    named_win_local(c"BufModifiedSet", AutoEvent::BufModifiedSet),
    named_win_local(c"BufNew", AutoEvent::BufNew),
    named_win_local(c"BufNewFile", AutoEvent::BufNewFile),
    named_win_local(c"BufRead", AutoEvent::BufReadPost),
    named_win_local(c"BufReadCmd", AutoEvent::BufReadCmd),
    named_win_local(c"BufReadPost", AutoEvent::BufReadPost),
    named_win_local(c"BufReadPre", AutoEvent::BufReadPre),
    named_win_local(c"BufUnload", AutoEvent::BufUnload),
    named_win_local(c"BufWinEnter", AutoEvent::BufWinEnter),
    named_win_local(c"BufWinLeave", AutoEvent::BufWinLeave),
    named_win_local(c"BufWipeout", AutoEvent::BufWipeout),
    named_win_local(c"BufWrite", AutoEvent::BufWritePre),
    named_win_local(c"BufWriteCmd", AutoEvent::BufWriteCmd),
    named_win_local(c"BufWritePost", AutoEvent::BufWritePost),
    named_win_local(c"BufWritePre", AutoEvent::BufWritePre),
    named(c"ChanInfo", AutoEvent::ChanInfo),
    named(c"ChanOpen", AutoEvent::ChanOpen),
    named(c"CmdlineChanged", AutoEvent::CmdlineChanged),
    named(c"CmdlineEnter", AutoEvent::CmdlineEnter),
    named(c"CmdlineLeave", AutoEvent::CmdlineLeave),
    named(c"CmdlineLeavePre", AutoEvent::CmdlineLeavePre),
    named(c"CmdUndefined", AutoEvent::CmdUndefined),
    named(c"CmdwinEnter", AutoEvent::CmdwinEnter),
    named(c"CmdwinLeave", AutoEvent::CmdwinLeave),
    named(c"ColorScheme", AutoEvent::ColorScheme),
    named(c"ColorSchemePre", AutoEvent::ColorSchemePre),
    named(c"CompleteChanged", AutoEvent::CompleteChanged),
    named(c"CompleteDone", AutoEvent::CompleteDone),
    named(c"CompleteDonePre", AutoEvent::CompleteDonePre),
    named_win_local(c"CursorHold", AutoEvent::CursorHold),
    named_win_local(c"CursorHoldI", AutoEvent::CursorHoldI),
    named_win_local(c"CursorMoved", AutoEvent::CursorMoved),
    named_win_local(c"CursorMovedC", AutoEvent::CursorMovedC),
    named_win_local(c"CursorMovedI", AutoEvent::CursorMovedI),
    named(c"DiagnosticChanged", AutoEvent::DiagnosticChanged),
    named(c"DiffUpdated", AutoEvent::DiffUpdated),
    named(c"DirChanged", AutoEvent::DirChanged),
    named(c"DirChangedPre", AutoEvent::DirChangedPre),
    named(c"EncodingChanged", AutoEvent::EncodingChanged),
    named(c"ExitPre", AutoEvent::ExitPre),
    named_win_local(c"FileAppendCmd", AutoEvent::FileAppendCmd),
    named_win_local(c"FileAppendPost", AutoEvent::FileAppendPost),
    named_win_local(c"FileAppendPre", AutoEvent::FileAppendPre),
    named_win_local(c"FileChangedRO", AutoEvent::FileChangedRO),
    named_win_local(c"FileChangedShell", AutoEvent::FileChangedShell),
    named_win_local(c"FileChangedShellPost", AutoEvent::FileChangedShellPost),
    named(c"FileEncoding", AutoEvent::EncodingChanged),
    named_win_local(c"FileReadCmd", AutoEvent::FileReadCmd),
    named_win_local(c"FileReadPost", AutoEvent::FileReadPost),
    named_win_local(c"FileReadPre", AutoEvent::FileReadPre),
    named_win_local(c"FileType", AutoEvent::FileType),
    named_win_local(c"FileWriteCmd", AutoEvent::FileWriteCmd),
    named_win_local(c"FileWritePost", AutoEvent::FileWritePost),
    named_win_local(c"FileWritePre", AutoEvent::FileWritePre),
    named_win_local(c"FilterReadPost", AutoEvent::FilterReadPost),
    named_win_local(c"FilterReadPre", AutoEvent::FilterReadPre),
    named_win_local(c"FilterWritePost", AutoEvent::FilterWritePost),
    named_win_local(c"FilterWritePre", AutoEvent::FilterWritePre),
    named(c"FocusGained", AutoEvent::FocusGained),
    named(c"FocusLost", AutoEvent::FocusLost),
    named(c"FuncUndefined", AutoEvent::FuncUndefined),
    named(c"GUIEnter", AutoEvent::GUIEnter),
    named(c"GUIFailed", AutoEvent::GUIFailed),
    named_win_local(c"InsertChange", AutoEvent::InsertChange),
    named_win_local(c"InsertCharPre", AutoEvent::InsertCharPre),
    named_win_local(c"InsertEnter", AutoEvent::InsertEnter),
    named_win_local(c"InsertLeave", AutoEvent::InsertLeave),
    named_win_local(c"InsertLeavePre", AutoEvent::InsertLeavePre),
    named(c"LspAttach", AutoEvent::LspAttach),
    named(c"LspDetach", AutoEvent::LspDetach),
    named(c"LspNotify", AutoEvent::LspNotify),
    named(c"LspProgress", AutoEvent::LspProgress),
    named(c"LspRequest", AutoEvent::LspRequest),
    named(c"LspTokenUpdate", AutoEvent::LspTokenUpdate),
    named(c"MarkSet", AutoEvent::MarkSet),
    named(c"MenuPopup", AutoEvent::MenuPopup),
    named(c"ModeChanged", AutoEvent::ModeChanged),
    named(c"OptionSet", AutoEvent::OptionSet),
    named(c"PackChanged", AutoEvent::PackChanged),
    named(c"PackChangedPre", AutoEvent::PackChangedPre),
    named(c"Progress", AutoEvent::Progress),
    named(c"QuickFixCmdPost", AutoEvent::QuickFixCmdPost),
    named(c"QuickFixCmdPre", AutoEvent::QuickFixCmdPre),
    named(c"QuitPre", AutoEvent::QuitPre),
    named_win_local(c"RecordingEnter", AutoEvent::RecordingEnter),
    named_win_local(c"RecordingLeave", AutoEvent::RecordingLeave),
    named(c"RemoteReply", AutoEvent::RemoteReply),
    named(c"SafeState", AutoEvent::SafeState),
    named_win_local(c"SearchWrapped", AutoEvent::SearchWrapped),
    named(c"SessionLoadPost", AutoEvent::SessionLoadPost),
    named(c"SessionLoadPre", AutoEvent::SessionLoadPre),
    named(c"SessionWritePost", AutoEvent::SessionWritePost),
    named(c"ShellCmdPost", AutoEvent::ShellCmdPost),
    named_win_local(c"ShellFilterPost", AutoEvent::ShellFilterPost),
    named(c"Signal", AutoEvent::Signal),
    named(c"SourceCmd", AutoEvent::SourceCmd),
    named(c"SourcePost", AutoEvent::SourcePost),
    named(c"SourcePre", AutoEvent::SourcePre),
    named(c"SpellFileMissing", AutoEvent::SpellFileMissing),
    named(c"StdinReadPost", AutoEvent::StdinReadPost),
    named(c"StdinReadPre", AutoEvent::StdinReadPre),
    named(c"SwapExists", AutoEvent::SwapExists),
    named(c"Syntax", AutoEvent::Syntax),
    named(c"TabClosed", AutoEvent::TabClosed),
    named(c"TabClosedPre", AutoEvent::TabClosedPre),
    named(c"TabEnter", AutoEvent::TabEnter),
    named(c"TabLeave", AutoEvent::TabLeave),
    named(c"TabNew", AutoEvent::TabNew),
    named(c"TabNewEntered", AutoEvent::TabNewEntered),
    named(c"TermChanged", AutoEvent::TermChanged),
    named(c"TermClose", AutoEvent::TermClose),
    named(c"TermEnter", AutoEvent::TermEnter),
    named(c"TermLeave", AutoEvent::TermLeave),
    named(c"TermOpen", AutoEvent::TermOpen),
    named(c"TermRequest", AutoEvent::TermRequest),
    named(c"TermResponse", AutoEvent::TermResponse),
    named_win_local(c"TextChanged", AutoEvent::TextChanged),
    named_win_local(c"TextChangedI", AutoEvent::TextChangedI),
    named_win_local(c"TextChangedP", AutoEvent::TextChangedP),
    named_win_local(c"TextChangedT", AutoEvent::TextChangedT),
    named_win_local(c"TextYankPost", AutoEvent::TextYankPost),
    named(c"UIEnter", AutoEvent::UIEnter),
    named(c"UILeave", AutoEvent::UILeave),
    named(c"User", AutoEvent::User),
    named(c"VimEnter", AutoEvent::VimEnter),
    named(c"VimLeave", AutoEvent::VimLeave),
    named(c"VimLeavePre", AutoEvent::VimLeavePre),
    named(c"VimResized", AutoEvent::VimResized),
    named(c"VimResume", AutoEvent::VimResume),
    named(c"VimSuspend", AutoEvent::VimSuspend),
    named_win_local(c"WinClosed", AutoEvent::WinClosed),
    named_win_local(c"WinEnter", AutoEvent::WinEnter),
    named_win_local(c"WinLeave", AutoEvent::WinLeave),
    named(c"WinNew", AutoEvent::WinNew),
    named(c"WinNewPre", AutoEvent::WinNewPre),
    named_win_local(c"WinResized", AutoEvent::WinResized),
    named_win_local(c"WinScrolled", AutoEvent::WinScrolled),
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
pub(super) fn au_event_vec(event: AutoEvent) -> *mut AutoCmdVec {
    autocmds
        .ptr()
        .cast::<AutoCmdVec>()
        .wrapping_add(event.index())
}
