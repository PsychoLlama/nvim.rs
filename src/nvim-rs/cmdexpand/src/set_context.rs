//! Context dispatch for command-line completion by command name.
//!
//! Ports `set_context_by_cmdname` from `cmdexpand.c` to Rust.

#![allow(clippy::too_many_lines, non_upper_case_globals)]

use libc::{c_char, c_int, c_uint};

use crate::context::ExpandContext;
use crate::ExpandHandle;

// =============================================================================
// CMD_* constants from ex_cmds_enum.generated.h (positional enum values)
// =============================================================================

// NOTE: These values are derived from the generated C enum cmdidx_T.
// They must stay in sync with build/include/ex_cmds_enum.generated.h.

const CMD_abbreviate: c_int = 1;
const CMD_aboveleft: c_int = 3;
const CMD_amenu: c_int = 5;
const CMD_and: c_int = 549;
const CMD_anoremenu: c_int = 6;
const CMD_argdelete: c_int = 9;
const CMD_argdo: c_int = 10;
const CMD_augroup: c_int = 18;
const CMD_aunmenu: c_int = 19;
const CMD_autocmd: c_int = 17;
const CMD_bdelete: c_int = 25;
const CMD_belowright: c_int = 26;
const CMD_botright: c_int = 31;
const CMD_breakadd: c_int = 35;
const CMD_breakdel: c_int = 36;
const CMD_browse: c_int = 38;
const CMD_bufdo: c_int = 40;
const CMD_buffer: c_int = 20;
const CMD_bunload: c_int = 41;
const CMD_bwipeout: c_int = 42;
const CMD_cabbrev: c_int = 46;
const CMD_caddexpr: c_int = 50;
const CMD_call: c_int = 53;
const CMD_cdo: c_int = 62;
const CMD_cd: c_int = 61;
const CMD_cexpr: c_int = 64;
const CMD_cfdo: c_int = 66;
const CMD_cgetexpr: c_int = 70;
const CMD_chdir: c_int = 71;
const CMD_checkhealth: c_int = 73;
const CMD_checktime: c_int = 75;
const CMD_cmap: c_int = 81;
const CMD_cmapclear: c_int = 82;
const CMD_cmenu: c_int = 83;
const CMD_cnoreabbrev: c_int = 88;
const CMD_cnoremap: c_int = 87;
const CMD_cnoremenu: c_int = 89;
const CMD_colorscheme: c_int = 92;
const CMD_command: c_int = 93;
const CMD_compiler: c_int = 95;
const CMD_confirm: c_int = 97;
const CMD_const: c_int = 99;
const CMD_cunabbrev: c_int = 106;
const CMD_cunmap: c_int = 105;
const CMD_cunmenu: c_int = 107;
const CMD_debug: c_int = 111;
const CMD_delcommand: c_int = 114;
const CMD_delfunction: c_int = 115;
const CMD_diffget: c_int = 119;
const CMD_diffput: c_int = 122;
const CMD_djump: c_int = 126;
const CMD_dlist: c_int = 127;
const CMD_doautoall: c_int = 129;
const CMD_doautocmd: c_int = 128;
const CMD_dsearch: c_int = 131;
const CMD_dsplit: c_int = 132;
const CMD_echo: c_int = 135;
const CMD_echoerr: c_int = 136;
const CMD_echohl: c_int = 137;
const CMD_echomsg: c_int = 138;
const CMD_echon: c_int = 139;
const CMD_elseif: c_int = 141;
const CMD_emenu: c_int = 142;
const CMD_equal: c_int = 551;
const CMD_execute: c_int = 151;
const CMD_filetype: c_int = 156;
const CMD_filter: c_int = 157;
const CMD_find: c_int = 158;
const CMD_folddoclosed: c_int = 165;
const CMD_folddoopen: c_int = 164;
const CMD_for: c_int = 167;
const CMD_function: c_int = 168;
const CMD_global: c_int = 170;
const CMD_help: c_int = 176;
const CMD_hide: c_int = 181;
const CMD_highlight: c_int = 180;
const CMD_history: c_int = 182;
const CMD_horizontal: c_int = 183;
const CMD_iabbrev: c_int = 185;
const CMD_if: c_int = 187;
const CMD_ijump: c_int = 188;
const CMD_ilist: c_int = 189;
const CMD_imap: c_int = 190;
const CMD_imapclear: c_int = 191;
const CMD_imenu: c_int = 192;
const CMD_inoreabbrev: c_int = 194;
const CMD_inoremap: c_int = 193;
const CMD_inoremenu: c_int = 195;
const CMD_isearch: c_int = 198;
const CMD_isplit: c_int = 199;
const CMD_iunabbrev: c_int = 201;
const CMD_iunmap: c_int = 200;
const CMD_iunmenu: c_int = 202;
const CMD_keepalt: c_int = 209;
const CMD_keepjumps: c_int = 207;
const CMD_keepmarks: c_int = 206;
const CMD_keeppatterns: c_int = 208;
const CMD_laddexpr: c_int = 216;
const CMD_language: c_int = 215;
const CMD_lcd: c_int = 225;
const CMD_lchdir: c_int = 226;
const CMD_ldo: c_int = 228;
const CMD_leftabove: c_int = 230;
const CMD_let: c_int = 231;
const CMD_lexpr: c_int = 232;
const CMD_lfdo: c_int = 234;
const CMD_lgetexpr: c_int = 238;
const CMD_lmap: c_int = 246;
const CMD_lmapclear: c_int = 247;
const CMD_lnoremap: c_int = 249;
const CMD_lockmarks: c_int = 255;
const CMD_ltag: c_int = 262;
const CMD_lua: c_int = 264;
const CMD_lunmap: c_int = 263;
const CMD_map: c_int = 274;
const CMD_mapclear: c_int = 275;
const CMD_match: c_int = 277;
const CMD_menu: c_int = 278;
const CMD_messages: c_int = 280;
const CMD_nmap: c_int = 291;
const CMD_nmapclear: c_int = 292;
const CMD_nmenu: c_int = 293;
const CMD_nnoremap: c_int = 294;
const CMD_nnoremenu: c_int = 295;
const CMD_noautocmd: c_int = 297;
const CMD_noreabbrev: c_int = 299;
const CMD_noremap: c_int = 296;
const CMD_noremenu: c_int = 300;
const CMD_noswapfile: c_int = 301;
const CMD_nunmap: c_int = 304;
const CMD_nunmenu: c_int = 305;
const CMD_omap: c_int = 307;
const CMD_omapclear: c_int = 308;
const CMD_omenu: c_int = 309;
const CMD_onoremap: c_int = 311;
const CMD_onoremenu: c_int = 312;
const CMD_ounmap: c_int = 314;
const CMD_ounmenu: c_int = 315;
const CMD_ownsyntax: c_int = 316;
const CMD_packadd: c_int = 318;
const CMD_pbuffer: c_int = 320;
const CMD_popup: c_int = 327;
const CMD_profdel: c_int = 332;
const CMD_profile: c_int = 331;
const CMD_psearch: c_int = 333;
const CMD_ptag: c_int = 334;
const CMD_ptjump: c_int = 337;
const CMD_ptselect: c_int = 342;
const CMD_restart: c_int = 368;
const CMD_retab: c_int = 369;
const CMD_return: c_int = 370;
const CMD_rightbelow: c_int = 373;
const CMD_runtime: c_int = 375;
const CMD_sandbox: c_int = 385;
const CMD_sbuffer: c_int = 387;
const CMD_scriptnames: c_int = 396;
const CMD_set: c_int = 398;
const CMD_setfiletype: c_int = 399;
const CMD_setglobal: c_int = 400;
const CMD_setlocal: c_int = 401;
const CMD_sfind: c_int = 402;
const CMD_sign: c_int = 405;
const CMD_silent: c_int = 406;
const CMD_smap: c_int = 410;
const CMD_smapclear: c_int = 411;
const CMD_snoremap: c_int = 415;
const CMD_stag: c_int = 430;
const CMD_stjump: c_int = 435;
const CMD_stselect: c_int = 436;
const CMD_substitute: c_int = 381;
const CMD_sunmap: c_int = 438;
const CMD_syntax: c_int = 443;
const CMD_syntime: c_int = 444;
const CMD_tab: c_int = 452;
const CMD_tabdo: c_int = 454;
const CMD_tabfind: c_int = 456;
const CMD_tag: c_int = 450;
const CMD_tcd: c_int = 447;
const CMD_tchdir: c_int = 448;
const CMD_tjump: c_int = 473;
const CMD_tlmenu: c_int = 475;
const CMD_tlnoremenu: c_int = 476;
const CMD_tlunmenu: c_int = 477;
const CMD_tmenu: c_int = 478;
const CMD_topleft: c_int = 483;
const CMD_tselect: c_int = 488;
const CMD_tunmenu: c_int = 489;
const CMD_unabbreviate: c_int = 494;
const CMD_unlet: c_int = 497;
const CMD_unmap: c_int = 499;
const CMD_unmenu: c_int = 500;
const CMD_unsilent: c_int = 501;
const CMD_verbose: c_int = 505;
const CMD_vertical: c_int = 506;
const CMD_vglobal: c_int = 503;
const CMD_vmap: c_int = 512;
const CMD_vmapclear: c_int = 513;
const CMD_vmenu: c_int = 514;
const CMD_vnoremap: c_int = 515;
const CMD_vnoremenu: c_int = 517;
const CMD_vunmap: c_int = 519;
const CMD_vunmenu: c_int = 520;
const CMD_while: c_int = 524;
const CMD_windo: c_int = 527;
const CMD_xmap: c_int = 538;
const CMD_xmapclear: c_int = 539;
const CMD_xnoremap: c_int = 541;
const CMD_xunmap: c_int = 543;
// Special values (negative)
const CMD_USER: c_int = -1;
const CMD_USER_BUF: c_int = -2;

