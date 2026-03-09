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

const CMD_ABOVELEFT: c_int = 3;
const CMD_AND: c_int = 549;
const CMD_BELOWRIGHT: c_int = 26;
const CMD_BOTRIGHT: c_int = 31;
const CMD_BROWSE: c_int = 38;
const CMD_CALL: c_int = 53;
const CMD_CATCH: c_int = 54;
const CMD_CONFIRM: c_int = 97;
const CMD_CONST: c_int = 99;
const CMD_DELFUNCTION: c_int = 115;
const CMD_DJUMP: c_int = 126;
const CMD_DLIST: c_int = 127;
const CMD_DSEARCH: c_int = 131;
const CMD_DSPLIT: c_int = 132;
const CMD_ECHO: c_int = 135;
const CMD_ECHOERR: c_int = 136;
const CMD_ECHOMSG: c_int = 138;
const CMD_ECHON: c_int = 139;
const CMD_ELSE: c_int = 140;
const CMD_ELSEIF: c_int = 141;
const CMD_ENDIF: c_int = 143;
const CMD_ENDFOR: c_int = 145;
const CMD_ENDTRY: c_int = 146;
const CMD_ENDWHILE: c_int = 147;
const CMD_EVAL: c_int = 149;
const CMD_EXECUTE: c_int = 151;
const CMD_FILTER: c_int = 157;
const CMD_FINALLY: c_int = 159;
const CMD_FOR: c_int = 167;
const CMD_FUNCTION: c_int = 168;
const CMD_HELP: c_int = 176;
const CMD_HIDE: c_int = 181;
const CMD_HORIZONTAL: c_int = 183;
const CMD_IF: c_int = 187;
const CMD_IJUMP: c_int = 188;
const CMD_ILIST: c_int = 189;
const CMD_ISEARCH: c_int = 198;
const CMD_ISPLIT: c_int = 199;
const CMD_KEEPALT: c_int = 209;
const CMD_KEEPJUMPS: c_int = 207;
const CMD_KEEPMARKS: c_int = 206;
const CMD_KEEPPATTERNS: c_int = 208;
const CMD_LEFTABOVE: c_int = 230;
const CMD_LET: c_int = 231;
const CMD_LOCKMARKS: c_int = 255;
const CMD_LOCKVAR: c_int = 256;
const CMD_LUA: c_int = 264;
const CMD_MATCH: c_int = 277;
const CMD_MZSCHEME: c_int = 287;
const CMD_NOAUTOCMD: c_int = 297;
const CMD_NOSWAPFILE: c_int = 301;
const CMD_PERL: c_int = 322;
const CMD_PSEARCH: c_int = 333;
const CMD_PYTHON: c_int = 345;
const CMD_PY3: c_int = 348;
const CMD_PYTHON3: c_int = 350;
const CMD_PYTHONX: c_int = 354;
const CMD_PYX: c_int = 352;
const CMD_RETURN: c_int = 370;
const CMD_RIGHTBELOW: c_int = 373;
const CMD_RUBY: c_int = 377;
const CMD_SILENT: c_int = 406;
const CMD_SMAGIC: c_int = 409;
const CMD_SNOMAGIC: c_int = 414;
const CMD_SUBSTITUTE: c_int = 381;
const CMD_SYNTAX: c_int = 443;
const CMD_TAB: c_int = 452;
const CMD_TCL: c_int = 467;
const CMD_THROW: c_int = 472;
const CMD_TILDE: c_int = 554;
const CMD_TOPLEFT: c_int = 483;
const CMD_TRY: c_int = 487;
const CMD_UNLET: c_int = 497;
const CMD_UNLOCKVAR: c_int = 498;
const CMD_VERBOSE: c_int = 505;
const CMD_VERTICAL: c_int = 506;
const CMD_WHILE: c_int = 524;
const CMD_WINCMD: c_int = 526;

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
    fn nvim_get_no_wait_return() -> c_int;
    fn nvim_set_no_wait_return(val: c_int);

    // Error messages
    fn nvim_get_e_secure() -> *const c_char;
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
    fn not_exiting();
    fn getout(exitval: c_int);
    fn win_close(wp: WinHandle, free_buf: bool, force: bool) -> c_int;
    fn nvim_docmd_one_window_p(addr_count: c_int) -> c_int;

    // is_other_file helpers
    fn nvim_docmd_get_curbuf_fnum() -> c_int;
    fn nvim_docmd_curbuf_file_id_valid() -> c_int;
    fn nvim_docmd_get_curbuf_sfname() -> *const c_char;
    fn path_fnamecmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn otherfile(fname: *const c_char) -> c_int;
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_msg_verbose_cmd(lnum: LinenrT, cmd: *const c_char) {
    let no_wait = nvim_get_no_wait_return();
    nvim_set_no_wait_return(no_wait + 1);
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
    nvim_set_no_wait_return(no_wait);
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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
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
        emsg(nvim_get_e_secure());
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
#[no_mangle]
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
#[no_mangle]
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
        not_exiting();
    } else {
        // quit last window
        if rs_only_one_window() != 0 && nvim_docmd_one_window_p(addr_count) != 0 {
            getout(0);
        }
        not_exiting();
        // close window; may free buffer
        let free_buf = !buf_hidden || forceit_bool;
        win_close(wp, free_buf, forceit_bool);
    }
}
