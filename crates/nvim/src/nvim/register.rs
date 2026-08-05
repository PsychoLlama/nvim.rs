use crate::src::nvim::api::private::helpers::{cbuf_to_string, copy_string, cstr_to_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{
    EVENT_RECORDINGENTER, EVENT_RECORDINGLEAVE, EVENT_TEXTYANKPOST, apply_autocmds, has_event,
};
use crate::src::nvim::buffer::{
    buf_is_empty, buflist_findnr, buflist_findpat, buflist_name_nr, getaltfname,
};
use crate::src::nvim::buffer_updates::buf_updates_send_changes;
use crate::src::nvim::change::{changed_bytes, changed_lines, del_chars};
use crate::src::nvim::charset::{getdigits_int, ptr2cells, skipwhite, transchar};
use crate::src::nvim::clipboard;
use crate::src::nvim::cursor::{
    coladvance_force, gchar_cursor, get_cursor_line_len, get_cursor_line_ptr, get_cursor_pos_len,
    get_cursor_pos_ptr, getviscol, getvpos,
};
use crate::src::nvim::drawscreen::{showmode, update_screen};
use crate::src::nvim::edit::{
    beginline, get_last_insert, get_last_insert_save, oneright, stuff_inserted,
};
use crate::src::nvim::eval::typval::tv_list_set_lock;
use crate::src::nvim::eval::typval::{
    tv_dict_add_bool, tv_dict_add_list, tv_dict_add_str, tv_dict_set_keys_readonly, tv_list_alloc,
    tv_list_append_allocated_string, tv_list_append_string,
};
use crate::src::nvim::eval::{eval_to_string, get_v_event, restore_v_event};
use crate::src::nvim::ex_cmds2::check_fname;
use crate::src::nvim::ex_getln::{cmdline_paste_str, getcmdline};
use crate::src::nvim::extmark::{extmark_splice, extmark_splice_cols};
use crate::src::nvim::file_search::file_name_at_cursor;
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::garray::{ga_append, ga_clear, ga_concat_len, ga_init, ga_set_growsize};
use crate::src::nvim::getchar::{
    AppendCharToRedobuff, beep_flush, get_recorded, ins_typebuf, stuffReadbuff, stuffcharReadbuff,
    stuffescaped,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::HLF_8;
use crate::src::nvim::indent::{get_indent, preprocs_left, set_indent, tabstop_padding};
use crate::src::nvim::insexpand::{ins_compl_delete, ins_compl_preinsert_effect};
use crate::src::nvim::keycodes::{vim_strsave_escape_ks, vim_unescape_ks};
use crate::src::nvim::main::{
    Columns, State, VIsual_active, VIsual_mode, cmdmod, curbuf, curwin, e_nobufnr, e_noinstext,
    e_nolastcmd, e_noprevre, e_resulting_text_too_long, got_int, last_cmdline, msg_ext_skip_flush,
    must_redraw, new_last_cmdline, p_ch, p_cpo, p_report, p_sel, pending_end_reg_executing,
    redir_reg, reg_executing, reg_recorded, reg_recording, restart_edit, textlock,
};
use crate::src::nvim::mark::mark_adjust;
use crate::src::nvim::mbyte::{
    mb_charlen, mb_string2cells, mb_string2cells_len, mb_tolower, utf_head_off,
    utf_ptr2StrCharInfo, utf_ptr2cells_len, utf_ptr2len_len, utfc_next, utfc_ptr2len,
};
use crate::src::nvim::memline::{decl, ml_append, ml_get, ml_get_buf, ml_get_len, ml_replace};
use crate::src::nvim::memory::{
    memchrsub, memcnt, xcalloc, xfree, xmalloc, xmallocz, xmemdupz, xrealloc, xstrdup,
};
use crate::src::nvim::message::{
    emsg, emsg_invreg, message_filtered, msg, msg_ext_set_kind, msg_outtrans_len, msg_putchar,
    msg_puts, msg_puts_hl, msg_puts_title, msgmore, semsg, smsg,
};
use crate::src::nvim::r#move::{changed_cline_bef_curs, invalidate_botline_win, update_topline};
use crate::src::nvim::normal::find_ident_under_cursor;
use crate::src::nvim::ops::{adjust_cursor_eol, block_prep, charwise_block_prep, get_op_char};
use crate::src::nvim::option::get_ve_flags;
use crate::src::nvim::options::{kOptVeFlagAll, kOptVeFlagOnemore};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, atoi, gettext, memcpy, memmove, memset, ngettext, snprintf, strcpy,
    strlen, strncmp,
};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::plines::{getvcol, init_charsize_arg, win_charsize};
use crate::src::nvim::pos::{MAXCOL, MAXLNUM};
use crate::src::nvim::regexp::RE_SEARCH;
use crate::src::nvim::search::{BACKWARD, FORWARD, last_search_pat, set_last_search_pat};
use crate::src::nvim::state::REPLACE_FLAG;
use crate::src::nvim::strings::{vim_snprintf, vim_strchr, vim_strsave_escaped_ext};
use crate::src::nvim::terminal::terminal_paste;
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    AdditionalData, Arena, BoolVarValue, CMD_index, CharsizeArg, CharsizeKind, Direction,
    ExtmarkOp, GRegFlags, MotionType, OptInt, RemapValues, StrCharInfo, String_0, Timestamp,
    UndoObjectType, VAR_FIXED, bcount_t, block_def, buf_T, cmd_addr_T, colnr_T, dict_T, exarg_T,
    garray_T, hashitem_T, hashtab_T, int64_t, kBoolVarFalse, kBoolVarTrue, linenr_T, list_T,
    oparg_T, pos_T, ptrdiff_t, save_v_event_T, size_t, ssize_t, uint8_t, yankreg_T,
};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::undo::{u_save, u_save_cursor};
pub const kExtmarkMove: UndoObjectType = 1;
pub const kExtmarkSplice: UndoObjectType = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kExtmarkNoUndo: ExtmarkOp = 2;
pub const kExtmarkUndo: ExtmarkOp = 1;
pub const kExtmarkNOOP: ExtmarkOp = 0;
pub const CMD_USER: CMD_index = -1;
pub const CMD_snext: CMD_index = 414;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const CMOD_KEEPPATTERNS: C2Rust_Unnamed_18 = 4096;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_18 = 2048;
pub const CMOD_KEEPJUMPS: C2Rust_Unnamed_18 = 1024;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const PUT_BLOCK_INNER: C2Rust_Unnamed_20 = 64;
pub const PUT_LINE_FORWARD: C2Rust_Unnamed_20 = 32;
pub const PUT_LINE_SPLIT: C2Rust_Unnamed_20 = 16;
pub const PUT_LINE: C2Rust_Unnamed_20 = 8;
pub const PUT_CURSLINE: C2Rust_Unnamed_20 = 4;
pub const PUT_CURSEND: C2Rust_Unnamed_20 = 2;
pub const PUT_FIXINDENT: C2Rust_Unnamed_20 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NUM_REGISTERS: C2Rust_Unnamed_21 = 39;
pub const PLUS_REGISTER: C2Rust_Unnamed_21 = 38;
pub const STAR_REGISTER: C2Rust_Unnamed_21 = 37;
pub const NUM_SAVED_REGISTERS: C2Rust_Unnamed_21 = 37;
pub const DELETION_REGISTER: C2Rust_Unnamed_21 = 36;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const kGRegNoExpr: GRegFlags = 1;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const YREG_PUT: C2Rust_Unnamed_22 = 2;
pub const YREG_YANK: C2Rust_Unnamed_22 = 1;
pub const YREG_PASTE: C2Rust_Unnamed_22 = 0;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_23 = 4;
pub const BL_SOL: C2Rust_Unnamed_23 = 2;
pub const BL_WHITE: C2Rust_Unnamed_23 = 1;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const FNAME_HYP: C2Rust_Unnamed_24 = 4;
pub const FNAME_EXP: C2Rust_Unnamed_24 = 2;
pub const FNAME_MESS: C2Rust_Unnamed_24 = 1;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const SIN_NOMARK: C2Rust_Unnamed_26 = 8;
pub const SIN_UNDO: C2Rust_Unnamed_26 = 4;
pub const SIN_INSERT: C2Rust_Unnamed_26 = 2;
pub const SIN_CHANGED: C2Rust_Unnamed_26 = 1;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const FIND_STRING: C2Rust_Unnamed_27 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CPO_REGAPPEND: ::core::ffi::c_int = '>' as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_A: ::core::ffi::c_int = 1;
pub const Ctrl_F: ::core::ffi::c_int = 6;
pub const Ctrl_L: ::core::ffi::c_int = 12;
pub const Ctrl_P: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const Ctrl_R: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const Ctrl_U: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const Ctrl_W: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
static expr_line: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());
static execreg_lastc: GlobalCell<::core::ffi::c_int> = GlobalCell::new(NUL);
static y_regs: GlobalCell<[yankreg_T; 39]> = GlobalCell::new([
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
    yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    },
]);
static y_previous: GlobalCell<*mut yankreg_T> =
    GlobalCell::new(::core::ptr::null_mut::<yankreg_T>());
