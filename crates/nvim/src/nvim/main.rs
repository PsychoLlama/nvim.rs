use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
use crate::src::nvim::options::{
    kOptArabic, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptErrorfile, kOptKeymap, kOptRightleft,
    kOptShadafile, kOptShortmess, kOptVerbosefile, kOptWindow,
};
use crate::src::nvim::profile::time_msg;
use crate::src::nvim::types::{
    __pthread_internal_list, __pthread_list_t, __pthread_mutex_s, __pthread_rwlock_arch_t,
    AdditionalData, Array, Callback, Callback_data, DecorState, FILE, GridView, ListLenSpecials,
    Loop, LuaRef, LuaRetMode, MTNode, MTPos, Map_String_int, Map_int_ptr_t, Map_uint64_t_ptr_t,
    MapHash, MarkTreeIter, MarkTreeIter_s, MultiQueue, NS, Object, OptInt, OptValType, Proc,
    RgbValue, ScreenGrid, Set_String, Set_int, Set_uint32_t, Set_uint64_t, StlClickDefinition,
    String_0, TriState, VarLockStatus, VarType, VimVarIndex, WinExtmark, XDGVarType, alist_T,
    aucmdwin_T, bln_values, buf_T, bufref_T, caller_scope, cmdmod_T, colnr_T, disptick_T, estack_T,
    estack_T_es_info, etype_T, evalarg_T, except_T, file_comparison, fmark_T, fmarkv_T, frame_T,
    garray_T, handle_T, hlf_T, int16_t, int32_t, int64_t, kFalse, kNone, key_extra, linenr_T,
    lpos_T, match_T, msglist_T, nlua_ref_state_t, nvim_stats_s, optmagic_T, pos_T, proftime_T,
    pthread_mutex_t, pthread_rwlock_t, ptr_t, reg_extmatch_T, regmatch_T, regmmatch_T, regprog_T,
    sattr_T, schar_T, scid_T, sctx_T, size_t, tabpage_T, typebuf_T, uint8_t, uint32_t, uint64_t,
    uv__io_t, uv__queue, uv_async_s_u, uv_async_t, uv_handle_t, uv_handle_type,
    uv_loop_s_active_reqs, uv_loop_s_timer_heap, uv_loop_t, uv_signal_s, uv_signal_s_tree_entry,
    uv_signal_s_u, uv_signal_t, uv_timer_s_node, uv_timer_s_u, uv_timer_t, vimmenu_T, win_T,
    xfmark_T,
};
use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

mod entry;
pub use self::entry::*;
mod args;
mod buffers;
mod config;
mod exit;
mod remote;
mod usage;
pub use self::exit::*;
use crate::src::nvim::eval::typval::kCallbackNone;
use crate::src::nvim::highlight_group::HLF_NONE;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::state::MODE_NORMAL;

/// A C string literal as the fixed-size `c_char` array a global holds.
///
/// c2rust spells every `static char foo[] = "…"` as a `transmute` from the
/// byte string, which is one `unsafe` block per message and 194 of them in
/// this file alone. Copying the bytes is const-evaluable, needs no `unsafe`,
/// and unlike `transmute` it works with a const generic length.
pub(crate) const fn c_bytes<const N: usize>(bytes: &[u8; N]) -> [c_char; N] {
    let mut out = [0 as c_char; N];
    let mut i = 0;
    while i < N {
        out[i] = bytes[i] as c_char;
        i += 1;
    }
    out
}

/// Record a startup-timing message, if `--startuptime` asked for one.
///
/// The C spells this as the `TIME_MSG` macro; it appears two dozen times
/// across the startup, and the `time_fd` test is the whole of it.
pub(crate) unsafe fn time_msg_at(what: &core::ffi::CStr) {
    // SAFETY: `time_fd` is the startup-timing file, opened once by
    // `init_startuptime` and closed by `time_finish`.
    unsafe {
        if !time_fd.get().is_null() {
            time_msg(what.as_ptr(), ::core::ptr::null::<proftime_T>());
        }
    }
}

pub(crate) const VAR_FIXED: VarLockStatus = 2;
pub(crate) const VAR_NUMBER: VarType = 1;
pub(crate) const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub(crate) const kListLenMayKnow: ListLenSpecials = -3;
pub const OPTION_MAGIC_OFF: optmagic_T = 2;
pub const OPTION_MAGIC_ON: optmagic_T = 1;
pub(crate) const OPTION_MAGIC_NOT_SET: optmagic_T = 0;
pub(crate) const kOptValTypeString: OptValType = 2;
pub(crate) const kOptValTypeNumber: OptValType = 1;
pub(crate) const kOptValTypeBoolean: OptValType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AucmdWinVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub(crate) const BLN_LISTED: bln_values = 2;
pub(crate) const VV_EXITREASON: VimVarIndex = 105;
pub(crate) const VV_STARTTIME: VimVarIndex = 104;
pub(crate) const VV_VIM_DID_INIT: VimVarIndex = 94;
pub(crate) const VV_EXITING: VimVarIndex = 91;
pub(crate) const VV_ARGF: VimVarIndex = 88;
pub(crate) const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub(crate) const VV_PROGPATH: VimVarIndex = 60;
pub(crate) const VV_OLDFILES: VimVarIndex = 58;
pub(crate) const VV_SWAPCOMMAND: VimVarIndex = 49;
pub(crate) const VV_PROGNAME: VimVarIndex = 27;
pub(crate) const kXDGConfigDirs: XDGVarType = 5;
pub(crate) const EVAL_EVALUATE: c_uint = 1;
pub(crate) const ECMD_HIDE: c_uint = 1;
pub(crate) const ECMD_LASTL: c_int = 0;
pub(crate) const READ_STDIN: c_uint = 4;
pub(crate) const READ_NEW: c_uint = 1;
pub(crate) const ETYPE_ENV: etype_T = 7;
pub(crate) const ETYPE_ARGS: etype_T = 6;
pub(crate) const ETYPE_TOP: etype_T = 0;
pub(crate) const KE_NOP: key_extra = 97;
pub(crate) const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mparm_T {
    pub argc: c_int,
    pub argv: *mut *mut c_char,
    pub use_vimrc: *mut c_char,
    pub clean: bool,
    pub n_commands: c_int,
    pub commands: [*mut c_char; 10],
    pub cmds_tofree: [c_char; 10],
    pub n_pre_commands: c_int,
    pub pre_commands: [*mut c_char; 10],
    pub luaf: *mut c_char,
    pub lua_arg0: c_int,
    pub edit_type: c_int,
    pub tagname: *mut c_char,
    pub use_ef: *mut c_char,
    pub input_istext: bool,
    pub no_swap_file: c_int,
    pub use_debug_break_level: c_int,
    pub window_count: c_int,
    pub window_layout: c_int,
    pub diff_mode: c_int,
    pub listen_addr: *mut c_char,
    pub remote: c_int,
    pub server_addr: *mut c_char,
    pub scriptin: *mut c_char,
    pub scriptout: *mut c_char,
    pub scriptout_append: bool,
    pub had_stdin_file: bool,
}
pub(crate) const EDIT_QF: c_uint = 4;
pub(crate) const WIN_TABS: c_uint = 3;
pub(crate) const WIN_VER: c_uint = 2;
pub(crate) const WIN_HOR: c_uint = 1;
pub(crate) const EDIT_STDIN: c_uint = 2;
pub(crate) const kEqualFiles: file_comparison = 1;
pub(crate) const DOSO_VIMRC: c_uint = 1;
pub(crate) const DOSO_NONE: c_uint = 0;
pub(crate) const EDIT_FILE: c_uint = 1;
pub(crate) const EDIT_TAG: c_uint = 3;
pub(crate) const EDIT_NONE: c_uint = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PumWant {
    pub active: bool,
    pub item: c_int,
    pub insert: bool,
    pub finish: bool,
}
pub(crate) const NULL_0: *mut c_void = ::core::ptr::null_mut::<c_void>();
pub static arena_alloc_count: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub(crate) const DEFAULT_MAXPATHL: c_int = 4096 as c_int;
pub(crate) const MAXPATHL: c_int = DEFAULT_MAXPATHL;
pub(crate) const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub(crate) const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub(crate) const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as c_int,
    ga_maxlen: 0 as c_int,
    ga_itemsize: 0 as c_int,
    ga_growsize: 1 as c_int,
    ga_data: NULL_0,
};
pub static g_min_log_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) const SESSION_FILE: [c_char; 12] = c_bytes(b"Session.vim\0");
pub(crate) const OK: c_int = 1 as c_int;
pub(crate) const FAIL: c_int = 0 as c_int;
pub static namespace_ids: GlobalCell<Map_String_int> = GlobalCell::new(Map_String_int {
    set: Set_String {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<String_0>(),
    },
    values: ::core::ptr::null_mut::<c_int>(),
});
pub static namespace_localscope: GlobalCell<Set_uint32_t> = GlobalCell::new(Set_uint32_t {
    h: MapHash {
        n_buckets: 0 as uint32_t,
        size: 0 as uint32_t,
        n_occupied: 0 as uint32_t,
        upper_bound: 0 as uint32_t,
        n_keys: 0 as uint32_t,
        keys_capacity: 0 as uint32_t,
        hash: ::core::ptr::null_mut::<uint32_t>(),
    },
    keys: ::core::ptr::null_mut::<uint32_t>(),
});
pub static next_namespace_id: GlobalCell<handle_T> = GlobalCell::new(1 as handle_T);
pub static buffer_handles: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static window_handles: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static tabpage_handles: GlobalCell<Map_int_ptr_t> = GlobalCell::new(Map_int_ptr_t {
    set: Set_int {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<c_int>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static ui_ext_names: GlobalCell<[*const c_char; 10]> = GlobalCell::new([
    b"ext_cmdline\0".as_ptr() as *const c_char,
    b"ext_popupmenu\0".as_ptr() as *const c_char,
    b"ext_tabline\0".as_ptr() as *const c_char,
    b"ext_wildmenu\0".as_ptr() as *const c_char,
    b"ext_messages\0".as_ptr() as *const c_char,
    b"ext_linegrid\0".as_ptr() as *const c_char,
    b"ext_multigrid\0".as_ptr() as *const c_char,
    b"ext_hlstate\0".as_ptr() as *const c_char,
    b"ext_termcolors\0".as_ptr() as *const c_char,
    b"_debug_float\0".as_ptr() as *const c_char,
]);
pub(crate) const NUL: c_int = '\0' as c_int;
pub(crate) const PATHSEP: c_int = '/' as c_int;
pub static last_cursormoved_win: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static last_cursormoved: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
});
pub static autocmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_no_enter: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static autocmd_no_leave: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static au_new_curbuf: GlobalCell<bufref_T> = GlobalCell::new(bufref_T::new());
pub static au_pending_free_buf: GlobalCell<*mut buf_T> =
    GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static au_pending_free_win: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static autocmd_fname: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static autocmd_fname_full: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_bufnr: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static autocmd_match: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static did_cursorhold: GlobalCell<bool> = GlobalCell::new(true);
#[unsafe(no_mangle)]
pub static aucmd_win_vec: GlobalCell<AucmdWinVec> = GlobalCell::new(AucmdWinVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<aucmdwin_T>(),
});
pub static deferred_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static msg_loclist: GlobalCell<*mut c_char> =
    GlobalCell::new(b"[Location List]\0".as_ptr() as *const c_char as *mut c_char);
