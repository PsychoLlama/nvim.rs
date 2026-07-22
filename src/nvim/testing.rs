use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::typval::{
    tv_check_for_float_or_nr_arg, tv_check_for_opt_number_arg, tv_check_for_opt_string_arg,
    tv_check_for_opt_string_or_list_arg, tv_check_for_string_or_number_arg, tv_clear,
    tv_dict_add_tv, tv_dict_alloc, tv_dict_find, tv_equal, tv_get_float, tv_get_number_chk,
    tv_get_string, tv_get_string_buf_chk, tv_get_string_chk,
};
use crate::src::nvim::eval::vars::{
    assert_error, get_vim_var_nr, get_vim_var_str, get_vim_var_tv, set_vim_var_string,
};
use crate::src::nvim::eval_1::{garbage_collect, pattern_match};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::main::{
    called_emsg, called_vim_beep, did_emsg, e_cant_read_file_str, emsg_assert_fails_context,
    emsg_assert_fails_lnum, emsg_assert_fails_msg, emsg_on_display, emsg_silent, got_int,
    in_assert_fails, lines_left, msg_col, need_wait_return, no_wait_return, suppress_errthrow,
    trylevel, IObuff, Rows,
};
use crate::src::nvim::mbyte::{mb_cptr2char_adv, utf_ptr2char};
use crate::src::nvim::memory::{xfree, xstrdup, xstrlcpy};
use crate::src::nvim::message::{emsg, msg_reset_scroll};
use crate::src::nvim::os::fs::os_fopen;
use crate::src::nvim::os::libc::{fclose, fgetc, gettext, memmove, strcmp, strlen, strstr};
use crate::src::nvim::runtime::{estack_sfile, exestack};
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_safelen};
pub use crate::src::nvim::types::{
    ApiDispatchWrapper, Arena, Array, AutoPat, AutoPatCmd, AutoPatCmd_S, BoolVarValue, Boolean,
    Dict, Error, ErrorType, EvalFuncData, Float, Integer, KeyValuePair, LuaRef,
    MsgpackRpcRequestHandler, Object, ObjectType, ScopeDictDictItem, ScopeType, SpecialVarValue,
    String_0, VarLockStatus, VarType, VimVarIndex, _IO_codecvt, _IO_lock_t, _IO_marker,
    _IO_wide_data, __off64_t, __off_t, auto_event, blob_T, blobvar_S, dict_T, dictitem_T,
    dictvar_S, estack_T, estack_T_es_info as C2Rust_Unnamed_2, estack_arg_T, etype_T, event_T,
    except_T, except_type_T, float_T, funccall_S, funccall_S_fc_fixvar as C2Rust_Unnamed_0,
    funccall_T, garray_T, hash_T, hashitem_T, hashtab_T, int32_t, int64_t, key_value_pair,
    linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S, listwatch_T, msglist,
    msglist_T, object, object_data as C2Rust_Unnamed, partial_S, partial_T, proftime_T, ptrdiff_t,
    queue, regprog, regprog_T, scid_T, sctx_T, size_t, typval_T, typval_vval_union, ufunc_S,
    ufunc_T, uint64_t, uint8_t, varnumber_T, vim_exception, FILE, QUEUE, _IO_FILE,
};
extern "C" {
    fn ga_clear(gap: *mut garray_T);
    fn ga_init(gap: *mut garray_T, itemsize: ::core::ffi::c_int, growsize: ::core::ffi::c_int);
    fn ga_concat(gap: *mut garray_T, s: *const ::core::ffi::c_char);
    fn ga_concat_len(gap: *mut garray_T, s: *const ::core::ffi::c_char, len: size_t);
    fn ga_append(gap: *mut garray_T, c: uint8_t);
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
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
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_1 = 65;
pub const ET_INTERRUPT: except_type_T = 2;
pub const ET_ERROR: except_type_T = 1;
pub const ET_USER: except_type_T = 0;
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
pub const ETYPE_SPELL: etype_T = 9;
pub const ETYPE_INTERNAL: etype_T = 8;
pub const ETYPE_ENV: etype_T = 7;
pub const ETYPE_ARGS: etype_T = 6;
pub const ETYPE_EXCEPT: etype_T = 5;
pub const ETYPE_MODELINE: etype_T = 4;
pub const ETYPE_AUCMD: etype_T = 3;
pub const ETYPE_UFUNC: etype_T = 2;
pub const ETYPE_SCRIPT: etype_T = 1;
pub const ETYPE_TOP: etype_T = 0;
pub const ESTACK_SCRIPT: estack_arg_T = 3;
pub const ESTACK_STACK: estack_arg_T = 2;
pub const ESTACK_SFILE: estack_arg_T = 1;
pub const ESTACK_NONE: estack_arg_T = 0;
pub type assert_type_T = ::core::ffi::c_uint;
pub const ASSERT_OTHER: assert_type_T = 5;
pub const ASSERT_FAILS: assert_type_T = 4;
pub const ASSERT_NOTMATCH: assert_type_T = 3;
pub const ASSERT_MATCH: assert_type_T = 2;
pub const ASSERT_NOTEQUAL: assert_type_T = 1;
pub const ASSERT_EQUAL: assert_type_T = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const BS: ::core::ffi::c_int = 8;
pub const TAB: ::core::ffi::c_int = 9;
pub const NL: ::core::ffi::c_int = 10;
pub const FF: ::core::ffi::c_int = 12;
pub const CAR: ::core::ffi::c_int = 13;
pub const ESC: ::core::ffi::c_int = 27;
#[inline]
unsafe extern "C" fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    return (*l).lv_len;
}
#[inline]
unsafe extern "C" fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_first;
}
#[inline]
unsafe extern "C" fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut::<listitem_T>();
    }
    return (*l).lv_last;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static e_assert_fails_second_arg: GlobalCell<[::core::ffi::c_char; 90]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<
        [u8; 90],
        [::core::ffi::c_char; 90],
    >(
        *b"E856: \"assert_fails()\" second argument must be a string or a list with one or two strings\0",
    )
});
static e_assert_fails_fourth_argument: GlobalCell<[::core::ffi::c_char; 57]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 57], [::core::ffi::c_char; 57]>(
            *b"E1115: \"assert_fails()\" fourth argument must be a number\0",
        )
    });
