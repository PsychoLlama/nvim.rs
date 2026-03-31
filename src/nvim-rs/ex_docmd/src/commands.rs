//! Ex command handler implementations (Phase 1 migration).
//!
//! This module contains Rust implementations of ex command handlers
//! migrated from ex_docmd.c.

use std::ffi::{c_char, c_int, c_void};

use crate::ExArgHandle;
use nvim_normal::types::OpargT;

// =============================================================================
// Type aliases
// =============================================================================

/// Line number type (matches linenr_T in C = int32_t).
type LinenrT = i32;

/// Opaque handle to a win_T.
type WinHandle = *mut c_void;

// =============================================================================
// CMD_ enum constants for skip_cmd
// =============================================================================

pub(crate) const CMD_APPEND: c_int = 0;
pub(crate) const CMD_AUTOCMD: c_int = 17;
pub(crate) const CMD_AT: c_int = 553;
pub(crate) const CMD_BANG: c_int = 547;
pub(crate) const CMD_BDELETE: c_int = 25;
pub(crate) const CMD_BUNLOAD: c_int = 41;
pub(crate) const CMD_BWIPEOUT: c_int = 42;
pub(crate) const CMD_CC: c_int = 59;
pub(crate) const CMD_CHANGE: c_int = 43;
pub(crate) const CMD_CHECKTIME: c_int = 75;
pub(crate) const CMD_DIFFGET: c_int = 119;
pub(crate) const CMD_DIFFPUT: c_int = 122;
pub(crate) const CMD_EDIT: c_int = 133;
pub(crate) const CMD_FILE: c_int = 154;
pub(crate) const CMD_INSERT: c_int = 184;
pub(crate) const CMD_IPUT: c_int = 197;
pub(crate) const CMD_K: c_int = 205;
pub(crate) const CMD_LL: c_int = 243;
pub(crate) const CMD_LVIMGREP: c_int = 267;
pub(crate) const CMD_LVIMGREPADD: c_int = 268;
pub(crate) const CMD_NEXT: c_int = 555;
pub(crate) const CMD_PUT: c_int = 343;
pub(crate) const CMD_REDIR: c_int = 362;
pub(crate) const CMD_SIZE: c_int = 556;
pub(crate) const CMD_TABMOVE: c_int = 458;
pub(crate) const CMD_TABNEXT: c_int = 460;
pub(crate) const CMD_VIMGREP: c_int = 509;
pub(crate) const CMD_VIMGREPADD: c_int = 510;
pub(crate) const CMD_ABOVELEFT: c_int = 3;
pub(crate) const CMD_AND: c_int = 549;
pub(crate) const CMD_BELOWRIGHT: c_int = 26;
pub(crate) const CMD_BOTRIGHT: c_int = 31;
pub(crate) const CMD_BROWSE: c_int = 38;
pub(crate) const CMD_CALL: c_int = 53;
pub(crate) const CMD_CATCH: c_int = 54;
pub(crate) const CMD_CONFIRM: c_int = 97;
pub(crate) const CMD_CONST: c_int = 99;
pub(crate) const CMD_DELFUNCTION: c_int = 115;
pub(crate) const CMD_DJUMP: c_int = 126;
pub(crate) const CMD_DLIST: c_int = 127;
pub(crate) const CMD_DSEARCH: c_int = 131;
pub(crate) const CMD_DSPLIT: c_int = 132;
pub(crate) const CMD_ECHO: c_int = 135;
pub(crate) const CMD_ECHOERR: c_int = 136;
pub(crate) const CMD_ECHOMSG: c_int = 138;
pub(crate) const CMD_ECHON: c_int = 139;
pub(crate) const CMD_ELSE: c_int = 140;
pub(crate) const CMD_ELSEIF: c_int = 141;
pub(crate) const CMD_ENDIF: c_int = 143;
pub(crate) const CMD_ENDFOR: c_int = 145;
pub(crate) const CMD_ENDTRY: c_int = 146;
pub(crate) const CMD_ENDWHILE: c_int = 147;
pub(crate) const CMD_EVAL: c_int = 149;
pub(crate) const CMD_EXECUTE: c_int = 151;
pub(crate) const CMD_FILTER: c_int = 157;
pub(crate) const CMD_FINALLY: c_int = 159;
pub(crate) const CMD_FOR: c_int = 167;
pub(crate) const CMD_FUNCTION: c_int = 168;
pub(crate) const CMD_GREP: c_int = 172;
pub(crate) const CMD_GREPADD: c_int = 173;
pub(crate) const CMD_LGREP: c_int = 239;
pub(crate) const CMD_LGREPADD: c_int = 240;
pub(crate) const CMD_LMAKE: c_int = 248;
pub(crate) const CMD_MAKE: c_int = 273;
pub(crate) const CMD_HELP: c_int = 176;
pub(crate) const CMD_HIDE: c_int = 181;
pub(crate) const CMD_HORIZONTAL: c_int = 183;
pub(crate) const CMD_IF: c_int = 187;
pub(crate) const CMD_IJUMP: c_int = 188;
pub(crate) const CMD_ILIST: c_int = 189;
pub(crate) const CMD_ISEARCH: c_int = 198;
pub(crate) const CMD_ISPLIT: c_int = 199;
pub(crate) const CMD_KEEPALT: c_int = 209;
pub(crate) const CMD_KEEPJUMPS: c_int = 207;
pub(crate) const CMD_KEEPMARKS: c_int = 206;
pub(crate) const CMD_KEEPPATTERNS: c_int = 208;
pub(crate) const CMD_LEFTABOVE: c_int = 230;
pub(crate) const CMD_LET: c_int = 231;
pub(crate) const CMD_LOCKMARKS: c_int = 255;
pub(crate) const CMD_LOCKVAR: c_int = 256;
pub(crate) const CMD_LUA: c_int = 264;
pub(crate) const CMD_MATCH: c_int = 277;
pub(crate) const CMD_MZSCHEME: c_int = 287;
pub(crate) const CMD_NOAUTOCMD: c_int = 297;
pub(crate) const CMD_NOSWAPFILE: c_int = 301;
pub(crate) const CMD_PERL: c_int = 322;
pub(crate) const CMD_PSEARCH: c_int = 333;
pub(crate) const CMD_PYTHON: c_int = 345;
pub(crate) const CMD_PY3: c_int = 348;
pub(crate) const CMD_PYTHON3: c_int = 350;
pub(crate) const CMD_PYTHONX: c_int = 354;
pub(crate) const CMD_PYX: c_int = 352;
pub(crate) const CMD_RETURN: c_int = 370;
pub(crate) const CMD_RIGHTBELOW: c_int = 373;
pub(crate) const CMD_RUBY: c_int = 377;
pub(crate) const CMD_SILENT: c_int = 406;
pub(crate) const CMD_SMAGIC: c_int = 409;
pub(crate) const CMD_SNOMAGIC: c_int = 414;
pub(crate) const CMD_SUBSTITUTE: c_int = 381;
pub(crate) const CMD_SYNTAX: c_int = 443;
pub(crate) const CMD_TAB: c_int = 452;
pub(crate) const CMD_TCL: c_int = 467;
pub(crate) const CMD_THROW: c_int = 472;
pub(crate) const CMD_TILDE: c_int = 554;
pub(crate) const CMD_TOPLEFT: c_int = 483;
pub(crate) const CMD_TRY: c_int = 487;
pub(crate) const CMD_UNLET: c_int = 497;
pub(crate) const CMD_UNLOCKVAR: c_int = 498;
pub(crate) const CMD_VERBOSE: c_int = 505;
pub(crate) const CMD_VERTICAL: c_int = 506;
pub(crate) const CMD_WHILE: c_int = 524;
pub(crate) const CMD_WINCMD: c_int = 526;
pub(crate) const CMD_FOLDOPEN: c_int = 166;
pub(crate) const CMD_TABFIRST: c_int = 457;
pub(crate) const CMD_TABLAST: c_int = 459;
pub(crate) const CMD_TABPREVIOUS: c_int = 463;
pub(crate) const CMD_TABNEXT_BACKWARD: c_int = 464; // CMD_tabNext
pub(crate) const CMD_TABREWIND: c_int = 465;
pub(crate) const CMD_EARLIER: c_int = 134;
pub(crate) const CMD_DELETE: c_int = 110;
pub(crate) const CMD_YANK: c_int = 546;
pub(crate) const CMD_RSHIFT: c_int = 553;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    static mut msg_silent: c_int;
    static mut restart_edit: c_int;
    // Message functions
    fn msg(s: *const c_char, a: c_int);
    fn smsg(a: c_int, fmt: *const c_char, ...);
    fn semsg(fmt: *const c_char, ...);
    fn emsg(s: *const c_char);
    fn msg_puts(s: *const c_char);

    // Verbose message helpers
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    static mut no_wait_return: c_int;

    // Error messages
    static e_secure: c_char;

    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn xfree(p: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn isupper(c: c_int) -> c_int;

    // Character-level iteration helpers
    #[link_name = "utfc_ptr2len"]
    fn nvim_docmd_utfc_ptr2len(p: *const c_char) -> c_int;

    // eap accessors
    fn nvim_docmd_get_curbuf_line_count() -> LinenrT;

    // Global state accessors
    static cmdwin_type: c_int;
    fn nvim_set_cmdwin_result(val: c_int);
    fn nvim_curbuf_locked() -> c_int;

    // Redir accessors (redir_fd/reg/vname are in globals.h)
    fn nvim_docmd_get_redir_fd() -> *mut c_void;
    fn nvim_docmd_set_redir_fd(fd: *mut c_void);
    static mut redir_reg: c_int;
    fn nvim_docmd_set_redir_reg(reg: c_int);
    static mut redir_vname: bool;
    fn nvim_docmd_set_redir_vname(val: c_int);
    static mut redir_off: bool;

    // Redir helpers
    fn nvim_docmd_close_redir();
    fn nvim_docmd_fclose_redir_fd();
    fn nvim_docmd_get_redir_vname() -> c_int;
    fn var_redir_stop();
    fn open_exfile(fname: *const c_char, forceit: c_int, mode: *const c_char) -> *mut c_void;
    fn expand_env_save(arg: *const c_char) -> *mut c_char;
    fn valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn write_reg_contents(regname: c_int, str_: *const c_char, len: isize, must_append: c_int);
    fn var_redir_start(name: *const c_char, append: bool) -> c_int;

    // ex_normal helpers
    fn nvim_docmd_get_p_mmd() -> c_int;
    fn nvim_docmd_curbuf_has_terminal() -> c_int;
    fn nvim_docmd_curwin_in_terminal_mode() -> c_int;
    fn expr_map_locked() -> bool;
    fn save_current_state(save: *mut c_void) -> bool;
    fn restore_current_state(save: *mut c_void);
    fn exec_normal_cmd(cmd: *const c_char, remap: c_int, silent: bool);
    fn check_cursor_moved(wp: WinHandle);
    fn update_topline_cursor();
    fn setmouse();
    fn ui_cursor_shape();

    // curwin accessors for ex_normal
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_docmd_set_curwin_cursor_lnum(lnum: LinenrT);
    fn nvim_docmd_set_curwin_cursor_col(col: i32);

    // Filetype helpers
    fn nvim_docmd_get_filetype_detect() -> c_int;
    fn nvim_docmd_set_filetype_detect(val: c_int);
    fn nvim_docmd_get_filetype_plugin() -> c_int;
    fn nvim_docmd_set_filetype_plugin(val: c_int);
    fn nvim_docmd_get_filetype_indent() -> c_int;
    fn nvim_docmd_set_filetype_indent(val: c_int);
    fn source_runtime(fname: *const c_char, flags: c_int) -> c_int;
    fn do_doautocmd(eap_arg: *mut c_char, do_msg: bool, did_something: *mut bool) -> c_int;
    fn do_modelines(flags: c_int);

    // ex_quit helpers
    fn text_locked() -> bool;
    fn text_locked_msg();
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_buffer(wp: WinHandle) -> *mut c_void;
    fn before_quit_autocmds(wp: WinHandle, quit_all: bool, forceit: bool) -> bool;
    fn nvim_docmd_check_more(message: c_int, forceit: c_int) -> c_int;
    fn rs_only_one_window() -> c_int;
    fn nvim_ex2_buf_hide(buf: *mut c_void) -> bool;
    fn check_changed(buf: *mut c_void, flags: c_int) -> bool;
    fn check_changed_any(hidden: bool, unload: bool) -> bool;
    fn getout(exitval: c_int);
    fn win_close(wp: WinHandle, free_buf: bool, force: bool) -> c_int;
    fn nvim_docmd_one_window_p(addr_count: c_int) -> c_int;

    // memory allocation
    fn xcalloc(count: usize, size: usize) -> *mut c_void;

    // eap argt and other needed accessors
    fn nvim_skip_expr_arg(arg: *mut *mut c_char);

    // is_other_file helpers
    fn nvim_docmd_get_curbuf_fnum() -> c_int;
    fn nvim_docmd_curbuf_file_id_valid() -> c_int;
    fn nvim_docmd_get_curbuf_sfname() -> *const c_char;
    fn path_fnamecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn otherfile(fname: *const c_char) -> c_int;

    // Phase 3: changedir_func helpers
    fn nvim_allbuf_locked() -> bool;
    fn nvim_get_prevdir(scope: c_int) -> *mut c_char;
    fn nvim_set_prevdir(scope: c_int, pdir: *mut c_char);
    fn nvim_os_dirname_namebuff() -> c_int;
    fn nvim_expand_env_home_namebuff();
    fn nvim_get_p_cdh() -> c_int;
    fn nvim_vim_chdir(dir: *const c_char) -> c_int;
    fn nvim_do_autocmd_dirchanged_manual_pre(new_dir: *const c_char, scope: c_int);
    fn nvim_post_chdir(scope: c_int, dir_differs: bool);
    #[link_name = "pathcmp"]
    fn nvim_pathcmp_unlen(a: *const c_char, b: *const c_char, maxlen: c_int) -> c_int;
    fn nvim_get_namebuff() -> *mut c_char;
    fn xstrdup(str: *const c_char) -> *mut c_char;

    // Phase 5: ex_fold / ex_foldopen / ex_digraphs helpers
    fn rs_foldManualAllowed(create: bool) -> c_int;
    fn rs_foldCreate(wp: WinHandle, start_lnum: LinenrT, end_lnum: LinenrT);
    fn rs_opFoldRange(
        first_lnum: LinenrT,
        last_lnum: LinenrT,
        opening: c_int,
        recurse: c_int,
        had_visual: bool,
    );
    fn putdigraph(str: *mut c_char);
    fn rs_listdigraphs(use_headers: c_int);

    // Phase 5: ex_mode helpers
    fn nvim_docmd_set_must_redraw(val: c_int);
    // Phase 5: ex_swapname helpers
    fn nvim_docmd_get_curbuf_swapname() -> *const c_char;
    // Phase 6: ex_tabnext helpers
    fn goto_tabpage(n: c_int);
    fn nvim_docmd_parse_tabnext_count(eap: ExArgHandle, errmsg_set: *mut c_int) -> c_int;

    // Phase 7: ex_undo helpers
    fn u_undo(count: c_int);
    fn u_undo_and_forget(count: c_int, do_buf_event: bool) -> bool;
    fn undo_time(step: c_int, sec: bool, file: bool, absolute: bool);
    fn nvim_curbuf_get_u_seq_cur() -> c_int;
    fn nvim_docmd_undo_count_steps(step: LinenrT, found: *mut c_int) -> c_int;
    // eval_vars helpers
    fn eval_vars(
        src: *mut c_char,
        srcstart: *const c_char,
        usedlen: *mut usize,
        lnump: *mut i32,
        errormsg: *mut *const c_char,
        escaped: *mut c_int,
        empty_is_error: bool,
    ) -> *mut c_char;

    // Phase 8: ex_sleep / do_sleep helpers
    fn nvim_docmd_cursor_valid_curwin() -> c_int;
    fn nvim_docmd_setcursor_mayforce_curwin();
    fn ui_busy_start();
    fn ui_busy_stop();
    fn nvim_docmd_loop_sleep(msec: i64);
    fn vpeekc() -> c_int;

    // Phase 10: ex_operators helpers
    fn nvim_set_virtual_op_false();
    fn nvim_set_virtual_op_none();
    fn setpcmark();
    fn beginline(flags: c_int);
    fn end_visual_mode();
    fn clear_oparg(oap: *mut OpargT);
    fn op_delete(oap: *mut OpargT) -> c_int;
    fn op_yank(oap: *mut OpargT, message: bool) -> bool;
    fn op_shift(oap: *mut OpargT, curs_top: bool, amount: c_int);
    fn nvim_curwin_get_w_p_rl() -> c_int;
    fn nvim_docmd_ex_may_print_impl(eap: ExArgHandle);
    fn beep_flush();
    fn do_join(count: usize, insert_space: bool, save_undo: bool, use_cursor: bool, setmark: bool);
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
}

