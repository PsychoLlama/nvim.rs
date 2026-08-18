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
    api_clear_error, arena_array, arena_dict, arena_string, array_add, cstr_as_string,
    cstr_to_string, dict_put,
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
    tv_dict_copy, tv_dict_find, tv_dict_get_string, tv_get_number_chk, tv_get_string, tv_ht_iter,
    tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict, tv_list_append_string,
    tv_list_append_tv, tv_list_set_ret,
};
use crate::eval::userfunc::{func_tbl_get, restore_funccal, save_funccal};
use crate::eval::vars::new_script_vars;
use crate::eval::{eval_to_number, get_copyID};
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
    GA_EMPTY_INIT_VALUE, IObuff, NameBuff, cmdmod, curbuf, current_sctx, debug_break_level,
    debug_tick, did_source_packages, do_profiling, e_argreq, e_dirnotf, e_interr, e_invarg,
    e_invargNval, e_norange, e_notopen, ex_nesting_level, global_busy, got_int, listcmd_busy,
    msg_col, p_cpo, p_enc, p_ic, p_lpl, p_pp, p_rtp, p_verbose, time_fd,
};
use crate::map::{map_put_ref_String_int, map_ref_String_int, mh_get_String, mh_put_String};
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
use crate::os::cshim::{gettext, memmove, snprintf, strncasecmp, strncmp, strstr};
use crate::os::env::{
    default_lib_dir, expand_env, expand_env_save, home_replace, home_replace_save, os_setenv,
    vim_get_prefix_from_exepath, vim_getenv,
};
use crate::os::fs::{os_file_is_readable, os_isdir, os_open, os_set_cloexec};
use crate::os::input::line_breakcheck;
use crate::os::stdpaths::{get_appname, stdpaths_get_xdg_var};
use crate::path::{
    FreeWild, add_pathsep, append_path, concat_fnames, fix_fname, gen_expand_wildcards,
    get_past_head, path_fnamecmp, path_fnamencmp, path_tail, path_with_extension, vim_ispathsep,
    vim_ispathsep_nocolon,
};
use crate::profile::{
    prof_child_enter, prof_child_exit, profile_add, profile_end, profile_init, profile_self,
    profile_start, profile_sub_wait, profile_zero, script_line_end, script_line_start, time_msg,
    time_pop, time_push,
};
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec, vim_regfree};
use crate::strings::{vim_snprintf, vim_snprintf_safelen, vim_strchr};
use crate::types::{
    Arena, Array, BoolVarValue, CONV_NONE, Dict, DoInRuntimepathCB, DoInRuntimepathCBFn, Error,
    EvalFuncData, FILE, Integer, LineGetter, LineGetterFn, LuaRetMode, MHPutStatus, Map_String_int,
    MapHash, Object, ObjectType, OptVal, OptValData, OptValType, Set_String, String_0, TriState,
    UV_MUTEX_INIT, VAR_DICT, VAR_FIXED, VAR_LOCKED, XDGVarType, cmd_addr_T, dict_T, estack_T,
    estack_T_es_info, estack_arg_T, etype_T, exarg_T, expand_T, funccal_entry_T, garray_T,
    handle_T, int64_t, kBoolVarFalse, kErrorTypeNone, kFalse, kNone, kObjectTypeBoolean,
    kObjectTypeDict, kObjectTypeInteger, kObjectTypeString, kTrue, linenr_T, list_T, object_data,
    optset_T, proftime_T, ptrdiff_t, regmatch_T, scid_T, scriptitem_T, sctx_T, size_t, ssize_t,
    typval_T, typval_vval_union, ufunc_T, uint8_t, uint32_t, uv_mutex_t, varnumber_T, vimconv_T,
};
use crate::usercmd::add_win_cmd_modifiers;
use crate::{semsg_c, smsg_c};
use ::libc::{__errno_location, fclose, fdopen, fgets, strcasecmp, strcat, strcmp, strcpy, strlen};

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
/// `xp_context`: the completion this command line wants.
pub const EXPAND_RUNTIME: ::core::ffi::c_int = 51;
pub const kOptValTypeString: OptValType = 2;
/// `:finish` as a pending control-flow reason, for `report_make_pending`.
pub const CSTP_FINISH: ::core::ffi::c_int = 32;
pub const ADDR_LINES: cmd_addr_T = 0;
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
pub const DOCMD_REPEAT: ::core::ffi::c_int = 4;
pub const DOCMD_NOWAIT: ::core::ffi::c_int = 2;
pub const DOCMD_VERBOSE: ::core::ffi::c_int = 1;
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
pub const DIP_DIRFILE: ::core::ffi::c_uint = 512;
/// `do_in_path` flags: which runtime directories to visit and what to match.
pub const DIP_AFTER: ::core::ffi::c_uint = 128;
pub const DIP_NOAFTER: ::core::ffi::c_uint = 64;
pub const DIP_NORTP: ::core::ffi::c_uint = 32;
pub const DIP_OPT: ::core::ffi::c_uint = 16;
pub const DIP_START: ::core::ffi::c_uint = 8;
pub const DIP_ERR: ::core::ffi::c_uint = 4;
pub const DIP_DIR: ::core::ffi::c_uint = 2;
pub const DIP_ALL: ::core::ffi::c_uint = 1;
#[derive(Copy, Clone)]
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
    pub has_lua: TriState,
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
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
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
unsafe fn set_has_String(mut set: *mut Set_String, mut key: String_0) -> bool {
    unsafe {
        return mh_get_String(set, key) != MH_TOMBSTONE as uint32_t;
    }
}
#[inline]
unsafe fn set_put_String(
    mut set: *mut Set_String,
    mut key: String_0,
    mut key_alloc: *mut *mut String_0,
) -> bool {
    unsafe {
        let mut status: MHPutStatus = kMHExisting;
        let mut k: uint32_t = mh_put_String(set, key, &raw mut status);
        if !key_alloc.is_null() {
            *key_alloc = (*set).keys.offset(k as isize);
        }
        return status as ::core::ffi::c_uint
            != kMHExisting as ::core::ffi::c_int as ::core::ffi::c_uint;
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
        // The absent value is `value_init_int`, a `static int` upstream never
        // writes: zero.
        let mut k: uint32_t = mh_get_String(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            0
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const SYS_OPTWIN_FILE: &::core::ffi::CStr = c"$VIMRUNTIME/scripts/optwin.lua";
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_CONCAT: ::core::ffi::c_int = 'C' as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
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
pub static exestack: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<estack_T>() as ::core::ffi::c_int,
    ga_growsize: 50 as ::core::ffi::c_int,
    ga_data: NULL_0,
});
pub static script_items: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<*mut scriptitem_T>() as ::core::ffi::c_int,
    ga_growsize: 20 as ::core::ffi::c_int,
    ga_data: NULL_0,
});
static ga_loaded: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL_0,
});
static last_current_SID_seq: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);
static runtime_search_path_valid: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static runtime_search_path_valid_thread: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
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
static runtime_expand_flags: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: &::core::ffi::CStr = c"rb";
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
