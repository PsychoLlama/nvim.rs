//! The editor's global state: upstream's `globals.h` has no translation unit
//! of its own, so the transpiler parked every `EXTERN` declaration here beside
//! `main()`. What follows is that header — roughly a thousand `GlobalCell`s
//! read from all over the tree. The startup path lives in the submodules.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::global_cell::{ConstTable, GlobalCell, SharedCell};
use crate::keycodes::ModMask;
use crate::options::{
    kOptArabic, kOptCbFlagUnnamed, kOptCbFlagUnnamedplus, kOptErrorfile, kOptKeymap, kOptRightleft,
    kOptShadafile, kOptShortmess, kOptVerbosefile, kOptWindow,
};
use crate::profile::time_msg;
use crate::registry::{IdSet, SlotTable, id_set};
use crate::types::{
    AdditionalData, ArgList, Array, AucmdWin, BlnFlags, Buffer, BufferRef, Callback, Channel,
    CmdMod, CmdModFlags, ColNr, DecorState, DispTick, EStack, EStackType, EstackInfo, Exception,
    FILE, FileComparison, FileMark, FileMarkView, Frame, GArray, Handle, Hlf, LPos, LineNr, Loop,
    LuaRef, LuaRetMode, MTNode, MTPos, MarkTreeIter, MarkTreeIterLevel, MatchState, MsgList,
    MultiQueue, NS, NluaRefState, Object, OptMagic, Pos, Proc, ProfTime, Refcount, RegExtMatch,
    RegMMatch, RegMatch, RegProg, RgbValue, ScreenGrid, ScriptCtx, StlClickDefinition, StlSyntax,
    Tabpage, UV_MUTEX_INIT, UV_RWLOCK_INIT, VimMenu, WinExtmark, Window, XDGVarType, XFileMark,
    caller_scope, int16_t, int32_t, int64_t, nvim_stats_s, size_t, uint8_t, uint32_t, uint64_t,
    uv__io_t, uv__queue, uv_async_s_u, uv_async_t, uv_handle_t, uv_handle_type,
    uv_loop_s_active_reqs, uv_loop_s_timer_heap, uv_loop_t, uv_signal_s, uv_signal_s_tree_entry,
    uv_signal_s_u, uv_signal_t, uv_timer_s_node, uv_timer_s_u, uv_timer_t,
};
use crate::winlayer::{BufId, TabId, WinId};
use core::ffi::{CStr, c_char, c_int, c_long, c_uint, c_void};

mod entry;
pub use self::entry::*;
mod args;
mod buffers;
mod config;
mod exit;
mod remote;
mod usage;
pub use self::exit::*;
use crate::highlight_group::HLF_NONE;
use crate::normal::VisualMode;
use crate::pos::MAXLNUM;
use crate::state::MODE_NORMAL;

/// A C string literal as the fixed-size `c_char` array a global holds.
///
/// c2rust spelled every `static char foo[] = "…"` as a `transmute` from the
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
/// The C spells this as the `TIME_MSG` macro. Safe: no raw pointer crosses
/// the boundary, and the `time_fd` test is the whole of it.
pub(crate) fn time_msg_at(what: &CStr) {
    if !time_fd.get().is_null() {
        // SAFETY: `time_fd` is the startup-timing file, opened once by
        // `init_startuptime` and closed by `time_finish`; `what` outlives the
        // call and the second argument is the "no elapsed time" null.
        unsafe { time_msg(what.as_ptr(), ::core::ptr::null::<ProfTime>()) };
    }
}

pub(crate) const UV_UNKNOWN_HANDLE: uv_handle_type = 0;
pub const OPTION_MAGIC_OFF: OptMagic = 2;
pub const OPTION_MAGIC_ON: OptMagic = 1;
pub(crate) const OPTION_MAGIC_NOT_SET: OptMagic = 0;
pub struct AucmdWinVec {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut AucmdWin,
}
pub(crate) const BLN_LISTED: BlnFlags = 2;
pub(crate) const kXDGConfigDirs: XDGVarType = 5;
pub(crate) const READ_STDIN: c_uint = 4;
pub(crate) const READ_NEW: c_uint = 1;
pub(crate) const ETYPE_ENV: EStackType = 7;
pub(crate) const ETYPE_ARGS: EStackType = 6;
pub(crate) const ETYPE_TOP: EStackType = 0;
pub(crate) const kRetObject: LuaRetMode = 0;
#[derive(Clone)] // not `Copy`: it owns several of its strings
pub struct MainParams {
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
pub(crate) const kEqualFiles: FileComparison = 1;
pub(crate) const DOSO_VIMRC: c_uint = 1;
pub(crate) const DOSO_NONE: c_uint = 0;
pub(crate) const EDIT_FILE: c_uint = 1;
pub(crate) const EDIT_TAG: c_uint = 3;
pub(crate) const EDIT_NONE: c_uint = 0;
#[derive(Copy, Clone)]
pub struct PumWant {
    pub active: bool,
    pub item: c_int,
    pub insert: bool,
    pub finish: bool,
}
pub static arena_alloc_count: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
pub(crate) const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub(crate) const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub static g_min_log_level: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) const SESSION_FILE: &CStr = c"Session.vim";
/// Namespace name -> id, in creation order.
///
/// khash, which this was, is insertion-ordered with a swap-remove; nothing
/// ever removes a namespace, so the order `nvim_get_namespaces` renders and
/// `describe_ns` searches is creation order. [`SlotTable`] keeps it.
pub(crate) static namespace_ids: GlobalCell<SlotTable<Box<[u8]>, Handle>> =
    GlobalCell::new(SlotTable::new());