// =============================================================================
// HLF_E constant (error message highlight)
// =============================================================================

/// HLF_E = 6 (see highlight_defs.h enum order)
const HLF_E: c_int = 6;

/// REMAP_YES = 0, REMAP_NONE = -1 (see keycodes.h)
const REMAP_YES: c_int = 0;
const REMAP_NONE: c_int = -1;

/// OK = 1, FAIL = 0 (Nvim conventions)
const OK: c_int = 1;

/// CCGD_AW = 1, CCGD_FORCEIT = 4, CCGD_EXCMD = 16
const CCGD_AW: c_int = 1;
const CCGD_MULTWIN: c_int = 2;
const CCGD_FORCEIT: c_int = 4;
const CCGD_EXCMD: c_int = 16;

/// TriState values: kNone=-1, kFalse=0, kTrue=1
const K_TRUE: c_int = 1;
const K_FALSE: c_int = 0;
// K_NONE = -1 (unused - kept for reference)

/// K_SPECIAL key value (from keycodes.h)
const K_SPECIAL: u8 = 0x80;
/// KS_SPECIAL key secondary byte
const KS_SPECIAL: u8 = 191;
/// KE_FILLER key extra byte
const KE_FILLER: u8 = 1;

/// Ctrl_C = 3
const CTRL_C: c_int = 3;

/// UPD_CLEAR = 50 (from drawscreen.h)
const UPD_CLEAR: c_int = 50;
/// UPD_NOT_VALID = 40
const UPD_NOT_VALID: c_int = 40;
/// UPD_SOME_VALID = 35
const UPD_SOME_VALID: c_int = 35;
/// UPD_INVERTED = 20
const UPD_INVERTED: c_int = 20;

/// MODE_CMDLINE = 0x08
const MODE_CMDLINE: c_int = 0x08;
/// MODE_INSERT = 0x10
const MODE_INSERT: c_int = 0x10;

/// CMD_startinsert = 431, CMD_startgreplace = 432, CMD_startreplace = 433
pub(crate) const CMD_STARTINSERT: c_int = 431;

// =============================================================================
// verify_command - smile easter egg
// =============================================================================

/// Rust implementation of verify_command.
///
/// Displays the smile easter egg if `cmd` == "smile".
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
#[unsafe(export_name = "verify_command")]
#[rustfmt::skip]
pub unsafe extern "C" fn rs_verify_command(cmd: *const c_char) {
    // Check if cmd == "smile"
    if cmd.is_null() {
        return;
    }
    let bytes = std::ffi::CStr::from_ptr(cmd).to_bytes();
    if bytes != b"smile" {
        return;
    }

    let a = HLF_E;
    msg(c" #xxn`          #xnxx`        ,+x@##@Mz;`        .xxxxxxxxxxxxxxnz+,      znnnnnnnnnnnnnnnn.".as_ptr(), a);
    msg(c" n###z          x####`      :x##########W+`      ,###########@WW##M;    W################.".as_ptr(), a);
    msg(c" n####;         x####`    `z##############W:     ,#################   W################.".as_ptr(), a);
    msg(c" n####W.        x####`   ,W#################+    ,##############  W################.".as_ptr(), a);
    msg(c" n#####n        x####`   @###################    ,##############i W################.".as_ptr(), a);
    msg(c" n######i       x####`  .#########@W@########*   ,##############W`W################.".as_ptr(), a);
    msg(c" n######@.      x####`  x######W*.  `;n#######:  ,###x,,,,:*M######iW###@:,,,,,,,,,,,`".as_ptr(), a);
    msg(c" n#######n      x####` *######+`       :M#####M  ,###n      `x#####xW###@`".as_ptr(), a);
    msg(c" n########*     x####``@####@;          `x#####i ,###n       ,#####@W###@`".as_ptr(), a);
    msg(c" n########@     x####`*#####i            `M####M ,###n        x#########@`".as_ptr(), a);
    msg(c" n#########     x####`M####z              :#####:,###n        z#########@`".as_ptr(), a);
    msg(c" n#########*    x####,#####.               n####+,###n        n#########@`".as_ptr(), a);
    msg(c" n####@####@,   x####i####x                ;####x,###n       `W#####@####+++++++++++i".as_ptr(), a);
    msg(c" n####*#####M`  x#########*                `####@,###n       i#####MW###############W".as_ptr(), a);
    msg(c" n####.######+  x####z####;                 W####,###n      i@######W###############W".as_ptr(), a);
    msg(c" n####.`W#####: x####n####:                 M####:###@nnnnnW#######,W###############W".as_ptr(), a);
    msg(c" n####. :#####M`x####z####;                 W####,###############z W###############W".as_ptr(), a);
    msg(c" n####.  #######x#########*                `####W,#############W` W###############W".as_ptr(), a);
    msg(c" n####.  `M#####W####i####x                ;####x,############W,  W####+**********i".as_ptr(), a);
    msg(c" n####.   ,##########,#####.               n####+,###########n.   W###@`".as_ptr(), a);
    msg(c" n####.    ##########`M####z              :#####:,########Wz:     W###@`".as_ptr(), a);
    msg(c" n####.    x#########`*#####i            `M####M ,###x.....`        W###@`".as_ptr(), a);
    msg(c" n####.    ,@########``@####@;          `x#####i ,###n              W###@`".as_ptr(), a);
    msg(c" n####.     *########` *#####@+`       ,M#####M  ,###n              W###@`".as_ptr(), a);
    msg(c" n####.      x#######`  x######W*.  `;n######@:  ,###n              W###@,,,,,,,,,,,,`".as_ptr(), a);
    msg(c" n####.      .@######`  .#########@W@########*   ,###n              W################,".as_ptr(), a);
    msg(c" n####.       i######`   @###################    ,###n              W################,".as_ptr(), a);
    msg(c" n####.        n#####`   ,W#################+    ,###n              W################,".as_ptr(), a);
    msg(c" n####.        .@####`    .n##############W;     ,###n              W################,".as_ptr(), a);
    msg(c" n####.         i####`      :x##########W+`      ,###n              W################,".as_ptr(), a);
    msg(c" +nnnn`          +nnn`        ,+x@##@Mz;`        .nnnn+              zxxxxxxxxxxxxxxxx.".as_ptr(), a);
    msg(c" ".as_ptr(), a);
    msg(c"                                                                                 ,+M@#Mi".as_ptr(), a);
    msg(c"                                                                              .z########".as_ptr(), a);
    msg(c"                                                                             i@#########i".as_ptr(), a);
    msg(c"                                                                           `############W`".as_ptr(), a);
    msg(c"                                                                          `n#############i".as_ptr(), a);
    msg(c"                                                                         `n##############n".as_ptr(), a);
    msg(c"     ``                                                                  z###############@`".as_ptr(), a);
    msg(c"    `W@z,                                                               ##################,".as_ptr(), a);
    msg(c"    *#####`                                                             i############@x@###i".as_ptr(), a);
    msg(c"    ######M.                                                           :#############n`,W##+".as_ptr(), a);
    msg(c"    +######@:                                                         .W#########M@##+  *##z".as_ptr(), a);
    msg(c"    :#######@:                                                        `x########@#x###*  ,##n".as_ptr(), a);
    msg(c"    `@#######@;                                                       z#########M*@nW#i  .##x".as_ptr(), a);
    msg(c"     z########@i                                                     *###########WM#@#,  `##x".as_ptr(), a);
    msg(c"     i##########+                                                   ;###########*n###@   `##x".as_ptr(), a);
    msg(c"     `@#MM#######x,                                                ,@#########zM,`z##M   `@#x".as_ptr(), a);
    msg(c"      n##M#W#######n.            `.:i*+#zzzz##+i:.`             ,W#########Wii,`n@#@` n@##n".as_ptr(), a);
    msg(c"      ;###@#x#######n         `,i#nW@#####@@WWW@@####@Mzi.        ,W##########@z.. ;zM#+i####z".as_ptr(), a);
    msg(c"       x####nz########    .;#x@##@Wn#*;,.`      ``,:*#x@##M+,    ;@########xz@WM+#` `n@#######".as_ptr(), a);
    msg(c"       ,@####M########xi#@##@Mzi,`                     .+x###Mi:n##########Mz```.:i  *@######*".as_ptr(), a);
    msg(c"        *#####W#########ix+:`                               :n#############z:       `*.`M######i".as_ptr(), a);
    msg(c"        i#W##nW@+@##@#M@;                                     ;W@@##########W,        i`x@#####,".as_ptr(), a);
    msg(c"        `@@n@Wn#@iMW*#*:                                        `iz#z@######x.           M######`".as_ptr(), a);
    msg(c"         z##zM###x`*, .`                                              `iW#####W;:`        +#####M".as_ptr(), a);
    msg(c"         ,###nn##n`                                                    ,#####x;`        ,;@######".as_ptr(), a);
    msg(c"          x###xz#.                                                       in###+        `:######@.".as_ptr(), a);
    msg(c"          ;####n+                                                         `Mnx##xi`   , zM#######".as_ptr(), a);
    msg(c"          `W####+                i.                                          `.+x###@#. :n,z######:".as_ptr(), a);
    msg(c"           z####@`              ;#:                                             .ii@###@;.*M*z####@`".as_ptr(), a);
    msg(c"           i####M         `   `i@#,           ::                                  +#n##@+@##W####n".as_ptr(), a);
    msg(c"           :####x    ,i. ##xzM###@`     i.   .@@,                                  .z####x#######*".as_ptr(), a);
    msg(c"           ,###W;   i##Wz#########     :##   z##n                                  ,@########x###:".as_ptr(), a);
    msg(c"            n##n   `W###########M`;n,  i#x  ,###@i                                  *W########W#@`".as_ptr(), a);
    msg(c"           .@##+  `x###########@. z#+ .M#W``x#####n`                                `;#######@z#x".as_ptr(), a);
    msg(c"           n###z :W############@  z#*  @##xM#######@n;                               `########nW+".as_ptr(), a);
    msg(c"          ;####nW##############W :@#* `@#############*                               :########z@i`".as_ptr(), a);
    msg(c"          M##################### M##:  @#############@:                              *W########M#".as_ptr(), a);
    msg(c"         ;#####################i.##x`  W#############W,                              :n########zx".as_ptr(), a);
    msg(c"         x####################@.`x;    @#############z.                              .@########W#".as_ptr(), a);
    msg(c"        ,######################`        W###############x*,`                          W######zM#i".as_ptr(), a);
    msg(c"        #######################:        z##################@x+*#zzi                   `@#########.".as_ptr(), a);
    msg(c"        W########W#z#M#########;        *##########################z                   :@#######@`".as_ptr(), a);
    msg(c"       `@#######x`;#z ,x#######;        z###########M###xnM@########*                  :M######@".as_ptr(), a);
    msg(c"       i########, x#@`  z######;        *##########i *#@`  `+########+`                  n######.".as_ptr(), a);
    msg(c"       n#######@` M##,  `W#####.        *#########z  ###;    z########M:                 :W####n".as_ptr(), a);
    msg(c"       M#######M  n##.   x####x          `x########:  z##+    M#########@;                .n###+".as_ptr(), a);
    msg(c"       W#######@` :#W   `@####:           `@######W   i###   ;###########@.                n##n".as_ptr(), a);
    msg(c"       W########z` ,,  .x####z             @######@`  `W#;  `W############*                *###;".as_ptr(), a);
    msg(c"      `@#########Mi,:*n@####W`              W#######*   ..  `n#############i                i###x".as_ptr(), a);
    msg(c"      .#####################z               `@#######@*`    .x############n:`               ;####.".as_ptr(), a);
    msg(c"      :####################x`,,`             `W#########@x#+#@#############i                ,####:".as_ptr(), a);
    msg(c"      ;###################x#@###xi`           *############################:                `####i".as_ptr(), a);
    msg(c"      i##################+#######M,            x##########################@`                W###i".as_ptr(), a);
    msg(c"      *################@; @#######@,            .W#########################@                x###:".as_ptr(), a);
    msg(c"      .+M#############z.  M########x             ,W########################@`               ####.".as_ptr(), a);
    msg(c"      *M*;z@########x:    :W######i               .M########################i               i###:".as_ptr(), a);
    msg(c"      *##@z;#@####x:        :z###@i                `########################x               .###;".as_ptr(), a);
    msg(c"      *#####n;#@##            ;##*                   ,x#####################@`               W##*".as_ptr(), a);
    msg(c"      *#######n;*            :M##W*,                   *W####################`               n##z".as_ptr(), a);
    msg(c"      i########@.         ,*n######M*`                   `###################M               *##M".as_ptr(), a);
    msg(c"      i########n        `z#####@@#####Wi                   ,M################;               ,##@`".as_ptr(), a);
    msg(c"      ;WMWW@###*       .x##@ni.``.:+zW##z`                 `n##############z                  @##,".as_ptr(), a);
    msg(c"      .*++*i;;;.      .M#@+`           .##n                  `x############x`                 n##i".as_ptr(), a);
    msg(c"      :########*      x#W,               *#+                  *###########M`                  +##+".as_ptr(), a);
    msg(c"      ,#########     :#@:                 ##:                   #nzzzzzzzzzz.                  :##x".as_ptr(), a);
    msg(c"      .#####Wz+`     ##+                   `MM`                  .znnnnnnnnn.                  `@#@`".as_ptr(), a);
    msg(c"      `@@ni;*nMz`    @W`                    :#+                   .x#######n                   x##,".as_ptr(), a);
    msg(c"       i;z@#####,   .#*                      z#:                   ;;;*zW##;                   ###i".as_ptr(), a);
    msg(c"       z########:   :#;                       `Wx                  +###Wni;n.                   ;##z".as_ptr(), a);
    msg(c"       n########W:  .#*                        ,#,                 ;#######@+                   `@#M".as_ptr(), a);
    msg(c"      .###########n;.MM                          n*                 ;iM#######*                   x#@`".as_ptr(), a);
    msg(c"      :#############@;;                          .n`               ,#W*iW#####W`                  +##,".as_ptr(), a);
    msg(c"      ,##############.                            ix.             `x###M;#######                  ,##i".as_ptr(), a);
    msg(c"      .#############@`                             x@n**#W######z;M###@.                          W##".as_ptr(), a);
    msg(c"      .##############W:                            .x############@*;zW#;                          z#x".as_ptr(), a);
    msg(c"      ,###############@;                            `##############@n*;.                           i#@".as_ptr(), a);
    msg(c"      ,#################i                             :n##############W`                           .##,".as_ptr(), a);
    msg(c"      ,###################`                             .+W##########W,                            `##i".as_ptr(), a);
    msg(c"      :###################@zi,`                            ;zM@@@WMn*`                              @#z".as_ptr(), a);
    msg(c"      :#######################@x+*i;;:i#M,                 ``                                      M#W".as_ptr(), a);
    msg(c"      ;##############################@x.                                                            n##,".as_ptr(), a);
    msg(c"      i#####################@W@@@Wxz*:`                                                             *##+".as_ptr(), a);
    msg(c"      *######################+```                                                                   :##M".as_ptr(), a);
    msg(c"      ########################M;                                                                    `@##,".as_ptr(), a);
    msg(c"      z#########################x,                                                                   z###".as_ptr(), a);
    msg(c"      n############################n:                                                                ;##W`".as_ptr(), a);
    msg(c"      x###############################Mz#++##*                                                       `W##i".as_ptr(), a);
    msg(c"      M##########@`                                                                                   ###x".as_ptr(), a);
    msg(c"      W###########`                                                                                   .###,".as_ptr(), a);
    msg(c"      @##########M                                                                                     n##z".as_ptr(), a);
    msg(c"      @##################z*i@WMMMx#x@#####,.                                                          :##@.".as_ptr(), a);
    msg(c"     `#####################@xi`   `::,*                                                                x##+".as_ptr(), a);
    msg(c"     .#####################@#M.                                                                        ;##@`".as_ptr(), a);
    msg(c"     ,#####################:.                                                                           M##i".as_ptr(), a);
    msg(c"     ;###################ni`                                                                            i##M".as_ptr(), a);
    msg(c"     *#################W#`                                                                              `W##,".as_ptr(), a);
    msg(c"     z#################@Wx+.                                                                             +###".as_ptr(), a);
    msg(c"     x######################z.                                                                          .@#@`".as_ptr(), a);
    msg(c"    `@#######################@;                                                                           z##;".as_ptr(), a);
    msg(c"    :##########################:                                                                          :##z".as_ptr(), a);
    msg(c"    +#########################W#                                                                           M#W".as_ptr(), a);
    msg(c"    W################@n+*i;:,`                                                                            +##,".as_ptr(), a);
    msg(c"   :##################WMxz+,                                                                              ,##i".as_ptr(), a);
    msg(c"   n#######################W..,                                                                             W##".as_ptr(), a);
    msg(c"  +#########################WW@+. .:.                                                                       z#x".as_ptr(), a);
    msg(c" `@#############################@@###:                                                                       *#W".as_ptr(), a);
    msg(c" #################################Wz:                                                                         :#@".as_ptr(), a);
    msg(c",@###############################i                                                                            .##".as_ptr(), a);
    msg(c"n@@@@@@@#########################+                                                                             `##".as_ptr(), a);
    msg(c"`      `.:.`.,:iii;;;;;;;;iii;;;:`       `.``                                                                `nW".as_ptr(), a);
}