// =============================================================================
// Option flags
// =============================================================================

const OPT_GLOBAL: c_int = 0x01;
const OPT_LOCAL: c_int = 0x02;

// =============================================================================
// WOP flags
// =============================================================================

const K_OPT_WOP_FLAG_TAGFILE: c_uint = 0x02;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    fn nvim_get_wop_flags() -> c_uint;
    fn get_findfunc() -> *const c_char;

    // Context-setting functions still in C
    fn set_context_in_user_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn set_context_in_autocmd(xp: ExpandHandle, arg: *mut c_char, doautocmd: bool) -> *mut c_char;
    fn set_context_in_set_cmd(xp: ExpandHandle, arg: *mut c_char, opt_flags: c_int);
    fn set_context_in_syntax_cmd(xp: ExpandHandle, arg: *const c_char);
    fn set_context_in_echohl_cmd(xp: ExpandHandle, arg: *const c_char);
    fn set_context_in_highlight_cmd(xp: ExpandHandle, arg: *const c_char);
    fn set_context_in_sign_cmd(xp: ExpandHandle, arg: *mut c_char);
    fn set_context_in_map_cmd(
        xp: ExpandHandle,
        cmd: *mut c_char,
        arg: *mut c_char,
        forceit: c_int,
        isabbrev: c_int,
        isunmap: c_int,
        cmdidx: c_int,
    ) -> *mut c_char;
    fn set_context_in_menu_cmd(
        xp: ExpandHandle,
        cmd: *const c_char,
        arg: *mut c_char,
        forceit: bool,
    ) -> *mut c_char;
    fn set_context_in_runtime_cmd(xp: ExpandHandle, arg: *const c_char);
    fn set_context_in_profile_cmd(xp: ExpandHandle, arg: *const c_char);
    fn set_context_in_user_cmdarg(
        cmd: *const c_char,
        arg: *const c_char,
        argt: u32,
        context: c_int,
        xp: ExpandHandle,
        forceit: bool,
    ) -> *const c_char;
    fn set_context_for_expression(xp: ExpandHandle, arg: *mut c_char, cmdidx: c_int);

    // Already-ported Rust functions called from here
    fn rs_find_cmd_after_global_cmd(arg: *const c_char) -> *const c_char;
    fn rs_find_cmd_after_substitute_cmd(arg: *const c_char) -> *const c_char;
    fn rs_find_cmd_after_isearch_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_in_filter_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_in_match_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_in_unlet_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_in_lang_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_in_breakadd_cmd(
        xp: ExpandHandle,
        arg: *const c_char,
        breakpt_cmd_type: c_int,
    ) -> *const c_char;
    fn rs_set_context_in_scriptnames_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_in_filetype_cmd(xp: ExpandHandle, arg: *const c_char) -> *const c_char;
    fn rs_set_context_with_pattern(xp: ExpandHandle);

    // vim_strchr
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
}

