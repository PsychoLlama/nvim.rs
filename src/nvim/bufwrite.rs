use crate::src::nvim::autocmd::{apply_autocmds_exarg, aucmd_prepbuf, aucmd_restbuf};
use crate::src::nvim::buffer::{bt_nofilename, buf_set_file_id, bufref_valid, set_bufref};
use crate::src::nvim::change::unchanged;
use crate::src::nvim::drawscreen::status_redraw_all;
use crate::src::nvim::eval::vars::eval_charconvert;
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_eval::{aborting, should_abort};
use crate::src::nvim::fileio::{
    add_quoted_fname, buf_store_file_info, filemess, get_fio_flags, match_file_list, modname,
    msg_add_fileformat, msg_add_lines, need_conversion, set_rw_fname, time_differs, vim_rename,
    vim_tempname, write_eintr,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::input::ask_yesno;
use crate::src::nvim::main::{
    cmdmod, curbuf, e_empty_buffer, e_fsync, e_interr, e_longname, ex_no_reprint, exiting, got_int,
    msg_scroll, msg_silent, need_maketitle, no_wait_return, p_bdir, p_bex, p_bk, p_bsk, p_ccv,
    p_cpo, p_fs, p_pm, p_wb, IObuff,
};
use crate::src::nvim::mbyte::{enc_canonize, my_iconv_open, utf_ptr2char, utf_ptr2len_len};
use crate::src::nvim::memline::{
    get_file_in_dir, make_percent_swname, ml_get_buf, ml_preserve, ml_timestamp,
};
use crate::src::nvim::memory::{verbose_try_malloc, xfree, xmalloc, xmemcpyz, xstrlcat};
use crate::src::nvim::message::{emsg, msg, msg_progress, msg_puts_hl, semsg, set_keep_msg};
use crate::src::nvim::option::{copy_option_part, get_bkc_flags, get_fileformat_force, shortmess};
use crate::src::nvim::os::fs::{
    os_chown, os_close, os_copy, os_copy_xattr, os_fchown, os_file_is_writable, os_file_settime,
    os_fileinfo, os_fileinfo_hardlinks, os_fileinfo_id_equal, os_fileinfo_link, os_free_acl,
    os_fsync, os_get_acl, os_getperm, os_isdir, os_mkdir_recurse, os_nodetype, os_open,
    os_path_exists, os_remove, os_set_acl, os_setperm,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, close, getgid, gettext, getuid, iconv, iconv_close, memmove,
    snprintf, strlen,
};
use crate::src::nvim::path::{after_pathsep, path_fnamecmp, path_tail};
use crate::src::nvim::sha256::Sha256;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_add, vim_strchr};
pub use crate::src::nvim::types::{
    AdditionalData, AlignTextPos, BoolVarValue, BufUpdateCallbacks, CMD_index, Callback,
    CallbackType, Callback_data as C2Rust_Unnamed_5, ChangedtickDictItem, DecorExt,
    DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed_2, ExtmarkUndoObject, FileID, FileInfo, FloatAnchor,
    FloatRelative, GridView, Intersection, LineGetter, LuaRef, MTKey, MTNode, MTPos, MapHash,
    Map_int64_t_int64_t, Map_int64_t_ptr_t, Map_uint32_t_uint32_t, Map_uint64_t_ptr_t, MarkTree,
    OptInt, ScopeDictDictItem, ScopeType, ScreenGrid, Set_int64_t, Set_uint32_t, Set_uint64_t,
    SpecialVarValue, StlClickDefinition, StlClickDefinition_type_0 as C2Rust_Unnamed_12, Terminal,
    Timestamp, VarLockStatus, VarType, VirtLines, VirtText, VirtTextChunk, VirtTextPos, WinConfig,
    WinInfo, WinSplit, WinStyle, Window, __gid_t, __off_t, __time_t, __uid_t, aco_save_T, alist_T,
    auto_event, bhdr_T, blob_T, blobvar_S, blocknr_T, buf_T, bufref_T, bufstate_T, chunksize_T,
    cmd_addr_T, cmdidx_T, cmdmod_T, colnr_T, cstack_T, cstack_T_cs_pend as C2Rust_Unnamed_14,
    dict_T, dictvar_S, disptick_T, eslist_T, eslist_elem, event_T, exarg, exarg_T,
    extmark_undo_vec_t, fcs_chars_T, file_buffer, file_buffer_b_signcols as C2Rust_Unnamed_3,
    file_buffer_b_wininfo as C2Rust_Unnamed_11, file_buffer_update_callbacks as C2Rust_Unnamed_0,
    file_buffer_update_channels as C2Rust_Unnamed_1, float_T, fmark_T, fmarkv_T, frame_S, frame_T,
    funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_6, funccall_T, garray_T, gid_t, handle_T,
    hash_T, hashitem_T, hashtab_T, iconv_t, infoptr_T, int16_t, int32_t, int64_t, lcs_chars_T,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, llpos_T, lpos_T,
    mapblock, mapblock_T, match_T, matchitem, matchitem_T, memfile_T, memline_T, mfdirty_T,
    mtnode_inner_s, mtnode_s, off_T, off_t, partial_S, partial_T, pos_T, pos_save_T, proftime_T,
    ptr_t, qf_info_S, qf_info_T, queue, reg_extmatch_T, regmatch_T, regmmatch_T, regprog,
    regprog_T, sattr_T, schar_T, scid_T, sctx_T, size_t, ssize_t, syn_state,
    syn_state_sst_union as C2Rust_Unnamed_4, syn_time_T, synblock_T, synstate_T, taggy_T, terminal,
    time_t, typval_T, typval_vval_union, u_entry, u_entry_T, u_header, u_header_T,
    u_header_uh_alt_next as C2Rust_Unnamed_8, u_header_uh_alt_prev as C2Rust_Unnamed_7,
    u_header_uh_next as C2Rust_Unnamed_10, u_header_uh_prev as C2Rust_Unnamed_9, ufunc_S, ufunc_T,
    uid_t, uint16_t, uint32_t, uint64_t, uint8_t, undo_object, uv_gid_t, uv_stat_t, uv_timespec_t,
    uv_uid_t, varnumber_T, vim_acl_T, virt_line, visualinfo_T, win_T, window_S, wininfo_S,
    winopt_T, wline_T, xfmark_T, QUEUE,
};
use crate::src::nvim::ui::ui_flush;
use crate::src::nvim::undo::{curbufIsChanged, u_unchanged, u_update_save_nr, u_write_undo};
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
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_COUNT: C2Rust_Unnamed_13 = 76;
pub const HLF_PRE: C2Rust_Unnamed_13 = 75;
pub const HLF_OK: C2Rust_Unnamed_13 = 74;
pub const HLF_SO: C2Rust_Unnamed_13 = 73;
pub const HLF_SE: C2Rust_Unnamed_13 = 72;
pub const HLF_TSNC: C2Rust_Unnamed_13 = 71;
pub const HLF_TS: C2Rust_Unnamed_13 = 70;
pub const HLF_BFOOTER: C2Rust_Unnamed_13 = 69;
pub const HLF_BTITLE: C2Rust_Unnamed_13 = 68;
pub const HLF_CU: C2Rust_Unnamed_13 = 67;
pub const HLF_WBRNC: C2Rust_Unnamed_13 = 66;
pub const HLF_WBR: C2Rust_Unnamed_13 = 65;
pub const HLF_BORDER: C2Rust_Unnamed_13 = 64;
pub const HLF_MSG: C2Rust_Unnamed_13 = 63;
pub const HLF_NFLOAT: C2Rust_Unnamed_13 = 62;
pub const HLF_MSGSEP: C2Rust_Unnamed_13 = 61;
pub const HLF_INACTIVE: C2Rust_Unnamed_13 = 60;
pub const HLF_0: C2Rust_Unnamed_13 = 59;
pub const HLF_QFL: C2Rust_Unnamed_13 = 58;
pub const HLF_MC: C2Rust_Unnamed_13 = 57;
pub const HLF_CUL: C2Rust_Unnamed_13 = 56;
pub const HLF_CUC: C2Rust_Unnamed_13 = 55;
pub const HLF_TPF: C2Rust_Unnamed_13 = 54;
pub const HLF_TPS: C2Rust_Unnamed_13 = 53;
pub const HLF_TP: C2Rust_Unnamed_13 = 52;
pub const HLF_PBR: C2Rust_Unnamed_13 = 51;
pub const HLF_PST: C2Rust_Unnamed_13 = 50;
pub const HLF_PSB: C2Rust_Unnamed_13 = 49;
pub const HLF_PSX: C2Rust_Unnamed_13 = 48;
pub const HLF_PNX: C2Rust_Unnamed_13 = 47;
pub const HLF_PSK: C2Rust_Unnamed_13 = 46;
pub const HLF_PNK: C2Rust_Unnamed_13 = 45;
pub const HLF_PMSI: C2Rust_Unnamed_13 = 44;
pub const HLF_PMNI: C2Rust_Unnamed_13 = 43;
pub const HLF_PSI: C2Rust_Unnamed_13 = 42;
pub const HLF_PNI: C2Rust_Unnamed_13 = 41;
pub const HLF_SPL: C2Rust_Unnamed_13 = 40;
pub const HLF_SPR: C2Rust_Unnamed_13 = 39;
pub const HLF_SPC: C2Rust_Unnamed_13 = 38;
pub const HLF_SPB: C2Rust_Unnamed_13 = 37;
pub const HLF_CONCEAL: C2Rust_Unnamed_13 = 36;
pub const HLF_SC: C2Rust_Unnamed_13 = 35;
pub const HLF_TXA: C2Rust_Unnamed_13 = 34;
pub const HLF_TXD: C2Rust_Unnamed_13 = 33;
pub const HLF_DED: C2Rust_Unnamed_13 = 32;
pub const HLF_CHD: C2Rust_Unnamed_13 = 31;
pub const HLF_ADD: C2Rust_Unnamed_13 = 30;
pub const HLF_FC: C2Rust_Unnamed_13 = 29;
pub const HLF_FL: C2Rust_Unnamed_13 = 28;
pub const HLF_WM: C2Rust_Unnamed_13 = 27;
pub const HLF_W: C2Rust_Unnamed_13 = 26;
pub const HLF_VNC: C2Rust_Unnamed_13 = 25;
pub const HLF_V: C2Rust_Unnamed_13 = 24;
pub const HLF_T: C2Rust_Unnamed_13 = 23;
pub const HLF_VSP: C2Rust_Unnamed_13 = 22;
pub const HLF_C: C2Rust_Unnamed_13 = 21;
pub const HLF_SNC: C2Rust_Unnamed_13 = 20;
pub const HLF_S: C2Rust_Unnamed_13 = 19;
pub const HLF_R: C2Rust_Unnamed_13 = 18;
pub const HLF_CLF: C2Rust_Unnamed_13 = 17;
pub const HLF_CLS: C2Rust_Unnamed_13 = 16;
pub const HLF_CLN: C2Rust_Unnamed_13 = 15;
pub const HLF_LNB: C2Rust_Unnamed_13 = 14;
pub const HLF_LNA: C2Rust_Unnamed_13 = 13;
pub const HLF_N: C2Rust_Unnamed_13 = 12;
pub const HLF_CM: C2Rust_Unnamed_13 = 11;
pub const HLF_M: C2Rust_Unnamed_13 = 10;
pub const HLF_LC: C2Rust_Unnamed_13 = 9;
pub const HLF_L: C2Rust_Unnamed_13 = 8;
pub const HLF_I: C2Rust_Unnamed_13 = 7;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub const HLF_D: C2Rust_Unnamed_13 = 5;
pub const HLF_AT: C2Rust_Unnamed_13 = 4;
pub const HLF_TERM: C2Rust_Unnamed_13 = 3;
pub const HLF_EOB: C2Rust_Unnamed_13 = 2;
pub const HLF_8: C2Rust_Unnamed_13 = 1;
pub const HLF_NONE: C2Rust_Unnamed_13 = 0;
pub const CMD_USER_BUF: CMD_index = -2;
pub const CMD_USER: CMD_index = -1;
pub const CMD_SIZE: CMD_index = 557;
pub const CMD_Next: CMD_index = 556;
pub const CMD_tilde: CMD_index = 555;
pub const CMD_at: CMD_index = 554;
pub const CMD_rshift: CMD_index = 553;
pub const CMD_equal: CMD_index = 552;
pub const CMD_lshift: CMD_index = 551;
pub const CMD_and: CMD_index = 550;
pub const CMD_pound: CMD_index = 549;
pub const CMD_bang: CMD_index = 548;
pub const CMD_z: CMD_index = 547;
pub const CMD_yank: CMD_index = 546;
pub const CMD_xunmenu: CMD_index = 545;
pub const CMD_xunmap: CMD_index = 544;
pub const CMD_xnoremenu: CMD_index = 543;
pub const CMD_xnoremap: CMD_index = 542;
pub const CMD_xmenu: CMD_index = 541;
pub const CMD_xmapclear: CMD_index = 540;
pub const CMD_xmap: CMD_index = 539;
pub const CMD_xall: CMD_index = 538;
pub const CMD_xit: CMD_index = 537;
pub const CMD_wviminfo: CMD_index = 536;
pub const CMD_wundo: CMD_index = 535;
pub const CMD_wshada: CMD_index = 534;
pub const CMD_wqall: CMD_index = 533;
pub const CMD_wq: CMD_index = 532;
pub const CMD_wprevious: CMD_index = 531;
pub const CMD_wnext: CMD_index = 530;
pub const CMD_winpos: CMD_index = 529;
pub const CMD_windo: CMD_index = 528;
pub const CMD_wincmd: CMD_index = 527;
pub const CMD_winsize: CMD_index = 526;
pub const CMD_while: CMD_index = 525;
pub const CMD_wall: CMD_index = 524;
pub const CMD_wNext: CMD_index = 523;
pub const CMD_write: CMD_index = 522;
pub const CMD_vunmenu: CMD_index = 521;
pub const CMD_vunmap: CMD_index = 520;
pub const CMD_vsplit: CMD_index = 519;
pub const CMD_vnoremenu: CMD_index = 518;
pub const CMD_vnew: CMD_index = 517;
pub const CMD_vnoremap: CMD_index = 516;
pub const CMD_vmenu: CMD_index = 515;
pub const CMD_vmapclear: CMD_index = 514;
pub const CMD_vmap: CMD_index = 513;
pub const CMD_viusage: CMD_index = 512;
pub const CMD_vimgrepadd: CMD_index = 511;
pub const CMD_vimgrep: CMD_index = 510;
pub const CMD_view: CMD_index = 509;
pub const CMD_visual: CMD_index = 508;
pub const CMD_vertical: CMD_index = 507;
pub const CMD_verbose: CMD_index = 506;
pub const CMD_version: CMD_index = 505;
pub const CMD_vglobal: CMD_index = 504;
pub const CMD_update: CMD_index = 503;
pub const CMD_unsilent: CMD_index = 502;
pub const CMD_unmenu: CMD_index = 501;
pub const CMD_unmap: CMD_index = 500;
pub const CMD_unlockvar: CMD_index = 499;
pub const CMD_unlet: CMD_index = 498;
pub const CMD_uniq: CMD_index = 497;
pub const CMD_unhide: CMD_index = 496;
pub const CMD_unabbreviate: CMD_index = 495;
pub const CMD_undolist: CMD_index = 494;
pub const CMD_undojoin: CMD_index = 493;
pub const CMD_undo: CMD_index = 492;
pub const CMD_tunmap: CMD_index = 491;
pub const CMD_tunmenu: CMD_index = 490;
pub const CMD_tselect: CMD_index = 489;
pub const CMD_try: CMD_index = 488;
pub const CMD_trust: CMD_index = 487;
pub const CMD_trewind: CMD_index = 486;
pub const CMD_tprevious: CMD_index = 485;
pub const CMD_topleft: CMD_index = 484;
pub const CMD_tnoremap: CMD_index = 483;
pub const CMD_tnext: CMD_index = 482;
pub const CMD_tmapclear: CMD_index = 481;
pub const CMD_tmap: CMD_index = 480;
pub const CMD_tmenu: CMD_index = 479;
pub const CMD_tlunmenu: CMD_index = 478;
pub const CMD_tlnoremenu: CMD_index = 477;
pub const CMD_tlmenu: CMD_index = 476;
pub const CMD_tlast: CMD_index = 475;
pub const CMD_tjump: CMD_index = 474;
pub const CMD_throw: CMD_index = 473;
pub const CMD_tfirst: CMD_index = 472;
pub const CMD_terminal: CMD_index = 471;
pub const CMD_tclfile: CMD_index = 470;
pub const CMD_tcldo: CMD_index = 469;
pub const CMD_tcl: CMD_index = 468;
pub const CMD_tabs: CMD_index = 467;
pub const CMD_tabrewind: CMD_index = 466;
pub const CMD_tabNext: CMD_index = 465;
pub const CMD_tabprevious: CMD_index = 464;
pub const CMD_tabonly: CMD_index = 463;
pub const CMD_tabnew: CMD_index = 462;
pub const CMD_tabnext: CMD_index = 461;
pub const CMD_tablast: CMD_index = 460;
pub const CMD_tabmove: CMD_index = 459;
pub const CMD_tabfirst: CMD_index = 458;
pub const CMD_tabfind: CMD_index = 457;
pub const CMD_tabedit: CMD_index = 456;
pub const CMD_tabdo: CMD_index = 455;
pub const CMD_tabclose: CMD_index = 454;
pub const CMD_tab: CMD_index = 453;
pub const CMD_tags: CMD_index = 452;
pub const CMD_tag: CMD_index = 451;
pub const CMD_tNext: CMD_index = 450;
pub const CMD_tchdir: CMD_index = 449;
pub const CMD_tcd: CMD_index = 448;
pub const CMD_t: CMD_index = 447;
pub const CMD_syncbind: CMD_index = 446;
pub const CMD_syntime: CMD_index = 445;
pub const CMD_syntax: CMD_index = 444;
pub const CMD_swapname: CMD_index = 443;
pub const CMD_sview: CMD_index = 442;
pub const CMD_suspend: CMD_index = 441;
pub const CMD_sunmenu: CMD_index = 440;
pub const CMD_sunmap: CMD_index = 439;
pub const CMD_sunhide: CMD_index = 438;
pub const CMD_stselect: CMD_index = 437;
pub const CMD_stjump: CMD_index = 436;
pub const CMD_stopinsert: CMD_index = 435;
pub const CMD_startreplace: CMD_index = 434;
pub const CMD_startgreplace: CMD_index = 433;
pub const CMD_startinsert: CMD_index = 432;
pub const CMD_stag: CMD_index = 431;
pub const CMD_stop: CMD_index = 430;
pub const CMD_srewind: CMD_index = 429;
pub const CMD_sprevious: CMD_index = 428;
pub const CMD_spellwrong: CMD_index = 427;
pub const CMD_spellundo: CMD_index = 426;
pub const CMD_spellrare: CMD_index = 425;
pub const CMD_spellrepall: CMD_index = 424;
pub const CMD_spellinfo: CMD_index = 423;
pub const CMD_spelldump: CMD_index = 422;
pub const CMD_spellgood: CMD_index = 421;
pub const CMD_split: CMD_index = 420;
pub const CMD_sort: CMD_index = 419;
pub const CMD_source: CMD_index = 418;
pub const CMD_snoremenu: CMD_index = 417;
pub const CMD_snoremap: CMD_index = 416;
pub const CMD_snomagic: CMD_index = 415;
pub const CMD_snext: CMD_index = 414;
pub const CMD_smenu: CMD_index = 413;
pub const CMD_smapclear: CMD_index = 412;
pub const CMD_smap: CMD_index = 411;
pub const CMD_smagic: CMD_index = 410;
pub const CMD_slast: CMD_index = 409;
pub const CMD_sleep: CMD_index = 408;
pub const CMD_silent: CMD_index = 407;
pub const CMD_sign: CMD_index = 406;
pub const CMD_simalt: CMD_index = 405;
pub const CMD_sfirst: CMD_index = 404;
pub const CMD_sfind: CMD_index = 403;
pub const CMD_setlocal: CMD_index = 402;
pub const CMD_setglobal: CMD_index = 401;
pub const CMD_setfiletype: CMD_index = 400;
pub const CMD_set: CMD_index = 399;
pub const CMD_scriptencoding: CMD_index = 398;
pub const CMD_scriptnames: CMD_index = 397;
pub const CMD_sbrewind: CMD_index = 396;
pub const CMD_sbprevious: CMD_index = 395;
pub const CMD_sbnext: CMD_index = 394;
pub const CMD_sbmodified: CMD_index = 393;
pub const CMD_sblast: CMD_index = 392;
pub const CMD_sbfirst: CMD_index = 391;
pub const CMD_sball: CMD_index = 390;
pub const CMD_sbNext: CMD_index = 389;
pub const CMD_sbuffer: CMD_index = 388;
pub const CMD_saveas: CMD_index = 387;
pub const CMD_sandbox: CMD_index = 386;
pub const CMD_sall: CMD_index = 385;
pub const CMD_sargument: CMD_index = 384;
pub const CMD_sNext: CMD_index = 383;
pub const CMD_substitute: CMD_index = 382;
pub const CMD_rviminfo: CMD_index = 381;
pub const CMD_rubyfile: CMD_index = 380;
pub const CMD_rubydo: CMD_index = 379;
pub const CMD_ruby: CMD_index = 378;
pub const CMD_rundo: CMD_index = 377;
pub const CMD_runtime: CMD_index = 376;
pub const CMD_rshada: CMD_index = 375;
pub const CMD_rightbelow: CMD_index = 374;
pub const CMD_right: CMD_index = 373;
pub const CMD_rewind: CMD_index = 372;
pub const CMD_return: CMD_index = 371;
pub const CMD_retab: CMD_index = 370;
pub const CMD_restart: CMD_index = 369;
pub const CMD_resize: CMD_index = 368;
pub const CMD_registers: CMD_index = 367;
pub const CMD_redrawtabline: CMD_index = 366;
pub const CMD_redrawstatus: CMD_index = 365;
pub const CMD_redraw: CMD_index = 364;
pub const CMD_redir: CMD_index = 363;
pub const CMD_redo: CMD_index = 362;
pub const CMD_recover: CMD_index = 361;
pub const CMD_read: CMD_index = 360;
pub const CMD_qall: CMD_index = 359;
pub const CMD_quitall: CMD_index = 358;
pub const CMD_quit: CMD_index = 357;
pub const CMD_pyxfile: CMD_index = 356;
pub const CMD_pythonx: CMD_index = 355;
pub const CMD_pyxdo: CMD_index = 354;
pub const CMD_pyx: CMD_index = 353;
pub const CMD_py3file: CMD_index = 352;
pub const CMD_python3: CMD_index = 351;
pub const CMD_py3do: CMD_index = 350;
pub const CMD_py3: CMD_index = 349;
pub const CMD_pyfile: CMD_index = 348;
pub const CMD_pydo: CMD_index = 347;
pub const CMD_python: CMD_index = 346;
pub const CMD_pwd: CMD_index = 345;
pub const CMD_put: CMD_index = 344;
pub const CMD_ptselect: CMD_index = 343;
pub const CMD_ptrewind: CMD_index = 342;
pub const CMD_ptprevious: CMD_index = 341;
pub const CMD_ptnext: CMD_index = 340;
pub const CMD_ptlast: CMD_index = 339;
pub const CMD_ptjump: CMD_index = 338;
pub const CMD_ptfirst: CMD_index = 337;
pub const CMD_ptNext: CMD_index = 336;
pub const CMD_ptag: CMD_index = 335;
pub const CMD_psearch: CMD_index = 334;
pub const CMD_profdel: CMD_index = 333;
pub const CMD_profile: CMD_index = 332;
pub const CMD_previous: CMD_index = 331;
pub const CMD_preserve: CMD_index = 330;
pub const CMD_ppop: CMD_index = 329;
pub const CMD_popup: CMD_index = 328;
pub const CMD_pop: CMD_index = 327;
pub const CMD_pedit: CMD_index = 326;
pub const CMD_perlfile: CMD_index = 325;
pub const CMD_perldo: CMD_index = 324;
pub const CMD_perl: CMD_index = 323;
pub const CMD_pclose: CMD_index = 322;
pub const CMD_pbuffer: CMD_index = 321;
pub const CMD_packloadall: CMD_index = 320;
pub const CMD_packadd: CMD_index = 319;
pub const CMD_print: CMD_index = 318;
pub const CMD_ownsyntax: CMD_index = 317;
pub const CMD_ounmenu: CMD_index = 316;
pub const CMD_ounmap: CMD_index = 315;
pub const CMD_options: CMD_index = 314;
pub const CMD_onoremenu: CMD_index = 313;
pub const CMD_onoremap: CMD_index = 312;
pub const CMD_only: CMD_index = 311;
pub const CMD_omenu: CMD_index = 310;
pub const CMD_omapclear: CMD_index = 309;
pub const CMD_omap: CMD_index = 308;
pub const CMD_oldfiles: CMD_index = 307;
pub const CMD_nunmenu: CMD_index = 306;
pub const CMD_nunmap: CMD_index = 305;
pub const CMD_number: CMD_index = 304;
pub const CMD_normal: CMD_index = 303;
pub const CMD_noswapfile: CMD_index = 302;
pub const CMD_noremenu: CMD_index = 301;
pub const CMD_noreabbrev: CMD_index = 300;
pub const CMD_nohlsearch: CMD_index = 299;
pub const CMD_noautocmd: CMD_index = 298;
pub const CMD_noremap: CMD_index = 297;
pub const CMD_nnoremenu: CMD_index = 296;
pub const CMD_nnoremap: CMD_index = 295;
pub const CMD_nmenu: CMD_index = 294;
pub const CMD_nmapclear: CMD_index = 293;
pub const CMD_nmap: CMD_index = 292;
pub const CMD_new: CMD_index = 291;
pub const CMD_next: CMD_index = 290;
pub const CMD_mzfile: CMD_index = 289;
pub const CMD_mzscheme: CMD_index = 288;
pub const CMD_mode: CMD_index = 287;
pub const CMD_mkview: CMD_index = 286;
pub const CMD_mkvimrc: CMD_index = 285;
pub const CMD_mkspell: CMD_index = 284;
pub const CMD_mksession: CMD_index = 283;
pub const CMD_mkexrc: CMD_index = 282;
pub const CMD_messages: CMD_index = 281;
pub const CMD_menutranslate: CMD_index = 280;
pub const CMD_menu: CMD_index = 279;
pub const CMD_match: CMD_index = 278;
pub const CMD_marks: CMD_index = 277;
pub const CMD_mapclear: CMD_index = 276;
pub const CMD_map: CMD_index = 275;
pub const CMD_make: CMD_index = 274;
pub const CMD_mark: CMD_index = 273;
pub const CMD_move: CMD_index = 272;
pub const CMD_lsp: CMD_index = 271;
pub const CMD_ls: CMD_index = 270;
pub const CMD_lwindow: CMD_index = 269;
pub const CMD_lvimgrepadd: CMD_index = 268;
pub const CMD_lvimgrep: CMD_index = 267;
pub const CMD_luafile: CMD_index = 266;
pub const CMD_luado: CMD_index = 265;
pub const CMD_lua: CMD_index = 264;
pub const CMD_lunmap: CMD_index = 263;
pub const CMD_ltag: CMD_index = 262;
pub const CMD_lrewind: CMD_index = 261;
pub const CMD_lpfile: CMD_index = 260;
pub const CMD_lprevious: CMD_index = 259;
pub const CMD_lopen: CMD_index = 258;
pub const CMD_lolder: CMD_index = 257;
pub const CMD_lockvar: CMD_index = 256;
pub const CMD_lockmarks: CMD_index = 255;
pub const CMD_loadkeymap: CMD_index = 254;
pub const CMD_loadview: CMD_index = 253;
pub const CMD_lnfile: CMD_index = 252;
pub const CMD_lnewer: CMD_index = 251;
pub const CMD_lnext: CMD_index = 250;
pub const CMD_lnoremap: CMD_index = 249;
pub const CMD_lmake: CMD_index = 248;
pub const CMD_lmapclear: CMD_index = 247;
pub const CMD_lmap: CMD_index = 246;
pub const CMD_llist: CMD_index = 245;
pub const CMD_llast: CMD_index = 244;
pub const CMD_ll: CMD_index = 243;
pub const CMD_lhistory: CMD_index = 242;
pub const CMD_lhelpgrep: CMD_index = 241;
pub const CMD_lgrepadd: CMD_index = 240;
pub const CMD_lgrep: CMD_index = 239;
pub const CMD_lgetexpr: CMD_index = 238;
pub const CMD_lgetbuffer: CMD_index = 237;
pub const CMD_lgetfile: CMD_index = 236;
pub const CMD_lfirst: CMD_index = 235;
pub const CMD_lfdo: CMD_index = 234;
pub const CMD_lfile: CMD_index = 233;
pub const CMD_lexpr: CMD_index = 232;
pub const CMD_let: CMD_index = 231;
pub const CMD_leftabove: CMD_index = 230;
pub const CMD_left: CMD_index = 229;
pub const CMD_ldo: CMD_index = 228;
pub const CMD_lclose: CMD_index = 227;
pub const CMD_lchdir: CMD_index = 226;
pub const CMD_lcd: CMD_index = 225;
pub const CMD_lbottom: CMD_index = 224;
pub const CMD_lbelow: CMD_index = 223;
pub const CMD_lbefore: CMD_index = 222;
pub const CMD_lbuffer: CMD_index = 221;
pub const CMD_later: CMD_index = 220;
pub const CMD_lafter: CMD_index = 219;
pub const CMD_laddfile: CMD_index = 218;
pub const CMD_laddbuffer: CMD_index = 217;
pub const CMD_laddexpr: CMD_index = 216;
pub const CMD_language: CMD_index = 215;
pub const CMD_labove: CMD_index = 214;
pub const CMD_last: CMD_index = 213;
pub const CMD_lNfile: CMD_index = 212;
pub const CMD_lNext: CMD_index = 211;
pub const CMD_list: CMD_index = 210;
pub const CMD_keepalt: CMD_index = 209;
pub const CMD_keeppatterns: CMD_index = 208;
pub const CMD_keepjumps: CMD_index = 207;
pub const CMD_keepmarks: CMD_index = 206;
pub const CMD_k: CMD_index = 205;
pub const CMD_jumps: CMD_index = 204;
pub const CMD_join: CMD_index = 203;
pub const CMD_iunmenu: CMD_index = 202;
pub const CMD_iunabbrev: CMD_index = 201;
pub const CMD_iunmap: CMD_index = 200;
pub const CMD_isplit: CMD_index = 199;
pub const CMD_isearch: CMD_index = 198;
pub const CMD_iput: CMD_index = 197;
pub const CMD_intro: CMD_index = 196;
pub const CMD_inoremenu: CMD_index = 195;
pub const CMD_inoreabbrev: CMD_index = 194;
pub const CMD_inoremap: CMD_index = 193;
pub const CMD_imenu: CMD_index = 192;
pub const CMD_imapclear: CMD_index = 191;
pub const CMD_imap: CMD_index = 190;
pub const CMD_ilist: CMD_index = 189;
pub const CMD_ijump: CMD_index = 188;
pub const CMD_if: CMD_index = 187;
pub const CMD_iabclear: CMD_index = 186;
pub const CMD_iabbrev: CMD_index = 185;
pub const CMD_insert: CMD_index = 184;
pub const CMD_horizontal: CMD_index = 183;
pub const CMD_history: CMD_index = 182;
pub const CMD_hide: CMD_index = 181;
pub const CMD_highlight: CMD_index = 180;
pub const CMD_helptags: CMD_index = 179;
pub const CMD_helpgrep: CMD_index = 178;
pub const CMD_helpclose: CMD_index = 177;
pub const CMD_help: CMD_index = 176;
pub const CMD_gvim: CMD_index = 175;
pub const CMD_gui: CMD_index = 174;
pub const CMD_grepadd: CMD_index = 173;
pub const CMD_grep: CMD_index = 172;
pub const CMD_goto: CMD_index = 171;
pub const CMD_global: CMD_index = 170;
pub const CMD_fclose: CMD_index = 169;
pub const CMD_function: CMD_index = 168;
pub const CMD_for: CMD_index = 167;
pub const CMD_foldopen: CMD_index = 166;
pub const CMD_folddoclosed: CMD_index = 165;
pub const CMD_folddoopen: CMD_index = 164;
pub const CMD_foldclose: CMD_index = 163;
pub const CMD_fold: CMD_index = 162;
pub const CMD_first: CMD_index = 161;
pub const CMD_finish: CMD_index = 160;
pub const CMD_finally: CMD_index = 159;
pub const CMD_find: CMD_index = 158;
pub const CMD_filter: CMD_index = 157;
pub const CMD_filetype: CMD_index = 156;
pub const CMD_files: CMD_index = 155;
pub const CMD_file: CMD_index = 154;
pub const CMD_exusage: CMD_index = 153;
pub const CMD_exit: CMD_index = 152;
pub const CMD_execute: CMD_index = 151;
pub const CMD_ex: CMD_index = 150;
pub const CMD_eval: CMD_index = 149;
pub const CMD_enew: CMD_index = 148;
pub const CMD_endwhile: CMD_index = 147;
pub const CMD_endtry: CMD_index = 146;
pub const CMD_endfor: CMD_index = 145;
pub const CMD_endfunction: CMD_index = 144;
pub const CMD_endif: CMD_index = 143;
pub const CMD_emenu: CMD_index = 142;
pub const CMD_elseif: CMD_index = 141;
pub const CMD_else: CMD_index = 140;
pub const CMD_echon: CMD_index = 139;
pub const CMD_echomsg: CMD_index = 138;
pub const CMD_echohl: CMD_index = 137;
pub const CMD_echoerr: CMD_index = 136;
pub const CMD_echo: CMD_index = 135;
pub const CMD_earlier: CMD_index = 134;
pub const CMD_edit: CMD_index = 133;
pub const CMD_dsplit: CMD_index = 132;
pub const CMD_dsearch: CMD_index = 131;
pub const CMD_drop: CMD_index = 130;
pub const CMD_doautoall: CMD_index = 129;
pub const CMD_doautocmd: CMD_index = 128;
pub const CMD_dlist: CMD_index = 127;
pub const CMD_djump: CMD_index = 126;
pub const CMD_digraphs: CMD_index = 125;
pub const CMD_diffthis: CMD_index = 124;
pub const CMD_diffsplit: CMD_index = 123;
pub const CMD_diffput: CMD_index = 122;
pub const CMD_diffpatch: CMD_index = 121;
pub const CMD_diffoff: CMD_index = 120;
pub const CMD_diffget: CMD_index = 119;
pub const CMD_diffupdate: CMD_index = 118;
pub const CMD_display: CMD_index = 117;
pub const CMD_detach: CMD_index = 116;
pub const CMD_delfunction: CMD_index = 115;
pub const CMD_delcommand: CMD_index = 114;
pub const CMD_defer: CMD_index = 113;
pub const CMD_debuggreedy: CMD_index = 112;
pub const CMD_debug: CMD_index = 111;
pub const CMD_delmarks: CMD_index = 110;
pub const CMD_delete: CMD_index = 109;
pub const CMD_cwindow: CMD_index = 108;
pub const CMD_cunmenu: CMD_index = 107;
pub const CMD_cunabbrev: CMD_index = 106;
pub const CMD_cunmap: CMD_index = 105;
pub const CMD_crewind: CMD_index = 104;
pub const CMD_cquit: CMD_index = 103;
pub const CMD_cpfile: CMD_index = 102;
pub const CMD_cprevious: CMD_index = 101;
pub const CMD_copen: CMD_index = 100;
pub const CMD_const: CMD_index = 99;
pub const CMD_connect: CMD_index = 98;
pub const CMD_confirm: CMD_index = 97;
pub const CMD_continue: CMD_index = 96;
pub const CMD_compiler: CMD_index = 95;
pub const CMD_comclear: CMD_index = 94;
pub const CMD_command: CMD_index = 93;
pub const CMD_colorscheme: CMD_index = 92;
pub const CMD_colder: CMD_index = 91;
pub const CMD_copy: CMD_index = 90;
pub const CMD_cnoremenu: CMD_index = 89;
pub const CMD_cnoreabbrev: CMD_index = 88;
pub const CMD_cnoremap: CMD_index = 87;
pub const CMD_cnfile: CMD_index = 86;
pub const CMD_cnewer: CMD_index = 85;
pub const CMD_cnext: CMD_index = 84;
pub const CMD_cmenu: CMD_index = 83;
pub const CMD_cmapclear: CMD_index = 82;
pub const CMD_cmap: CMD_index = 81;
pub const CMD_clearjumps: CMD_index = 80;
pub const CMD_close: CMD_index = 79;
pub const CMD_clast: CMD_index = 78;
pub const CMD_clist: CMD_index = 77;
pub const CMD_chistory: CMD_index = 76;
pub const CMD_checktime: CMD_index = 75;
pub const CMD_checkpath: CMD_index = 74;
pub const CMD_checkhealth: CMD_index = 73;
pub const CMD_changes: CMD_index = 72;
pub const CMD_chdir: CMD_index = 71;
pub const CMD_cgetexpr: CMD_index = 70;
pub const CMD_cgetbuffer: CMD_index = 69;
pub const CMD_cgetfile: CMD_index = 68;
pub const CMD_cfirst: CMD_index = 67;
pub const CMD_cfdo: CMD_index = 66;
pub const CMD_cfile: CMD_index = 65;
pub const CMD_cexpr: CMD_index = 64;
pub const CMD_center: CMD_index = 63;
pub const CMD_cdo: CMD_index = 62;
pub const CMD_cd: CMD_index = 61;
pub const CMD_cclose: CMD_index = 60;
pub const CMD_cc: CMD_index = 59;
pub const CMD_cbottom: CMD_index = 58;
pub const CMD_cbelow: CMD_index = 57;
pub const CMD_cbefore: CMD_index = 56;
pub const CMD_cbuffer: CMD_index = 55;
pub const CMD_catch: CMD_index = 54;
pub const CMD_call: CMD_index = 53;
pub const CMD_cafter: CMD_index = 52;
pub const CMD_caddfile: CMD_index = 51;
pub const CMD_caddexpr: CMD_index = 50;
pub const CMD_caddbuffer: CMD_index = 49;
pub const CMD_cabove: CMD_index = 48;
pub const CMD_cabclear: CMD_index = 47;
pub const CMD_cabbrev: CMD_index = 46;
pub const CMD_cNfile: CMD_index = 45;
pub const CMD_cNext: CMD_index = 44;
pub const CMD_change: CMD_index = 43;
pub const CMD_bwipeout: CMD_index = 42;
pub const CMD_bunload: CMD_index = 41;
pub const CMD_bufdo: CMD_index = 40;
pub const CMD_buffers: CMD_index = 39;
pub const CMD_browse: CMD_index = 38;
pub const CMD_breaklist: CMD_index = 37;
pub const CMD_breakdel: CMD_index = 36;
pub const CMD_breakadd: CMD_index = 35;
pub const CMD_break: CMD_index = 34;
pub const CMD_brewind: CMD_index = 33;
pub const CMD_bprevious: CMD_index = 32;
pub const CMD_botright: CMD_index = 31;
pub const CMD_bnext: CMD_index = 30;
pub const CMD_bmodified: CMD_index = 29;
pub const CMD_blast: CMD_index = 28;
pub const CMD_bfirst: CMD_index = 27;
pub const CMD_belowright: CMD_index = 26;
pub const CMD_bdelete: CMD_index = 25;
pub const CMD_balt: CMD_index = 24;
pub const CMD_badd: CMD_index = 23;
pub const CMD_ball: CMD_index = 22;
pub const CMD_bNext: CMD_index = 21;
pub const CMD_buffer: CMD_index = 20;
pub const CMD_aunmenu: CMD_index = 19;
pub const CMD_augroup: CMD_index = 18;
pub const CMD_autocmd: CMD_index = 17;
pub const CMD_ascii: CMD_index = 16;
pub const CMD_argument: CMD_index = 15;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argedit: CMD_index = 12;
pub const CMD_argdedupe: CMD_index = 11;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_argdelete: CMD_index = 9;
pub const CMD_argadd: CMD_index = 8;
pub const CMD_args: CMD_index = 7;
pub const CMD_anoremenu: CMD_index = 6;
pub const CMD_amenu: CMD_index = 5;
pub const CMD_all: CMD_index = 4;
pub const CMD_aboveleft: CMD_index = 3;
pub const CMD_abclear: CMD_index = 2;
pub const CMD_abbreviate: CMD_index = 1;
pub const CMD_append: CMD_index = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const CMOD_NOSWAPFILE: C2Rust_Unnamed_15 = 8192;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_15 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_15 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_15 = 1024;
pub const CMOD_KEEPMARKS: C2Rust_Unnamed_15 = 512;
pub const CMOD_KEEPALT: C2Rust_Unnamed_15 = 256;
pub const CMOD_CONFIRM: C2Rust_Unnamed_15 = 128;
pub const CMOD_BROWSE: C2Rust_Unnamed_15 = 64;
pub const CMOD_HIDE: C2Rust_Unnamed_15 = 32;
pub const CMOD_NOAUTOCMD: C2Rust_Unnamed_15 = 16;
pub const CMOD_UNSILENT: C2Rust_Unnamed_15 = 8;
pub const CMOD_ERRSILENT: C2Rust_Unnamed_15 = 4;
pub const CMOD_SILENT: C2Rust_Unnamed_15 = 2;
pub const CMOD_SANDBOX: C2Rust_Unnamed_15 = 1;
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
pub struct Error_T {
    pub num: *const ::core::ffi::c_char,
    pub msg: *mut ::core::ffi::c_char,
    pub arg: ::core::ffi::c_int,
    pub alloc: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bw_info {
    pub bw_fd: ::core::ffi::c_int,
    pub bw_buf: *mut ::core::ffi::c_char,
    pub bw_len: ::core::ffi::c_int,
    pub bw_flags: ::core::ffi::c_int,
    pub bw_first: ::core::ffi::c_int,
    pub bw_conv_buf: *mut ::core::ffi::c_char,
    pub bw_conv_buflen: size_t,
    pub bw_conv_error: ::core::ffi::c_int,
    pub bw_conv_error_lnum: linenr_T,
    pub bw_start_lnum: linenr_T,
    pub bw_iconv_fd: iconv_t,
}
pub const WRITEBUFSIZE: C2Rust_Unnamed_17 = 8192;
pub const SHM_WRI: C2Rust_Unnamed_20 = 119;
pub const SHM_WRITE: C2Rust_Unnamed_20 = 87;
pub const FIO_LATIN1: C2Rust_Unnamed_16 = 1;
pub const FIO_ENDIAN_L: C2Rust_Unnamed_16 = 128;
pub const FIO_UTF16: C2Rust_Unnamed_16 = 16;
pub const FIO_UCS2: C2Rust_Unnamed_16 = 4;
pub const FIO_UCS4: C2Rust_Unnamed_16 = 8;
pub const FIO_NOCONVERT: C2Rust_Unnamed_16 = 8192;
pub const FIO_UTF8: C2Rust_Unnamed_16 = 2;
pub const ICONV_MULT: C2Rust_Unnamed_18 = 8;
pub const kOptBkcFlagBreakhardlink: C2Rust_Unnamed_19 = 16;
pub const kOptBkcFlagBreaksymlink: C2Rust_Unnamed_19 = 8;
pub const kOptBkcFlagAuto: C2Rust_Unnamed_19 = 2;
pub const kOptBkcFlagYes: C2Rust_Unnamed_19 = 1;
pub const SHM_OVER: C2Rust_Unnamed_20 = 111;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const FIO_ALL: C2Rust_Unnamed_16 = -1;
pub const FIO_UCSBOM: C2Rust_Unnamed_16 = 16384;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kOptBkcFlagNo: C2Rust_Unnamed_19 = 4;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const SHM_SEARCHCOUNT: C2Rust_Unnamed_20 = 83;
pub const SHM_FILEINFO: C2Rust_Unnamed_20 = 70;
pub const SHM_RECORDING: C2Rust_Unnamed_20 = 113;
pub const SHM_COMPLETIONSCAN: C2Rust_Unnamed_20 = 67;
pub const SHM_COMPLETIONMENU: C2Rust_Unnamed_20 = 99;
pub const SHM_INTRO: C2Rust_Unnamed_20 = 73;
pub const SHM_ATTENTION: C2Rust_Unnamed_20 = 65;
pub const SHM_SEARCH: C2Rust_Unnamed_20 = 115;
pub const SHM_OVERALL: C2Rust_Unnamed_20 = 79;
pub const SHM_TRUNCALL: C2Rust_Unnamed_20 = 84;
pub const SHM_TRUNC: C2Rust_Unnamed_20 = 116;
pub const SHM_ABBREVIATIONS: C2Rust_Unnamed_20 = 97;
pub const SHM_LINES: C2Rust_Unnamed_20 = 108;
pub const SHM_MOD: C2Rust_Unnamed_20 = 109;
pub const SHM_RO: C2Rust_Unnamed_20 = 114;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const __O_NOFOLLOW: ::core::ffi::c_int = 0o400000 as ::core::ffi::c_int;
pub const O_NOFOLLOW: ::core::ffi::c_int = __O_NOFOLLOW;
pub const UV_FS_COPYFILE_FICLONE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_READERR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const BF_WRITE_MASK: ::core::ffi::c_int = BF_NOTEDITED + BF_NEW + BF_READERR;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 126] = unsafe {
    ::core::mem::transmute::<
        [u8; 126],
        [::core::ffi::c_char; 126],
    >(
        *b"int buf_write_make_backup(char *, _Bool, FileInfo *, vim_acl_T, int, unsigned int, _Bool, _Bool, _Bool *, char **, Error_T *)\0",
    )
};
#[inline(always)]
unsafe extern "C" fn buf_get_changedtick(buf: *const buf_T) -> varnumber_T {
    return (*buf).changedtick_di.di_tv.vval.v_number;
}
static err_readonly: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"is read-only (cannot override: \"W\" in 'cpoptions')\0".as_ptr()
        as *const ::core::ffi::c_char,
);
static e_patchmode_cant_touch_empty_original_file: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E206: Patchmode: can't touch empty original file\0",
        )
    });