// =============================================================================
// skip_cmd
// =============================================================================

/// Rust implementation of skip_cmd.
///
/// Returns true if the command should be skipped (not executed).
/// Used during conditional statement parsing.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "skip_cmd"]
pub unsafe extern "C" fn rs_skip_cmd(eap: ExArgHandle) -> c_int {
    if eap.is_null() {
        return 0;
    }

    let skip = (*eap).skip != 0;
    if !skip {
        return 0;
    }

    let cmdidx = (*eap).cmdidx;

    // Commands that need evaluation even when skipping
    let needs_eval = matches!(
        cmdidx,
        CMD_WHILE
            | CMD_ENDWHILE
            | CMD_FOR
            | CMD_ENDFOR
            | CMD_IF
            | CMD_ELSEIF
            | CMD_ELSE
            | CMD_ENDIF
            | CMD_TRY
            | CMD_CATCH
            | CMD_FINALLY
            | CMD_ENDTRY
            | CMD_FUNCTION
    );

    // Commands that handle '|' themselves
    let handles_bar = matches!(
        cmdidx,
        CMD_ABOVELEFT
            | CMD_AND
            | CMD_BELOWRIGHT
            | CMD_BOTRIGHT
            | CMD_BROWSE
            | CMD_CALL
            | CMD_CONFIRM
            | CMD_CONST
            | CMD_DELFUNCTION
            | CMD_DJUMP
            | CMD_DLIST
            | CMD_DSEARCH
            | CMD_DSPLIT
            | CMD_ECHO
            | CMD_ECHOERR
            | CMD_ECHOMSG
            | CMD_ECHON
            | CMD_EVAL
            | CMD_EXECUTE
            | CMD_FILTER
            | CMD_HELP
            | CMD_HIDE
            | CMD_HORIZONTAL
            | CMD_IJUMP
            | CMD_ILIST
            | CMD_ISEARCH
            | CMD_ISPLIT
            | CMD_KEEPALT
            | CMD_KEEPJUMPS
            | CMD_KEEPMARKS
            | CMD_KEEPPATTERNS
            | CMD_LEFTABOVE
            | CMD_LET
            | CMD_LOCKMARKS
            | CMD_LOCKVAR
            | CMD_LUA
            | CMD_MATCH
            | CMD_MZSCHEME
            | CMD_NOAUTOCMD
            | CMD_NOSWAPFILE
            | CMD_PERL
            | CMD_PSEARCH
            | CMD_PYTHON
            | CMD_PY3
            | CMD_PYTHON3
            | CMD_PYTHONX
            | CMD_PYX
            | CMD_RETURN
            | CMD_RIGHTBELOW
            | CMD_RUBY
            | CMD_SILENT
            | CMD_SMAGIC
            | CMD_SNOMAGIC
            | CMD_SUBSTITUTE
            | CMD_SYNTAX
            | CMD_TAB
            | CMD_TCL
            | CMD_THROW
            | CMD_TILDE
            | CMD_TOPLEFT
            | CMD_UNLET
            | CMD_UNLOCKVAR
            | CMD_VERBOSE
            | CMD_VERTICAL
            | CMD_WINCMD
    );

    if needs_eval || handles_bar {
        0
    } else {
        1
    }
}

// =============================================================================
// msg_verbose_cmd
// =============================================================================

/// Rust implementation of msg_verbose_cmd.
///
/// Prints the executed command for when 'verbose' is set.
/// If lnum == 0, only prints the command.
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
#[export_name = "msg_verbose_cmd"]
pub unsafe extern "C" fn rs_msg_verbose_cmd(lnum: LinenrT, cmd: *const c_char) {
    let no_wait = no_wait_return;
    no_wait_return = no_wait + 1;
    verbose_enter_scroll();

    if lnum == 0 {
        smsg(0, c"Executing: %s".as_ptr(), cmd);
    } else {
        smsg(0, c"line %ld: %s".as_ptr(), lnum as std::ffi::c_long, cmd);
    }

    if msg_silent == 0 {
        msg_puts(c"\n".as_ptr());
    }

    verbose_leave_scroll();
    no_wait_return = no_wait;
}

// =============================================================================
// is_other_file
// =============================================================================

/// Rust implementation of is_other_file.
///
/// Returns 1 if the given fnum/ffname differs from the current buffer.
///
/// # Safety
///
/// `ffname` may be NULL or a valid null-terminated C string.
#[export_name = "is_other_file"]
pub unsafe extern "C" fn rs_is_other_file(fnum: c_int, ffname: *const c_char) -> c_int {
    if fnum != 0 {
        if fnum == nvim_docmd_get_curbuf_fnum() {
            return 0;
        }
        return 1;
    }

    if ffname.is_null() {
        return 1;
    }

    if *ffname == 0 {
        return 0;
    }

    // If file_id is not valid but sfname is set, compare by name
    if nvim_docmd_curbuf_file_id_valid() == 0 {
        let sfname = nvim_docmd_get_curbuf_sfname();
        if !sfname.is_null() && *sfname != 0 {
            return c_int::from(path_fnamecmp(ffname, sfname) != 0);
        }
    }

    otherfile(ffname)
}

// =============================================================================
// ex_redir
// =============================================================================

/// Rust implementation of ex_redir.
///
/// Handles `:redir` command to redirect output to file/register/variable.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_redir"]
pub unsafe extern "C" fn rs_ex_redir(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    let arg_start = (*eap).arg;
    if arg_start.is_null() {
        return;
    }

    // Check if arg == "END" (case-insensitive)
    let arg_cstr = std::ffi::CStr::from_ptr(arg_start);
    let arg_bytes = arg_cstr.to_bytes();

    if arg_bytes.eq_ignore_ascii_case(b"end") {
        nvim_docmd_close_redir();
        return;
    }

    let mut arg = arg_start;

    if *arg == b'>' as c_char {
        arg = arg.add(1);
        let mode = if *arg == b'>' as c_char {
            arg = arg.add(1);
            c"a".as_ptr()
        } else {
            c"w".as_ptr()
        };
        arg = skipwhite(arg);

        nvim_docmd_close_redir();

        // Expand environment variables and "~/".
        let fname = expand_env_save(arg);
        if fname.is_null() {
            return;
        }

        let forceit = c_int::from((*eap).forceit);
        let fd = open_exfile(fname, forceit, mode);
        xfree(fname as *mut c_void);
        nvim_docmd_set_redir_fd(fd);
    } else if *arg == b'@' as c_char {
        // redirect to a register a-z (resp. A-Z for appending)
        nvim_docmd_close_redir();
        arg = arg.add(1);

        if valid_yank_reg(*arg as c_int, true) && *arg != b'_' as c_char {
            let reg = *arg as c_int;
            arg = arg.add(1);

            if *arg == b'>' as c_char && *arg.add(1) == b'>' as c_char {
                // append
                arg = arg.add(2);
                nvim_docmd_set_redir_reg(reg);
            } else {
                // Can use both "@a" and "@a>".
                if *arg == b'>' as c_char {
                    arg = arg.add(1);
                }
                nvim_docmd_set_redir_reg(reg);
                // Make register empty when not using @A-@Z and the command is valid.
                if *arg == 0 && isupper(reg) == 0 {
                    write_reg_contents(reg, c"".as_ptr(), 0, 0i32);
                }
            }
        }

        if *arg != 0 {
            nvim_docmd_set_redir_reg(0);
            semsg(crate::errors::E_INVARG2_STR.as_ptr(), arg_start);
        }
    } else if *arg == b'=' as c_char && *arg.add(1) == b'>' as c_char {
        // redirect to a variable
        nvim_docmd_close_redir();
        arg = arg.add(2);

        let append = if *arg == b'>' as c_char {
            arg = arg.add(1);
            1
        } else {
            0
        };

        if var_redir_start(skipwhite(arg), append != 0) == OK {
            nvim_docmd_set_redir_vname(1);
        }
    } else {
        // TODO: redirect to a buffer
        semsg(crate::errors::E_INVARG2_STR.as_ptr(), arg_start);
    }

    // Make sure redirection is not off.
    if !nvim_docmd_get_redir_fd().is_null() || redir_reg != 0 || redir_vname {
        redir_off = false;
    }
}

/// Close active redirection.
///
/// Closes redirect-to-file, clears register redirect, and stops variable redirect.
///
/// # Safety
/// Accesses C globals (redir_fd, redir_reg, redir_vname). Must be called from C context.
#[export_name = "nvim_docmd_close_redir"]
pub unsafe extern "C" fn rs_close_redir_impl() {
    if !nvim_docmd_get_redir_fd().is_null() {
        nvim_docmd_fclose_redir_fd();
    }
    nvim_docmd_set_redir_reg(0);
    if nvim_docmd_get_redir_vname() != 0 {
        var_redir_stop();
        nvim_docmd_set_redir_vname(0);
    }
}

// =============================================================================
// ex_normal
// =============================================================================

/// Size of save_state_T (large enough for the C struct).
/// The actual C struct save_state_T is opaque; we allocate storage on the stack.
/// 4096 bytes is safe for the largest known save_state_T layout.
const SAVE_STATE_SIZE: usize = 4096;

/// Rust implementation of ex_normal.
///
/// Executes normal mode commands specified by `:normal`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_normal"]
pub unsafe extern "C" fn rs_ex_normal(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    // Check if we're in terminal mode
    if nvim_docmd_curbuf_has_terminal() != 0 && nvim_docmd_curwin_in_terminal_mode() != 0 {
        emsg(c"Can't re-enter normal mode from terminal mode".as_ptr());
        return;
    }

    if expr_map_locked() {
        emsg((&raw const e_secure).cast::<c_char>());
        return;
    }

    if crate::ex_normal_busy >= nvim_docmd_get_p_mmd() {
        emsg(c"E192: Recursive use of :normal too deep".as_ptr());
        return;
    }

    // vgetc() expects K_SPECIAL to have been escaped. Count extra bytes needed.
    let eap_arg = (*eap).arg;
    let mut arg: *mut c_char = std::ptr::null_mut();

    {
        let mut extra_len: usize = 0;
        let mut p = eap_arg;
        while *p != 0 {
            let char_len = nvim_docmd_utfc_ptr2len(p) as usize;
            if char_len > 1 {
                // Multi-byte character: scan trail bytes for K_SPECIAL
                let mut l = char_len - 1;
                let mut q = p.add(1);
                while l > 0 {
                    if *q == K_SPECIAL as c_char {
                        extra_len += 2;
                    }
                    q = q.add(1);
                    l -= 1;
                }
                p = p.add(char_len);
            } else {
                p = p.add(1);
            }
        }

        if extra_len > 0 {
            let orig_len = strlen(eap_arg);
            arg = xmalloc(orig_len + extra_len + 1) as *mut c_char;
            let mut dst_idx = 0usize;
            let mut p = eap_arg;
            while *p != 0 {
                let char_len = nvim_docmd_utfc_ptr2len(p) as usize;
                // Copy first byte
                *arg.add(dst_idx) = *p;
                dst_idx += 1;
                if char_len > 1 {
                    let mut l = char_len - 1;
                    let mut q = p.add(1);
                    while l > 0 {
                        *arg.add(dst_idx) = *q;
                        dst_idx += 1;
                        if *q == K_SPECIAL as c_char {
                            *arg.add(dst_idx) = KS_SPECIAL as c_char;
                            dst_idx += 1;
                            *arg.add(dst_idx) = KE_FILLER as c_char;
                            dst_idx += 1;
                        }
                        q = q.add(1);
                        l -= 1;
                    }
                    p = p.add(char_len);
                } else {
                    p = p.add(1);
                }
            }
            *arg.add(dst_idx) = 0;
        }
    }

    let busy = crate::ex_normal_busy;
    crate::ex_normal_busy = busy + 1;

    // Allocate save_state_T on the stack
    let mut save_state_buf = [0u8; SAVE_STATE_SIZE];
    let save_state = save_state_buf.as_mut_ptr() as *mut c_void;

    if save_current_state(save_state) {
        let addr_count = (*eap).addr_count;
        loop {
            if addr_count != 0 {
                let line1 = (*eap).line1;
                let curwin = nvim_get_curwin();
                nvim_docmd_set_curwin_cursor_lnum(line1);
                nvim_docmd_set_curwin_cursor_col(0);
                check_cursor_moved(curwin);
                (*eap).line1 = line1 + 1;
            }

            let cmd_to_run = if !arg.is_null() { arg } else { eap_arg };
            let forceit = (*eap).forceit != 0;
            let remap = if forceit { REMAP_NONE } else { REMAP_YES };
            exec_normal_cmd(cmd_to_run, remap, false);

            if addr_count == 0 {
                break;
            }
            let line1 = (*eap).line1;
            let line2 = (*eap).line2;
            if line1 > line2 || got_int {
                break;
            }
        }
    }

    // Might not return to the main loop when in an event handler.
    update_topline_cursor();
    restore_current_state(save_state);

    let busy = crate::ex_normal_busy;
    crate::ex_normal_busy = busy - 1;

    setmouse();
    ui_cursor_shape();

    if !arg.is_null() {
        xfree(arg as *mut c_void);
    }
}

// =============================================================================
// ex_filetype
// =============================================================================

/// Rust implementation of ex_filetype.
///
/// Handles `:filetype` command to enable/disable filetype detection.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_filetype"]
pub unsafe extern "C" fn rs_ex_filetype(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    let arg = (*eap).arg;
    if arg.is_null() {
        return;
    }

    if *arg == 0 {
        // Print current status
        let detect_str = tristate_str(nvim_docmd_get_filetype_detect());
        let plugin_val = nvim_docmd_get_filetype_plugin();
        let indent_val = nvim_docmd_get_filetype_indent();
        let detect_val = nvim_docmd_get_filetype_detect();

        let plugin_str = if plugin_val == K_TRUE {
            if detect_val == K_TRUE {
                c"ON".as_ptr()
            } else {
                c"(on)".as_ptr()
            }
        } else {
            c"OFF".as_ptr()
        };

        let indent_str = if indent_val == K_TRUE {
            if detect_val == K_TRUE {
                c"ON".as_ptr()
            } else {
                c"(on)".as_ptr()
            }
        } else {
            c"OFF".as_ptr()
        };

        smsg(
            0,
            c"filetype detection:%s  plugin:%s  indent:%s".as_ptr(),
            detect_str,
            plugin_str,
            indent_str,
        );
        return;
    }

    let mut p = arg;
    let mut plugin = false;
    let mut indent = false;

    // Accept "plugin" and "indent" in any order.
    loop {
        let remaining = std::ffi::CStr::from_ptr(p).to_bytes();
        if remaining.starts_with(b"plugin") {
            plugin = true;
            p = skipwhite(p.add(6));
            continue;
        }
        if remaining.starts_with(b"indent") {
            indent = true;
            p = skipwhite(p.add(6));
            continue;
        }
        break;
    }

    let p_bytes = std::ffi::CStr::from_ptr(p).to_bytes();
    const DIP_ALL: c_int = 0x01;

    if p_bytes == b"on" || p_bytes == b"detect" {
        let first_byte = p_bytes[0];
        if first_byte == b'o' || nvim_docmd_get_filetype_detect() != K_TRUE {
            source_runtime(c"filetype.lua filetype.vim".as_ptr(), DIP_ALL);
            nvim_docmd_set_filetype_detect(K_TRUE);
            if plugin {
                source_runtime(c"ftplugin.vim".as_ptr(), DIP_ALL);
                nvim_docmd_set_filetype_plugin(K_TRUE);
            }
            if indent {
                source_runtime(c"indent.vim".as_ptr(), DIP_ALL);
                nvim_docmd_set_filetype_indent(K_TRUE);
            }
        }
        if first_byte == b'd' {
            do_doautocmd(
                c"filetypedetect BufRead".as_ptr() as *mut c_char,
                true,
                std::ptr::null_mut(),
            );
            do_modelines(0);
        }
    } else if p_bytes == b"off" {
        if plugin || indent {
            if plugin {
                source_runtime(c"ftplugof.vim".as_ptr(), DIP_ALL);
                nvim_docmd_set_filetype_plugin(K_FALSE);
            }
            if indent {
                source_runtime(c"indoff.vim".as_ptr(), DIP_ALL);
                nvim_docmd_set_filetype_indent(K_FALSE);
            }
        } else {
            source_runtime(c"ftoff.vim".as_ptr(), DIP_ALL);
            nvim_docmd_set_filetype_detect(K_FALSE);
        }
    } else {
        semsg(crate::errors::E_INVARG2_STR.as_ptr(), p);
    }
}

