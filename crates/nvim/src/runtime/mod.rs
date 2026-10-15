//! The runtime: where the editor looks for the files it is made of, and what
//! it remembers about the ones it has run.
//!
//! Nine children, one per question asked of it — see each one's own docs:
//! [`estack`] (where am I executing), [`search`] (where does a file along
//! 'runtimepath' live), [`cache`] (the precomputed form of that search),
//! [`pack`] (what a package adds), [`expand`] (what completion offers),
//! [`rtp`] (how the default 'runtimepath' is built), [`source`] (running a
//! script), [`reader`] (reading one) and [`script`] (the registry of the ones
//! already run).
//!
//! This file keeps what they share: the flag constants, the struct layouts,
//! and the family's process-wide state. That last part is the reason the
//! parent holds no functions of its own —
//!
//! | cell | written by | read by |
//! | ---- | ---------- | ------- |
//! | `EXESTACK` | [`estack`]'s push/pop, and the provider's saved frame | `estack`, the garbage collector's trim, and every error message's prefix |
//! | `SCRIPT_ITEMS` | [`script`]'s `new_script_item` | [`script`]'s `with_script_item` and the readers beside it, which ~20 files outside the family call |
//! | `LOADED_AUTOLOADS` | [`script`]'s `script_autoload`, once per autoload file | [`script`] — nothing else touches it |
//! | `LAST_SCRIPT_SEQ` | [`source`] | [`source`] |
//! | `RUNTIME_SEARCH_PATH` | [`cache`], and [`pack`]'s splice | [`cache`], via [`search`]'s `do_in_runtimepath_visit` |
//! | `SEARCH_PATH_VALID` | [`cache`]'s invalidate/validate pair, [`pack`] | [`cache`], [`pack`] |
//! | `THREAD_SEARCH_PATH` | [`cache`], for the off-main-loop reader | [`search`] — **a `Mutex`, see below** |
//! | `THREAD_SEARCH_PATH_VALID` | [`cache`], [`pack`] | [`cache`] |
//! | `RUNTIME_EXPAND_FLAGS` | [`search`]'s `set_context_in_runtime_cmd` | [`expand`]'s `runtime_cmd_matches` |
#![forbid(unsafe_code)]
// The constants here keep upstream's spelling (`kXDG*`, `kRet*`).
#![allow(non_upper_case_globals)]

pub(crate) mod state;
use crate::autocmd::has_autocmd;
use crate::cmdexpand::globpath;
use crate::debugger::state::{debug_break_level, debug_tick};
use crate::debugger::{dbg_breakpoint, dbg_find_breakpoint_named, profiling_forced};
use crate::eval::typval::{
    dict_copy, dict_find, dict_get_string_alloc, tv_check_for_opt_dict_arg, tv_dict_alloc,
    tv_dict_alloc_lock, tv_get_number_chk, tv_list_alloc, tv_list_alloc_ret,
};
use crate::eval::userfunc::{CallStackAside, all_funcs};
use crate::eval::vars::new_script_vars;
use crate::eval::{eval_to_number, get_copy_id};
use crate::ex_docmd::state::{cmdmod, ex_nesting_level, global_busy, listcmd_busy};
use crate::ex_docmd::{do_cmdline, do_cmdline_cmd, do_exedit};
use crate::ex_eval::{
    PendingAction, PendingValue, aborting, cleanup_conditionals, cond_stack_of, report_pending,
};
use crate::getchar::openscript;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::keycodes::Ctrl_V;
use crate::lua::executor::{nlua_exec_file, nlua_exec_lines, nlua_is_deferred_safe};
use crate::mbyte::cluster_len;
use crate::memory::XString;
use crate::message::state::msg_col;
use crate::message::{e_argreq, e_interr, e_invarg, e_norange};
use crate::message::{
    emsg, message_filtered, msg_display, msg_ext_set_kind, msg_ext_ui_flush, msg_putchar,
    verbose_enter, verbose_leave,
};
use crate::option::vars::{p_enc, p_ic, p_lpl, p_verbose};
use crate::option::{set_option_value_give_err, vimrc_found};
use crate::options::kOptRuntimepath;
use crate::os::cshim::gettext;
use crate::os::env::vim_get_prefix_from_exepath;
use crate::os::fs::os_file_is_readable;
use crate::os::input::line_breakcheck;
use crate::os::stdpaths::get_appname;
use crate::path::{
    append_path, concat_fnames, fix_fname, gen_expand_wildcards, past_head_index, path_fnamecmp,
    path_fnamencmp, path_with_extension, tail_index, vim_ispathsep, vim_ispathsep_nocolon,
};
use crate::profile::{do_profiling, startup_timing};
use crate::profile::{
    prof_child_enter, prof_child_exit, profile_add, profile_end, profile_init, profile_self,
    profile_start, profile_sub_wait, profile_zero, script_line_end, script_line_start, time_msg,
    time_msg_at, time_pop, time_push,
};
use crate::regexp::{RE_MAGIC, RE_STRING};
use crate::registry::{IdMap, IdSet, id_map, id_set};
use crate::runtime::state::{current_sctx, did_source_packages};
use crate::types::AutoEvent;
use crate::types::{
    ApiDict, Array, CONV_NONE, EStack, EStackArg, EStackType, EstackInfo, EvalFuncData, ExArg,
    Expand, FrameName, Integer, LineNr, LuaRetMode, Object, OptSet, OptVal, ProfTime, ScriptCtx,
    ScriptId, ScriptItem, String_0, TypVal, UserFunc, VAR_DICT, VarLock, VarNumber, VimConv,
    XDGVarType, int64_t, kBoolVarFalse, ptrdiff_t,
};
use core::cell::Cell;
use std::rc::Rc;
use std::sync::Mutex;

