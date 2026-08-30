//! The runtime: where the editor looks for the files it is made of, and what
//! it remembers about the ones it has run.
//!
//! Eight children, one per question asked of it — see each one's own docs:
//! [`estack`] (where am I executing), [`search`] (where does a file along
//! 'runtimepath' live), [`cache`] (the precomputed form of that search),
//! [`pack`] (what a package adds), [`expand`] (what completion offers),
//! [`rtp`] (how the default 'runtimepath' is built), [`source`] (running a
//! script) and [`script`] (the registry of the ones already run).
//!
//! This file keeps what they share: the flag constants, the struct layouts,
//! and the family's process-wide state. That last part is the reason the
//! parent holds no functions of its own —
//!
//! | cell | written by | read by |
//! | ---- | ---------- | ------- |
//! | `exestack` | [`estack`]'s push/pop | `estack`, `script`, and every error message's prefix |
//! | `script_items` | [`source`] as it registers a script | [`script`], [`source`], and ~20 files outside the family |
//! | `ga_loaded` | [`script`]'s `script_autoload`, once per autoload file | [`script`] — nothing else touches it |
//! | `last_current_SID_seq` | [`source`] | [`source`] |
//! | `runtime_search_path` | [`cache`] | [`cache`], via [`search`]'s `do_in_runtimepath` |
//! | `runtime_search_path_valid` | [`cache`]'s invalidate/validate pair | [`cache`] |
//! | `runtime_search_path_thread` + `_mutex` | [`cache`], for the off-main-loop reader | [`cache`], [`search`] — **`SharedCell`, see below** |
//! | `runtime_search_path_valid_thread` | [`cache`], [`pack`] | [`cache`] |
//! | `runtime_expand_flags` | [`search`]'s `set_context_in_runtime_cmd` | [`expand`]'s `expand_runtime_cmd` |
#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::{
    arena_array, arena_dict, arena_string, array_add, cstr_as_string, cstr_to_string, dict_put,
};
use crate::autocmd::{
    EVENT_SOURCECMD, EVENT_SOURCEPOST, EVENT_SOURCEPRE, apply_autocmds, has_autocmd,
};
use crate::charset::{skip_to_newline, skiptowhite, skiptowhite_esc, skipwhite, skipwhite_len};
use crate::cmdexpand::globpath;
use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint, has_profiling};
use crate::eval::typval::{
    tv_check_for_opt_dict_arg, tv_dict_add_bool, tv_dict_add_dict, tv_dict_add_func,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_alloc_lock,
    tv_dict_copy, tv_dict_find, tv_dict_get_string_alloc, tv_get_number_chk, tv_ht_iter,
    tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict, tv_list_append_string,
    tv_list_append_tv, tv_list_set_ret,
};
use crate::eval::userfunc::{func_tbl_get, restore_funccal, save_funccal};
use crate::eval::vars::new_script_vars;
use crate::eval::{eval_to_number, get_copy_id};
use crate::event::libuv::{uv_mutex_init, uv_mutex_lock, uv_mutex_unlock};
use crate::ex_docmd::{do_cmdline, do_cmdline_cmd, do_exedit, getline_cookie, getline_equal};
use crate::ex_eval::{aborting, cleanup_conditionals, report_make_pending};
use crate::garray::{
    ga_append, ga_clear_strings, ga_concat, ga_concat_len, ga_grow, ga_init,
    ga_remove_duplicate_strings, ga_set_growsize,
};
use crate::getchar::openscript;
use crate::global_cell::{GlobalCell, SharedCell};
use crate::keycodes::Ctrl_V;
use crate::lua::executor::{nlua_exec, nlua_exec_file, nlua_exec_ga, nlua_is_deferred_safe};
use crate::main::{
    GA_EMPTY_INIT_VALUE, cmdmod, curbuf, current_sctx, debug_break_level, debug_tick,
    did_source_packages, do_profiling, e_argreq, e_interr, e_invarg, e_norange, ex_nesting_level,
    global_busy, got_int, listcmd_busy, msg_col, p_enc, p_ic, p_lpl, p_pp, p_rtp, p_verbose,
    time_fd,
};
use crate::map::{map_put_ref_string_int, map_ref_string_int, mh_get_string, mh_put_string};
use crate::mbyte::{convert_setup, enc_canonize, string_convert, utf_head_off, utfc_ptr2len};
use crate::memline::ml_get;
use crate::memory::{
    strequal, try_malloc, xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup, xstrlcat,
    xstrlcpy,
};
use crate::message::{
    emsg, message_filtered, msg_ext_set_kind, msg_ext_ui_flush, msg_outtrans, msg_putchar,
    verbose_enter, verbose_leave,
};
use crate::option::{copy_option_part, set_option_value_give_err, vimrc_found};
use crate::options::kOptRuntimepath;
use crate::os::cshim::{gettext, snprintf, strncasecmp, strstr};
use crate::os::env::{
    default_lib_dir, expand_env, expand_env_save, home_replace, home_replace_save, os_setenv,
    vim_get_prefix_from_exepath, vim_getenv,
};
use crate::os::fs::{os_file_is_readable, os_isdir, os_open, os_set_cloexec};
use crate::os::input::line_breakcheck;
use crate::os::stdpaths::{get_appname, stdpaths_get_xdg_var};
use crate::path::{
    add_pathsep, append_path, concat_fnames, fix_fname, free_wild, gen_expand_wildcards,
    get_past_head, path_fnamecmp, path_fnamencmp, path_tail, path_with_extension, vim_ispathsep,
    vim_ispathsep_nocolon,
};
use crate::profile::{
    prof_child_enter, prof_child_exit, profile_add, profile_end, profile_init, profile_self,
    profile_start, profile_sub_wait, profile_zero, script_line_end, script_line_start, time_msg,
    time_pop, time_push,
};
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec, vim_regfree};
use crate::strings::{vim_snprintf, vim_snprintf_safelen};
use crate::types::{
    Arena, Array, BoolVarValue, CONV_NONE, Dict, DoInRuntimepathCB, DoInRuntimepathCBFn, Error,
    EvalFuncData, FILE, Integer, LineGetter, LineGetterFn, LuaRetMode, MHPutStatus, Map_String_int,
    MapHash, Object, ObjectType, OptVal, OptValData, OptValType, Set_String, String_0,
    UV_MUTEX_INIT, VAR_DICT, VarLock, XDGVarType, dict_T, estack_T, estack_T_es_info, estack_arg_T,
    etype_T, exarg_T, expand_T, funccal_entry_T, garray_T, handle_T, int64_t, kBoolVarFalse,
    kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger, kObjectTypeString, linenr_T, list_T,
    object_data, optset_T, proftime_T, ptrdiff_t, regmatch_T, scid_T, scriptitem_T, sctx_T, size_t,
    ssize_t, typval_T, typval_vval_union, ufunc_T, uint8_t, uint32_t, uv_mutex_t, varnumber_T,
    vimconv_T,
};
use crate::usercmd::add_win_cmd_modifiers;
use ::libc::{__errno_location, fclose, fdopen, fgets, strcasecmp, strcat, strcpy};