/// Convert TriState int to ON/OFF string.
unsafe fn tristate_str(val: c_int) -> *const c_char {
    if val == K_TRUE {
        c"ON".as_ptr()
    } else {
        c"OFF".as_ptr()
    }
}

// =============================================================================
// ex_quit
// =============================================================================

/// Rust implementation of ex_quit.
///
/// Handles `:quit` command.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "ex_quit"]
pub unsafe extern "C" fn rs_ex_quit(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    if cmdwin_type != 0 {
        nvim_set_cmdwin_result(CTRL_C);
        return;
    }

    // Don't quit while editing the command line.
    if text_locked() {
        text_locked_msg();
        return;
    }

    // Find the target window.
    let wp = if (*eap).addr_count > 0 {
        let mut wnr = (*eap).line2;
        let mut wp = nvim_get_firstwin();
        while !nvim_win_get_next(wp).is_null() {
            wnr -= 1;
            if wnr <= 0 {
                break;
            }
            wp = nvim_win_get_next(wp);
        }
        wp
    } else {
        nvim_get_curwin()
    };

    // Refuse to quit when locked.
    if nvim_curbuf_locked() != 0 {
        return;
    }

    let forceit_bool = (*eap).forceit != 0;
    let forceit = c_int::from(forceit_bool);

    // Trigger QuitPre and maybe ExitPre
    if before_quit_autocmds(wp, false, forceit_bool) {
        return;
    }

    // If there is only one relevant window we will exit.
    if nvim_docmd_check_more(0, forceit) == OK && rs_only_one_window() != 0 {
        crate::exiting = true;
    }

    let buf = nvim_win_get_buffer(wp);
    let buf_hidden = nvim_ex2_buf_hide(buf);
    let p_awa = crate::p_awa as c_int;

    let check_flags = (if p_awa != 0 { CCGD_AW } else { 0 })
        | (if forceit_bool { CCGD_FORCEIT } else { 0 })
        | CCGD_EXCMD;

    let addr_count = (*eap).addr_count;

    if (!buf_hidden && check_changed(buf, check_flags))
        || nvim_docmd_check_more(1, forceit) != OK
        || (rs_only_one_window() != 0 && check_changed_any(forceit_bool, true))
    {
        crate::exiting = false;
    } else {
        // quit last window
        if rs_only_one_window() != 0 && nvim_docmd_one_window_p(addr_count) != 0 {
            getout(0);
        }
        crate::exiting = false;
        // close window; may free buffer
        let free_buf = !buf_hidden || forceit_bool;
        win_close(wp, free_buf, forceit_bool);
    }
}

// =============================================================================
// Phase 3: Filename Expansion and Edit Handlers
// =============================================================================

/// Rust implementation of changedir_func.
///
/// Changes the current directory with scope (global/tab/window) and handles
/// previous directory tracking.
///
/// # Safety
///
/// `new_dir` must be a valid C string or NULL.
#[export_name = "changedir_func"]
pub unsafe extern "C" fn rs_changedir_func(new_dir: *mut c_char, scope: c_int) -> bool {
    if new_dir.is_null() || nvim_allbuf_locked() {
        return false;
    }

    // ":cd -": Change to previous directory
    let mut new_dir = new_dir;
    let is_dash = std::ffi::CStr::from_ptr(new_dir as *const c_char).to_bytes() == b"-";
    if is_dash {
        let pdir = nvim_get_prevdir(scope);
        if pdir.is_null() {
            emsg(c"E186: No previous directory".as_ptr());
            return false;
        }
        new_dir = pdir;
    }

    // Get current directory into pdir (OS_DIRNAME returns 1 (OK) on success).
    let pdir = if nvim_os_dirname_namebuff() == 1 {
        xstrdup(nvim_get_namebuff() as *const c_char)
    } else {
        std::ptr::null_mut()
    };

    // For UNIX ":cd" means: go to home directory.
    // On other systems too if 'cdhome' is set.
    if *new_dir == 0 && nvim_get_p_cdh() != 0 {
        nvim_expand_env_home_namebuff();
        new_dir = nvim_get_namebuff();
    }

    let dir_differs = pdir.is_null() || nvim_pathcmp_unlen(pdir, new_dir, -1) != 0;
    if dir_differs {
        nvim_do_autocmd_dirchanged_manual_pre(new_dir, scope);
        if nvim_vim_chdir(new_dir) != 0 {
            emsg(crate::errors::gt(crate::errors::E_FAILED_STR.as_ptr()));
            xfree(pdir as *mut c_void);
            return false;
        }
    }

    nvim_set_prevdir(scope, pdir);
    nvim_post_chdir(scope, dir_differs);

    true
}

// =============================================================================
// Phase 3: Filename Expansion and Edit Handlers
// =============================================================================

/// Opaque handle to expand_T (for wildcard expansion).
type ExpandTHandle = *mut c_void;

/// EX_NOSPC = 0x010
const EX_NOSPC: u32 = 0x010;

extern "C" {
    // expand_filename helpers
    #[link_name = "skip_grep_pat"]
    fn rs_skip_grep_pat(eap: ExArgHandle) -> *mut c_char;
    fn nvim_path_has_wildcard(p: *const c_char) -> bool;
    fn nvim_is_expand_char(c: c_int) -> bool;
    fn nvim_eval_vars_wrap(
        eap: ExArgHandle,
        p: *mut c_char,
        srclenp: *mut usize,
        errormsgp: *mut *const c_char,
        escapedp: *mut c_int,
    ) -> *mut c_char;
    fn nvim_has_dollar_or_tilde(s: *const c_char) -> bool;
    #[link_name = "expand_env_save"]
    fn nvim_expand_env_save(s: *const c_char) -> *mut c_char;
    // repl_cmdline accessors
    fn nvim_docmd_get_do_ecmd_cmd_dollar() -> *mut c_char;
    fn nvim_repl_has_exclaim(s: *const c_char) -> bool;
    fn nvim_vim_strsave_escaped_shell(s: *const c_char) -> *mut c_char;
    fn nvim_vim_strsave_escaped_bang(s: *const c_char) -> *mut c_char;
    fn nvim_expand_env_esc_namebuff_notilde(str: *const c_char);
    fn nvim_backslash_halve(p: *mut c_char);
    fn nvim_ExpandT_size() -> usize;
    fn nvim_ExpandInit(xpc: ExpandTHandle);
    fn nvim_ExpandOne_files(
        xpc: ExpandTHandle,
        str: *const c_char,
        wildflags: c_int,
        icase: bool,
    ) -> *mut c_char;
    fn nvim_get_p_wic() -> c_int;
}

/// Rust implementation of repl_cmdline.
///
/// Replaces `src[0..srclen]` in `*cmdlinep` with `repl`, reallocating the
/// command line buffer and fixing up all pointers inside `eap` that referred
/// into the old buffer.
///
/// Returns a pointer into the new buffer pointing just after the replacement.
///
/// # Safety
///
/// All pointers must be valid. `src` must point into `*cmdlinep`.
#[export_name = "repl_cmdline"]
pub unsafe extern "C" fn rs_repl_cmdline(
    eap: ExArgHandle,
    src: *mut c_char,
    srclen: usize,
    repl: *mut c_char,
    cmdlinep: *mut *mut c_char,
) -> *mut c_char {
    let old_cmdline = *cmdlinep;
    let repl_len = strlen(repl as *const c_char);
    let offset = (src as usize).wrapping_sub(old_cmdline as usize);

    // Length of tail: everything after the replaced span
    let tail_start = src.add(srclen);
    let tail_len = strlen(tail_start as *const c_char);

    // Allocate new buffer: prefix + repl + tail + NUL + optional nextcmd + NUL
    let nextcmd = (*eap).nextcmd;
    let nextcmd_extra = if !nextcmd.is_null() {
        strlen(nextcmd as *const c_char) + 1
    } else {
        0
    };
    let new_len = offset + repl_len + tail_len + 2 + nextcmd_extra;
    let new_cmdline = xmalloc(new_len) as *mut c_char;

    // Copy prefix (before replacement)
    std::ptr::copy_nonoverlapping(old_cmdline, new_cmdline, offset);

    // Copy replacement
    std::ptr::copy_nonoverlapping(repl, new_cmdline.add(offset), repl_len);

    // Copy tail (after replaced span)
    let new_tail = new_cmdline.add(offset + repl_len);
    std::ptr::copy_nonoverlapping(tail_start, new_tail, tail_len + 1); // +1 for NUL

    // The return value: pointer just after the replacement in the new buffer
    let ret_src = new_tail;

    // Fix up nextcmd
    if !nextcmd.is_null() {
        let nc_offset = offset + repl_len + tail_len + 1;
        let nc_dst = new_cmdline.add(nc_offset);
        let nc_len = nextcmd_extra - 1;
        std::ptr::copy_nonoverlapping(nextcmd, nc_dst, nc_len + 1);
        (*eap).nextcmd = nc_dst;
    }

    // Fix up eap->cmd
    let old_cmd = (*eap).cmd;
    let cmd_offset = (old_cmd as usize).wrapping_sub(old_cmdline as usize);
    (*eap).cmd = new_cmdline.add(cmd_offset);
    // Fix up eap->arg
    let old_arg = (*eap).arg;
    let arg_offset = (old_arg as usize).wrapping_sub(old_cmdline as usize);
    (*eap).arg = new_cmdline.add(arg_offset);
    // Fix up eap->args[j]
    let argc = (*eap).argc;
    let args = (*eap).args;
    for j in 0..argc {
        let arg_j = *args.add(j);
        let arg_j_offset = (arg_j as usize).wrapping_sub(old_cmdline as usize);
        if offset >= arg_j_offset {
            // Replacement is after this arg: offset relative to start stays the same
            *args.add(j) = new_cmdline.add(arg_j_offset);
        } else {
            // Replacement is before this arg: shift by (repl_len - srclen)
            let new_offset = (arg_j_offset as isize + repl_len as isize - srclen as isize) as usize;
            *args.add(j) = new_cmdline.add(new_offset);
        }
    }

    // Fix up eap->do_ecmd_cmd (if set and not dollar_command)
    let do_ecmd_cmd = (*eap).do_ecmd_cmd;
    let dollar_cmd = nvim_docmd_get_do_ecmd_cmd_dollar();
    if !do_ecmd_cmd.is_null() && do_ecmd_cmd != dollar_cmd {
        let dec_offset = (do_ecmd_cmd as usize).wrapping_sub(old_cmdline as usize);
        (*eap).do_ecmd_cmd = new_cmdline.add(dec_offset);
    }

    // Free old command line and update cmdlinep
    xfree(old_cmdline as *mut c_void);
    *cmdlinep = new_cmdline;

    ret_src
}

/// Rust implementation of expand_filename.
///
/// Expands `%`, `#`, `<cword>` etc. in eap->arg, and optionally expands
/// wildcards if EX_NOSPC is set.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "expand_filename"]
pub unsafe extern "C" fn rs_expand_filename(
    eap: ExArgHandle,
    cmdlinep: *mut *mut c_char,
    errormsgp: *mut *const c_char,
) -> c_int {
    const OK_VAL: c_int = 1;
    const FAIL_VAL: c_int = 0;
    const WILD_LIST_NOTFOUND: c_int = 0x01;
    const WILD_NOERROR: c_int = 0x02;
    const WILD_ADD_SLASH: c_int = 0x04;
    // CMD_ constants for no-escape cases
    const CMD_BANG: c_int = 64;
    const CMD_GREP: c_int = 173;
    const CMD_GREPADD: c_int = 174;
    const CMD_LGREP: c_int = 238;
    const CMD_LGREPADD: c_int = 239;
    const CMD_LMAKE: c_int = 242;
    const CMD_MAKE: c_int = 269;
    const CMD_TERMINAL: c_int = 497;

    // Skip a regexp pattern for ":vimgrep[add] pat file..."
    let mut p: *mut c_char = rs_skip_grep_pat(eap);

    let has_wildcards = nvim_path_has_wildcard(p);

    while *p != 0 {
        // Skip over `=expr`, wildcards in it are not expanded.
        if *p == b'`' as c_char && *p.add(1) == b'=' as c_char {
            p = p.add(2);
            nvim_skip_expr_arg(&mut p);
            if *p == b'`' as c_char {
                p = p.add(1);
            }
            continue;
        }

        // Quick check for expansion chars (%, #, <).
        if !nvim_is_expand_char(*p as c_int) {
            p = p.add(1);
            continue;
        }

        let mut srclen: usize = 0;
        let mut escaped: c_int = 0;
        let repl = nvim_eval_vars_wrap(eap, p, &mut srclen, errormsgp, &mut escaped);
        if !(*errormsgp).is_null() {
            return FAIL_VAL;
        }
        if repl.is_null() {
            p = p.add(srclen);
            continue;
        }

        // Expand ~/file and $HOME/file in replacement.
        let repl = if nvim_has_dollar_or_tilde(repl as *const c_char) {
            let new_repl = nvim_expand_env_save(repl as *const c_char);
            xfree(repl as *mut c_void);
            new_repl
        } else {
            repl
        };

        let cmdidx = (*eap).cmdidx;
        let usefilter = (*eap).usefilter != 0;
        let argt = (*eap).argt;

        // Escape whitespace for non-shell commands.
        let repl = if !usefilter
            && escaped == 0
            && cmdidx != CMD_BANG
            && cmdidx != CMD_GREP
            && cmdidx != CMD_GREPADD
            && cmdidx != CMD_LGREP
            && cmdidx != CMD_LGREPADD
            && cmdidx != CMD_LMAKE
            && cmdidx != CMD_MAKE
            && cmdidx != CMD_TERMINAL
            && (argt & EX_NOSPC) == 0
        {
            let new_repl = nvim_vim_strsave_escaped_shell(repl as *const c_char);
            xfree(repl as *mut c_void);
            new_repl
        } else {
            repl
        };

        // Escape '!' for shell commands.
        let repl = if (usefilter || cmdidx == CMD_BANG || cmdidx == CMD_TERMINAL)
            && nvim_repl_has_exclaim(repl as *const c_char)
        {
            let new_repl = nvim_vim_strsave_escaped_bang(repl as *const c_char);
            xfree(repl as *mut c_void);
            new_repl
        } else {
            repl
        };

        p = rs_repl_cmdline(eap, p, srclen, repl, cmdlinep);
        xfree(repl as *mut c_void);
    }

    // One file argument: Expand wildcards.
    // Don't do this with ":r !command" or ":w !command".
    let argt = (*eap).argt;
    if (argt & EX_NOSPC) != 0 && ((*eap).usefilter == 0) {
        let mut has_wildcards = has_wildcards;

        // May expand environment variables.
        if has_wildcards {
            let arg = (*eap).arg;
            if nvim_has_dollar_or_tilde(arg as *const c_char) {
                nvim_expand_env_esc_namebuff_notilde(arg as *const c_char);
                let nb = nvim_get_namebuff();
                has_wildcards = nvim_path_has_wildcard(nb as *const c_char);
                let arglen = strlen(arg as *const c_char);
                rs_repl_cmdline(eap, arg, arglen, nb, cmdlinep);
            }
        }

        // Halve backslashes (Vi compatible). On Unix: only if no wildcards.
        let arg = (*eap).arg;
        #[cfg(unix)]
        if !has_wildcards {
            nvim_backslash_halve(arg);
        }
        #[cfg(not(unix))]
        nvim_backslash_halve(arg);

        if has_wildcards {
            let wildflags = WILD_LIST_NOTFOUND | WILD_NOERROR | WILD_ADD_SLASH;
            let icase = nvim_get_p_wic() != 0;
            let xpc_size = nvim_ExpandT_size();
            let xpc = xcalloc(1, xpc_size) as ExpandTHandle;
            nvim_ExpandInit(xpc);
            let arg = (*eap).arg;
            let p_result = nvim_ExpandOne_files(xpc, arg as *const c_char, wildflags, icase);
            xfree(xpc);

            if p_result.is_null() {
                return FAIL_VAL;
            }
            let arglen = strlen(arg as *const c_char);
            rs_repl_cmdline(eap, arg, arglen, p_result, cmdlinep);
            xfree(p_result as *mut c_void);
        }
    }

    OK_VAL
}

// =============================================================================
// Phase 1 (batch plan): Simple Ex Command Handlers
// =============================================================================

