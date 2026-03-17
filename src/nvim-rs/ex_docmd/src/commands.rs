//! Ex command handler implementations (Phase 1 migration).
//!
//! This module contains Rust implementations of ex command handlers
//! migrated from ex_docmd.c.

use std::ffi::{c_char, c_int, c_void};

use crate::ExArgHandle;

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

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Message functions
    fn msg(s: *const c_char, a: c_int);
    fn smsg(a: c_int, fmt: *const c_char, ...);
    fn semsg(fmt: *const c_char, ...);
    fn emsg(s: *const c_char);
    fn msg_puts(s: *const c_char);

    // Verbose message helpers
    fn verbose_enter_scroll();
    fn verbose_leave_scroll();
    fn nvim_get_msg_silent() -> c_int;
    static mut no_wait_return: c_int;

    // Error messages
    static e_secure: c_char;
    fn nvim_get_e_invarg2() -> *const c_char;

    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn xfree(p: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn isupper(c: c_int) -> c_int;

    // Character-level iteration helpers
    fn nvim_docmd_utfc_ptr2len(p: *const c_char) -> c_int;

    // eap accessors
    fn nvim_eap_get_arg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_get_cmdidx(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_forceit(eap: ExArgHandle) -> bool;
    fn nvim_eap_get_skip(eap: ExArgHandle) -> c_int;
    fn nvim_eap_get_line1(eap: ExArgHandle) -> LinenrT;
    fn nvim_eap_get_line2(eap: ExArgHandle) -> LinenrT;
    fn nvim_eap_get_addr_count(eap: ExArgHandle) -> c_int;
    fn nvim_eap_set_line1(eap: ExArgHandle, line: LinenrT);

    // Global state accessors
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_set_cmdwin_result(val: c_int);
    fn nvim_docmd_set_exiting(val: c_int);
    fn nvim_curbuf_locked() -> c_int;
    fn nvim_docmd_get_p_awa() -> c_int;

    // Redir accessors (redir_fd/reg/vname are in globals.h/message.c)
    fn nvim_docmd_get_redir_fd() -> *mut c_void;
    fn nvim_docmd_set_redir_fd(fd: *mut c_void);
    fn nvim_get_redir_reg() -> c_int;
    fn nvim_docmd_set_redir_reg(reg: c_int);
    fn nvim_get_redir_vname() -> c_int;
    fn nvim_docmd_set_redir_vname(val: c_int);
    fn nvim_set_redir_off(val: c_int);

    // Redir helpers
    fn nvim_docmd_close_redir();
    fn open_exfile(fname: *const c_char, forceit: c_int, mode: *const c_char) -> *mut c_void;
    fn expand_env_save(arg: *const c_char) -> *mut c_char;
    fn valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn write_reg_contents(regname: c_int, str_: *const c_char, len: isize, must_append: c_int);
    fn var_redir_start(name: *const c_char, append: bool) -> c_int;

    // ex_normal helpers
    fn nvim_docmd_get_ex_normal_busy() -> c_int;
    fn nvim_docmd_set_ex_normal_busy(val: c_int);
    fn nvim_docmd_get_p_mmd() -> c_int;
    fn nvim_docmd_get_got_int() -> c_int;
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
    fn nvim_docmd_get_filetype_file() -> *const c_char;
    fn nvim_docmd_get_ftplugin_file() -> *const c_char;
    fn nvim_docmd_get_indent_file() -> *const c_char;
    fn nvim_docmd_get_ftplugof_file() -> *const c_char;
    fn nvim_docmd_get_indoff_file() -> *const c_char;
    fn nvim_docmd_get_ftoff_file() -> *const c_char;
    fn nvim_docmd_get_dip_all() -> c_int;

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
    fn nvim_eap_get_argt(eap: ExArgHandle) -> u32;
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
    fn nvim_pathcmp_unlen(a: *const c_char, b: *const c_char) -> c_int;
    fn nvim_get_namebuff() -> *mut c_char;
    fn nvim_get_e_failed() -> *const c_char;
    fn xstrdup(str: *const c_char) -> *mut c_char;
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

    let skip = nvim_eap_get_skip(eap);
    if skip == 0 {
        return 0;
    }

    let cmdidx = nvim_eap_get_cmdidx(eap);

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

    if nvim_get_msg_silent() == 0 {
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

    let arg_start = nvim_eap_get_arg(eap);
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

        let forceit = c_int::from(nvim_eap_get_forceit(eap));
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
            semsg(nvim_get_e_invarg2(), arg_start);
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
        semsg(nvim_get_e_invarg2(), arg_start);
    }

    // Make sure redirection is not off.
    if !nvim_docmd_get_redir_fd().is_null()
        || nvim_get_redir_reg() != 0
        || nvim_get_redir_vname() != 0
    {
        nvim_set_redir_off(0);
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

    if nvim_docmd_get_ex_normal_busy() >= nvim_docmd_get_p_mmd() {
        emsg(c"E192: Recursive use of :normal too deep".as_ptr());
        return;
    }

    // vgetc() expects K_SPECIAL to have been escaped. Count extra bytes needed.
    let eap_arg = nvim_eap_get_arg(eap);
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

    let busy = nvim_docmd_get_ex_normal_busy();
    nvim_docmd_set_ex_normal_busy(busy + 1);

    // Allocate save_state_T on the stack
    let mut save_state_buf = [0u8; SAVE_STATE_SIZE];
    let save_state = save_state_buf.as_mut_ptr() as *mut c_void;

    if save_current_state(save_state) {
        let addr_count = nvim_eap_get_addr_count(eap);
        loop {
            if addr_count != 0 {
                let line1 = nvim_eap_get_line1(eap);
                let curwin = nvim_get_curwin();
                nvim_docmd_set_curwin_cursor_lnum(line1);
                nvim_docmd_set_curwin_cursor_col(0);
                check_cursor_moved(curwin);
                nvim_eap_set_line1(eap, line1 + 1);
            }

            let cmd_to_run = if !arg.is_null() { arg } else { eap_arg };
            let forceit = nvim_eap_get_forceit(eap);
            let remap = if forceit { REMAP_NONE } else { REMAP_YES };
            exec_normal_cmd(cmd_to_run, remap, false);

            if addr_count == 0 {
                break;
            }
            let line1 = nvim_eap_get_line1(eap);
            let line2 = nvim_eap_get_line2(eap);
            if line1 > line2 || nvim_docmd_get_got_int() != 0 {
                break;
            }
        }
    }

    // Might not return to the main loop when in an event handler.
    update_topline_cursor();
    restore_current_state(save_state);

    let busy = nvim_docmd_get_ex_normal_busy();
    nvim_docmd_set_ex_normal_busy(busy - 1);

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

    let arg = nvim_eap_get_arg(eap);
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
    let dip_all = nvim_docmd_get_dip_all();

    if p_bytes == b"on" || p_bytes == b"detect" {
        let first_byte = p_bytes[0];
        if first_byte == b'o' || nvim_docmd_get_filetype_detect() != K_TRUE {
            source_runtime(nvim_docmd_get_filetype_file(), dip_all);
            nvim_docmd_set_filetype_detect(K_TRUE);
            if plugin {
                source_runtime(nvim_docmd_get_ftplugin_file(), dip_all);
                nvim_docmd_set_filetype_plugin(K_TRUE);
            }
            if indent {
                source_runtime(nvim_docmd_get_indent_file(), dip_all);
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
                source_runtime(nvim_docmd_get_ftplugof_file(), dip_all);
                nvim_docmd_set_filetype_plugin(K_FALSE);
            }
            if indent {
                source_runtime(nvim_docmd_get_indoff_file(), dip_all);
                nvim_docmd_set_filetype_indent(K_FALSE);
            }
        } else {
            source_runtime(nvim_docmd_get_ftoff_file(), dip_all);
            nvim_docmd_set_filetype_detect(K_FALSE);
        }
    } else {
        semsg(nvim_get_e_invarg2(), p);
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

    if nvim_get_cmdwin_type() != 0 {
        nvim_set_cmdwin_result(CTRL_C);
        return;
    }

    // Don't quit while editing the command line.
    if text_locked() {
        text_locked_msg();
        return;
    }

    // Find the target window.
    let wp = if nvim_eap_get_addr_count(eap) > 0 {
        let mut wnr = nvim_eap_get_line2(eap);
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

    let forceit_bool = nvim_eap_get_forceit(eap);
    let forceit = c_int::from(forceit_bool);

    // Trigger QuitPre and maybe ExitPre
    if before_quit_autocmds(wp, false, forceit_bool) {
        return;
    }

    // If there is only one relevant window we will exit.
    if nvim_docmd_check_more(0, forceit) == OK && rs_only_one_window() != 0 {
        nvim_docmd_set_exiting(1);
    }

    let buf = nvim_win_get_buffer(wp);
    let buf_hidden = nvim_ex2_buf_hide(buf);
    let p_awa = nvim_docmd_get_p_awa();

    let check_flags = (if p_awa != 0 { CCGD_AW } else { 0 })
        | (if forceit_bool { CCGD_FORCEIT } else { 0 })
        | CCGD_EXCMD;

    let addr_count = nvim_eap_get_addr_count(eap);

    if (!buf_hidden && check_changed(buf, check_flags))
        || nvim_docmd_check_more(1, forceit) != OK
        || (rs_only_one_window() != 0 && check_changed_any(forceit_bool, true))
    {
        nvim_docmd_set_exiting(0);
    } else {
        // quit last window
        if rs_only_one_window() != 0 && nvim_docmd_one_window_p(addr_count) != 0 {
            getout(0);
        }
        nvim_docmd_set_exiting(0);
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

    let dir_differs = pdir.is_null() || nvim_pathcmp_unlen(pdir, new_dir) != 0;
    if dir_differs {
        nvim_do_autocmd_dirchanged_manual_pre(new_dir, scope);
        if nvim_vim_chdir(new_dir) != 0 {
            emsg(nvim_get_e_failed());
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
    fn nvim_expand_env_save(s: *const c_char) -> *mut c_char;
    fn nvim_eap_get_usefilter(eap: ExArgHandle) -> bool;
    // repl_cmdline accessors
    fn nvim_eap_get_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_cmd(eap: ExArgHandle, p: *mut c_char);
    fn nvim_eap_set_arg(eap: ExArgHandle, p: *mut c_char);
    fn nvim_eap_get_nextcmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_nextcmd(eap: ExArgHandle, p: *mut c_char);
    fn nvim_eap_get_argc(eap: ExArgHandle) -> usize;
    fn nvim_eap_get_args(eap: ExArgHandle) -> *mut *mut c_char;
    fn nvim_eap_get_do_ecmd_cmd(eap: ExArgHandle) -> *mut c_char;
    fn nvim_eap_set_do_ecmd_cmd(eap: ExArgHandle, p: *mut c_char);
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
    let nextcmd = nvim_eap_get_nextcmd(eap);
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
        nvim_eap_set_nextcmd(eap, nc_dst);
    }

    // Fix up eap->cmd
    let old_cmd = nvim_eap_get_cmd(eap);
    let cmd_offset = (old_cmd as usize).wrapping_sub(old_cmdline as usize);
    nvim_eap_set_cmd(eap, new_cmdline.add(cmd_offset));

    // Fix up eap->arg
    let old_arg = nvim_eap_get_arg(eap);
    let arg_offset = (old_arg as usize).wrapping_sub(old_cmdline as usize);
    nvim_eap_set_arg(eap, new_cmdline.add(arg_offset));

    // Fix up eap->args[j]
    let argc = nvim_eap_get_argc(eap);
    let args = nvim_eap_get_args(eap);
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
    let do_ecmd_cmd = nvim_eap_get_do_ecmd_cmd(eap);
    let dollar_cmd = nvim_docmd_get_do_ecmd_cmd_dollar();
    if !do_ecmd_cmd.is_null() && do_ecmd_cmd != dollar_cmd {
        let dec_offset = (do_ecmd_cmd as usize).wrapping_sub(old_cmdline as usize);
        nvim_eap_set_do_ecmd_cmd(eap, new_cmdline.add(dec_offset));
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

        let cmdidx = nvim_eap_get_cmdidx(eap);
        let usefilter = nvim_eap_get_usefilter(eap);
        let argt = nvim_eap_get_argt(eap);

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
    let argt = nvim_eap_get_argt(eap);
    if (argt & EX_NOSPC) != 0 && !nvim_eap_get_usefilter(eap) {
        let mut has_wildcards = has_wildcards;

        // May expand environment variables.
        if has_wildcards {
            let arg = nvim_eap_get_arg(eap);
            if nvim_has_dollar_or_tilde(arg as *const c_char) {
                nvim_expand_env_esc_namebuff_notilde(arg as *const c_char);
                let nb = nvim_get_namebuff();
                has_wildcards = nvim_path_has_wildcard(nb as *const c_char);
                let arglen = strlen(arg as *const c_char);
                rs_repl_cmdline(eap, arg, arglen, nb, cmdlinep);
            }
        }

        // Halve backslashes (Vi compatible). On Unix: only if no wildcards.
        let arg = nvim_eap_get_arg(eap);
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
            let arg = nvim_eap_get_arg(eap);
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
    fn nvim_docmd_not_restarting();
    fn nvim_docmd_set_no_hlsearch(flag: bool);
    fn nvim_docmd_clear_restart_edit();
    fn nvim_docmd_set_stop_insert_mode();
    fn nvim_docmd_clearmode();
    fn nvim_docmd_get_e_invcmd() -> *const c_char;
    fn nvim_eap_set_errmsg_const(eap: ExArgHandle, msg: *const c_char);
    fn nvim_eap_get_errmsg(eap: ExArgHandle) -> *mut c_char;
    fn nvim_docmd_do_exbuffer(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_mod(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_next(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_prev(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_rewind(eap: ExArgHandle);
    fn nvim_docmd_goto_buffer_last(eap: ExArgHandle);
    fn nvim_docmd_ex_highlight(eap: ExArgHandle);
    fn nvim_docmd_do_bang(addr_count: c_int, eap: ExArgHandle, forceit: bool);
    fn nvim_docmd_ml_preserve();
    fn nvim_docmd_u_redo();
    fn nvim_docmd_pum_make_popup(arg: *const c_char, forceit: bool);
    fn nvim_docmd_wundo(arg: *const c_char, forceit: bool);
    fn nvim_docmd_rundo(arg: *const c_char);
    fn nvim_docmd_get_tabpage_arg(eap: ExArgHandle) -> c_int;
    fn tabpage_move(nr: c_int);
    fn nvim_docmd_checkpath(forceit: bool);
    fn nvim_docmd_redraw_all_later_some_valid();
    fn nvim_docmd_set_pressedreturn(val: bool);
    fn nvim_docmd_ex_psearch(eap: ExArgHandle);
    fn nvim_docmd_get_e_nogvim() -> *const c_char;
}

/// ":buffer" -- delegates to do_exbuffer.
#[export_name = "ex_buffer"]
pub unsafe extern "C" fn rs_ex_buffer(eap: ExArgHandle) {
    nvim_docmd_do_exbuffer(eap);
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
    nvim_docmd_ex_highlight(eap);
}

/// not_exiting -- clear exiting flag (already exists as C function, not migrated).
/// not_restarting -- clear restarting flag.
#[export_name = "not_restarting"]
pub unsafe extern "C" fn rs_not_restarting() {
    nvim_docmd_not_restarting();
}

/// ":preserve" -- call ml_preserve.
#[export_name = "ex_preserve"]
pub unsafe extern "C" fn rs_ex_preserve(eap: ExArgHandle) {
    let _ = eap;
    nvim_docmd_ml_preserve();
}

/// ":redo" -- call u_redo(1).
#[export_name = "ex_redo"]
pub unsafe extern "C" fn rs_ex_redo(eap: ExArgHandle) {
    let _ = eap;
    nvim_docmd_u_redo();
}

/// ":!" -- call do_bang.
#[export_name = "ex_bang"]
pub unsafe extern "C" fn rs_ex_bang(eap: ExArgHandle) {
    let addr_count = nvim_eap_get_addr_count(eap);
    let forceit = nvim_eap_get_forceit(eap);
    nvim_docmd_do_bang(addr_count, eap, forceit);
}

/// Command modifier used in the wrong context.
#[export_name = "ex_wrongmodifier"]
pub unsafe extern "C" fn rs_ex_wrongmodifier(eap: ExArgHandle) {
    let msg = nvim_docmd_get_e_invcmd();
    nvim_eap_set_errmsg_const(eap, msg);
}

/// ":nogui" -- set error message (Nvim has no built-in GUI).
#[export_name = "ex_nogui"]
pub unsafe extern "C" fn rs_ex_nogui(eap: ExArgHandle) {
    let msg = nvim_docmd_get_e_nogvim();
    nvim_eap_set_errmsg_const(eap, msg);
}

/// ":popup" -- call pum_make_popup.
#[export_name = "ex_popup"]
pub unsafe extern "C" fn rs_ex_popup(eap: ExArgHandle) {
    let arg = nvim_eap_get_arg(eap);
    let forceit = nvim_eap_get_forceit(eap);
    nvim_docmd_pum_make_popup(arg as *const c_char, forceit);
}

/// ":wundo" -- write undo file.
#[export_name = "ex_wundo"]
pub unsafe extern "C" fn rs_ex_wundo(eap: ExArgHandle) {
    let arg = nvim_eap_get_arg(eap);
    let forceit = nvim_eap_get_forceit(eap);
    nvim_docmd_wundo(arg as *const c_char, forceit);
}

/// ":rundo" -- read undo file.
#[export_name = "ex_rundo"]
pub unsafe extern "C" fn rs_ex_rundo(eap: ExArgHandle) {
    let arg = nvim_eap_get_arg(eap);
    nvim_docmd_rundo(arg as *const c_char);
}

/// ":tabmove" -- move tab page.
#[export_name = "ex_tabmove"]
pub unsafe extern "C" fn rs_ex_tabmove(eap: ExArgHandle) {
    let tab_number = nvim_docmd_get_tabpage_arg(eap);
    let errmsg = nvim_eap_get_errmsg(eap);
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
    nvim_docmd_redraw_all_later_some_valid();
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
    let forceit = nvim_eap_get_forceit(eap);
    nvim_docmd_checkpath(forceit);
}

/// ":psearch" -- preview search (delegates to ex_psearch via C since it calls ex_findpat).
/// We set g_do_tagpreview and call ex_findpat indirectly.
#[export_name = "ex_psearch"]
pub unsafe extern "C" fn rs_ex_psearch(eap: ExArgHandle) {
    nvim_docmd_ex_psearch(eap);
}

/// set_pressedreturn -- set ex_pressedreturn flag.
#[export_name = "set_pressedreturn"]
pub unsafe extern "C" fn rs_set_pressedreturn(val: bool) {
    nvim_docmd_set_pressedreturn(val);
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
    fn nvim_docmd_do_autocmd(eap: ExArgHandle, arg: *const c_char, forceit: c_int);
    fn nvim_docmd_do_augroup(arg: *const c_char, forceit: c_int);
    fn nvim_docmd_get_e_curdir() -> *const c_char;
    fn nvim_get_secure() -> c_int;
    fn nvim_set_secure(val: c_int);
    fn nvim_docmd_check_nomodeline(argp: *mut *mut c_char) -> c_int;
    fn nvim_docmd_before_quit_all(eap: ExArgHandle) -> c_int;
    fn nvim_docmd_ex_shada(eap: ExArgHandle);
    fn nvim_docmd_ex_folddo(eap: ExArgHandle);
    fn nvim_docmd_ex_redrawtabline();
    fn nvim_docmd_ex_join(eap: ExArgHandle);
    fn nvim_docmd_ex_put(eap: ExArgHandle);
    fn nvim_docmd_ex_iput(eap: ExArgHandle);
    fn nvim_docmd_ex_equal(eap: ExArgHandle);
    fn nvim_docmd_ex_recover(eap: ExArgHandle);
    fn nvim_eap_set_errmsg(eap: ExArgHandle, msg: *mut c_char);
    fn nvim_docmd_ex_setfiletype(eap: ExArgHandle);
}

/// DOBUF_* constants for do_bufdel.
const DOBUF_UNLOAD: c_int = 2;
const DOBUF_DEL: c_int = 3;
const DOBUF_WIPE: c_int = 4;

/// ":bunload" / ":bdelete" / ":bwipeout".
#[export_name = "ex_bunload"]
pub unsafe extern "C" fn rs_ex_bunload(eap: ExArgHandle) {
    let cmdidx = nvim_eap_get_cmdidx(eap);
    let command = if cmdidx == CMD_BDELETE {
        DOBUF_DEL
    } else if cmdidx == CMD_BWIPEOUT {
        DOBUF_WIPE
    } else {
        DOBUF_UNLOAD
    };
    let arg = nvim_eap_get_arg(eap);
    let addr_count = nvim_eap_get_addr_count(eap);
    let line1 = nvim_eap_get_line1(eap);
    let line2 = nvim_eap_get_line2(eap);
    let forceit = nvim_eap_get_forceit(eap);
    let errmsg = nvim_docmd_do_bufdel(
        command,
        arg as *const c_char,
        addr_count,
        line1,
        line2,
        forceit as c_int,
    );
    nvim_eap_set_errmsg(eap, errmsg);
}

/// ":autocmd" / ":augroup".
#[export_name = "ex_autocmd"]
pub unsafe extern "C" fn rs_ex_autocmd(eap: ExArgHandle) {
    let secure = nvim_get_secure();
    if secure != 0 {
        nvim_set_secure(2);
        let e_curdir = nvim_docmd_get_e_curdir();
        nvim_eap_set_errmsg_const(eap, e_curdir);
    } else {
        let cmdidx = nvim_eap_get_cmdidx(eap);
        let arg = nvim_eap_get_arg(eap);
        let forceit = nvim_eap_get_forceit(eap);
        if cmdidx == CMD_AUTOCMD {
            nvim_docmd_do_autocmd(eap, arg as *const c_char, forceit as c_int);
        } else {
            nvim_docmd_do_augroup(arg as *const c_char, forceit as c_int);
        }
    }
}

/// ":doautocmd".
#[export_name = "ex_doautocmd"]
pub unsafe extern "C" fn rs_ex_doautocmd(eap: ExArgHandle) {
    let mut arg = nvim_eap_get_arg(eap);
    let call_do_modelines = nvim_docmd_check_nomodeline(&mut arg);
    let mut did_aucmd = false;
    do_doautocmd(arg, false, &mut did_aucmd);
    if call_do_modelines != 0 && did_aucmd {
        do_modelines(0);
    }
}

/// ":quitall".
#[export_name = "ex_quitall"]
pub unsafe extern "C" fn rs_ex_quitall(eap: ExArgHandle) {
    if nvim_docmd_before_quit_all(eap) == 0 {
        // FAIL
        return;
    }
    nvim_docmd_set_exiting(1);
    let forceit = nvim_eap_get_forceit(eap);
    if !forceit && check_changed_any(false, false) {
        nvim_docmd_set_exiting(0);
        return;
    }
    getout(0);
}

/// ":setfiletype [FALLBACK] {name}".
#[export_name = "ex_setfiletype"]
pub unsafe extern "C" fn rs_ex_setfiletype(eap: ExArgHandle) {
    nvim_docmd_ex_setfiletype(eap);
}

/// ":rshada" / ":wshada".
#[export_name = "ex_shada"]
pub unsafe extern "C" fn rs_ex_shada(eap: ExArgHandle) {
    nvim_docmd_ex_shada(eap);
}

/// ":folddo" / ":folddoclosed".
#[export_name = "ex_folddo"]
pub unsafe extern "C" fn rs_ex_folddo(eap: ExArgHandle) {
    nvim_docmd_ex_folddo(eap);
}

/// ":redrawtabline".
#[export_name = "ex_redrawtabline"]
pub unsafe extern "C" fn rs_ex_redrawtabline(eap: ExArgHandle) {
    let _ = eap;
    nvim_docmd_ex_redrawtabline();
}

/// ":join".
#[export_name = "ex_join"]
pub unsafe extern "C" fn rs_ex_join(eap: ExArgHandle) {
    nvim_docmd_ex_join(eap);
}

/// ":put".
#[export_name = "ex_put"]
pub unsafe extern "C" fn rs_ex_put(eap: ExArgHandle) {
    nvim_docmd_ex_put(eap);
}

/// ":iput".
#[export_name = "ex_iput"]
pub unsafe extern "C" fn rs_ex_iput(eap: ExArgHandle) {
    nvim_docmd_ex_iput(eap);
}

/// ":=" (equal).
#[export_name = "ex_equal"]
pub unsafe extern "C" fn rs_ex_equal(eap: ExArgHandle) {
    nvim_docmd_ex_equal(eap);
}

/// ":recover".
#[export_name = "ex_recover"]
pub unsafe extern "C" fn rs_ex_recover(eap: ExArgHandle) {
    nvim_docmd_ex_recover(eap);
}

// Phase 3: Larger Ex Command Handlers

extern "C" {
    fn nvim_docmd_ex_winsize(eap: ExArgHandle);
    fn nvim_docmd_ex_colorscheme(eap: ExArgHandle);
    fn nvim_docmd_ex_mark(eap: ExArgHandle);
    fn nvim_docmd_ex_print(eap: ExArgHandle);
    fn nvim_docmd_ex_edit(eap: ExArgHandle);
    fn nvim_docmd_ex_pwd(eap: ExArgHandle);
    fn nvim_docmd_ex_only(eap: ExArgHandle);
    fn nvim_docmd_ex_close(eap: ExArgHandle);
    fn nvim_docmd_get_argopt_name(idx: c_int) -> *mut c_char;
}

/// ":winsize" (obsolete).
#[export_name = "ex_winsize"]
pub unsafe extern "C" fn rs_ex_winsize(eap: ExArgHandle) {
    nvim_docmd_ex_winsize(eap);
}

/// ":colorscheme".
#[export_name = "ex_colorscheme"]
pub unsafe extern "C" fn rs_ex_colorscheme(eap: ExArgHandle) {
    nvim_docmd_ex_colorscheme(eap);
}

/// ":mark" / ":k".
#[export_name = "ex_mark"]
pub unsafe extern "C" fn rs_ex_mark(eap: ExArgHandle) {
    nvim_docmd_ex_mark(eap);
}

/// ":print" / ":list" / ":number".
#[export_name = "ex_print"]
pub unsafe extern "C" fn rs_ex_print(eap: ExArgHandle) {
    nvim_docmd_ex_print(eap);
}

/// ":edit" / ":badd" / ":balt" / ":visual" / ":enew".
#[export_name = "ex_edit"]
pub unsafe extern "C" fn rs_ex_edit(eap: ExArgHandle) {
    nvim_docmd_ex_edit(eap);
}

/// ":pwd".
#[export_name = "ex_pwd"]
pub unsafe extern "C" fn rs_ex_pwd(eap: ExArgHandle) {
    nvim_docmd_ex_pwd(eap);
}

/// ":only".
#[export_name = "ex_only"]
pub unsafe extern "C" fn rs_ex_only(eap: ExArgHandle) {
    nvim_docmd_ex_only(eap);
}

/// ":close".
#[export_name = "ex_close"]
pub unsafe extern "C" fn rs_ex_close(eap: ExArgHandle) {
    nvim_docmd_ex_close(eap);
}

/// check_more: check if more files remain; returns OK (0) or FAIL (non-0).
#[export_name = "check_more"]
pub unsafe extern "C" fn rs_check_more(message: c_int, forceit: c_int) -> c_int {
    nvim_docmd_check_more(message, forceit)
}

/// before_quit_all: pre-quit-all checks.
#[export_name = "before_quit_all"]
pub unsafe extern "C" fn rs_before_quit_all(eap: ExArgHandle) -> c_int {
    nvim_docmd_before_quit_all(eap)
}

/// get_argopt_name: expansion for ++opt names.
#[export_name = "get_argopt_name"]
pub unsafe extern "C" fn rs_get_argopt_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    nvim_docmd_get_argopt_name(idx)
}

// Phase 4: Substantial Command Handlers

extern "C" {
    fn nvim_docmd_ex_range_without_command(eap: ExArgHandle) -> *mut c_char;
    fn nvim_docmd_ex_tabclose(eap: ExArgHandle);
    fn nvim_docmd_ex_hide(eap: ExArgHandle);
    fn nvim_docmd_ex_exit(eap: ExArgHandle);
    fn nvim_docmd_ex_resize(eap: ExArgHandle);
    fn nvim_docmd_ex_cd(eap: ExArgHandle);
    fn nvim_docmd_ex_wincmd(eap: ExArgHandle);
    fn nvim_docmd_ex_copymove(eap: ExArgHandle);
    fn nvim_docmd_ex_at(eap: ExArgHandle);
    fn nvim_docmd_ex_later(eap: ExArgHandle);
    fn nvim_docmd_ex_redraw(eap: ExArgHandle);
    fn nvim_docmd_ex_redrawstatus(eap: ExArgHandle);
    fn nvim_docmd_ex_startinsert(eap: ExArgHandle);
}

/// ex_range_without_command: handle range-only commands.
#[export_name = "ex_range_without_command"]
pub unsafe extern "C" fn rs_ex_range_without_command(eap: ExArgHandle) -> *mut c_char {
    nvim_docmd_ex_range_without_command(eap)
}

/// ":tabclose".
#[export_name = "ex_tabclose"]
pub unsafe extern "C" fn rs_ex_tabclose(eap: ExArgHandle) {
    nvim_docmd_ex_tabclose(eap);
}

/// ":hide".
#[export_name = "ex_hide"]
pub unsafe extern "C" fn rs_ex_hide(eap: ExArgHandle) {
    nvim_docmd_ex_hide(eap);
}

/// ":exit" / ":xit" / ":wq".
#[export_name = "ex_exit"]
pub unsafe extern "C" fn rs_ex_exit(eap: ExArgHandle) {
    nvim_docmd_ex_exit(eap);
}

/// ":resize".
#[export_name = "ex_resize"]
pub unsafe extern "C" fn rs_ex_resize(eap: ExArgHandle) {
    nvim_docmd_ex_resize(eap);
}

/// ":cd" / ":tcd" / ":lcd" / ":chdir" etc.
#[export_name = "ex_cd"]
pub unsafe extern "C" fn rs_ex_cd(eap: ExArgHandle) {
    nvim_docmd_ex_cd(eap);
}

/// ":wincmd".
#[export_name = "ex_wincmd"]
pub unsafe extern "C" fn rs_ex_wincmd(eap: ExArgHandle) {
    nvim_docmd_ex_wincmd(eap);
}

/// ":copy" / ":move".
#[export_name = "ex_copymove"]
pub unsafe extern "C" fn rs_ex_copymove(eap: ExArgHandle) {
    nvim_docmd_ex_copymove(eap);
}

/// ":@" (execute register).
#[export_name = "ex_at"]
pub unsafe extern "C" fn rs_ex_at(eap: ExArgHandle) {
    nvim_docmd_ex_at(eap);
}

/// ":earlier" / ":later".
#[export_name = "ex_later"]
pub unsafe extern "C" fn rs_ex_later(eap: ExArgHandle) {
    nvim_docmd_ex_later(eap);
}

/// ":redraw".
#[export_name = "ex_redraw"]
pub unsafe extern "C" fn rs_ex_redraw(eap: ExArgHandle) {
    nvim_docmd_ex_redraw(eap);
}

/// ":redrawstatus".
#[export_name = "ex_redrawstatus"]
pub unsafe extern "C" fn rs_ex_redrawstatus(eap: ExArgHandle) {
    nvim_docmd_ex_redrawstatus(eap);
}

/// ":startinsert" / ":startreplace" / ":startgreplace".
#[export_name = "ex_startinsert"]
pub unsafe extern "C" fn rs_ex_startinsert(eap: ExArgHandle) {
    nvim_docmd_ex_startinsert(eap);
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
    fn nvim_docmd_e319_msg() -> *const c_char;
}

/// not_exiting -- clear exiting flag.
#[export_name = "not_exiting"]
pub unsafe extern "C" fn rs_not_exiting() {
    nvim_docmd_set_exiting(0);
}

/// ":cquit" -- quit with error code.
#[export_name = "ex_cquit"]
pub unsafe extern "C" fn rs_ex_cquit(eap: ExArgHandle) {
    let status = if nvim_eap_get_addr_count(eap) > 0 {
        nvim_eap_get_line2(eap)
    } else {
        1 // EXIT_FAILURE
    };
    ui_call_error_exit(status);
    getout(status);
}

/// ":fclose" -- remove floating window.
#[export_name = "ex_fclose"]
pub unsafe extern "C" fn rs_ex_fclose(eap: ExArgHandle) {
    win_float_remove(nvim_eap_get_forceit(eap), nvim_eap_get_line1(eap));
}

/// ex_ni -- command is not available in this version.
#[export_name = "ex_ni"]
pub unsafe extern "C" fn rs_ex_ni(eap: ExArgHandle) {
    if nvim_eap_get_skip(eap) == 0 {
        nvim_eap_set_errmsg_const(eap, nvim_docmd_e319_msg());
    }
}

/// ex_script_ni -- not-implemented stub for script commands (skips <<EOF blocks).
#[export_name = "ex_script_ni"]
pub unsafe extern "C" fn rs_ex_script_ni(eap: ExArgHandle) {
    if nvim_eap_get_skip(eap) == 0 {
        rs_ex_ni(eap);
    } else {
        xfree(script_get(eap, std::ptr::null_mut()) as *mut c_void);
    }
}

/// ":stop" -- suspend Neovim.
#[export_name = "ex_stop"]
pub unsafe extern "C" fn rs_ex_stop(eap: ExArgHandle) {
    if !nvim_eap_get_forceit(eap) {
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
    let new_val = if nvim_eap_get_cmdidx(eap) == CMD_SMAGIC {
        1
    } else {
        2
    };
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
    let new_val = if nvim_eap_get_cmdidx(eap) == CMD_SMAGIC {
        1
    } else {
        2
    };
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