pub static msg_qflist: GlobalCell<*mut c_char> =
    GlobalCell::new(b"[Quickfix List]\0".as_ptr() as *const c_char as *mut c_char);
pub static channels: GlobalCell<Map_uint64_t_ptr_t> = GlobalCell::new(Map_uint64_t_ptr_t {
    set: Set_uint64_t {
        h: MapHash {
            n_buckets: 0 as uint32_t,
            size: 0 as uint32_t,
            n_occupied: 0 as uint32_t,
            upper_bound: 0 as uint32_t,
            n_keys: 0 as uint32_t,
            keys_capacity: 0 as uint32_t,
            hash: ::core::ptr::null_mut::<uint32_t>(),
        },
        keys: ::core::ptr::null_mut::<uint64_t>(),
    },
    values: ::core::ptr::null_mut::<ptr_t>(),
});
pub static on_print: GlobalCell<Callback> = GlobalCell::new(Callback {
    data: Callback_data {
        funcref: ::core::ptr::null_mut::<c_char>(),
    },
    type_0: kCallbackNone,
});
pub static decor_state: GlobalCell<DecorState> = GlobalCell::new(DecorState {
    itr: [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [MarkTreeIter_s { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }],
    slots: Vec::new(),
    ranges_i: Vec::new(),
    current_end: 0,
    future_begin: 0,
    free_slot_i: 0,
    new_range_ordering: 0,
    win: ::core::ptr::null_mut::<win_T>(),
    top_row: 0,
    row: 0,
    col_last: 0,
    current: 0,
    eol_col: 0,
    conceal: 0,
    conceal_char: 0,
    conceal_attr: 0,
    spell: kFalse,
    running_decor_provider: false,
    itr_valid: false,
});
pub static diff_context: GlobalCell<c_int> = GlobalCell::new(6 as c_int);
pub static diff_foldcolumn: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
pub static diff_need_scrollbind: GlobalCell<bool> = GlobalCell::new(false);
pub static need_diff_redraw: GlobalCell<bool> = GlobalCell::new(false);
/// The `ui_watched` extmarks the redraw in progress has passed positions for.
///
/// Filled by `draw_virt_text` one window line at a time and drained by
/// `win_update` once the window is done, so it never outlives one window's
/// redraw. It was upstream's hand-rolled growable array; nothing outside the
/// two of them ever named its layout.
pub static win_extmark_arr: GlobalCell<Vec<WinExtmark>> = GlobalCell::new(Vec::new());
pub static updating_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static redraw_not_allowed: GlobalCell<bool> = GlobalCell::new(false);
pub static screen_search_hl: GlobalCell<match_T> = GlobalCell::new(match_T {
    rm: regmmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startpos: [lpos_T { lnum: 0, col: 0 }; 10],
        endpos: [lpos_T { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    },
    buf: ::core::ptr::null_mut::<buf_T>(),
    lnum: 0,
    attr: 0,
    attr_cur: 0,
    first_lnum: 0,
    startcol: 0,
    endcol: 0,
    is_addpos: false,
    has_cursor: false,
    tm: 0,
});
pub static search_hl_has_cursor_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static e_api_spawn_failed: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E903: Could not spawn API job\0"));
pub static e_argreq: GlobalCell<[c_char; 24]> =
    GlobalCell::new(c_bytes(b"E471: Argument required\0"));
pub static e_backslash: GlobalCell<[c_char; 39]> =
    GlobalCell::new(c_bytes(b"E10: \\ should be followed by /, ? or &\0"));
pub static e_cmdwin: GlobalCell<[c_char; 65]> = GlobalCell::new(c_bytes(
    b"E11: Invalid in command-line window; <CR> executes, CTRL-C quits\0",
));
pub static e_curdir: GlobalCell<[c_char; 69]> = GlobalCell::new(c_bytes(
    b"E12: Command not allowed in secure mode in current dir or tag search\0",
));
pub static e_invalid_buffer_name_str: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E158: Invalid buffer name: %s\0"));
pub static e_command_too_recursive: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E169: Command too recursive\0"));
pub static e_buffer_is_not_loaded: GlobalCell<[c_char; 27]> =
    GlobalCell::new(c_bytes(b"E681: Buffer is not loaded\0"));
pub static e_endif: GlobalCell<[c_char; 21]> = GlobalCell::new(c_bytes(b"E171: Missing :endif\0"));
pub static e_endtry: GlobalCell<[c_char; 22]> =
    GlobalCell::new(c_bytes(b"E600: Missing :endtry\0"));
pub static e_endwhile: GlobalCell<[c_char; 24]> =
    GlobalCell::new(c_bytes(b"E170: Missing :endwhile\0"));
pub static e_endfor: GlobalCell<[c_char; 22]> =
    GlobalCell::new(c_bytes(b"E170: Missing :endfor\0"));
pub static e_while: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E588: :endwhile without :while\0"));
pub static e_for: GlobalCell<[c_char; 27]> =
    GlobalCell::new(c_bytes(b"E588: :endfor without :for\0"));
pub static e_exists: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"E13: File exists (add ! to override)\0"));
pub static e_failed: GlobalCell<[c_char; 21]> = GlobalCell::new(c_bytes(b"E472: Command failed\0"));
pub static e_intern2: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E685: Internal error: %s\0"));
pub static e_interr: GlobalCell<[c_char; 12]> = GlobalCell::new(c_bytes(b"Interrupted\0"));
pub static e_invarg: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E474: Invalid argument\0"));
pub static e_invarg2: GlobalCell<[c_char; 27]> =
    GlobalCell::new(c_bytes(b"E475: Invalid argument: %s\0"));
pub static e_invargval: GlobalCell<[c_char; 36]> =
    GlobalCell::new(c_bytes(b"E475: Invalid value for argument %s\0"));
pub static e_invargNval: GlobalCell<[c_char; 40]> =
    GlobalCell::new(c_bytes(b"E475: Invalid value for argument %s: %s\0"));
pub static e_duparg2: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E983: Duplicate argument: %s\0"));
pub static e_invexpr2: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E15: Invalid expression: \"%s\"\0"));
pub static e_invrange: GlobalCell<[c_char; 19]> = GlobalCell::new(c_bytes(b"E16: Invalid range\0"));
pub static e_invcmd: GlobalCell<[c_char; 22]> =
    GlobalCell::new(c_bytes(b"E476: Invalid command\0"));
pub static e_isadir2: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E17: \"%s\" is a directory\0"));
pub static e_no_spell: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"E756: Spell checking is not possible\0"));
pub static e_invchan: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E900: Invalid channel id\0"));
pub static e_invchanjob: GlobalCell<[c_char; 36]> =
    GlobalCell::new(c_bytes(b"E900: Invalid channel id: not a job\0"));
pub static e_jobspawn: GlobalCell<[c_char; 40]> =
    GlobalCell::new(c_bytes(b"E903: Process failed to start: %s: \"%s\"\0"));
pub static e_channotpty: GlobalCell<[c_char; 27]> =
    GlobalCell::new(c_bytes(b"E904: channel is not a pty\0"));
pub static e_stdiochan2: GlobalCell<[c_char; 38]> =
    GlobalCell::new(c_bytes(b"E905: Couldn't open stdio channel: %s\0"));
pub static e_invstream: GlobalCell<[c_char; 33]> =
    GlobalCell::new(c_bytes(b"E906: invalid stream for channel\0"));