/// The namespaces that are window-local rather than visible everywhere.
/// Membership only; never walked.
pub(crate) static namespace_localscope: GlobalCell<IdSet<uint32_t>> = GlobalCell::new(id_set());
pub static next_namespace_id: GlobalCell<Handle> = GlobalCell::new(1 as Handle);
pub static ui_ext_names: ConstTable<[*const c_char; 10]> = ConstTable::new([
    c"ext_cmdline".as_ptr(),
    c"ext_popupmenu".as_ptr(),
    c"ext_tabline".as_ptr(),
    c"ext_wildmenu".as_ptr(),
    c"ext_messages".as_ptr(),
    c"ext_linegrid".as_ptr(),
    c"ext_multigrid".as_ptr(),
    c"ext_hlstate".as_ptr(),
    c"ext_termcolors".as_ptr(),
    c"_debug_float".as_ptr(),
]);
pub(crate) const PATHSEP: c_int = '/' as c_int;
pub static last_cursormoved_win: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
pub static last_cursormoved: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0 as LineNr,
    col: 0 as ColNr,
    coladd: 0 as ColNr,
});
pub static autocmd_busy: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_no_enter: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static autocmd_no_leave: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static au_new_curbuf: GlobalCell<BufferRef> = GlobalCell::new(BufferRef::new());
pub static autocmd_fname: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static autocmd_fname_full: GlobalCell<bool> = GlobalCell::new(false);
pub static autocmd_bufnr: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static autocmd_match: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static did_cursorhold: GlobalCell<bool> = GlobalCell::new(true);
pub static aucmd_win_vec: GlobalCell<AucmdWinVec> = GlobalCell::new(AucmdWinVec {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<AucmdWin>(),
});
pub static deferred_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static msg_loclist: GlobalCell<*mut c_char> =
    GlobalCell::new(c"[Location List]".as_ptr() as *mut c_char);
pub static msg_qflist: GlobalCell<*mut c_char> =
    GlobalCell::new(c"[Quickfix List]".as_ptr() as *mut c_char);
/// Every open channel, by id. See [`crate::registry`] for the order this
/// keeps and the reentrancy rule it answers: a channel's callback runs Lua
/// and Vimscript, which can open and close channels, so nothing holds a
/// borrow of this across one.
pub(crate) static channels: GlobalCell<SlotTable<uint64_t, *mut Channel>> =
    GlobalCell::new(SlotTable::new());