// The carve of the transpiled module; see each child's docs.
mod cache;
mod estack;
mod expand;
mod pack;
mod rtp;
mod script;
mod search;
mod source;

pub use self::cache::*;
pub use self::estack::*;
pub use self::expand::*;
pub use self::pack::*;
pub use self::rtp::*;
pub use self::script::*;
pub use self::search::*;
pub use self::source::*;

pub const kMHExisting: MHPutStatus = 0;
pub const kOptValTypeString: OptValType = 2;
/// `:finish` as a pending control-flow reason, for `report_make_pending`.
pub const CSTP_FINISH: ::core::ffi::c_int = 32;
/// `globpath` flags.
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const kXDGNone: XDGVarType = -1;
/// `do_cmdline` flags.
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
/// `gen_expand_wildcards` flags: what a wildcard expansion may return.
/// `do_source`'s `is_vimrc`: whether the file being sourced is the vimrc.
pub const DOSO_VIMRC: ::core::ffi::c_int = 1;
pub const DOSO_NONE: ::core::ffi::c_int = 0;
crate::flag_set! {
    /// How a runtime-file search should behave -- upstream's `DIP_*`, the
    /// `flags` argument [`do_in_path`] and everything under it thread:
    /// which directories to visit, what to match there, and what a miss
    /// means.
    pub struct RuntimeOpts;

    /// Visit every match rather than stopping at the first.
    const ALL = 1;
    /// Look for directories, not files.
    const DIR = 2;
    /// Finding nothing is an error the user hears about.
    const ERR = 4;
    /// Also search `{packpath}/pack/*/start/*`.
    const START = 8;
    /// Also search `{packpath}/pack/*/opt/*`.
    const OPT = 16;
    /// Do not search `'runtimepath'` itself -- only the package trees
    /// [`Self::START`] and [`Self::OPT`] name.
    const NORTP = 32;
    /// Visit only the entries that are *not* under an `after/` directory.
    const NOAFTER = 64;
    /// Visit only the entries that *are* under an `after/` directory.
    const AFTER = 128;
    /// Look for both directories and files.
    const DIRFILE = 512;
}
#[derive(Clone)]
pub struct source_cookie_T {
    pub fp: *mut FILE,
    pub nextline: *mut ::core::ffi::c_char,
    pub sourcing_lnum: linenr_T,
    pub finished: bool,
    pub source_from_buf_or_str: bool,
    pub buf_lnum: ::core::ffi::c_int,
    pub buflines: garray_T,
    pub breakpoint: linenr_T,
    pub fname: *mut ::core::ffi::c_char,
    pub dbg_tick: ::core::ffi::c_int,
    pub level: ::core::ffi::c_int,
    pub conv: vimconv_T,
}
#[derive(Copy, Clone)]
pub struct RuntimeSearchPath {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut SearchPathItem,
}
#[derive(Copy, Clone)]
pub struct SearchPathItem {
    pub path: *mut ::core::ffi::c_char,
    pub after: bool,
    pub pack_inserted: bool,
    pub has_lua: Option<bool>,
    pub pos_in_rtp: size_t,
}
#[derive(Copy, Clone)]
pub struct CharVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
pub struct StringVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut String_0,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_String = Set_String {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<String_0>(),
};
pub const MAP_INIT: Map_String_int = Map_String_int {
    set: SET_INIT,
    values: ::core::ptr::null_mut::<::core::ffi::c_int>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe fn set_has_string(mut set: *mut Set_String, mut key: String_0) -> bool {
    unsafe { mh_get_string(set, key) != MH_TOMBSTONE as uint32_t }
}
#[inline]
unsafe fn set_put_string(
    mut set: *mut Set_String,
    mut key: String_0,
    mut key_alloc: *mut *mut String_0,
) -> bool {
    let mut status: MHPutStatus = kMHExisting;
    let mut k: uint32_t = unsafe { mh_put_string(set, key, &raw mut status) };
    if !key_alloc.is_null() {
        unsafe { *key_alloc = (*set).keys.offset(k as isize) };
    }
    status as ::core::ffi::c_uint != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint
}
#[inline]
unsafe fn map_put_string_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
    mut value: ::core::ffi::c_int,
) {
    let mut val: *mut ::core::ffi::c_int = unsafe {
        map_put_ref_string_int(
            map,
            key,
            ::core::ptr::null_mut::<*mut String_0>(),
            ::core::ptr::null_mut::<bool>(),
        )
    };
    unsafe { *val = value };
}
#[inline]
unsafe fn map_get_string_int(
    mut map: *mut Map_String_int,
    mut key: String_0,
) -> ::core::ffi::c_int {
    // The absent value is `value_init_int`, a `static int` upstream never
    // writes: zero.
    let mut k: uint32_t = unsafe { mh_get_string(&raw mut (*map).set, key) };
    if k == MH_TOMBSTONE as uint32_t {
        0
    } else {
        unsafe { *(*map).values.offset(k as isize) }
    }
}
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const SYS_OPTWIN_FILE: &::core::ffi::CStr = c"$VIMRUNTIME/scripts/optwin.lua";
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const SID_MODELINE: ::core::ffi::c_int = -1;
pub const SID_CMDARG: ::core::ffi::c_int = -2;
pub const SID_CARG: ::core::ffi::c_int = -3;
pub const SID_ENV: ::core::ffi::c_int = -4;
pub const SID_ERROR: ::core::ffi::c_int = -5;
pub const SID_WINLAYOUT: ::core::ffi::c_int = -7;
pub const SID_LUA: ::core::ffi::c_int = -8;
pub const SID_API_CLIENT: ::core::ffi::c_int = -9;
pub const SID_STR: ::core::ffi::c_int = -10;
/// The execution stack, outermost frame first -- see [`estack`].
///
/// A `Vec`, not a `garray_T`: nothing outside this crate reads it, the
/// element type is fixed, and every walk over it is then a checked one.
/// Reach it through [`GlobalCell::with`]/[`GlobalCell::with_mut`], which also
/// catch a push made while a walk holds a borrow.
pub static exestack: GlobalCell<Vec<estack_T>> = GlobalCell::new(Vec::new());
/// The script registry, script 1 at index 0 -- see [`script`].
///
/// A `Vec`, not a `garray_T`: the element type is fixed, ids are never
/// reused so the vector only ever grows, and [`script::script_item`] can then
/// be a *safe* bounds-checked lookup instead of an offset off `ga_data`.
/// Reach it through [`script::script_item`], [`script::script_count`] and
/// [`script::script_id_valid`], never directly.
pub static script_items: GlobalCell<Vec<*mut scriptitem_T>> = GlobalCell::new(Vec::new());
/// Every autoload script `script_autoload` has already run, by path.
///
/// A `Vec`, not a `garray_T` of owned `char *`: the list is private to
/// [`script`], is only ever appended to and scanned, and never freed.
static ga_loaded: GlobalCell<Vec<Vec<u8>>> = GlobalCell::new(Vec::new());
static last_current_SID_seq: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static runtime_search_path_valid: GlobalCell<bool> = GlobalCell::new(false);
static runtime_search_path_valid_thread: GlobalCell<bool> = GlobalCell::new(false);
static runtime_search_path_ref: GlobalCell<*mut ::core::ffi::c_int> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_int>());
static runtime_search_path: GlobalCell<RuntimeSearchPath> = GlobalCell::new(RuntimeSearchPath {
    size: 0,
    capacity: 0,
    items: ::core::ptr::null_mut::<SearchPathItem>(),
});
/// The snapshot the worker threads read, and the mutex that guards it.
///
/// These two are [`SharedCell`]s rather than [`GlobalCell`]s because
/// `runtime_get_named_thread` really does run off the main thread — it is
/// `vim._get_runtime` as a `vim.uv.new_thread` state sees it, and every
/// `require()` in such a thread reaches it. `GlobalCell`'s debug main-thread
/// assertion would abort there; the mutex is the coordination, exactly as it
/// is in C.
static runtime_search_path_thread: SharedCell<RuntimeSearchPath> =
    SharedCell::new(RuntimeSearchPath {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<SearchPathItem>(),
    });
static runtime_search_path_mutex: SharedCell<uv_mutex_t> = SharedCell::new(UV_MUTEX_INIT);
static runtime_expand_flags: GlobalCell<RuntimeOpts> = GlobalCell::new(RuntimeOpts::NONE);
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
