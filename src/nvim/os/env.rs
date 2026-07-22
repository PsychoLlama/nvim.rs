use crate::src::nvim::charset::{skipwhite, vim_isIDc, vim_isfilec};
use crate::src::nvim::cmdexpand::{ExpandInit, ExpandOne};
use crate::src::nvim::eval::fs::modify_fname;
use crate::src::nvim::eval::vars::get_vim_var_str;
use crate::src::nvim::eval_1::skip_expr;
use crate::src::nvim::event::libuv::{
    uv_err_name, uv_os_getenv, uv_os_homedir, uv_os_setenv, uv_os_unsetenv, uv_strerror,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{
    didset_vim, didset_vimruntime, nvim_testing, os_buf, p_hf, IObuff, NameBuff,
};
use crate::src::nvim::memory::{
    xfree, xmalloc, xmemcpyz, xmemdupz, xmemrchr, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::message::internal_error;
use crate::src::nvim::os::fs::{os_dirname, os_isdir, os_realpath};
use crate::src::nvim::os::libc::{
    __assert_fail, environ, getpid, memcpy, strcasecmp, strchr, strcmp, strcpy, strlen, strncmp,
    strpbrk,
};
use crate::src::nvim::os::users::os_get_userdir;
use crate::src::nvim::path::{
    after_pathsep, append_path, concat_fnames, path_fnamencmp, path_is_absolute, path_tail,
    path_tail_with_sep, vim_ispathsep,
};
use crate::src::nvim::strings::{striequal, vim_strchr, vim_strsave_escaped};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, Callback, CallbackType,
    Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt, DecorHighlightInline,
    DecorInlineData, DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2,
    Direction, ExtmarkUndoObject, FileID, FloatAnchor, FloatRelative, GridView, Intersection,
    LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash, Map_int64_t_int64_t, Map_int64_t_ptr_t,
    Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree, OptInt, ScopeDictDictItem, ScopeType,
    ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t, SpecialVarValue, StlClickDefinition,
    StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal, Timestamp, VarLockStatus, VarType,
    VimVarIndex, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig, WinInfo, WinSplit,
    WinStyle, Window, __pid_t, __time_t, alist_T, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T,
    bufstate_T, chunksize_T, colnr_T, dict_T, dictvar_S, disptick_T, evalarg_T, expand_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, handle_T, hash_T,
    hashitem_T, hashtab_T, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T, linenr_T, list_T,
    listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T, mapblock,
    mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T, mtnode_inner_s,
    mtnode_s, partial_S, partial_T, pos_T, pos_save_T, proftime_T, ptr_t, ptrdiff_t, qf_info_S,
    qf_info_T, queue, reg_extmatch_T, regmmatch_T, regprog, regprog_T, sattr_T, schar_T, scid_T,
    sctx_T, size_t, syn_state, syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T,
    synstate_T, taggy_T, terminal, time_t, typval_T, typval_vval_union, u_entry, u_entry_T,
    u_header, u_header_T, u_header_uh_alt_next as C2Rust_Unnamed_8,
    u_header_uh_alt_prev as C2Rust_Unnamed_7, u_header_uh_next as C2Rust_Unnamed_10,
    u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T, uint16_t, uint32_t, uint64_t, uint8_t,
    undo_object, varnumber_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S, winopt_T,
    wline_T, xfmark_T, xp_prefix_T, QUEUE,
};
extern "C" {
    fn uname(__name: *mut utsname) -> ::core::ffi::c_int;
}
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ERRNO_MAX: C2Rust_Unnamed = -4096;
pub const UV_ENOEXEC: C2Rust_Unnamed = -8;
pub const UV_EUNATCH: C2Rust_Unnamed = -49;
pub const UV_ENODATA: C2Rust_Unnamed = -61;
pub const UV_ESOCKTNOSUPPORT: C2Rust_Unnamed = -94;
pub const UV_EILSEQ: C2Rust_Unnamed = -84;
pub const UV_EFTYPE: C2Rust_Unnamed = -4028;
pub const UV_ENOTTY: C2Rust_Unnamed = -25;
pub const UV_EREMOTEIO: C2Rust_Unnamed = -121;
pub const UV_EHOSTDOWN: C2Rust_Unnamed = -112;
pub const UV_EMLINK: C2Rust_Unnamed = -31;
pub const UV_ENXIO: C2Rust_Unnamed = -6;
pub const UV_EOF: C2Rust_Unnamed = -4095;
pub const UV_UNKNOWN: C2Rust_Unnamed = -4094;
pub const UV_EXDEV: C2Rust_Unnamed = -18;
pub const UV_ETXTBSY: C2Rust_Unnamed = -26;
pub const UV_ETIMEDOUT: C2Rust_Unnamed = -110;
pub const UV_ESRCH: C2Rust_Unnamed = -3;
pub const UV_ESPIPE: C2Rust_Unnamed = -29;
pub const UV_ESHUTDOWN: C2Rust_Unnamed = -108;
pub const UV_EROFS: C2Rust_Unnamed = -30;
pub const UV_ERANGE: C2Rust_Unnamed = -34;
pub const UV_EPROTOTYPE: C2Rust_Unnamed = -91;
pub const UV_EPROTONOSUPPORT: C2Rust_Unnamed = -93;
pub const UV_EPROTO: C2Rust_Unnamed = -71;
pub const UV_EPIPE: C2Rust_Unnamed = -32;
pub const UV_EPERM: C2Rust_Unnamed = -1;
pub const UV_EOVERFLOW: C2Rust_Unnamed = -75;
pub const UV_ENOTSUP: C2Rust_Unnamed = -95;
pub const UV_ENOTSOCK: C2Rust_Unnamed = -88;
pub const UV_ENOTEMPTY: C2Rust_Unnamed = -39;
pub const UV_ENOTDIR: C2Rust_Unnamed = -20;
pub const UV_ENOTCONN: C2Rust_Unnamed = -107;
pub const UV_ENOSYS: C2Rust_Unnamed = -38;
pub const UV_ENOSPC: C2Rust_Unnamed = -28;
pub const UV_ENOPROTOOPT: C2Rust_Unnamed = -92;
pub const UV_ENONET: C2Rust_Unnamed = -64;
pub const UV_ENOMEM: C2Rust_Unnamed = -12;
pub const UV_ENOENT: C2Rust_Unnamed = -2;
pub const UV_ENODEV: C2Rust_Unnamed = -19;
pub const UV_ENOBUFS: C2Rust_Unnamed = -105;
pub const UV_ENFILE: C2Rust_Unnamed = -23;
pub const UV_ENETUNREACH: C2Rust_Unnamed = -101;
pub const UV_ENETDOWN: C2Rust_Unnamed = -100;
pub const UV_ENAMETOOLONG: C2Rust_Unnamed = -36;
pub const UV_EMSGSIZE: C2Rust_Unnamed = -90;
pub const UV_EMFILE: C2Rust_Unnamed = -24;
pub const UV_ELOOP: C2Rust_Unnamed = -40;
pub const UV_EISDIR: C2Rust_Unnamed = -21;
pub const UV_EISCONN: C2Rust_Unnamed = -106;
pub const UV_EIO: C2Rust_Unnamed = -5;
pub const UV_EINVAL: C2Rust_Unnamed = -22;
pub const UV_EINTR: C2Rust_Unnamed = -4;
pub const UV_EHOSTUNREACH: C2Rust_Unnamed = -113;
pub const UV_EFBIG: C2Rust_Unnamed = -27;
pub const UV_EFAULT: C2Rust_Unnamed = -14;
pub const UV_EEXIST: C2Rust_Unnamed = -17;
pub const UV_EDESTADDRREQ: C2Rust_Unnamed = -89;
pub const UV_ECONNRESET: C2Rust_Unnamed = -104;
pub const UV_ECONNREFUSED: C2Rust_Unnamed = -111;
pub const UV_ECONNABORTED: C2Rust_Unnamed = -103;
pub const UV_ECHARSET: C2Rust_Unnamed = -4080;
pub const UV_ECANCELED: C2Rust_Unnamed = -125;
pub const UV_EBUSY: C2Rust_Unnamed = -16;
pub const UV_EBADF: C2Rust_Unnamed = -9;
pub const UV_EALREADY: C2Rust_Unnamed = -114;
pub const UV_EAI_SOCKTYPE: C2Rust_Unnamed = -3011;
pub const UV_EAI_SERVICE: C2Rust_Unnamed = -3010;
pub const UV_EAI_PROTOCOL: C2Rust_Unnamed = -3014;
pub const UV_EAI_OVERFLOW: C2Rust_Unnamed = -3009;
pub const UV_EAI_NONAME: C2Rust_Unnamed = -3008;
pub const UV_EAI_NODATA: C2Rust_Unnamed = -3007;
pub const UV_EAI_MEMORY: C2Rust_Unnamed = -3006;
pub const UV_EAI_FAMILY: C2Rust_Unnamed = -3005;
pub const UV_EAI_FAIL: C2Rust_Unnamed = -3004;
pub const UV_EAI_CANCELED: C2Rust_Unnamed = -3003;
pub const UV_EAI_BADHINTS: C2Rust_Unnamed = -3013;
pub const UV_EAI_BADFLAGS: C2Rust_Unnamed = -3002;
pub const UV_EAI_AGAIN: C2Rust_Unnamed = -3001;
pub const UV_EAI_ADDRFAMILY: C2Rust_Unnamed = -3000;
pub const UV_EAGAIN: C2Rust_Unnamed = -11;
pub const UV_EAFNOSUPPORT: C2Rust_Unnamed = -97;
pub const UV_EADDRNOTAVAIL: C2Rust_Unnamed = -99;
pub const UV_EADDRINUSE: C2Rust_Unnamed = -98;
pub const UV_EACCES: C2Rust_Unnamed = -13;
pub const UV_E2BIG: C2Rust_Unnamed = -7;
pub const kVPosWinCol: VirtTextPos = 5;
pub const kVPosRightAlign: VirtTextPos = 4;
pub const kVPosOverlay: VirtTextPos = 3;
pub const kVPosInline: VirtTextPos = 2;
pub const kVPosEndOfLineRightAlign: VirtTextPos = 1;
pub const kVPosEndOfLine: VirtTextPos = 0;
pub const kCallbackLua: CallbackType = 3;
pub const kCallbackPartial: CallbackType = 2;
pub const kCallbackFuncref: CallbackType = 1;
pub const kCallbackNone: CallbackType = 0;
pub const VAR_DEF_SCOPE: ScopeType = 2;
pub const VAR_SCOPE: ScopeType = 1;
pub const VAR_NO_SCOPE: ScopeType = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_LOCKED: VarLockStatus = 1;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
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
pub const kStlClickFuncRun: C2Rust_Unnamed_12 = 3;
pub const kStlClickTabClose: C2Rust_Unnamed_12 = 2;
pub const kStlClickTabSwitch: C2Rust_Unnamed_12 = 1;
pub const kStlClickDisabled: C2Rust_Unnamed_12 = 0;
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
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub const MF_DIRTY_NO: mfdirty_T = 0;
pub const BACKWARD_FILE: Direction = -3;
pub const FORWARD_FILE: Direction = 3;
pub const BACKWARD: Direction = -1;
pub const FORWARD: Direction = 1;
pub const kDirectionNotSet: Direction = 0;
pub const XP_PREFIX_INV: xp_prefix_T = 2;
pub const XP_PREFIX_NO: xp_prefix_T = 1;
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const EXPAND_BUF_LEN: C2Rust_Unnamed_13 = 256;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_LSP: C2Rust_Unnamed_14 = 64;
pub const EXPAND_LUA: C2Rust_Unnamed_14 = 63;
pub const EXPAND_CHECKHEALTH: C2Rust_Unnamed_14 = 62;
pub const EXPAND_RETAB: C2Rust_Unnamed_14 = 61;
pub const EXPAND_PATTERN_IN_BUF: C2Rust_Unnamed_14 = 60;
pub const EXPAND_FILETYPECMD: C2Rust_Unnamed_14 = 59;
pub const EXPAND_FINDFUNC: C2Rust_Unnamed_14 = 58;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_14 = 57;
pub const EXPAND_DIRS_IN_CDPATH: C2Rust_Unnamed_14 = 56;
pub const EXPAND_KEYMAP: C2Rust_Unnamed_14 = 55;
pub const EXPAND_ARGOPT: C2Rust_Unnamed_14 = 54;
pub const EXPAND_SETTING_SUBTRACT: C2Rust_Unnamed_14 = 53;
pub const EXPAND_STRING_SETTING: C2Rust_Unnamed_14 = 52;
pub const EXPAND_RUNTIME: C2Rust_Unnamed_14 = 51;
pub const EXPAND_SCRIPTNAMES: C2Rust_Unnamed_14 = 50;
pub const EXPAND_BREAKPOINT: C2Rust_Unnamed_14 = 49;
pub const EXPAND_DIFF_BUFFERS: C2Rust_Unnamed_14 = 48;
pub const EXPAND_ARGLIST: C2Rust_Unnamed_14 = 47;
pub const EXPAND_MAPCLEAR: C2Rust_Unnamed_14 = 46;
pub const EXPAND_MESSAGES: C2Rust_Unnamed_14 = 45;
pub const EXPAND_PACKADD: C2Rust_Unnamed_14 = 44;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_14 = 43;
pub const EXPAND_SYNTIME: C2Rust_Unnamed_14 = 42;
pub const EXPAND_USER: C2Rust_Unnamed_14 = 41;
pub const EXPAND_HISTORY: C2Rust_Unnamed_14 = 40;
pub const EXPAND_LOCALES: C2Rust_Unnamed_14 = 39;
pub const EXPAND_OWNSYNTAX: C2Rust_Unnamed_14 = 38;
pub const EXPAND_FILES_IN_PATH: C2Rust_Unnamed_14 = 37;
pub const EXPAND_FILETYPE: C2Rust_Unnamed_14 = 36;
pub const EXPAND_PROFILE: C2Rust_Unnamed_14 = 35;
pub const EXPAND_SIGN: C2Rust_Unnamed_14 = 34;
pub const EXPAND_SHELLCMD: C2Rust_Unnamed_14 = 33;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_14 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_14 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_14 = 30;
pub const EXPAND_COMPILER: C2Rust_Unnamed_14 = 29;
pub const EXPAND_COLORS: C2Rust_Unnamed_14 = 28;
pub const EXPAND_LANGUAGE: C2Rust_Unnamed_14 = 27;
pub const EXPAND_ENV_VARS: C2Rust_Unnamed_14 = 26;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_14 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_14 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_14 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_14 = 22;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_EXPRESSION: C2Rust_Unnamed_14 = 20;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_14 = 19;
pub const EXPAND_FUNCTIONS: C2Rust_Unnamed_14 = 18;
pub const EXPAND_TAGS_LISTFILES: C2Rust_Unnamed_14 = 17;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_14 = 16;
pub const EXPAND_USER_VARS: C2Rust_Unnamed_14 = 15;
pub const EXPAND_AUGROUP: C2Rust_Unnamed_14 = 14;
pub const EXPAND_HIGHLIGHT: C2Rust_Unnamed_14 = 13;
pub const EXPAND_SYNTAX: C2Rust_Unnamed_14 = 12;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_EVENTS: C2Rust_Unnamed_14 = 10;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_14 = 9;
pub const EXPAND_HELP: C2Rust_Unnamed_14 = 8;
pub const EXPAND_OLD_SETTING: C2Rust_Unnamed_14 = 7;
pub const EXPAND_TAGS: C2Rust_Unnamed_14 = 6;
pub const EXPAND_BOOL_SETTINGS: C2Rust_Unnamed_14 = 5;
pub const EXPAND_SETTINGS: C2Rust_Unnamed_14 = 4;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_14 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_14 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_OK: C2Rust_Unnamed_14 = -1;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WILD_PUM_WANT: C2Rust_Unnamed_15 = 13;
pub const WILD_PAGEDOWN: C2Rust_Unnamed_15 = 12;
pub const WILD_PAGEUP: C2Rust_Unnamed_15 = 11;
pub const WILD_APPLY: C2Rust_Unnamed_15 = 10;
pub const WILD_CANCEL: C2Rust_Unnamed_15 = 9;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_15 = 8;
pub const WILD_LONGEST: C2Rust_Unnamed_15 = 7;
pub const WILD_ALL: C2Rust_Unnamed_15 = 6;
pub const WILD_PREV: C2Rust_Unnamed_15 = 5;
pub const WILD_NEXT: C2Rust_Unnamed_15 = 4;
pub const WILD_EXPAND_KEEP: C2Rust_Unnamed_15 = 3;
pub const WILD_EXPAND_FREE: C2Rust_Unnamed_15 = 2;
pub const WILD_FREE: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WILD_FUNC_TRIGGER: C2Rust_Unnamed_16 = 65536;
pub const WILD_MAY_EXPAND_PATTERN: C2Rust_Unnamed_16 = 32768;
pub const WILD_NOSELECT: C2Rust_Unnamed_16 = 16384;
pub const BUF_DIFF_FILTER: C2Rust_Unnamed_16 = 8192;
pub const WILD_BUFLASTUSED: C2Rust_Unnamed_16 = 4096;
pub const WILD_NOERROR: C2Rust_Unnamed_16 = 2048;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_16 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_16 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_16 = 256;
pub const WILD_ESCAPE: C2Rust_Unnamed_16 = 128;
pub const WILD_SILENT: C2Rust_Unnamed_16 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_16 = 32;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_16 = 16;
pub const WILD_NO_BEEP: C2Rust_Unnamed_16 = 8;
pub const WILD_USE_NL: C2Rust_Unnamed_16 = 4;
pub const WILD_HOME_REPLACE: C2Rust_Unnamed_16 = 2;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_16 = 1;
pub const VV_EXITREASON: VimVarIndex = 105;
pub const VV_STARTTIME: VimVarIndex = 104;
pub const VV_VIRTNUM: VimVarIndex = 103;
pub const VV_RELNUM: VimVarIndex = 102;
pub const VV_LUA: VimVarIndex = 101;
pub const VV__NULL_BLOB: VimVarIndex = 100;
pub const VV__NULL_DICT: VimVarIndex = 99;
pub const VV__NULL_LIST: VimVarIndex = 98;
pub const VV__NULL_STRING: VimVarIndex = 97;
pub const VV_MSGPACK_TYPES: VimVarIndex = 96;
pub const VV_STDERR: VimVarIndex = 95;
pub const VV_VIM_DID_INIT: VimVarIndex = 94;
pub const VV_STACKTRACE: VimVarIndex = 93;
pub const VV_MAXCOL: VimVarIndex = 92;
pub const VV_EXITING: VimVarIndex = 91;
pub const VV_COLLATE: VimVarIndex = 90;
pub const VV_ARGV: VimVarIndex = 89;
pub const VV_ARGF: VimVarIndex = 88;
pub const VV_ECHOSPACE: VimVarIndex = 87;
pub const VV_VERSIONLONG: VimVarIndex = 86;
pub const VV_EVENT: VimVarIndex = 85;
pub const VV_TYPE_BLOB: VimVarIndex = 84;
pub const VV_TYPE_BOOL: VimVarIndex = 83;
pub const VV_TYPE_FLOAT: VimVarIndex = 82;
pub const VV_TYPE_DICT: VimVarIndex = 81;
pub const VV_TYPE_LIST: VimVarIndex = 80;
pub const VV_TYPE_FUNC: VimVarIndex = 79;
pub const VV_TYPE_STRING: VimVarIndex = 78;
pub const VV_TYPE_NUMBER: VimVarIndex = 77;
pub const VV_TESTING: VimVarIndex = 76;
pub const VV_VIM_DID_ENTER: VimVarIndex = 75;
pub const VV_NUMBERSIZE: VimVarIndex = 74;
pub const VV_NUMBERMIN: VimVarIndex = 73;
pub const VV_NUMBERMAX: VimVarIndex = 72;
pub const VV_NULL: VimVarIndex = 71;
pub const VV_TRUE: VimVarIndex = 70;
pub const VV_FALSE: VimVarIndex = 69;
pub const VV_ERRORS: VimVarIndex = 68;
pub const VV_OPTION_TYPE: VimVarIndex = 67;
pub const VV_OPTION_COMMAND: VimVarIndex = 66;
pub const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
pub const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
pub const VV_OPTION_OLD: VimVarIndex = 63;
pub const VV_OPTION_NEW: VimVarIndex = 62;
pub const VV_COMPLETED_ITEM: VimVarIndex = 61;
pub const VV_PROGPATH: VimVarIndex = 60;
pub const VV_WINDOWID: VimVarIndex = 59;
pub const VV_OLDFILES: VimVarIndex = 58;
pub const VV_HLSEARCH: VimVarIndex = 57;
pub const VV_SEARCHFORWARD: VimVarIndex = 56;
pub const VV_OP: VimVarIndex = 55;
pub const VV_MOUSE_COL: VimVarIndex = 54;
pub const VV_MOUSE_LNUM: VimVarIndex = 53;
pub const VV_MOUSE_WINID: VimVarIndex = 52;
pub const VV_MOUSE_WIN: VimVarIndex = 51;
pub const VV_CHAR: VimVarIndex = 50;
pub const VV_SWAPCOMMAND: VimVarIndex = 49;
pub const VV_SWAPCHOICE: VimVarIndex = 48;
pub const VV_SWAPNAME: VimVarIndex = 47;
pub const VV_SCROLLSTART: VimVarIndex = 46;
pub const VV_BEVAL_TEXT: VimVarIndex = 45;
pub const VV_BEVAL_COL: VimVarIndex = 44;
pub const VV_BEVAL_LNUM: VimVarIndex = 43;
pub const VV_BEVAL_WINID: VimVarIndex = 42;
pub const VV_BEVAL_WINNR: VimVarIndex = 41;
pub const VV_BEVAL_BUFNR: VimVarIndex = 40;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_PROFILING: VimVarIndex = 37;
pub const VV_KEY: VimVarIndex = 36;
pub const VV_VAL: VimVarIndex = 35;
pub const VV_INSERTMODE: VimVarIndex = 34;
pub const VV_CMDBANG: VimVarIndex = 33;
pub const VV_REG: VimVarIndex = 32;
pub const VV_THROWPOINT: VimVarIndex = 31;
pub const VV_EXCEPTION: VimVarIndex = 30;
pub const VV_DYING: VimVarIndex = 29;
pub const VV_SEND_SERVER: VimVarIndex = 28;
pub const VV_PROGNAME: VimVarIndex = 27;
pub const VV_FOLDLEVEL: VimVarIndex = 26;
pub const VV_FOLDDASHES: VimVarIndex = 25;
pub const VV_FOLDEND: VimVarIndex = 24;
pub const VV_FOLDSTART: VimVarIndex = 23;
pub const VV_CMDARG: VimVarIndex = 22;
pub const VV_FNAME_DIFF: VimVarIndex = 21;
pub const VV_FNAME_NEW: VimVarIndex = 20;
pub const VV_FNAME_OUT: VimVarIndex = 19;
pub const VV_FNAME_IN: VimVarIndex = 18;
pub const VV_CC_TO: VimVarIndex = 17;
pub const VV_CC_FROM: VimVarIndex = 16;
pub const VV_CTYPE: VimVarIndex = 15;
pub const VV_LC_TIME: VimVarIndex = 14;
pub const VV_LANG: VimVarIndex = 13;
pub const VV_FNAME: VimVarIndex = 12;
pub const VV_TERMRESPONSE: VimVarIndex = 11;
pub const VV_TERMREQUEST: VimVarIndex = 10;
pub const VV_LNUM: VimVarIndex = 9;
pub const VV_VERSION: VimVarIndex = 8;
pub const VV_THIS_SESSION: VimVarIndex = 7;
pub const VV_SHELL_ERROR: VimVarIndex = 6;
pub const VV_STATUSMSG: VimVarIndex = 5;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub const VV_ERRMSG: VimVarIndex = 3;
pub const VV_PREVCOUNT: VimVarIndex = 2;
pub const VV_COUNT1: VimVarIndex = 1;
pub const VV_COUNT: VimVarIndex = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct utsname {
    pub sysname: [::core::ffi::c_char; 65],
    pub nodename: [::core::ffi::c_char; 65],
    pub release: [::core::ffi::c_char; 65],
    pub version: [::core::ffi::c_char; 65],
    pub machine: [::core::ffi::c_char; 65],
    pub domainname: [::core::ffi::c_char; 65],
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const RUNTIME_DIRNAME: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"runtime\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LOGLVL_ERR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub static default_vim_dir: GlobalCell<*mut ::core::ffi::c_char> = GlobalCell::new(
    b"/usr/local/share/nvim\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
);
pub static default_vimruntime_dir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(concat!(env!("NVIM_DEFAULT_VIMRUNTIME_DIR"), "\0").as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
pub static default_lib_dir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(concat!(env!("NVIM_DEFAULT_LIB_DIR"), "\0").as_ptr()
        as *const ::core::ffi::c_char as *mut ::core::ffi::c_char);
pub unsafe extern "C" fn env_init() {
    nvim_testing.set(os_env_exists(
        b"NVIM_TEST\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    ));
}
#[no_mangle]
pub unsafe extern "C" fn os_getenv(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut size: size_t = INIT_SIZE as size_t;
    let mut buf: [::core::ffi::c_char; 64] = [0; 64];
    r = uv_os_getenv(
        name,
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut size,
    );
    if r == UV_ENOBUFS as ::core::ffi::c_int {
        e = xmalloc(size) as *mut ::core::ffi::c_char;
        r = uv_os_getenv(name, e, &raw mut size);
        if r != 0 as ::core::ffi::c_int
            || size == 0 as size_t
            || *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut e as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
    } else if r != 0 as ::core::ffi::c_int
        || size == 0 as size_t
        || buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
    {
        e = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        e = xmemdupz(
            &raw mut buf as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            size,
        ) as *mut ::core::ffi::c_char;
    }
    if r != 0 as ::core::ffi::c_int
        && r != UV_ENOENT as ::core::ffi::c_int
        && r != UV_UNKNOWN as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_getenv\0".as_ptr() as *const ::core::ffi::c_char,
            98 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return e;
}
pub const INIT_SIZE: ::core::ffi::c_int = 64 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn os_getenv_buf(
    name: *const ::core::ffi::c_char,
    buf: *mut ::core::ffi::c_char,
    bufsize: size_t,
) -> *mut ::core::ffi::c_char {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut size: size_t = bufsize;
    let mut r: ::core::ffi::c_int = uv_os_getenv(name, buf, &raw mut size);
    if r == UV_ENOBUFS as ::core::ffi::c_int {
        let mut e: *mut ::core::ffi::c_char = xmalloc(size) as *mut ::core::ffi::c_char;
        r = uv_os_getenv(name, e, &raw mut size);
        if r == 0 as ::core::ffi::c_int
            && size != 0 as size_t
            && *e.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            xmemcpyz(
                buf as *mut ::core::ffi::c_void,
                e as *const ::core::ffi::c_void,
                (if bufsize < size { bufsize } else { size }).wrapping_sub(1 as size_t),
            );
        }
        xfree(e as *mut ::core::ffi::c_void);
    }
    if r != 0 as ::core::ffi::c_int
        || size == 0 as size_t
        || *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        if r != 0 as ::core::ffi::c_int
            && r != UV_ENOENT as ::core::ffi::c_int
            && r != UV_UNKNOWN as ::core::ffi::c_int
        {
            logmsg(
                LOGLVL_ERR,
                ::core::ptr::null::<::core::ffi::c_char>(),
                b"os_getenv_buf\0".as_ptr() as *const ::core::ffi::c_char,
                129 as ::core::ffi::c_int,
                true_0 != 0,
                b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
                r,
                uv_err_name(r),
            );
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return buf;
}
#[no_mangle]
pub unsafe extern "C" fn os_getenv_noalloc(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return os_getenv_buf(
        name,
        NameBuff.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn os_env_exists(
    mut name: *const ::core::ffi::c_char,
    mut nonempty: bool,
) -> bool {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    let mut buf: [::core::ffi::c_char; 2] = [0; 2];
    let mut size: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 2]>();
    let mut r: ::core::ffi::c_int = uv_os_getenv(
        name,
        &raw mut buf as *mut ::core::ffi::c_char,
        &raw mut size,
    );
    '_c2rust_label: {
        if r != UV_EINVAL as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"r != UV_EINVAL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                165 as ::core::ffi::c_uint,
                b"_Bool os_env_exists(const char *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if r != 0 as ::core::ffi::c_int
        && r != UV_ENOENT as ::core::ffi::c_int
        && r != UV_ENOBUFS as ::core::ffi::c_int
    {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_env_exists\0".as_ptr() as *const ::core::ffi::c_char,
            167 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_getenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return r == 0 as ::core::ffi::c_int && (!nonempty || size > 0 as size_t)
        || r == UV_ENOBUFS as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn os_setenv(
    mut name: *const ::core::ffi::c_char,
    mut value: *const ::core::ffi::c_char,
    mut overwrite: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    if overwrite == 0 && os_env_exists(name, false_0 != 0) as ::core::ffi::c_int != 0 {
        return 0 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = 0;
    r = uv_os_setenv(name, value);
    '_c2rust_label: {
        if r != UV_EINVAL as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"r != UV_EINVAL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                204 as ::core::ffi::c_uint,
                b"int os_setenv(const char *, const char *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if r != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_setenv\0".as_ptr() as *const ::core::ffi::c_char,
            206 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_setenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return if r == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
#[no_mangle]
pub unsafe extern "C" fn os_unsetenv(mut name: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
        return -1 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = uv_os_unsetenv(name);
    if r != 0 as ::core::ffi::c_int {
        logmsg(
            LOGLVL_ERR,
            ::core::ptr::null::<::core::ffi::c_char>(),
            b"os_unsetenv\0".as_ptr() as *const ::core::ffi::c_char,
            220 as ::core::ffi::c_int,
            true_0 != 0,
            b"uv_os_unsetenv(%s) failed: %d %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
            r,
            uv_err_name(r),
        );
    }
    return if r == 0 as ::core::ffi::c_int {
        0 as ::core::ffi::c_int
    } else {
        -1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn os_get_fullenv_size() -> size_t {
    let mut len: size_t = 0 as size_t;
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    while !(*environ.offset(len as isize)).is_null() {
        len = len.wrapping_add(1);
    }
    return len;
}
pub unsafe extern "C" fn os_free_fullenv(mut env: *mut *mut ::core::ffi::c_char) {
    if env.is_null() {
        return;
    }
    let mut it: *mut *mut ::core::ffi::c_char = env;
    while !(*it).is_null() {
        let mut ptr_: *mut *mut ::core::ffi::c_void = it as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        it = it.offset(1);
    }
    xfree(env as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn os_copy_fullenv(
    mut env: *mut *mut ::core::ffi::c_char,
    mut env_size: size_t,
) {
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i < env_size && !(*environ.offset(i as isize)).is_null() {
        *env.offset(i as isize) = xstrdup(*environ.offset(i as isize));
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn os_getenvname_at_index(mut index: size_t) -> *mut ::core::ffi::c_char {
    extern "C" {
        #[link_name = "environ"]
        static mut environ_0: *mut *mut ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i <= index {
        if (*environ.offset(i as isize)).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        i = i.wrapping_add(1);
    }
    let mut str: *mut ::core::ffi::c_char = *environ.offset(index as isize);
    '_c2rust_label: {
        if !str.is_null() {
        } else {
            __assert_fail(
                b"str != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                375 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let end: *const ::core::ffi::c_char = strchr(str, '=' as ::core::ffi::c_int);
    '_c2rust_label_0: {
        if !end.is_null() {
        } else {
            __assert_fail(
                b"end != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                377 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut len: ptrdiff_t = end.offset_from(str);
    '_c2rust_label_1: {
        if len > 0 as ptrdiff_t {
        } else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                379 as ::core::ffi::c_uint,
                b"char *os_getenvname_at_index(size_t)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return xmemdupz(str as *const ::core::ffi::c_void, len as size_t) as *mut ::core::ffi::c_char;
}
#[no_mangle]
pub unsafe extern "C" fn os_get_pid() -> int64_t {
    return getpid() as int64_t;
}
pub unsafe extern "C" fn os_hint_priority() {}
#[no_mangle]
pub unsafe extern "C" fn os_get_hostname(mut hostname: *mut ::core::ffi::c_char, mut size: size_t) {
    let mut vutsname: utsname = utsname {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    if uname(&raw mut vutsname) < 0 as ::core::ffi::c_int {
        *hostname = NUL as ::core::ffi::c_char;
    } else {
        xstrlcpy(
            hostname,
            &raw mut vutsname.nodename as *mut ::core::ffi::c_char,
            size,
        );
    };
}
static homedir: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
pub unsafe extern "C" fn init_homedir() {
    xfree(homedir.get() as *mut ::core::ffi::c_void);
    homedir.set(::core::ptr::null_mut::<::core::ffi::c_char>());
    let mut var: *mut ::core::ffi::c_char =
        os_getenv(b"HOME\0".as_ptr() as *const ::core::ffi::c_char);
    let mut tofree: *mut ::core::ffi::c_char = var;
    if var.is_null() {
        var = os_uv_homedir();
    }
    if !var.is_null()
        && !os_realpath(
            var,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
        )
        .is_null()
    {
        var = IObuff.ptr() as *mut ::core::ffi::c_char;
    }
    if (var.is_null() || *var as ::core::ffi::c_int == NUL)
        && os_dirname(
            os_buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) == OK
    {
        var = os_buf.ptr() as *mut ::core::ffi::c_char;
    }
    if !var.is_null() {
        homedir.set(xstrdup(var));
    }
    xfree(tofree as *mut ::core::ffi::c_void);
}
static homedir_buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
unsafe extern "C" fn os_uv_homedir() -> *mut ::core::ffi::c_char {
    (*homedir_buf.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut homedir_size: size_t = MAXPATHL as size_t;
    let mut ret_value: ::core::ffi::c_int = uv_os_homedir(
        homedir_buf.ptr() as *mut ::core::ffi::c_char,
        &raw mut homedir_size,
    );
    if ret_value == 0 as ::core::ffi::c_int && homedir_size < MAXPATHL as size_t {
        return homedir_buf.ptr() as *mut ::core::ffi::c_char;
    }
    logmsg(
        LOGLVL_ERR,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"os_uv_homedir\0".as_ptr() as *const ::core::ffi::c_char,
        570 as ::core::ffi::c_int,
        true_0 != 0,
        b"uv_os_homedir() failed %d: %s\0".as_ptr() as *const ::core::ffi::c_char,
        ret_value,
        uv_strerror(ret_value),
    );
    (*homedir_buf.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn expand_env_save(
    mut src: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return expand_env_save_opt(src, false_0 != 0);
}
pub unsafe extern "C" fn expand_env_save_opt(
    mut src: *mut ::core::ffi::c_char,
    mut one: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    expand_env_esc(
        src,
        p,
        MAXPATHL,
        false_0 != 0,
        one,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
    return p;
}
pub unsafe extern "C" fn expand_env(
    mut src: *mut ::core::ffi::c_char,
    mut dst: *mut ::core::ffi::c_char,
    mut dstlen: ::core::ffi::c_int,
) -> size_t {
    return expand_env_esc(
        src,
        dst,
        dstlen,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn expand_env_esc(
    mut srcp: *const ::core::ffi::c_char,
    mut dst: *mut ::core::ffi::c_char,
    mut dstlen: ::core::ffi::c_int,
    mut esc: bool,
    mut one: bool,
    mut prefix: *mut ::core::ffi::c_char,
) -> size_t {
    let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut var: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut copy_char: bool = false;
    let mut mustfree: bool = false;
    let mut at_start: bool = true_0 != 0;
    let dst_start: *mut ::core::ffi::c_char = dst;
    let mut prefix_len: ::core::ffi::c_int = if prefix.is_null() {
        0 as ::core::ffi::c_int
    } else {
        strlen(prefix) as ::core::ffi::c_int
    };
    let mut src: *mut ::core::ffi::c_char = skipwhite(srcp);
    dstlen -= 1;
    while *src as ::core::ffi::c_int != 0 && dstlen > 0 as ::core::ffi::c_int {
        if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '`' as ::core::ffi::c_int
            && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            var = src;
            src = src.offset(2 as ::core::ffi::c_int as isize);
            skip_expr(&raw mut src, ::core::ptr::null_mut::<evalarg_T>());
            if *src as ::core::ffi::c_int == '`' as ::core::ffi::c_int {
                src = src.offset(1);
            }
            let mut len: size_t = src.offset_from(var) as size_t;
            if len > dstlen as size_t {
                len = dstlen as size_t;
            }
            memcpy(
                dst as *mut ::core::ffi::c_void,
                var as *const ::core::ffi::c_void,
                len,
            );
            dst = dst.offset(len as isize);
            dstlen -= len as ::core::ffi::c_int;
        } else {
            copy_char = true_0 != 0;
            if *src as ::core::ffi::c_int == '$' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == '~' as ::core::ffi::c_int
                    && at_start as ::core::ffi::c_int != 0
            {
                mustfree = false_0 != 0;
                if *src as ::core::ffi::c_int != '~' as ::core::ffi::c_int {
                    tail = src.offset(1 as ::core::ffi::c_int as isize);
                    var = dst;
                    let mut c: ::core::ffi::c_int = dstlen - 1 as ::core::ffi::c_int;
                    if *tail as ::core::ffi::c_int == '{' as ::core::ffi::c_int
                        && !vim_isIDc('{' as ::core::ffi::c_int)
                    {
                        tail = tail.offset(1);
                        loop {
                            let c2rust_fresh0 = c;
                            c = c - 1;
                            if !(c2rust_fresh0 > 0 as ::core::ffi::c_int
                                && *tail as ::core::ffi::c_int != NUL
                                && *tail as ::core::ffi::c_int != '}' as ::core::ffi::c_int)
                            {
                                break;
                            }
                            let c2rust_fresh1 = tail;
                            tail = tail.offset(1);
                            let c2rust_fresh2 = var;
                            var = var.offset(1);
                            *c2rust_fresh2 = *c2rust_fresh1;
                        }
                    } else {
                        loop {
                            let c2rust_fresh3 = c;
                            c = c - 1;
                            if !(c2rust_fresh3 > 0 as ::core::ffi::c_int
                                && *tail as ::core::ffi::c_int != NUL
                                && vim_isIDc(*tail as uint8_t as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0)
                            {
                                break;
                            }
                            let c2rust_fresh4 = tail;
                            tail = tail.offset(1);
                            let c2rust_fresh5 = var;
                            var = var.offset(1);
                            *c2rust_fresh5 = *c2rust_fresh4;
                        }
                    }
                    if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '{' as ::core::ffi::c_int
                        && *tail as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                    {
                        var = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else {
                        if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '{' as ::core::ffi::c_int
                        {
                            tail = tail.offset(1);
                        }
                        *var = NUL as ::core::ffi::c_char;
                        var = vim_getenv(dst);
                        mustfree = true_0 != 0;
                    }
                } else if *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || vim_ispathsep(
                        *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0
                    || !vim_strchr(
                        b" ,\t\n\0".as_ptr() as *const ::core::ffi::c_char,
                        *src.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null()
                {
                    var = homedir.get();
                    tail = src.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    tail = src;
                    var = dst;
                    let mut c_0: ::core::ffi::c_int = dstlen - 1 as ::core::ffi::c_int;
                    loop {
                        let c2rust_fresh6 = c_0;
                        c_0 = c_0 - 1;
                        if !(c2rust_fresh6 > 0 as ::core::ffi::c_int
                            && *tail as ::core::ffi::c_int != 0
                            && vim_isfilec(*tail as uint8_t as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                                != 0
                            && !vim_ispathsep(*tail as ::core::ffi::c_int))
                        {
                            break;
                        }
                        let c2rust_fresh7 = tail;
                        tail = tail.offset(1);
                        let c2rust_fresh8 = var;
                        var = var.offset(1);
                        *c2rust_fresh8 = *c2rust_fresh7;
                    }
                    *var = NUL as ::core::ffi::c_char;
                    var = if *dst as ::core::ffi::c_int == NUL {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    } else {
                        os_get_userdir(dst.offset(1 as ::core::ffi::c_int as isize))
                    };
                    mustfree = true_0 != 0;
                    if var.is_null() {
                        let mut xpc: expand_T = expand_T {
                            xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_context: 0,
                            xp_pattern_len: 0,
                            xp_prefix: XP_PREFIX_NONE,
                            xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_luaref: 0,
                            xp_script_ctx: sctx_T {
                                sc_sid: 0,
                                sc_seq: 0,
                                sc_lnum: 0,
                                sc_chan: 0,
                            },
                            xp_backslash: 0,
                            xp_shell: false,
                            xp_numfiles: 0,
                            xp_col: 0,
                            xp_selected: 0,
                            xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            xp_buf: [0; 256],
                            xp_search_dir: kDirectionNotSet,
                            xp_pre_incsearch_pos: pos_T {
                                lnum: 0,
                                col: 0,
                                coladd: 0,
                            },
                        };
                        ExpandInit(&raw mut xpc);
                        xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
                        var = ExpandOne(
                            &raw mut xpc,
                            dst,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            WILD_ADD_SLASH as ::core::ffi::c_int
                                | WILD_SILENT as ::core::ffi::c_int,
                            WILD_EXPAND_FREE as ::core::ffi::c_int,
                        );
                        mustfree = true_0 != 0;
                    }
                }
                if esc as ::core::ffi::c_int != 0
                    && !var.is_null()
                    && !strpbrk(var, b" \t\0".as_ptr() as *const ::core::ffi::c_char).is_null()
                {
                    let mut p: *mut ::core::ffi::c_char =
                        vim_strsave_escaped(var, b" \t\0".as_ptr() as *const ::core::ffi::c_char);
                    if mustfree {
                        xfree(var as *mut ::core::ffi::c_void);
                    }
                    var = p;
                    mustfree = true_0 != 0;
                }
                if !var.is_null() && *var as ::core::ffi::c_int != NUL {
                    let mut c_1: ::core::ffi::c_int = strlen(var) as ::core::ffi::c_int;
                    if (c_1 as size_t)
                        .wrapping_add(strlen(tail))
                        .wrapping_add(1 as size_t)
                        < dstlen as ::core::ffi::c_uint as size_t
                    {
                        strcpy(dst, var);
                        dstlen -= c_1;
                        if after_pathsep(dst, dst.offset(c_1 as isize)) != 0
                            && vim_ispathsep(*tail as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            tail = tail.offset(1);
                        }
                        dst = dst.offset(c_1 as isize);
                        src = tail;
                        copy_char = false_0 != 0;
                    }
                }
                if mustfree {
                    xfree(var as *mut ::core::ffi::c_void);
                }
            }
            if copy_char {
                at_start = false_0 != 0;
                if *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && *src.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
                {
                    let c2rust_fresh9 = src;
                    src = src.offset(1);
                    let c2rust_fresh10 = dst;
                    dst = dst.offset(1);
                    *c2rust_fresh10 = *c2rust_fresh9;
                    dstlen -= 1;
                } else if (*src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ' ' as ::core::ffi::c_int
                    || *src.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == ',' as ::core::ffi::c_int)
                    && !one
                {
                    at_start = true_0 != 0;
                }
                if dstlen > 0 as ::core::ffi::c_int {
                    let c2rust_fresh11 = src;
                    src = src.offset(1);
                    let c2rust_fresh12 = dst;
                    dst = dst.offset(1);
                    *c2rust_fresh12 = *c2rust_fresh11;
                    dstlen -= 1;
                    if !prefix.is_null()
                        && src.offset(-(prefix_len as isize)) >= srcp as *mut ::core::ffi::c_char
                        && strncmp(
                            src.offset(-(prefix_len as isize)),
                            prefix,
                            prefix_len as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        at_start = true_0 != 0;
                    }
                }
            }
        }
    }
    *dst = NUL as ::core::ffi::c_char;
    return dst.offset_from(dst_start) as size_t;
}
unsafe extern "C" fn vim_runtime_dir(
    mut vimdir: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if vimdir.is_null() || *vimdir as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut p: *mut ::core::ffi::c_char =
        concat_fnames(vimdir, RUNTIME_DIRNAME.as_ptr(), true_0 != 0);
    if os_isdir(p) {
        return p;
    }
    xfree(p as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn remove_tail(
    mut path: *mut ::core::ffi::c_char,
    mut pend: *mut ::core::ffi::c_char,
    mut dirname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(dirname);
    let mut new_tail: *mut ::core::ffi::c_char = pend
        .offset(-(len as isize))
        .offset(-(1 as ::core::ffi::c_int as isize));
    if new_tail >= path
        && path_fnamencmp(new_tail, dirname, len) == 0 as ::core::ffi::c_int
        && (new_tail == path || after_pathsep(path, new_tail) != 0)
    {
        return new_tail;
    }
    return pend;
}
pub unsafe extern "C" fn vim_env_iter(
    delim: ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    iter: *const ::core::ffi::c_void,
    dir: *mut *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_void {
    let mut varval: *const ::core::ffi::c_char = iter as *const ::core::ffi::c_char;
    if varval.is_null() {
        varval = val;
    }
    *dir = varval;
    let dirend: *const ::core::ffi::c_char = strchr(varval, delim as ::core::ffi::c_int);
    if dirend.is_null() {
        *len = strlen(varval);
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *len = dirend.offset_from(varval) as size_t;
    return dirend.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn vim_env_iter_rev(
    delim: ::core::ffi::c_char,
    val: *const ::core::ffi::c_char,
    iter: *const ::core::ffi::c_void,
    dir: *mut *const ::core::ffi::c_char,
    len: *mut size_t,
) -> *const ::core::ffi::c_void {
    let mut varend: *const ::core::ffi::c_char = iter as *const ::core::ffi::c_char;
    if varend.is_null() {
        varend = val
            .offset(strlen(val) as isize)
            .offset(-(1 as ::core::ffi::c_int as isize));
    }
    let varlen: size_t = (varend.offset_from(val) as size_t).wrapping_add(1 as size_t);
    let colon: *const ::core::ffi::c_char =
        xmemrchr(val as *const ::core::ffi::c_void, delim as uint8_t, varlen)
            as *const ::core::ffi::c_char;
    if colon.is_null() {
        *len = varlen;
        *dir = val;
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    *dir = colon.offset(1 as ::core::ffi::c_int as isize);
    *len = varend.offset_from(colon) as size_t;
    return colon.offset(-(1 as ::core::ffi::c_int as isize)) as *const ::core::ffi::c_void;
}
pub unsafe extern "C" fn vim_get_prefix_from_exepath(mut exe_name: *mut ::core::ffi::c_char) {
    xstrlcpy(
        exe_name,
        get_vim_var_str(VV_PROGPATH),
        (MAXPATHL as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
    );
    let mut path_end: *mut ::core::ffi::c_char = path_tail_with_sep(exe_name);
    *path_end = NUL as ::core::ffi::c_char;
    path_end = path_tail(exe_name);
    *path_end = NUL as ::core::ffi::c_char;
}
pub unsafe extern "C" fn vim_getenv(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if *get_vim_var_str(VV_PROGPATH).offset(0 as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            != '\0' as ::core::ffi::c_int
        {
        } else {
            __assert_fail(
                b"get_vim_var_str(VV_PROGPATH)[0] != NUL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                956 as ::core::ffi::c_uint,
                b"char *vim_getenv(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut kos_env_path: *mut ::core::ffi::c_char = os_getenv(name);
    if !kos_env_path.is_null() {
        return kos_env_path;
    }
    let mut vimruntime: bool = strcmp(name, b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char)
        == 0 as ::core::ffi::c_int;
    if !vimruntime
        && strcmp(name, b"VIM\0".as_ptr() as *const ::core::ffi::c_char) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut vim_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if vimruntime as ::core::ffi::c_int != 0
        && *default_vimruntime_dir.get() as ::core::ffi::c_int == NUL
    {
        kos_env_path = os_getenv(b"VIM\0".as_ptr() as *const ::core::ffi::c_char);
        if !kos_env_path.is_null() {
            vim_path = vim_runtime_dir(kos_env_path);
            if vim_path.is_null() {
                vim_path = kos_env_path;
            } else {
                xfree(kos_env_path as *mut ::core::ffi::c_void);
            }
        }
    }
    if vim_path.is_null() {
        if !(*p_hf.ptr()).is_null() && vim_strchr(p_hf.get(), '$' as ::core::ffi::c_int).is_null() {
            vim_path = p_hf.get();
        }
        let mut exe_name: [::core::ffi::c_char; 4096] = [0; 4096];
        if vim_path.is_null() {
            vim_get_prefix_from_exepath(&raw mut exe_name as *mut ::core::ffi::c_char);
            if append_path(
                &raw mut exe_name as *mut ::core::ffi::c_char,
                b"share/nvim/runtime/\0".as_ptr() as *const ::core::ffi::c_char,
                MAXPATHL as size_t,
            ) == OK
            {
                vim_path = &raw mut exe_name as *mut ::core::ffi::c_char;
            }
        }
        if !vim_path.is_null() {
            let mut vim_path_end: *mut ::core::ffi::c_char = path_tail(vim_path);
            if vim_path == p_hf.get() {
                vim_path_end = remove_tail(
                    vim_path,
                    vim_path_end,
                    b"doc\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
            if !vimruntime {
                vim_path_end = remove_tail(
                    vim_path,
                    vim_path_end,
                    RUNTIME_DIRNAME.as_ptr() as *mut ::core::ffi::c_char,
                );
            }
            if vim_path_end > vim_path && after_pathsep(vim_path, vim_path_end) != 0 {
                vim_path_end = vim_path_end.offset(-1);
            }
            '_c2rust_label_0: {
                if vim_path_end >= vim_path {
                } else {
                    __assert_fail(
                        b"vim_path_end >= vim_path\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1027 as ::core::ffi::c_uint,
                        b"char *vim_getenv(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            vim_path = xmemdupz(
                vim_path as *const ::core::ffi::c_void,
                vim_path_end.offset_from(vim_path) as size_t,
            ) as *mut ::core::ffi::c_char;
            if !os_isdir(vim_path) {
                xfree(vim_path as *mut ::core::ffi::c_void);
                vim_path = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        }
        '_c2rust_label_1: {
            if vim_path != &raw mut exe_name as *mut ::core::ffi::c_char {
            } else {
                __assert_fail(
                    b"vim_path != exe_name\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1035 as ::core::ffi::c_uint,
                    b"char *vim_getenv(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
    }
    if vim_path.is_null() {
        if vimruntime as ::core::ffi::c_int != 0
            && *default_vimruntime_dir.get() as ::core::ffi::c_int != NUL
        {
            vim_path = xstrdup(default_vimruntime_dir.get());
        } else if *default_vim_dir.get() as ::core::ffi::c_int != NUL {
            if vimruntime as ::core::ffi::c_int != 0 && {
                vim_path = vim_runtime_dir(default_vim_dir.get());
                vim_path.is_null()
            } {
                vim_path = xstrdup(default_vim_dir.get());
            }
        }
    }
    if !vim_path.is_null() {
        if vimruntime {
            os_setenv(
                b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char,
                vim_path,
                1 as ::core::ffi::c_int,
            );
            didset_vimruntime.set(true_0 != 0);
        } else {
            os_setenv(
                b"VIM\0".as_ptr() as *const ::core::ffi::c_char,
                vim_path,
                1 as ::core::ffi::c_int,
            );
            didset_vim.set(true_0 != 0);
        }
    }
    return vim_path;
}
pub unsafe extern "C" fn home_replace(
    buf: *const buf_T,
    mut src: *const ::core::ffi::c_char,
    dst: *mut ::core::ffi::c_char,
    mut dstlen: size_t,
    one: bool,
) -> size_t {
    let mut dirlen: size_t = 0 as size_t;
    let mut envlen: size_t = 0 as size_t;
    if src.is_null() {
        *dst = NUL as ::core::ffi::c_char;
        return 0 as size_t;
    }
    if !buf.is_null() && (*buf).b_help as ::core::ffi::c_int != 0 {
        let dlen: size_t = xstrlcpy(dst, path_tail(src), dstlen);
        return if dlen < dstlen.wrapping_sub(1 as size_t) {
            dlen
        } else {
            dstlen.wrapping_sub(1 as size_t)
        };
    }
    if !(*homedir.ptr()).is_null() {
        dirlen = strlen(homedir.get());
    }
    let mut homedir_env: *mut ::core::ffi::c_char =
        os_getenv(b"HOME\0".as_ptr() as *const ::core::ffi::c_char);
    let mut homedir_env_mod: *mut ::core::ffi::c_char = homedir_env;
    let mut must_free: bool = false_0 != 0;
    if !homedir_env_mod.is_null()
        && *homedir_env_mod as ::core::ffi::c_int == '~' as ::core::ffi::c_int
    {
        must_free = true_0 != 0;
        let mut usedlen: size_t = 0 as size_t;
        let mut flen: size_t = strlen(homedir_env_mod);
        let mut fbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        modify_fname(
            b":p\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            false_0 != 0,
            &raw mut usedlen,
            &raw mut homedir_env_mod,
            &raw mut fbuf,
            &raw mut flen,
        );
        flen = strlen(homedir_env_mod);
        '_c2rust_label: {
            if homedir_env_mod != homedir_env {
            } else {
                __assert_fail(
                    b"homedir_env_mod != homedir_env\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    b"src/nvim/os/env.rs\0".as_ptr()
                        as *const ::core::ffi::c_char,
                    1123 as ::core::ffi::c_uint,
                    b"size_t home_replace(const buf_T *const, const char *, char *const, size_t, const _Bool)\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if vim_ispathsep(
            *homedir_env_mod.offset(flen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        ) {
            *homedir_env_mod.offset(flen.wrapping_sub(1 as size_t) as isize) =
                NUL as ::core::ffi::c_char;
        }
    }
    if !homedir_env_mod.is_null() {
        envlen = strlen(homedir_env_mod);
    }
    if !one {
        src = skipwhite(src);
    }
    let mut dst_p: *mut ::core::ffi::c_char = dst;
    while *src as ::core::ffi::c_int != 0 && dstlen > 0 as size_t {
        let mut p: *mut ::core::ffi::c_char = homedir.get();
        let mut len: size_t = dirlen;
        loop {
            if len != 0
                && path_fnamencmp(src, p, len) == 0 as ::core::ffi::c_int
                && (vim_ispathsep(*src.offset(len as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
                    || !one
                        && (*src.offset(len as isize) as ::core::ffi::c_int
                            == ',' as ::core::ffi::c_int
                            || *src.offset(len as isize) as ::core::ffi::c_int
                                == ' ' as ::core::ffi::c_int)
                    || *src.offset(len as isize) as ::core::ffi::c_int == NUL)
            {
                src = src.offset(len as isize);
                dstlen = dstlen.wrapping_sub(1);
                if dstlen > 0 as size_t {
                    let c2rust_fresh13 = dst_p;
                    dst_p = dst_p.offset(1);
                    *c2rust_fresh13 = '~' as ::core::ffi::c_char;
                }
                break;
            } else {
                if p == homedir_env_mod {
                    break;
                }
                p = homedir_env_mod;
                len = envlen;
            }
        }
        if dstlen == 0 as size_t {
            break;
        } else {
            while *src as ::core::ffi::c_int != 0
                && (one as ::core::ffi::c_int != 0
                    || *src as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                        && *src as ::core::ffi::c_int != ' ' as ::core::ffi::c_int)
                && {
                    dstlen = dstlen.wrapping_sub(1);
                    dstlen > 0 as size_t
                }
            {
                let c2rust_fresh14 = src;
                src = src.offset(1);
                let c2rust_fresh15 = dst_p;
                dst_p = dst_p.offset(1);
                *c2rust_fresh15 = *c2rust_fresh14;
            }
            if dstlen == 0 as size_t {
                break;
            }
            while (*src as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                || *src as ::core::ffi::c_int == ',' as ::core::ffi::c_int)
                && {
                    dstlen = dstlen.wrapping_sub(1);
                    dstlen > 0 as size_t
                }
            {
                let c2rust_fresh16 = src;
                src = src.offset(1);
                let c2rust_fresh17 = dst_p;
                dst_p = dst_p.offset(1);
                *c2rust_fresh17 = *c2rust_fresh16;
            }
        }
    }
    *dst_p = NUL as ::core::ffi::c_char;
    xfree(homedir_env as *mut ::core::ffi::c_void);
    if must_free {
        xfree(homedir_env_mod as *mut ::core::ffi::c_void);
    }
    return dst_p.offset_from(dst) as size_t;
}
pub unsafe extern "C" fn home_replace_save(
    mut buf: *mut buf_T,
    mut src: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = 3 as size_t;
    if !src.is_null() {
        len = len.wrapping_add(strlen(src));
    }
    let mut dst: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    home_replace(buf, src, dst, len, true_0 != 0);
    return dst;
}
pub unsafe extern "C" fn get_env_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if idx >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"idx >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1216 as ::core::ffi::c_uint,
                b"char *get_env_name(expand_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut envname: *mut ::core::ffi::c_char = os_getenvname_at_index(idx as size_t);
    if !envname.is_null() {
        xstrlcpy(
            &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char,
            envname,
            EXPAND_BUF_LEN as ::core::ffi::c_int as size_t,
        );
        xfree(envname as *mut ::core::ffi::c_void);
        return &raw mut (*xp).xp_buf as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn os_setenv_append_path(mut fname: *const ::core::ffi::c_char) -> bool {
    if !path_is_absolute(fname) {
        internal_error(b"os_setenv_append_path()\0".as_ptr() as *const ::core::ffi::c_char);
        return false_0 != 0;
    }
    let mut tail: *const ::core::ffi::c_char =
        path_tail_with_sep(fname as *mut ::core::ffi::c_char);
    let mut dirlen: size_t = tail.offset_from(fname) as size_t;
    '_c2rust_label: {
        if tail >= fname
            && dirlen.wrapping_add(1 as size_t)
                < ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
        {
        } else {
            __assert_fail(
                b"tail >= fname && dirlen + 1 < sizeof(os_buf)\0".as_ptr()
                    as *const ::core::ffi::c_char,
                b"src/nvim/os/env.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1247 as ::core::ffi::c_uint,
                b"_Bool os_setenv_append_path(const char *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xmemcpyz(
        os_buf.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        fname as *const ::core::ffi::c_void,
        dirlen,
    );
    let mut path: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    let pathlen: size_t = if !path.is_null() {
        strlen(path)
    } else {
        0 as size_t
    };
    let newlen: size_t = pathlen.wrapping_add(dirlen).wrapping_add(2 as size_t);
    let mut retval: bool = false_0 != 0;
    if newlen < MAX_ENVPATHLEN as size_t {
        let mut temp: *mut ::core::ffi::c_char = xmalloc(newlen) as *mut ::core::ffi::c_char;
        if pathlen == 0 as size_t {
            *temp.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        } else {
            xstrlcpy(temp, path, newlen);
            if ENV_SEPCHAR
                != *path.offset(pathlen.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            {
                xstrlcat(temp, ENV_SEPSTR.as_ptr(), newlen);
            }
        }
        xstrlcat(temp, os_buf.ptr() as *mut ::core::ffi::c_char, newlen);
        os_setenv(
            b"PATH\0".as_ptr() as *const ::core::ffi::c_char,
            temp,
            1 as ::core::ffi::c_int,
        );
        xfree(temp as *mut ::core::ffi::c_void);
        retval = true_0 != 0;
    }
    xfree(path as *mut ::core::ffi::c_void);
    return retval;
}
pub const MAX_ENVPATHLEN: ::core::ffi::c_int = INT_MAX;
#[no_mangle]
pub unsafe extern "C" fn os_shell_is_cmdexe(mut sh: *const ::core::ffi::c_char) -> bool {
    if *sh as ::core::ffi::c_int == NUL {
        return false_0 != 0;
    }
    if striequal(sh, b"$COMSPEC\0".as_ptr() as *const ::core::ffi::c_char) {
        let mut comspec: *mut ::core::ffi::c_char =
            os_getenv_noalloc(b"COMSPEC\0".as_ptr() as *const ::core::ffi::c_char);
        return striequal(
            b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char,
            path_tail(comspec),
        );
    }
    if striequal(sh, b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int != 0
        || striequal(sh, b"cmd\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int != 0
    {
        return true_0 != 0;
    }
    return striequal(
        b"cmd.exe\0".as_ptr() as *const ::core::ffi::c_char,
        path_tail(sh),
    );
}
pub unsafe extern "C" fn vim_unsetenv_ext(mut var: *const ::core::ffi::c_char) {
    os_unsetenv(var);
    if strcasecmp(
        var as *mut ::core::ffi::c_char,
        b"VIM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        didset_vim.set(false_0 != 0);
    } else if strcasecmp(
        var as *mut ::core::ffi::c_char,
        b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        didset_vimruntime.set(false_0 != 0);
    }
}
pub unsafe extern "C" fn vim_setenv_ext(
    mut name: *const ::core::ffi::c_char,
    mut val: *const ::core::ffi::c_char,
) {
    os_setenv(name, val, 1 as ::core::ffi::c_int);
    if strcasecmp(
        name as *mut ::core::ffi::c_char,
        b"HOME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ) == 0 as ::core::ffi::c_int
    {
        init_homedir();
    } else if didset_vim.get() as ::core::ffi::c_int != 0
        && strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"VIM\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        didset_vim.set(false_0 != 0);
    } else if didset_vimruntime.get() as ::core::ffi::c_int != 0
        && strcasecmp(
            name as *mut ::core::ffi::c_char,
            b"VIMRUNTIME\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
    {
        didset_vimruntime.set(false_0 != 0);
    }
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const ENV_SEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b":\0") };