pub static e_invstreamrpc: GlobalCell<[c_char; 48]> = GlobalCell::new(c_bytes(
    b"E906: invalid stream for rpc channel, use 'rpc'\0",
));
pub static e_streamkey: GlobalCell<[c_char; 68]> = GlobalCell::new(c_bytes(
    b"E5210: dict key '%s' already set for buffered stream in channel %lu\0",
));
pub static e_libcall: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"E364: Library call failed for \"%s()\"\0"));
pub static e_fsync: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E667: Fsync failed: %s\0"));
pub static e_mkdir: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"E739: Cannot create directory %s: %s\0"));
pub static e_markinval: GlobalCell<[c_char; 34]> =
    GlobalCell::new(c_bytes(b"E19: Mark has invalid line number\0"));
pub static e_marknotset: GlobalCell<[c_char; 18]> =
    GlobalCell::new(c_bytes(b"E20: Mark not set\0"));
pub static e_modifiable: GlobalCell<[c_char; 46]> =
    GlobalCell::new(c_bytes(b"E21: Cannot make changes, 'modifiable' is off\0"));
pub static e_nesting: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E22: Scripts nested too deep\0"));
pub static e_noalt: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E23: No alternate file\0"));
pub static e_noabbr: GlobalCell<[c_char; 26]> =
    GlobalCell::new(c_bytes(b"E24: No such abbreviation\0"));
pub static e_nobang: GlobalCell<[c_char; 19]> = GlobalCell::new(c_bytes(b"E477: No ! allowed\0"));
pub static e_nogroup: GlobalCell<[c_char; 38]> =
    GlobalCell::new(c_bytes(b"E28: No such highlight group name: %s\0"));
pub static e_noinstext: GlobalCell<[c_char; 26]> =
    GlobalCell::new(c_bytes(b"E29: No inserted text yet\0"));
pub static e_nolastcmd: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E30: No previous command line\0"));
pub static e_nomap: GlobalCell<[c_char; 21]> = GlobalCell::new(c_bytes(b"E31: No such mapping\0"));
pub static e_noident: GlobalCell<[c_char; 33]> =
    GlobalCell::new(c_bytes(b"E349: No identifier under cursor\0"));
pub static e_nomatch: GlobalCell<[c_char; 15]> = GlobalCell::new(c_bytes(b"E479: No match\0"));
pub static e_nomatch2: GlobalCell<[c_char; 19]> = GlobalCell::new(c_bytes(b"E480: No match: %s\0"));
pub static e_noname: GlobalCell<[c_char; 18]> = GlobalCell::new(c_bytes(b"E32: No file name\0"));
pub static e_nopresub: GlobalCell<[c_char; 47]> =
    GlobalCell::new(c_bytes(b"E33: No previous substitute regular expression\0"));
pub static e_noprev: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E34: No previous command\0"));
pub static e_noprevre: GlobalCell<[c_char; 36]> =
    GlobalCell::new(c_bytes(b"E35: No previous regular expression\0"));
pub static e_norange: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E481: No range allowed\0"));
pub static e_noroom: GlobalCell<[c_char; 21]> = GlobalCell::new(c_bytes(b"E36: Not enough room\0"));
pub static e_notmp: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E483: Can't get temp file name\0"));
pub static e_notopen: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E484: Can't open file %s\0"));
pub static e_notopen_2: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E484: Can't open file %s: %s\0"));
pub static e_cant_read_file_str: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E485: Can't read file %s\0"));
pub static e_null: GlobalCell<[c_char; 19]> = GlobalCell::new(c_bytes(b"E38: Null argument\0"));
pub static e_number_exp: GlobalCell<[c_char; 21]> =
    GlobalCell::new(c_bytes(b"E39: Number expected\0"));
pub static e_openerrf: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E40: Can't open errorfile %s\0"));
pub static e_outofmem: GlobalCell<[c_char; 20]> =
    GlobalCell::new(c_bytes(b"E41: Out of memory!\0"));
pub static e_patnotf: GlobalCell<[c_char; 18]> = GlobalCell::new(c_bytes(b"Pattern not found\0"));
pub static e_patnotf2: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E486: Pattern not found: %s\0"));
pub static e_positive: GlobalCell<[c_char; 32]> =
    GlobalCell::new(c_bytes(b"E487: Argument must be positive\0"));
pub static e_prev_dir: GlobalCell<[c_char; 43]> =
    GlobalCell::new(c_bytes(b"E459: Cannot go back to previous directory\0"));
pub static e_no_errors: GlobalCell<[c_char; 15]> = GlobalCell::new(c_bytes(b"E42: No Errors\0"));
pub static e_loclist: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E776: No location list\0"));
pub static e_re_damg: GlobalCell<[c_char; 26]> =
    GlobalCell::new(c_bytes(b"E43: Damaged match string\0"));
pub static e_re_corr: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E44: Corrupted regexp program\0"));
pub static e_readonly: GlobalCell<[c_char; 50]> = GlobalCell::new(c_bytes(
    b"E45: 'readonly' option is set (add ! to override)\0",
));
pub static e_letwrong: GlobalCell<[c_char; 34]> =
    GlobalCell::new(c_bytes(b"E734: Wrong variable type for %s=\0"));
pub static e_illvar: GlobalCell<[c_char; 32]> =
    GlobalCell::new(c_bytes(b"E461: Illegal variable name: %s\0"));
pub static e_cannot_mod: GlobalCell<[c_char; 38]> =
    GlobalCell::new(c_bytes(b"E995: Cannot modify existing variable\0"));
pub static e_cannot_change_readonly_variable_str: GlobalCell<[c_char; 45]> =
    GlobalCell::new(c_bytes(b"E46: Cannot change read-only variable \"%.*s\"\0"));
pub static e_dictreq: GlobalCell<[c_char; 26]> =
    GlobalCell::new(c_bytes(b"E715: Dictionary required\0"));
pub static e_blobidx: GlobalCell<[c_char; 35]> =
    GlobalCell::new(c_bytes(b"E979: Blob index out of range: %ld\0"));
pub static e_invalblob: GlobalCell<[c_char; 33]> =
    GlobalCell::new(c_bytes(b"E978: Invalid operation for Blob\0"));
pub static e_toomanyarg: GlobalCell<[c_char; 42]> =
    GlobalCell::new(c_bytes(b"E118: Too many arguments for function: %s\0"));
pub static e_toofewarg: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E119: Not enough arguments for function: %s\0"));
pub static e_dictkey: GlobalCell<[c_char; 42]> =
    GlobalCell::new(c_bytes(b"E716: Key not present in Dictionary: \"%s\"\0"));
pub static e_dictkey_len: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E716: Key not present in Dictionary: \"%.*s\"\0"));
pub static e_listreq: GlobalCell<[c_char; 20]> = GlobalCell::new(c_bytes(b"E714: List required\0"));
pub static e_listblobreq: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E897: List or Blob required\0"));
pub static e_listblobarg: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E899: Argument of %s must be a List or Blob\0"));
pub static e_listdictarg: GlobalCell<[c_char; 50]> = GlobalCell::new(c_bytes(
    b"E712: Argument of %s must be a List or Dictionary\0",
));
pub static e_listdictblobarg: GlobalCell<[c_char; 56]> = GlobalCell::new(c_bytes(
    b"E896: Argument of %s must be a List, Dictionary or Blob\0",
));
pub static e_readerrf: GlobalCell<[c_char; 35]> =
    GlobalCell::new(c_bytes(b"E47: Error while reading errorfile\0"));
pub static e_sandbox: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E48: Not allowed in sandbox\0"));
pub static e_secure: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E523: Not allowed here\0"));
pub static e_textlock: GlobalCell<[c_char; 50]> = GlobalCell::new(c_bytes(
    b"E565: Not allowed to change text or change window\0",
));
pub static e_screenmode: GlobalCell<[c_char; 40]> =
    GlobalCell::new(c_bytes(b"E359: Screen mode setting not supported\0"));
pub static e_scroll: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E49: Invalid scroll size\0"));
pub static e_shellempty: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E91: 'shell' option is empty\0"));
pub static e_swapclose: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E72: Close error on swap file\0"));
pub static e_toocompl: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E74: Command too complex\0"));
pub static e_longname: GlobalCell<[c_char; 19]> = GlobalCell::new(c_bytes(b"E75: Name too long\0"));
pub static e_toomany: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E77: Too many file names\0"));
pub static e_trailing: GlobalCell<[c_char; 26]> =
    GlobalCell::new(c_bytes(b"E488: Trailing characters\0"));
pub static e_trailing_arg: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E488: Trailing characters: %s\0"));
pub static e_umark: GlobalCell<[c_char; 18]> = GlobalCell::new(c_bytes(b"E78: Unknown mark\0"));
pub static e_wildexpand: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E79: Cannot expand wildcards\0"));
pub static e_winheight: GlobalCell<[c_char; 56]> = GlobalCell::new(c_bytes(
    b"E591: 'winheight' cannot be smaller than 'winminheight'\0",
));
pub static e_winwidth: GlobalCell<[c_char; 54]> = GlobalCell::new(c_bytes(
    b"E592: 'winwidth' cannot be smaller than 'winminwidth'\0",
));
pub static e_write: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E80: Error while writing\0"));
pub static e_zerocount: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E939: Positive count required\0"));
pub static e_usingsid: GlobalCell<[c_char; 41]> =
    GlobalCell::new(c_bytes(b"E81: Using <SID> not in a script context\0"));