static e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines: GlobalCell<
    [::core::ffi::c_char; 79],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 79], [::core::ffi::c_char; 79]>(
        *b"E883: Search pattern and expression register may not contain two or more lines\0",
    )
});
pub unsafe extern "C" fn get_unname_register() -> ::core::ffi::c_int {
    return if (*y_previous.ptr()).is_null() {
        -1 as ::core::ffi::c_int
    } else {
        (*y_previous.ptr())
            .offset_from((y_regs.ptr() as *mut yankreg_T).offset(0 as ::core::ffi::c_int as isize))
            as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn get_y_register(mut reg: ::core::ffi::c_int) -> *mut yankreg_T {
    return (y_regs.ptr() as *mut yankreg_T).offset(reg as isize);
}
pub unsafe extern "C" fn get_y_previous() -> *mut yankreg_T {
    return y_previous.get();
}
pub unsafe extern "C" fn get_expr_register() -> ::core::ffi::c_int {
    let mut new_line: *mut ::core::ffi::c_char = getcmdline(
        '=' as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        true_0 != 0,
    );
    if new_line.is_null() {
        return NUL;
    }
    if *new_line as ::core::ffi::c_int == NUL {
        xfree(new_line as *mut ::core::ffi::c_void);
    } else {
        set_expr_line(new_line);
    }
    return '=' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn set_expr_line(mut new_line: *mut ::core::ffi::c_char) {
    xfree(expr_line.get() as *mut ::core::ffi::c_void);
    expr_line.set(new_line);
}
pub unsafe extern "C" fn get_expr_line() -> *mut ::core::ffi::c_char {
    static nested: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    if (*expr_line.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut expr_copy: *mut ::core::ffi::c_char = xstrdup(expr_line.get());
    if nested.get() >= 10 as ::core::ffi::c_int {
        return expr_copy;
    }
    (*nested.ptr()) += 1;
    let mut rv: *mut ::core::ffi::c_char = eval_to_string(expr_copy, true_0 != 0, false_0 != 0);
    (*nested.ptr()) -= 1;
    xfree(expr_copy as *mut ::core::ffi::c_void);
    return rv;
}
pub unsafe extern "C" fn get_expr_line_src() -> *mut ::core::ffi::c_char {
    if (*expr_line.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return xstrdup(expr_line.get());
}
pub unsafe extern "C" fn valid_yank_reg(
    mut regname: ::core::ffi::c_int,
    mut writing: bool,
) -> bool {
    if regname > 0 as ::core::ffi::c_int
        && (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0)
        || !writing
            && !vim_strchr(b"/.%:=\0".as_ptr() as *const ::core::ffi::c_char, regname).is_null()
        || regname == '#' as ::core::ffi::c_int
        || regname == '"' as ::core::ffi::c_int
        || regname == '-' as ::core::ffi::c_int
        || regname == '_' as ::core::ffi::c_int
        || regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
    {
        return true_0 != 0;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn get_default_register_name() -> ::core::ffi::c_int {
    let mut name: ::core::ffi::c_int = NUL;
    clipboard::adjust_clipboard_name(&mut name, true, false);
    return name;
}
pub unsafe extern "C" fn op_reg_iter(
    iter: *const ::core::ffi::c_void,
    regs: *const yankreg_T,
    name: *mut ::core::ffi::c_char,
    reg: *mut yankreg_T,
    mut is_unnamed: *mut bool,
) -> *const ::core::ffi::c_void {
    *name = NUL as ::core::ffi::c_char;
    let mut iter_reg: *const yankreg_T = if iter.is_null() {
        regs.offset(0 as ::core::ffi::c_int as isize)
    } else {
        iter as *const yankreg_T
    };
    while iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
        < NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
        && reg_empty(iter_reg) as ::core::ffi::c_int != 0
    {
        iter_reg = iter_reg.offset(1);
    }
    if iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
        == NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
        || reg_empty(iter_reg) as ::core::ffi::c_int != 0
    {
        return ::core::ptr::null::<::core::ffi::c_void>();
    }
    let mut iter_off: ::core::ffi::c_int =
        iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
    *name = get_register_name(iter_off) as ::core::ffi::c_char;
    *reg = *iter_reg;
    *is_unnamed = iter_reg == y_previous.get() as *const yankreg_T;
    loop {
        iter_reg = iter_reg.offset(1);
        if iter_reg.offset_from(regs.offset(0 as ::core::ffi::c_int as isize))
            >= NUM_SAVED_REGISTERS as ::core::ffi::c_int as isize
        {
            break;
        }
        if !reg_empty(iter_reg) {
            return iter_reg as *mut ::core::ffi::c_void;
        }
    }
    return ::core::ptr::null::<::core::ffi::c_void>();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn op_global_reg_iter(
    iter: *const ::core::ffi::c_void,
    name: *mut ::core::ffi::c_char,
    reg: *mut yankreg_T,
    mut is_unnamed: *mut bool,
) -> *const ::core::ffi::c_void {
    return op_reg_iter(iter, y_regs.ptr() as *mut yankreg_T, name, reg, is_unnamed);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn op_reg_set(
    name: ::core::ffi::c_char,
    reg: yankreg_T,
    mut is_unnamed: bool,
) -> bool {
    let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
    if i == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    free_register((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
    (*y_regs.ptr())[i as usize] = reg;
    if is_unnamed {
        y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn op_reg_get(name: ::core::ffi::c_char) -> *const yankreg_T {
    let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
    if i == -1 as ::core::ffi::c_int {
        return ::core::ptr::null::<yankreg_T>();
    }
    return (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
}
pub unsafe extern "C" fn op_reg_set_previous(name: ::core::ffi::c_char) -> bool {
    let mut i: ::core::ffi::c_int = op_reg_index(name as ::core::ffi::c_int);
    if i == -1 as ::core::ffi::c_int {
        return false_0 != 0;
    }
    y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(i as isize));
    return true_0 != 0;
}
pub unsafe extern "C" fn update_yankreg_width(mut reg: *mut yankreg_T) {
    if (*reg).y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        let mut maxlen: size_t = 0 as size_t;
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            let mut rowlen: size_t = mb_string2cells_len(
                (*(*reg).y_array.offset(i as isize)).data,
                (*(*reg).y_array.offset(i as isize)).size,
            );
            maxlen = if maxlen > rowlen { maxlen } else { rowlen };
            i = i.wrapping_add(1);
        }
        '_c2rust_label: {
            if maxlen <= 2147483647 as ::core::ffi::c_int as size_t {
            } else {
                __assert_fail(
                    b"maxlen <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    295 as ::core::ffi::c_uint,
                    b"void update_yankreg_width(yankreg_T *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        (*reg).y_width = (if (*reg).y_width > maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        {
            (*reg).y_width as ::core::ffi::c_int
        } else {
            maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        }) as colnr_T;
    }
}
pub unsafe extern "C" fn get_yank_register(
    mut regname: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
) -> *mut yankreg_T {
    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    if (mode == YREG_PASTE as ::core::ffi::c_int || mode == YREG_PUT as ::core::ffi::c_int)
        && clipboard::get_clipboard(regname, &mut reg, false)
    {
        return reg;
    } else if mode == YREG_PUT as ::core::ffi::c_int
        && (regname == '*' as ::core::ffi::c_int || regname == '+' as ::core::ffi::c_int)
    {
        static empty_reg: GlobalCell<yankreg_T> = GlobalCell::new(yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        });
        return empty_reg.ptr();
    } else if mode != YREG_YANK as ::core::ffi::c_int
        && (regname == 0 as ::core::ffi::c_int
            || regname == '"' as ::core::ffi::c_int
            || regname == '*' as ::core::ffi::c_int
            || regname == '+' as ::core::ffi::c_int)
        && !(*y_previous.ptr()).is_null()
    {
        return y_previous.get();
    }
    let mut i: ::core::ffi::c_int = op_reg_index(regname);
    if i == -1 as ::core::ffi::c_int {
        i = 0 as ::core::ffi::c_int;
    }
    reg = (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
    if mode == YREG_YANK as ::core::ffi::c_int {
        y_previous.set(reg);
    }
    return reg;
}
pub unsafe extern "C" fn yank_register_mline(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut *mut yankreg_T,
) -> bool {
    *reg = ::core::ptr::null_mut::<yankreg_T>();
    if regname != 0 as ::core::ffi::c_int && !valid_yank_reg(regname, false_0 != 0) {
        return false_0 != 0;
    }
    if regname == '_' as ::core::ffi::c_int {
        return false_0 != 0;
    }
    *reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
    return (**reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int;
}
pub unsafe extern "C" fn copy_register(mut name: ::core::ffi::c_int) -> *mut yankreg_T {
    let mut reg: *mut yankreg_T = get_yank_register(name, YREG_PASTE as ::core::ffi::c_int);
    let mut copy: *mut yankreg_T = xmalloc(::core::mem::size_of::<yankreg_T>()) as *mut yankreg_T;
    *copy = *reg;
    if (*copy).y_size == 0 as size_t {
        (*copy).y_array = ::core::ptr::null_mut::<String_0>();
    } else {
        (*copy).y_array =
            xcalloc((*copy).y_size, ::core::mem::size_of::<String_0>()) as *mut String_0;
        let mut i: size_t = 0 as size_t;
        while i < (*copy).y_size {
            *(*copy).y_array.offset(i as isize) = copy_string(
                *(*reg).y_array.offset(i as isize),
                ::core::ptr::null_mut::<Arena>(),
            );
            i = i.wrapping_add(1);
        }
    }
    return copy;
}
unsafe extern "C" fn stuff_yank(
    mut regname: ::core::ffi::c_int,
    mut p: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if regname != 0 as ::core::ffi::c_int && !valid_yank_reg(regname, true_0 != 0) {
        xfree(p as *mut ::core::ffi::c_void);
        return FAIL;
    }
    if regname == '_' as ::core::ffi::c_int {
        xfree(p as *mut ::core::ffi::c_void);
        return OK;
    }
    let plen: size_t = strlen(p);
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_YANK as ::core::ffi::c_int);
    if is_append_register(regname) as ::core::ffi::c_int != 0 && !(*reg).y_array.is_null() {
        let mut pp: *mut String_0 = (*reg)
            .y_array
            .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize);
        let tmplen: size_t = (*pp).size.wrapping_add(plen);
        let mut tmp: *mut ::core::ffi::c_char =
            xmalloc(tmplen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        memcpy(
            tmp as *mut ::core::ffi::c_void,
            (*pp).data as *const ::core::ffi::c_void,
            (*pp).size,
        );
        memcpy(
            tmp.offset((*pp).size as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            plen,
        );
        *tmp.offset(tmplen as isize) = NUL as ::core::ffi::c_char;
        xfree(p as *mut ::core::ffi::c_void);
        xfree((*pp).data as *mut ::core::ffi::c_void);
        *pp = String_0 {
            data: tmp,
            size: tmplen,
        };
    } else {
        free_register(reg);
        (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        (*reg).y_array = xmalloc(::core::mem::size_of::<String_0>()) as *mut String_0;
        *(*reg).y_array.offset(0 as ::core::ffi::c_int as isize) = String_0 {
            data: p,
            size: plen,
        };
        (*reg).y_size = 1 as size_t;
        (*reg).y_type = kMTCharWise;
    }
    (*reg).timestamp = os_time();
    return OK;
}
pub unsafe extern "C" fn do_record(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static regname: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    let mut retval: ::core::ffi::c_int = 0;
    if reg_recording.get() == 0 as ::core::ffi::c_int {
        if c < 0 as ::core::ffi::c_int
            || !(c as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && c as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                || c as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                    && c as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
                || ascii_isdigit(c) as ::core::ffi::c_int != 0)
                && c != '"' as ::core::ffi::c_int
        {
            retval = FAIL;
        } else {
            reg_recording.set(c);
            showmode();
            regname.set(c);
            retval = OK;
            apply_autocmds(
                EVENT_RECORDINGENTER,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
    } else {
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        let mut p: *mut ::core::ffi::c_char = get_recorded();
        if !p.is_null() {
            vim_unescape_ks(p);
            tv_dict_add_str(
                dict,
                b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                p,
            );
        }
        let mut buf: [::core::ffi::c_char; 67] = [0; 67];
        buf[0 as ::core::ffi::c_int as usize] = regname.get() as ::core::ffi::c_char;
        buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        tv_dict_add_str(
            dict,
            b"regname\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        tv_dict_set_keys_readonly(dict);
        apply_autocmds(
            EVENT_RECORDINGLEAVE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);
        reg_recorded.set(reg_recording.get());
        reg_recording.set(0 as ::core::ffi::c_int);
        if p_ch.get() == 0 as OptInt || ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
            showmode();
        } else {
            msg(
                b"\0".as_ptr() as *const ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
        }
        if p.is_null() {
            retval = FAIL;
        } else {
            let mut old_y_previous: *mut yankreg_T = y_previous.get();
            retval = stuff_yank(regname.get(), p);
            y_previous.set(old_y_previous);
        }
    }
    return retval;
}
unsafe extern "C" fn put_in_typebuf(
    mut s: *mut ::core::ffi::c_char,
    mut esc: bool,
    mut colon: bool,
    mut silent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    put_reedit_in_typebuf(silent);
    if colon {
        retval = ins_typebuf(
            b"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            REMAP_NONE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            silent != 0,
        );
    }
    if retval == OK {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if esc {
            p = vim_strsave_escape_ks(s);
        } else {
            p = s;
        }
        if p.is_null() {
            retval = FAIL;
        } else {
            retval = ins_typebuf(
                p,
                if esc as ::core::ffi::c_int != 0 {
                    REMAP_NONE as ::core::ffi::c_int
                } else {
                    REMAP_YES as ::core::ffi::c_int
                },
                0 as ::core::ffi::c_int,
                true_0 != 0,
                silent != 0,
            );
        }
        if esc {
            xfree(p as *mut ::core::ffi::c_void);
        }
    }
    if colon as ::core::ffi::c_int != 0 && retval == OK {
        retval = ins_typebuf(
            b":\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            REMAP_NONE as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            true_0 != 0,
            silent != 0,
        );
    }
    return retval;
}
unsafe extern "C" fn put_reedit_in_typebuf(mut silent: ::core::ffi::c_int) {
    let mut buf: [uint8_t; 3] = [0; 3];
    if restart_edit.get() == NUL {
        return;
    }
    if restart_edit.get() == 'V' as ::core::ffi::c_int {
        buf[0 as ::core::ffi::c_int as usize] = 'g' as uint8_t;
        buf[1 as ::core::ffi::c_int as usize] = 'R' as uint8_t;
        buf[2 as ::core::ffi::c_int as usize] = NUL as uint8_t;
    } else {
        buf[0 as ::core::ffi::c_int as usize] = (if restart_edit.get() == 'I' as ::core::ffi::c_int
        {
            'i' as ::core::ffi::c_int
        } else {
            restart_edit.get()
        }) as uint8_t;
        buf[1 as ::core::ffi::c_int as usize] = NUL as uint8_t;
    }
    if ins_typebuf(
        &raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char,
        REMAP_NONE as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        true_0 != 0,
        silent != 0,
    ) == OK
    {
        restart_edit.set(NUL);
    }
}
unsafe extern "C" fn execreg_line_continuation(
    mut lines: *mut String_0,
    mut idx: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut cmd_start: size_t = *idx;
    '_c2rust_label: {
        if cmd_start > 0 as size_t {
        } else {
            __assert_fail(
                b"cmd_start > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                575 as ::core::ffi::c_uint,
                b"char *execreg_line_continuation(String *, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let cmd_end: size_t = cmd_start;
    let mut ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut ga,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        400 as ::core::ffi::c_int,
    );
    loop {
        cmd_start = cmd_start.wrapping_sub(1);
        if cmd_start <= 0 as size_t {
            break;
        }
        let mut p: *mut ::core::ffi::c_char = skipwhite((*lines.offset(cmd_start as isize)).data);
        if *p as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
            && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '"' as ::core::ffi::c_int
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '\\' as ::core::ffi::c_int
                || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != ' ' as ::core::ffi::c_int)
        {
            break;
        }
    }
    let mut tmp: *mut String_0 = lines.offset(cmd_start as isize);
    ga_concat_len(&raw mut ga, (*tmp).data, (*tmp).size);
    let mut j: size_t = cmd_start.wrapping_add(1 as size_t);
    while j <= cmd_end {
        tmp = lines.offset(j as isize);
        let mut p_0: *mut ::core::ffi::c_char = skipwhite((*tmp).data);
        if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            if ga.ga_len > 400 as ::core::ffi::c_int {
                ga_set_growsize(
                    &raw mut ga,
                    if ga.ga_len < 8000 as ::core::ffi::c_int {
                        ga.ga_len
                    } else {
                        8000 as ::core::ffi::c_int
                    },
                );
            }
            p_0 = p_0.offset(1);
            ga_concat_len(
                &raw mut ga,
                p_0,
                (*tmp).data.offset((*tmp).size as isize).offset_from(p_0) as size_t,
            );
        }
        j = j.wrapping_add(1);
    }
    ga_append(&raw mut ga, NUL as uint8_t);
    let mut str: *mut ::core::ffi::c_char =
        xmemdupz(ga.ga_data, ga.ga_len as size_t) as *mut ::core::ffi::c_char;
    ga_clear(&raw mut ga);
    *idx = cmd_start;
    return str;
}
pub unsafe extern "C" fn do_execreg(
    mut regname: ::core::ffi::c_int,
    mut colon: ::core::ffi::c_int,
    mut addcr: ::core::ffi::c_int,
    mut silent: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    if regname == '@' as ::core::ffi::c_int {
        if execreg_lastc.get() == NUL {
            emsg(gettext(
                b"E748: No previously used register\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return FAIL;
        }
        regname = execreg_lastc.get();
    }
    if regname == '%' as ::core::ffi::c_int
        || regname == '#' as ::core::ffi::c_int
        || !valid_yank_reg(regname, false_0 != 0)
    {
        emsg_invreg(regname);
        return FAIL;
    }
    execreg_lastc.set(regname);
    if regname == '_' as ::core::ffi::c_int {
        return OK;
    }
    if regname == ':' as ::core::ffi::c_int {
        if (*last_cmdline.ptr()).is_null() {
            emsg(gettext(
                &raw const e_nolastcmd as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            new_last_cmdline.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        let mut p: *mut ::core::ffi::c_char = vim_strsave_escaped_ext(
            last_cmdline.get(),
            b"\x01\x02\x03\x04\x05\x06\x07\x08\t\n\x0B\x0C\r\x0E\x0F\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\0"
                .as_ptr() as *const ::core::ffi::c_char,
            Ctrl_V as ::core::ffi::c_char,
            false_0 != 0,
        );
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && strncmp(
                p,
                b"'<,'>\0".as_ptr() as *const ::core::ffi::c_char,
                5 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            retval = put_in_typebuf(
                p.offset(5 as ::core::ffi::c_int as isize),
                true_0 != 0,
                true_0 != 0,
                silent,
            );
        } else {
            retval = put_in_typebuf(p, true_0 != 0, true_0 != 0, silent);
        }
        xfree(p as *mut ::core::ffi::c_void);
    } else if regname == '=' as ::core::ffi::c_int {
        let mut p_0: *mut ::core::ffi::c_char = get_expr_line();
        if p_0.is_null() {
            return FAIL;
        }
        retval = put_in_typebuf(p_0, true_0 != 0, colon != 0, silent);
        xfree(p_0 as *mut ::core::ffi::c_void);
    } else if regname == '.' as ::core::ffi::c_int {
        let mut p_1: *mut ::core::ffi::c_char = get_last_insert_save();
        if p_1.is_null() {
            emsg(gettext(
                &raw const e_noinstext as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        retval = put_in_typebuf(p_1, false_0 != 0, colon != 0, silent);
        xfree(p_1 as *mut ::core::ffi::c_void);
    } else {
        let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        if (*reg).y_array.is_null() {
            return FAIL;
        }
        let mut remap: ::core::ffi::c_int = if colon != 0 {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            REMAP_YES as ::core::ffi::c_int
        };
        put_reedit_in_typebuf(silent);
        let mut i: size_t = (*reg).y_size;
        loop {
            let c2rust_fresh1 = i;
            i = i.wrapping_sub(1);
            if c2rust_fresh1 <= 0 as size_t {
                break;
            }
            if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                || i < (*reg).y_size.wrapping_sub(1 as size_t)
                || addcr != 0
            {
                if ins_typebuf(
                    b"\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    remap,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    silent != 0,
                ) == FAIL
                {
                    return FAIL;
                }
            }
            let mut str: *mut ::core::ffi::c_char = (*(*reg).y_array.offset(i as isize)).data;
            let mut free_str: bool = false_0 != 0;
            if colon != 0 && i > 0 as size_t {
                let mut p_2: *mut ::core::ffi::c_char = skipwhite(str);
                if *p_2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p_2.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '"' as ::core::ffi::c_int
                        && *p_2.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '\\' as ::core::ffi::c_int
                        && *p_2.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == ' ' as ::core::ffi::c_int
                {
                    str = execreg_line_continuation((*reg).y_array, &raw mut i);
                    free_str = true_0 != 0;
                }
            }
            let mut escaped: *mut ::core::ffi::c_char = vim_strsave_escape_ks(str);
            if free_str {
                xfree(str as *mut ::core::ffi::c_void);
            }
            retval = ins_typebuf(
                escaped,
                remap,
                0 as ::core::ffi::c_int,
                true_0 != 0,
                silent != 0,
            );
            xfree(escaped as *mut ::core::ffi::c_void);
            if retval == FAIL {
                return FAIL;
            }
            if colon != 0
                && ins_typebuf(
                    b":\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                    remap,
                    0 as ::core::ffi::c_int,
                    true_0 != 0,
                    silent != 0,
                ) == FAIL
            {
                return FAIL;
            }
        }
        reg_executing.set(if regname == 0 as ::core::ffi::c_int {
            '"' as ::core::ffi::c_int
        } else {
            regname
        });
        pending_end_reg_executing.set(false_0 != 0);
    }
    return retval;
}
pub unsafe extern "C" fn insert_reg(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut literally_arg: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = OK;
    let mut allocated: bool = false;
    let literally: bool = literally_arg as ::core::ffi::c_int != 0
        || is_literal_register(regname) as ::core::ffi::c_int != 0;
    os_breakcheck();
    if got_int.get() {
        return FAIL;
    }
    if regname != NUL && !valid_yank_reg(regname, false_0 != 0) {
        return FAIL;
    }
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if regname == '.' as ::core::ffi::c_int {
        retval = stuff_inserted(NUL, 1 as ::core::ffi::c_int, true_0);
    } else if get_spec_reg(regname, &raw mut arg, &raw mut allocated, true_0 != 0) {
        if arg.is_null() {
            return FAIL;
        }
        stuffescaped(arg, literally);
        if allocated {
            xfree(arg as *mut ::core::ffi::c_void);
        }
    } else {
        if reg.is_null() {
            reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        }
        if (*reg).y_array.is_null() {
            retval = FAIL;
        } else {
            let mut i: size_t = 0 as size_t;
            while i < (*reg).y_size {
                if regname == '-' as ::core::ffi::c_int
                    && (*reg).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                {
                    let mut dir: Direction = BACKWARD;
                    if State.get() & REPLACE_FLAG != 0 as ::core::ffi::c_int {
                        let mut curpos: pos_T = pos_T {
                            lnum: 0,
                            col: 0,
                            coladd: 0,
                        };
                        if u_save_cursor() == FAIL {
                            return FAIL;
                        }
                        del_chars(
                            mb_charlen(
                                (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data,
                            ),
                            true_0,
                        );
                        curpos = (*curwin.get()).w_cursor;
                        if oneright() == FAIL {
                            dir = FORWARD;
                        }
                        (*curwin.get()).w_cursor = curpos;
                    }
                    AppendCharToRedobuff(Ctrl_R);
                    AppendCharToRedobuff(regname);
                    do_put(
                        regname,
                        ::core::ptr::null_mut::<yankreg_T>(),
                        dir as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                        PUT_CURSEND as ::core::ffi::c_int,
                    );
                } else {
                    stuffescaped((*(*reg).y_array.offset(i as isize)).data, literally);
                    if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                        || i < (*reg).y_size.wrapping_sub(1 as size_t)
                    {
                        stuffcharReadbuff('\n' as ::core::ffi::c_int);
                    }
                }
                i = i.wrapping_add(1);
            }
        }
    }
    return retval;
}
pub unsafe extern "C" fn get_spec_reg(
    mut regname: ::core::ffi::c_int,
    mut argp: *mut *mut ::core::ffi::c_char,
    mut allocated: *mut bool,
    mut errmsg: bool,
) -> bool {
    *argp = ::core::ptr::null_mut::<::core::ffi::c_char>();
    *allocated = false_0 != 0;
    let mut cnt: size_t = 0;
    match regname {
        37 => {
            if errmsg {
                check_fname();
            }
            *argp = (*curbuf.get()).b_fname;
            return true_0 != 0;
        }
        35 => {
            *argp = getaltfname(errmsg);
            return true_0 != 0;
        }
        61 => {
            *argp = get_expr_line();
            *allocated = true_0 != 0;
            return true_0 != 0;
        }
        58 => {
            if (*last_cmdline.ptr()).is_null() && errmsg as ::core::ffi::c_int != 0 {
                emsg(gettext(
                    &raw const e_nolastcmd as *const ::core::ffi::c_char,
                ));
            }
            *argp = last_cmdline.get();
            return true_0 != 0;
        }
        47 => {
            if last_search_pat().is_null() && errmsg as ::core::ffi::c_int != 0 {
                emsg(gettext(&raw const e_noprevre as *const ::core::ffi::c_char));
            }
            *argp = last_search_pat();
            return true_0 != 0;
        }
        46 => {
            *argp = get_last_insert_save();
            *allocated = true_0 != 0;
            if (*argp).is_null() && errmsg as ::core::ffi::c_int != 0 {
                emsg(gettext(
                    &raw const e_noinstext as *const ::core::ffi::c_char,
                ));
            }
            return true_0 != 0;
        }
        Ctrl_F | Ctrl_P => {
            if !errmsg {
                return false_0 != 0;
            }
            *argp = file_name_at_cursor(
                FNAME_MESS as ::core::ffi::c_int
                    | FNAME_HYP as ::core::ffi::c_int
                    | (if regname == Ctrl_P {
                        FNAME_EXP as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<linenr_T>(),
            );
            *allocated = true_0 != 0;
            return true_0 != 0;
        }
        Ctrl_W | Ctrl_A => {
            if !errmsg {
                return false_0 != 0;
            }
            cnt = find_ident_under_cursor(
                argp,
                if regname == Ctrl_W {
                    FIND_IDENT as ::core::ffi::c_int | FIND_STRING as ::core::ffi::c_int
                } else {
                    FIND_STRING as ::core::ffi::c_int
                },
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            *argp = (if cnt != 0 {
                xmemdupz(*argp as *const ::core::ffi::c_void, cnt)
            } else {
                NULL_0
            }) as *mut ::core::ffi::c_char;
            *allocated = true_0 != 0;
            return true_0 != 0;
        }
        Ctrl_L => {
            if !errmsg {
                return false_0 != 0;
            }
            *argp = ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum);
            return true_0 != 0;
        }
        95 => {
            *argp = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            return true_0 != 0;
        }
        _ => {}
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn cmdline_paste_reg(
    mut regname: ::core::ffi::c_int,
    mut literally_arg: bool,
    mut remcr: bool,
) -> bool {
    let literally: bool = literally_arg as ::core::ffi::c_int != 0
        || is_literal_register(regname) as ::core::ffi::c_int != 0;
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
    if (*reg).y_array.is_null() {
        return FAIL != 0;
    }
    let mut i: size_t = 0 as size_t;
    while i < (*reg).y_size {
        cmdline_paste_str((*(*reg).y_array.offset(i as isize)).data, literally);
        if i < (*reg).y_size.wrapping_sub(1 as size_t) && !remcr {
            cmdline_paste_str(b"\r\0".as_ptr() as *const ::core::ffi::c_char, literally);
        }
        os_breakcheck();
        if got_int.get() {
            return FAIL != 0;
        }
        i = i.wrapping_add(1);
    }
    return OK != 0;
}
pub unsafe extern "C" fn shift_delete_registers(mut y_append: bool) {
    free_register((y_regs.ptr() as *mut yankreg_T).offset(9 as ::core::ffi::c_int as isize));
    let mut n: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
    while n > 1 as ::core::ffi::c_int {
        (*y_regs.ptr())[n as usize] = (*y_regs.ptr())[(n - 1 as ::core::ffi::c_int) as usize];
        n -= 1;
    }
    if !y_append {
        y_previous.set((y_regs.ptr() as *mut yankreg_T).offset(1 as ::core::ffi::c_int as isize));
    }
    (*y_regs.ptr())[1 as ::core::ffi::c_int as usize].y_array = ::core::ptr::null_mut::<String_0>();
}
pub unsafe extern "C" fn free_register(mut reg: *mut yankreg_T) {
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*reg).additional_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL_0;
    let _ = *ptr_;
    if (*reg).y_array.is_null() {
        return;
    }
    let mut i: size_t = (*reg).y_size;
    loop {
        let c2rust_fresh0 = i;
        i = i.wrapping_sub(1);
        if c2rust_fresh0 <= 0 as size_t {
            break;
        }
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*(*reg).y_array.offset(i as isize)).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_0;
        let _ = *ptr__0;
        (*(*reg).y_array.offset(i as isize)).size = 0 as size_t;
    }
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*reg).y_array as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL_0;
    let _ = *ptr__1;
}
unsafe extern "C" fn yank_copy_line(
    mut reg: *mut yankreg_T,
    mut bd: *mut block_def,
    mut y_idx: size_t,
    mut exclude_trailing_space: bool,
) {
    if exclude_trailing_space {
        (*bd).endspaces = 0 as ::core::ffi::c_int;
    }
    let mut size: ::core::ffi::c_int = (*bd).startspaces + (*bd).endspaces + (*bd).textlen;
    '_c2rust_label: {
        if size >= 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"size >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                985 as ::core::ffi::c_uint,
                b"void yank_copy_line(yankreg_T *, struct block_def *, size_t, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut pnew: *mut ::core::ffi::c_char = xmallocz(size as size_t) as *mut ::core::ffi::c_char;
    (*(*reg).y_array.offset(y_idx as isize)).data = pnew;
    memset(
        pnew as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).startspaces as size_t,
    );
    pnew = pnew.offset((*bd).startspaces as isize);
    memmove(
        pnew as *mut ::core::ffi::c_void,
        (*bd).textstart as *const ::core::ffi::c_void,
        (*bd).textlen as size_t,
    );
    pnew = pnew.offset((*bd).textlen as isize);
    memset(
        pnew as *mut ::core::ffi::c_void,
        ' ' as ::core::ffi::c_int,
        (*bd).endspaces as size_t,
    );
    pnew = pnew.offset((*bd).endspaces as isize);
    if exclude_trailing_space {
        let mut s: ::core::ffi::c_int = (*bd).textlen + (*bd).endspaces;
        while s > 0 as ::core::ffi::c_int
            && ascii_iswhite(
                *(*bd)
                    .textstart
                    .offset(s as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                != 0
        {
            s =
                s - utf_head_off(
                    (*bd).textstart,
                    (*bd)
                        .textstart
                        .offset(s as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize)),
                ) - 1 as ::core::ffi::c_int;
            pnew = pnew.offset(-1);
        }
    }
    *pnew = NUL as ::core::ffi::c_char;
    (*(*reg).y_array.offset(y_idx as isize)).size =
        pnew.offset_from((*(*reg).y_array.offset(y_idx as isize)).data) as size_t;
}
pub unsafe extern "C" fn op_yank_reg(
    mut oap: *mut oparg_T,
    mut message: bool,
    mut reg: *mut yankreg_T,
    mut append: bool,
) {
    let mut newreg: yankreg_T = yankreg_T {
        y_array: ::core::ptr::null_mut::<String_0>(),
        y_size: 0,
        y_type: kMTCharWise,
        y_width: 0,
        timestamp: 0,
        additional_data: ::core::ptr::null_mut::<AdditionalData>(),
    };
    let mut yank_type: MotionType = (*oap).motion_type;
    let mut yanklines: size_t = (*oap).line_count as size_t;
    let mut yankendlnum: linenr_T = (*oap).end.lnum;
    let mut bd: block_def = block_def {
        startspaces: 0,
        endspaces: 0,
        textlen: 0,
        textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        textcol: 0,
        start_vcol: 0,
        end_vcol: 0,
        is_short: 0,
        is_MAX: 0,
        is_oneChar: 0,
        pre_whitesp: 0,
        pre_whitesp_c: 0,
        end_char_vcols: 0,
        start_char_vcols: 0,
    };
    let mut curr: *mut yankreg_T = reg;
    if append as ::core::ffi::c_int != 0 && !(*reg).y_array.is_null() {
        reg = &raw mut newreg;
    } else {
        free_register(reg);
    }
    if (*oap).motion_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
        && (*oap).start.col == 0 as ::core::ffi::c_int
        && !(*oap).inclusive
        && (!(*oap).is_VIsual || *p_sel.get() as ::core::ffi::c_int == 'o' as ::core::ffi::c_int)
        && (*oap).end.col == 0 as ::core::ffi::c_int
        && yanklines > 1 as size_t
    {
        yank_type = kMTLineWise;
        yankendlnum -= 1;
        yanklines = yanklines.wrapping_sub(1);
    }
    (*reg).y_size = yanklines;
    (*reg).y_type = yank_type;
    (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
    (*reg).y_array = xcalloc(yanklines, ::core::mem::size_of::<String_0>()) as *mut String_0;
    (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    (*reg).timestamp = os_time();
    let mut y_idx: size_t = 0 as size_t;
    let mut lnum: linenr_T = (*oap).start.lnum;
    if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        (*reg).y_width = (*oap).end_vcol - (*oap).start_vcol;
        if (*curwin.get()).w_curswant == MAXCOL as ::core::ffi::c_int
            && (*reg).y_width > 0 as ::core::ffi::c_int
        {
            (*reg).y_width -= 1;
        }
    }
    while lnum <= yankendlnum {
        let mut tmp: ::core::ffi::c_int = 0;
        match (*reg).y_type as ::core::ffi::c_int {
            2 => {
                block_prep(oap, &raw mut bd, lnum, false_0 != 0);
                yank_copy_line(reg, &raw mut bd, y_idx, (*oap).excl_tr_ws);
            }
            1 => {
                *(*reg).y_array.offset(y_idx as isize) =
                    cbuf_to_string(ml_get(lnum), ml_get_len(lnum) as size_t);
            }
            0 => {
                charwise_block_prep(
                    (*oap).start,
                    (*oap).end,
                    &raw mut bd,
                    lnum,
                    (*oap).inclusive,
                );
                tmp = strlen(bd.textstart) as ::core::ffi::c_int;
                if tmp < bd.textlen {
                    bd.textlen = tmp;
                }
                yank_copy_line(reg, &raw mut bd, y_idx, false_0 != 0);
            }
            -1 => {
                abort();
            }
            _ => {}
        }
        lnum += 1;
        y_idx = y_idx.wrapping_add(1);
    }
    if curr != reg {
        let mut j: size_t = 0;
        let mut new_ptr: *mut String_0 = xmalloc(
            ::core::mem::size_of::<String_0>()
                .wrapping_mul((*curr).y_size.wrapping_add((*reg).y_size)),
        ) as *mut String_0;
        j = 0 as size_t;
        while j < (*curr).y_size {
            *new_ptr.offset(j as isize) = *(*curr).y_array.offset(j as isize);
            j = j.wrapping_add(1);
        }
        xfree((*curr).y_array as *mut ::core::ffi::c_void);
        (*curr).y_array = new_ptr;
        if yank_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            (*curr).y_type = kMTLineWise;
        }
        if (*curr).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && vim_strchr(p_cpo.get(), CPO_REGAPPEND).is_null()
        {
            let mut pnew: *mut ::core::ffi::c_char = xmalloc(
                (*(*curr)
                    .y_array
                    .offset((*curr).y_size.wrapping_sub(1 as size_t) as isize))
                .size
                .wrapping_add((*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size)
                .wrapping_add(1 as size_t),
            ) as *mut ::core::ffi::c_char;
            j = j.wrapping_sub(1);
            strcpy(pnew, (*(*curr).y_array.offset(j as isize)).data);
            strcpy(
                pnew.offset((*(*curr).y_array.offset(j as isize)).size as isize),
                (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data,
            );
            xfree((*(*curr).y_array.offset(j as isize)).data as *mut ::core::ffi::c_void);
            *(*curr).y_array.offset(j as isize) = String_0 {
                data: pnew,
                size: (*(*curr).y_array.offset(j as isize))
                    .size
                    .wrapping_add((*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size),
            };
            j = j.wrapping_add(1);
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).data
                    as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size = 0 as size_t;
            y_idx = 1 as size_t;
        } else {
            y_idx = 0 as size_t;
        }
        while y_idx < (*reg).y_size {
            let c2rust_fresh2 = y_idx;
            y_idx = y_idx.wrapping_add(1);
            let c2rust_fresh3 = j;
            j = j.wrapping_add(1);
            *(*curr).y_array.offset(c2rust_fresh3 as isize) =
                *(*reg).y_array.offset(c2rust_fresh2 as isize);
        }
        (*curr).y_size = j;
        xfree((*reg).y_array as *mut ::core::ffi::c_void);
    }
    if message {
        if yank_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && yanklines == 1 as size_t
        {
            yanklines = 0 as size_t;
        }
        if yanklines > p_report.get() as size_t {
            let mut namebuf: [::core::ffi::c_char; 100] = [0; 100];
            if (*oap).regname == NUL {
                *(&raw mut namebuf as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
            } else {
                vim_snprintf(
                    &raw mut namebuf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 100]>(),
                    gettext(b" into \"%c\0".as_ptr() as *const ::core::ffi::c_char),
                    (*oap).regname,
                );
            }
            update_topline(curwin.get());
            if must_redraw.get() != 0 {
                update_screen();
            }
            if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"block of %ld line yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"block of %ld lines yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        yanklines as ::core::ffi::c_ulong,
                    ),
                    yanklines as int64_t,
                    &raw mut namebuf as *mut ::core::ffi::c_char,
                );
            } else {
                smsg(
                    0 as ::core::ffi::c_int,
                    ngettext(
                        b"%ld line yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        b"%ld lines yanked%s\0".as_ptr() as *const ::core::ffi::c_char,
                        yanklines as ::core::ffi::c_ulong,
                    ),
                    yanklines as int64_t,
                    &raw mut namebuf as *mut ::core::ffi::c_char,
                );
            }
        }
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        (*curbuf.get()).b_op_start = (*oap).start;
        (*curbuf.get()).b_op_end = (*oap).end;
        if yank_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
            (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
            (*curbuf.get()).b_op_end.col = MAXCOL as ::core::ffi::c_int as colnr_T;
        }
        if yank_type as ::core::ffi::c_int != kMTLineWise as ::core::ffi::c_int && !(*oap).inclusive
        {
            decl(&raw mut (*curbuf.get()).b_op_end);
        }
    }
}
pub unsafe extern "C" fn format_reg_type(
    mut reg_type: MotionType,
    mut reg_width: colnr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
) {
    '_c2rust_label: {
        if buf_len > 1 as size_t {
        } else {
            __assert_fail(
                b"buf_len > 1\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/register.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1176 as ::core::ffi::c_uint,
                b"void format_reg_type(MotionType, colnr_T, char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    match reg_type as ::core::ffi::c_int {
        1 => {
            *buf.offset(0 as ::core::ffi::c_int as isize) = 'V' as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        0 => {
            *buf.offset(0 as ::core::ffi::c_int as isize) = 'v' as ::core::ffi::c_char;
            *buf.offset(1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        2 => {
            snprintf(
                buf,
                buf_len,
                b"\x16%d\0".as_ptr() as *const ::core::ffi::c_char,
                reg_width as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
        }
        -1 => {
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        _ => {}
    };
}
pub unsafe extern "C" fn do_autocmd_textyankpost(mut oap: *mut oparg_T, mut reg: *mut yankreg_T) {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() as ::core::ffi::c_int != 0 || !has_event(EVENT_TEXTYANKPOST) {
        return;
    }
    recursive.set(true_0 != 0);
    let mut save_v_event: save_v_event_T = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: hashtab_T {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut::<hashitem_T>(),
            ht_smallarray: [hashitem_T {
                hi_hash: 0,
                hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            }; 16],
        },
    };
    let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
    let list: *mut list_T = tv_list_alloc((*reg).y_size as ptrdiff_t);
    let mut i: size_t = 0 as size_t;
    while i < (*reg).y_size {
        tv_list_append_string(
            list,
            (*(*reg).y_array.offset(i as isize)).data,
            (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
        );
        i = i.wrapping_add(1);
    }
    tv_list_set_lock(list, VAR_FIXED);
    tv_dict_add_list(
        dict,
        b"regcontents\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
        list,
    );
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    format_reg_type(
        (*reg).y_type,
        (*reg).y_width,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 67]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 67]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as size_t,
            ),
    );
    tv_dict_add_str(
        dict,
        b"regtype\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    buf[0 as ::core::ffi::c_int as usize] = (*oap).regname as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    tv_dict_add_str(
        dict,
        b"regname\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    tv_dict_add_bool(
        dict,
        b"inclusive\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (if (*oap).inclusive as ::core::ffi::c_int != 0 {
            kBoolVarTrue as ::core::ffi::c_int
        } else {
            kBoolVarFalse as ::core::ffi::c_int
        }) as BoolVarValue,
    );
    buf[0 as ::core::ffi::c_int as usize] = get_op_char((*oap).op_type) as ::core::ffi::c_char;
    buf[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    tv_dict_add_str(
        dict,
        b"operator\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    tv_dict_add_bool(
        dict,
        b"visual\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        (if (*oap).is_VIsual as ::core::ffi::c_int != 0 {
            kBoolVarTrue as ::core::ffi::c_int
        } else {
            kBoolVarFalse as ::core::ffi::c_int
        }) as BoolVarValue,
    );
    tv_dict_set_keys_readonly(dict);
    (*textlock.ptr()) += 1;
    apply_autocmds(
        EVENT_TEXTYANKPOST,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    (*textlock.ptr()) -= 1;
    restore_v_event(dict, &raw mut save_v_event);
    recursive.set(false_0 != 0);
}
pub unsafe extern "C" fn op_yank(mut oap: *mut oparg_T, mut message: bool) -> bool {
    if (*oap).regname != 0 as ::core::ffi::c_int && !valid_yank_reg((*oap).regname, true_0 != 0) {
        beep_flush();
        return false_0 != 0;
    }
    if (*oap).regname == '_' as ::core::ffi::c_int {
        return true_0 != 0;
    }
    let mut reg: *mut yankreg_T =
        get_yank_register((*oap).regname, YREG_YANK as ::core::ffi::c_int);
    op_yank_reg(oap, message, reg, is_append_register((*oap).regname));
    clipboard::set_clipboard((*oap).regname, reg);
    do_autocmd_textyankpost(oap, reg);
    return true_0 != 0;
}
pub unsafe extern "C" fn do_put(
    mut regname: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut dir: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) {
    let mut split_pos: colnr_T = 0;
    let mut col: colnr_T = 0;
    let mut len_0: ::core::ffi::c_int = 0;
    let mut totlen: size_t = 0 as size_t;
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut y_type: MotionType = kMTCharWise;
    let mut y_size: size_t = 0;
    let mut y_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut vcol: colnr_T = 0 as colnr_T;
    let mut y_array: *mut String_0 = ::core::ptr::null_mut::<String_0>();
    let mut nr_lines: linenr_T = 0 as linenr_T;
    let mut allocated: bool = false_0 != 0;
    let orig_start: pos_T = (*curbuf.get()).b_op_start;
    let orig_end: pos_T = (*curbuf.get()).b_op_end;
    let mut cur_ve_flags: ::core::ffi::c_uint = get_ve_flags(curwin.get());
    if ins_compl_preinsert_effect() {
        ins_compl_delete(false_0 != 0);
    }
    (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
    (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
    if regname == '.' as ::core::ffi::c_int && reg.is_null() {
        let mut non_linewise_vis: bool = VIsual_active.get() as ::core::ffi::c_int != 0
            && VIsual_mode.get() != 'V' as ::core::ffi::c_int;
        let mut command_start_char: ::core::ffi::c_char =
            (if non_linewise_vis as ::core::ffi::c_int != 0 {
                'c' as ::core::ffi::c_int
            } else if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                'i' as ::core::ffi::c_int
            } else if dir == FORWARD as ::core::ffi::c_int {
                'a' as ::core::ffi::c_int
            } else {
                'i' as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
        if flags & PUT_LINE as ::core::ffi::c_int != 0 {
            do_put(
                '_' as ::core::ffi::c_int,
                ::core::ptr::null_mut::<yankreg_T>(),
                dir,
                1 as ::core::ffi::c_int,
                PUT_LINE as ::core::ffi::c_int,
            );
        }
        if flags & PUT_LINE as ::core::ffi::c_int != 0 {
            stuffcharReadbuff(command_start_char as ::core::ffi::c_int);
            while count > 0 as ::core::ffi::c_int {
                stuff_inserted(
                    NUL,
                    1 as ::core::ffi::c_int,
                    (count != 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
                );
                if count != 1 as ::core::ffi::c_int {
                    stuffReadbuff(b"\n \0".as_ptr() as *const ::core::ffi::c_char);
                    stuffcharReadbuff(Ctrl_U);
                }
                count -= 1;
            }
        } else {
            stuff_inserted(command_start_char as ::core::ffi::c_int, count, false_0);
        }
        if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
            if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                stuffReadbuff(b"j0\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                let mut cursor_pos: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                let mut one_past_line: bool = *cursor_pos as ::core::ffi::c_int == NUL;
                let mut eol: bool = false_0 != 0;
                if !one_past_line {
                    eol = *cursor_pos.offset(utfc_ptr2len(cursor_pos) as isize)
                        as ::core::ffi::c_int
                        == NUL;
                }
                let mut ve_allows: bool = cur_ve_flags
                    == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                    || cur_ve_flags
                        == kOptVeFlagOnemore as ::core::ffi::c_int as ::core::ffi::c_uint;
                let mut eof: bool = (*curbuf.get()).b_ml.ml_line_count
                    == (*curwin.get()).w_cursor.lnum
                    && one_past_line as ::core::ffi::c_int != 0;
                if ve_allows as ::core::ffi::c_int != 0
                    || !(eol as ::core::ffi::c_int != 0 || eof as ::core::ffi::c_int != 0)
                {
                    stuffcharReadbuff('l' as ::core::ffi::c_int);
                }
            }
        } else if flags & PUT_LINE as ::core::ffi::c_int != 0 {
            stuffReadbuff(b"g'[\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if command_start_char as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
            if u_save(
                (*curwin.get()).w_cursor.lnum,
                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
            ) == FAIL
            {
                return;
            }
        }
        return;
    }
    let mut insert_string: String_0 = String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0 as size_t,
    };
    if reg.is_null()
        && get_spec_reg(
            regname,
            &raw mut insert_string.data,
            &raw mut allocated,
            true_0 != 0,
        ) as ::core::ffi::c_int
            != 0
    {
        if insert_string.data.is_null() {
            return;
        }
    }
    if (*curbuf.get()).terminal.is_null() {
        if u_save(
            (*curwin.get()).w_cursor.lnum,
            (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
        ) == FAIL
        {
            return;
        }
    }
    if !insert_string.data.is_null() {
        insert_string.size = strlen(insert_string.data);
        y_type = kMTCharWise;
        if regname == '=' as ::core::ffi::c_int {
            loop {
                y_size = 0 as size_t;
                let mut ptr: *mut ::core::ffi::c_char = insert_string.data;
                let mut ptrlen: size_t = insert_string.size;
                while !ptr.is_null() {
                    if !y_array.is_null() {
                        (*y_array.offset(y_size as isize)).data = ptr;
                    }
                    y_size = y_size.wrapping_add(1);
                    let mut tmp: *mut ::core::ffi::c_char =
                        vim_strchr(ptr, '\n' as ::core::ffi::c_int);
                    if tmp.is_null() {
                        if !y_array.is_null() {
                            (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size =
                                ptrlen;
                        }
                    } else {
                        if !y_array.is_null() {
                            *tmp = NUL as ::core::ffi::c_char;
                            (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size =
                                tmp.offset_from(ptr) as size_t;
                            ptrlen = ptrlen.wrapping_sub(
                                (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                    .size
                                    .wrapping_add(1 as size_t),
                            );
                        }
                        tmp = tmp.offset(1);
                        if *tmp as ::core::ffi::c_int == NUL {
                            y_type = kMTLineWise;
                            break;
                        }
                    }
                    ptr = tmp;
                }
                if !y_array.is_null() {
                    break;
                }
                y_array = xmalloc(y_size.wrapping_mul(::core::mem::size_of::<String_0>()))
                    as *mut String_0;
            }
        } else {
            y_size = 1 as size_t;
            y_array = &raw mut insert_string;
        }
    } else {
        if reg.is_null() {
            reg = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
        }
        y_type = (*reg).y_type;
        y_width = (*reg).y_width as ::core::ffi::c_int;
        y_size = (*reg).y_size;
        y_array = (*reg).y_array;
    }
    '_end: {
        if !(*curbuf.get()).terminal.is_null() {
            terminal_paste(count, y_array, y_size);
        } else {
            split_pos = 0 as colnr_T;
            if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                if flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0 {
                    if u_save_cursor() == FAIL {
                        break '_end;
                    } else {
                        let mut curline: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                        let mut p: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        let p_orig: *mut ::core::ffi::c_char = p;
                        let plen: size_t = get_cursor_pos_len() as size_t;
                        if dir == FORWARD as ::core::ffi::c_int && *p as ::core::ffi::c_int != NUL {
                            p = p.offset(utfc_ptr2len(p) as isize);
                        }
                        split_pos = p.offset_from(curline) as colnr_T;
                        let mut ptr_0: *mut ::core::ffi::c_char = xmemdupz(
                            p as *const ::core::ffi::c_void,
                            plen.wrapping_sub(p.offset_from(p_orig) as size_t),
                        )
                            as *mut ::core::ffi::c_char;
                        ml_append(
                            (*curwin.get()).w_cursor.lnum,
                            ptr_0,
                            0 as colnr_T,
                            false_0 != 0,
                        );
                        xfree(ptr_0 as *mut ::core::ffi::c_void);
                        ptr_0 = xmemdupz(
                            get_cursor_line_ptr() as *const ::core::ffi::c_void,
                            split_pos as size_t,
                        ) as *mut ::core::ffi::c_char;
                        ml_replace((*curwin.get()).w_cursor.lnum, ptr_0, false_0 != 0);
                        nr_lines += 1;
                        dir = FORWARD as ::core::ffi::c_int;
                        buf_updates_send_changes(
                            curbuf.get(),
                            (*curwin.get()).w_cursor.lnum,
                            1 as int64_t,
                            1 as int64_t,
                        );
                    }
                }
                if flags & PUT_LINE_FORWARD as ::core::ffi::c_int != 0 {
                    (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_end;
                    dir = FORWARD as ::core::ffi::c_int;
                }
                (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
            }
            if flags & PUT_LINE as ::core::ffi::c_int != 0 {
                y_type = kMTLineWise;
            }
            if y_size == 0 as size_t || y_array.is_null() {
                semsg(
                    gettext(
                        b"E353: Nothing in register %s\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    if regname == 0 as ::core::ffi::c_int {
                        b"\"\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        transchar(regname) as *const ::core::ffi::c_char
                    },
                );
            } else {
                if y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    lnum = (*curwin.get()).w_cursor.lnum + y_size as linenr_T + 1 as linenr_T;
                    lnum = if lnum < (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T {
                        lnum
                    } else {
                        (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T
                    };
                    if u_save((*curwin.get()).w_cursor.lnum - 1 as linenr_T, lnum) == FAIL {
                        break '_end;
                    }
                } else if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                    lnum = (*curwin.get()).w_cursor.lnum;
                    if dir == BACKWARD as ::core::ffi::c_int {
                        hasFolding(
                            curwin.get(),
                            lnum,
                            &raw mut lnum,
                            ::core::ptr::null_mut::<linenr_T>(),
                        );
                    } else {
                        hasFolding(
                            curwin.get(),
                            lnum,
                            ::core::ptr::null_mut::<linenr_T>(),
                            &raw mut lnum,
                        );
                    }
                    if dir == FORWARD as ::core::ffi::c_int {
                        lnum += 1;
                    }
                    if (if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0 {
                        u_save(0 as linenr_T, 2 as linenr_T)
                    } else {
                        u_save(lnum - 1 as linenr_T, lnum)
                    }) == FAIL
                    {
                        break '_end;
                    } else {
                        if dir == FORWARD as ::core::ffi::c_int {
                            (*curwin.get()).w_cursor.lnum = lnum - 1 as linenr_T;
                        } else {
                            (*curwin.get()).w_cursor.lnum = lnum;
                        }
                        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    }
                } else if u_save_cursor() == FAIL {
                    break '_end;
                }
                if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                    && y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                {
                    if gchar_cursor() == TAB {
                        let mut viscol: ::core::ffi::c_int = getviscol();
                        let mut ts: OptInt = (*curbuf.get()).b_p_ts;
                        if if dir == FORWARD as ::core::ffi::c_int {
                            (tabstop_padding(viscol as colnr_T, ts, (*curbuf.get()).b_p_vts_array)
                                != 1 as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                        } else {
                            ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int)
                                as ::core::ffi::c_int
                        } != 0
                        {
                            coladvance_force(viscol as colnr_T);
                        } else {
                            (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                        }
                    } else if (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                        || gchar_cursor() == NUL
                    {
                        coladvance_force(
                            getviscol()
                                + (dir == FORWARD as ::core::ffi::c_int) as ::core::ffi::c_int,
                        );
                    }
                }
                lnum = (*curwin.get()).w_cursor.lnum;
                col = (*curwin.get()).w_cursor.col;
                if y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    let mut incr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut bd: block_def = block_def {
                        startspaces: 0,
                        endspaces: 0,
                        textlen: 0,
                        textstart: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        textcol: 0,
                        start_vcol: 0,
                        end_vcol: 0,
                        is_short: 0,
                        is_MAX: 0,
                        is_oneChar: 0,
                        pre_whitesp: 0,
                        pre_whitesp_c: 0,
                        end_char_vcols: 0,
                        start_char_vcols: 0,
                    };
                    let mut c: ::core::ffi::c_int = gchar_cursor();
                    let mut endcol2: colnr_T = 0 as colnr_T;
                    if dir == FORWARD as ::core::ffi::c_int && c != NUL {
                        if cur_ve_flags
                            == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            getvcol(
                                curwin.get(),
                                &raw mut (*curwin.get()).w_cursor,
                                &raw mut col,
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut endcol2,
                            );
                        } else {
                            getvcol(
                                curwin.get(),
                                &raw mut (*curwin.get()).w_cursor,
                                ::core::ptr::null_mut::<colnr_T>(),
                                ::core::ptr::null_mut::<colnr_T>(),
                                &raw mut col,
                            );
                        }
                        (*curwin.get()).w_cursor.col += utfc_ptr2len(get_cursor_pos_ptr());
                        col += 1;
                    } else {
                        getvcol(
                            curwin.get(),
                            &raw mut (*curwin.get()).w_cursor,
                            &raw mut col,
                            ::core::ptr::null_mut::<colnr_T>(),
                            &raw mut endcol2,
                        );
                    }
                    col += (*curwin.get()).w_cursor.coladd;
                    if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint
                        && ((*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                            || endcol2 == (*curwin.get()).w_cursor.col)
                    {
                        if dir == FORWARD as ::core::ffi::c_int && c == NUL {
                            col += 1;
                        }
                        if dir != FORWARD as ::core::ffi::c_int
                            && c != NUL
                            && (*curwin.get()).w_cursor.coladd > 0 as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_cursor.col += 1;
                        }
                        if c == TAB {
                            if dir == BACKWARD as ::core::ffi::c_int
                                && (*curwin.get()).w_cursor.col != 0
                            {
                                (*curwin.get()).w_cursor.col -= 1;
                            }
                            if dir == FORWARD as ::core::ffi::c_int
                                && col as ::core::ffi::c_int - 1 as ::core::ffi::c_int == endcol2
                            {
                                (*curwin.get()).w_cursor.col += 1;
                            }
                        }
                    }
                    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    bd.textcol = 0 as ::core::ffi::c_int as colnr_T;
                    let mut i: size_t = 0 as size_t;
                    while i < y_size {
                        let mut spaces: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut shortline: ::core::ffi::c_char = 0;
                        let mut lines_appended: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        bd.startspaces = 0 as ::core::ffi::c_int;
                        bd.endspaces = 0 as ::core::ffi::c_int;
                        vcol = 0 as ::core::ffi::c_int as colnr_T;
                        let mut delcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if (*curwin.get()).w_cursor.lnum > (*curbuf.get()).b_ml.ml_line_count {
                            if ml_append(
                                (*curbuf.get()).b_ml.ml_line_count,
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                                1 as colnr_T,
                                false_0 != 0,
                            ) == FAIL
                            {
                                break;
                            }
                            nr_lines += 1;
                            lines_appended = 1 as ::core::ffi::c_int;
                        }
                        let mut oldp: *mut ::core::ffi::c_char = get_cursor_line_ptr();
                        let mut oldlen: colnr_T = get_cursor_line_len();
                        let mut csarg: CharsizeArg = CharsizeArg::default();
                        let mut cstype: CharsizeKind = init_charsize_arg(
                            &mut csarg,
                            curwin.get(),
                            (*curwin.get()).w_cursor.lnum,
                            oldp,
                        );
                        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(oldp);
                        vcol = 0 as ::core::ffi::c_int as colnr_T;
                        while vcol < col && *ci.ptr as ::core::ffi::c_int != NUL {
                            incr = win_charsize(
                                cstype,
                                vcol as ::core::ffi::c_int,
                                ci.ptr,
                                ci.chr.value,
                                &mut csarg,
                            )
                            .width;
                            vcol += incr;
                            ci = utfc_next(ci);
                        }
                        let mut ptr_1: *mut ::core::ffi::c_char = ci.ptr;
                        bd.textcol = ptr_1.offset_from(oldp) as colnr_T;
                        shortline = (vcol < col || vcol == col && *ptr_1 == 0) as ::core::ffi::c_int
                            as ::core::ffi::c_char;
                        if vcol < col {
                            bd.startspaces = (col - vcol) as ::core::ffi::c_int;
                        } else if vcol > col {
                            bd.endspaces = (vcol - col) as ::core::ffi::c_int;
                            bd.startspaces = incr - bd.endspaces;
                            bd.textcol -= 1;
                            delcount = 1 as ::core::ffi::c_int;
                            bd.textcol -= utf_head_off(oldp, oldp.offset(bd.textcol as isize));
                            if *oldp.offset(bd.textcol as isize) as ::core::ffi::c_int != TAB {
                                delcount = 0 as ::core::ffi::c_int;
                                bd.endspaces = 0 as ::core::ffi::c_int;
                            }
                        }
                        let yanklen: ::core::ffi::c_int =
                            (*y_array.offset(i as isize)).size as ::core::ffi::c_int;
                        if flags & PUT_BLOCK_INNER as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                        {
                            spaces = y_width + 1 as ::core::ffi::c_int;
                            cstype = init_charsize_arg(
                                &mut csarg,
                                curwin.get(),
                                0 as linenr_T,
                                (*y_array.offset(i as isize)).data,
                            );
                            ci = utf_ptr2StrCharInfo((*y_array.offset(i as isize)).data);
                            while *ci.ptr as ::core::ffi::c_int != NUL {
                                spaces -= win_charsize(
                                    cstype,
                                    0 as ::core::ffi::c_int,
                                    ci.ptr,
                                    ci.chr.value,
                                    &mut csarg,
                                )
                                .width;
                                ci = utfc_next(ci);
                            }
                            spaces = if spaces > 0 as ::core::ffi::c_int {
                                spaces
                            } else {
                                0 as ::core::ffi::c_int
                            };
                        }
                        if yanklen + spaces != 0 as ::core::ffi::c_int
                            && count
                                > (INT_MAX - (bd.startspaces + bd.endspaces)) / (yanklen + spaces)
                        {
                            emsg(gettext(
                                &raw const e_resulting_text_too_long as *const ::core::ffi::c_char,
                            ));
                            break;
                        } else {
                            totlen = (count as size_t)
                                .wrapping_mul((yanklen + spaces) as size_t)
                                .wrapping_add(bd.startspaces as size_t)
                                .wrapping_add(bd.endspaces as size_t);
                            let mut newp: *mut ::core::ffi::c_char = xmalloc(
                                totlen
                                    .wrapping_add(oldlen as size_t)
                                    .wrapping_add(1 as size_t),
                            )
                                as *mut ::core::ffi::c_char;
                            ptr_1 = newp;
                            memmove(
                                ptr_1 as *mut ::core::ffi::c_void,
                                oldp as *const ::core::ffi::c_void,
                                bd.textcol as size_t,
                            );
                            ptr_1 = ptr_1.offset(bd.textcol as isize);
                            memset(
                                ptr_1 as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                bd.startspaces as size_t,
                            );
                            ptr_1 = ptr_1.offset(bd.startspaces as isize);
                            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            while j < count {
                                memmove(
                                    ptr_1 as *mut ::core::ffi::c_void,
                                    (*y_array.offset(i as isize)).data
                                        as *const ::core::ffi::c_void,
                                    yanklen as size_t,
                                );
                                ptr_1 = ptr_1.offset(yanklen as isize);
                                if (j < count - 1 as ::core::ffi::c_int || shortline == 0)
                                    && spaces > 0 as ::core::ffi::c_int
                                {
                                    memset(
                                        ptr_1 as *mut ::core::ffi::c_void,
                                        ' ' as ::core::ffi::c_int,
                                        spaces as size_t,
                                    );
                                    ptr_1 = ptr_1.offset(spaces as isize);
                                } else {
                                    totlen = totlen.wrapping_sub(spaces as size_t);
                                }
                                j += 1;
                            }
                            memset(
                                ptr_1 as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                bd.endspaces as size_t,
                            );
                            ptr_1 = ptr_1.offset(bd.endspaces as isize);
                            let mut columns: ::core::ffi::c_int = oldlen as ::core::ffi::c_int
                                - bd.textcol as ::core::ffi::c_int
                                - delcount
                                + 1 as ::core::ffi::c_int;
                            '_c2rust_label: {
                                if columns >= 0 as ::core::ffi::c_int {
                                } else {
                                    __assert_fail(
                                        b"columns >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                        b"src/nvim/register.rs\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        1731 as ::core::ffi::c_uint,
                                        b"void do_put(int, yankreg_T *, int, int, int)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            memmove(
                                ptr_1 as *mut ::core::ffi::c_void,
                                oldp.offset(bd.textcol as isize).offset(delcount as isize)
                                    as *const ::core::ffi::c_void,
                                columns as size_t,
                            );
                            ml_replace((*curwin.get()).w_cursor.lnum, newp, false_0 != 0);
                            extmark_splice_cols(
                                curbuf.get(),
                                (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int
                                    - 1 as ::core::ffi::c_int,
                                bd.textcol,
                                delcount as colnr_T,
                                totlen as colnr_T + lines_appended as colnr_T,
                                kExtmarkUndo,
                            );
                            (*curwin.get()).w_cursor.lnum += 1;
                            if i == 0 as size_t {
                                (*curwin.get()).w_cursor.col += bd.startspaces;
                            }
                            i = i.wrapping_add(1);
                        }
                    }
                    changed_lines(
                        curbuf.get(),
                        lnum,
                        0 as colnr_T,
                        (*curbuf.get()).b_op_start.lnum + y_size as linenr_T - nr_lines,
                        nr_lines,
                        true_0 != 0,
                    );
                    (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    (*curbuf.get()).b_op_start.lnum = lnum;
                    (*curbuf.get()).b_op_end.lnum = (*curwin.get()).w_cursor.lnum - 1 as linenr_T;
                    (*curbuf.get()).b_op_end.col = (if bd.textcol as ::core::ffi::c_int
                        + totlen as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int
                        > 0 as ::core::ffi::c_int
                    {
                        bd.textcol as ::core::ffi::c_int + totlen as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as colnr_T;
                    (*curbuf.get()).b_op_end.coladd = 0 as ::core::ffi::c_int as colnr_T;
                    if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                        (*curwin.get()).w_cursor = (*curbuf.get()).b_op_end;
                        (*curwin.get()).w_cursor.col += 1;
                        let mut len: colnr_T = get_cursor_line_len();
                        (*curwin.get()).w_cursor.col = if (*curwin.get()).w_cursor.col < len {
                            (*curwin.get()).w_cursor.col
                        } else {
                            len
                        };
                    } else {
                        (*curwin.get()).w_cursor.lnum = lnum;
                    }
                } else {
                    let yanklen_0: ::core::ffi::c_int =
                        (*y_array.offset(0 as ::core::ffi::c_int as isize)).size
                            as ::core::ffi::c_int;
                    if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                        if dir == FORWARD as ::core::ffi::c_int && gchar_cursor() != NUL {
                            let mut bytelen: ::core::ffi::c_int =
                                utfc_ptr2len(get_cursor_pos_ptr());
                            col += bytelen;
                            if yanklen_0 != 0 {
                                (*curwin.get()).w_cursor.col += bytelen;
                                (*curbuf.get()).b_op_end.col += bytelen;
                            }
                        }
                        (*curbuf.get()).b_op_start = (*curwin.get()).w_cursor;
                    } else if dir == BACKWARD as ::core::ffi::c_int {
                        lnum -= 1;
                    }
                    let mut new_cursor: pos_T = (*curwin.get()).w_cursor;
                    if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                        && y_size == 1 as size_t
                    {
                        let mut end_lnum: linenr_T = 0 as linenr_T;
                        let mut start_lnum: linenr_T = lnum;
                        let mut first_byte_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if VIsual_active.get() {
                            end_lnum = if (*curbuf.get()).b_visual.vi_end.lnum
                                > (*curbuf.get()).b_visual.vi_start.lnum
                            {
                                (*curbuf.get()).b_visual.vi_end.lnum
                            } else {
                                (*curbuf.get()).b_visual.vi_start.lnum
                            };
                            if end_lnum > start_lnum {
                                let mut pos: pos_T = pos_T {
                                    lnum: lnum,
                                    col: col,
                                    coladd: 0 as colnr_T,
                                };
                                getvcol(
                                    curwin.get(),
                                    &raw mut pos,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                    &raw mut vcol,
                                    ::core::ptr::null_mut::<colnr_T>(),
                                );
                            }
                        }
                        if count == 0 as ::core::ffi::c_int || yanklen_0 == 0 as ::core::ffi::c_int
                        {
                            if VIsual_active.get() {
                                lnum = end_lnum;
                            }
                        } else if count > INT_MAX / yanklen_0 {
                            emsg(gettext(
                                &raw const e_resulting_text_too_long as *const ::core::ffi::c_char,
                            ));
                        } else {
                            totlen = (count as size_t).wrapping_mul(yanklen_0 as size_t);
                            loop {
                                let mut oldp_0: *mut ::core::ffi::c_char = ml_get(lnum);
                                let mut oldlen_0: colnr_T = ml_get_len(lnum);
                                if lnum > start_lnum {
                                    let mut pos_0: pos_T = pos_T {
                                        lnum: lnum,
                                        col: 0,
                                        coladd: 0,
                                    };
                                    if getvpos(curwin.get(), &raw mut pos_0, vcol) {
                                        col = pos_0.col;
                                    } else {
                                        col = MAXCOL as ::core::ffi::c_int as colnr_T;
                                    }
                                }
                                if VIsual_active.get() as ::core::ffi::c_int != 0 && col > oldlen_0
                                {
                                    lnum += 1;
                                } else {
                                    let mut newp_0: *mut ::core::ffi::c_char = xmalloc(
                                        totlen
                                            .wrapping_add(oldlen_0 as size_t)
                                            .wrapping_add(1 as size_t),
                                    )
                                        as *mut ::core::ffi::c_char;
                                    memmove(
                                        newp_0 as *mut ::core::ffi::c_void,
                                        oldp_0 as *const ::core::ffi::c_void,
                                        col as size_t,
                                    );
                                    let mut ptr_2: *mut ::core::ffi::c_char =
                                        newp_0.offset(col as isize);
                                    let mut i_0: size_t = 0 as size_t;
                                    while i_0 < count as size_t {
                                        memmove(
                                            ptr_2 as *mut ::core::ffi::c_void,
                                            (*y_array.offset(0 as ::core::ffi::c_int as isize)).data
                                                as *const ::core::ffi::c_void,
                                            yanklen_0 as size_t,
                                        );
                                        ptr_2 = ptr_2.offset(yanklen_0 as isize);
                                        i_0 = i_0.wrapping_add(1);
                                    }
                                    memmove(
                                        ptr_2 as *mut ::core::ffi::c_void,
                                        oldp_0.offset(col as isize) as *const ::core::ffi::c_void,
                                        ((oldlen_0 - col) as size_t).wrapping_add(1 as size_t),
                                    );
                                    ml_replace(lnum, newp_0, false_0 != 0);
                                    first_byte_off = utf_head_off(
                                        newp_0,
                                        ptr_2.offset(-(1 as ::core::ffi::c_int as isize)),
                                    );
                                    if lnum == (*curwin.get()).w_cursor.lnum {
                                        changed_cline_bef_curs(curwin.get());
                                        invalidate_botline_win(curwin.get());
                                        (*curwin.get()).w_cursor.col +=
                                            totlen.wrapping_sub(1 as size_t) as colnr_T;
                                    }
                                    changed_bytes(lnum, col);
                                    extmark_splice_cols(
                                        curbuf.get(),
                                        lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                        col,
                                        0 as colnr_T,
                                        totlen as colnr_T,
                                        kExtmarkUndo,
                                    );
                                    if VIsual_active.get() {
                                        lnum += 1;
                                    }
                                }
                                if !(VIsual_active.get() as ::core::ffi::c_int != 0
                                    && lnum <= end_lnum)
                                {
                                    break;
                                }
                            }
                            if VIsual_active.get() {
                                lnum -= 1;
                            }
                        }
                        (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                        (*curbuf.get()).b_op_end.col -= first_byte_off;
                        if totlen != 0
                            && (restart_edit.get() != 0 as ::core::ffi::c_int
                                || flags & PUT_CURSEND as ::core::ffi::c_int != 0)
                        {
                            (*curwin.get()).w_cursor.col += 1;
                        } else {
                            (*curwin.get()).w_cursor.col -= first_byte_off;
                        }
                    } else {
                        let mut new_lnum: linenr_T = new_cursor.lnum;
                        let mut indent: ::core::ffi::c_int = 0;
                        let mut orig_indent: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut indent_diff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut first_indent: bool = true_0 != 0;
                        let mut lendiff: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if flags & PUT_FIXINDENT as ::core::ffi::c_int != 0 {
                            orig_indent = get_indent();
                        }
                        let mut cnt: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                        '_error: while cnt <= count {
                            let mut i_1: size_t = 0 as size_t;
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                                lnum = new_cursor.lnum;
                                let mut ptr_3: *mut ::core::ffi::c_char =
                                    ml_get(lnum).offset(col as isize);
                                let mut ptrlen_0: size_t =
                                    (ml_get_len(lnum) as size_t).wrapping_sub(col as size_t);
                                totlen = (*y_array
                                    .offset(y_size.wrapping_sub(1 as size_t) as isize))
                                .size;
                                let mut newp_1: *mut ::core::ffi::c_char = xmalloc(
                                    ptrlen_0.wrapping_add(totlen).wrapping_add(1 as size_t),
                                )
                                    as *mut ::core::ffi::c_char;
                                strcpy(
                                    newp_1,
                                    (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                        .data,
                                );
                                strcpy(newp_1.offset(totlen as isize), ptr_3);
                                ml_append(lnum, newp_1, 0 as colnr_T, false_0 != 0);
                                new_lnum += 1;
                                xfree(newp_1 as *mut ::core::ffi::c_void);
                                let mut oldp_1: *mut ::core::ffi::c_char = ml_get(lnum);
                                newp_1 = xmalloc(
                                    (col as size_t)
                                        .wrapping_add(yanklen_0 as size_t)
                                        .wrapping_add(1 as size_t),
                                )
                                    as *mut ::core::ffi::c_char;
                                memmove(
                                    newp_1 as *mut ::core::ffi::c_void,
                                    oldp_1 as *const ::core::ffi::c_void,
                                    col as size_t,
                                );
                                memmove(
                                    newp_1.offset(col as isize) as *mut ::core::ffi::c_void,
                                    (*y_array.offset(0 as ::core::ffi::c_int as isize)).data
                                        as *const ::core::ffi::c_void,
                                    (yanklen_0 as size_t).wrapping_add(1 as size_t),
                                );
                                ml_replace(lnum, newp_1, false_0 != 0);
                                (*curwin.get()).w_cursor.lnum = lnum;
                                i_1 = 1 as size_t;
                            }
                            while i_1 < y_size {
                                if y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int
                                    || i_1 < y_size.wrapping_sub(1 as size_t)
                                {
                                    if ml_append(
                                        lnum,
                                        (*y_array.offset(i_1 as isize)).data,
                                        0 as colnr_T,
                                        false_0 != 0,
                                    ) == FAIL
                                    {
                                        break '_error;
                                    }
                                    new_lnum += 1;
                                }
                                lnum += 1;
                                nr_lines += 1;
                                if flags & PUT_FIXINDENT as ::core::ffi::c_int != 0 {
                                    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
                                    (*curwin.get()).w_cursor.lnum = lnum;
                                    let mut ptr_4: *mut ::core::ffi::c_char = ml_get(lnum);
                                    if cnt == count && i_1 == y_size.wrapping_sub(1 as size_t) {
                                        lendiff = ml_get_len(lnum) as ::core::ffi::c_int;
                                    }
                                    if *ptr_4 as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                                        && preprocs_left() as ::core::ffi::c_int != 0
                                    {
                                        indent = 0 as ::core::ffi::c_int;
                                    } else if *ptr_4 as ::core::ffi::c_int == NUL {
                                        indent = 0 as ::core::ffi::c_int;
                                    } else if first_indent {
                                        indent_diff = orig_indent - get_indent();
                                        indent = orig_indent;
                                        first_indent = false_0 != 0;
                                    } else {
                                        indent = get_indent() + indent_diff;
                                        if indent < 0 as ::core::ffi::c_int {
                                            indent = 0 as ::core::ffi::c_int;
                                        }
                                    }
                                    set_indent(indent, SIN_NOMARK as ::core::ffi::c_int);
                                    (*curwin.get()).w_cursor = old_pos;
                                    if cnt == count && i_1 == y_size.wrapping_sub(1 as size_t) {
                                        lendiff -= ml_get_len(lnum) as ::core::ffi::c_int;
                                    }
                                }
                                i_1 = i_1.wrapping_add(1);
                            }
                            let mut totsize: bcount_t = 0 as bcount_t;
                            let mut lastsize: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
                                || y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                                    && flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0
                            {
                                i_1 = 0 as size_t;
                                while i_1 < y_size.wrapping_sub(1 as size_t) {
                                    totsize += (*y_array.offset(i_1 as isize)).size as bcount_t
                                        + 1 as bcount_t;
                                    i_1 = i_1.wrapping_add(1);
                                }
                                lastsize = (*y_array
                                    .offset(y_size.wrapping_sub(1 as size_t) as isize))
                                .size
                                    as ::core::ffi::c_int;
                                totsize += lastsize as bcount_t;
                            }
                            if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                                extmark_splice(
                                    curbuf.get(),
                                    new_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    col,
                                    0 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    0 as bcount_t,
                                    y_size as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    lastsize as colnr_T,
                                    totsize,
                                    kExtmarkUndo,
                                );
                            } else if y_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                                && flags & PUT_LINE_SPLIT as ::core::ffi::c_int != 0
                            {
                                extmark_splice(
                                    curbuf.get(),
                                    new_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                                    split_pos,
                                    0 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    0 as bcount_t,
                                    y_size as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                                    0 as colnr_T,
                                    totsize + 2 as bcount_t,
                                    kExtmarkUndo,
                                );
                            }
                            if cnt == 1 as ::core::ffi::c_int {
                                new_lnum = lnum;
                            }
                            cnt += 1;
                        }
                        if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                            (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                            if dir == FORWARD as ::core::ffi::c_int {
                                (*curbuf.get()).b_op_start.lnum += 1;
                            }
                        }
                        let mut kind: ExtmarkOp = (if y_type as ::core::ffi::c_int
                            == kMTLineWise as ::core::ffi::c_int
                            && flags & PUT_LINE_SPLIT as ::core::ffi::c_int == 0
                        {
                            kExtmarkUndo as ::core::ffi::c_int
                        } else {
                            kExtmarkNOOP as ::core::ffi::c_int
                        }) as ExtmarkOp;
                        mark_adjust(
                            (*curbuf.get()).b_op_start.lnum
                                + (y_type as ::core::ffi::c_int
                                    == kMTCharWise as ::core::ffi::c_int)
                                    as ::core::ffi::c_int,
                            MAXLNUM as ::core::ffi::c_int as linenr_T,
                            nr_lines,
                            0 as linenr_T,
                            kind,
                        );
                        if y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int {
                            changed_lines(
                                curbuf.get(),
                                (*curwin.get()).w_cursor.lnum,
                                col,
                                (*curwin.get()).w_cursor.lnum + 1 as linenr_T,
                                nr_lines,
                                true_0 != 0,
                            );
                        } else {
                            changed_lines(
                                curbuf.get(),
                                (*curbuf.get()).b_op_start.lnum,
                                0 as colnr_T,
                                (*curbuf.get()).b_op_start.lnum,
                                nr_lines,
                                true_0 != 0,
                            );
                        }
                        (*curbuf.get()).b_op_end.lnum = new_lnum;
                        col = (if 0 as ::core::ffi::c_int
                            > (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size
                                as ::core::ffi::c_int
                                - lendiff
                        {
                            0 as ::core::ffi::c_int
                        } else {
                            (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size
                                as ::core::ffi::c_int
                                - lendiff
                        }) as colnr_T;
                        if col > 1 as ::core::ffi::c_int {
                            (*curbuf.get()).b_op_end.col =
                                (col as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
                            if (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize)).size
                                > 0 as size_t
                            {
                                (*curbuf.get()).b_op_end.col -= utf_head_off(
                                    (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                        .data,
                                    (*y_array.offset(y_size.wrapping_sub(1 as size_t) as isize))
                                        .data
                                        .offset(
                                            (*y_array
                                                .offset(y_size.wrapping_sub(1 as size_t) as isize))
                                            .size
                                                as isize,
                                        )
                                        .offset(-(1 as ::core::ffi::c_int as isize)),
                                );
                            }
                        } else {
                            (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
                        }
                        if flags & PUT_CURSLINE as ::core::ffi::c_int != 0 {
                            (*curwin.get()).w_cursor.lnum = lnum;
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                        } else if flags & PUT_CURSEND as ::core::ffi::c_int != 0 {
                            if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int {
                                if lnum >= (*curbuf.get()).b_ml.ml_line_count {
                                    (*curwin.get()).w_cursor.lnum =
                                        (*curbuf.get()).b_ml.ml_line_count;
                                } else {
                                    (*curwin.get()).w_cursor.lnum = lnum + 1 as linenr_T;
                                }
                                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            } else {
                                (*curwin.get()).w_cursor.lnum = new_lnum;
                                (*curwin.get()).w_cursor.col = col;
                                (*curbuf.get()).b_op_end = (*curwin.get()).w_cursor;
                                if col > 1 as ::core::ffi::c_int {
                                    (*curbuf.get()).b_op_end.col = (col as ::core::ffi::c_int
                                        - 1 as ::core::ffi::c_int)
                                        as colnr_T;
                                }
                            }
                        } else if y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
                        {
                            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                            if dir == FORWARD as ::core::ffi::c_int {
                                (*curwin.get()).w_cursor.lnum += 1;
                            }
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                        } else {
                            (*curwin.get()).w_cursor = new_cursor;
                        }
                    }
                }
                msgmore(nr_lines as ::core::ffi::c_int);
                (*curwin.get()).w_set_curswant = true_0;
                len_0 = get_cursor_line_len();
                if (*curwin.get()).w_cursor.col > len_0 {
                    if cur_ve_flags == kOptVeFlagAll as ::core::ffi::c_int as ::core::ffi::c_uint {
                        (*curwin.get()).w_cursor.coladd =
                            ((*curwin.get()).w_cursor.col as ::core::ffi::c_int - len_0) as colnr_T;
                    }
                    (*curwin.get()).w_cursor.col = len_0 as colnr_T;
                }
            }
        }
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        (*curbuf.get()).b_op_start = orig_start;
        (*curbuf.get()).b_op_end = orig_end;
    }
    if allocated {
        xfree(insert_string.data as *mut ::core::ffi::c_void);
    }
    if regname == '=' as ::core::ffi::c_int {
        xfree(y_array as *mut ::core::ffi::c_void);
    }
    if (*curbuf.get()).terminal.is_null() {
        VIsual_active.set(false_0 != 0);
    }
    adjust_cursor_eol();
}
unsafe extern "C" fn dis_msg(mut p: *const ::core::ffi::c_char, mut skip_esc: bool) {
    let mut n: ::core::ffi::c_int = Columns.get() - 6 as ::core::ffi::c_int;
    while *p as ::core::ffi::c_int != NUL
        && !(*p as ::core::ffi::c_int == ESC
            && skip_esc as ::core::ffi::c_int != 0
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        && {
            n -= ptr2cells(p);
            n >= 0 as ::core::ffi::c_int
        }
    {
        let mut l: ::core::ffi::c_int = 0;
        l = utfc_ptr2len(p);
        if l > 1 as ::core::ffi::c_int {
            msg_outtrans_len(p, l, 0 as ::core::ffi::c_int, false_0 != 0);
            p = p.offset(l as isize);
        } else {
            let c2rust_fresh4 = p;
            p = p.offset(1);
            msg_outtrans_len(
                c2rust_fresh4,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                false_0 != 0,
            );
        }
    }
    os_breakcheck();
}
pub unsafe fn ex_display(mut eap: *mut exarg_T) {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut yb: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut type_0: ::core::ffi::c_int = 0;
    if !arg.is_null() && *arg as ::core::ffi::c_int == NUL {
        arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut hl_id: ::core::ffi::c_int = HLF_8;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    msg_ext_skip_flush.set(true_0 != 0);
    msg_puts_title(gettext(
        b"\nType Name Content\0".as_ptr() as *const ::core::ffi::c_char
    ));
    let mut i: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    while i < NUM_REGISTERS as ::core::ffi::c_int && !got_int.get() {
        let mut name: ::core::ffi::c_int = get_register_name(i);
        if !(!arg.is_null() && vim_strchr(arg, name).is_null()) {
            match get_reg_type(name, ::core::ptr::null_mut::<colnr_T>()) as ::core::ffi::c_int {
                1 => {
                    type_0 = 'l' as ::core::ffi::c_int;
                }
                0 => {
                    type_0 = 'c' as ::core::ffi::c_int;
                }
                _ => {
                    type_0 = 'b' as ::core::ffi::c_int;
                }
            }
            if i == -1 as ::core::ffi::c_int {
                if !(*y_previous.ptr()).is_null() {
                    yb = y_previous.get();
                } else {
                    yb = (y_regs.ptr() as *mut yankreg_T).offset(0 as ::core::ffi::c_int as isize);
                }
            } else {
                yb = (y_regs.ptr() as *mut yankreg_T).offset(i as isize);
            }
            clipboard::get_clipboard(name, &mut yb, true);
            if !(name == mb_tolower(redir_reg.get())
                || redir_reg.get() == '"' as ::core::ffi::c_int && yb == y_previous.get())
            {
                if !(*yb).y_array.is_null() {
                    let mut do_show: bool = false_0 != 0;
                    let mut j: size_t = 0 as size_t;
                    while !do_show && j < (*yb).y_size {
                        do_show = !message_filtered((*(*yb).y_array.offset(j as isize)).data);
                        j = j.wrapping_add(1);
                    }
                    if do_show as ::core::ffi::c_int != 0 || (*yb).y_size == 0 as size_t {
                        msg_putchar('\n' as ::core::ffi::c_int);
                        msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                        msg_putchar(type_0);
                        msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                        msg_putchar('"' as ::core::ffi::c_int);
                        msg_putchar(name);
                        msg_puts(b"   \0".as_ptr() as *const ::core::ffi::c_char);
                        let mut n: ::core::ffi::c_int = Columns.get() - 11 as ::core::ffi::c_int;
                        let mut j_0: size_t = 0 as size_t;
                        while j_0 < (*yb).y_size && n > 1 as ::core::ffi::c_int {
                            if j_0 != 0 {
                                msg_puts_hl(
                                    b"^J\0".as_ptr() as *const ::core::ffi::c_char,
                                    hl_id,
                                    false_0 != 0,
                                );
                                n -= 2 as ::core::ffi::c_int;
                            }
                            p = (*(*yb).y_array.offset(j_0 as isize)).data;
                            while *p as ::core::ffi::c_int != NUL && {
                                n -= ptr2cells(p);
                                n >= 0 as ::core::ffi::c_int
                            } {
                                let mut clen: ::core::ffi::c_int = utfc_ptr2len(p);
                                msg_outtrans_len(p, clen, 0 as ::core::ffi::c_int, false_0 != 0);
                                p = p.offset((clen - 1 as ::core::ffi::c_int) as isize);
                                p = p.offset(1);
                            }
                            j_0 = j_0.wrapping_add(1);
                        }
                        if n > 1 as ::core::ffi::c_int
                            && (*yb).y_type as ::core::ffi::c_int
                                == kMTLineWise as ::core::ffi::c_int
                        {
                            msg_puts_hl(
                                b"^J\0".as_ptr() as *const ::core::ffi::c_char,
                                hl_id,
                                false_0 != 0,
                            );
                        }
                    }
                    os_breakcheck();
                }
            }
        }
        i += 1;
    }
    let mut insert: String_0 = get_last_insert();
    p = insert.data;
    if !p.is_null()
        && (arg.is_null() || !vim_strchr(arg, '.' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(p)
    {
        msg_puts(b"\n  c  \".   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(p, true_0 != 0);
    }
    if !(*last_cmdline.ptr()).is_null()
        && (arg.is_null() || !vim_strchr(arg, ':' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(last_cmdline.get())
    {
        msg_puts(b"\n  c  \":   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(last_cmdline.get(), false_0 != 0);
    }
    if !(*curbuf.get()).b_fname.is_null()
        && (arg.is_null() || !vim_strchr(arg, '%' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered((*curbuf.get()).b_fname)
    {
        msg_puts(b"\n  c  \"%   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg((*curbuf.get()).b_fname, false_0 != 0);
    }
    if (arg.is_null() || !vim_strchr(arg, '%' as ::core::ffi::c_int).is_null()) && !got_int.get() {
        let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dummy: linenr_T = 0;
        if buflist_name_nr(0 as ::core::ffi::c_int, &raw mut fname, &raw mut dummy) != FAIL
            && !message_filtered(fname)
        {
            msg_puts(b"\n  c  \"#   \0".as_ptr() as *const ::core::ffi::c_char);
            dis_msg(fname, false_0 != 0);
        }
    }
    if !last_search_pat().is_null()
        && (arg.is_null() || !vim_strchr(arg, '/' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(last_search_pat())
    {
        msg_puts(b"\n  c  \"/   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(last_search_pat(), false_0 != 0);
    }
    if !(*expr_line.ptr()).is_null()
        && (arg.is_null() || !vim_strchr(arg, '=' as ::core::ffi::c_int).is_null())
        && !got_int.get()
        && !message_filtered(expr_line.get())
    {
        msg_puts(b"\n  c  \"=   \0".as_ptr() as *const ::core::ffi::c_char);
        dis_msg(expr_line.get(), false_0 != 0);
    }
    msg_ext_skip_flush.set(false_0 != 0);
}
pub unsafe extern "C" fn get_reg_type(
    mut regname: ::core::ffi::c_int,
    mut reg_width: *mut colnr_T,
) -> MotionType {
    's_19: {
        'c_46756: {
            'c_46754: {
                'c_46752: {
                    'c_46750: {
                        'c_46748: {
                            'c_46746: {
                                'c_46744: {
                                    'c_46742: {
                                        match regname {
                                            35 => {}
                                            61 => {}
                                            58 => {
                                                break 'c_46742;
                                            }
                                            47 => {
                                                break 'c_46744;
                                            }
                                            46 => {
                                                break 'c_46746;
                                            }
                                            Ctrl_F => {
                                                break 'c_46748;
                                            }
                                            Ctrl_P => {
                                                break 'c_46750;
                                            }
                                            Ctrl_W => {
                                                break 'c_46752;
                                            }
                                            Ctrl_A => {
                                                break 'c_46754;
                                            }
                                            37 | 95 => {
                                                break 'c_46756;
                                            }
                                            _ => {
                                                break 's_19;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return kMTCharWise;
    }
    if regname != NUL && !valid_yank_reg(regname, false_0 != 0) {
        return kMTUnknown;
    }
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PASTE as ::core::ffi::c_int);
    if !(*reg).y_array.is_null() {
        if !reg_width.is_null()
            && (*reg).y_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int
        {
            *reg_width = (*reg).y_width;
        }
        return (*reg).y_type;
    }
    return kMTUnknown;
}
unsafe extern "C" fn get_reg_wrap_one_line(
    mut s: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if flags & kGRegList as ::core::ffi::c_int == 0 {
        return s as *mut ::core::ffi::c_void;
    }
    let list: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
    tv_list_append_allocated_string(list, s);
    return list as *mut ::core::ffi::c_void;
}
pub unsafe extern "C" fn get_reg_contents(
    mut regname: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_void {
    if regname == '=' as ::core::ffi::c_int {
        if flags & kGRegNoExpr as ::core::ffi::c_int != 0 {
            return NULL_0;
        }
        if flags & kGRegExprSrc as ::core::ffi::c_int != 0 {
            return get_reg_wrap_one_line(get_expr_line_src(), flags);
        }
        return get_reg_wrap_one_line(get_expr_line(), flags);
    }
    if regname == '@' as ::core::ffi::c_int {
        regname = '"' as ::core::ffi::c_int;
    }
    if regname != NUL && !valid_yank_reg(regname, false_0 != 0) {
        return NULL_0;
    }
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut allocated: bool = false;
    if get_spec_reg(regname, &raw mut retval, &raw mut allocated, false_0 != 0) {
        if retval.is_null() {
            return NULL_0;
        }
        if allocated {
            return get_reg_wrap_one_line(retval, flags);
        }
        return get_reg_wrap_one_line(xstrdup(retval), flags);
    }
    let mut reg: *mut yankreg_T = get_yank_register(regname, YREG_PUT as ::core::ffi::c_int);
    if (*reg).y_array.is_null() {
        return NULL_0;
    }
    if flags & kGRegList as ::core::ffi::c_int != 0 {
        let list: *mut list_T = tv_list_alloc((*reg).y_size as ptrdiff_t);
        let mut i: size_t = 0 as size_t;
        while i < (*reg).y_size {
            tv_list_append_string(
                list,
                (*(*reg).y_array.offset(i as isize)).data,
                (*(*reg).y_array.offset(i as isize)).size as ::core::ffi::c_int as ssize_t,
            );
            i = i.wrapping_add(1);
        }
        return list as *mut ::core::ffi::c_void;
    }
    let mut len: size_t = 0 as size_t;
    let mut i_0: size_t = 0 as size_t;
    while i_0 < (*reg).y_size {
        len = len.wrapping_add((*(*reg).y_array.offset(i_0 as isize)).size);
        if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            || i_0 < (*reg).y_size.wrapping_sub(1 as size_t)
        {
            len = len.wrapping_add(1);
        }
        i_0 = i_0.wrapping_add(1);
    }
    retval = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    len = 0 as size_t;
    let mut i_1: size_t = 0 as size_t;
    while i_1 < (*reg).y_size {
        strcpy(
            retval.offset(len as isize),
            (*(*reg).y_array.offset(i_1 as isize)).data,
        );
        len = len.wrapping_add((*(*reg).y_array.offset(i_1 as isize)).size);
        if (*reg).y_type as ::core::ffi::c_int == kMTLineWise as ::core::ffi::c_int
            || i_1 < (*reg).y_size.wrapping_sub(1 as size_t)
        {
            let c2rust_fresh5 = len;
            len = len.wrapping_add(1);
            *retval.offset(c2rust_fresh5 as isize) = '\n' as ::core::ffi::c_char;
        }
        i_1 = i_1.wrapping_add(1);
    }
    *retval.offset(len as isize) = NUL as ::core::ffi::c_char;
    return retval as *mut ::core::ffi::c_void;
}
unsafe extern "C" fn init_write_reg(
    mut name: ::core::ffi::c_int,
    mut old_y_previous: *mut *mut yankreg_T,
    mut must_append: bool,
) -> *mut yankreg_T {
    if !valid_yank_reg(name, true_0 != 0) {
        emsg_invreg(name);
        return ::core::ptr::null_mut::<yankreg_T>();
    }
    *old_y_previous = y_previous.get();
    let mut reg: *mut yankreg_T = get_yank_register(name, YREG_YANK as ::core::ffi::c_int);
    if !is_append_register(name) && !must_append {
        free_register(reg);
    }
    return reg;
}
unsafe extern "C" fn str_to_reg(
    mut y_ptr: *mut yankreg_T,
    mut yank_type: MotionType,
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    mut blocklen: colnr_T,
    mut str_list: bool,
) {
    if (*y_ptr).y_array.is_null() {
        (*y_ptr).y_size = 0 as size_t;
    }
    if yank_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
        yank_type = (if str_list as ::core::ffi::c_int != 0
            || len > 0 as size_t
                && (*str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int == NL
                    || *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                        == CAR)
        {
            kMTLineWise as ::core::ffi::c_int
        } else {
            kMTCharWise as ::core::ffi::c_int
        }) as MotionType;
    }
    let mut newlines: size_t = 0 as size_t;
    let mut extraline: bool = false_0 != 0;
    let mut append: bool = false_0 != 0;
    if str_list {
        let mut ss: *mut *mut ::core::ffi::c_char = str as *mut *mut ::core::ffi::c_char;
        while !(*ss).is_null() {
            newlines = newlines.wrapping_add(1);
            ss = ss.offset(1);
        }
    } else {
        newlines = memcnt(
            str as *const ::core::ffi::c_void,
            '\n' as ::core::ffi::c_char,
            len,
        );
        if yank_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            || len == 0 as size_t
            || *str.offset(len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                != '\n' as ::core::ffi::c_int
        {
            extraline = true;
            newlines = newlines.wrapping_add(1);
        }
        if (*y_ptr).y_size > 0 as size_t
            && (*y_ptr).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
        {
            append = true_0 != 0;
            newlines = newlines.wrapping_sub(1);
        }
    }
    if (*y_ptr).y_size.wrapping_add(newlines) == 0 as size_t {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*y_ptr).y_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        return;
    }
    let mut pp: *mut String_0 = xrealloc(
        (*y_ptr).y_array as *mut ::core::ffi::c_void,
        (*y_ptr)
            .y_size
            .wrapping_add(newlines)
            .wrapping_mul(::core::mem::size_of::<String_0>()),
    ) as *mut String_0;
    (*y_ptr).y_array = pp;
    let mut lnum: size_t = (*y_ptr).y_size;
    let mut maxlen: size_t = 0 as size_t;
    if str_list {
        let mut ss_0: *mut *mut ::core::ffi::c_char = str as *mut *mut ::core::ffi::c_char;
        while !(*ss_0).is_null() {
            *pp.offset(lnum as isize) = cstr_to_string(*ss_0);
            if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                let mut charlen: size_t = mb_string2cells(*ss_0);
                maxlen = if maxlen > charlen { maxlen } else { charlen };
            }
            ss_0 = ss_0.offset(1);
            lnum = lnum.wrapping_add(1);
        }
    } else {
        let mut line_len: size_t = 0;
        let mut start: *const ::core::ffi::c_char = str;
        let mut end: *const ::core::ffi::c_char = str.offset(len as isize);
        while start < end.offset(extraline as ::core::ffi::c_int as isize) {
            let mut charlen_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut line_end: *const ::core::ffi::c_char = start;
            while line_end < end {
                if *line_end as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                    break;
                }
                if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
                    charlen_0 += utf_ptr2cells_len(
                        line_end,
                        end.offset_from(line_end) as ::core::ffi::c_int,
                    );
                }
                if *line_end as ::core::ffi::c_int == NUL {
                    line_end = line_end.offset(1);
                } else {
                    line_end = line_end.offset(utf_ptr2len_len(
                        line_end,
                        end.offset_from(line_end) as ::core::ffi::c_int,
                    ) as isize);
                }
            }
            '_c2rust_label: {
                if line_end.offset_from(start) >= 0 as isize {
                } else {
                    __assert_fail(
                        b"line_end - start >= 0\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/nvim/register.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        2491 as ::core::ffi::c_uint,
                        b"void str_to_reg(yankreg_T *, MotionType, const char *, size_t, colnr_T, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            line_len = line_end.offset_from(start) as size_t;
            maxlen = if maxlen > charlen_0 as size_t {
                maxlen
            } else {
                charlen_0 as size_t
            };
            let mut extra: size_t = if append as ::core::ffi::c_int != 0 {
                lnum = lnum.wrapping_sub(1);
                (*pp.offset(lnum as isize)).size
            } else {
                0 as size_t
            };
            let mut s: *mut ::core::ffi::c_char =
                xmallocz(line_len.wrapping_add(extra)) as *mut ::core::ffi::c_char;
            if extra > 0 as size_t {
                memcpy(
                    s as *mut ::core::ffi::c_void,
                    (*pp.offset(lnum as isize)).data as *const ::core::ffi::c_void,
                    extra,
                );
            }
            if line_len > 0 as size_t {
                memcpy(
                    s.offset(extra as isize) as *mut ::core::ffi::c_void,
                    start as *const ::core::ffi::c_void,
                    line_len,
                );
            }
            let mut s_len: size_t = extra.wrapping_add(line_len);
            if append {
                xfree((*pp.offset(lnum as isize)).data as *mut ::core::ffi::c_void);
                append = false_0 != 0;
            }
            *pp.offset(lnum as isize) = String_0 {
                data: s,
                size: s_len,
            };
            memchrsub(
                (*pp.offset(lnum as isize)).data as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                '\n' as ::core::ffi::c_char,
                s_len,
            );
            start = start.offset(line_len.wrapping_add(1 as size_t) as isize);
            lnum = lnum.wrapping_add(1);
        }
    }
    (*y_ptr).y_type = yank_type;
    (*y_ptr).y_size = lnum;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*y_ptr).additional_data as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL_0;
    let _ = *ptr__0;
    (*y_ptr).timestamp = os_time();
    if yank_type as ::core::ffi::c_int == kMTBlockWise as ::core::ffi::c_int {
        (*y_ptr).y_width = (if blocklen == -1 as ::core::ffi::c_int {
            maxlen as ::core::ffi::c_int - 1 as ::core::ffi::c_int
        } else {
            blocklen as ::core::ffi::c_int
        }) as colnr_T;
    } else {
        (*y_ptr).y_width = 0 as ::core::ffi::c_int as colnr_T;
    };
}
unsafe extern "C" fn finish_write_reg(
    mut name: ::core::ffi::c_int,
    mut reg: *mut yankreg_T,
    mut old_y_previous: *mut yankreg_T,
) {
    clipboard::set_clipboard(name, reg);
    if name != '"' as ::core::ffi::c_int {
        y_previous.set(old_y_previous);
    }
}
pub unsafe extern "C" fn write_reg_contents(
    mut name: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut len: ssize_t,
    mut must_append: ::core::ffi::c_int,
) {
    write_reg_contents_ex(name, str, len, must_append != 0, kMTUnknown, 0 as colnr_T);
}
pub unsafe extern "C" fn write_reg_contents_lst(
    mut name: ::core::ffi::c_int,
    mut strings: *mut *mut ::core::ffi::c_char,
    mut must_append: bool,
    mut yank_type: MotionType,
    mut block_len: colnr_T,
) {
    if name == '/' as ::core::ffi::c_int || name == '=' as ::core::ffi::c_int {
        let mut s: *mut ::core::ffi::c_char = *strings.offset(0 as ::core::ffi::c_int as isize);
        if (*strings.offset(0 as ::core::ffi::c_int as isize)).is_null() {
            s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if !(*strings.offset(1 as ::core::ffi::c_int as isize)).is_null() {
            emsg(gettext(
                (e_search_pattern_and_expression_register_may_not_contain_two_or_more_lines.ptr()
                    as *const _) as *const ::core::ffi::c_char,
            ));
            return;
        }
        write_reg_contents_ex(name, s, -1 as ssize_t, must_append, yank_type, block_len);
        return;
    }
    if name == '_' as ::core::ffi::c_int {
        return;
    }
    let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    reg = init_write_reg(name, &raw mut old_y_previous, must_append);
    if reg.is_null() {
        return;
    }
    str_to_reg(
        reg,
        yank_type,
        strings as *mut ::core::ffi::c_char,
        strlen(strings as *mut ::core::ffi::c_char),
        block_len,
        true_0 != 0,
    );
    finish_write_reg(name, reg, old_y_previous);
}
pub unsafe extern "C" fn write_reg_contents_ex(
    mut name: ::core::ffi::c_int,
    mut str: *const ::core::ffi::c_char,
    mut len: ssize_t,
    mut must_append: bool,
    mut yank_type: MotionType,
    mut block_len: colnr_T,
) {
    if len < 0 as ssize_t {
        len = strlen(str) as ssize_t;
    }
    if name == '/' as ::core::ffi::c_int {
        set_last_search_pat(str, RE_SEARCH as ::core::ffi::c_int, true, true);
        return;
    }
    if name == '#' as ::core::ffi::c_int {
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if ascii_isdigit(*str as ::core::ffi::c_int) {
            let mut num: ::core::ffi::c_int = atoi(str);
            buf = buflist_findnr(num);
            if buf.is_null() {
                semsg(
                    gettext(&raw const e_nobufnr as *const ::core::ffi::c_char),
                    num as int64_t,
                );
            }
        } else {
            buf = buflist_findnr(buflist_findpat(
                str,
                str.offset(len as isize),
                true_0 != 0,
                false_0 != 0,
                false_0 != 0,
            ));
        }
        if buf.is_null() {
            return;
        }
        (*curwin.get()).w_alt_fnum = (*buf).handle as ::core::ffi::c_int;
        return;
    }
    if name == '=' as ::core::ffi::c_int {
        let mut offset: size_t = 0 as size_t;
        let mut totlen: size_t = len as size_t;
        if must_append as ::core::ffi::c_int != 0 && !(*expr_line.ptr()).is_null() {
            let mut exprlen: size_t = strlen(expr_line.get());
            totlen = totlen.wrapping_add(exprlen);
            offset = exprlen;
        }
        expr_line.set(xrealloc(
            expr_line.get() as *mut ::core::ffi::c_void,
            totlen.wrapping_add(1 as size_t),
        ) as *mut ::core::ffi::c_char);
        memcpy(
            (*expr_line.ptr()).offset(offset as isize) as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len as size_t,
        );
        *(*expr_line.ptr()).offset(totlen as isize) = NUL as ::core::ffi::c_char;
        return;
    }
    if name == '_' as ::core::ffi::c_int {
        return;
    }
    let mut old_y_previous: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    let mut reg: *mut yankreg_T = ::core::ptr::null_mut::<yankreg_T>();
    reg = init_write_reg(name, &raw mut old_y_previous, must_append);
    if reg.is_null() {
        return;
    }
    str_to_reg(reg, yank_type, str, len as size_t, block_len, false_0 != 0);
    finish_write_reg(name, reg, old_y_previous);
}
pub unsafe extern "C" fn prepare_yankreg_from_object(
    mut reg: *mut yankreg_T,
    mut regtype: String_0,
    mut _lines: size_t,
) -> bool {
    let mut type_0: ::core::ffi::c_char = (if !regtype.data.is_null() {
        *regtype.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    } else {
        NUL
    }) as ::core::ffi::c_char;
    match type_0 as ::core::ffi::c_int {
        0 => {
            (*reg).y_type = kMTUnknown;
        }
        118 | 99 => {
            (*reg).y_type = kMTCharWise;
        }
        86 | 108 => {
            (*reg).y_type = kMTLineWise;
        }
        98 | Ctrl_V => {
            (*reg).y_type = kMTBlockWise;
        }
        _ => return false_0 != 0,
    }
    (*reg).y_width = 0 as ::core::ffi::c_int as colnr_T;
    if regtype.size > 1 as size_t {
        if (*reg).y_type as ::core::ffi::c_int != kMTBlockWise as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if !ascii_isdigit(
            *regtype.data.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        ) {
            return false_0 != 0;
        }
        let mut p: *const ::core::ffi::c_char =
            regtype.data.offset(1 as ::core::ffi::c_int as isize);
        (*reg).y_width = (getdigits_int(
            &raw mut p as *mut *mut ::core::ffi::c_char,
            false_0 != 0,
            1 as ::core::ffi::c_int,
        ) - 1 as ::core::ffi::c_int) as colnr_T;
        if regtype.size > p.offset_from(regtype.data) as size_t {
            return false_0 != 0;
        }
    }
    (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
    (*reg).timestamp = 0 as Timestamp;
    return true_0 != 0;
}
pub unsafe extern "C" fn finish_yankreg_from_object(
    mut reg: *mut yankreg_T,
    mut clipboard_adjust: bool,
) {
    if (*reg).y_size > 0 as size_t
        && (*(*reg)
            .y_array
            .offset((*reg).y_size.wrapping_sub(1 as size_t) as isize))
        .size
            == 0 as size_t
    {
        if (*reg).y_type as ::core::ffi::c_int != kMTCharWise as ::core::ffi::c_int {
            if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int
                || clipboard_adjust as ::core::ffi::c_int != 0
            {
                (*reg).y_size = (*reg).y_size.wrapping_sub(1);
            }
            if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
                (*reg).y_type = kMTLineWise;
            }
        }
    } else if (*reg).y_type as ::core::ffi::c_int == kMTUnknown as ::core::ffi::c_int {
        (*reg).y_type = kMTCharWise;
    }
    update_yankreg_width(reg);
}
#[inline]
pub unsafe fn is_literal_register(regname: ::core::ffi::c_int) -> bool {
    return regname == '*' as ::core::ffi::c_int
        || regname == '+' as ::core::ffi::c_int
        || (regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(regname) as ::core::ffi::c_int != 0);
}
#[inline]
pub unsafe fn op_reg_index(regname: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if ascii_isdigit(regname) {
        return regname - '0' as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'a' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
    {
        return regname as uint8_t as ::core::ffi::c_int - 'A' as ::core::ffi::c_int
            + 10 as ::core::ffi::c_int;
    } else if regname == '-' as ::core::ffi::c_int {
        return DELETION_REGISTER as ::core::ffi::c_int;
    } else if regname == '*' as ::core::ffi::c_int {
        return STAR_REGISTER as ::core::ffi::c_int;
    } else if regname == '+' as ::core::ffi::c_int {
        return PLUS_REGISTER as ::core::ffi::c_int;
    } else {
        return -1 as ::core::ffi::c_int;
    };
}
#[inline]
unsafe extern "C" fn is_append_register(mut regname: ::core::ffi::c_int) -> bool {
    return regname as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && regname as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint;
}
#[inline]
pub unsafe fn get_register_name(mut num: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if num == -1 as ::core::ffi::c_int {
        return '"' as ::core::ffi::c_int;
    } else if num < 10 as ::core::ffi::c_int {
        return num + '0' as ::core::ffi::c_int;
    } else if num == DELETION_REGISTER as ::core::ffi::c_int {
        return '-' as ::core::ffi::c_int;
    } else if num == STAR_REGISTER as ::core::ffi::c_int {
        return '*' as ::core::ffi::c_int;
    } else if num == PLUS_REGISTER as ::core::ffi::c_int {
        return '+' as ::core::ffi::c_int;
    } else {
        return num + 'a' as ::core::ffi::c_int - 10 as ::core::ffi::c_int;
    };
}
#[inline]
unsafe extern "C" fn reg_empty(reg: *const yankreg_T) -> bool {
    return (*reg).y_array.is_null()
        || (*reg).y_size == 0 as size_t
        || (*reg).y_size == 1 as size_t
            && (*reg).y_type as ::core::ffi::c_int == kMTCharWise as ::core::ffi::c_int
            && (*(*reg).y_array.offset(0 as ::core::ffi::c_int as isize)).size == 0 as size_t;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