// =============================================================================
// NUL sentinel
// =============================================================================

const NUL: c_char = 0;

// =============================================================================
// Implementation
// =============================================================================

/// Dispatch completion context setup based on the command name/index.
///
/// Ports the C `set_context_by_cmdname` static function to Rust.
///
/// # Safety
///
/// All pointers must be valid. `xp` must point to a valid `expand_T`.
/// `cmd` and `arg` must be valid null-terminated C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_set_context_by_cmdname(
    cmd: *const c_char,
    cmdidx: c_int,
    xp: ExpandHandle,
    arg: *const c_char,
    argt: u32,
    context: c_int,
    forceit: bool,
) -> *const c_char {
    let xp = &mut *xp;
    let xph = std::ptr::from_mut(xp);

    match cmdidx {
        // find/sfind/tabfind: promote EXPAND_FILES to EXPAND_FILES_IN_PATH or EXPAND_FINDFUNC
        c if c == CMD_find || c == CMD_sfind || c == CMD_tabfind => {
            if xp.xp_context == ExpandContext::Files as c_int {
                if *get_findfunc() == NUL {
                    xp.xp_context = ExpandContext::FilesInPath as c_int;
                } else {
                    xp.xp_context = ExpandContext::Findfunc as c_int;
                }
            }
        }

        // cd/chdir/lcd/lchdir/tcd/tchdir: promote EXPAND_FILES to EXPAND_DIRS_IN_CDPATH
        c if c == CMD_cd
            || c == CMD_chdir
            || c == CMD_lcd
            || c == CMD_lchdir
            || c == CMD_tcd
            || c == CMD_tchdir =>
        {
            if xp.xp_context == ExpandContext::Files as c_int {
                xp.xp_context = ExpandContext::DirsInCdpath as c_int;
            }
        }

        c if c == CMD_help => {
            xp.xp_context = ExpandContext::Help as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        // Command modifiers: return the argument unchanged (pass through to next cmd)
        c if c == CMD_aboveleft
            || c == CMD_argdo
            || c == CMD_belowright
            || c == CMD_botright
            || c == CMD_browse
            || c == CMD_bufdo
            || c == CMD_cdo
            || c == CMD_cfdo
            || c == CMD_confirm
            || c == CMD_debug
            || c == CMD_folddoclosed
            || c == CMD_folddoopen
            || c == CMD_hide
            || c == CMD_horizontal
            || c == CMD_keepalt
            || c == CMD_keepjumps
            || c == CMD_keepmarks
            || c == CMD_keeppatterns
            || c == CMD_ldo
            || c == CMD_leftabove
            || c == CMD_lfdo
            || c == CMD_lockmarks
            || c == CMD_noautocmd
            || c == CMD_noswapfile
            || c == CMD_restart
            || c == CMD_rightbelow
            || c == CMD_sandbox
            || c == CMD_silent
            || c == CMD_tab
            || c == CMD_tabdo
            || c == CMD_topleft
            || c == CMD_unsilent
            || c == CMD_verbose
            || c == CMD_vertical
            || c == CMD_windo =>
        {
            return arg;
        }

        c if c == CMD_filter => {
            return rs_set_context_in_filter_cmd(xph, arg);
        }

        c if c == CMD_match => {
            return rs_set_context_in_match_cmd(xph, arg);
        }

        c if c == CMD_command => {
            return set_context_in_user_cmd(xph, arg);
        }

        c if c == CMD_delcommand => {
            xp.xp_context = ExpandContext::UserCommands as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_global || c == CMD_vglobal => {
            let nextcmd = rs_find_cmd_after_global_cmd(arg);
            if nextcmd.is_null() && may_expand_pattern() {
                rs_set_context_with_pattern(xph);
            }
            return nextcmd;
        }

        c if c == CMD_and || c == CMD_substitute => {
            let nextcmd = rs_find_cmd_after_substitute_cmd(arg);
            if nextcmd.is_null() && may_expand_pattern() {
                rs_set_context_with_pattern(xph);
            }
            return nextcmd;
        }

        c if c == CMD_isearch
            || c == CMD_dsearch
            || c == CMD_ilist
            || c == CMD_dlist
            || c == CMD_ijump
            || c == CMD_psearch
            || c == CMD_djump
            || c == CMD_isplit
            || c == CMD_dsplit =>
        {
            return rs_find_cmd_after_isearch_cmd(xph, arg);
        }

        c if c == CMD_autocmd => {
            return set_context_in_autocmd(xph, arg.cast_mut(), false).cast_const();
        }

        c if c == CMD_doautocmd || c == CMD_doautoall => {
            return set_context_in_autocmd(xph, arg.cast_mut(), true).cast_const();
        }

        c if c == CMD_set => {
            set_context_in_set_cmd(xph, arg.cast_mut(), 0);
        }

        c if c == CMD_setglobal => {
            set_context_in_set_cmd(xph, arg.cast_mut(), OPT_GLOBAL);
        }

        c if c == CMD_setlocal => {
            set_context_in_set_cmd(xph, arg.cast_mut(), OPT_LOCAL);
        }

        c if c == CMD_tag
            || c == CMD_stag
            || c == CMD_ptag
            || c == CMD_ltag
            || c == CMD_tselect
            || c == CMD_stselect
            || c == CMD_ptselect
            || c == CMD_tjump
            || c == CMD_stjump
            || c == CMD_ptjump =>
        {
            let wop = nvim_get_wop_flags();
            if (wop & K_OPT_WOP_FLAG_TAGFILE) != 0 {
                xp.xp_context = ExpandContext::TagsListfiles as c_int;
            } else {
                xp.xp_context = ExpandContext::Tags as c_int;
            }
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_augroup => {
            xp.xp_context = ExpandContext::Augroup as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_syntax => {
            set_context_in_syntax_cmd(xph, arg);
        }

        c if c == CMD_const
            || c == CMD_let
            || c == CMD_if
            || c == CMD_elseif
            || c == CMD_while
            || c == CMD_for
            || c == CMD_echo
            || c == CMD_echon
            || c == CMD_execute
            || c == CMD_echomsg
            || c == CMD_echoerr
            || c == CMD_call
            || c == CMD_return
            || c == CMD_cexpr
            || c == CMD_caddexpr
            || c == CMD_cgetexpr
            || c == CMD_lexpr
            || c == CMD_laddexpr
            || c == CMD_lgetexpr =>
        {
            set_context_for_expression(xph, arg.cast_mut(), cmdidx);
        }

        c if c == CMD_unlet => {
            return rs_set_context_in_unlet_cmd(xph, arg);
        }

        c if c == CMD_function || c == CMD_delfunction => {
            xp.xp_context = ExpandContext::UserFunc as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_echohl => {
            set_context_in_echohl_cmd(xph, arg);
        }

        c if c == CMD_highlight => {
            set_context_in_highlight_cmd(xph, arg);
        }

        c if c == CMD_sign => {
            set_context_in_sign_cmd(xph, arg.cast_mut());
        }

        // bdelete/bwipeout/bunload: skip past all space-separated args
        c if c == CMD_bdelete || c == CMD_bwipeout || c == CMD_bunload => {
            let mut a = arg;
            loop {
                let sp = nvim_vim_strchr(a, c_int::from(b' '));
                if sp.is_null() {
                    break;
                }
                xp.xp_pattern = sp.cast_mut();
                a = sp.add(1);
            }
            xp.xp_context = ExpandContext::Buffers as c_int;
            xp.xp_pattern = a.cast_mut();
        }

        c if c == CMD_buffer || c == CMD_sbuffer || c == CMD_pbuffer || c == CMD_checktime => {
            xp.xp_context = ExpandContext::Buffers as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_diffget || c == CMD_diffput => {
            xp.xp_context = ExpandContext::DiffBuffers as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_USER || c == CMD_USER_BUF => {
            return set_context_in_user_cmdarg(cmd, arg, argt, context, xph, forceit);
        }

        c if c == CMD_map
            || c == CMD_noremap
            || c == CMD_nmap
            || c == CMD_nnoremap
            || c == CMD_vmap
            || c == CMD_vnoremap
            || c == CMD_omap
            || c == CMD_onoremap
            || c == CMD_imap
            || c == CMD_inoremap
            || c == CMD_cmap
            || c == CMD_cnoremap
            || c == CMD_lmap
            || c == CMD_lnoremap
            || c == CMD_smap
            || c == CMD_snoremap
            || c == CMD_xmap
            || c == CMD_xnoremap =>
        {
            return set_context_in_map_cmd(
                xph,
                cmd.cast_mut(),
                arg.cast_mut(),
                c_int::from(forceit),
                0,
                0,
                cmdidx,
            )
            .cast_const();
        }

        c if c == CMD_unmap
            || c == CMD_nunmap
            || c == CMD_vunmap
            || c == CMD_ounmap
            || c == CMD_iunmap
            || c == CMD_cunmap
            || c == CMD_lunmap
            || c == CMD_sunmap
            || c == CMD_xunmap =>
        {
            return set_context_in_map_cmd(
                xph,
                cmd.cast_mut(),
                arg.cast_mut(),
                c_int::from(forceit),
                0,
                1,
                cmdidx,
            )
            .cast_const();
        }

        c if c == CMD_mapclear
            || c == CMD_nmapclear
            || c == CMD_vmapclear
            || c == CMD_omapclear
            || c == CMD_imapclear
            || c == CMD_cmapclear
            || c == CMD_lmapclear
            || c == CMD_smapclear
            || c == CMD_xmapclear =>
        {
            xp.xp_context = ExpandContext::Mapclear as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_abbreviate
            || c == CMD_noreabbrev
            || c == CMD_cabbrev
            || c == CMD_cnoreabbrev
            || c == CMD_iabbrev
            || c == CMD_inoreabbrev =>
        {
            return set_context_in_map_cmd(
                xph,
                cmd.cast_mut(),
                arg.cast_mut(),
                c_int::from(forceit),
                1,
                0,
                cmdidx,
            )
            .cast_const();
        }

        c if c == CMD_unabbreviate || c == CMD_cunabbrev || c == CMD_iunabbrev => {
            return set_context_in_map_cmd(
                xph,
                cmd.cast_mut(),
                arg.cast_mut(),
                c_int::from(forceit),
                1,
                1,
                cmdidx,
            )
            .cast_const();
        }

        c if c == CMD_menu
            || c == CMD_noremenu
            || c == CMD_unmenu
            || c == CMD_amenu
            || c == CMD_anoremenu
            || c == CMD_aunmenu
            || c == CMD_nmenu
            || c == CMD_nnoremenu
            || c == CMD_nunmenu
            || c == CMD_vmenu
            || c == CMD_vnoremenu
            || c == CMD_vunmenu
            || c == CMD_omenu
            || c == CMD_onoremenu
            || c == CMD_ounmenu
            || c == CMD_imenu
            || c == CMD_inoremenu
            || c == CMD_iunmenu
            || c == CMD_cmenu
            || c == CMD_cnoremenu
            || c == CMD_cunmenu
            || c == CMD_tlmenu
            || c == CMD_tlnoremenu
            || c == CMD_tlunmenu
            || c == CMD_tmenu
            || c == CMD_tunmenu
            || c == CMD_popup
            || c == CMD_emenu =>
        {
            return set_context_in_menu_cmd(xph, cmd, arg.cast_mut(), forceit).cast_const();
        }

        c if c == CMD_colorscheme => {
            xp.xp_context = ExpandContext::Colors as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_compiler => {
            xp.xp_context = ExpandContext::Compiler as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_ownsyntax => {
            xp.xp_context = ExpandContext::Ownsyntax as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_setfiletype => {
            xp.xp_context = ExpandContext::Filetype as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_packadd => {
            xp.xp_context = ExpandContext::Packadd as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_runtime => {
            set_context_in_runtime_cmd(xph, arg);
        }

        c if c == CMD_language => {
            return rs_set_context_in_lang_cmd(xph, arg);
        }

        c if c == CMD_profile => {
            set_context_in_profile_cmd(xph, arg);
        }

        c if c == CMD_checkhealth => {
            xp.xp_context = ExpandContext::Checkhealth as c_int;
        }

        c if c == CMD_retab => {
            xp.xp_context = ExpandContext::Retab as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_messages => {
            xp.xp_context = ExpandContext::Messages as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_history => {
            xp.xp_context = ExpandContext::History as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_syntime => {
            xp.xp_context = ExpandContext::Syntime as c_int;
            xp.xp_pattern = arg.cast_mut();
        }

        c if c == CMD_argdelete => {
            let mut a = arg;
            loop {
                let sp = nvim_vim_strchr(a, c_int::from(b' '));
                if sp.is_null() {
                    break;
                }
                xp.xp_pattern = sp.cast_mut();
                a = sp.add(1);
            }
            xp.xp_context = ExpandContext::Arglist as c_int;
            xp.xp_pattern = a.cast_mut();
        }

        c if c == CMD_breakadd || c == CMD_profdel || c == CMD_breakdel => {
            let breakpt_cmd_type = if cmdidx == CMD_breakadd {
                0
            } else if cmdidx == CMD_breakdel {
                1
            } else {
                2
            };
            return rs_set_context_in_breakadd_cmd(xph, arg, breakpt_cmd_type);
        }

        c if c == CMD_scriptnames => {
            return rs_set_context_in_scriptnames_cmd(xph, arg);
        }

        c if c == CMD_filetype => {
            return rs_set_context_in_filetype_cmd(xph, arg);
        }

        c if c == CMD_lua || c == CMD_equal => {
            xp.xp_context = ExpandContext::Lua as c_int;
        }

        _ => {}
    }

    std::ptr::null()
}

// =============================================================================
// Helper: read the may_expand_pattern flag from C
// =============================================================================

extern "C" {
    fn nvim_cmdexpand_get_may_expand_pattern() -> bool;
}

fn may_expand_pattern() -> bool {
    // SAFETY: simple accessor reading a C global
    unsafe { nvim_cmdexpand_get_may_expand_pattern() }
}