pub static on_print: GlobalCell<Callback> = GlobalCell::new(Callback::None);
pub static decor_state: GlobalCell<DecorState> = GlobalCell::new(DecorState {
    itr: [MarkTreeIter {
        pos: MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [MarkTreeIterLevel { oldcol: 0, i: 0 }; 20],
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
    win: ::core::ptr::null_mut::<Window>(),
    top_row: 0,
    row: 0,
    col_last: 0,
    current: 0,
    eol_col: 0,
    conceal: 0,
    conceal_char: 0,
    conceal_attr: 0,
    spell: Some(false),
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
pub static screen_search_hl: GlobalCell<MatchState> = GlobalCell::new(MatchState {
    rm: RegMMatch {
        regprog: ::core::ptr::null_mut::<RegProg>(),
        startpos: [LPos { lnum: 0, col: 0 }; 10],
        endpos: [LPos { lnum: 0, col: 0 }; 10],
        rmm_matchcol: 0,
        rmm_ic: 0,
        rmm_maxcol: 0,
    },
    buf: ::core::ptr::null_mut::<Buffer>(),
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
pub static search_hl_has_cursor_lnum: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
pub static e_api_spawn_failed: &CStr = c"E903: Could not spawn API job";
pub static e_argreq: &CStr = c"E471: Argument required";
pub static e_backslash: &CStr = c"E10: \\ should be followed by /, ? or &";
pub static e_cmdwin: &CStr = c"E11: Invalid in command-line window; <CR> executes, CTRL-C quits";
pub static e_curdir: &CStr =
    c"E12: Command not allowed in secure mode in current dir or tag search";
pub static e_invalid_buffer_name_str: &CStr = c"E158: Invalid buffer name: %s";
pub static e_command_too_recursive: &CStr = c"E169: Command too recursive";
pub static e_buffer_is_not_loaded: &CStr = c"E681: Buffer is not loaded";
pub static e_endif: &CStr = c"E171: Missing :endif";
pub static e_endtry: &CStr = c"E600: Missing :endtry";
pub static e_endwhile: &CStr = c"E170: Missing :endwhile";
pub static e_endfor: &CStr = c"E170: Missing :endfor";
pub static e_while: &CStr = c"E588: :endwhile without :while";
pub static e_for: &CStr = c"E588: :endfor without :for";
pub static e_exists: &CStr = c"E13: File exists (add ! to override)";
pub static e_failed: &CStr = c"E472: Command failed";
pub static e_intern2: &CStr = c"E685: Internal error: %s";
pub static e_interr: &CStr = c"Interrupted";
pub static e_invarg: &CStr = c"E474: Invalid argument";
pub static e_invarg2: &CStr = c"E475: Invalid argument: %s";
pub static e_invargval: &CStr = c"E475: Invalid value for argument %s";
pub static e_invargNval: &CStr = c"E475: Invalid value for argument %s: %s";
pub static e_duparg2: &CStr = c"E983: Duplicate argument: %s";
pub static e_invexpr2: &CStr = c"E15: Invalid expression: \"%s\"";
pub static e_invrange: &CStr = c"E16: Invalid range";
pub static e_invcmd: &CStr = c"E476: Invalid command";
pub static e_isadir2: &CStr = c"E17: \"%s\" is a directory";
pub static e_no_spell: &CStr = c"E756: Spell checking is not possible";
pub static e_invchan: &CStr = c"E900: Invalid channel id";
pub static e_invchanjob: &CStr = c"E900: Invalid channel id: not a job";
pub static e_jobspawn: &CStr = c"E903: Process failed to start: %s: \"%s\"";
pub static e_channotpty: &CStr = c"E904: channel is not a pty";
pub static e_stdiochan2: &CStr = c"E905: Couldn't open stdio channel: %s";
pub static e_invstream: &CStr = c"E906: invalid stream for channel";
pub static e_invstreamrpc: &CStr = c"E906: invalid stream for rpc channel, use 'rpc'";
pub static e_streamkey: &CStr =
    c"E5210: dict key '%s' already set for buffered stream in channel %lu";
pub static e_libcall: &CStr = c"E364: Library call failed for \"%s()\"";
pub static e_fsync: &CStr = c"E667: Fsync failed: %s";
pub static e_mkdir: &CStr = c"E739: Cannot create directory %s: %s";
pub static e_markinval: &CStr = c"E19: Mark has invalid line number";
pub static e_marknotset: &CStr = c"E20: Mark not set";
pub static e_modifiable: &CStr = c"E21: Cannot make changes, 'modifiable' is off";
pub static e_nesting: &CStr = c"E22: Scripts nested too deep";
pub static e_noalt: &CStr = c"E23: No alternate file";
pub static e_noabbr: &CStr = c"E24: No such abbreviation";
pub static e_nobang: &CStr = c"E477: No ! allowed";
pub static e_nogroup: &CStr = c"E28: No such highlight group name: %s";
pub static e_noinstext: &CStr = c"E29: No inserted text yet";
pub static e_nolastcmd: &CStr = c"E30: No previous command line";
pub static e_nomap: &CStr = c"E31: No such mapping";
pub static e_noident: &CStr = c"E349: No identifier under cursor";
pub static e_nomatch: &CStr = c"E479: No match";
pub static e_nomatch2: &CStr = c"E480: No match: %s";
pub static e_noname: &CStr = c"E32: No file name";
pub static e_nopresub: &CStr = c"E33: No previous substitute regular expression";
pub static e_noprev: &CStr = c"E34: No previous command";
pub static e_noprevre: &CStr = c"E35: No previous regular expression";
pub static e_norange: &CStr = c"E481: No range allowed";
pub static e_noroom: &CStr = c"E36: Not enough room";
pub static e_notmp: &CStr = c"E483: Can't get temp file name";
pub static e_notopen: &CStr = c"E484: Can't open file %s";
pub static e_notopen_2: &CStr = c"E484: Can't open file %s: %s";
pub static e_cant_read_file_str: &CStr = c"E485: Can't read file %s";
pub static e_null: &CStr = c"E38: Null argument";
pub static e_number_exp: &CStr = c"E39: Number expected";
pub static e_openerrf: &CStr = c"E40: Can't open errorfile %s";
pub static e_outofmem: &CStr = c"E41: Out of memory!";
pub static e_patnotf: &CStr = c"Pattern not found";
pub static e_patnotf2: &CStr = c"E486: Pattern not found: %s";
pub static e_positive: &CStr = c"E487: Argument must be positive";
pub static e_prev_dir: &CStr = c"E459: Cannot go back to previous directory";
pub static e_no_errors: &CStr = c"E42: No Errors";
pub static e_loclist: &CStr = c"E776: No location list";
pub static e_re_damg: &CStr = c"E43: Damaged match string";
pub static e_re_corr: &CStr = c"E44: Corrupted regexp program";
pub static e_readonly: &CStr = c"E45: 'readonly' option is set (add ! to override)";
pub static e_letwrong: &CStr = c"E734: Wrong variable type for %s=";
pub static e_illvar: &CStr = c"E461: Illegal variable name: %s";
pub static e_cannot_mod: &CStr = c"E995: Cannot modify existing variable";
pub static e_cannot_change_readonly_variable_str: &CStr =
    c"E46: Cannot change read-only variable \"%.*s\"";
pub static e_dictreq: &CStr = c"E715: Dictionary required";
pub static e_blobidx: &CStr = c"E979: Blob index out of range: %ld";
pub static e_invalblob: &CStr = c"E978: Invalid operation for Blob";
pub static e_toomanyarg: &CStr = c"E118: Too many arguments for function: %s";
pub static e_toofewarg: &CStr = c"E119: Not enough arguments for function: %s";
pub static e_dictkey: &CStr = c"E716: Key not present in Dictionary: \"%s\"";
pub static e_dictkey_len: &CStr = c"E716: Key not present in Dictionary: \"%.*s\"";
pub static e_listreq: &CStr = c"E714: List required";
pub static e_listblobreq: &CStr = c"E897: List or Blob required";
pub static e_listblobarg: &CStr = c"E899: Argument of %s must be a List or Blob";
pub static e_listdictarg: &CStr = c"E712: Argument of %s must be a List or Dictionary";
pub static e_listdictblobarg: &CStr = c"E896: Argument of %s must be a List, Dictionary or Blob";
pub static e_readerrf: &CStr = c"E47: Error while reading errorfile";
pub static e_sandbox: &CStr = c"E48: Not allowed in sandbox";
pub static e_secure: &CStr = c"E523: Not allowed here";
pub static e_textlock: &CStr = c"E565: Not allowed to change text or change window";
pub static e_screenmode: &CStr = c"E359: Screen mode setting not supported";
pub static e_scroll: &CStr = c"E49: Invalid scroll size";
pub static e_shellempty: &CStr = c"E91: 'shell' option is empty";
pub static e_swapclose: &CStr = c"E72: Close error on swap file";
pub static e_toocompl: &CStr = c"E74: Command too complex";
pub static e_longname: &CStr = c"E75: Name too long";
pub static e_toomany: &CStr = c"E77: Too many file names";
pub static e_trailing: &CStr = c"E488: Trailing characters";
pub static e_trailing_arg: &CStr = c"E488: Trailing characters: %s";
pub static e_umark: &CStr = c"E78: Unknown mark";
pub static e_wildexpand: &CStr = c"E79: Cannot expand wildcards";
pub static e_winheight: &CStr = c"E591: 'winheight' cannot be smaller than 'winminheight'";
pub static e_winwidth: &CStr = c"E592: 'winwidth' cannot be smaller than 'winminwidth'";
pub static e_write: &CStr = c"E80: Error while writing";
pub static e_zerocount: &CStr = c"E939: Positive count required";
pub static e_usingsid: &CStr = c"E81: Using <SID> not in a script context";
pub static e_missingparen: &CStr = c"E107: Missing parentheses: %s";
pub static e_empty_buffer: &CStr = c"E749: Empty buffer";
pub static e_nobufnr: &CStr = c"E86: Buffer %ld does not exist";
pub static e_no_write_since_last_change: &CStr = c"E37: No write since last change";
pub static e_no_write_since_last_change_add_bang_to_override: &CStr =
    c"E37: No write since last change (add ! to override)";
pub static e_no_write_since_last_change_for_buffer_nr_add_bang_to_override: &CStr =
    c"E89: No write since last change for buffer %d (add ! to override)";
pub static e_buffer_nr_not_found: &CStr = c"E92: Buffer %d not found";
pub static e_unknown_function_str: &CStr = c"E117: Unknown function: %s";
pub static e_str_not_inside_function: &CStr = c"E193: %s not inside a function";
pub static e_job_still_running: &CStr = c"E948: Job still running";
pub static e_job_still_running_add_bang_to_end_the_job: &CStr =
    c"E948: Job still running (add ! to end the job)";
pub static e_invalpat: &CStr = c"E682: Invalid search pattern or delimiter";
pub static e_bufloaded: &CStr = c"E139: File is loaded in another buffer";
pub static e_notset: &CStr = c"E764: Option '%s' is not set";
pub static e_dirnotf: &CStr = c"E919: Directory not found in '%s': \"%s\"";
pub static e_au_recursive: &CStr = c"E952: Autocommand caused recursive behavior";
pub static e_menu_only_exists_in_another_mode: &CStr = c"E328: Menu only exists in another mode";
pub static e_autocmd_close: &CStr = c"E813: Cannot close autocmd window";
pub static e_list_index_out_of_range_nr: &CStr = c"E684: List index out of range: %ld";
pub static e_listarg: &CStr = c"E686: Argument of %s must be a List";
pub static e_unsupportedoption: &CStr = c"E519: Option not supported";
pub static e_fnametoolong: &CStr = c"E856: Filename too long";
pub static e_using_float_as_string: &CStr = c"E806: Using a Float as a String";
pub static e_cannot_edit_other_buf: &CStr = c"E788: Not allowed to edit another buffer now";
pub static e_using_number_as_bool_nr: &CStr = c"E1023: Using a Number as a Bool: %d";
pub static e_not_callable_type_str: &CStr = c"E1085: Not a callable type: %s";
pub static e_auabort: &CStr = c"E855: Autocommands caused command to abort";
pub static e_api_error: &CStr = c"E5555: API call: %s";
pub static e_fast_api_disabled: &CStr = c"E5560: %s must not be called in a fast event context";
pub static e_floatonly: &CStr = c"E5601: Cannot close window, only floating window would remain";
pub static e_floatexchange: &CStr = c"E5602: Cannot exchange or rotate float";
pub static e_cant_find_directory_str_in_cdpath: &CStr =
    c"E344: Can't find directory \"%s\" in cdpath";
pub static e_cant_find_file_str_in_path: &CStr = c"E345: Can't find file \"%s\" in path";
pub static e_no_more_directory_str_found_in_cdpath: &CStr =
    c"E346: No more directory \"%s\" found in cdpath";
pub static e_no_more_file_str_found_in_path: &CStr = c"E347: No more file \"%s\" found in path";
pub static e_value_is_locked: &CStr = c"E741: Value is locked";
pub static e_value_is_locked_str: &CStr = c"E741: Value is locked: %.*s";
pub static e_cannot_change_value: &CStr = c"E742: Cannot change value";
pub static e_cannot_change_value_of_str: &CStr = c"E742: Cannot change value of %.*s";
pub static e_cannot_set_variable_in_sandbox_str: &CStr =
    c"E794: Cannot set variable in the sandbox: \"%.*s\"";
pub static e_cannot_delete_variable_str: &CStr = c"E795: Cannot delete variable %.*s";
pub static e_invalwindow: &CStr = c"E957: Invalid window number";
pub static e_problem_creating_internal_diff: &CStr = c"E960: Problem creating the internal diff";
pub static e_cannot_define_autocommands_for_all_events: &CStr =
    c"E1155: Cannot define autocommands for ALL events";
pub static e_cannot_change_arglist_recursively: &CStr =
    c"E1156: Cannot change the argument list recursively";
pub static e_resulting_text_too_long: &CStr = c"E1240: Resulting text too long";
pub static e_line_number_out_of_range: &CStr = c"E1247: Line number out of range";
pub static e_highlight_group_name_invalid_char: &CStr = c"E5248: Invalid character in group name";
pub static e_highlight_group_name_too_long: &CStr = c"E1249: Highlight group name too long";
pub static e_string_required: &CStr = c"E928: String required";
pub static e_invalid_column_number_nr: &CStr = c"E964: Invalid column number: %ld";
pub static e_invalid_line_number_nr: &CStr = c"E966: Invalid line number: %ld";
pub static e_reduce_of_an_empty_str_with_no_initial_value: &CStr =
    c"E998: Reduce of an empty %s with no initial value";
pub static e_invalid_value_for_blob_nr: &CStr = c"E1239: Invalid value for blob: 0xlX";
pub static e_stray_closing_curly_str: &CStr = c"E1278: Stray '}' without a matching '{': %s";
pub static e_missing_close_curly_str: &CStr = c"E1279: Missing '}': %s";
pub static e_cannot_change_menus_while_listing: &CStr = c"E1310: Cannot change menus while listing";
pub static e_not_allowed_to_change_window_layout_in_this_autocmd: &CStr =
    c"E1312: Not allowed to change the window layout in this autocmd";
pub static e_val_too_large_len: &CStr = c"E1510: Value too large: %.*s";
pub static e_undobang_cannot_redo_or_move_branch: &CStr =
    c"E5767: Cannot use :undo! to redo or move to a different undo branch";
pub static e_winfixbuf_cannot_go_to_buffer: &CStr =
    c"E1513: Cannot switch buffer. 'winfixbuf' is enabled";
pub static e_invalid_return_type_from_findfunc: &CStr =
    c"E1514: 'findfunc' did not return a List type";
pub static e_cannot_switch_to_a_closing_buffer: &CStr = c"E1546: Cannot switch to a closing buffer";
pub static e_cannot_have_more_than_nr_diff_anchors: &CStr =
    c"E1549: Cannot have more than %d diff anchors";
pub static e_failed_to_find_all_diff_anchors: &CStr = c"E1550: Failed to find all diff anchors";
pub static e_diff_anchors_with_hidden_windows: &CStr =
    c"E1562: Diff anchors cannot be used with hidden diff windows";
pub static e_leadtab_requires_tab: &CStr =
    c"E1572: 'listchars' field \"leadtab\" requires \"tab\" to be specified";
pub static e_invalid_format_string_single_percent_s: &CStr =
    c"E1577: Invalid format string, only one \"%s\" is allowed";
pub static e_cannot_read_from_str_2: &CStr = c"E282: Cannot read from \"%s\"";
pub static e_unknown_option2: &CStr = c"E355: Unknown option: %s";
pub static top_bot_msg: &CStr = c"search hit TOP, continuing at BOTTOM";
pub static bot_top_msg: &CStr = c"search hit BOTTOM, continuing at TOP";
pub static line_msg: &CStr = c" line ";
pub static msg_ext_skip_flush: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_overwrite: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_ext_skip_verbose: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());
pub static msg_grid_pos: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_scrolled_at_flush: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_grid_scroll_discount: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static msg_listdo_overwrite: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
// TV_CSTRING (SIZE_MAX - 1): c2rust dropped the initializer expression and
// left 0, which is a valid pointer-sentinel value and would corrupt any
// caller comparing against it (the unit tests do, via FFI).
pub static kTVCstring: GlobalCell<size_t> = GlobalCell::new(18446744073709551614);
pub static disable_fold_update: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
#[unsafe(no_mangle)]
pub static test_disable_char_avail: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SYS_VIMRC_FILE: &CStr = c"$VIM/sysinit.vim";
pub(crate) const VIMRC_FILE: &CStr = c".nvimrc";
pub static g_stats: GlobalCell<nvim_stats_s> = GlobalCell::new(nvim_stats_s {
    fsync: 0 as int64_t,
    redraw: 0 as int64_t,
    log_skip: 0 as int16_t,
});
pub(crate) const NO_BUFFERS: c_int = 1 as c_int;
pub static Rows: GlobalCell<c_int> = GlobalCell::new(24 as c_int);
pub static Columns: GlobalCell<c_int> = GlobalCell::new(80 as c_int);
pub static mod_mask: GlobalCell<ModMask> = GlobalCell::new(ModMask::NONE);
pub static vgetc_mod_mask: GlobalCell<ModMask> = GlobalCell::new(ModMask::NONE);
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
pub static dollar_vcol: GlobalCell<ColNr> = GlobalCell::new(-1 as ColNr);
pub static edit_submode: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_pre: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_extra: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub static edit_submode_highl: GlobalCell<Hlf> = GlobalCell::new(HLF_NONE);
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
pub static current_exception: GlobalCell<*mut Exception> =
    GlobalCell::new(::core::ptr::null_mut::<Exception>());
pub static did_throw: GlobalCell<bool> = GlobalCell::new(false);
pub static need_rethrow: GlobalCell<bool> = GlobalCell::new(false);
pub static check_cstack: GlobalCell<bool> = GlobalCell::new(false);
pub static trylevel: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static force_abort: GlobalCell<bool> = GlobalCell::new(false);
pub static msg_list: GlobalCell<*mut *mut MsgList> =
    GlobalCell::new(::core::ptr::null_mut::<*mut MsgList>());
pub static suppress_errthrow: GlobalCell<bool> = GlobalCell::new(false);
pub static caught_stack: GlobalCell<*mut Exception> =
    GlobalCell::new(::core::ptr::null_mut::<Exception>());
pub static may_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static want_garbage_collect: GlobalCell<bool> = GlobalCell::new(false);
pub static garbage_collect_at_exit: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SID_CMDARG: c_int = -2 as c_int;
pub(crate) const SID_CARG: c_int = -3 as c_int;
pub(crate) const SID_ENV: c_int = -4 as c_int;
pub static current_sctx: GlobalCell<ScriptCtx> = GlobalCell::new(ScriptCtx::NONE);
pub static current_ui: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static did_source_packages: GlobalCell<bool> = GlobalCell::new(false);
pub static provider_caller_scope: GlobalCell<caller_scope> = GlobalCell::new(caller_scope {
    script_ctx: ScriptCtx::NONE,
    es_entry: EStack {
        es_lnum: 0,
        es_name: ::core::ptr::null_mut::<c_char>(),
        es_type: ETYPE_TOP,
        es_info: EstackInfo::None,
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
pub static search_match_lines: GlobalCell<LineNr> = GlobalCell::new(0);
pub static search_match_endcol: GlobalCell<ColNr> = GlobalCell::new(0);
pub static search_first_line: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
pub static search_last_line: GlobalCell<LineNr> = GlobalCell::new(MAXLNUM as c_int as LineNr);
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
pub static root_menu: GlobalCell<*mut VimMenu> =
    GlobalCell::new(::core::ptr::null_mut::<VimMenu>());
pub static sys_menu: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static firstwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub(crate) static lastwin: GlobalCell<Option<WinId>> = GlobalCell::new(None);
pub static prevwin: GlobalCell<*mut Window> = GlobalCell::new(::core::ptr::null_mut::<Window>());
#[unsafe(no_mangle)]
pub static curwin: GlobalCell<*mut Window> = GlobalCell::new(::core::ptr::null_mut::<Window>());
pub static topframe: GlobalCell<*mut Frame> = GlobalCell::new(::core::ptr::null_mut::<Frame>());
pub(crate) static first_tabpage: GlobalCell<Option<TabId>> = GlobalCell::new(None);
pub static curtab: GlobalCell<*mut Tabpage> = GlobalCell::new(::core::ptr::null_mut::<Tabpage>());
pub static lastused_tabpage: GlobalCell<*mut Tabpage> =
    GlobalCell::new(::core::ptr::null_mut::<Tabpage>());
pub static redraw_tabline: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static firstbuf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub(crate) static lastbuf: GlobalCell<Option<BufId>> = GlobalCell::new(None);
pub static curbuf: GlobalCell<*mut Buffer> = GlobalCell::new(::core::ptr::null_mut::<Buffer>());
pub static global_alist: GlobalCell<ArgList> = GlobalCell::new(ArgList {
    al_ga: Vec::new(),
    al_refcount: Refcount::ZERO,
    id: 0,
});
pub static max_alist_id: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static arg_had_last: GlobalCell<bool> = GlobalCell::new(false);
pub static ru_col: GlobalCell<c_int> = GlobalCell::new(0);
pub static ru_wid: GlobalCell<c_int> = GlobalCell::new(0);
pub static sc_col: GlobalCell<c_int> = GlobalCell::new(0);
#[unsafe(no_mangle)]
pub static starting: GlobalCell<c_int> = GlobalCell::new(2 as c_int);
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
pub static sandbox: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static silent_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static VIsual_select_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static VIsual_select_exclu_adj: GlobalCell<bool> = GlobalCell::new(false);
pub static restart_VIsual_select: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static VIsual_reselect: GlobalCell<c_int> = GlobalCell::new(0);
pub static redo_VIsual_busy: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static resel_VIsual_mode: GlobalCell<VisualMode> = GlobalCell::new(VisualMode::NONE);
pub static resel_VIsual_line_count: GlobalCell<LineNr> = GlobalCell::new(0);
pub static resel_VIsual_vcol: GlobalCell<ColNr> = GlobalCell::new(0);
pub static where_paste_started: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static did_ai: GlobalCell<bool> = GlobalCell::new(false);
pub static ai_col: GlobalCell<ColNr> = GlobalCell::new(0 as ColNr);
pub static end_comment_pending: GlobalCell<c_int> = GlobalCell::new('\0' as c_int);
pub static did_syncbind: GlobalCell<bool> = GlobalCell::new(false);
pub static did_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si: GlobalCell<bool> = GlobalCell::new(false);
pub static can_si_back: GlobalCell<bool> = GlobalCell::new(false);
pub static old_indent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static saved_cursor: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0 as LineNr,
    col: 0 as ColNr,
    coladd: 0 as ColNr,
});
pub static Insstart: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static Insstart_orig: GlobalCell<Pos> = GlobalCell::new(Pos {
    lnum: 0,
    col: 0,
    coladd: 0,
});
pub static orig_line_count: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
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
pub static cmdmod: GlobalCell<CmdMod> = GlobalCell::new(CmdMod {
    cmod_flags: CmdModFlags::NONE,
    cmod_split: 0,
    cmod_tab: 0,
    cmod_filter_pat: ::core::ptr::null_mut::<c_char>(),
    cmod_filter_regmatch: RegMatch {
        regprog: ::core::ptr::null_mut::<RegProg>(),
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
pub static emsg_silent: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static emsg_noredir: GlobalCell<bool> = GlobalCell::new(false);
pub static cmd_silent: GlobalCell<bool> = GlobalCell::new(false);
pub static in_assert_fails: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) const SEA_NONE: c_int = 0 as c_int;
pub(crate) const SEA_DIALOG: c_int = 1 as c_int;
pub(crate) const SEA_QUIT: c_int = 2 as c_int;
pub static swap_exists_action: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static swap_exists_did_quit: GlobalCell<bool> = GlobalCell::new(false);
pub static RedrawingDisabled: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static readonlymode: GlobalCell<bool> = GlobalCell::new(false);
pub static recoverymode: GlobalCell<bool> = GlobalCell::new(false);
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
pub static re_extmatch_in: GlobalCell<*mut RegExtMatch> =
    GlobalCell::new(::core::ptr::null_mut::<RegExtMatch>());
pub static re_extmatch_out: GlobalCell<*mut RegExtMatch> =
    GlobalCell::new(::core::ptr::null_mut::<RegExtMatch>());
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
    GlobalCell::new(c" \t\\\"|".as_ptr() as *mut c_char);
pub static keep_help_flag: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_off: GlobalCell<bool> = GlobalCell::new(false);
pub static redir_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static redir_reg: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static redir_vname: GlobalCell<bool> = GlobalCell::new(false);
pub static capture_ga: GlobalCell<*mut GArray> = GlobalCell::new(::core::ptr::null_mut::<GArray>());
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
pub static cmdwin_buf: GlobalCell<*mut Buffer> = GlobalCell::new(::core::ptr::null_mut::<Buffer>());
pub static cmdwin_win: GlobalCell<*mut Window> = GlobalCell::new(::core::ptr::null_mut::<Window>());
pub static cmdwin_old_curwin: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
pub static cmdline_win: GlobalCell<*mut Window> =
    GlobalCell::new(::core::ptr::null_mut::<Window>());
pub static no_lines_msg: &CStr = c"--No lines in buffer--";
pub static sub_nsubs: GlobalCell<c_int> = GlobalCell::new(0);
pub static sub_nlines: GlobalCell<LineNr> = GlobalCell::new(0);
pub static stl_syntax: GlobalCell<StlSyntax> = GlobalCell::new(StlSyntax::NONE);
pub static no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
pub static typebuf_was_filled: GlobalCell<bool> = GlobalCell::new(false);
pub static virtual_op: GlobalCell<Option<bool>> = GlobalCell::new(None);
#[unsafe(no_mangle)]
pub static display_tick: GlobalCell<DispTick> = GlobalCell::new(0 as DispTick);
pub static spell_redraw_lnum: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
pub static time_fd: GlobalCell<*mut FILE> = GlobalCell::new(::core::ptr::null_mut::<FILE>());
pub static vim_ignored: GlobalCell<c_int> = GlobalCell::new(0);
pub static embedded_mode: GlobalCell<bool> = GlobalCell::new(false);
pub static headless_mode: GlobalCell<bool> = GlobalCell::new(false);
/// The Windows release `windowsversion()` reports. Nothing here writes it,
/// so it stays the empty string a non-Windows build always answered.
pub static windowsVersion: [c_char; 20] = [0 as c_char; 20];
pub static magic_overruled: GlobalCell<OptMagic> = GlobalCell::new(OPTION_MAGIC_NOT_SET);
pub static skip_win_fix_cursor: GlobalCell<bool> = GlobalCell::new(false);
pub static skip_win_fix_scroll: GlobalCell<bool> = GlobalCell::new(false);
pub static skip_update_topline: GlobalCell<bool> = GlobalCell::new(false);
pub static default_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());
pub static resizing_screen: GlobalCell<bool> = GlobalCell::new(false);
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
pub static nlua_global_refs: GlobalCell<*mut NluaRefState> =
    GlobalCell::new(::core::ptr::null_mut::<NluaRefState>());
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
        wq_mutex: UV_MUTEX_INIT,
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
        cloexec_lock: UV_RWLOCK_INIT,
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
    mutex: UV_MUTEX_INIT,
    recursive: 0,
    closing: false,
});
static argv0: GlobalCell<*mut c_char> = GlobalCell::new(::core::ptr::null_mut::<c_char>());
static err_arg_missing: GlobalCell<*const c_char> =
    GlobalCell::new(c"Argument missing after".as_ptr());
static err_opt_garbage: GlobalCell<*const c_char> =
    GlobalCell::new(c"Garbage after option argument".as_ptr());
static err_opt_unknown: GlobalCell<*const c_char> =
    GlobalCell::new(c"Unknown option argument".as_ptr());
static err_too_many_args: GlobalCell<*const c_char> =
    GlobalCell::new(c"Too many edit arguments".as_ptr());
static err_extra_cmd: GlobalCell<*const c_char> = GlobalCell::new(
    c"Too many \"+command\", \"-c command\" or \"--cmd command\" arguments".as_ptr(),
);
pub static tslua_query_parse_count: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub(crate) const MAX_ARG_CMDS: c_int = 10 as c_int;
/// An unset entry of [`namedfm`]; a `const` because `XFileMark` is not `Copy`.
const UNSET_NAMED_MARK: XFileMark = XFileMark {
    fmark: FileMark {
        mark: Pos {
            lnum: 0 as LineNr,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: FileMarkView {
            topline_offset: 0,
            skipcol: 0,
        },
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    fname: ::core::ptr::null_mut::<c_char>(),
};
pub static namedfm: GlobalCell<[XFileMark; 36]> = GlobalCell::new([UNSET_NAMED_MARK; 36]);
pub static ch_before_blocking_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub static repeat_luaref: GlobalCell<LuaRef> = GlobalCell::new(-2 as LuaRef);
pub static used_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static nvim_testing: GlobalCell<bool> = GlobalCell::new(false);
pub static pum_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());
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
pub static ui_client_channel_id: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub static ui_client_error_exit: GlobalCell<c_int> = GlobalCell::new(-1 as c_int);
pub static ui_client_exit_status: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static ui_client_attached: GlobalCell<bool> = GlobalCell::new(false);
pub static ui_client_forward_stdin: GlobalCell<bool> = GlobalCell::new(false);
pub static tabpage_move_disallowed: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub static float_anchor_str: GlobalCell<[*const c_char; 4]> = GlobalCell::new([
    c"NW".as_ptr(),
    c"NE".as_ptr(),
    c"SW".as_ptr(),
    c"SE".as_ptr(),
]);
pub(crate) const WRITEBIN: &CStr = c"wb";
pub(crate) const APPENDBIN: &CStr = c"ab";