pub static e_missingparen: GlobalCell<[c_char; 30]> =
    GlobalCell::new(c_bytes(b"E107: Missing parentheses: %s\0"));
pub static e_empty_buffer: GlobalCell<[c_char; 19]> =
    GlobalCell::new(c_bytes(b"E749: Empty buffer\0"));
pub static e_nobufnr: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E86: Buffer %ld does not exist\0"));
pub static e_no_write_since_last_change: GlobalCell<[c_char; 32]> =
    GlobalCell::new(c_bytes(b"E37: No write since last change\0"));
pub static e_no_write_since_last_change_add_bang_to_override: GlobalCell<[c_char; 52]> =
    GlobalCell::new(c_bytes(
        b"E37: No write since last change (add ! to override)\0",
    ));
pub static e_no_write_since_last_change_for_buffer_nr_add_bang_to_override: GlobalCell<
    [c_char; 66],
> = GlobalCell::new(c_bytes(
    b"E89: No write since last change for buffer %d (add ! to override)\0",
));
pub static e_buffer_nr_not_found: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E92: Buffer %d not found\0"));
pub static e_unknown_function_str: GlobalCell<[c_char; 27]> =
    GlobalCell::new(c_bytes(b"E117: Unknown function: %s\0"));
pub static e_str_not_inside_function: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E193: %s not inside a function\0"));
pub static e_job_still_running: GlobalCell<[c_char; 24]> =
    GlobalCell::new(c_bytes(b"E948: Job still running\0"));
pub static e_job_still_running_add_bang_to_end_the_job: GlobalCell<[c_char; 47]> =
    GlobalCell::new(c_bytes(b"E948: Job still running (add ! to end the job)\0"));
pub static e_invalpat: GlobalCell<[c_char; 42]> =
    GlobalCell::new(c_bytes(b"E682: Invalid search pattern or delimiter\0"));
pub static e_bufloaded: GlobalCell<[c_char; 39]> =
    GlobalCell::new(c_bytes(b"E139: File is loaded in another buffer\0"));
pub static e_notset: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E764: Option '%s' is not set\0"));
pub static e_dirnotf: GlobalCell<[c_char; 40]> =
    GlobalCell::new(c_bytes(b"E919: Directory not found in '%s': \"%s\"\0"));
pub static e_au_recursive: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E952: Autocommand caused recursive behavior\0"));
pub static e_menu_only_exists_in_another_mode: GlobalCell<[c_char; 39]> =
    GlobalCell::new(c_bytes(b"E328: Menu only exists in another mode\0"));
pub static e_autocmd_close: GlobalCell<[c_char; 34]> =
    GlobalCell::new(c_bytes(b"E813: Cannot close autocmd window\0"));
pub static e_list_index_out_of_range_nr: GlobalCell<[c_char; 35]> =
    GlobalCell::new(c_bytes(b"E684: List index out of range: %ld\0"));
pub static e_listarg: GlobalCell<[c_char; 36]> =
    GlobalCell::new(c_bytes(b"E686: Argument of %s must be a List\0"));
pub static e_unsupportedoption: GlobalCell<[c_char; 27]> =
    GlobalCell::new(c_bytes(b"E519: Option not supported\0"));
pub static e_fnametoolong: GlobalCell<[c_char; 24]> =
    GlobalCell::new(c_bytes(b"E856: Filename too long\0"));
pub static e_using_float_as_string: GlobalCell<[c_char; 32]> =
    GlobalCell::new(c_bytes(b"E806: Using a Float as a String\0"));
pub static e_cannot_edit_other_buf: GlobalCell<[c_char; 45]> =
    GlobalCell::new(c_bytes(b"E788: Not allowed to edit another buffer now\0"));
pub static e_using_number_as_bool_nr: GlobalCell<[c_char; 36]> =
    GlobalCell::new(c_bytes(b"E1023: Using a Number as a Bool: %d\0"));
pub static e_not_callable_type_str: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E1085: Not a callable type: %s\0"));
pub static e_auabort: GlobalCell<[c_char; 43]> =
    GlobalCell::new(c_bytes(b"E855: Autocommands caused command to abort\0"));
pub static e_api_error: GlobalCell<[c_char; 20]> =
    GlobalCell::new(c_bytes(b"E5555: API call: %s\0"));
pub static e_fast_api_disabled: GlobalCell<[c_char; 53]> = GlobalCell::new(c_bytes(
    b"E5560: %s must not be called in a fast event context\0",
));
pub static e_floatonly: GlobalCell<[c_char; 62]> = GlobalCell::new(c_bytes(
    b"E5601: Cannot close window, only floating window would remain\0",
));
pub static e_floatexchange: GlobalCell<[c_char; 39]> =
    GlobalCell::new(c_bytes(b"E5602: Cannot exchange or rotate float\0"));
pub static e_cant_find_directory_str_in_cdpath: GlobalCell<[c_char; 42]> =
    GlobalCell::new(c_bytes(b"E344: Can't find directory \"%s\" in cdpath\0"));
pub static e_cant_find_file_str_in_path: GlobalCell<[c_char; 35]> =
    GlobalCell::new(c_bytes(b"E345: Can't find file \"%s\" in path\0"));
pub static e_no_more_directory_str_found_in_cdpath: GlobalCell<[c_char; 45]> =
    GlobalCell::new(c_bytes(b"E346: No more directory \"%s\" found in cdpath\0"));
pub static e_no_more_file_str_found_in_path: GlobalCell<[c_char; 38]> =
    GlobalCell::new(c_bytes(b"E347: No more file \"%s\" found in path\0"));
pub static e_value_is_locked: GlobalCell<[c_char; 22]> =
    GlobalCell::new(c_bytes(b"E741: Value is locked\0"));
pub static e_value_is_locked_str: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E741: Value is locked: %.*s\0"));
pub static e_cannot_change_value: GlobalCell<[c_char; 26]> =
    GlobalCell::new(c_bytes(b"E742: Cannot change value\0"));
pub static e_cannot_change_value_of_str: GlobalCell<[c_char; 34]> =
    GlobalCell::new(c_bytes(b"E742: Cannot change value of %.*s\0"));
pub static e_cannot_set_variable_in_sandbox_str: GlobalCell<[c_char; 49]> = GlobalCell::new(
    c_bytes(b"E794: Cannot set variable in the sandbox: \"%.*s\"\0"),
);
pub static e_cannot_delete_variable_str: GlobalCell<[c_char; 34]> =
    GlobalCell::new(c_bytes(b"E795: Cannot delete variable %.*s\0"));
pub static e_invalwindow: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E957: Invalid window number\0"));
pub static e_problem_creating_internal_diff: GlobalCell<[c_char; 41]> =
    GlobalCell::new(c_bytes(b"E960: Problem creating the internal diff\0"));
pub static e_cannot_define_autocommands_for_all_events: GlobalCell<[c_char; 49]> = GlobalCell::new(
    c_bytes(b"E1155: Cannot define autocommands for ALL events\0"),
);
pub static e_cannot_change_arglist_recursively: GlobalCell<[c_char; 51]> = GlobalCell::new(
    c_bytes(b"E1156: Cannot change the argument list recursively\0"),
);
pub static e_resulting_text_too_long: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E1240: Resulting text too long\0"));
pub static e_line_number_out_of_range: GlobalCell<[c_char; 32]> =
    GlobalCell::new(c_bytes(b"E1247: Line number out of range\0"));
pub static e_highlight_group_name_invalid_char: GlobalCell<[c_char; 39]> =
    GlobalCell::new(c_bytes(b"E5248: Invalid character in group name\0"));
pub static e_highlight_group_name_too_long: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"E1249: Highlight group name too long\0"));
pub static e_string_required: GlobalCell<[c_char; 22]> =
    GlobalCell::new(c_bytes(b"E928: String required\0"));
pub static e_invalid_column_number_nr: GlobalCell<[c_char; 33]> =
    GlobalCell::new(c_bytes(b"E964: Invalid column number: %ld\0"));
pub static e_invalid_line_number_nr: GlobalCell<[c_char; 31]> =
    GlobalCell::new(c_bytes(b"E966: Invalid line number: %ld\0"));
pub static e_reduce_of_an_empty_str_with_no_initial_value: GlobalCell<[c_char; 50]> =
    GlobalCell::new(c_bytes(
        b"E998: Reduce of an empty %s with no initial value\0",
    ));
pub static e_invalid_value_for_blob_nr: GlobalCell<[c_char; 36]> =
    GlobalCell::new(c_bytes(b"E1239: Invalid value for blob: 0xlX\0"));
pub static e_stray_closing_curly_str: GlobalCell<[c_char; 44]> =
    GlobalCell::new(c_bytes(b"E1278: Stray '}' without a matching '{': %s\0"));
pub static e_missing_close_curly_str: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"E1279: Missing '}': %s\0"));
pub static e_cannot_change_menus_while_listing: GlobalCell<[c_char; 41]> =
    GlobalCell::new(c_bytes(b"E1310: Cannot change menus while listing\0"));
pub static e_not_allowed_to_change_window_layout_in_this_autocmd: GlobalCell<[c_char; 63]> =
    GlobalCell::new(c_bytes(
        b"E1312: Not allowed to change the window layout in this autocmd\0",
    ));