static e_assert_fails_fifth_argument: GlobalCell<[::core::ffi::c_char; 56]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 56], [::core::ffi::c_char; 56]>(
            *b"E1116: \"assert_fails()\" fifth argument must be a string\0",
        )
    });
static e_calling_test_garbagecollect_now_while_v_testing_is_not_set: GlobalCell<
    [::core::ffi::c_char; 68],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 68], [::core::ffi::c_char; 68]>(
        *b"E1142: Calling test_garbagecollect_now() while v:testing is not set\0",
    )
});
unsafe extern "C" fn prepare_assert_error(mut gap: *mut garray_T) {
    let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
    ga_init(gap, 1 as ::core::ffi::c_int, 100 as ::core::ffi::c_int);
    if !sname.is_null() {
        ga_concat(gap, sname);
        if (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            > 0 as linenr_T
        {
            ga_concat(gap, b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
    if (*((*exestack.ptr()).ga_data as *mut estack_T)
        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
    .es_lnum
        > 0 as linenr_T
    {
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        let mut buflen: size_t = vim_snprintf_safelen(
            &raw mut buf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as size_t,
                ),
            b"line %ld\0".as_ptr() as *const ::core::ffi::c_char,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum as int64_t,
        );
        ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
    }
    if !sname.is_null()
        || (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum
            > 0 as linenr_T
    {
        ga_concat_len(
            gap,
            b": \0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    }
    xfree(sname as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn ga_concat_esc(
    mut gap: *mut garray_T,
    mut p: *const ::core::ffi::c_char,
    mut clen: ::core::ffi::c_int,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if clen > 1 as ::core::ffi::c_int {
        memmove(
            &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            clen as size_t,
        );
        buf[clen as usize] = NUL as ::core::ffi::c_char;
        ga_concat_len(
            gap,
            &raw mut buf as *mut ::core::ffi::c_char,
            clen as size_t,
        );
        return;
    }
    match *p as ::core::ffi::c_int {
        BS => {
            ga_concat_len(
                gap,
                b"\\b\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        ESC => {
            ga_concat_len(
                gap,
                b"\\e\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        FF => {
            ga_concat_len(
                gap,
                b"\\f\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        NL => {
            ga_concat_len(
                gap,
                b"\\n\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        TAB => {
            ga_concat_len(
                gap,
                b"\\t\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        CAR => {
            ga_concat_len(
                gap,
                b"\\r\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        92 => {
            ga_concat_len(
                gap,
                b"\\\\\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        _ => {
            if (*p as uint8_t as ::core::ffi::c_int) < ' ' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == 0x7f as ::core::ffi::c_int
            {
                let mut buflen: size_t = vim_snprintf_safelen(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    NUMBUFLEN as ::core::ffi::c_int as size_t,
                    b"\\x%02x\0".as_ptr() as *const ::core::ffi::c_char,
                    *p as ::core::ffi::c_int,
                );
                ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
            } else {
                ga_append(gap, *p as uint8_t);
            }
        }
    };
}
unsafe extern "C" fn ga_concat_shorten_esc(
    mut gap: *mut garray_T,
    mut str: *const ::core::ffi::c_char,
) {
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    if str.is_null() {
        ga_concat_len(
            gap,
            b"NULL\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        );
        return;
    }
    let mut p: *const ::core::ffi::c_char = str;
    while *p as ::core::ffi::c_int != NUL {
        let mut same_len: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut s: *const ::core::ffi::c_char = p;
        let c: ::core::ffi::c_int = mb_cptr2char_adv(&raw mut s);
        let clen: ::core::ffi::c_int = s.offset_from(p) as ::core::ffi::c_int;
        while *s as ::core::ffi::c_int != NUL && c == utf_ptr2char(s) {
            same_len += 1;
            s = s.offset(clen as isize);
        }
        if same_len > 20 as ::core::ffi::c_int {
            ga_concat_len(
                gap,
                b"\\[\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
            ga_concat_esc(gap, p, clen);
            ga_concat_len(
                gap,
                b" occurs \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            );
            let mut buflen: size_t = vim_snprintf_safelen(
                &raw mut buf as *mut ::core::ffi::c_char,
                NUMBUFLEN as ::core::ffi::c_int as size_t,
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                same_len,
            );
            ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
            ga_concat_len(
                gap,
                b" times]\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            );
            p = s;
        } else {
            ga_concat_esc(gap, p, clen);
            p = p.offset(clen as isize);
        }
    }
}
unsafe extern "C" fn fill_assert_error(
    mut gap: *mut garray_T,
    mut opt_msg_tv: *mut typval_T,
    mut exp_str: *const ::core::ffi::c_char,
    mut exp_tv_arg: *mut typval_T,
    mut got_tv_arg: *mut typval_T,
    mut atype: assert_type_T,
) {
    let mut exp_tv: *mut typval_T = exp_tv_arg;
    let mut got_tv: *mut typval_T = got_tv_arg;
    let mut did_copy: bool = false_0 != 0;
    let mut omitted: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*opt_msg_tv).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && !((*opt_msg_tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*opt_msg_tv).vval.v_string.is_null()
                || *(*opt_msg_tv).vval.v_string as ::core::ffi::c_int == NUL))
    {
        let mut tofree: *mut ::core::ffi::c_char =
            encode_tv2echo(opt_msg_tv, ::core::ptr::null_mut::<size_t>());
        ga_concat(gap, tofree);
        xfree(tofree as *mut ::core::ffi::c_void);
        ga_concat_len(
            gap,
            b": \0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
        );
    }
    if atype as ::core::ffi::c_uint == ASSERT_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        || atype as ::core::ffi::c_uint
            == ASSERT_NOTMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ga_concat_len(
            gap,
            b"Pattern \0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        );
    } else if atype as ::core::ffi::c_uint
        == ASSERT_NOTEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        ga_concat_len(
            gap,
            b"Expected not equal to \0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        );
    } else {
        ga_concat_len(
            gap,
            b"Expected \0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        );
    }
    if exp_str.is_null() {
        if atype as ::core::ffi::c_uint
            != ASSERT_NOTEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*exp_tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*got_tv).v_type as ::core::ffi::c_uint
                == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*exp_tv).vval.v_dict.is_null()
            && !(*got_tv).vval.v_dict.is_null()
        {
            let mut exp_d: *mut dict_T = (*exp_tv).vval.v_dict;
            let mut got_d: *mut dict_T = (*got_tv).vval.v_dict;
            did_copy = true_0 != 0;
            (*exp_tv).vval.v_dict = tv_dict_alloc();
            (*got_tv).vval.v_dict = tv_dict_alloc();
            let mut todo: ::core::ffi::c_int = (*exp_d).dv_hashtab.ht_used as ::core::ffi::c_int;
            let mut hi: *const hashitem_T = (*exp_d).dv_hashtab.ht_array;
            while todo > 0 as ::core::ffi::c_int {
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    let mut item2: *mut dictitem_T =
                        tv_dict_find(got_d, (*hi).hi_key, -1 as ptrdiff_t);
                    if item2.is_null()
                        || !tv_equal(
                            &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                            &raw mut (*item2).di_tv,
                            false_0 != 0,
                        )
                    {
                        let key_len: size_t = strlen((*hi).hi_key);
                        tv_dict_add_tv(
                            (*exp_tv).vval.v_dict,
                            (*hi).hi_key,
                            key_len,
                            &raw mut (*((*hi).hi_key.offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                        );
                        if !item2.is_null() {
                            tv_dict_add_tv(
                                (*got_tv).vval.v_dict,
                                (*hi).hi_key,
                                key_len,
                                &raw mut (*item2).di_tv,
                            );
                        }
                    } else {
                        omitted += 1;
                    }
                    todo -= 1;
                }
                hi = hi.offset(1);
            }
            todo = (*got_d).dv_hashtab.ht_used as ::core::ffi::c_int;
            let mut hi_0: *const hashitem_T = (*got_d).dv_hashtab.ht_array;
            while todo > 0 as ::core::ffi::c_int {
                if !((*hi_0).hi_key.is_null()
                    || (*hi_0).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    let mut item2_0: *mut dictitem_T =
                        tv_dict_find(exp_d, (*hi_0).hi_key, -1 as ptrdiff_t);
                    if item2_0.is_null() {
                        let key_len_0: size_t = strlen((*hi_0).hi_key);
                        tv_dict_add_tv(
                            (*got_tv).vval.v_dict,
                            (*hi_0).hi_key,
                            key_len_0,
                            &raw mut (*((*hi_0)
                                .hi_key
                                .offset(-(17 as ::core::ffi::c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                        );
                    }
                    todo -= 1;
                }
                hi_0 = hi_0.offset(1);
            }
        }
        let mut tofree_0: *mut ::core::ffi::c_char =
            encode_tv2string(exp_tv, ::core::ptr::null_mut::<size_t>());
        ga_concat_shorten_esc(gap, tofree_0);
        xfree(tofree_0 as *mut ::core::ffi::c_void);
    } else {
        if atype as ::core::ffi::c_uint == ASSERT_FAILS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                b"'\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
        }
        ga_concat_shorten_esc(gap, exp_str);
        if atype as ::core::ffi::c_uint == ASSERT_FAILS as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                b"'\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
        }
    }
    if atype as ::core::ffi::c_uint != ASSERT_NOTEQUAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if atype as ::core::ffi::c_uint == ASSERT_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                b" does not match \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
            );
        } else if atype as ::core::ffi::c_uint
            == ASSERT_NOTMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ga_concat_len(
                gap,
                b" does match \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            );
        } else {
            ga_concat_len(
                gap,
                b" but got \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            );
        }
        let mut tofree_1: *mut ::core::ffi::c_char =
            encode_tv2string(got_tv, ::core::ptr::null_mut::<size_t>());
        ga_concat_shorten_esc(gap, tofree_1);
        xfree(tofree_1 as *mut ::core::ffi::c_void);
        if omitted != 0 as ::core::ffi::c_int {
            let mut buf: [::core::ffi::c_char; 100] = [0; 100];
            let mut buflen: size_t = vim_snprintf_safelen(
                &raw mut buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                b" - %d equal item%s omitted\0".as_ptr() as *const ::core::ffi::c_char,
                omitted,
                if omitted == 1 as ::core::ffi::c_int {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"s\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            ga_concat_len(gap, &raw mut buf as *mut ::core::ffi::c_char, buflen);
        }
    }
    if did_copy {
        tv_clear(exp_tv);
        tv_clear(got_tv);
    }
}
unsafe extern "C" fn assert_equal_common(
    mut argvars: *mut typval_T,
    mut atype: assert_type_T,
) -> ::core::ffi::c_int {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if tv_equal(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        argvars.offset(1 as ::core::ffi::c_int as isize),
        false_0 != 0,
    ) as ::core::ffi::c_int
        != (atype as ::core::ffi::c_uint
            == ASSERT_EQUAL as ::core::ffi::c_int as ::core::ffi::c_uint)
            as ::core::ffi::c_int
    {
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null::<::core::ffi::c_char>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            argvars.offset(1 as ::core::ffi::c_int as isize),
            atype,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn assert_match_common(
    mut argvars: *mut typval_T,
    mut atype: assert_type_T,
) -> ::core::ffi::c_int {
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let pat: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    let text: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    );
    if !pat.is_null()
        && !text.is_null()
        && pattern_match(pat, text, false_0 != 0)
            != (atype as ::core::ffi::c_uint
                == ASSERT_MATCH as ::core::ffi::c_int as ::core::ffi::c_uint)
                as ::core::ffi::c_int
    {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null::<::core::ffi::c_char>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            argvars.offset(1 as ::core::ffi::c_int as isize),
            atype,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn assert_bool(
    mut argvars: *mut typval_T,
    mut is_true: bool,
) -> ::core::ffi::c_int {
    let mut error: bool = false_0 != 0;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) == 0 as varnumber_T) as ::core::ffi::c_int
            == is_true as ::core::ffi::c_int
        || error as ::core::ffi::c_int != 0)
        && ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BOOL as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_bool as ::core::ffi::c_uint
                != (if is_true as ::core::ffi::c_int != 0 {
                    kBoolVarTrue as ::core::ffi::c_int
                } else {
                    kBoolVarFalse as ::core::ffi::c_int
                }) as BoolVarValue as ::core::ffi::c_uint)
    {
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            if is_true as ::core::ffi::c_int != 0 {
                b"True\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"False\0".as_ptr() as *const ::core::ffi::c_char
            },
            ::core::ptr::null_mut::<typval_T>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            ASSERT_OTHER,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn assert_append_cmd_or_arg(
    mut gap: *mut garray_T,
    mut argvars: *mut typval_T,
    mut cmd: *const ::core::ffi::c_char,
) {
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let tofree: *mut ::core::ffi::c_char = encode_tv2echo(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<size_t>(),
        );
        ga_concat(gap, tofree);
        xfree(tofree as *mut ::core::ffi::c_void);
    } else {
        ga_concat(gap, cmd);
    };
}
unsafe extern "C" fn assert_beeps(
    mut argvars: *mut typval_T,
    mut no_beep: bool,
) -> ::core::ffi::c_int {
    let cmd: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    called_vim_beep.set(false_0 != 0);
    suppress_errthrow.set(true_0 != 0);
    emsg_silent.set(false_0);
    do_cmdline_cmd(cmd);
    if if no_beep as ::core::ffi::c_int != 0 {
        called_vim_beep.get() as ::core::ffi::c_int
    } else {
        !called_vim_beep.get() as ::core::ffi::c_int
    } != 0
    {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        prepare_assert_error(&raw mut ga);
        if no_beep {
            ga_concat_len(
                &raw mut ga,
                b"command did beep: \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
            );
        } else {
            ga_concat_len(
                &raw mut ga,
                b"command did not beep: \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
            );
        }
        ga_concat(&raw mut ga, cmd);
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        ret = 1 as ::core::ffi::c_int;
    }
    suppress_errthrow.set(false_0 != 0);
    emsg_on_display.set(false_0 != 0);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_beeps(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_beeps(argvars, false_0 != 0) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_nobeep(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_beeps(argvars, true_0 != 0) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_equal(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_equal_common(argvars, ASSERT_EQUAL) as varnumber_T;
}
unsafe extern "C" fn assert_equalfile(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
    let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
    let fname1: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut buf1 as *mut ::core::ffi::c_char,
    );
    let fname2: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut buf2 as *mut ::core::ffi::c_char,
    );
    if fname1.is_null() || fname2.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    (*IObuff.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    let mut IObufflen: size_t = 0 as size_t;
    let fd1: *mut FILE = os_fopen(fname1, READBIN.as_ptr());
    let mut line1: [::core::ffi::c_char; 200] = [0; 200];
    let mut line2: [::core::ffi::c_char; 200] = [0; 200];
    let mut lineidx: ptrdiff_t = 0 as ptrdiff_t;
    if fd1.is_null() {
        IObufflen = vim_snprintf_safelen(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            &raw const e_cant_read_file_str as *const ::core::ffi::c_char,
            fname1,
        );
    } else {
        let fd2: *mut FILE = os_fopen(fname2, READBIN.as_ptr());
        if fd2.is_null() {
            fclose(fd1);
            IObufflen = vim_snprintf_safelen(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                &raw const e_cant_read_file_str as *const ::core::ffi::c_char,
                fname2,
            );
        } else {
            let mut linecount: int64_t = 1 as int64_t;
            let mut count: int64_t = 0 as int64_t;
            loop {
                let c1: ::core::ffi::c_int = fgetc(fd1);
                let c2: ::core::ffi::c_int = fgetc(fd2);
                if c1 == EOF {
                    if c2 != EOF {
                        IObufflen = xstrlcpy(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            b"first file is shorter\0".as_ptr() as *const ::core::ffi::c_char,
                            IOSIZE as size_t,
                        );
                    }
                    break;
                } else if c2 == EOF {
                    IObufflen = xstrlcpy(
                        IObuff.ptr() as *mut ::core::ffi::c_char,
                        b"second file is shorter\0".as_ptr() as *const ::core::ffi::c_char,
                        IOSIZE as size_t,
                    );
                    break;
                } else {
                    line1[lineidx as usize] = c1 as ::core::ffi::c_char;
                    line2[lineidx as usize] = c2 as ::core::ffi::c_char;
                    lineidx += 1;
                    if c1 != c2 {
                        IObufflen = vim_snprintf_safelen(
                            IObuff.ptr() as *mut ::core::ffi::c_char,
                            IOSIZE as size_t,
                            b"difference at byte %ld, line %ld\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            count,
                            linecount,
                        );
                        break;
                    } else {
                        if c1 == NL {
                            linecount += 1;
                            lineidx = 0 as ptrdiff_t;
                        } else if lineidx + 2 as ptrdiff_t
                            == ::core::mem::size_of::<[::core::ffi::c_char; 200]>() as ptrdiff_t
                        {
                            memmove(
                                &raw mut line1 as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (&raw mut line1 as *mut ::core::ffi::c_char)
                                    .offset(100 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                (lineidx - 100 as ptrdiff_t) as size_t,
                            );
                            memmove(
                                &raw mut line2 as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (&raw mut line2 as *mut ::core::ffi::c_char)
                                    .offset(100 as ::core::ffi::c_int as isize)
                                    as *const ::core::ffi::c_void,
                                (lineidx - 100 as ptrdiff_t) as size_t,
                            );
                            lineidx -= 100 as ptrdiff_t;
                        }
                        count += 1;
                    }
                }
            }
            fclose(fd1);
            fclose(fd2);
        }
    }
    if IObufflen > 0 as size_t {
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        prepare_assert_error(&raw mut ga);
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let tofree: *mut ::core::ffi::c_char = encode_tv2echo(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<size_t>(),
            );
            ga_concat(&raw mut ga, tofree);
            xfree(tofree as *mut ::core::ffi::c_void);
            ga_concat_len(
                &raw mut ga,
                b": \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
            );
        }
        ga_concat_len(
            &raw mut ga,
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IObufflen,
        );
        if lineidx > 0 as ptrdiff_t {
            line1[lineidx as usize] = NUL as ::core::ffi::c_char;
            line2[lineidx as usize] = NUL as ::core::ffi::c_char;
            ga_concat_len(
                &raw mut ga,
                b" after \"\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            );
            ga_concat_len(
                &raw mut ga,
                &raw mut line1 as *mut ::core::ffi::c_char,
                lineidx as size_t,
            );
            if strcmp(
                &raw mut line1 as *mut ::core::ffi::c_char,
                &raw mut line2 as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
            {
                ga_concat_len(
                    &raw mut ga,
                    b"\" vs \"\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                );
                ga_concat_len(
                    &raw mut ga,
                    &raw mut line2 as *mut ::core::ffi::c_char,
                    lineidx as size_t,
                );
            }
            ga_concat_len(
                &raw mut ga,
                b"\"\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            );
        }
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_equalfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_equalfile(argvars) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_notequal(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_equal_common(argvars, ASSERT_NOTEQUAL) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_exception(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let error: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *get_vim_var_str(VV_EXCEPTION) as ::core::ffi::c_int == NUL {
        prepare_assert_error(&raw mut ga);
        ga_concat_len(
            &raw mut ga,
            b"v:exception is not set\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        (*rettv).vval.v_number = 1 as varnumber_T;
    } else if !error.is_null() && strstr(get_vim_var_str(VV_EXCEPTION), error).is_null() {
        prepare_assert_error(&raw mut ga);
        fill_assert_error(
            &raw mut ga,
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null::<::core::ffi::c_char>(),
            argvars.offset(0 as ::core::ffi::c_int as isize),
            get_vim_var_tv(VV_EXCEPTION),
            ASSERT_OTHER,
        );
        assert_error(&raw mut ga);
        ga_clear(&raw mut ga);
        (*rettv).vval.v_number = 1 as varnumber_T;
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_fails(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let save_trylevel: ::core::ffi::c_int = trylevel.get();
    let called_emsg_before: ::core::ffi::c_int = called_emsg.get();
    let mut wrong_arg_msg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if tv_check_for_string_or_number_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_string_or_list_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && ((*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (tv_check_for_opt_number_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
                    || (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type
                        as ::core::ffi::c_uint
                        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                        && tv_check_for_opt_string_arg(argvars, 4 as ::core::ffi::c_int) == FAIL))
    {
        return;
    }
    trylevel.set(0 as ::core::ffi::c_int);
    suppress_errthrow.set(true_0 != 0);
    in_assert_fails.set(true_0 != 0);
    (*no_wait_return.ptr()) += 1;
    let cmd: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    do_cmdline_cmd(cmd);
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false_0 != 0);
    '_theend: {
        if called_emsg.get() == called_emsg_before {
            prepare_assert_error(&raw mut ga);
            ga_concat_len(
                &raw mut ga,
                b"command did not fail: \0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
            );
            assert_append_cmd_or_arg(&raw mut ga, argvars, cmd);
            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            (*rettv).vval.v_number = 1 as varnumber_T;
        } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut buf: [::core::ffi::c_char; 65] = [0; 65];
            let mut expected: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            let mut expected_str: *const ::core::ffi::c_char =
                ::core::ptr::null::<::core::ffi::c_char>();
            let mut error_found: bool = false_0 != 0;
            let mut error_found_index: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            let mut actual: *mut ::core::ffi::c_char = (if (*emsg_assert_fails_msg.ptr()).is_null()
            {
                b"[unknown]\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                emsg_assert_fails_msg.get() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char;
            if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                expected = tv_get_string_buf_chk(
                    argvars.offset(1 as ::core::ffi::c_int as isize),
                    &raw mut buf as *mut ::core::ffi::c_char,
                );
                error_found = expected.is_null() || strstr(actual, expected).is_null();
            } else if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type
                as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let list: *const list_T = (*argvars.offset(1 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list;
                if list.is_null()
                    || tv_list_len(list) < 1 as ::core::ffi::c_int
                    || tv_list_len(list) > 2 as ::core::ffi::c_int
                {
                    wrong_arg_msg =
                        (e_assert_fails_second_arg.ptr() as *const _) as *const ::core::ffi::c_char;
                    break '_theend;
                } else {
                    let mut tv: *const typval_T = &raw mut (*(tv_list_first
                        as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                        list
                    ))
                    .li_tv;
                    expected = tv_get_string_buf_chk(tv, &raw mut buf as *mut ::core::ffi::c_char);
                    if expected.is_null() {
                        break '_theend;
                    } else if pattern_match(expected, actual, false_0 != 0) == 0 {
                        error_found = true_0 != 0;
                        expected_str = expected;
                    } else if tv_list_len(list) == 2 as ::core::ffi::c_int {
                        actual = xstrdup(get_vim_var_str(VV_ERRMSG));
                        tofree = actual;
                        tv = &raw mut (*(tv_list_last
                            as unsafe extern "C" fn(*const list_T) -> *mut listitem_T)(
                            list
                        ))
                        .li_tv;
                        expected =
                            tv_get_string_buf_chk(tv, &raw mut buf as *mut ::core::ffi::c_char);
                        if expected.is_null() {
                            break '_theend;
                        } else if pattern_match(expected, actual, false_0 != 0) == 0 {
                            error_found = true_0 != 0;
                            expected_str = expected;
                        }
                    }
                }
            } else {
                wrong_arg_msg =
                    (e_assert_fails_second_arg.ptr() as *const _) as *const ::core::ffi::c_char;
                break '_theend;
            }
            if !error_found
                && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    wrong_arg_msg = (e_assert_fails_fourth_argument.ptr() as *const _)
                        as *const ::core::ffi::c_char;
                    break '_theend;
                } else {
                    if (*argvars.offset(3 as ::core::ffi::c_int as isize))
                        .vval
                        .v_number
                        >= 0 as varnumber_T
                        && (*argvars.offset(3 as ::core::ffi::c_int as isize))
                            .vval
                            .v_number
                            != emsg_assert_fails_lnum.get() as varnumber_T
                    {
                        error_found = true_0 != 0;
                        error_found_index = 3 as ::core::ffi::c_int;
                    }
                    if !error_found
                        && (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type
                            as ::core::ffi::c_uint
                            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            wrong_arg_msg = (e_assert_fails_fifth_argument.ptr() as *const _)
                                as *const ::core::ffi::c_char;
                            break '_theend;
                        } else if !(*argvars.offset(4 as ::core::ffi::c_int as isize))
                            .vval
                            .v_string
                            .is_null()
                            && pattern_match(
                                (*argvars.offset(4 as ::core::ffi::c_int as isize))
                                    .vval
                                    .v_string,
                                emsg_assert_fails_context.get(),
                                false_0 != 0,
                            ) == 0
                        {
                            error_found = true_0 != 0;
                            error_found_index = 4 as ::core::ffi::c_int;
                        }
                    }
                }
            }
            if error_found {
                let mut actual_tv: typval_T = typval_T {
                    v_type: VAR_UNKNOWN,
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union { v_number: 0 },
                };
                prepare_assert_error(&raw mut ga);
                if error_found_index == 3 as ::core::ffi::c_int {
                    actual_tv.v_type = VAR_NUMBER;
                    actual_tv.vval.v_number = emsg_assert_fails_lnum.get() as varnumber_T;
                } else if error_found_index == 4 as ::core::ffi::c_int {
                    actual_tv.v_type = VAR_STRING;
                    actual_tv.vval.v_string = emsg_assert_fails_context.get();
                } else {
                    actual_tv.v_type = VAR_STRING;
                    actual_tv.vval.v_string = actual;
                }
                fill_assert_error(
                    &raw mut ga,
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    expected_str,
                    argvars.offset(error_found_index as isize),
                    &raw mut actual_tv,
                    ASSERT_FAILS,
                );
                ga_concat_len(
                    &raw mut ga,
                    b": \0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 3]>().wrapping_sub(1 as size_t),
                );
                assert_append_cmd_or_arg(&raw mut ga, argvars, cmd);
                assert_error(&raw mut ga);
                ga_clear(&raw mut ga);
                (*rettv).vval.v_number = 1 as varnumber_T;
            }
        }
    }
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false_0 != 0);
    in_assert_fails.set(false_0 != 0);
    did_emsg.set(false_0);
    got_int.set(false_0 != 0);
    msg_col.set(0 as ::core::ffi::c_int);
    (*no_wait_return.ptr()) -= 1;
    need_wait_return.set(false_0 != 0);
    emsg_on_display.set(false_0 != 0);
    msg_reset_scroll();
    lines_left.set(Rows.get());
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        emsg_assert_fails_msg.ptr() as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    xfree(tofree as *mut ::core::ffi::c_void);
    set_vim_var_string(
        VV_ERRMSG,
        ::core::ptr::null::<::core::ffi::c_char>(),
        0 as ptrdiff_t,
    );
    if !wrong_arg_msg.is_null() {
        emsg(gettext(wrong_arg_msg));
    }
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_false(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_bool(argvars, false_0 != 0) as varnumber_T;
}
unsafe extern "C" fn assert_inrange(mut argvars: *mut typval_T) -> ::core::ffi::c_int {
    let mut error: bool = false_0 != 0;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let flower: float_T = tv_get_float(argvars.offset(0 as ::core::ffi::c_int as isize));
        let fupper: float_T = tv_get_float(argvars.offset(1 as ::core::ffi::c_int as isize));
        let factual: float_T = tv_get_float(argvars.offset(2 as ::core::ffi::c_int as isize));
        if factual < flower || factual > fupper {
            let mut ga: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            prepare_assert_error(&raw mut ga);
            let mut expected_str: [::core::ffi::c_char; 200] = [0; 200];
            vim_snprintf(
                &raw mut expected_str as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 200]>(),
                b"range %g - %g,\0".as_ptr() as *const ::core::ffi::c_char,
                flower,
                fupper,
            );
            fill_assert_error(
                &raw mut ga,
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut expected_str as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<typval_T>(),
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ASSERT_OTHER,
            );
            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1 as ::core::ffi::c_int;
        }
    } else {
        let lower: varnumber_T = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        let upper: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        let actual: varnumber_T = tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            return 0 as ::core::ffi::c_int;
        }
        if actual < lower || actual > upper {
            let mut ga_0: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            prepare_assert_error(&raw mut ga_0);
            let mut expected_str_0: [::core::ffi::c_char; 200] = [0; 200];
            vim_snprintf(
                &raw mut expected_str_0 as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 200]>(),
                b"range %ld - %ld,\0".as_ptr() as *const ::core::ffi::c_char,
                lower,
                upper,
            );
            fill_assert_error(
                &raw mut ga_0,
                argvars.offset(3 as ::core::ffi::c_int as isize),
                &raw mut expected_str_0 as *mut ::core::ffi::c_char,
                ::core::ptr::null_mut::<typval_T>(),
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ASSERT_OTHER,
            );
            assert_error(&raw mut ga_0);
            ga_clear(&raw mut ga_0);
            return 1 as ::core::ffi::c_int;
        }
    }
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_inrange(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_float_or_nr_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_float_or_nr_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        || tv_check_for_float_or_nr_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_string_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    (*rettv).vval.v_number = assert_inrange(argvars) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_match(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_match_common(argvars, ASSERT_MATCH) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_notmatch(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_match_common(argvars, ASSERT_NOTMATCH) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_report(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    prepare_assert_error(&raw mut ga);
    ga_concat(
        &raw mut ga,
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
    );
    assert_error(&raw mut ga);
    ga_clear(&raw mut ga);
    (*rettv).vval.v_number = 1 as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_assert_true(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = assert_bool(argvars, true_0 != 0) as varnumber_T;
}
#[no_mangle]
pub unsafe extern "C" fn f_test_garbagecollect_now(
    mut _argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if get_vim_var_nr(VV_TESTING) == 0 {
        emsg(gettext(
            (e_calling_test_garbagecollect_now_while_v_testing_is_not_set.ptr() as *const _)
                as *const ::core::ffi::c_char,
        ));
    } else {
        garbage_collect(true_0 != 0);
    };
}
#[no_mangle]
pub unsafe extern "C" fn f_test_write_list_log(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let fname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if fname.is_null() {
        return;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