static e_write_error_conversion_failed_make_fenc_empty_to_override: GlobalCell<
    [::core::ffi::c_char; 69],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 69], [::core::ffi::c_char; 69]>(
        *b"E513: Write error, conversion failed (make 'fenc' empty to override)\0",
    )
});
static e_write_error_conversion_failed_in_line_nr_make_fenc_empty_to_override: GlobalCell<
    [::core::ffi::c_char; 80],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 80], [::core::ffi::c_char; 80]>(
        *b"E513: Write error, conversion failed in line %d (make 'fenc' empty to override)\0",
    )
});
static e_write_error_file_system_full: GlobalCell<[::core::ffi::c_char; 38]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
            *b"E514: Write error (file system full?)\0",
        )
    });
static e_no_matching_autocommands_for_buftype_str_buffer: GlobalCell<[::core::ffi::c_char; 53]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
            *b"E676: No matching autocommands for buftype=%s buffer\0",
        )
    });
pub const SMALLBUFSIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C" fn ucs2bytes(
    mut c: ::core::ffi::c_uint,
    mut pp: *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    let mut p: *mut uint8_t = *pp as *mut uint8_t;
    let mut error: bool = false_0 != 0;
    if flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
        if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 = c as uint8_t;
            let c2rust_fresh4 = p;
            p = p.offset(1);
            *c2rust_fresh4 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = (c >> 16 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh6 = p;
            p = p.offset(1);
            *c2rust_fresh6 = (c >> 24 as ::core::ffi::c_int) as uint8_t;
        } else {
            let c2rust_fresh7 = p;
            p = p.offset(1);
            *c2rust_fresh7 = (c >> 24 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh8 = p;
            p = p.offset(1);
            *c2rust_fresh8 = (c >> 16 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh9 = p;
            p = p.offset(1);
            *c2rust_fresh9 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh10 = p;
            p = p.offset(1);
            *c2rust_fresh10 = c as uint8_t;
        }
    } else if flags & (FIO_UCS2 as ::core::ffi::c_int | FIO_UTF16 as ::core::ffi::c_int) != 0 {
        if c >= 0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint {
            if flags & FIO_UTF16 as ::core::ffi::c_int != 0 {
                c = c.wrapping_sub(0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint);
                if c >= 0x100000 as ::core::ffi::c_int as ::core::ffi::c_uint {
                    error = true_0 != 0;
                }
                let mut cc: ::core::ffi::c_int = (c >> 10 as ::core::ffi::c_int
                    & 0x3ff as ::core::ffi::c_uint)
                    .wrapping_add(0xd800 as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
                if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                    let c2rust_fresh11 = p;
                    p = p.offset(1);
                    *c2rust_fresh11 = cc as uint8_t;
                    let c2rust_fresh12 = p;
                    p = p.offset(1);
                    *c2rust_fresh12 = (cc >> 8 as ::core::ffi::c_int) as uint8_t;
                } else {
                    let c2rust_fresh13 = p;
                    p = p.offset(1);
                    *c2rust_fresh13 = (cc >> 8 as ::core::ffi::c_int) as uint8_t;
                    let c2rust_fresh14 = p;
                    p = p.offset(1);
                    *c2rust_fresh14 = cc as uint8_t;
                }
                c = (c & 0x3ff as ::core::ffi::c_uint).wrapping_add(0xdc00 as ::core::ffi::c_uint);
            } else {
                error = true_0 != 0;
            }
        }
        if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
            let c2rust_fresh15 = p;
            p = p.offset(1);
            *c2rust_fresh15 = c as uint8_t;
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
        } else {
            let c2rust_fresh17 = p;
            p = p.offset(1);
            *c2rust_fresh17 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh18 = p;
            p = p.offset(1);
            *c2rust_fresh18 = c as uint8_t;
        }
    } else if c >= 0x100 as ::core::ffi::c_uint {
        error = true_0 != 0;
        let c2rust_fresh19 = p;
        p = p.offset(1);
        *c2rust_fresh19 = 0xbf as uint8_t;
    } else {
        let c2rust_fresh20 = p;
        p = p.offset(1);
        *c2rust_fresh20 = c as uint8_t;
    }
    *pp = p as *mut ::core::ffi::c_char;
    return error;
}
unsafe extern "C" fn buf_write_convert_with_iconv(
    mut ip: *mut bw_info,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = *lenp;
    let mut from: *const ::core::ffi::c_char = *bufp;
    let mut fromlen: size_t = len as size_t;
    let mut tolen: size_t = (*ip).bw_conv_buflen;
    let mut to: *mut ::core::ffi::c_char = (*ip).bw_conv_buf;
    if (*ip).bw_first != 0 {
        let mut save_len: size_t = tolen;
        iconv(
            (*ip).bw_iconv_fd,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<size_t>(),
            &raw mut to,
            &raw mut tolen,
        );
        if to.is_null() {
            to = (*ip).bw_conv_buf;
            tolen = save_len;
        }
        (*ip).bw_first = false_0;
    }
    if iconv(
        (*ip).bw_iconv_fd,
        &raw mut from as *mut ::core::ffi::c_void as *mut *mut ::core::ffi::c_char,
        &raw mut fromlen,
        &raw mut to,
        &raw mut tolen,
    ) == -1 as ::core::ffi::c_int as size_t
        && *__errno_location() != ICONV_EINVAL
    {
        (*ip).bw_conv_error = true_0;
        return -1 as ::core::ffi::c_int;
    }
    *bufp = (*ip).bw_conv_buf;
    *lenp = to.offset_from((*ip).bw_conv_buf) as ::core::ffi::c_int;
    return len - fromlen as ::core::ffi::c_int;
}
unsafe extern "C" fn buf_write_convert(
    mut ip: *mut bw_info,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = (*ip).bw_flags;
    let mut wlen: ::core::ffi::c_int = *lenp;
    if flags
        & (FIO_UCS4 as ::core::ffi::c_int
            | FIO_UTF16 as ::core::ffi::c_int
            | FIO_UCS2 as ::core::ffi::c_int
            | FIO_LATIN1 as ::core::ffi::c_int)
        != 0
    {
        let mut c: ::core::ffi::c_uint = 0;
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = if flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
            *bufp
        } else {
            (*ip).bw_conv_buf
        };
        wlen = 0 as ::core::ffi::c_int;
        while wlen < *lenp {
            n = utf_ptr2len_len((*bufp).offset(wlen as isize), *lenp - wlen);
            if n > *lenp - wlen {
                break;
            }
            c = if n > 1 as ::core::ffi::c_int {
                utf_ptr2char((*bufp).offset(wlen as isize)) as ::core::ffi::c_uint
            } else {
                *(*bufp).offset(wlen as isize) as uint8_t as ::core::ffi::c_uint
            };
            if flags & FIO_LATIN1 as ::core::ffi::c_int == 0 {
                let mut need: size_t = (if flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                    4 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                }) as size_t;
                if flags & FIO_UTF16 as ::core::ffi::c_int != 0
                    && c >= 0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    need = 4 as size_t;
                }
                if (p.offset_from((*ip).bw_conv_buf) as size_t).wrapping_add(need)
                    > (*ip).bw_conv_buflen
                {
                    return FAIL;
                }
            }
            if ucs2bytes(c, &raw mut p, flags) as ::core::ffi::c_int != 0
                && (*ip).bw_conv_error == 0
            {
                (*ip).bw_conv_error = true_0;
                (*ip).bw_conv_error_lnum = (*ip).bw_start_lnum;
            }
            if c == NL as ::core::ffi::c_uint {
                (*ip).bw_start_lnum += 1;
            }
            wlen += n;
        }
        if flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
            *lenp = p.offset_from(*bufp) as ::core::ffi::c_int;
        } else {
            *bufp = (*ip).bw_conv_buf;
            *lenp = p.offset_from((*ip).bw_conv_buf) as ::core::ffi::c_int;
        }
    }
    if (*ip).bw_iconv_fd
        != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        )
    {
        return buf_write_convert_with_iconv(ip, bufp, lenp);
    }
    return wlen;
}
unsafe extern "C" fn buf_write_bytes(mut ip: *mut bw_info) -> ::core::ffi::c_int {
    let mut buf: *mut ::core::ffi::c_char = (*ip).bw_buf;
    let mut len: ::core::ffi::c_int = (*ip).bw_len;
    let mut flags: ::core::ffi::c_int = (*ip).bw_flags;
    let mut converted: ::core::ffi::c_int = len;
    let mut remaining: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & FIO_NOCONVERT as ::core::ffi::c_int == 0 {
        converted = buf_write_convert(ip, &raw mut buf, &raw mut len);
        if converted < 0 as ::core::ffi::c_int {
            return FAIL;
        }
        remaining = (*ip).bw_len - converted;
    }
    (*ip).bw_len = remaining;
    if (*ip).bw_fd >= 0 as ::core::ffi::c_int {
        let mut wlen: ::core::ffi::c_int =
            write_eintr((*ip).bw_fd, buf as *mut ::core::ffi::c_void, len as size_t)
                as ::core::ffi::c_int;
        if wlen < len {
            return FAIL;
        }
    }
    if remaining > 0 as ::core::ffi::c_int {
        memmove(
            (*ip).bw_buf as *mut ::core::ffi::c_void,
            (*ip).bw_buf.offset(converted as isize) as *const ::core::ffi::c_void,
            remaining as size_t,
        );
    }
    return OK;
}
unsafe extern "C" fn check_mtime(
    mut buf: *mut buf_T,
    mut file_info: *mut FileInfo,
) -> ::core::ffi::c_int {
    if (*buf).b_mtime_read != 0 as int64_t
        && time_differs(file_info, (*buf).b_mtime_read, (*buf).b_mtime_read_ns)
            as ::core::ffi::c_int
            != 0
    {
        msg_scroll.set(true_0);
        msg_silent.set(0 as ::core::ffi::c_int);
        msg(
            gettext(
                b"WARNING: The file has been changed since reading it!!!\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            HLF_E as ::core::ffi::c_int,
        );
        if ask_yesno(gettext(
            b"Do you really want to write to it\0".as_ptr() as *const ::core::ffi::c_char
        )) == 'n' as ::core::ffi::c_int
        {
            return FAIL;
        }
        msg_scroll.set(false_0);
    }
    return OK;
}
unsafe extern "C" fn make_bom(
    mut buf_in: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut uint8_t = buf_in as *mut uint8_t;
    let mut flags: ::core::ffi::c_int = get_fio_flags(name);
    if flags == FIO_LATIN1 as ::core::ffi::c_int || flags == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if flags == FIO_UTF8 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = 0xef as uint8_t;
        *buf.offset(1 as ::core::ffi::c_int as isize) = 0xbb as uint8_t;
        *buf.offset(2 as ::core::ffi::c_int as isize) = 0xbf as uint8_t;
        return 3 as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char = buf as *mut ::core::ffi::c_char;
    ucs2bytes(0xfeff as ::core::ffi::c_uint, &raw mut p, flags);
    return (p as *mut uint8_t).offset_from(buf) as ::core::ffi::c_int;
}
unsafe extern "C" fn buf_write_do_autocmds(
    mut buf: *mut buf_T,
    mut fnamep: *mut *mut ::core::ffi::c_char,
    mut sfnamep: *mut *mut ::core::ffi::c_char,
    mut ffnamep: *mut *mut ::core::ffi::c_char,
    mut start: linenr_T,
    mut endp: *mut linenr_T,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut filtering: bool,
    mut reset_changed: bool,
    mut overwriting: bool,
    mut whole: bool,
    orig_start: pos_T,
    orig_end: pos_T,
) -> ::core::ffi::c_int {
    let mut old_line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut msg_save: ::core::ffi::c_int = msg_scroll.get();
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut did_cmd: bool = false_0 != 0;
    let mut nofile_err: bool = false_0 != 0;
    let mut empty_memline: bool = (*buf).b_ml.ml_mfp.is_null();
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut sfname: *mut ::core::ffi::c_char = *sfnamep;
    let mut buf_ffname: bool = *ffnamep == (*buf).b_ffname;
    let mut buf_sfname: bool = sfname == (*buf).b_sfname;
    let mut buf_fname_f: bool = *fnamep == (*buf).b_ffname;
    let mut buf_fname_s: bool = *fnamep == (*buf).b_sfname;
    aucmd_prepbuf(&raw mut aco, buf);
    set_bufref(&raw mut bufref, buf);
    if append {
        did_cmd = apply_autocmds_exarg(
            EVENT_FILEAPPENDCMD,
            sfname,
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
        if !did_cmd {
            if overwriting as ::core::ffi::c_int != 0
                && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
            {
                nofile_err = true_0 != 0;
            } else {
                apply_autocmds_exarg(
                    EVENT_FILEAPPENDPRE,
                    sfname,
                    sfname,
                    false_0 != 0,
                    curbuf.get(),
                    eap,
                );
            }
        }
    } else if filtering {
        apply_autocmds_exarg(
            EVENT_FILTERWRITEPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else if reset_changed as ::core::ffi::c_int != 0 && whole as ::core::ffi::c_int != 0 {
        let mut was_changed: bool = curbufIsChanged();
        did_cmd = apply_autocmds_exarg(
            EVENT_BUFWRITECMD,
            sfname,
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
        if did_cmd {
            if was_changed as ::core::ffi::c_int != 0 && !curbufIsChanged() {
                u_unchanged(curbuf.get());
                u_update_save_nr(curbuf.get());
            }
        } else if overwriting as ::core::ffi::c_int != 0
            && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
        {
            nofile_err = true_0 != 0;
        } else {
            apply_autocmds_exarg(
                EVENT_BUFWRITEPRE,
                sfname,
                sfname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        }
    } else {
        did_cmd = apply_autocmds_exarg(
            EVENT_FILEWRITECMD,
            sfname,
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
        if !did_cmd {
            if overwriting as ::core::ffi::c_int != 0
                && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
            {
                nofile_err = true_0 != 0;
            } else {
                apply_autocmds_exarg(
                    EVENT_FILEWRITEPRE,
                    sfname,
                    sfname,
                    false_0 != 0,
                    curbuf.get(),
                    eap,
                );
            }
        }
    }
    aucmd_restbuf(&raw mut aco);
    if !bufref_valid(&raw mut bufref) {
        buf = ::core::ptr::null_mut::<buf_T>();
    }
    if buf.is_null()
        || (*buf).b_ml.ml_mfp.is_null() && !empty_memline
        || did_cmd as ::core::ffi::c_int != 0
        || nofile_err as ::core::ffi::c_int != 0
        || aborting() as ::core::ffi::c_int != 0
    {
        if !buf.is_null() && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0
        {
            (*buf).b_op_start = orig_start;
            (*buf).b_op_end = orig_end;
        }
        (*no_wait_return.ptr()) -= 1;
        msg_scroll.set(msg_save);
        if nofile_err {
            semsg(
                gettext(
                    (e_no_matching_autocommands_for_buftype_str_buffer.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                (*curbuf.get()).b_p_bt,
            );
        }
        if nofile_err as ::core::ffi::c_int != 0 || aborting() as ::core::ffi::c_int != 0 {
            return FAIL;
        }
        if did_cmd {
            if buf.is_null() {
                return OK;
            }
            if overwriting {
                ml_timestamp(buf);
                if append {
                    (*buf).b_flags &= !BF_NEW;
                } else {
                    (*buf).b_flags &= !BF_WRITE_MASK;
                }
            }
            if reset_changed as ::core::ffi::c_int != 0
                && (*buf).b_changed != 0
                && !append
                && (overwriting as ::core::ffi::c_int != 0
                    || !vim_strchr(p_cpo.get(), CPO_PLUS).is_null())
            {
                return FAIL;
            }
            return OK;
        }
        if !aborting() {
            emsg(gettext(
                b"E203: Autocommands deleted or unloaded buffer to be written\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        }
        return FAIL;
    }
    if (*buf).b_ml.ml_line_count != old_line_count {
        if whole {
            *endp = (*buf).b_ml.ml_line_count;
        } else if (*buf).b_ml.ml_line_count > old_line_count {
            *endp += (*buf).b_ml.ml_line_count - old_line_count;
        } else {
            *endp -= old_line_count - (*buf).b_ml.ml_line_count;
            if *endp < start {
                (*no_wait_return.ptr()) -= 1;
                msg_scroll.set(msg_save);
                emsg(gettext(
                    b"E204: Autocommand changed number of lines in unexpected way\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
        }
    }
    if buf_ffname {
        *ffnamep = (*buf).b_ffname;
    }
    if buf_sfname {
        *sfnamep = (*buf).b_sfname;
    }
    if buf_fname_f {
        *fnamep = (*buf).b_ffname;
    }
    if buf_fname_s {
        *fnamep = (*buf).b_sfname;
    }
    return NOTDONE;
}
unsafe extern "C" fn buf_write_do_post_autocmds(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut filtering: bool,
    mut reset_changed: bool,
    mut whole: bool,
) {
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    (*curbuf.get()).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
    aucmd_prepbuf(&raw mut aco, buf);
    if append {
        apply_autocmds_exarg(
            EVENT_FILEAPPENDPOST,
            fname,
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else if filtering {
        apply_autocmds_exarg(
            EVENT_FILTERWRITEPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else if reset_changed as ::core::ffi::c_int != 0 && whole as ::core::ffi::c_int != 0 {
        apply_autocmds_exarg(
            EVENT_BUFWRITEPOST,
            fname,
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else {
        apply_autocmds_exarg(
            EVENT_FILEWRITEPOST,
            fname,
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    }
    aucmd_restbuf(&raw mut aco);
}
#[inline]
unsafe extern "C" fn set_err_num(
    mut num: *const ::core::ffi::c_char,
    mut msg_0: *const ::core::ffi::c_char,
) -> Error_T {
    return Error_T {
        num: num,
        msg: msg_0 as *mut ::core::ffi::c_char,
        arg: 0 as ::core::ffi::c_int,
        alloc: false,
    };
}
#[inline]
unsafe extern "C" fn set_err(mut msg_0: *const ::core::ffi::c_char) -> Error_T {
    return Error_T {
        num: ::core::ptr::null::<::core::ffi::c_char>(),
        msg: msg_0 as *mut ::core::ffi::c_char,
        arg: 0 as ::core::ffi::c_int,
        alloc: false,
    };
}
#[inline]
unsafe extern "C" fn set_err_arg(
    mut msg_0: *const ::core::ffi::c_char,
    mut arg: ::core::ffi::c_int,
) -> Error_T {
    return Error_T {
        num: ::core::ptr::null::<::core::ffi::c_char>(),
        msg: msg_0 as *mut ::core::ffi::c_char,
        arg: arg,
        alloc: false,
    };
}
unsafe extern "C" fn emit_err(mut e: *mut Error_T) {
    if !(*e).num.is_null() {
        if (*e).arg != 0 as ::core::ffi::c_int {
            semsg(
                b"%s: %s%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                (*e).num,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                (*e).msg,
                uv_strerror((*e).arg),
            );
        } else {
            semsg(
                b"%s: %s%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*e).num,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                (*e).msg,
            );
        }
    } else if (*e).arg != 0 as ::core::ffi::c_int {
        semsg((*e).msg, uv_strerror((*e).arg));
    } else {
        emsg((*e).msg);
    }
    if (*e).alloc {
        xfree((*e).msg as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn get_fileinfo_os(
    mut fname: *mut ::core::ffi::c_char,
    mut file_info_old: *mut FileInfo,
    mut _overwriting: bool,
    mut perm: *mut ::core::ffi::c_int,
    mut device: *mut bool,
    mut newfile: *mut bool,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    *perm = -1 as ::core::ffi::c_int;
    if !os_fileinfo(fname, file_info_old) {
        *newfile = true_0 != 0;
    } else {
        *perm = (*file_info_old).stat.st_mode as ::core::ffi::c_int;
        if !((*file_info_old).stat.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t) {
            if (*file_info_old).stat.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t {
                *err = set_err_num(
                    b"E502\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"is a directory\0".as_ptr() as *const ::core::ffi::c_char),
                );
                return FAIL;
            }
            if os_nodetype(fname) != NODE_WRITABLE {
                *err = set_err_num(
                    b"E503\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"is not a file or writable device\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
                return FAIL;
            }
            *device = true_0 != 0;
            *newfile = true_0 != 0;
            *perm = -1 as ::core::ffi::c_int;
        }
    }
    return OK;
}
unsafe extern "C" fn get_fileinfo(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut overwriting: bool,
    mut forceit: bool,
    mut file_info_old: *mut FileInfo,
    mut perm: *mut ::core::ffi::c_int,
    mut device: *mut bool,
    mut newfile: *mut bool,
    mut readonly: *mut bool,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    if get_fileinfo_os(
        fname,
        file_info_old,
        overwriting,
        perm,
        device,
        newfile,
        err,
    ) == FAIL
    {
        return FAIL;
    }
    *readonly = false_0 != 0;
    if !*device && !*newfile {
        *readonly = os_file_is_writable(fname) == 0;
        if !forceit && *readonly as ::core::ffi::c_int != 0 {
            if !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null() {
                *err = set_err_num(
                    b"E504\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(err_readonly.get()),
                );
            } else {
                *err = set_err_num(
                    b"E505\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"is read-only (add ! to override)\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
            }
            return FAIL;
        }
        if overwriting as ::core::ffi::c_int != 0 && !forceit {
            let mut retval: ::core::ffi::c_int = check_mtime(buf, file_info_old);
            if retval == FAIL {
                return FAIL;
            }
        }
    }
    return OK;
}
pub unsafe extern "C" fn buf_get_backup_name(
    mut fname: *mut ::core::ffi::c_char,
    mut dirp: *mut *mut ::core::ffi::c_char,
    mut no_prepend_dot: bool,
    mut backup_ext: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut backup: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dir_len: size_t = copy_option_part(
        dirp,
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    let mut p: *mut ::core::ffi::c_char =
        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(dir_len as isize);
    if **dirp as ::core::ffi::c_int == NUL && !os_isdir(IObuff.ptr() as *mut ::core::ffi::c_char) {
        let mut ret: ::core::ffi::c_int = 0;
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        ret = os_mkdir_recurse(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0o755 as int32_t,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        if ret != 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    b"E303: Unable to create directory \"%s\" for backup file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                failed_dir,
                uv_strerror(ret),
            );
            xfree(failed_dir as *mut ::core::ffi::c_void);
        }
    }
    if dir_len > 1 as size_t
        && after_pathsep(IObuff.ptr() as *mut ::core::ffi::c_char, p) != 0
        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        p = make_percent_swname(IObuff.ptr() as *mut ::core::ffi::c_char, p, fname);
        if !p.is_null() {
            backup = modname(p, backup_ext, no_prepend_dot);
            xfree(p as *mut ::core::ffi::c_void);
        }
    }
    if backup.is_null() {
        let mut rootname: *mut ::core::ffi::c_char =
            get_file_in_dir(fname, IObuff.ptr() as *mut ::core::ffi::c_char);
        if !rootname.is_null() {
            backup = modname(rootname, backup_ext, no_prepend_dot);
            xfree(rootname as *mut ::core::ffi::c_void);
        }
    }
    return backup;
}
unsafe extern "C" fn buf_write_make_backup(
    mut fname: *mut ::core::ffi::c_char,
    mut append: bool,
    mut file_info_old: *mut FileInfo,
    mut acl: vim_acl_T,
    mut perm: ::core::ffi::c_int,
    mut bkc: ::core::ffi::c_uint,
    mut file_readonly: bool,
    mut forceit: bool,
    mut backup_copyp: *mut bool,
    mut backupp: *mut *mut ::core::ffi::c_char,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    let mut file_info: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let no_prepend_dot: bool = false_0 != 0;
    if bkc & kOptBkcFlagYes as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || append as ::core::ffi::c_int != 0
    {
        *backup_copyp = true_0 != 0;
    } else if bkc & kOptBkcFlagAuto as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if os_fileinfo_hardlinks(file_info_old) > 1 as uint64_t
            || !os_fileinfo_link(fname, &raw mut file_info)
            || !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
        {
            *backup_copyp = true_0 != 0;
        } else {
            let mut dirlen: size_t = path_tail(fname).offset_from(fname) as size_t;
            '_c2rust_label: {
                if dirlen < 4096 as size_t {
                } else {
                    __assert_fail(
                        b"dirlen < MAXPATHL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/bufwrite.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        743 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            let mut tmp_fname: [::core::ffi::c_char; 4096] = [0; 4096];
            xmemcpyz(
                &raw mut tmp_fname as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                dirlen,
            );
            let mut i: ::core::ffi::c_int = 4913 as ::core::ffi::c_int;
            loop {
                snprintf(
                    (&raw mut tmp_fname as *mut ::core::ffi::c_char).offset(dirlen as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>().wrapping_sub(dirlen),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                );
                if !os_fileinfo_link(
                    &raw mut tmp_fname as *mut ::core::ffi::c_char,
                    &raw mut file_info,
                ) {
                    break;
                }
                i += 123 as ::core::ffi::c_int;
            }
            let mut fd: ::core::ffi::c_int = os_open(
                &raw mut tmp_fname as *mut ::core::ffi::c_char,
                O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW,
                perm,
            );
            if fd < 0 as ::core::ffi::c_int {
                *backup_copyp = true_0 != 0;
            } else {
                os_fchown(
                    fd,
                    (*file_info_old).stat.st_uid as uv_uid_t,
                    (*file_info_old).stat.st_gid as uv_gid_t,
                );
                if !os_fileinfo(
                    &raw mut tmp_fname as *mut ::core::ffi::c_char,
                    &raw mut file_info,
                ) || file_info.stat.st_uid != (*file_info_old).stat.st_uid
                    || file_info.stat.st_gid != (*file_info_old).stat.st_gid
                    || file_info.stat.st_mode as ::core::ffi::c_int != perm
                {
                    *backup_copyp = true_0 != 0;
                }
                close(fd);
                os_remove(&raw mut tmp_fname as *mut ::core::ffi::c_char);
            }
        }
    }
    if bkc & kOptBkcFlagBreaksymlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || bkc & kOptBkcFlagBreakhardlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        let mut file_info_link_ok: bool = os_fileinfo_link(fname, &raw mut file_info);
        if bkc & kOptBkcFlagBreaksymlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && file_info_link_ok as ::core::ffi::c_int != 0
            && !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
        {
            *backup_copyp = false_0 != 0;
        }
        if bkc & kOptBkcFlagBreakhardlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && os_fileinfo_hardlinks(file_info_old) > 1 as uint64_t
            && (!file_info_link_ok
                || os_fileinfo_id_equal(&raw mut file_info, file_info_old) as ::core::ffi::c_int
                    != 0)
        {
            *backup_copyp = false_0 != 0;
        }
    }
    let mut backup_ext: *mut ::core::ffi::c_char = (if *p_bex.get() as ::core::ffi::c_int == NUL {
        b".bak\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        p_bex.get() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    if *backup_copyp {
        let mut some_error: bool = false_0 != 0;
        let mut dirp: *mut ::core::ffi::c_char = p_bdir.get();
        while *dirp != 0 {
            *backupp = buf_get_backup_name(fname, &raw mut dirp, no_prepend_dot, backup_ext);
            if (*backupp).is_null() {
                some_error = true_0 != 0;
                break;
            } else {
                let mut file_info_new: FileInfo = FileInfo {
                    stat: uv_stat_t {
                        st_dev: 0,
                        st_mode: 0,
                        st_nlink: 0,
                        st_uid: 0,
                        st_gid: 0,
                        st_rdev: 0,
                        st_ino: 0,
                        st_size: 0,
                        st_blksize: 0,
                        st_blocks: 0,
                        st_flags: 0,
                        st_gen: 0,
                        st_atim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_mtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_ctim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                        st_birthtim: uv_timespec_t {
                            tv_sec: 0,
                            tv_nsec: 0,
                        },
                    },
                };
                if os_fileinfo(*backupp, &raw mut file_info_new) {
                    if os_fileinfo_id_equal(&raw mut file_info_new, file_info_old) {
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            backupp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        let _ = *ptr_;
                    } else if p_bk.get() == 0 {
                        let mut wp: *mut ::core::ffi::c_char = (*backupp)
                            .offset(strlen(*backupp) as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize))
                            .offset(-(strlen(backup_ext) as isize));
                        wp = if wp > *backupp { wp } else { *backupp };
                        *wp = 'z' as ::core::ffi::c_char;
                        while *wp as ::core::ffi::c_int > 'a' as ::core::ffi::c_int
                            && os_fileinfo(*backupp, &raw mut file_info_new) as ::core::ffi::c_int
                                != 0
                        {
                            *wp -= 1;
                        }
                        if *wp as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                backupp as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__0);
                            *ptr__0 = NULL;
                            let _ = *ptr__0;
                        }
                    }
                }
                if (*backupp).is_null() {
                    continue;
                }
                os_remove(*backupp);
                if os_copy(fname, *backupp, UV_FS_COPYFILE_FICLONE) != 0 as ::core::ffi::c_int {
                    *err = set_err(gettext(
                        b"E509: Cannot create backup file (add ! to override)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                        backupp as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__1);
                    *ptr__1 = NULL;
                    let _ = *ptr__1;
                    *backupp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    os_setperm(*backupp, perm & 0o777 as ::core::ffi::c_int);
                    if file_info_new.stat.st_gid != (*file_info_old).stat.st_gid
                        && os_chown(
                            *backupp,
                            -1 as ::core::ffi::c_int as uv_uid_t,
                            (*file_info_old).stat.st_gid as uv_gid_t,
                        ) != 0 as ::core::ffi::c_int
                    {
                        os_setperm(
                            *backupp,
                            perm & 0o707 as ::core::ffi::c_int
                                | (perm & 0o7 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int,
                        );
                    }
                    os_file_settime(
                        *backupp,
                        (*file_info_old).stat.st_atim.tv_sec as ::core::ffi::c_double,
                        (*file_info_old).stat.st_mtim.tv_sec as ::core::ffi::c_double,
                    );
                    os_set_acl(*backupp, acl);
                    os_copy_xattr(fname, *backupp);
                    *err = set_err(::core::ptr::null::<::core::ffi::c_char>());
                    break;
                }
            }
        }
        if (*backupp).is_null() && (*err).msg.is_null() {
            *err = set_err(gettext(
                b"E509: Cannot create backup file (add ! to override)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        }
        if (some_error as ::core::ffi::c_int != 0 || !(*err).msg.is_null()) && !forceit {
            return FAIL;
        }
        *err = set_err(::core::ptr::null::<::core::ffi::c_char>());
    } else {
        if file_readonly as ::core::ffi::c_int != 0
            && !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
        {
            *err = set_err_num(
                b"E504\0".as_ptr() as *const ::core::ffi::c_char,
                gettext(err_readonly.get()),
            );
            return FAIL;
        }
        let mut dirp_0: *mut ::core::ffi::c_char = p_bdir.get();
        while *dirp_0 != 0 {
            *backupp = buf_get_backup_name(fname, &raw mut dirp_0, no_prepend_dot, backup_ext);
            if !(*backupp).is_null() {
                if p_bk.get() == 0 && os_path_exists(*backupp) as ::core::ffi::c_int != 0 {
                    let mut p: *mut ::core::ffi::c_char = (*backupp)
                        .offset(strlen(*backupp) as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize))
                        .offset(-(strlen(backup_ext) as isize));
                    p = if p > *backupp { p } else { *backupp };
                    *p = 'z' as ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int > 'a' as ::core::ffi::c_int
                        && os_path_exists(*backupp) as ::core::ffi::c_int != 0
                    {
                        *p -= 1;
                    }
                    if *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                        let mut ptr__2: *mut *mut ::core::ffi::c_void =
                            backupp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__2);
                        *ptr__2 = NULL;
                        let _ = *ptr__2;
                    }
                }
            }
            if (*backupp).is_null() {
                continue;
            }
            if vim_rename(fname, *backupp) == 0 as ::core::ffi::c_int {
                break;
            }
            let mut ptr__3: *mut *mut ::core::ffi::c_void =
                backupp as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__3);
            *ptr__3 = NULL;
            let _ = *ptr__3;
        }
        if (*backupp).is_null() && !forceit {
            *err = set_err(gettext(
                b"E510: Can't make backup file (add ! to override)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
    }
    return OK;
}
pub unsafe extern "C" fn buf_write(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut start: linenr_T,
    mut end: linenr_T,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut forceit: bool,
    mut reset_changed: bool,
    mut filtering: bool,
) -> ::core::ffi::c_int {
    let mut fenc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut converted: bool = false;
    let mut wb_flags: ::core::ffi::c_int = 0;
    let mut notconverted: bool = false;
    let mut no_eol: bool = false;
    let mut nchars: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut fileformat: ::core::ffi::c_int = 0;
    let mut checking_conversion: bool = false;
    let mut fd: ::core::ffi::c_int = 0;
    let mut fflags: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = 0;
    let mut dobackup: bool = false;
    let mut backup_copy: bool = false;
    let mut made_writable: bool = false;
    let mut wfname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = OK;
    let mut msg_save: ::core::ffi::c_int = msg_scroll.get();
    let mut prev_got_int: bool = got_int.get();
    let mut whole: bool = start == 1 as linenr_T && end == (*buf).b_ml.ml_line_count;
    let mut write_undo_file: bool = false_0 != 0;
    let mut sha_ctx = Sha256::new();
    let mut bkc: ::core::ffi::c_uint = get_bkc_flags(buf);
    if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    if (*buf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_empty_buffer as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if check_secure() {
        return FAIL;
    }
    if strlen(fname) >= MAXPATHL as size_t {
        emsg(gettext(&raw const e_longname as *const ::core::ffi::c_char));
        return FAIL;
    }
    let mut write_info: bw_info = bw_info {
        bw_fd: 0,
        bw_buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        bw_len: 0,
        bw_flags: 0,
        bw_first: 0,
        bw_conv_buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        bw_conv_buflen: 0,
        bw_conv_error: 0,
        bw_conv_error_lnum: 0,
        bw_start_lnum: 0,
        bw_iconv_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    write_info.bw_conv_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    write_info.bw_conv_error = false_0;
    write_info.bw_conv_error_lnum = 0 as ::core::ffi::c_int as linenr_T;
    write_info.bw_iconv_fd = ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
        -1 as ::core::ffi::c_int as usize,
    );
    ex_no_reprint.set(true_0 != 0);
    if (*buf).b_ffname.is_null()
        && reset_changed as ::core::ffi::c_int != 0
        && whole as ::core::ffi::c_int != 0
        && buf == curbuf.get()
        && !bt_nofilename(buf)
        && !filtering
        && (!append || !vim_strchr(p_cpo.get(), CPO_FNAMEAPP).is_null())
        && !vim_strchr(p_cpo.get(), CPO_FNAMEW).is_null()
    {
        if set_rw_fname(fname, sfname) == FAIL {
            return FAIL;
        }
        buf = curbuf.get();
    }
    if sfname.is_null() {
        sfname = fname;
    }
    let mut ffname: *mut ::core::ffi::c_char = fname;
    fname = sfname;
    let mut overwriting: bool = !(*buf).b_ffname.is_null()
        && path_fnamecmp(ffname, (*buf).b_ffname) == 0 as ::core::ffi::c_int;
    (*no_wait_return.ptr()) += 1;
    let orig_start: pos_T = (*buf).b_op_start;
    let orig_end: pos_T = (*buf).b_op_end;
    (*buf).b_op_start.lnum = start;
    (*buf).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
    (*buf).b_op_end.lnum = end;
    (*buf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
    let mut res: ::core::ffi::c_int = buf_write_do_autocmds(
        buf,
        &raw mut fname,
        &raw mut sfname,
        &raw mut ffname,
        start,
        &raw mut end,
        eap,
        append,
        filtering,
        reset_changed,
        overwriting,
        whole,
        orig_start,
        orig_end,
    );
    if res != NOTDONE {
        return res;
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        (*buf).b_op_start = orig_start;
        (*buf).b_op_end = orig_end;
    }
    if shortmess(SHM_OVER as ::core::ffi::c_int) as ::core::ffi::c_int != 0 && !exiting.get() {
        msg_scroll.set(false_0);
    } else {
        msg_scroll.set(true_0);
    }
    if !filtering {
        filemess(
            buf,
            fname,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
    msg_scroll.set(false_0);
    let mut buffer: *mut ::core::ffi::c_char =
        verbose_try_malloc(WRITEBUFSIZE as ::core::ffi::c_int as size_t)
            as *mut ::core::ffi::c_char;
    let mut bufsize: ::core::ffi::c_int = 0;
    let mut smallbuf: [::core::ffi::c_char; 256] = [0; 256];
    if buffer.is_null() {
        buffer = &raw mut smallbuf as *mut ::core::ffi::c_char;
        bufsize = SMALLBUFSIZE;
    } else {
        bufsize = WRITEBUFSIZE as ::core::ffi::c_int;
    }
    let mut err: Error_T = Error_T {
        num: ::core::ptr::null::<::core::ffi::c_char>(),
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        arg: 0,
        alloc: false,
    };
    let mut perm: ::core::ffi::c_int = 0;
    let mut newfile: bool = false_0 != 0;
    let mut device: bool = false_0 != 0;
    let mut file_readonly: bool = false_0 != 0;
    let mut backup: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fenc_tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut file_info_old: FileInfo = FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_birthtim: uv_timespec_t {
                tv_sec: 0,
                tv_nsec: 0,
            },
        },
    };
    let mut acl: vim_acl_T = NULL;
    '_nofail: {
        '_fail: {
            if get_fileinfo(
                buf,
                fname,
                overwriting,
                forceit,
                &raw mut file_info_old,
                &raw mut perm,
                &raw mut device,
                &raw mut newfile,
                &raw mut file_readonly,
                &raw mut err,
            ) != FAIL
            {
                if !newfile {
                    acl = os_get_acl(fname);
                }
                dobackup =
                    p_wb.get() != 0 || p_bk.get() != 0 || *p_pm.get() as ::core::ffi::c_int != NUL;
                if dobackup as ::core::ffi::c_int != 0
                    && *p_bsk.get() as ::core::ffi::c_int != NUL
                    && match_file_list(p_bsk.get(), sfname, ffname) as ::core::ffi::c_int != 0
                {
                    dobackup = false_0 != 0;
                }
                backup_copy = false_0 != 0;
                prev_got_int = got_int.get();
                got_int.set(false_0 != 0);
                (*buf).b_saving = true_0 != 0;
                if !(append as ::core::ffi::c_int != 0 && *p_pm.get() as ::core::ffi::c_int == NUL)
                    && !filtering
                    && perm >= 0 as ::core::ffi::c_int
                    && dobackup as ::core::ffi::c_int != 0
                {
                    if buf_write_make_backup(
                        fname,
                        append,
                        &raw mut file_info_old,
                        acl,
                        perm,
                        bkc,
                        file_readonly,
                        forceit,
                        &raw mut backup_copy,
                        &raw mut backup,
                        &raw mut err,
                    ) == FAIL
                    {
                        retval = FAIL;
                        break '_fail;
                    }
                }
                made_writable = false_0 != 0;
                if forceit as ::core::ffi::c_int != 0
                    && perm >= 0 as ::core::ffi::c_int
                    && perm & 0o200 as ::core::ffi::c_int == 0
                    && file_info_old.stat.st_uid == getuid() as uint64_t
                    && vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
                {
                    perm |= 0o200 as ::core::ffi::c_int;
                    os_setperm(fname, perm);
                    made_writable = true_0 != 0;
                }
                if forceit as ::core::ffi::c_int != 0
                    && overwriting as ::core::ffi::c_int != 0
                    && vim_strchr(p_cpo.get(), CPO_KEEPRO).is_null()
                {
                    (*buf).b_p_ro = false_0;
                    need_maketitle.set(true_0 != 0);
                    status_redraw_all();
                }
                end = if end < (*buf).b_ml.ml_line_count {
                    end
                } else {
                    (*buf).b_ml.ml_line_count
                };
                if (*buf).b_ml.ml_flags & ML_EMPTY != 0 {
                    start = end + 1 as linenr_T;
                }
                wfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                '_restore_backup: {
                    if reset_changed as ::core::ffi::c_int != 0
                        && !newfile
                        && overwriting as ::core::ffi::c_int != 0
                        && !(exiting.get() as ::core::ffi::c_int != 0 && !backup.is_null())
                    {
                        ml_preserve(
                            buf,
                            false_0 != 0,
                            if (*buf).b_p_fs >= 0 as ::core::ffi::c_int {
                                (*buf).b_p_fs
                            } else {
                                p_fs.get()
                            } != 0,
                        );
                        if got_int.get() {
                            err =
                                set_err(gettext(&raw const e_interr as *const ::core::ffi::c_char));
                            break '_restore_backup;
                        }
                    }
                    wfname = fname;
                    fenc = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    if !eap.is_null() && (*eap).force_enc != 0 as ::core::ffi::c_int {
                        fenc = (*eap).cmd.offset((*eap).force_enc as isize);
                        fenc = enc_canonize(fenc);
                        fenc_tofree = fenc;
                    } else {
                        fenc = (*buf).b_p_fenc;
                    }
                    converted = need_conversion(fenc);
                    wb_flags = 0 as ::core::ffi::c_int;
                    if converted {
                        wb_flags = get_fio_flags(fenc);
                        if wb_flags
                            & (FIO_UCS2 as ::core::ffi::c_int
                                | FIO_UCS4 as ::core::ffi::c_int
                                | FIO_UTF16 as ::core::ffi::c_int
                                | FIO_UTF8 as ::core::ffi::c_int)
                            != 0
                        {
                            if wb_flags
                                & (FIO_UCS2 as ::core::ffi::c_int
                                    | FIO_UTF16 as ::core::ffi::c_int
                                    | FIO_UTF8 as ::core::ffi::c_int)
                                != 0
                            {
                                write_info.bw_conv_buflen =
                                    (bufsize as size_t).wrapping_mul(2 as size_t);
                            } else {
                                write_info.bw_conv_buflen =
                                    (bufsize as size_t).wrapping_mul(4 as size_t);
                            }
                            write_info.bw_conv_buf = verbose_try_malloc(write_info.bw_conv_buflen)
                                as *mut ::core::ffi::c_char;
                            if write_info.bw_conv_buf.is_null() {
                                end = 0 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                    }
                    if converted as ::core::ffi::c_int != 0 && wb_flags == 0 as ::core::ffi::c_int {
                        write_info.bw_iconv_fd = my_iconv_open(
                            fenc,
                            b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                        if write_info.bw_iconv_fd
                            != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                -1 as ::core::ffi::c_int as usize,
                            )
                        {
                            write_info.bw_conv_buflen = (bufsize as size_t)
                                .wrapping_mul(ICONV_MULT as ::core::ffi::c_int as size_t);
                            write_info.bw_conv_buf = verbose_try_malloc(write_info.bw_conv_buflen)
                                as *mut ::core::ffi::c_char;
                            if write_info.bw_conv_buf.is_null() {
                                end = 0 as ::core::ffi::c_int as linenr_T;
                            }
                            write_info.bw_first = true_0;
                        } else if *p_ccv.get() as ::core::ffi::c_int != NUL {
                            wfname = vim_tempname();
                            if wfname.is_null() {
                                err = set_err(gettext(
                                    b"E214: Can't find temp file for writing\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                                break '_restore_backup;
                            }
                        }
                    }
                    notconverted = false_0 != 0;
                    if converted as ::core::ffi::c_int != 0
                        && wb_flags == 0 as ::core::ffi::c_int
                        && write_info.bw_iconv_fd
                            == ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                -1 as ::core::ffi::c_int as usize,
                            )
                        && wfname == fname
                    {
                        if !forceit {
                            err = set_err(gettext(
                                b"E213: Cannot convert (add ! to write without conversion)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                            break '_restore_backup;
                        } else {
                            notconverted = true_0 != 0;
                        }
                    }
                    no_eol = false_0 != 0;
                    nchars = 0;
                    lnum = 0;
                    fileformat = 0;
                    checking_conversion = false;
                    fd = 0;
                    checking_conversion = true_0 != 0;
                    loop {
                        if !converted || dobackup as ::core::ffi::c_int != 0 {
                            checking_conversion = false_0 != 0;
                        }
                        's_777: {
                            if checking_conversion {
                                fd = -1 as ::core::ffi::c_int;
                                write_info.bw_fd = fd;
                            } else {
                                fflags = O_WRONLY
                                    | (if append as ::core::ffi::c_int != 0 {
                                        if forceit as ::core::ffi::c_int != 0 {
                                            O_APPEND | O_CREAT
                                        } else {
                                            O_APPEND
                                        }
                                    } else {
                                        O_CREAT | O_TRUNC
                                    });
                                mode = if perm < 0 as ::core::ffi::c_int {
                                    0o666 as ::core::ffi::c_int
                                } else {
                                    perm & 0o777 as ::core::ffi::c_int
                                };
                                loop {
                                    fd = os_open(wfname, fflags, mode);
                                    if fd < 0 as ::core::ffi::c_int {
                                        if !err.msg.is_null() {
                                            break '_restore_backup;
                                        }
                                        let mut file_info: FileInfo = FileInfo {
                                            stat: uv_stat_t {
                                                st_dev: 0,
                                                st_mode: 0,
                                                st_nlink: 0,
                                                st_uid: 0,
                                                st_gid: 0,
                                                st_rdev: 0,
                                                st_ino: 0,
                                                st_size: 0,
                                                st_blksize: 0,
                                                st_blocks: 0,
                                                st_flags: 0,
                                                st_gen: 0,
                                                st_atim: uv_timespec_t {
                                                    tv_sec: 0,
                                                    tv_nsec: 0,
                                                },
                                                st_mtim: uv_timespec_t {
                                                    tv_sec: 0,
                                                    tv_nsec: 0,
                                                },
                                                st_ctim: uv_timespec_t {
                                                    tv_sec: 0,
                                                    tv_nsec: 0,
                                                },
                                                st_birthtim: uv_timespec_t {
                                                    tv_sec: 0,
                                                    tv_nsec: 0,
                                                },
                                            },
                                        };
                                        if !newfile
                                            && os_fileinfo_hardlinks(&raw mut file_info_old)
                                                > 1 as uint64_t
                                            || os_fileinfo_link(fname, &raw mut file_info)
                                                as ::core::ffi::c_int
                                                != 0
                                                && !os_fileinfo_id_equal(
                                                    &raw mut file_info,
                                                    &raw mut file_info_old,
                                                )
                                        {
                                            err = set_err(gettext(
                                                b"E166: Can't open linked file for writing\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ));
                                            break '_restore_backup;
                                        } else {
                                            err = set_err_arg(
                                                gettext(
                                                    b"E212: Can't open file for writing: %s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                fd,
                                            );
                                            if !(forceit as ::core::ffi::c_int != 0
                                                && vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
                                                && perm >= 0 as ::core::ffi::c_int)
                                            {
                                                break '_restore_backup;
                                            }
                                            if perm & 0o200 as ::core::ffi::c_int == 0 {
                                                made_writable = true_0 != 0;
                                            }
                                            perm |= 0o200 as ::core::ffi::c_int;
                                            if file_info_old.stat.st_uid != getuid() as uint64_t
                                                || file_info_old.stat.st_gid != getgid() as uint64_t
                                            {
                                                perm &= 0o777 as ::core::ffi::c_int;
                                            }
                                            if !append {
                                                os_remove(wfname);
                                            }
                                        }
                                    } else {
                                        write_info.bw_fd = fd;
                                        break 's_777;
                                    }
                                }
                            }
                        }
                        err = set_err(::core::ptr::null::<::core::ffi::c_char>());
                        write_info.bw_buf = buffer;
                        nchars = 0 as ::core::ffi::c_int;
                        let mut write_bin: ::core::ffi::c_int = 0;
                        if !eap.is_null() && (*eap).force_bin != 0 as ::core::ffi::c_int {
                            write_bin = ((*eap).force_bin == FORCE_BIN) as ::core::ffi::c_int;
                        } else {
                            write_bin = (*buf).b_p_bin;
                        }
                        if (*buf).b_p_bomb != 0
                            && write_bin == 0
                            && (!append || perm < 0 as ::core::ffi::c_int)
                        {
                            write_info.bw_len = make_bom(buffer, fenc);
                            if write_info.bw_len > 0 as ::core::ffi::c_int {
                                write_info.bw_flags =
                                    FIO_NOCONVERT as ::core::ffi::c_int | wb_flags;
                                if buf_write_bytes(&raw mut write_info) == FAIL {
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                } else {
                                    nchars += write_info.bw_len;
                                }
                            }
                        }
                        write_info.bw_start_lnum = start;
                        write_undo_file = (*buf).b_p_udf != 0
                            && overwriting as ::core::ffi::c_int != 0
                            && !append
                            && !filtering
                            && reset_changed as ::core::ffi::c_int != 0
                            && !checking_conversion;
                        if write_undo_file {
                            sha_ctx = Sha256::new();
                        }
                        write_info.bw_len = 0 as ::core::ffi::c_int;
                        write_info.bw_flags = wb_flags;
                        fileformat = get_fileformat_force(buf, eap);
                        let mut s: *mut ::core::ffi::c_char = buffer;
                        lnum = start;
                        while lnum <= end {
                            let mut ptr: *mut ::core::ffi::c_char =
                                ml_get_buf(buf, lnum).offset(-(1 as ::core::ffi::c_int as isize));
                            if write_undo_file {
                                let line = ptr.offset(1 as ::core::ffi::c_int as isize);
                                // Include the terminating NUL as a line separator.
                                sha_ctx.update(::core::slice::from_raw_parts(
                                    line as *const u8,
                                    strlen(line) + 1,
                                ));
                            }
                            let mut c: ::core::ffi::c_char = 0;
                            loop {
                                ptr = ptr.offset(1);
                                c = *ptr;
                                if c as ::core::ffi::c_int == NUL {
                                    break;
                                }
                                if c as ::core::ffi::c_int == NL {
                                    *s = NUL as ::core::ffi::c_char;
                                } else if c as ::core::ffi::c_int == CAR && fileformat == EOL_MAC {
                                    *s = NL as ::core::ffi::c_char;
                                } else {
                                    *s = c;
                                }
                                s = s.offset(1);
                                write_info.bw_len += 1;
                                if write_info.bw_len != bufsize {
                                    continue;
                                }
                                if buf_write_bytes(&raw mut write_info) == FAIL {
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                    break;
                                } else {
                                    nchars += bufsize - write_info.bw_len;
                                    s = buffer.offset(write_info.bw_len as isize);
                                    write_info.bw_start_lnum = lnum;
                                }
                            }
                            if end == 0 as linenr_T
                                || lnum == end
                                    && (write_bin != 0 || (*buf).b_p_fixeol == 0)
                                    && (write_bin != 0 && lnum == (*buf).b_no_eol_lnum
                                        || lnum == (*buf).b_ml.ml_line_count && (*buf).b_p_eol == 0)
                            {
                                lnum += 1;
                                no_eol = true_0 != 0;
                                break;
                            } else {
                                if fileformat == EOL_UNIX {
                                    let c2rust_fresh0 = s;
                                    s = s.offset(1);
                                    *c2rust_fresh0 = NL as ::core::ffi::c_char;
                                } else {
                                    let c2rust_fresh1 = s;
                                    s = s.offset(1);
                                    *c2rust_fresh1 = CAR as ::core::ffi::c_char;
                                    if fileformat == EOL_DOS {
                                        write_info.bw_len += 1;
                                        if write_info.bw_len == bufsize {
                                            if buf_write_bytes(&raw mut write_info) == FAIL {
                                                end = 0 as ::core::ffi::c_int as linenr_T;
                                                break;
                                            } else {
                                                nchars += bufsize - write_info.bw_len;
                                                s = buffer.offset(write_info.bw_len as isize);
                                            }
                                        }
                                        let c2rust_fresh2 = s;
                                        s = s.offset(1);
                                        *c2rust_fresh2 = NL as ::core::ffi::c_char;
                                    }
                                }
                                write_info.bw_len += 1;
                                if write_info.bw_len == bufsize {
                                    if buf_write_bytes(&raw mut write_info) == FAIL {
                                        end = 0 as ::core::ffi::c_int as linenr_T;
                                        break;
                                    } else {
                                        nchars += bufsize - write_info.bw_len;
                                        s = buffer.offset(write_info.bw_len as isize);
                                        os_breakcheck();
                                        if got_int.get() {
                                            end = 0 as ::core::ffi::c_int as linenr_T;
                                            break;
                                        }
                                    }
                                }
                                lnum += 1;
                            }
                        }
                        if write_info.bw_len > 0 as ::core::ffi::c_int && end > 0 as linenr_T {
                            let mut remaining: ::core::ffi::c_int = write_info.bw_len;
                            if buf_write_bytes(&raw mut write_info) == FAIL {
                                end = 0 as ::core::ffi::c_int as linenr_T;
                            }
                            nchars += remaining - write_info.bw_len;
                        }
                        if end != 0 as linenr_T && write_info.bw_len > 0 as ::core::ffi::c_int {
                            write_info.bw_conv_error = true_0;
                            write_info.bw_conv_error_lnum = end;
                            end = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        if (*buf).b_p_fixeol == 0 && (*buf).b_p_eof != 0 {
                            write_eintr(
                                write_info.bw_fd,
                                b"\x1A\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                1 as size_t,
                            );
                        }
                        if !checking_conversion || end == 0 as linenr_T {
                            if !checking_conversion {
                                let mut error: ::core::ffi::c_int = 0;
                                if (if (*buf).b_p_fs >= 0 as ::core::ffi::c_int {
                                    (*buf).b_p_fs
                                } else {
                                    p_fs.get()
                                }) != 0
                                    && {
                                        error = os_fsync(fd);
                                        error != 0 as ::core::ffi::c_int
                                    }
                                    && error != UV_ENOTSUP as ::core::ffi::c_int
                                    && !device
                                {
                                    err = set_err_arg(
                                        &raw const e_fsync as *const ::core::ffi::c_char,
                                        error,
                                    );
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                }
                                if !backup_copy {
                                    os_copy_xattr(backup, wfname);
                                }
                                if !backup.is_null() && !backup_copy {
                                    let mut file_info_0: FileInfo = FileInfo {
                                        stat: uv_stat_t {
                                            st_dev: 0,
                                            st_mode: 0,
                                            st_nlink: 0,
                                            st_uid: 0,
                                            st_gid: 0,
                                            st_rdev: 0,
                                            st_ino: 0,
                                            st_size: 0,
                                            st_blksize: 0,
                                            st_blocks: 0,
                                            st_flags: 0,
                                            st_gen: 0,
                                            st_atim: uv_timespec_t {
                                                tv_sec: 0,
                                                tv_nsec: 0,
                                            },
                                            st_mtim: uv_timespec_t {
                                                tv_sec: 0,
                                                tv_nsec: 0,
                                            },
                                            st_ctim: uv_timespec_t {
                                                tv_sec: 0,
                                                tv_nsec: 0,
                                            },
                                            st_birthtim: uv_timespec_t {
                                                tv_sec: 0,
                                                tv_nsec: 0,
                                            },
                                        },
                                    };
                                    if !os_fileinfo(wfname, &raw mut file_info_0)
                                        || file_info_0.stat.st_uid != file_info_old.stat.st_uid
                                        || file_info_0.stat.st_gid != file_info_old.stat.st_gid
                                    {
                                        os_fchown(
                                            fd,
                                            file_info_old.stat.st_uid as uv_uid_t,
                                            file_info_old.stat.st_gid as uv_gid_t,
                                        );
                                        if perm >= 0 as ::core::ffi::c_int {
                                            os_setperm(wfname, perm);
                                        }
                                    }
                                    buf_set_file_id(buf);
                                } else if !(*buf).file_id_valid {
                                    buf_set_file_id(buf);
                                }
                                error = os_close(fd);
                                if error != 0 as ::core::ffi::c_int {
                                    err = set_err_arg(
                                        gettext(b"E512: Close failed: %s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        error,
                                    );
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                }
                                if made_writable {
                                    perm &= !(0o200 as ::core::ffi::c_int);
                                }
                                if perm >= 0 as ::core::ffi::c_int {
                                    os_setperm(wfname, perm);
                                }
                                if !backup_copy {
                                    os_set_acl(wfname, acl);
                                }
                                if wfname != fname {
                                    if end != 0 as linenr_T {
                                        if eval_charconvert(
                                            b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
                                            fenc,
                                            wfname,
                                            fname,
                                        ) == FAIL
                                        {
                                            write_info.bw_conv_error = true_0;
                                            end = 0 as ::core::ffi::c_int as linenr_T;
                                        }
                                    }
                                    os_remove(wfname);
                                    xfree(wfname as *mut ::core::ffi::c_void);
                                }
                            }
                            if end == 0 as linenr_T {
                                if err.msg.is_null() {
                                    if write_info.bw_conv_error != 0 {
                                        if write_info.bw_conv_error_lnum == 0 as linenr_T {
                                            err = set_err(
                                                gettext(
                                                    (e_write_error_conversion_failed_make_fenc_empty_to_override.ptr() as *const _)
                                                        as *const ::core::ffi::c_char,
                                                ),
                                            );
                                        } else {
                                            err = set_err(xmalloc(300 as size_t)
                                                as *const ::core::ffi::c_char);
                                            err.alloc = true_0 != 0;
                                            vim_snprintf(
                                                err.msg,
                                                300 as size_t,
                                                gettext(
                                                    (e_write_error_conversion_failed_in_line_nr_make_fenc_empty_to_override.ptr() as *const _)
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                write_info.bw_conv_error_lnum,
                                            );
                                        }
                                    } else if got_int.get() {
                                        err = set_err(gettext(
                                            &raw const e_interr as *const ::core::ffi::c_char,
                                        ));
                                    } else {
                                        err = set_err(gettext(
                                            (e_write_error_file_system_full.ptr() as *const _)
                                                as *const ::core::ffi::c_char,
                                        ));
                                    }
                                }
                                if !backup.is_null() {
                                    if backup_copy {
                                        if got_int.get() {
                                            msg(
                                                gettext(
                                                    &raw const e_interr
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                0 as ::core::ffi::c_int,
                                            );
                                            ui_flush();
                                        }
                                        if os_copy(backup, fname, UV_FS_COPYFILE_FICLONE)
                                            == 0 as ::core::ffi::c_int
                                        {
                                            end = 1 as ::core::ffi::c_int as linenr_T;
                                        }
                                    } else if vim_rename(backup, fname) == 0 as ::core::ffi::c_int {
                                        end = 1 as ::core::ffi::c_int as linenr_T;
                                    }
                                }
                                break '_fail;
                            } else {
                                lnum -= start;
                                (*no_wait_return.ptr()) -= 1;
                                if !filtering {
                                    add_quoted_fname(
                                        IObuff.ptr() as *mut ::core::ffi::c_char,
                                        IOSIZE as size_t,
                                        buf,
                                        fname,
                                    );
                                    let mut insert_space: bool = false_0 != 0;
                                    if write_info.bw_conv_error != 0 {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b" CONVERSION ERROR\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                        if write_info.bw_conv_error_lnum != 0 as linenr_T {
                                            vim_snprintf_add(
                                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                                IOSIZE as size_t,
                                                gettext(b" in line %ld;\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                write_info.bw_conv_error_lnum as int64_t,
                                            );
                                        }
                                    } else if notconverted {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b"[NOT converted]\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    } else if converted {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b"[converted]\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    }
                                    if device {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b"[Device]\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    } else if newfile {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(
                                                b"[New]\0".as_ptr() as *const ::core::ffi::c_char
                                            ),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    }
                                    if no_eol {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(
                                                b"[noeol]\0".as_ptr() as *const ::core::ffi::c_char
                                            ),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    }
                                    if msg_add_fileformat(fileformat) {
                                        insert_space = true_0 != 0;
                                    }
                                    msg_add_lines(
                                        insert_space as ::core::ffi::c_int,
                                        lnum,
                                        nchars as off_T,
                                    );
                                    if !shortmess(SHM_WRITE as ::core::ffi::c_int) {
                                        if append {
                                            xstrlcat(
                                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                                if shortmess(SHM_WRI as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                                    != 0
                                                {
                                                    gettext(b" [a]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                } else {
                                                    gettext(b" appended\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                },
                                                IOSIZE as size_t,
                                            );
                                        } else {
                                            xstrlcat(
                                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                                if shortmess(SHM_WRI as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                                    != 0
                                                {
                                                    gettext(b" [w]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                } else {
                                                    gettext(b" written\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                },
                                                IOSIZE as size_t,
                                            );
                                        }
                                    }
                                    set_keep_msg(
                                        msg_progress(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            b"bufwrite\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            b"success\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            0 as ::core::ffi::c_int,
                                            true_0 != 0,
                                            true_0 != 0,
                                        ),
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                                if reset_changed as ::core::ffi::c_int != 0
                                    && whole as ::core::ffi::c_int != 0
                                    && !append
                                    && write_info.bw_conv_error == 0
                                    && (overwriting as ::core::ffi::c_int != 0
                                        || !vim_strchr(p_cpo.get(), CPO_PLUS).is_null())
                                {
                                    unchanged(buf, true_0 != 0, false_0 != 0);
                                    let changedtick: varnumber_T = buf_get_changedtick(buf);
                                    if (*buf).b_last_changedtick + 1 as varnumber_T == changedtick {
                                        (*buf).b_last_changedtick = changedtick;
                                    }
                                    u_unchanged(buf);
                                    u_update_save_nr(buf);
                                }
                                if overwriting {
                                    ml_timestamp(buf);
                                    if append {
                                        (*buf).b_flags &= !BF_NEW;
                                    } else {
                                        (*buf).b_flags &= !BF_WRITE_MASK;
                                    }
                                }
                                if *p_pm.get() as ::core::ffi::c_int != 0
                                    && dobackup as ::core::ffi::c_int != 0
                                {
                                    let org: *mut ::core::ffi::c_char =
                                        modname(fname, p_pm.get(), false_0 != 0);
                                    if !backup.is_null() {
                                        if org.is_null() {
                                            emsg(gettext(
                                                b"E205: Patchmode: can't save original file\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ));
                                        } else if !os_path_exists(org) {
                                            vim_rename(backup, org);
                                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                                &raw mut backup as *mut *mut ::core::ffi::c_void;
                                            xfree(*ptr_);
                                            *ptr_ = NULL;
                                            let _ = *ptr_;
                                            os_file_settime(
                                                org,
                                                file_info_old.stat.st_atim.tv_sec
                                                    as ::core::ffi::c_double,
                                                file_info_old.stat.st_mtim.tv_sec
                                                    as ::core::ffi::c_double,
                                            );
                                        }
                                    } else {
                                        let mut empty_fd: ::core::ffi::c_int = 0;
                                        if org.is_null() || {
                                            empty_fd = os_open(
                                                org,
                                                O_CREAT | O_EXCL | O_NOFOLLOW,
                                                if perm < 0 as ::core::ffi::c_int {
                                                    0o666 as ::core::ffi::c_int
                                                } else {
                                                    perm & 0o777 as ::core::ffi::c_int
                                                },
                                            );
                                            empty_fd < 0 as ::core::ffi::c_int
                                        } {
                                            emsg(gettext(
                                                (e_patchmode_cant_touch_empty_original_file.ptr()
                                                    as *const _)
                                                    as *const ::core::ffi::c_char,
                                            ));
                                        } else {
                                            close(empty_fd);
                                        }
                                    }
                                    if !org.is_null() {
                                        os_setperm(
                                            org,
                                            os_getperm(fname) as ::core::ffi::c_int
                                                & 0o777 as ::core::ffi::c_int,
                                        );
                                        xfree(org as *mut ::core::ffi::c_void);
                                    }
                                }
                                if p_bk.get() == 0
                                    && !backup.is_null()
                                    && write_info.bw_conv_error == 0
                                    && os_remove(backup) != 0 as ::core::ffi::c_int
                                {
                                    emsg(gettext(b"E207: Can't delete backup file\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                }
                                break '_nofail;
                            }
                        } else {
                            checking_conversion = false_0 != 0;
                        }
                    }
                }
                if !backup.is_null() && wfname == fname {
                    if backup_copy {
                        if !os_path_exists(fname) {
                            vim_rename(backup, fname);
                        }
                        if os_path_exists(fname) {
                            os_remove(backup);
                        }
                    } else {
                        vim_rename(backup, fname);
                    }
                }
                if !newfile && !os_path_exists(fname) {
                    end = 0 as ::core::ffi::c_int as linenr_T;
                }
                if wfname != fname {
                    xfree(wfname as *mut ::core::ffi::c_void);
                }
            }
        }
        (*no_wait_return.ptr()) -= 1;
    }
    (*buf).b_saving = false_0 != 0;
    xfree(backup as *mut ::core::ffi::c_void);
    if buffer != &raw mut smallbuf as *mut ::core::ffi::c_char {
        xfree(buffer as *mut ::core::ffi::c_void);
    }
    xfree(fenc_tofree as *mut ::core::ffi::c_void);
    xfree(write_info.bw_conv_buf as *mut ::core::ffi::c_void);
    if write_info.bw_iconv_fd
        != ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        )
    {
        iconv_close(write_info.bw_iconv_fd);
        write_info.bw_iconv_fd = ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        );
    }
    os_free_acl(acl);
    if !err.msg.is_null() {
        add_quoted_fname(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            (IOSIZE - 100 as ::core::ffi::c_int) as size_t,
            buf,
            fname,
        );
        emit_err(&raw mut err);
        retval = FAIL;
        if end == 0 as linenr_T {
            let hl_id: ::core::ffi::c_int = HLF_E as ::core::ffi::c_int;
            msg_puts_hl(
                gettext(
                    b"\nWARNING: Original file may be lost or damaged\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                hl_id,
                true_0 != 0,
            );
            msg_puts_hl(
                gettext(
                    b"don't quit the editor until the file is successfully written!\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                hl_id,
                true_0 != 0,
            );
            if os_fileinfo(fname, &raw mut file_info_old) {
                buf_store_file_info(buf, &raw mut file_info_old);
                (*buf).b_mtime_read = (*buf).b_mtime;
                (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
            }
        }
    }
    msg_scroll.set(msg_save);
    if retval == OK && write_undo_file as ::core::ffi::c_int != 0 {
        let mut hash: [uint8_t; 32] = sha_ctx.finish();
        u_write_undo(
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0 != 0,
            buf,
            &raw mut hash as *mut uint8_t,
        );
    }
    if !should_abort(retval) {
        buf_write_do_post_autocmds(buf, fname, eap, append, filtering, reset_changed, whole);
        if aborting() {
            retval = false_0;
        }
    }
    got_int.set(got_int.get() as ::core::ffi::c_int | prev_got_int as ::core::ffi::c_int != 0);
    return retval;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_FNAMEW: ::core::ffi::c_int = 'F' as ::core::ffi::c_int;
pub const CPO_FNAMEAPP: ::core::ffi::c_int = 'P' as ::core::ffi::c_int;
pub const CPO_FWRITE: ::core::ffi::c_int = 'W' as ::core::ffi::c_int;
pub const CPO_KEEPRO: ::core::ffi::c_int = 'Z' as ::core::ffi::c_int;
pub const CPO_PLUS: ::core::ffi::c_int = '+' as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;