pub static e_val_too_large_len: GlobalCell<[c_char; 29]> =
    GlobalCell::new(c_bytes(b"E1510: Value too large: %.*s\0"));
pub static e_undobang_cannot_redo_or_move_branch: GlobalCell<[c_char; 68]> = GlobalCell::new(
    c_bytes(b"E5767: Cannot use :undo! to redo or move to a different undo branch\0"),
);
pub static e_winfixbuf_cannot_go_to_buffer: GlobalCell<[c_char; 52]> = GlobalCell::new(c_bytes(
    b"E1513: Cannot switch buffer. 'winfixbuf' is enabled\0",
));
pub static e_invalid_return_type_from_findfunc: GlobalCell<[c_char; 45]> =
    GlobalCell::new(c_bytes(b"E1514: 'findfunc' did not return a List type\0"));
pub static e_cannot_switch_to_a_closing_buffer: GlobalCell<[c_char; 41]> =
    GlobalCell::new(c_bytes(b"E1546: Cannot switch to a closing buffer\0"));
pub static e_cannot_have_more_than_nr_diff_anchors: GlobalCell<[c_char; 45]> =
    GlobalCell::new(c_bytes(b"E1549: Cannot have more than %d diff anchors\0"));
pub static e_failed_to_find_all_diff_anchors: GlobalCell<[c_char; 39]> =
    GlobalCell::new(c_bytes(b"E1550: Failed to find all diff anchors\0"));
pub static e_diff_anchors_with_hidden_windows: GlobalCell<[c_char; 60]> = GlobalCell::new(c_bytes(
    b"E1562: Diff anchors cannot be used with hidden diff windows\0",
));
pub static e_leadtab_requires_tab: GlobalCell<[c_char; 66]> = GlobalCell::new(c_bytes(
    b"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified\0",
));
pub static e_invalid_format_string_single_percent_s: GlobalCell<[c_char; 55]> = GlobalCell::new(
    c_bytes(b"E1577: Invalid format string, only one \"%s\" is allowed\0"),
);
pub static e_cannot_read_from_str_2: GlobalCell<[c_char; 28]> =
    GlobalCell::new(c_bytes(b"E282: Cannot read from \"%s\"\0"));
pub(crate) static e_conflicting_configs: GlobalCell<[c_char; 38]> =
    GlobalCell::new(c_bytes(b"E5422: Conflicting configs: \"%s\" \"%s\"\0"));
pub static e_unknown_option2: GlobalCell<[c_char; 25]> =
    GlobalCell::new(c_bytes(b"E355: Unknown option: %s\0"));
pub static top_bot_msg: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"search hit TOP, continuing at BOTTOM\0"));
pub static bot_top_msg: GlobalCell<[c_char; 37]> =
    GlobalCell::new(c_bytes(b"search hit BOTTOM, continuing at TOP\0"));
pub static line_msg: GlobalCell<[c_char; 7]> = GlobalCell::new(c_bytes(b" line \0"));
pub static EVALARG_EVALUATE: GlobalCell<evalarg_T> = GlobalCell::new(evalarg_T {
    eval_flags: EVAL_EVALUATE as c_int,
    eval_getline: None,
    eval_cookie: ::core::ptr::null_mut::<c_void>(),
    eval_tofree: ::core::ptr::null_mut::<c_char>(),
});
pub static msg_ext_skip_flush: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_overwrite: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_skip_verbose: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<c_int>(),
    rows: 0 as c_int,
    cols: 0 as c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as c_int,
    comp_row: 0 as c_int,
    comp_col: 0 as c_int,
    comp_width: 0 as c_int,
    comp_height: 0 as c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
pub static msg_grid_pos: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_grid_adj: GlobalCell<GridView> = GlobalCell::new(GridView {
    target: ::core::ptr::null_mut::<ScreenGrid>(),
    row_offset: 0,
    col_offset: 0,
});
pub static msg_scrolled_at_flush: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_grid_scroll_discount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_listdo_overwrite: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
// TV_CSTRING (SIZE_MAX - 1): c2rust dropped the initializer expression and
// left 0, which is a valid pointer-sentinel value and would corrupt any
// caller comparing against it (the unit tests do, via FFI).
#[unsafe(no_mangle)]
pub static kTVCstring: GlobalCell<size_t> = GlobalCell::new(18446744073709551614);
pub static disable_fold_update: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const IOSIZE: c_int = 1024 as c_int + 1 as c_int;
pub(crate) const SYS_VIMRC_FILE: [c_char; 17] = c_bytes(b"$VIM/sysinit.vim\0");
pub(crate) const VIMRC_FILE: [c_char; 8] = c_bytes(b".nvimrc\0");
pub static g_stats: GlobalCell<nvim_stats_s> = GlobalCell::new(nvim_stats_s {
    fsync: 0 as int64_t,
    redraw: 0 as int64_t,
    log_skip: 0 as int16_t,
});
pub(crate) const NO_BUFFERS: c_int = 1 as c_int;
pub static Rows: GlobalCell<c_int> = GlobalCell::new(24 as c_int);
pub static Columns: GlobalCell<c_int> = GlobalCell::new(80 as c_int);
pub static mod_mask: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static vgetc_mod_mask: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static vgetc_char: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdline_row: GlobalCell<c_int> = GlobalCell::new(0);
pub static redraw_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static redraw_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static clear_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static mode_displayed: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdline_star: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static redrawing_cmdline: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdline_was_last_drawn: GlobalCell<bool> = GlobalCell::new(false);
pub static exec_from_reg: GlobalCell<bool> = GlobalCell::new(false);
pub static dollar_vcol: GlobalCell<colnr_T> = GlobalCell::new(-1 as colnr_T);
pub static edit_submode: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_pre: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_extra: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_highl: GlobalCell<hlf_T> = GlobalCell::new(HLF_NONE);
pub static cmdmsg_rl: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static msg_row: GlobalCell<c_int> = GlobalCell::new(0);
pub static msg_scrolled: GlobalCell<c_int> = GlobalCell::new(0);
pub static msg_scrolled_ign: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_did_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub static keep_msg: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static keep_msg_hl_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static need_fileinfo: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_scroll: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_didout: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_didany: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_nowait: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_off: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static info_message: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_hist_off: GlobalCell<bool> = GlobalCell::new(false);
pub static need_clr_eos: GlobalCell<bool> = GlobalCell::new(false);
#[unsafe(no_mangle)]
pub static emsg_skip: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_severe: GlobalCell<bool> = GlobalCell::new(false);
pub static emsg_assert_fails_msg: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static emsg_assert_fails_lnum: GlobalCell<c_long> = GlobalCell::new(0 as c_long);
pub static emsg_assert_fails_context: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static did_endif: GlobalCell<bool> = GlobalCell::new(false);
pub static did_emsg: GlobalCell<c_int> = GlobalCell::new(0);
pub static called_vim_beep: GlobalCell<bool> = GlobalCell::new(false);
pub static did_emsg_syntax: GlobalCell<bool> = GlobalCell::new(false);
pub static called_emsg: GlobalCell<c_int> = GlobalCell::new(0);
pub static ex_exitval: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_on_display: GlobalCell<bool> = GlobalCell::new(false);
pub static rc_did_emsg: GlobalCell<bool> = GlobalCell::new(false);
pub static no_wait_return: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static need_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub static did_wait_return: GlobalCell<bool> = GlobalCell::new(false);
pub static need_maketitle: GlobalCell<bool> = GlobalCell::new(true);
pub static quit_more: GlobalCell<bool> = GlobalCell::new(false);
pub static vgetc_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static didset_vim: GlobalCell<bool> = GlobalCell::new(false);
pub static didset_vimruntime: GlobalCell<bool> = GlobalCell::new(false);
pub static lines_left: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static msg_no_more: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_nesting_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static debug_break_level: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static debug_did_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static debug_tick: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static debug_backtrace_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static do_profiling: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static current_exception: GlobalCell<*mut except_T> =
    GlobalCell::new(::core::ptr::null_mut::<except_T>());
pub static did_throw: GlobalCell<bool> = GlobalCell::new(false);
pub static need_rethrow: GlobalCell<bool> = GlobalCell::new(false);
pub static check_cstack: GlobalCell<bool> = GlobalCell::new(false);
pub static trylevel: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static force_abort: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_list: GlobalCell<*mut *mut msglist_T> =
    GlobalCell::new(::core::ptr::null_mut::<*mut msglist_T>());
pub static suppress_errthrow: GlobalCell<bool> = GlobalCell::new(false);
pub static caught_stack: GlobalCell<*mut except_T> =
    GlobalCell::new(::core::ptr::null_mut::<except_T>());