// The carve of the transpiled module; see each child's docs.
mod cache;
mod estack;
mod expand;
mod pack;
mod reader;
mod rtp;
mod script;
mod search;
mod source;

pub(crate) use self::cache::*;
pub use self::estack::*;
pub(crate) use self::expand::*;
pub use self::pack::*;
pub(crate) use self::reader::*;
pub use self::rtp::*;
pub use self::script::*;
pub use self::search::*;
pub use self::source::*;

/// The room upstream's `MAXPATHL` buffers had, terminator included.
const PATH_ROOM: usize = crate::types::MAXPATHL as usize;
/// `:finish` as a pending control-flow reason, for `report_pending`.
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
pub const ETYPE_AUCMD: EStackType = 3;
pub const ETYPE_UFUNC: EStackType = 2;
pub const ETYPE_SCRIPT: EStackType = 1;
pub const ETYPE_TOP: EStackType = 0;
pub const ESTACK_SCRIPT: EStackArg = 3;
pub const ESTACK_STACK: EStackArg = 2;
pub const ESTACK_SFILE: EStackArg = 1;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
/// `gen_expand_wildcards` flags: what a wildcard expansion may return.
/// `do_source`'s `is_vimrc`: whether the file being sourced is the vimrc.
pub const DOSO_VIMRC: ::core::ffi::c_int = 1;
pub const DOSO_NONE: ::core::ffi::c_int = 0;
crate::flag_set! {
    /// How a runtime-file search should behave -- upstream's `DIP_*`, the
    /// `flags` argument [`do_in_path_visit`] and everything under it thread:
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
/// One directory of the cached search path -- see [`cache`].
#[derive(Clone)]
pub struct SearchPathItem {
    pub path: XString,
    /// Whether it is an `after/` directory.
    pub after: bool,
    /// Whether `:packadd` spliced it in rather than a rebuild finding it.
    pub pack_inserted: bool,
    /// Whether it has a `lua/` subdirectory, once something asked.
    pub has_lua: Cell<Option<bool>>,
    /// The byte offset in 'runtimepath' of the entry that put it here.
    pub pos_in_rtp: usize,
}

impl SearchPathItem {
    /// A directory a rebuild found.
    pub(crate) fn new(path: XString, after: bool, pos_in_rtp: usize) -> Self {
        SearchPathItem {
            path,
            after,
            pack_inserted: false,
            has_lua: Cell::new(None),
            pos_in_rtp,
        }
    }
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const ARRAY_DICT_INIT: Array = Array::EMPTY;
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
/// A `Vec`, not a `GArray`: nothing outside this crate reads it, the
/// element type is fixed, and every walk over it is then a checked one.
/// Reach it through [`GlobalCell::with`]/[`GlobalCell::with_mut`], which also
/// catch a push made while a walk holds a borrow.
pub(crate) static EXESTACK: GlobalCell<Vec<EStack>> = GlobalCell::new(Vec::new());
/// The script registry, script 1 at index 0 -- see [`script`].
///
/// The items are the vector's own: ids are never reused so it only ever
/// grows, and an item is reached by its id, borrowed for one call at a time
/// through [`script::with_script_item`] -- or copied out by
/// [`script::script_name`] -- never held.
pub(crate) static SCRIPT_ITEMS: GlobalCell<Vec<ScriptItem>> = GlobalCell::new(Vec::new());
/// Every autoload script `script_autoload` has already run, by path.
///
/// A `Vec`, not a `GArray` of owned `char *`: the list is private to
/// [`script`], is only ever appended to and scanned, and never freed.
static LOADED_AUTOLOADS: GlobalCell<Vec<Vec<u8>>> = GlobalCell::new(Vec::new());
static LAST_SCRIPT_SEQ: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
/// Whether [`RUNTIME_SEARCH_PATH`] still describes 'runtimepath' and
/// 'packpath'.
static SEARCH_PATH_VALID: GlobalCell<bool> = GlobalCell::new(false);
/// Whether [`THREAD_SEARCH_PATH`] is a copy of the valid cache.
static THREAD_SEARCH_PATH_VALID: GlobalCell<bool> = GlobalCell::new(false);
/// The cached search path; a walk holds a clone of the [`Rc`] for as long as
/// it reads, which is what lets a rebuild run underneath it.
static RUNTIME_SEARCH_PATH: GlobalCell<Option<Rc<Vec<SearchPathItem>>>> = GlobalCell::new(None);
/// The snapshot the worker threads read.
///
/// A `Mutex` rather than a [`GlobalCell`] because `runtime_get_named_thread`
/// really does run off the main thread — it is `vim._get_runtime` as a
/// `vim.uv.new_thread` state sees it, and every `require()` in such a thread
/// reaches it.
static THREAD_SEARCH_PATH: Mutex<Vec<SearchPathItem>> = Mutex::new(Vec::new());
static RUNTIME_EXPAND_FLAGS: GlobalCell<RuntimeOpts> = GlobalCell::new(RuntimeOpts::NONE);
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