extern "C" {
    // Phase 1 C accessors
    static mut restarting: bool;
    fn nvim_docmd_set_no_hlsearch(flag: bool);
    fn nvim_docmd_clear_restart_edit();
    fn nvim_docmd_set_stop_insert_mode();
    fn nvim_docmd_clearmode();
    fn nvim_docmd_do_exbuffer_impl(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_mod(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_next(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_prev(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_rewind(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_last(eap: ExArgHandle);
    fn do_highlight(line: *const c_char, forceit: bool, init: bool);
    fn nvim_docmd_do_bang(addr_count: c_int, eap: ExArgHandle, forceit: bool);
    fn ml_preserve(buf: *mut c_void, message: bool, do_fsync: bool);
    fn u_redo(count: c_int);
    fn nvim_docmd_pum_make_popup(arg: *const c_char, forceit: bool);
    fn nvim_docmd_wundo(arg: *const c_char, forceit: bool);
    fn nvim_docmd_rundo(arg: *const c_char);
    fn tabpage_move(nr: c_int);
    fn nvim_docmd_checkpath(forceit: bool);
    fn nvim_set_ex_pressedreturn(val: bool);
}

/// ":buffer" -- delegates to do_exbuffer.
#[export_name = "ex_buffer"]
pub unsafe extern "C" fn rs_ex_buffer(eap: ExArgHandle) {
    nvim_docmd_do_exbuffer_impl(eap);
}

/// ":bmodified" and similar -- goto modified buffer.
#[export_name = "ex_bmodified"]
pub unsafe extern "C" fn rs_ex_bmodified(eap: ExArgHandle) {
    nvim_docmd_goto_buffer_mod(eap);
}

/// ":bnext" -- go to next buffer.
#[export_name = "ex_bnext"]
pub unsafe extern "C" fn rs_ex_bnext(eap: ExArgHandle) {
    nvim_docmd_goto_buffer_next(eap);
}

/// ":bprevious" -- go to previous buffer.
#[export_name = "ex_bprevious"]
pub unsafe extern "C" fn rs_ex_bprevious(eap: ExArgHandle) {
    nvim_docmd_goto_buffer_prev(eap);
}

/// ":brewind" / ":bfirst" -- go to first buffer.
#[export_name = "ex_brewind"]
pub unsafe extern "C" fn rs_ex_brewind(eap: ExArgHandle) {
    nvim_docmd_goto_buffer_rewind(eap);
}

/// ":blast" -- go to last buffer.
#[export_name = "ex_blast"]
pub unsafe extern "C" fn rs_ex_blast(eap: ExArgHandle) {
    nvim_docmd_goto_buffer_last(eap);
}

/// ":highlight" -- call do_highlight (including easter egg).
#[export_name = "ex_highlight"]
pub unsafe extern "C" fn rs_ex_highlight(eap: ExArgHandle) {
    let arg = (*eap).arg;
    let cmd = (*eap).cmd;
    if !arg.is_null() && *arg == 0 && !cmd.is_null() && *cmd.add(2) == b'!' as c_char {
        msg(c"Greetings, Vim user!".as_ptr(), 0);
    }
    do_highlight(arg, (*eap).forceit != 0, false);
}

/// not_exiting -- clear exiting flag (already exists as C function, not migrated).
/// not_restarting -- clear restarting flag.
#[export_name = "not_restarting"]
pub unsafe extern "C" fn rs_not_restarting() {
    restarting = false;
}

/// ":preserve" -- call ml_preserve.
#[export_name = "ex_preserve"]
pub unsafe extern "C" fn rs_ex_preserve(eap: ExArgHandle) {
    let _ = eap;
    ml_preserve(nvim_get_curbuf(), true, true);
}

/// ":redo" -- call u_redo(1).
#[export_name = "ex_redo"]
pub unsafe extern "C" fn rs_ex_redo(eap: ExArgHandle) {
    let _ = eap;
    u_redo(1);
}

/// ":!" -- call do_bang.
#[export_name = "ex_bang"]
pub unsafe extern "C" fn rs_ex_bang(eap: ExArgHandle) {
    let addr_count = (*eap).addr_count;
    let forceit = (*eap).forceit != 0;
    nvim_docmd_do_bang(addr_count, eap, forceit);
}

/// Command modifier used in the wrong context.
#[export_name = "ex_wrongmodifier"]
pub unsafe extern "C" fn rs_ex_wrongmodifier(eap: ExArgHandle) {
    (*eap).errmsg = crate::gt(crate::E_INVCMD_STR.as_ptr()) as *mut c_char;
}

/// ":nogui" -- set error message (Nvim has no built-in GUI).
#[export_name = "ex_nogui"]
pub unsafe extern "C" fn rs_ex_nogui(eap: ExArgHandle) {
    (*eap).errmsg = crate::gt(crate::E_NOGVIM_STR.as_ptr()) as *mut c_char;
}

/// ":popup" -- call pum_make_popup.
#[export_name = "ex_popup"]
pub unsafe extern "C" fn rs_ex_popup(eap: ExArgHandle) {
    let arg = (*eap).arg;
    let forceit = (*eap).forceit != 0;
    nvim_docmd_pum_make_popup(arg as *const c_char, forceit);
}

/// ":wundo" -- write undo file.
#[export_name = "ex_wundo"]
pub unsafe extern "C" fn rs_ex_wundo(eap: ExArgHandle) {
    let arg = (*eap).arg;
    let forceit = (*eap).forceit != 0;
    nvim_docmd_wundo(arg as *const c_char, forceit);
}

/// ":rundo" -- read undo file.
#[export_name = "ex_rundo"]
pub unsafe extern "C" fn rs_ex_rundo(eap: ExArgHandle) {
    let arg = (*eap).arg;
    nvim_docmd_rundo(arg as *const c_char);
}

/// ":tabmove" -- move tab page.
#[export_name = "ex_tabmove"]
pub unsafe extern "C" fn rs_ex_tabmove(eap: ExArgHandle) {
    let tab_number = crate::address::rs_get_tabpage_arg(eap);
    let errmsg = (*eap).errmsg;
    if errmsg.is_null() {
        tabpage_move(tab_number);
    }
}

/// set_no_hlsearch -- set the no_hlsearch flag.
#[export_name = "set_no_hlsearch"]
pub unsafe extern "C" fn rs_set_no_hlsearch(flag: bool) {
    nvim_docmd_set_no_hlsearch(flag);
}

/// ":nohlsearch" -- disable search highlighting.
#[export_name = "ex_nohlsearch"]
pub unsafe extern "C" fn rs_ex_nohlsearch(eap: ExArgHandle) {
    let _ = eap;
    nvim_docmd_set_no_hlsearch(true);
    redraw_all_later(UPD_SOME_VALID);
}

/// ":stopinsert" -- stop insert mode.
#[export_name = "ex_stopinsert"]
pub unsafe extern "C" fn rs_ex_stopinsert(eap: ExArgHandle) {
    let _ = eap;
    nvim_docmd_clear_restart_edit();
    nvim_docmd_set_stop_insert_mode();
    nvim_docmd_clearmode();
}

/// ":checkpath" -- find pattern in path.
#[export_name = "ex_checkpath"]
pub unsafe extern "C" fn rs_ex_checkpath(eap: ExArgHandle) {
    let forceit = (*eap).forceit != 0;
    nvim_docmd_checkpath(forceit);
}

/// ":psearch" -- preview search.
#[export_name = "ex_psearch"]
pub unsafe extern "C" fn rs_ex_psearch(eap: ExArgHandle) {
    g_do_tagpreview = p_pvh as c_int;
    // Call the Rust ex_findpat implementation directly.
    crate::cmd_impl::rs_ex_findpat(eap);
    g_do_tagpreview = 0;
}

/// set_pressedreturn -- set ex_pressedreturn flag.
#[export_name = "set_pressedreturn"]
pub unsafe extern "C" fn rs_set_pressedreturn(val: bool) {
    nvim_set_ex_pressedreturn(val);
}

// =============================================================================
// Phase 2 (batch plan): Medium Ex Command Handlers
// =============================================================================

extern "C" {
    // Phase 2 C accessors
    fn nvim_docmd_do_bufdel(
        command: c_int,
        arg: *const c_char,
        addr_count: c_int,
        start_bnr: c_int,
        end_bnr: c_int,
        forceit: c_int,
    ) -> *mut c_char;
    fn do_autocmd(eap: ExArgHandle, arg: *mut c_char, forceit: c_int);
    fn do_augroup(arg: *mut c_char, del_group: c_int);
    fn check_nomodeline(argp: *mut *mut c_char) -> bool;
    // Phase 22: before_quit_all, ex_range_without_command helpers
    fn nvim_docmd_get_exmode_plus() -> *mut c_char;
    static mut exmode_active: bool;
    fn invalid_range(eap: ExArgHandle) -> *mut c_char;
    fn correct_range(eap: ExArgHandle);
    // Phase 20: recover, winsize, setfiletype helpers
    static mut recoverymode: bool;
    fn nvim_docmd_setfname_curbuf(arg: *const c_char) -> c_int;
    fn ml_recover(checkext: bool);
    fn rs_ascii_isdigit(c: c_int) -> c_int;
    fn screen_resize(width: c_int, height: c_int);
    fn nvim_docmd_curbuf_get_did_filetype() -> bool;
    fn nvim_docmd_curbuf_set_did_filetype(val: bool);
    fn nvim_docmd_set_filetype_option(arg: *const c_char);
    // Phase 19: psearch, shada, folddo helpers
    static mut g_do_tagpreview: c_int;
    static mut p_pvh: std::ffi::c_long;
    static mut p_shada: *mut c_char;
    fn rs_shada_read_everything(fname: *const c_char, forceit: bool, missing_ok: bool) -> c_int;
    fn rs_shada_write_file(file: *const c_char, nomerge: bool) -> c_int;
    fn hasFolding(win: WinHandle, lnum: LinenrT, firstp: *mut LinenrT, lastp: *mut LinenrT)
        -> bool;
    fn ml_setmarked(lnum: LinenrT);
    fn ml_clearmarked();
    fn global_exe(cmd: *mut c_char);
}

/// DOBUF_* constants for do_bufdel.
const DOBUF_UNLOAD: c_int = 2;
const DOBUF_DEL: c_int = 3;
const DOBUF_WIPE: c_int = 4;

/// ":bunload" / ":bdelete" / ":bwipeout".
#[export_name = "ex_bunload"]
pub unsafe extern "C" fn rs_ex_bunload(eap: ExArgHandle) {
    let cmdidx = (*eap).cmdidx;
    let command = if cmdidx == CMD_BDELETE {
        DOBUF_DEL
    } else if cmdidx == CMD_BWIPEOUT {
        DOBUF_WIPE
    } else {
        DOBUF_UNLOAD
    };
    let arg = (*eap).arg;
    let addr_count = (*eap).addr_count;
    let line1 = (*eap).line1;
    let line2 = (*eap).line2;
    let forceit = (*eap).forceit != 0;
    let errmsg = nvim_docmd_do_bufdel(
        command,
        arg as *const c_char,
        addr_count,
        line1,
        line2,
        forceit as c_int,
    );
    (*eap).errmsg = errmsg;
}

/// ":autocmd" / ":augroup".
#[export_name = "ex_autocmd"]
pub unsafe extern "C" fn rs_ex_autocmd(eap: ExArgHandle) {
    let secure = crate::secure;
    if secure != 0 {
        crate::secure = 2;
        (*eap).errmsg = crate::gt(crate::E_CURDIR_STR.as_ptr()) as *mut c_char;
    } else {
        let cmdidx = (*eap).cmdidx;
        let arg = (*eap).arg;
        let forceit = (*eap).forceit != 0;
        if cmdidx == CMD_AUTOCMD {
            do_autocmd(eap, arg, forceit as c_int);
        } else {
            do_augroup(arg, forceit as c_int);
        }
    }
}

/// ":doautocmd".
#[export_name = "ex_doautocmd"]
pub unsafe extern "C" fn rs_ex_doautocmd(eap: ExArgHandle) {
    let mut arg = (*eap).arg;
    let call_do_modelines = check_nomodeline(&mut arg) as c_int;
    let mut did_aucmd = false;
    do_doautocmd(arg, false, &mut did_aucmd);
    if call_do_modelines != 0 && did_aucmd {
        do_modelines(0);
    }
}

/// ":quitall".
#[export_name = "ex_quitall"]
pub unsafe extern "C" fn rs_ex_quitall(eap: ExArgHandle) {
    if rs_before_quit_all(eap) == 0 {
        // FAIL
        return;
    }
    crate::exiting = true;
    let forceit = (*eap).forceit != 0;
    if !forceit && check_changed_any(false, false) {
        crate::exiting = false;
        return;
    }
    getout(0);
}

/// ":setfiletype [FALLBACK] {name}".
#[export_name = "ex_setfiletype"]
pub unsafe extern "C" fn rs_ex_setfiletype(eap: ExArgHandle) {
    if nvim_docmd_curbuf_get_did_filetype() {
        return;
    }
    let arg = (*eap).arg;
    const FALLBACK: &[u8] = b"FALLBACK ";
    // Check if arg starts with "FALLBACK "
    let (arg_to_set, is_fallback) = if !arg.is_null() && {
        let prefix = std::slice::from_raw_parts(arg as *const u8, FALLBACK.len());
        prefix == FALLBACK
    } {
        (arg.add(FALLBACK.len()), true)
    } else {
        (arg, false)
    };
    nvim_docmd_set_filetype_option(arg_to_set as *const c_char);
    if is_fallback {
        nvim_docmd_curbuf_set_did_filetype(false);
    }
}

/// ":rshada" / ":wshada".
#[export_name = "ex_shada"]
pub unsafe extern "C" fn rs_ex_shada(eap: ExArgHandle) {
    let save_shada = p_shada;
    if p_shada.is_null() || *p_shada == 0 {
        p_shada = c"'100".as_ptr() as *mut c_char;
    }
    let cmdidx = (*eap).cmdidx;
    let arg = (*eap).arg;
    let forceit = (*eap).forceit != 0;
    if cmdidx == CMD_RVIMINFO || cmdidx == CMD_RSHADA {
        rs_shada_read_everything(arg as *const c_char, forceit, false);
    } else {
        rs_shada_write_file(arg as *const c_char, forceit);
    }
    p_shada = save_shada;
}

/// ":folddo" / ":folddoclosed".
#[export_name = "ex_folddo"]
pub unsafe extern "C" fn rs_ex_folddo(eap: ExArgHandle) {
    let line1 = (*eap).line1;
    let line2 = (*eap).line2;
    let cmdidx = (*eap).cmdidx;
    let win = nvim_get_curwin();
    let mut lnum = line1;
    while lnum <= line2 {
        if hasFolding(win, lnum, std::ptr::null_mut(), std::ptr::null_mut())
            == (cmdidx == CMD_FOLDDOCLOSED)
        {
            ml_setmarked(lnum);
        }
        lnum += 1;
    }
    let arg = (*eap).arg;
    global_exe(arg);
    ml_clearmarked();
}

/// ":redrawtabline".
#[export_name = "ex_redrawtabline"]
pub unsafe extern "C" fn rs_ex_redrawtabline(_eap: ExArgHandle) {
    let r = RedrawingDisabled;
    let p = p_lz;
    RedrawingDisabled = 0;
    p_lz = false;
    draw_tabline();
    RedrawingDisabled = r;
    p_lz = p;
    ui_flush();
}

/// ":join".
#[export_name = "ex_join"]
pub unsafe extern "C" fn rs_ex_join(eap: ExArgHandle) {
    let line1 = (*eap).line1;
    let line2 = (*eap).line2;
    nvim_docmd_set_curwin_cursor_lnum(line1);
    let line2 = if line1 == line2 {
        if (*eap).addr_count >= 2 {
            return;
        }
        if line2 == nvim_docmd_get_curbuf_line_count() {
            beep_flush();
            return;
        }
        let new_line2 = line2 + 1;
        (*eap).line2 = new_line2;
        new_line2
    } else {
        line2
    };
    do_join(
        (line2 - line1 + 1) as usize,
        (*eap).forceit == 0,
        true,
        true,
        true,
    );
    beginline(BL_WHITE | BL_FIX);
    nvim_docmd_ex_may_print_impl(eap);
}

/// ":put".
#[export_name = "ex_put"]
pub unsafe extern "C" fn rs_ex_put(eap: ExArgHandle) {
    let mut line2 = (*eap).line2;
    let mut forceit = (*eap).forceit != 0;
    if line2 == 0 {
        line2 = 1;
        (*eap).line2 = line2;
        forceit = true;
        (*eap).forceit = (true) as c_int;
    }
    nvim_docmd_set_curwin_cursor_lnum(line2);
    check_cursor_col(nvim_get_curwin());
    let regname = (*eap).regname;
    let dir = if forceit { BACKWARD } else { FORWARD };
    do_put(
        regname,
        std::ptr::null_mut(),
        dir,
        1,
        PUT_LINE | PUT_CURSLINE,
    );
}

/// ":iput".
#[export_name = "ex_iput"]
pub unsafe extern "C" fn rs_ex_iput(eap: ExArgHandle) {
    let mut line2 = (*eap).line2;
    let mut forceit = (*eap).forceit != 0;
    if line2 == 0 {
        line2 = 1;
        (*eap).line2 = line2;
        forceit = true;
        (*eap).forceit = (true) as c_int;
    }
    nvim_docmd_set_curwin_cursor_lnum(line2);
    check_cursor_col(nvim_get_curwin());
    let regname = (*eap).regname;
    let dir = if forceit { BACKWARD } else { FORWARD };
    do_put(
        regname,
        std::ptr::null_mut(),
        dir,
        1,
        PUT_LINE | PUT_CURSLINE | PUT_FIXINDENT,
    );
}

/// ":=" (equal).
#[export_name = "ex_equal"]
pub unsafe extern "C" fn rs_ex_equal(eap: ExArgHandle) {
    let arg = (*eap).arg;
    if !arg.is_null() && (*arg != 0 && *arg as u8 != b'|') {
        ex_lua(eap);
    } else {
        let nextcmd = find_nextcmd(arg as *const c_char);
        (*eap).nextcmd = nextcmd;
        let line2 = (*eap).line2;
        smsg(0, c"%ld".as_ptr(), line2 as std::ffi::c_long);
    }
}

/// ":recover".
#[export_name = "ex_recover"]
pub unsafe extern "C" fn rs_ex_recover(eap: ExArgHandle) {
    recoverymode = true;
    let curbuf = nvim_get_curbuf();
    let p_awa = crate::p_awa as c_int;
    let forceit = (*eap).forceit != 0;
    let flags = (if p_awa != 0 { CCGD_AW } else { 0 })
        | CCGD_MULTWIN
        | (if forceit { CCGD_FORCEIT } else { 0 })
        | CCGD_EXCMD;
    if !check_changed(curbuf, flags) {
        let arg = (*eap).arg;
        if arg.is_null() || *arg == 0 || nvim_docmd_setfname_curbuf(arg as *const c_char) == OK {
            ml_recover(true);
        }
    }
    recoverymode = false;
}

// Phase 3: Larger Ex Command Handlers

extern "C" {
    fn nvim_docmd_get_argopt_name(idx: c_int) -> *mut c_char;
    // Phase 23: ex_edit helpers
    fn nvim_docmd_check_can_set_curbuf_forceit(forceit: bool) -> bool;
    fn nvim_docmd_bt_prompt_curbuf() -> bool;

    // Phase 21 helpers
    fn nvim_docmd_eval_to_string_g_colors_name() -> *mut c_char;
    fn load_colors(name: *mut c_char) -> c_int;
    fn nvim_docmd_curbuf_ml_empty() -> bool;
    fn os_breakcheck();
    fn nvim_docmd_get_curwin_cursor_pos(lnum: *mut c_int, col: *mut c_int, coladd: *mut c_int);
    fn nvim_docmd_set_curwin_cursor_pos(lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_docmd_get_last_chdir_reason() -> *const c_char;
    fn nvim_docmd_curwin_has_localdir() -> bool;
    fn nvim_docmd_curtab_has_localdir() -> bool;
    fn nvim_docmd_nth_window(nr: c_int) -> WinHandle;
    fn win_goto(wp: WinHandle);
    fn close_others(message: c_int, forceit: c_int);
    fn nvim_docmd_ex_win_close_impl(forceit: c_int, win: WinHandle, tp: *mut c_void);
    fn setmark(name: c_int) -> c_int;
    fn rs_print_line(lnum: LinenrT, use_number: c_int, list: c_int, first: c_int);
    static mut got_int: bool;
    static mut ex_no_reprint: bool;
    static e_argreq: c_char;
    static e_trailing_arg: c_char;
    static e_empty_buffer: c_char;
}

/// ":winsize" (obsolete).
#[export_name = "ex_winsize"]
pub unsafe extern "C" fn rs_ex_winsize(eap: ExArgHandle) {
    let mut arg = (*eap).arg;
    if rs_ascii_isdigit(*(arg as *const u8) as c_int) == 0 {
        semsg(crate::errors::E_INVARG2_STR.as_ptr(), arg as *const c_char);
        return;
    }
    let w = getdigits_int(&mut arg, false, 10);
    arg = skipwhite(arg as *const c_char);
    let p = arg;
    let h = getdigits_int(&mut arg, false, 10);
    if *(p as *const u8) != 0 && *(arg as *const u8) == 0 {
        screen_resize(w, h);
    } else {
        emsg(c"E465: :winsize requires two number arguments".as_ptr());
    }
}

/// ":colorscheme".
#[export_name = "ex_colorscheme"]
pub unsafe extern "C" fn rs_ex_colorscheme(eap: ExArgHandle) {
    let arg = (*eap).arg;
    if arg.is_null() || *(arg as *const u8) == 0 {
        let p = nvim_docmd_eval_to_string_g_colors_name();
        if !p.is_null() {
            msg(p as *const c_char, 0);
            xfree(p as *mut c_void);
        } else {
            msg(c"default".as_ptr(), 0);
        }
    } else if load_colors(arg as *mut c_char) == 0 {
        // FAIL = 0
        semsg(
            c"E185: Cannot find color scheme '%s'".as_ptr(),
            arg as *const c_char,
        );
    }
}

/// ":mark" / ":k".
#[export_name = "ex_mark"]
pub unsafe extern "C" fn rs_ex_mark(eap: ExArgHandle) {
    let arg = (*eap).arg;
    if arg.is_null() || *(arg as *const u8) == 0 {
        emsg(&e_argreq as *const c_char);
        return;
    }
    if *(arg.add(1) as *const u8) != 0 {
        semsg(&e_trailing_arg as *const c_char, arg as *const c_char);
        return;
    }
    let mut saved_lnum: c_int = 0;
    let mut saved_col: c_int = 0;
    let mut saved_coladd: c_int = 0;
    nvim_docmd_get_curwin_cursor_pos(&mut saved_lnum, &mut saved_col, &mut saved_coladd);
    nvim_docmd_set_curwin_cursor_lnum((*eap).line2);
    beginline(BL_WHITE | BL_FIX);
    if setmark(*(arg as *const u8) as c_int) == 0 {
        emsg(c"E191: Argument must be a letter or forward/backward quote".as_ptr());
    }
    nvim_docmd_set_curwin_cursor_pos(saved_lnum, saved_col, saved_coladd);
}

/// ":print" / ":list" / ":number".
#[export_name = "ex_print"]
pub unsafe extern "C" fn rs_ex_print(eap: ExArgHandle) {
    if nvim_docmd_curbuf_ml_empty() {
        emsg(&e_empty_buffer as *const c_char);
    } else {
        let line1 = (*eap).line1;
        let line2 = (*eap).line2;
        let cmdidx = (*eap).cmdidx;
        let flags = (*eap).flags;
        let mut line = line1;
        while line <= line2 && !unsafe { got_int } {
            rs_print_line(
                line,
                (cmdidx == CMD_NUMBER || cmdidx == CMD_POUND || (flags & EXFLAG_NR) != 0) as c_int,
                (cmdidx == CMD_LIST || (flags & EXFLAG_LIST) != 0) as c_int,
                (line == line1) as c_int,
            );
            line += 1;
            os_breakcheck();
        }
        setpcmark();
        nvim_docmd_set_curwin_cursor_lnum(line2);
        beginline(BL_SOL | BL_FIX);
    }
    ex_no_reprint = true;
}

/// Internal helper: print working directory.
unsafe fn do_ex_pwd() {
    if nvim_os_dirname_namebuff() == 1 {
        let namebuff = nvim_get_namebuff();
        if p_verbose > 0 {
            let context: *const c_char = if !nvim_docmd_get_last_chdir_reason().is_null() {
                nvim_docmd_get_last_chdir_reason()
            } else if nvim_docmd_curwin_has_localdir() {
                c"window".as_ptr()
            } else if nvim_docmd_curtab_has_localdir() {
                c"tabpage".as_ptr()
            } else {
                c"global".as_ptr()
            };
            smsg(0, c"[%s] %s".as_ptr(), context, namebuff);
        } else {
            msg(namebuff as *const c_char, 0);
        }
    } else {
        emsg(c"E187: Unknown".as_ptr());
    }
}

/// ":edit" / ":badd" / ":balt" / ":visual" / ":enew".
#[export_name = "ex_edit"]
pub unsafe extern "C" fn rs_ex_edit(eap: ExArgHandle) {
    let cmdidx = (*eap).cmdidx;
    let arg = (*eap).arg;
    let ffname = if cmdidx == CMD_ENEW {
        std::ptr::null()
    } else {
        arg as *const c_char
    };
    // Exclude commands which keep the window's current buffer
    if cmdidx != CMD_BADD
        && cmdidx != CMD_BALT
        && (rs_is_other_file(0, ffname) != 0
            && !nvim_docmd_check_can_set_curbuf_forceit((*eap).forceit != 0))
    {
        return;
    }
    // prevent use of :edit on prompt-buffers
    if nvim_docmd_bt_prompt_curbuf()
        && cmdidx == CMD_EDIT
        && (arg.is_null() || *(arg as *const u8) == 0)
    {
        emsg(c"cannot :edit a prompt buffer".as_ptr());
        return;
    }
    super::cmd_impl::rs_do_exedit_impl(eap, std::ptr::null_mut());
}

/// ":pwd".
#[export_name = "ex_pwd"]
pub unsafe extern "C" fn rs_ex_pwd(_eap: ExArgHandle) {
    do_ex_pwd();
}

/// ":only".
#[export_name = "ex_only"]
pub unsafe extern "C" fn rs_ex_only(eap: ExArgHandle) {
    let wp = if (*eap).addr_count > 0 {
        let line2 = (*eap).line2 as c_int;
        let mut wnr = line2;
        let mut wp = nvim_get_firstwin();
        while wnr > 1 {
            wnr -= 1;
            let next = nvim_win_get_next(wp);
            if next.is_null() {
                break;
            }
            wp = next;
        }
        wp
    } else {
        nvim_get_curwin()
    };
    let curwin = nvim_get_curwin();
    if wp != curwin {
        win_goto(wp);
    }
    close_others(1, (*eap).forceit as c_int);
}

/// ":close".
#[export_name = "ex_close"]
pub unsafe extern "C" fn rs_ex_close(eap: ExArgHandle) {
    if cmdwin_type != 0 {
        nvim_set_cmdwin_result(CTRL_C);
        return;
    }
    if text_locked() || nvim_curbuf_locked() != 0 {
        return;
    }
    if (*eap).addr_count == 0 {
        nvim_docmd_ex_win_close_impl(
            c_int::from((*eap).forceit),
            nvim_get_curwin(),
            std::ptr::null_mut(),
        );
    } else {
        let win = nvim_docmd_nth_window((*eap).line2 as c_int);
        nvim_docmd_ex_win_close_impl(c_int::from((*eap).forceit), win, std::ptr::null_mut());
    }
}

/// check_more: check if more files remain; returns OK (0) or FAIL (non-0).
#[export_name = "check_more"]
pub unsafe extern "C" fn rs_check_more(message: c_int, forceit: c_int) -> c_int {
    nvim_docmd_check_more(message, forceit)
}

/// before_quit_all: pre-quit-all checks.
#[export_name = "before_quit_all"]
pub unsafe extern "C" fn rs_before_quit_all(eap: ExArgHandle) -> c_int {
    if cmdwin_type != 0 {
        let forceit = (*eap).forceit != 0;
        nvim_set_cmdwin_result(if forceit { K_XF1 } else { K_XF2 });
        return 0; // FAIL
    }
    if text_locked() {
        text_locked_msg();
        return 0; // FAIL
    }
    let forceit = (*eap).forceit != 0;
    if before_quit_autocmds(nvim_get_curwin(), true, forceit) {
        return 0; // FAIL
    }
    OK
}

/// get_argopt_name: expansion for ++opt names.
#[export_name = "get_argopt_name"]
pub unsafe extern "C" fn rs_get_argopt_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    nvim_docmd_get_argopt_name(idx)
}

// Phase 4: Substantial Command Handlers

extern "C" {
    // Phase 23: ex_at helpers
    fn nvim_docmd_typebuf_tb_len() -> c_int;
    fn nvim_docmd_p_cpo_has_execbuf() -> bool;
    fn nvim_docmd_do_cmdline_getexline();
    fn do_execreg(regname: c_int, colon: c_int, addcr: c_int, silent: c_int) -> c_int;
    fn stuff_empty() -> c_int;
    static mut exec_from_reg: bool;

    // Phase 18: ex_exit, ex_resize, ex_cd helpers
    static mut KeyTyped: bool;
    static mut p_cdh: bool;
    static mut p_verbose: std::ffi::c_long;

    fn not_exiting();
    fn curbufIsChanged() -> bool;
    fn do_write(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_buf_hide_curwin() -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn rs_win_setwidth_win(width: c_int, wp: WinHandle);
    fn rs_win_setheight_win(height: c_int, wp: WinHandle);
    fn atol(s: *const c_char) -> std::ffi::c_long;
    fn changedir_func(new_dir: *const c_char, scope: c_int) -> bool;

    // Phase 17: tabclose, hide, wincmd, copymove helpers
    static mut postponed_split_flags: c_int;
    static mut postponed_split_tab: c_int;

    fn nvim_get_curbuf() -> *mut c_void;
    fn rs_find_tabpage(n: c_int) -> *mut c_void;
    fn nvim_docmd_is_only_tabpage() -> c_int;
    fn nvim_docmd_tabpage_close_impl(forceit: c_int);
    fn nvim_docmd_tabpage_close_other_impl(tp: *mut c_void, forceit: c_int);
    fn nvim_docmd_tabpage_is_current(tp: *mut c_void) -> c_int;
    fn nvim_docmd_get_cmdmod_cmod_split() -> c_int;
    fn nvim_docmd_get_cmdmod_cmod_tab() -> c_int;
    fn nvim_docmd_get_address_for_copymove(
        eap: ExArgHandle,
        errormsg: *mut *const c_char,
    ) -> LinenrT;
    fn get_flags(eap: ExArgHandle);
    fn u_clearline(buf: *mut c_void);
    fn rs_do_window(nchar: c_int, count: c_int, xchar: c_int);
    fn check_nextcmd(p: *const c_char) -> *mut c_char;
    fn rs_do_move(line1: LinenrT, line2: LinenrT, dest: LinenrT) -> c_int;
    fn rs_ex_copy(line1: LinenrT, line2: LinenrT, n: LinenrT);

    // Phase 14: redraw/startinsert helpers
    static mut RedrawingDisabled: c_int;
    static mut p_lz: bool;
    static mut cmdpreview: bool;
    static mut VIsual_active: bool;
    static mut redraw_cmdline: bool;
    static mut msg_didout: bool;
    static mut msg_col: c_int;
    static mut need_wait_return: bool;
    static mut need_maketitle: bool;
    static mut State: c_int;

    fn draw_tabline();
    fn validate_cursor(wp: WinHandle);
    fn update_topline(wp: WinHandle);
    fn redraw_all_later(type_: c_int);
    fn redraw_curbuf_later(type_: c_int);
    fn update_screen();
    fn maketitle();
    fn status_redraw_all();
    fn status_redraw_curbuf();
    fn redraw_statuslines();
    fn showmode();
    fn rs_set_cursor_for_append_to_line();
    fn nvim_docmd_set_curwin_curswant(val: c_int);

    // Phase 16: ex_put, ex_iput, ex_equal helpers
    fn do_put(regname: c_int, reg: *mut c_void, dir: c_int, count: c_int, flags: c_int);
    fn ex_lua(eap: ExArgHandle);
    fn find_nextcmd(p: *const c_char) -> *mut c_char;
    fn check_cursor_col(win: WinHandle);
}

/// do_put direction constants (from vim_defs.h).
const FORWARD: c_int = 1;
const BACKWARD: c_int = -1;

/// do_put flag constants (from register_defs.h).
const PUT_FIXINDENT: c_int = 1;
const PUT_CURSLINE: c_int = 4;
const PUT_LINE: c_int = 8;

/// CMD_ constants for Phase 17-19.
const CMD_MOVE: c_int = 271;
const CMD_LCD: c_int = 225;
const CMD_LCHDIR: c_int = 226;
const CMD_TCD: c_int = 447;
const CMD_TCHDIR: c_int = 448;
const CMD_RSHADA: c_int = 374;
const CMD_RVIMINFO: c_int = 380;
const CMD_FOLDDOCLOSED: c_int = 165;
const CMD_NUMBER: c_int = 303;
const CMD_POUND: c_int = 548;
const CMD_LIST: c_int = 210;
const BL_WHITE: c_int = 1;
const BL_FIX: c_int = 4;
const BL_SOL: c_int = 2;
const EXFLAG_LIST: c_int = 0x01;
const EXFLAG_NR: c_int = 0x02;
const CMD_PRINT: c_int = 317;
const EX_RANGE: u32 = 0x001;
const EX_TRLBAR: u32 = 0x100;
const EX_COUNT: u32 = 0x400;
const CMD_ENEW: c_int = 148;
const CMD_BADD: c_int = 23;
const CMD_BALT: c_int = 24;
const K_XF1: c_int = -14845;
const K_XF2: c_int = -15101;

/// ex_range_without_command: handle range-only commands.
#[export_name = "ex_range_without_command"]
pub unsafe extern "C" fn rs_ex_range_without_command(eap: ExArgHandle) -> *mut c_char {
    let mut errormsg: *mut c_char = std::ptr::null_mut();
    let cmd = (*eap).cmd;
    let exmode_plus_p1 = nvim_docmd_get_exmode_plus().add(1);
    if !cmd.is_null() && *(cmd as *const u8) == b'|' || (exmode_active && cmd != exmode_plus_p1) {
        (*eap).cmdidx = CMD_PRINT;
        (*eap).argt = EX_RANGE | EX_COUNT | EX_TRLBAR;
        let err = invalid_range(eap);
        if err.is_null() {
            correct_range(eap);
            rs_ex_print(eap);
        } else {
            errormsg = err;
        }
    } else if (*eap).addr_count != 0 {
        let line_count = nvim_docmd_get_curbuf_line_count();
        let line2 = (*eap).line2.min(line_count);
        (*eap).line2 = line2;
        if line2 < 0 {
            errormsg = crate::errors::E_INVRANGE_STR.as_ptr() as *mut c_char;
        } else {
            let new_lnum = if line2 == 0 { 1 } else { line2 };
            nvim_docmd_set_curwin_cursor_lnum(new_lnum);
            beginline(BL_SOL | BL_FIX);
        }
    }
    errormsg
}

/// ":tabclose".
#[export_name = "ex_tabclose"]
pub unsafe extern "C" fn rs_ex_tabclose(eap: ExArgHandle) {
    const K_IGNORE: c_int = -13821;
    if cmdwin_type != 0 {
        nvim_set_cmdwin_result(K_IGNORE);
        return;
    }

    if nvim_docmd_is_only_tabpage() != 0 {
        emsg(c"E784: Cannot close last tab page".as_ptr());
        return;
    }

    let tab_number = crate::address::rs_get_tabpage_arg(eap);
    if !(*eap).errmsg.is_null() {
        return;
    }

    let tp = rs_find_tabpage(tab_number);
    if tp.is_null() {
        beep_flush();
        return;
    }

    let forceit = (*eap).forceit as c_int;
    if nvim_docmd_tabpage_is_current(tp) == 0 {
        nvim_docmd_tabpage_close_other_impl(tp, forceit);
    } else if !text_locked() && nvim_curbuf_locked() == 0 {
        nvim_docmd_tabpage_close_impl(forceit);
    }
}

/// ":hide".
#[export_name = "ex_hide"]
pub unsafe extern "C" fn rs_ex_hide(eap: ExArgHandle) {
    if (*eap).skip != 0 {
        return;
    }
    let forceit = (*eap).forceit != 0;
    if (*eap).addr_count == 0 {
        win_close(nvim_get_curwin(), false, forceit);
    } else {
        let win = nvim_docmd_nth_window((*eap).line2 as c_int);
        win_close(win, false, forceit);
    }
}

/// ":exit" / ":xit" / ":wq".
#[export_name = "ex_exit"]
pub unsafe extern "C" fn rs_ex_exit(eap: ExArgHandle) {
    const CTRL_C: c_int = 3;
    const CMD_WQ: c_int = 531;
    const FAIL: c_int = 0;
    const OK: c_int = 1;

    if cmdwin_type != 0 {
        nvim_set_cmdwin_result(CTRL_C);
        return;
    }
    if text_locked() {
        text_locked_msg();
        return;
    }

    let forceit = (*eap).forceit != 0;
    // we plan to exit if there is only one relevant window
    if nvim_docmd_check_more(0, forceit as c_int) == OK && rs_only_one_window() != 0 {
        crate::exiting = true;
    }
    let cmdidx = (*eap).cmdidx;
    if ((cmdidx == CMD_WQ || curbufIsChanged()) && do_write(eap) == FAIL)
        || before_quit_autocmds(nvim_get_curwin(), false, forceit)
        || nvim_docmd_check_more(1, forceit as c_int) == FAIL
        || (rs_only_one_window() != 0 && check_changed_any(forceit, false))
    {
        not_exiting();
    } else {
        if rs_only_one_window() != 0 {
            getout(0);
        }
        not_exiting();
        let free_buf = nvim_docmd_buf_hide_curwin() == 0;
        win_close(nvim_get_curwin(), free_buf, forceit);
    }
}

/// ":resize".
#[export_name = "ex_resize"]
pub unsafe extern "C" fn rs_ex_resize(eap: ExArgHandle) {
    const WSP_VERT: c_int = 0x02;

    let mut wp = nvim_get_curwin();

    if (*eap).addr_count > 0 {
        let mut n = (*eap).line2;
        wp = nvim_get_firstwin();
        while !nvim_win_get_next(wp).is_null() {
            n -= 1;
            if n <= 0 {
                break;
            }
            wp = nvim_win_get_next(wp);
        }
    }

    let arg = (*eap).arg;
    let n_raw = atol(arg) as c_int;
    let cmod_split = nvim_docmd_get_cmdmod_cmod_split();
    if cmod_split & WSP_VERT != 0 {
        let n = if !arg.is_null() && (*(arg as *const u8) == b'-' || *(arg as *const u8) == b'+') {
            n_raw + nvim_win_get_w_width(wp)
        } else if n_raw == 0 && (arg.is_null() || *(arg as *const u8) == 0) {
            Columns
        } else {
            n_raw
        };
        rs_win_setwidth_win(n, wp);
    } else {
        let n = if !arg.is_null() && (*(arg as *const u8) == b'-' || *(arg as *const u8) == b'+') {
            n_raw + nvim_win_get_w_height(wp)
        } else if n_raw == 0 && (arg.is_null() || *(arg as *const u8) == 0) {
            Rows - 1
        } else {
            n_raw
        };
        rs_win_setheight_win(n, wp);
    }
}

/// ":cd" / ":tcd" / ":lcd" / ":chdir" etc.
#[export_name = "ex_cd"]
pub unsafe extern "C" fn rs_ex_cd(eap: ExArgHandle) {
    const CD_SCOPE_WINDOW: c_int = 0;
    const CD_SCOPE_TABPAGE: c_int = 1;
    const CD_SCOPE_GLOBAL: c_int = 2;

    let new_dir = (*eap).arg;
    if new_dir.is_null() || (*(new_dir as *const u8) == 0 && !p_cdh) {
        do_ex_pwd();
        return;
    }

    let cmdidx = (*eap).cmdidx;
    let scope = if cmdidx == CMD_TCD || cmdidx == CMD_TCHDIR {
        CD_SCOPE_TABPAGE
    } else if cmdidx == CMD_LCD || cmdidx == CMD_LCHDIR {
        CD_SCOPE_WINDOW
    } else {
        CD_SCOPE_GLOBAL
    };

    if changedir_func(new_dir as *const c_char, scope) && (KeyTyped || p_verbose >= 5) {
        do_ex_pwd();
    }
}

/// ":wincmd".
#[export_name = "ex_wincmd"]
pub unsafe extern "C" fn rs_ex_wincmd(eap: ExArgHandle) {
    let arg = (*eap).arg;
    if arg.is_null() {
        return;
    }
    let first_char = *(arg as *const u8);
    let mut xchar: c_int = 0; // NUL
    let p: *const c_char;
    if first_char == b'g' || first_char == 7 {
        // Ctrl_G = 7
        if *(arg.add(1) as *const u8) == 0 {
            // NUL
            emsg(crate::errors::E_INVARG_STR.as_ptr());
            return;
        }
        xchar = *(arg.add(1) as *const u8) as c_int;
        p = arg.add(2);
    } else {
        p = arg.add(1);
    }

    let nextcmd = check_nextcmd(p);
    (*eap).nextcmd = nextcmd as *mut c_char;
    let p2 = skipwhite(p);
    if *(p2 as *const u8) != 0 && *(p2 as *const u8) != b'"' && nextcmd.is_null() {
        emsg(crate::errors::E_INVARG_STR.as_ptr());
    } else if (*eap).skip == 0 {
        postponed_split_flags = nvim_docmd_get_cmdmod_cmod_split();
        postponed_split_tab = nvim_docmd_get_cmdmod_cmod_tab();
        let count = if (*eap).addr_count > 0 {
            (*eap).line2
        } else {
            0
        };
        rs_do_window(first_char as c_int, count, xchar);
        postponed_split_flags = 0;
        postponed_split_tab = 0;
    }
}

/// ":copy" / ":move".
#[export_name = "ex_copymove"]
pub unsafe extern "C" fn rs_ex_copymove(eap: ExArgHandle) {
    let mut errormsg: *const c_char = std::ptr::null();
    let n = nvim_docmd_get_address_for_copymove(eap, &mut errormsg);
    if (*eap).arg.is_null() {
        // error detected
        if !errormsg.is_null() {
            emsg(errormsg);
        }
        (*eap).nextcmd = std::ptr::null_mut();
        return;
    }
    get_flags(eap);

    const MAXLNUM: LinenrT = 0x7fffffff;
    let line_count = nvim_docmd_get_curbuf_line_count();
    if n == MAXLNUM || n < 0 || n > line_count {
        emsg(crate::errors::E_INVRANGE_STR.as_ptr());
        return;
    }

    let line1 = (*eap).line1;
    let line2 = (*eap).line2;
    let cmdidx = (*eap).cmdidx;
    if cmdidx == CMD_MOVE {
        if rs_do_move(line1, line2, n) == 0 {
            // FAIL
            return;
        }
    } else {
        rs_ex_copy(line1, line2, n);
    }
    u_clearline(nvim_get_curbuf());
    beginline(BL_SOL | BL_FIX);
    nvim_docmd_ex_may_print_impl(eap);
}

/// ":@" (execute register).
#[export_name = "ex_at"]
pub unsafe extern "C" fn rs_ex_at(eap: ExArgHandle) {
    let prev_len = nvim_docmd_typebuf_tb_len();

    nvim_docmd_set_curwin_cursor_lnum((*eap).line2);
    check_cursor_col(nvim_get_curwin());

    // Get the register name. No name means use the previous one.
    let arg = (*eap).arg;
    let c = if arg.is_null() || *(arg as *const u8) == 0 {
        b'@' as c_int
    } else {
        *(arg as *const u8) as c_int
    };

    // Put the register in the typeahead buffer with the "silent" flag.
    if do_execreg(c, 1, nvim_docmd_p_cpo_has_execbuf() as c_int, 1) == 0 {
        beep_flush();
        return;
    }

    let save_efr = exec_from_reg;
    exec_from_reg = true;

    // Execute from the typeahead buffer.
    // Continue until the stuff buffer is empty and all added characters
    // have been consumed.
    while stuff_empty() == 0 || nvim_docmd_typebuf_tb_len() > prev_len {
        nvim_docmd_do_cmdline_getexline();
    }

    exec_from_reg = save_efr;
}

/// ":earlier" / ":later".
#[export_name = "ex_later"]
pub unsafe extern "C" fn rs_ex_later(eap: ExArgHandle) {
    let mut count = 0i32;
    let mut sec = false;
    let mut file = false;
    let arg = (*eap).arg;
    let mut p = arg;
    if p.is_null() || *p == 0 {
        count = 1;
    } else if (*(p as *const u8)).is_ascii_digit() {
        let mut pp = p as *mut c_char;
        count = getdigits_int(&mut pp as *mut *mut c_char, false, 0);
        p = pp;
        match *(p as *const u8) {
            b's' => {
                p = p.add(1);
                sec = true;
            }
            b'm' => {
                p = p.add(1);
                sec = true;
                count *= 60;
            }
            b'h' => {
                p = p.add(1);
                sec = true;
                count *= 60 * 60;
            }
            b'd' => {
                p = p.add(1);
                sec = true;
                count *= 24 * 60 * 60;
            }
            b'f' => {
                p = p.add(1);
                file = true;
            }
            _ => {}
        }
    }
    if !p.is_null() && *p != 0 {
        semsg(crate::errors::E_INVARG2_STR.as_ptr(), arg);
    } else {
        let step = if (*eap).cmdidx == CMD_EARLIER {
            -count
        } else {
            count
        };
        undo_time(step, sec, file, false);
    }
}

/// ":redraw".
#[export_name = "ex_redraw"]
pub unsafe extern "C" fn rs_ex_redraw(eap: ExArgHandle) {
    if cmdpreview {
        return; // Ignore :redraw during 'inccommand' preview. #9777
    }
    let r = RedrawingDisabled;
    let p = p_lz;
    RedrawingDisabled = 0;
    p_lz = false;
    validate_cursor(nvim_get_curwin());
    update_topline(nvim_get_curwin());
    if (*eap).forceit != 0 {
        redraw_all_later(UPD_NOT_VALID);
        redraw_cmdline = true;
    } else if VIsual_active {
        redraw_curbuf_later(UPD_INVERTED);
    }
    update_screen();
    if need_maketitle {
        maketitle();
    }
    RedrawingDisabled = r;
    p_lz = p;
    // Reset msg_didout, so that a message that's there is overwritten.
    msg_didout = false;
    msg_col = 0;
    // No need to wait after an intentional redraw.
    need_wait_return = false;
    ui_flush();
}

/// ":redrawstatus".
#[export_name = "ex_redrawstatus"]
pub unsafe extern "C" fn rs_ex_redrawstatus(eap: ExArgHandle) {
    if cmdpreview {
        return; // Ignore :redrawstatus during 'inccommand' preview. #9777
    }
    let r = RedrawingDisabled;
    let p = p_lz;
    if (*eap).forceit != 0 {
        status_redraw_all();
    } else {
        status_redraw_curbuf();
    }
    RedrawingDisabled = 0;
    p_lz = false;
    if State & MODE_CMDLINE != 0 {
        redraw_statuslines();
    } else {
        if VIsual_active {
            redraw_curbuf_later(UPD_INVERTED);
        }
        update_screen();
    }
    RedrawingDisabled = r;
    p_lz = p;
    ui_flush();
}

/// ":startinsert" / ":startreplace" / ":startgreplace".
///
/// CMD_startinsert=431, CMD_startgreplace=432, CMD_startreplace=433
#[export_name = "ex_startinsert"]
pub unsafe extern "C" fn rs_ex_startinsert(eap: ExArgHandle) {
    const CMD_STARTGREPLACE: c_int = CMD_STARTINSERT + 1; // 432
                                                          // CMD_startreplace = 433

    let forceit = (*eap).forceit != 0;
    if forceit {
        // cursor line can be zero on startup
        let lnum = (*eap).line1;
        if lnum == 0 {
            nvim_docmd_set_curwin_cursor_lnum(1);
        }
        rs_set_cursor_for_append_to_line();
    }

    // Ignore the command when already in Insert mode.
    if State & MODE_INSERT != 0 {
        return;
    }

    let cmdidx = (*eap).cmdidx;
    // First assignment (matches the if/elseif/else in C)
    let restart_char = if cmdidx == CMD_STARTINSERT {
        b'a' as c_int
    } else if cmdidx == CMD_STARTGREPLACE {
        b'V' as c_int
    } else {
        // CMD_startreplace
        b'R' as c_int
    };
    // Override for non-forceit startinsert: 'a' becomes 'i'
    let restart_char = if !forceit && cmdidx == CMD_STARTINSERT {
        b'i' as c_int
    } else {
        restart_char
    };
    restart_edit = restart_char;

    if !forceit {
        nvim_docmd_set_curwin_curswant(0); // avoid MAXCOL
    }

    if VIsual_active {
        showmode();
    }
}

// =============================================================================
// Phase 3: Migrate medium-complexity command implementations
// =============================================================================

extern "C" {
    // Phase 3: new FFI functions
    fn ui_call_error_exit(status: c_int);
    fn win_float_remove(bang: bool, count: c_int);
    fn autowrite_all();
    fn may_trigger_vim_suspend_resume(suspend: bool);
    fn ui_call_suspend();
    fn ui_flush();
    fn nvim_tag_get_magic_overruled() -> c_int;
    fn nvim_tag_set_magic_overruled(val: c_int);
    fn ex_substitute(eap: ExArgHandle);
    fn ex_substitute_preview(
        eap: ExArgHandle,
        cmdpreview_ns: c_int,
        cmdpreview_bufnr: c_int,
    ) -> c_int;
    fn script_get(eap: ExArgHandle, lenp: *mut usize) -> *mut c_char;
}

/// not_exiting -- clear exiting flag.
#[export_name = "not_exiting"]
pub unsafe extern "C" fn rs_not_exiting() {
    crate::exiting = false;
}

/// ":cquit" -- quit with error code.
#[export_name = "ex_cquit"]
pub unsafe extern "C" fn rs_ex_cquit(eap: ExArgHandle) {
    let status = if (*eap).addr_count > 0 {
        (*eap).line2
    } else {
        1 // EXIT_FAILURE
    };
    ui_call_error_exit(status);
    getout(status);
}

/// ":fclose" -- remove floating window.
#[export_name = "ex_fclose"]
pub unsafe extern "C" fn rs_ex_fclose(eap: ExArgHandle) {
    win_float_remove((*eap).forceit != 0, (*eap).line1);
}

/// ex_ni -- command is not available in this version.
#[export_name = "ex_ni"]
pub unsafe extern "C" fn rs_ex_ni(eap: ExArgHandle) {
    if (*eap).skip == 0 {
        (*eap).errmsg = crate::errors::gt(crate::errors::E319_MSG_STR.as_ptr()) as *mut c_char;
    }
}

/// ex_script_ni -- not-implemented stub for script commands (skips <<EOF blocks).
#[export_name = "ex_script_ni"]
pub unsafe extern "C" fn rs_ex_script_ni(eap: ExArgHandle) {
    if (*eap).skip == 0 {
        rs_ex_ni(eap);
    } else {
        xfree(script_get(eap, std::ptr::null_mut()) as *mut c_void);
    }
}

/// ":stop" -- suspend Neovim.
#[export_name = "ex_stop"]
pub unsafe extern "C" fn rs_ex_stop(eap: ExArgHandle) {
    if (*eap).forceit == 0 {
        autowrite_all();
    }
    may_trigger_vim_suspend_resume(true);
    ui_call_suspend();
    ui_flush();
}

/// ":smagic" and ":snomagic" -- substitute with magic overrule.
#[export_name = "ex_submagic"]
pub unsafe extern "C" fn rs_ex_submagic(eap: ExArgHandle) {
    let saved = nvim_tag_get_magic_overruled();
    // OPTION_MAGIC_ON = 1, OPTION_MAGIC_OFF = 2
    let new_val = if (*eap).cmdidx == CMD_SMAGIC { 1 } else { 2 };
    nvim_tag_set_magic_overruled(new_val);
    ex_substitute(eap);
    nvim_tag_set_magic_overruled(saved);
}

/// ":smagic" and ":snomagic" preview callback.
#[export_name = "ex_submagic_preview"]
pub unsafe extern "C" fn rs_ex_submagic_preview(
    eap: ExArgHandle,
    cmdpreview_ns: c_int,
    cmdpreview_bufnr: c_int,
) -> c_int {
    let saved = nvim_tag_get_magic_overruled();
    let new_val = if (*eap).cmdidx == CMD_SMAGIC { 1 } else { 2 };
    nvim_tag_set_magic_overruled(new_val);
    let retv = ex_substitute_preview(eap, cmdpreview_ns, cmdpreview_bufnr);
    nvim_tag_set_magic_overruled(saved);
    retv
}

// =============================================================================
// find_cmdline_var -- Phase 4 pure Rust migration
// =============================================================================

/// Spec string table (matches SPEC_* enum in ex_docmd.c).
static SPEC_STRINGS: &[&[u8]] = &[
    b"%",        // SPEC_PERC = 0
    b"#",        // SPEC_HASH = 1
    b"<cword>",  // SPEC_CWORD = 2
    b"<cWORD>",  // SPEC_CCWORD = 3
    b"<cexpr>",  // SPEC_CEXPR = 4
    b"<cfile>",  // SPEC_CFILE = 5
    b"<sfile>",  // SPEC_SFILE = 6
    b"<slnum>",  // SPEC_SLNUM = 7
    b"<stack>",  // SPEC_STACK = 8
    b"<script>", // SPEC_SCRIPT = 9
    b"<afile>",  // SPEC_AFILE = 10
    b"<abuf>",   // SPEC_ABUF = 11
    b"<amatch>", // SPEC_AMATCH = 12
    b"<sflnum>", // SPEC_SFLNUM = 13
    b"<SID>",    // SPEC_SID = 14
];

/// Check `src` for starting with a special cmdline variable.
/// Returns the index of the match, or -1 if no match.
/// Sets `*usedlen` to the length of the matched spec string.
///
/// Matches C `find_cmdline_var()`.
#[export_name = "find_cmdline_var"]
pub unsafe extern "C" fn rs_find_cmdline_var(src: *const c_char, usedlen: *mut usize) -> isize {
    if src.is_null() {
        return -1;
    }
    let src_len = strlen(src);
    let src_bytes = std::slice::from_raw_parts(src as *const u8, src_len);

    for (i, spec) in SPEC_STRINGS.iter().enumerate() {
        if src_bytes.starts_with(spec) {
            if !usedlen.is_null() {
                *usedlen = spec.len();
            }
            return i as isize;
        }
    }
    -1
}

// =============================================================================
// Phase 5: ex_fold, ex_foldopen, ex_digraphs, ex_mode, ex_swapname
// =============================================================================

/// `:fold`: Create a manual fold.
///
/// Matches C `ex_fold()`.
#[export_name = "ex_fold"]
pub unsafe extern "C" fn rs_ex_fold(eap: ExArgHandle) {
    if rs_foldManualAllowed(true) != 0 {
        rs_foldCreate(nvim_get_curwin(), (*eap).line1, (*eap).line2);
    }
}

/// `:foldopen` / `:foldclose`: Open or close a fold.
///
/// Matches C `ex_foldopen()`.
#[export_name = "ex_foldopen"]
pub unsafe extern "C" fn rs_ex_foldopen(eap: ExArgHandle) {
    let opening = c_int::from((*eap).cmdidx == CMD_FOLDOPEN);
    let recurse = c_int::from((*eap).forceit);
    rs_opFoldRange((*eap).line1, (*eap).line2, opening, recurse, false);
}

/// `:digraphs`: List or add digraphs.
///
/// Matches C `ex_digraphs()`.
#[export_name = "ex_digraphs"]
pub unsafe extern "C" fn rs_ex_digraphs(eap: ExArgHandle) {
    let arg = (*eap).arg;
    if !arg.is_null() && *arg != 0 {
        putdigraph(arg);
    } else {
        rs_listdigraphs(c_int::from((*eap).forceit));
    }
}

/// `:mode`: Redraw the screen or report invalid argument.
///
/// Matches C `ex_mode()`.
#[export_name = "ex_mode"]
pub unsafe extern "C" fn rs_ex_mode(eap: ExArgHandle) {
    let arg = (*eap).arg;
    if arg.is_null() || *arg == 0 {
        nvim_docmd_set_must_redraw(UPD_CLEAR);
        rs_ex_redraw(eap);
    } else {
        emsg(crate::gt(crate::E_SCREENMODE_STR.as_ptr()));
    }
}

/// `:swapname`: Show the swap file name of the current buffer.
///
/// Matches C `ex_swapname()`.
#[export_name = "ex_swapname"]
pub unsafe extern "C" fn rs_ex_swapname(_eap: ExArgHandle) {
    let fname = nvim_docmd_get_curbuf_swapname();
    if fname.is_null() {
        msg(
            crate::errors::gt(crate::errors::E_NO_SWAP_FILE_STR.as_ptr()),
            0,
        );
    } else {
        msg(fname, 0);
    }
}

// =============================================================================
// Phase 6: ex_tabnext
// =============================================================================

/// `:tabnext`, `:tabprevious`, `:tabfirst`, `:tablast` etc.
///
/// Matches C `ex_tabnext()`.
#[export_name = "ex_tabnext"]
pub unsafe extern "C" fn rs_ex_tabnext(eap: ExArgHandle) {
    let cmdidx = (*eap).cmdidx;
    if cmdidx == CMD_TABFIRST || cmdidx == CMD_TABREWIND {
        goto_tabpage(1);
    } else if cmdidx == CMD_TABLAST {
        goto_tabpage(9999);
    } else if cmdidx == CMD_TABPREVIOUS || cmdidx == CMD_TABNEXT_BACKWARD {
        let arg = (*eap).arg;
        let tab_number = if !arg.is_null() && *arg != 0 {
            let mut errmsg_set: c_int = 0;
            let n = nvim_docmd_parse_tabnext_count(eap, &mut errmsg_set);
            if errmsg_set != 0 {
                return;
            }
            n
        } else if (*eap).addr_count == 0 {
            1
        } else {
            let n = (*eap).line2 as c_int;
            if n < 1 {
                (*eap).errmsg = crate::errors::E_INVRANGE_STR.as_ptr() as *mut c_char;
                return;
            }
            n
        };
        goto_tabpage(-tab_number);
    } else {
        // CMD_tabnext and everything else
        let tab_number = crate::address::rs_get_tabpage_arg(eap);
        if (*eap).errmsg.is_null() {
            goto_tabpage(tab_number);
        }
    }
}

// =============================================================================
// Phase 7: ex_undo
// =============================================================================

/// `:undo` -- undo last change(s), optionally to a target sequence number.
///
/// Matches C `ex_undo()`.
#[export_name = "ex_undo"]
pub unsafe extern "C" fn rs_ex_undo(eap: ExArgHandle) {
    if (*eap).addr_count != 1 {
        if (*eap).forceit != 0 {
            let _ = u_undo_and_forget(1, true); // :undo!
        } else {
            u_undo(1); // :undo
        }
        return;
    }

    let step = (*eap).line2;

    if (*eap).forceit != 0 {
        // :undo! N -- must go to an earlier change in the same branch
        if step >= nvim_curbuf_get_u_seq_cur() as LinenrT {
            emsg(crate::gt(crate::E_UNDOBANG_STR.as_ptr()));
            return;
        }
        let mut found: c_int = 0;
        let count = nvim_docmd_undo_count_steps(step, &mut found);
        if found == 0 {
            emsg(crate::gt(crate::E_UNDOBANG_STR.as_ptr()));
            return;
        }
        let _ = u_undo_and_forget(count, true);
    } else {
        // :undo N -- navigate undo tree to sequence N
        undo_time(step as c_int, false, false, true);
    }
}

// =============================================================================
// Phase 8: ex_sleep and do_sleep
// =============================================================================

/// Sleep for `msec` milliseconds, optionally hiding the cursor.
///
/// Matches C `do_sleep()`.
#[export_name = "do_sleep"]
pub unsafe extern "C" fn rs_do_sleep(msec: i64, hide_cursor: bool) {
    if hide_cursor {
        ui_busy_start();
    }
    ui_flush(); // flush before waiting
    nvim_docmd_loop_sleep(msec);
    // If CTRL-C interrupted the sleep, drop it from the input buffer.
    if got_int {
        vpeekc();
    }
    if hide_cursor {
        ui_busy_stop();
    }
}

/// `:sleep`: Sleep for a given time.
///
/// Matches C `ex_sleep()`.
#[export_name = "ex_sleep"]
pub unsafe extern "C" fn rs_ex_sleep(eap: ExArgHandle) {
    if nvim_docmd_cursor_valid_curwin() != 0 {
        nvim_docmd_setcursor_mayforce_curwin();
    }
    let len_base = (*eap).line2 as i64;
    let arg = (*eap).arg;
    let len = if arg.is_null() || *arg == 0 {
        // No suffix: interpret as seconds
        len_base * 1000
    } else if *arg == b'm' as c_char {
        // 'm' suffix: milliseconds
        len_base
    } else {
        semsg(crate::errors::E_INVARG2_STR.as_ptr(), arg as *const c_char);
        return;
    };
    rs_do_sleep(len, (*eap).forceit != 0);
}

// =============================================================================
// Phase 10: ex_operators (Phase 10 migration)
// =============================================================================

/// MotionType kMTLineWise = 1
const K_MT_LINEWISE: c_int = 1;

/// Operator type constants (from ops.h)
const OP_DELETE: c_int = 1;
const OP_YANK: c_int = 2;
const OP_LSHIFT: c_int = 4;
const OP_RSHIFT: c_int = 5;

/// Rust implementation of `ex_operators`.
///
/// Handles `:delete`, `:yank`, `:<`, and `:>` commands.
///
/// # Safety
///
/// `eap` must be a valid `exarg_T` pointer.
#[export_name = "ex_operators"]
pub unsafe extern "C" fn rs_ex_operators(eap: ExArgHandle) {
    let mut oa = OpargT::default();

    clear_oparg(&raw mut oa);
    oa.regname = (*eap).regname;
    oa.start.lnum = (*eap).line1;
    oa.end.lnum = (*eap).line2;
    oa.line_count = (*eap).line2 - (*eap).line1 + 1;
    oa.motion_type = K_MT_LINEWISE;
    nvim_set_virtual_op_false();

    let cmdidx = (*eap).cmdidx;
    if cmdidx != CMD_YANK {
        // position cursor for undo
        setpcmark();
        nvim_docmd_set_curwin_cursor_lnum((*eap).line1);
        beginline(BL_SOL | BL_FIX);
    }

    if VIsual_active {
        end_visual_mode();
    }

    if cmdidx == CMD_DELETE {
        oa.op_type = OP_DELETE;
        op_delete(&raw mut oa);
    } else if cmdidx == CMD_YANK {
        oa.op_type = OP_YANK;
        op_yank(&raw mut oa, true);
    } else {
        // CMD_rshift or CMD_lshift
        let p_rl = nvim_curwin_get_w_p_rl();
        if (cmdidx == CMD_RSHIFT) ^ (p_rl != 0) {
            oa.op_type = OP_RSHIFT;
        } else {
            oa.op_type = OP_LSHIFT;
        }
        op_shift(&raw mut oa, false, (*eap).amount);
    }
    nvim_set_virtual_op_none();
    nvim_docmd_ex_may_print_impl(eap);
}

// =============================================================================
// nvim_docmd_expand_sfile_impl - replace <sfile> in a command string
// =============================================================================

/// Replace all `<sfile>` occurrences in `arg` by evaluating them through
/// `eval_vars`. Returns a newly allocated string or NULL on
/// error.
///
/// # Safety
///
/// `arg` must be a valid null-terminated C string.
#[export_name = "nvim_docmd_expand_sfile_impl"]
pub unsafe extern "C" fn rs_expand_sfile_impl(arg: *const c_char) -> *mut c_char {
    use std::ptr;

    const SFILE: &[u8] = b"<sfile>";

    // Work with mutable raw pointers mirroring the C logic.
    // `result` always owns the current heap string; `p` scans through it.
    let mut result = xstrdup(arg);
    let mut offset: usize = 0; // byte offset into result where scanning continues

    loop {
        // Re-derive p each iteration so it stays valid after reallocations.
        let p = result.add(offset);
        if *p == 0 {
            break;
        }

        // Check for "<sfile>" prefix at p.
        let mut matches = true;
        for (i, &b) in SFILE.iter().enumerate() {
            if *p.add(i) as u8 != b {
                matches = false;
                break;
            }
        }

        if !matches {
            offset += 1;
            continue;
        }

        // Replace "<sfile>" with the evaluated expansion.
        let mut srclen: usize = 0;
        let mut errormsg: *const c_char = ptr::null();
        let repl = eval_vars(
            p,
            result,
            &raw mut srclen,
            ptr::null_mut(),
            &raw mut errormsg,
            ptr::null_mut(),
            true,
        );

        if !errormsg.is_null() {
            if *errormsg != 0 {
                emsg(errormsg);
            }
            xfree(result.cast());
            return ptr::null_mut();
        }

        if repl.is_null() {
            // no match (cannot happen per C comment)
            offset += srclen;
            continue;
        }

        let result_len = strlen(result);
        let repl_len = strlen(repl);
        let new_len = result_len - srclen + repl_len + 1;

        let newres = xmalloc(new_len).cast::<c_char>();
        // prefix before the match
        ptr::copy_nonoverlapping(result, newres, offset);
        // replacement text
        ptr::copy_nonoverlapping(repl, newres.add(offset), repl_len);
        // suffix after the match (including NUL)
        let suffix_len = result_len - offset - srclen + 1;
        ptr::copy_nonoverlapping(p.add(srclen), newres.add(offset + repl_len), suffix_len);

        xfree(repl.cast());
        xfree(result.cast());

        result = newres;
        offset += repl_len; // continue scanning after the inserted replacement
    }

    result
}