pub static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SID_CMDARG: c_int = -2 as c_int;
pub(crate) const SID_CARG: c_int = -3 as c_int;
pub(crate) const SID_ENV: c_int = -4 as c_int;
pub static current_sctx: GlobalCell<sctx_T> = GlobalCell::new(sctx_T {
    sc_sid: 0 as scid_T,
    sc_seq: 0 as c_int,
    sc_lnum: 0 as linenr_T,
    sc_chan: 0 as uint64_t,
});
pub static current_ui: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static did_source_packages: GlobalCell<bool> = GlobalCell::new(false);
pub static provider_caller_scope: GlobalCell<caller_scope> = GlobalCell::new(caller_scope {
    script_ctx: sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    },
    es_entry: estack_T {
        es_lnum: 0,
        es_name: ::core::ptr::null_mut::<c_char>(),
        es_type: ETYPE_TOP,
        es_info: estack_T_es_info {
            sctx: ::core::ptr::null_mut::<sctx_T>(),
        },
    },
    autocmd_fname: ::core::ptr::null_mut::<c_char>(),
    autocmd_match: ::core::ptr::null_mut::<c_char>(),
    autocmd_fname_full: false,
    autocmd_bufnr: 0,
    funccalp: ::core::ptr::null_mut::<c_void>(),
});
pub static provider_call_nesting: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static t_colors: GlobalCell<c_int> = GlobalCell::new(256 as c_int);
pub static include_none: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static include_default: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static include_link: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static highlight_match: GlobalCell<bool> = GlobalCell::new(false);
pub static search_match_lines: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static search_match_endcol: GlobalCell<colnr_T> = GlobalCell::new(0);
pub static search_first_line: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static search_last_line: GlobalCell<linenr_T> = GlobalCell::new(MAXLNUM as c_int as linenr_T);
pub static no_smartcase: GlobalCell<bool> = GlobalCell::new(false);
pub static need_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static did_check_timestamps: GlobalCell<bool> = GlobalCell::new(false);
pub static no_check_timestamps: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static mouse_grid: GlobalCell<c_int> = GlobalCell::new(0);
pub static mouse_row: GlobalCell<c_int> = GlobalCell::new(0);
pub static mouse_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static mouse_past_bottom: GlobalCell<bool> = GlobalCell::new(false);
pub static mouse_past_eol: GlobalCell<bool> = GlobalCell::new(false);
pub static mouse_dragging: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static root_menu: GlobalCell<*mut vimmenu_T> =
    GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
pub static sys_menu: GlobalCell<bool> = GlobalCell::new(false);
#[unsafe(no_mangle)]
pub static firstwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[unsafe(no_mangle)]
pub static lastwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[unsafe(no_mangle)]
pub static prevwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
#[unsafe(no_mangle)]
pub static curwin: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static topframe: GlobalCell<*mut frame_T> = GlobalCell::new(::core::ptr::null_mut::<frame_T>());
#[unsafe(no_mangle)]
pub static first_tabpage: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
#[unsafe(no_mangle)]
pub static curtab: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
pub static lastused_tabpage: GlobalCell<*mut tabpage_T> =
    GlobalCell::new(::core::ptr::null_mut::<tabpage_T>());
pub static redraw_tabline: GlobalCell<bool> = GlobalCell::new(false);
pub static firstbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static lastbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
#[unsafe(no_mangle)]
pub static curbuf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static global_alist: GlobalCell<alist_T> = GlobalCell::new(alist_T {
    al_ga: garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<c_void>(),
    },
    al_refcount: 0,
    id: 0,
});
pub static max_alist_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static arg_had_last: GlobalCell<bool> = GlobalCell::new(false);
pub static ru_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static ru_wid: GlobalCell<c_int> = GlobalCell::new(0);
pub static sc_col: GlobalCell<c_int> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static starting: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
#[unsafe(no_mangle)]
pub static exiting: GlobalCell<bool> = GlobalCell::new(false);
pub static v_dying: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static stdin_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdout_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stderr_isatty: GlobalCell<bool> = GlobalCell::new(true);
pub static stdin_fd: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static full_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static secure: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static textlock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static allbuf_lock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static sandbox: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static silent_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static VIsual_active: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static VIsual_select_exclu_adj: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_VIsual_select: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static VIsual_reselect: GlobalCell<c_int> = GlobalCell::new(0);
pub static VIsual_mode: GlobalCell<c_int> = GlobalCell::new('v' as c_int);
pub static redo_VIsual_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static resel_VIsual_mode: GlobalCell<c_int> = GlobalCell::new('\0' as c_int);
pub static resel_VIsual_line_count: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static resel_VIsual_vcol: GlobalCell<colnr_T> = GlobalCell::new(0);
pub static where_paste_started: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static did_ai: GlobalCell<bool> = GlobalCell::new(false);
pub static ai_col: GlobalCell<colnr_T> = GlobalCell::new(0 as colnr_T);
pub static end_comment_pending: GlobalCell<c_int> = GlobalCell::new('\0' as c_int);
pub static did_syncbind: GlobalCell<bool> = GlobalCell::new(false);
pub static did_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si_back: GlobalCell<bool> = GlobalCell::new(false);
pub static old_indent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static saved_cursor: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0 as linenr_T,
    col: 0 as colnr_T,
    coladd: 0 as colnr_T,
});
pub static Insstart: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static Insstart_orig: GlobalCell<pos_T> = GlobalCell::new(pos_T {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static orig_line_count: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static vr_lines_changed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static inhibit_delete_count: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static fenc_default: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static State: GlobalCell<c_int> = GlobalCell::new(MODE_NORMAL);
pub static debug_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static finish_op: GlobalCell<bool> = GlobalCell::new(false);
pub static opcount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static motion_force: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub static pending_exmode_active: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_no_reprint: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdpreview: GlobalCell<bool> = GlobalCell::new(false);
pub static reg_recording: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static reg_executing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static pending_end_reg_executing: GlobalCell<bool> = GlobalCell::new(false);
pub static reg_recorded: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_mapping: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_zero_mapping: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static allow_keys: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_u_sync: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static u_sync_once: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static force_restart_edit: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_edit: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static arrow_used: GlobalCell<bool> = GlobalCell::new(false);
pub static ins_at_eol: GlobalCell<bool> = GlobalCell::new(false);
pub static no_abbr: GlobalCell<bool> = GlobalCell::new(true);
pub static mapped_ctrl_c: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ctrl_c_interrupts: GlobalCell<bool> = GlobalCell::new(true);
#[unsafe(no_mangle)]
pub static cmdmod: GlobalCell<cmdmod_T> = GlobalCell::new(cmdmod_T {
    cmod_flags: 0,
    cmod_split: 0,
    cmod_tab: 0,
    cmod_filter_pat: ::core::ptr::null_mut::<c_char>(),
    cmod_filter_regmatch: regmatch_T {
        regprog: ::core::ptr::null_mut::<regprog_T>(),
        startp: [::core::ptr::null_mut::<c_char>(); 10],
        endp: [::core::ptr::null_mut::<c_char>(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    },
    cmod_filter_force: false,
    cmod_verbose: 0,
    cmod_save_ei: ::core::ptr::null_mut::<c_char>(),
    cmod_did_sandbox: 0,
    cmod_verbose_save: 0,
    cmod_save_msg_silent: 0,
    cmod_save_msg_scroll: 0,
    cmod_did_esilent: 0,
});
pub static msg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static emsg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_noredir: GlobalCell<bool> = GlobalCell::new(false);
pub static cmd_silent: GlobalCell<bool> = GlobalCell::new(false);
pub static in_assert_fails: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SEA_NONE: c_int = 0 as c_int;
pub(crate) const SEA_DIALOG: c_int = 1 as c_int;
pub(crate) const SEA_QUIT: c_int = 2 as c_int;
pub static swap_exists_action: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static swap_exists_did_quit: GlobalCell<bool> = GlobalCell::new(false);
pub static IObuff: GlobalCell<[c_char; 1025]> = GlobalCell::new([0; 1025]);
#[unsafe(no_mangle)]
pub static NameBuff: GlobalCell<[c_char; 4096]> = GlobalCell::new([0; 4096]);
pub static msg_buf: GlobalCell<[c_char; 480]> = GlobalCell::new([0; 480]);
pub static os_buf: GlobalCell<[c_char; 4096]> = GlobalCell::new([0; 4096]);
pub static RedrawingDisabled: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static readonlymode: GlobalCell<bool> = GlobalCell::new(false);
pub static recoverymode: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf: GlobalCell<typebuf_T> = GlobalCell::new(typebuf_T {
    tb_buf: ::core::ptr::null_mut::<uint8_t>(),
    tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
    tb_buflen: 0 as c_int,
    tb_off: 0 as c_int,
    tb_len: 0 as c_int,
    tb_maplen: 0 as c_int,
    tb_silent: 0 as c_int,
    tb_no_abbr_cnt: 0 as c_int,
    tb_change_cnt: 0 as c_int,
});
pub static typebuf_was_empty: GlobalCell<bool> = GlobalCell::new(false);
pub static ex_normal_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static expr_map_lock: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ignore_script: GlobalCell<bool> = GlobalCell::new(false);
pub static stop_insert_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static KeyTyped: GlobalCell<bool> = GlobalCell::new(false);
pub static KeyStuffed: GlobalCell<c_int> = GlobalCell::new(0);
pub static maptick: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static must_redraw: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static skip_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub static do_redraw: GlobalCell<bool> = GlobalCell::new(false);
pub static must_redraw_pum: GlobalCell<bool> = GlobalCell::new(false);
pub static need_highlight_changed: GlobalCell<bool> = GlobalCell::new(true);
pub static scriptout: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static got_int: GlobalCell<bool> = GlobalCell::new(false);
pub static bangredo: GlobalCell<bool> = GlobalCell::new(false);
pub static searchcmdlen: GlobalCell<c_int> = GlobalCell::new(0);
pub static reg_do_extmatch: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static re_extmatch_in: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
pub static re_extmatch_out: GlobalCell<*mut reg_extmatch_T> =
    GlobalCell::new(::core::ptr::null_mut::<reg_extmatch_T>());
pub static did_outofmem_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static did_swapwrite_msg: GlobalCell<bool> = GlobalCell::new(false);
pub static global_busy: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static listcmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static need_start_insertmode: GlobalCell<bool> = GlobalCell::new(false);
pub static last_mode: GlobalCell<[c_char; 4]> = GlobalCell::new(c_bytes(b"n\0\0\0"));
pub static last_cmdline: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static repeat_cmdline: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static new_last_cmdline: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static postponed_split: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static postponed_split_flags: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static postponed_split_tab: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static g_do_tagpreview: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static g_tag_at_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub static replace_offset: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static escape_chars: GlobalCell<*mut c_char> =
    GlobalCell::new(b" \t\\\"|\0".as_ptr() as *const c_char as *mut c_char);
pub static keep_help_flag: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_off: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static redir_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static redir_vname: GlobalCell<bool> = GlobalCell::new(false);
pub static capture_ga: GlobalCell<*mut garray_T> =
    GlobalCell::new(::core::ptr::null_mut::<garray_T>());
pub static langmap_mapchar: GlobalCell<[uint8_t; 256]> = GlobalCell::new([0; 256]);
pub static save_p_ls: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static save_p_wmh: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static wild_menu_showing: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static globaldir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static last_chdir_reason: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static km_stopsel: GlobalCell<bool> = GlobalCell::new(false);
pub static km_startsel: GlobalCell<bool> = GlobalCell::new(false);
pub static cmdwin_type: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdwin_result: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdwin_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cmdwin_buf: GlobalCell<*mut buf_T> = GlobalCell::new(::core::ptr::null_mut::<buf_T>());
pub static cmdwin_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static cmdwin_old_curwin: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static cmdline_win: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
pub static no_lines_msg: GlobalCell<[c_char; 23]> =
    GlobalCell::new(c_bytes(b"--No lines in buffer--\0"));
pub static sub_nsubs: GlobalCell<c_int> = GlobalCell::new(0);
pub static sub_nlines: GlobalCell<linenr_T> = GlobalCell::new(0);
pub static wim_flags: GlobalCell<[uint8_t; 4]> = GlobalCell::new([0; 4]);
pub static stl_syntax: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf_was_filled: GlobalCell<bool> = GlobalCell::new(false);
pub static virtual_op: GlobalCell<TriState> = GlobalCell::new(kNone);
#[unsafe(no_mangle)]
pub static display_tick: GlobalCell<disptick_T> = GlobalCell::new(0 as disptick_T);
pub static spell_redraw_lnum: GlobalCell<linenr_T> = GlobalCell::new(0 as linenr_T);
pub static time_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static vim_ignored: GlobalCell<c_int> = GlobalCell::new(0);
pub static embedded_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static headless_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static windowsVersion: GlobalCell<[c_char; 20]> = GlobalCell::new([0 as c_char; 20]);
pub static magic_overruled: GlobalCell<optmagic_T> = GlobalCell::new(OPTION_MAGIC_NOT_SET);
pub static skip_win_fix_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub static skip_win_fix_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub static skip_update_topline: GlobalCell<bool> = GlobalCell::new(false);
pub static default_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<c_int>(),
    rows: 0 as c_int,
    cols: 0 as c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as c_int,
    comp_row: 0 as c_int,
    comp_col: 0 as c_int,
    comp_width: 0 as c_int,
    comp_height: 0 as c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
pub static default_gridview: GlobalCell<GridView> = GlobalCell::new(GridView {
    target: (default_grid.as_raw() as *const _) as *mut ScreenGrid,
    row_offset: 0,
    col_offset: 0,
});
pub static resizing_screen: GlobalCell<bool> = GlobalCell::new(false);
pub static linebuf_char: GlobalCell<*mut schar_T> =
    GlobalCell::new(::core::ptr::null_mut::<schar_T>());
pub static linebuf_attr: GlobalCell<*mut sattr_T> =
    GlobalCell::new(::core::ptr::null_mut::<sattr_T>());
pub static linebuf_vcol: GlobalCell<*mut colnr_T> =
    GlobalCell::new(::core::ptr::null_mut::<colnr_T>());
pub static linebuf_scratch: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static empty_string_option: GlobalCell<[c_char; 1]> = GlobalCell::new(c_bytes(b"\0"));
pub static p_ambw: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_acd: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ai: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bin: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bomb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bl: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_cin: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_channel: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cink: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cinsd: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cinw: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cfu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ofu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tsrfu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ci: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ar: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_aw: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_awa: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bg: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bk: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_bkc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static bkc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_bdir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static breakat_flags: GlobalCell<[c_char; 256]> = GlobalCell::new([0; 256]);
pub static bo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_bsk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_breakat: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bh: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_bt: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_busy: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cmp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cmp_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_enc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_deco: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ccv: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cino: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cedit: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cb: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_cwh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ch: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cms: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cpt: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cto: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_columns: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_confirm: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_cia: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cia_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_cot: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static cot_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_ac: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_act: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_acl: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pumborder: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pb: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ph: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_com: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cpo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_debug: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_def: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_inc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dia: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dip: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dict: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dg: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_dir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_dy: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static dy_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_ead: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_emoji: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ea: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ep: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_eb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ef: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_efm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_gefm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_gp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_eof: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_eol: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ei: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_et: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_exrc: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_fenc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fencs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ff: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ffs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_fic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ft: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fcs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ffu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fixeol: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_fcl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fdls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_fdo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static fdo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_fex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_flp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_fs: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_gd: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_guicursor: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_guifont: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_guifontwide: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_hlg: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hid: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_hl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_hls: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_hi: GlobalCell<OptInt> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static p_arshape: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_icon: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_iconstring: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_iminsert: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_imsearch: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_inf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_inex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_is: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_inde: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_indk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_icm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isi: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_isp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_js: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_jop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static jop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_keymap: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_kp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_km: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_langmap: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lnr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lrm: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lines: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_linespace: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_lisp: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lispwords: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_stal: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_lcs: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_lz: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_lpl: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_magic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_menc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mef: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mps: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mat: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mco: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mfd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mmd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mmp: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mis: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_mopt: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_msc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_msm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ml: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mle: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mls: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ma: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mod: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mouse: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mousem: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mousemev: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mousef: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mh: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_mousescroll: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_mousescroll_vert: GlobalCell<OptInt> = GlobalCell::new(3 as OptInt);
pub static p_mousescroll_hor: GlobalCell<OptInt> = GlobalCell::new(6 as OptInt);
pub static p_mouset: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_more: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_nf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_opfunc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_para: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_paste: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_pex: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_path: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_cdpath: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pi: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_pyx: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_qe: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ro: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_rdb: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static rdb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_rdt: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_re: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_report: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_pvh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_chi: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ari: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ri: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ru: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ruf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_pp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_qftf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_rtp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_scbk: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sj: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_so: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sbo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sections: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_secure: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sel: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_slm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ssop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static ssop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static p_sh: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_shcf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sp: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_shq: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_sxq: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
#[unsafe(no_mangle)]
pub static p_sxe: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_srr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_stmp: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_stl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wbr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_shm: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sbr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sc: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sloc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_sft: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sm: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_smd: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ss: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_siso: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_scs: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_si: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sta: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sts: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_sb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sua: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_swf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_smc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tpm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tal: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tpf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static tpf_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_tfu: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spf: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spo: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static spo_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_sps: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_spr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_sol: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_su: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_swb: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static swb_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_spk: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_syn: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tcl: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static tcl_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_ts: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tbs: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tc: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static tc_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_tl: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tr: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tags: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tgst: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tbidi: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_to: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_timeout: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_tm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_title: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_titlelen: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_titleold: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_titlestring: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tsr: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_tgc: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ttimeout: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ttm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_tf: GlobalCell<c_int> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static p_udir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_udf: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ul: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ur: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_uc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ut: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_shada: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_shadafile: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_termsync: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_vsts: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_vts: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_vdir: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_vop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static vop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_vb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_ve: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static ve_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_verbose: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_warn: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wop: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static wop_flags: GlobalCell<c_uint> = GlobalCell::new(0);
pub static p_window: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wak: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wig: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_ww: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wc: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wcm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wic: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wim: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wmnu: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_winborder: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static p_wh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wmh: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wmw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wiw: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_wm: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_ws: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_write: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wa: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wb: GlobalCell<c_int> = GlobalCell::new(0);
pub static p_wd: GlobalCell<OptInt> = GlobalCell::new(0);
pub static p_cdh: GlobalCell<c_int> = GlobalCell::new(0);
pub static highlight_attr: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub static highlight_attr_last: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub static highlight_user: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub static highlight_stlnc: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub static cterm_normal_fg_color: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static cterm_normal_bg_color: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static normal_fg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static normal_bg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static normal_sp: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub static ns_hl_global: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub static ns_hl_win: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub static ns_hl_fast: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub static ns_hl_active: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub static hl_attr_active: GlobalCell<*mut c_int> =
    GlobalCell::new((highlight_attr.as_raw() as *const _) as *mut c_int);
pub static curbuf_splice_pending: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) const LUA_GLOBALSINDEX: c_int = -10002 as c_int;
pub static nlua_global_refs: GlobalCell<*mut nlua_ref_state_t> =
    GlobalCell::new(::core::ptr::null_mut::<nlua_ref_state_t>());
pub static nlua_disable_preload: SharedCell<bool> = SharedCell::new(false);
pub static main_loop: SharedCell<Loop> = SharedCell::new(Loop {
    uv: uv_loop_t {
        data: ::core::ptr::null_mut::<c_void>(),
        active_handles: 0,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        active_reqs: uv_loop_s_active_reqs {
            unused: ::core::ptr::null_mut::<c_void>(),
        },
        internal_fields: ::core::ptr::null_mut::<c_void>(),
        stop_flag: 0,
        flags: 0,
        backend_fd: 0,
        pending_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watcher_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        watchers: ::core::ptr::null_mut::<*mut uv__io_t>(),
        nwatchers: 0,
        nfds: 0,
        wq: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        wq_mutex: pthread_mutex_t {
            __data: __pthread_mutex_s {
                __lock: 0,
                __count: 0,
                __owner: 0,
                __nusers: 0,
                __kind: 0,
                __spins: 0,
                __elision: 0,
                __list: __pthread_list_t {
                    __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
                    __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
                },
            },
        },
        wq_async: uv_async_t {
            data: ::core::ptr::null_mut::<c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: uv_async_s_u { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            async_cb: None,
            queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pending: 0,
        },
        cloexec_lock: pthread_rwlock_t {
            __data: __pthread_rwlock_arch_t {
                __readers: 0,
                __writers: 0,
                __wrphase_futex: 0,
                __writers_futex: 0,
                __pad3: 0,
                __pad4: 0,
                __cur_writer: 0,
                __shared: 0,
                __rwelision: 0,
                __pad1: [0; 7],
                __pad2: 0,
                __flags: 0,
            },
        },
        closing_handles: ::core::ptr::null_mut::<uv_handle_t>(),
        process_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        prepare_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        check_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        idle_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_handles: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        async_unused: None,
        async_io_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        async_wfd: 0,
        timer_heap: uv_loop_s_timer_heap {
            min: ::core::ptr::null_mut::<c_void>(),
            nelts: 0,
        },
        timer_counter: 0,
        time: 0,
        signal_pipefd: [0; 2],
        signal_io_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        child_watcher: uv_signal_t {
            data: ::core::ptr::null_mut::<c_void>(),
            loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
            type_0: UV_UNKNOWN_HANDLE,
            close_cb: None,
            handle_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            u: uv_signal_s_u { fd: 0 },
            next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
            flags: 0,
            signal_cb: None,
            signum: 0,
            tree_entry: uv_signal_s_tree_entry {
                rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
                rbe_color: 0,
            },
            caught_signals: 0,
            dispatched_signals: 0,
        },
        emfile_fd: 0,
        inotify_read_watcher: uv__io_t {
            cb: None,
            pending_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            watcher_queue: uv__queue {
                next: ::core::ptr::null_mut::<uv__queue>(),
                prev: ::core::ptr::null_mut::<uv__queue>(),
            },
            pevents: 0,
            events: 0,
            fd: 0,
        },
        inotify_watchers: ::core::ptr::null_mut::<c_void>(),
        inotify_fd: 0,
    },
    events: ::core::ptr::null_mut::<MultiQueue>(),
    thread_events: ::core::ptr::null_mut::<MultiQueue>(),
    fast_events: ::core::ptr::null_mut::<MultiQueue>(),
    children: ::core::ptr::null_mut::<Vec<*mut Proc>>(),
    children_watcher: uv_signal_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_signal_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        signal_cb: None,
        signum: 0,
        tree_entry: uv_signal_s_tree_entry {
            rbe_left: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_right: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_parent: ::core::ptr::null_mut::<uv_signal_s>(),
            rbe_color: 0,
        },
        caught_signals: 0,
        dispatched_signals: 0,
    },
    children_kill_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_timer_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: uv_timer_s_node {
            heap: [::core::ptr::null_mut::<c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    poll_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_timer_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: uv_timer_s_node {
            heap: [::core::ptr::null_mut::<c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    exit_delay_timer: uv_timer_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_timer_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        timer_cb: None,
        node: uv_timer_s_node {
            heap: [::core::ptr::null_mut::<c_void>(); 3],
        },
        timeout: 0,
        repeat: 0,
        start_id: 0,
    },
    async_0: uv_async_t {
        data: ::core::ptr::null_mut::<c_void>(),
        loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
        type_0: UV_UNKNOWN_HANDLE,
        close_cb: None,
        handle_queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        u: uv_async_s_u { fd: 0 },
        next_closing: ::core::ptr::null_mut::<uv_handle_t>(),
        flags: 0,
        async_cb: None,
        queue: uv__queue {
            next: ::core::ptr::null_mut::<uv__queue>(),
            prev: ::core::ptr::null_mut::<uv__queue>(),
        },
        pending: 0,
    },
    mutex: pthread_mutex_t {
        __data: __pthread_mutex_s {
            __lock: 0,
            __count: 0,
            __owner: 0,
            __nusers: 0,
            __kind: 0,
            __spins: 0,
            __elision: 0,
            __list: __pthread_list_t {
                __prev: ::core::ptr::null_mut::<__pthread_internal_list>(),
                __next: ::core::ptr::null_mut::<__pthread_internal_list>(),
            },
        },
    },
    recursive: 0,
    closing: false,
});
static argv0: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static err_arg_missing: GlobalCell<*const c_char> =
    GlobalCell::new(b"Argument missing after\0".as_ptr() as *const c_char);
static err_opt_garbage: GlobalCell<*const c_char> =
    GlobalCell::new(b"Garbage after option argument\0".as_ptr() as *const c_char);
static err_opt_unknown: GlobalCell<*const c_char> =
    GlobalCell::new(b"Unknown option argument\0".as_ptr() as *const c_char);
static err_too_many_args: GlobalCell<*const c_char> =
    GlobalCell::new(b"Too many edit arguments\0".as_ptr() as *const c_char);
static err_extra_cmd: GlobalCell<*const c_char> = GlobalCell::new(
    b"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments\0".as_ptr()
        as *const c_char,
);
pub static tslua_query_parse_count: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub(crate) const MAX_ARG_CMDS: c_int = 10 as c_int;
pub static namedfm: GlobalCell<[xfmark_T; 36]> = GlobalCell::new(
    [xfmark_T {
        fmark: fmark_T {
            mark: pos_T {
                lnum: 0 as linenr_T,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        },
        fname: ::core::ptr::null_mut::<c_char>(),
    }; 36],
);
pub static ch_before_blocking_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static showcmd_buf: GlobalCell<[c_char; 41]> = GlobalCell::new([0; 41]);
pub static repeat_luaref: GlobalCell<LuaRef> = GlobalCell::new(-2 as LuaRef);
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static pum_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid {
    handle: 0 as handle_T,
    chars: ::core::ptr::null_mut::<schar_T>(),
    attrs: ::core::ptr::null_mut::<sattr_T>(),
    vcols: ::core::ptr::null_mut::<colnr_T>(),
    line_offset: ::core::ptr::null_mut::<size_t>(),
    dirty_col: ::core::ptr::null_mut::<c_int>(),
    rows: 0 as c_int,
    cols: 0 as c_int,
    valid: false,
    throttled: false,
    blending: false,
    mouse_enabled: true,
    zindex: 0 as c_int,
    comp_row: 0 as c_int,
    comp_col: 0 as c_int,
    comp_width: 0 as c_int,
    comp_height: 0 as c_int,
    comp_index: 0 as size_t,
    comp_disabled: false,
    pending_comp_index_update: true,
});
#[unsafe(no_mangle)]
pub static pum_want: GlobalCell<PumWant> = GlobalCell::new(PumWant {
    active: false,
    item: 0,
    insert: false,
    finish: false,
});
pub static tab_page_click_defs: GlobalCell<*mut StlClickDefinition> =
    GlobalCell::new(::core::ptr::null_mut::<StlClickDefinition>());
pub static tab_page_click_defs_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub static ui_event_ns_id: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
pub static resize_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static ui_refresh_cmdheight: GlobalCell<bool> = GlobalCell::new(true);
pub static grid_line_buf_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub static grid_line_buf_char: GlobalCell<*mut schar_T> =
    GlobalCell::new(::core::ptr::null_mut::<schar_T>());
pub static grid_line_buf_attr: GlobalCell<*mut sattr_T> =
    GlobalCell::new(::core::ptr::null_mut::<sattr_T>());
pub static ui_client_channel_id: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static ui_client_error_exit: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static ui_client_exit_status: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static tabpage_move_disallowed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static float_anchor_str: GlobalCell<[*const c_char; 4]> = GlobalCell::new([
    b"NW\0".as_ptr() as *const c_char,
    b"NE\0".as_ptr() as *const c_char,
    b"SW\0".as_ptr() as *const c_char,
    b"SE\0".as_ptr() as *const c_char,
]);
pub(crate) const WRITEBIN: [c_char; 3] = c_bytes(b"wb\0");
pub(crate) const APPENDBIN: [c_char; 3] = c_bytes(b"ab\0");
unsafe extern "C" fn c2rust_run_static_initializers() {
    kTVCstring.set((18446744073709551615 as size_t).wrapping_sub(1 as size_t));
}
#[used]
#[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
#[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XIB"))]
#[cfg_attr(target_os = "macos", unsafe(link_section = "__DATA,__mod_init_func"))]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [c2rust_run_static_initializers];
