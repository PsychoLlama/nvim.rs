use crate::semsg_c;
use crate::src::nvim::api::private::helpers::{arena_dict, arena_string, cstr_as_string};
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{getdigits_int, skiptowhite, skipwhite};
use crate::src::nvim::eval::last_set_msg;
use crate::src::nvim::ex_docmd::{do_cmdline, ends_excmd};
use crate::src::nvim::garray::{ga_clear, ga_grow, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_8, HLF_D};
use crate::src::nvim::keycodes::{K_SPECIAL, KE_FILLER, replace_termcodes};
use crate::src::nvim::lua::executor::{
    api_free_luaref, api_new_luaref, nlua_do_ucmd, nlua_funcref_str, nlua_set_sctx,
};
use crate::src::nvim::main::{
    Columns, IObuff, cmdmod, curbuf, current_sctx, curtab, got_int, p_cpo, p_verbose,
};
use crate::src::nvim::mapping::set_context_in_map_cmd;
use crate::src::nvim::mbyte::{mb_copy_char, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::menu::set_context_in_menu_cmd;
use crate::src::nvim::message::{
    emsg, message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_outtrans_special, msg_putchar,
    msg_puts, msg_puts_hl, msg_puts_title,
};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    gettext, memmove, snprintf, strcat, strchr, strcmp, strcpy, strlen, strncasecmp, strncmp,
};
use crate::src::nvim::runtime::exestack;
use crate::src::nvim::strings::{arena_printf, vim_strchr, xstrnsave};
use crate::src::nvim::types::{
    Arena, CMD_SIZE, CMD_USER, CMD_USER_BUF, CMD_map, CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT,
    CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS, CMOD_KEEPMARKS, CMOD_KEEPPATTERNS, CMOD_LOCKMARKS,
    CMOD_NOAUTOCMD, CMOD_NOSWAPFILE, CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT, Dict, Integer,
    LuaRef, Object, OptInt, String_0, buf_T, cmd_addr_T, cmdmod_T, estack_T, exarg_T, expand_T,
    garray_T, int64_t, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger, kObjectTypeLuaRef,
    kObjectTypeNil, kObjectTypeString, key_value_pair, mod_entry_T, object,
    object_data as C2Rust_Unnamed, scid_T, sctx_T, size_t, ucmd_T, uint8_t, uint32_t, win_T,
};
use crate::src::nvim::window::{
    WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT, prevwin_curwin, tabpage_index,
};
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_SHELLCMDLINE: C2Rust_Unnamed_14 = 57;
pub const EXPAND_USER_ADDR_TYPE: C2Rust_Unnamed_14 = 43;
pub const EXPAND_USER_LUA: C2Rust_Unnamed_14 = 32;
pub const EXPAND_USER_LIST: C2Rust_Unnamed_14 = 31;
pub const EXPAND_USER_DEFINED: C2Rust_Unnamed_14 = 30;
pub const EXPAND_USER_COMPLETE: C2Rust_Unnamed_14 = 25;
pub const EXPAND_USER_NARGS: C2Rust_Unnamed_14 = 24;
pub const EXPAND_USER_CMD_FLAGS: C2Rust_Unnamed_14 = 23;
pub const EXPAND_USER_COMMANDS: C2Rust_Unnamed_14 = 22;
pub const EXPAND_MAPPINGS: C2Rust_Unnamed_14 = 16;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_BUFFERS: C2Rust_Unnamed_14 = 9;
pub const EXPAND_DIRECTORIES: C2Rust_Unnamed_14 = 3;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub const EXPAND_COMMANDS: C2Rust_Unnamed_14 = 1;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_17 = 8;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_17 = 2;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const UC_BUFFER: C2Rust_Unnamed_19 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_20 {
    pub expand: cmd_addr_T,
    pub name: *mut ::core::ffi::c_char,
    pub shortname: *mut ::core::ffi::c_char,
}
pub const ct_LT: C2Rust_Unnamed_21 = 8;
pub const ct_REGISTER: C2Rust_Unnamed_21 = 7;
pub const ct_MODS: C2Rust_Unnamed_21 = 6;
pub const ct_RANGE: C2Rust_Unnamed_21 = 5;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const ct_NONE: C2Rust_Unnamed_21 = 9;
pub const ct_LINE2: C2Rust_Unnamed_21 = 4;
pub const ct_LINE1: C2Rust_Unnamed_21 = 3;
pub const ct_COUNT: C2Rust_Unnamed_21 = 2;
pub const ct_BANG: C2Rust_Unnamed_21 = 1;
pub const ct_ARGS: C2Rust_Unnamed_21 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const EX_RANGE: ::core::ffi::c_uint = 0x1 as ::core::ffi::c_uint;
pub const EX_BANG: ::core::ffi::c_uint = 0x2 as ::core::ffi::c_uint;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_XFILE: ::core::ffi::c_uint = 0x8 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_DFLALL: ::core::ffi::c_uint = 0x20 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const EX_TRLBAR: ::core::ffi::c_uint = 0x100 as ::core::ffi::c_uint;
pub const EX_REGSTR: ::core::ffi::c_uint = 0x200 as ::core::ffi::c_uint;
pub const EX_COUNT: ::core::ffi::c_uint = 0x400 as ::core::ffi::c_uint;
pub const EX_ZEROR: ::core::ffi::c_uint = 0x1000 as ::core::ffi::c_uint;
pub const EX_BUFNAME: ::core::ffi::c_uint = 0x8000 as ::core::ffi::c_uint;
pub const EX_KEEPSCRIPT: ::core::ffi::c_uint = 0x4000000 as ::core::ffi::c_uint;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub static ucmds: GlobalCell<garray_T> = GlobalCell::new(garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: ::core::mem::size_of::<ucmd_T>() as ::core::ffi::c_int,
    ga_growsize: 4 as ::core::ffi::c_int,
    ga_data: NULL,
});
static e_argument_required_for_str: GlobalCell<[::core::ffi::c_char; 31]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
            *b"E179: Argument required for %s\0",
        )
    });
static e_no_such_user_defined_command_str: GlobalCell<[::core::ffi::c_char; 39]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 39], [::core::ffi::c_char; 39]>(
            *b"E184: No such user-defined command: %s\0",
        )
    });
static e_complete_used_without_allowing_arguments: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E1208: -complete used without allowing arguments\0",
        )
    });
static e_no_such_user_defined_command_in_current_buffer_str: GlobalCell<[::core::ffi::c_char; 58]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 58], [::core::ffi::c_char; 58]>(
            *b"E1237: No such user-defined command in current buffer: %s\0",
        )
    });
static command_complete: GlobalCell<[*const ::core::ffi::c_char; 64]> = GlobalCell::new([
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"command\0".as_ptr() as *const ::core::ffi::c_char,
    b"file\0".as_ptr() as *const ::core::ffi::c_char,
    b"dir\0".as_ptr() as *const ::core::ffi::c_char,
    b"option\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"tag\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"help\0".as_ptr() as *const ::core::ffi::c_char,
    b"buffer\0".as_ptr() as *const ::core::ffi::c_char,
    b"event\0".as_ptr() as *const ::core::ffi::c_char,
    b"menu\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"highlight\0".as_ptr() as *const ::core::ffi::c_char,
    b"augroup\0".as_ptr() as *const ::core::ffi::c_char,
    b"var\0".as_ptr() as *const ::core::ffi::c_char,
    b"mapping\0".as_ptr() as *const ::core::ffi::c_char,
    b"tag_listfiles\0".as_ptr() as *const ::core::ffi::c_char,
    b"function\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"expression\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"environment\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"color\0".as_ptr() as *const ::core::ffi::c_char,
    b"compiler\0".as_ptr() as *const ::core::ffi::c_char,
    b"custom\0".as_ptr() as *const ::core::ffi::c_char,
    b"customlist\0".as_ptr() as *const ::core::ffi::c_char,
    b"<Lua function>\0".as_ptr() as *const ::core::ffi::c_char,
    b"shellcmd\0".as_ptr() as *const ::core::ffi::c_char,
    b"sign\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"filetype\0".as_ptr() as *const ::core::ffi::c_char,
    b"file_in_path\0".as_ptr() as *const ::core::ffi::c_char,
    b"syntax\0".as_ptr() as *const ::core::ffi::c_char,
    b"locale\0".as_ptr() as *const ::core::ffi::c_char,
    b"history\0".as_ptr() as *const ::core::ffi::c_char,
    b"user\0".as_ptr() as *const ::core::ffi::c_char,
    b"syntime\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"packadd\0".as_ptr() as *const ::core::ffi::c_char,
    b"messages\0".as_ptr() as *const ::core::ffi::c_char,
    b"mapclear\0".as_ptr() as *const ::core::ffi::c_char,
    b"arglist\0".as_ptr() as *const ::core::ffi::c_char,
    b"diff_buffer\0".as_ptr() as *const ::core::ffi::c_char,
    b"breakpoint\0".as_ptr() as *const ::core::ffi::c_char,
    b"scriptnames\0".as_ptr() as *const ::core::ffi::c_char,
    b"runtime\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"keymap\0".as_ptr() as *const ::core::ffi::c_char,
    b"dir_in_path\0".as_ptr() as *const ::core::ffi::c_char,
    b"shellcmdline\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"filetypecmd\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"retab\0".as_ptr() as *const ::core::ffi::c_char,
    b"checkhealth\0".as_ptr() as *const ::core::ffi::c_char,
    b"lua\0".as_ptr() as *const ::core::ffi::c_char,
]);
static addr_type_complete: GlobalCell<[C2Rust_Unnamed_20; 9]> = GlobalCell::new([
    C2Rust_Unnamed_20 {
        expand: ADDR_ARGUMENTS,
        name: b"arguments\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"arg\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_LINES,
        name: b"lines\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"line\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_LOADED_BUFFERS,
        name: b"loaded_buffers\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        shortname: b"load\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_TABS,
        name: b"tabs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_BUFFERS,
        name: b"buffers\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"buf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_WINDOWS,
        name: b"windows\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"win\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_QUICKFIX,
        name: b"quickfix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"qf\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_OTHER,
        name: b"other\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        shortname: b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    },
    C2Rust_Unnamed_20 {
        expand: ADDR_NONE,
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        shortname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    },
]);
pub unsafe extern "C" fn find_ucmd(
    mut eap: *mut exarg_T,
    mut p: *mut ::core::ffi::c_char,
    mut full: *mut ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut complp: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = p.offset_from((*eap).cmd) as ::core::ffi::c_int;
    let mut matchlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut found: bool = false_0 != 0;
    let mut possible: bool = false_0 != 0;
    let mut amb_local: bool = false_0 != 0;
    let mut gap: *mut garray_T =
        &raw mut (*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_buffer).b_ucmds;
    loop {
        let mut j: ::core::ffi::c_int = 0;
        j = 0 as ::core::ffi::c_int;
        while j < (*gap).ga_len {
            let mut uc: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(j as isize);
            let mut cp: *mut ::core::ffi::c_char = (*eap).cmd;
            let mut np: *mut ::core::ffi::c_char = (*uc).uc_name;
            let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while k < len && *np as ::core::ffi::c_int != NUL && {
                let c2rust_fresh0 = cp;
                cp = cp.offset(1);
                let c2rust_fresh1 = np;
                np = np.offset(1);
                *c2rust_fresh0 as ::core::ffi::c_int == *c2rust_fresh1 as ::core::ffi::c_int
            } {
                k += 1;
            }
            if k == len
                || *np as ::core::ffi::c_int == NUL
                    && ascii_isdigit(*(*eap).cmd.offset(k as isize) as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
            {
                if k == len && found as ::core::ffi::c_int != 0 && *np as ::core::ffi::c_int != NUL
                {
                    if gap == ucmds.ptr() {
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    amb_local = true_0 != 0;
                }
                if !found || k == len && *np as ::core::ffi::c_int == NUL {
                    if k == len {
                        found = true_0 != 0;
                    } else {
                        possible = true_0 != 0;
                    }
                    if gap == ucmds.ptr() {
                        (*eap).cmdidx = CMD_USER;
                    } else {
                        (*eap).cmdidx = CMD_USER_BUF;
                    }
                    (*eap).argt = (*uc).uc_argt;
                    (*eap).useridx = j;
                    (*eap).addr_type = (*uc).uc_addr_type;
                    if !complp.is_null() {
                        *complp = (*uc).uc_compl;
                    }
                    if !xp.is_null() {
                        (*xp).xp_luaref = (*uc).uc_compl_luaref;
                        (*xp).xp_arg = (*uc).uc_compl_arg;
                        (*xp).xp_script_ctx = (*uc).uc_script_ctx;
                        (*xp).xp_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data
                            as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum;
                    }
                    matchlen = k;
                    if k == len && *np as ::core::ffi::c_int == NUL {
                        if !full.is_null() {
                            *full = true_0;
                        }
                        amb_local = false_0 != 0;
                        break;
                    }
                }
            }
            j += 1;
        }
        if j < (*gap).ga_len || gap == ucmds.ptr() {
            break;
        }
        gap = ucmds.ptr();
    }
    if amb_local {
        if !xp.is_null() {
            (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if found as ::core::ffi::c_int != 0 || possible as ::core::ffi::c_int != 0 {
        return p.offset((matchlen - len) as isize);
    }
    return p;
}
pub unsafe extern "C" fn set_context_in_user_cmd(
    mut xp: *mut expand_T,
    mut arg_in: *const ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    let mut arg: *const ::core::ffi::c_char = arg_in;
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    while *arg as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
        arg = arg.offset(1);
        p = skiptowhite(arg);
        if *p as ::core::ffi::c_int == NUL {
            p = strchr(arg, '=' as ::core::ffi::c_int);
            if p.is_null() {
                (*xp).xp_context = EXPAND_USER_CMD_FLAGS as ::core::ffi::c_int;
                (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            if strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"complete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_USER_COMPLETE as ::core::ffi::c_int;
                (*xp).xp_pattern =
                    (p as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize);
                return ::core::ptr::null::<::core::ffi::c_char>();
            } else if strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"nargs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_USER_NARGS as ::core::ffi::c_int;
                (*xp).xp_pattern =
                    (p as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize);
                return ::core::ptr::null::<::core::ffi::c_char>();
            } else if strncasecmp(
                arg as *mut ::core::ffi::c_char,
                b"addr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                p.offset_from(arg) as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                (*xp).xp_context = EXPAND_USER_ADDR_TYPE as ::core::ffi::c_int;
                (*xp).xp_pattern =
                    (p as *mut ::core::ffi::c_char).offset(1 as ::core::ffi::c_int as isize);
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        arg = skipwhite(p);
    }
    p = skiptowhite(arg);
    if *p as ::core::ffi::c_int == NUL {
        (*xp).xp_context = EXPAND_USER_COMMANDS as ::core::ffi::c_int;
        (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    return skipwhite(p);
}
pub unsafe extern "C" fn set_context_in_user_cmdarg(
    mut cmd: *const ::core::ffi::c_char,
    mut arg: *const ::core::ffi::c_char,
    mut argt: uint32_t,
    mut context: ::core::ffi::c_int,
    mut xp: *mut expand_T,
    mut forceit: bool,
) -> *const ::core::ffi::c_char {
    if context == EXPAND_NOTHING as ::core::ffi::c_int {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if argt & EX_XFILE as uint32_t != 0 {
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
    if context == EXPAND_MENUS as ::core::ffi::c_int {
        return set_context_in_menu_cmd(xp, cmd, arg as *mut ::core::ffi::c_char, forceit);
    }
    if context == EXPAND_COMMANDS as ::core::ffi::c_int {
        return arg;
    }
    if context == EXPAND_MAPPINGS as ::core::ffi::c_int {
        return set_context_in_map_cmd(
            xp,
            b"map\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            arg as *mut ::core::ffi::c_char,
            forceit,
            false_0 != 0,
            false_0 != 0,
            CMD_map,
        );
    }
    let mut p: *const ::core::ffi::c_char = arg;
    while *p != 0 {
        if *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
            arg = p.offset(1 as ::core::ffi::c_int as isize);
        } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        p = p.offset(utfc_ptr2len(p as *mut ::core::ffi::c_char) as isize);
    }
    (*xp).xp_pattern = arg as *mut ::core::ffi::c_char;
    (*xp).xp_context = context;
    return ::core::ptr::null::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn expand_user_command_name(
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return get_user_commands(
        ::core::ptr::null_mut::<expand_T>(),
        idx - CMD_SIZE as ::core::ffi::c_int,
    );
}
pub unsafe extern "C" fn get_user_commands(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let buf: *const buf_T = (*prevwin_curwin()).w_buffer;
    if idx < (*buf).b_ucmds.ga_len {
        return (*((*buf).b_ucmds.ga_data as *mut ucmd_T).offset(idx as isize)).uc_name;
    }
    idx -= (*buf).b_ucmds.ga_len;
    if idx < (*ucmds.ptr()).ga_len {
        let mut name: *mut ::core::ffi::c_char =
            (*((*ucmds.ptr()).ga_data as *mut ucmd_T).offset(idx as isize)).uc_name;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*buf).b_ucmds.ga_len {
            if strcmp(
                name,
                (*((*buf).b_ucmds.ga_data as *mut ucmd_T).offset(i as isize)).uc_name,
            ) == 0 as ::core::ffi::c_int
            {
                return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            i += 1;
        }
        return name;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_user_command_name(
    mut idx: ::core::ffi::c_int,
    mut cmdidx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if cmdidx == CMD_USER as ::core::ffi::c_int && idx < (*ucmds.ptr()).ga_len {
        return (*((*ucmds.ptr()).ga_data as *mut ucmd_T).offset(idx as isize)).uc_name;
    }
    if cmdidx == CMD_USER_BUF as ::core::ffi::c_int {
        let buf: *const buf_T = (*prevwin_curwin()).w_buffer;
        if idx < (*buf).b_ucmds.ga_len {
            return (*((*buf).b_ucmds.ga_data as *mut ucmd_T).offset(idx as isize)).uc_name;
        }
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_user_cmd_addr_type(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return (*addr_type_complete.ptr())[idx as usize].name;
}
pub unsafe extern "C" fn get_user_cmd_flags(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static user_cmd_flags: GlobalCell<[*mut ::core::ffi::c_char; 10]> = GlobalCell::new([
        b"addr\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"bang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"complete\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"count\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"nargs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"range\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"keepscript\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    if idx
        >= ::core::mem::size_of::<[*mut ::core::ffi::c_char; 10]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 10]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return (*user_cmd_flags.ptr())[idx as usize];
}
pub unsafe extern "C" fn get_user_cmd_nargs(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static user_cmd_nargs: GlobalCell<[*mut ::core::ffi::c_char; 5]> = GlobalCell::new([
        b"0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"1\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"*\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"?\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        b"+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    ]);
    if idx
        >= ::core::mem::size_of::<[*mut ::core::ffi::c_char; 5]>()
            .wrapping_div(::core::mem::size_of::<*mut ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*mut ::core::ffi::c_char; 5]>()
                    .wrapping_rem(::core::mem::size_of::<*mut ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return (*user_cmd_nargs.ptr())[idx as usize];
}
unsafe extern "C" fn get_command_complete(mut arg: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    if arg < 0 as ::core::ffi::c_int
        || arg
            >= ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
                .wrapping_div(
                    (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                        .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return (*command_complete.ptr())[arg as usize] as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn get_user_cmd_complete(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if idx
        >= ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(idx);
    if cmd_compl.is_null() || idx == EXPAND_USER_LUA as ::core::ffi::c_int {
        return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return cmd_compl;
}
pub unsafe extern "C" fn cmdcomplete_type_to_str(
    mut expand: ::core::ffi::c_int,
    mut compl_arg: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(expand);
    if cmd_compl.is_null() || expand == EXPAND_USER_LUA as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if expand == EXPAND_USER_LIST as ::core::ffi::c_int
        || expand == EXPAND_USER_DEFINED as ::core::ffi::c_int
    {
        let mut buflen: size_t = strlen(cmd_compl)
            .wrapping_add(strlen(compl_arg))
            .wrapping_add(2 as size_t);
        let mut buffer: *mut ::core::ffi::c_char = xmalloc(buflen) as *mut ::core::ffi::c_char;
        snprintf(
            buffer,
            buflen,
            b"%s,%s\0".as_ptr() as *const ::core::ffi::c_char,
            cmd_compl,
            compl_arg,
        );
        return buffer;
    }
    return xstrdup(cmd_compl);
}
pub unsafe extern "C" fn cmdcomplete_str_to_type(
    mut complete_str: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    if strncmp(
        complete_str,
        b"custom,\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return EXPAND_USER_DEFINED as ::core::ffi::c_int;
    }
    if strncmp(
        complete_str,
        b"customlist,\0".as_ptr() as *const ::core::ffi::c_char,
        11 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        return EXPAND_USER_LIST as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
        .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        ) as ::core::ffi::c_int
    {
        let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete(i);
        if !cmd_compl.is_null() {
            if strcmp(complete_str, (*command_complete.ptr())[i as usize])
                == 0 as ::core::ffi::c_int
            {
                return i;
            }
        }
        i += 1;
    }
    return EXPAND_NOTHING as ::core::ffi::c_int;
}
unsafe extern "C" fn uc_list(mut name: *mut ::core::ffi::c_char, mut name_len: size_t) {
    let mut found: bool = false_0 != 0;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    let mut gap: *const garray_T =
        &raw mut (*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_buffer).b_ucmds;
    loop {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            let mut a: uint32_t = (*cmd).uc_argt;
            if !(strncmp(name, (*cmd).uc_name, name_len) != 0 as ::core::ffi::c_int
                || message_filtered((*cmd).uc_name) as ::core::ffi::c_int != 0)
            {
                if !found {
                    msg_puts_title(gettext(
                        b"\n    Name              Args Address Complete    Definition\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                }
                found = true_0 != 0;
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    break;
                }
                let mut len: size_t = 4 as size_t;
                if a & EX_BANG as uint32_t != 0 {
                    msg_putchar('!' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if a & EX_REGSTR as uint32_t != 0 {
                    msg_putchar('"' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if gap != ucmds.ptr() as *const garray_T {
                    msg_putchar('b' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if a & EX_TRLBAR as uint32_t != 0 {
                    msg_putchar('|' as ::core::ffi::c_int);
                    len = len.wrapping_sub(1);
                }
                if len != 0 as size_t {
                    msg_puts(
                        (b"    \0".as_ptr() as *const ::core::ffi::c_char)
                            .add((4 as size_t).wrapping_sub(len)),
                    );
                }
                msg_outtrans((*cmd).uc_name, HLF_D, false_0 != 0);
                len = strlen((*cmd).uc_name).wrapping_add(4 as size_t);
                if len < 21 as size_t {
                    static spaces: GlobalCell<[::core::ffi::c_char; 18]> =
                        GlobalCell::new(unsafe {
                            ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(
                                *b"                 \0",
                            )
                        });
                    msg_puts(
                        (spaces.ptr() as *mut ::core::ffi::c_char)
                            .add(len.wrapping_sub(4 as size_t)),
                    );
                    len = 21 as size_t;
                }
                msg_putchar(' ' as ::core::ffi::c_int);
                len = len.wrapping_add(1);
                let over: int64_t = len as int64_t - 22 as int64_t;
                len = 0 as size_t;
                match a & (EX_EXTRA as uint32_t | EX_NOSPC as uint32_t | EX_NEEDARG as uint32_t) {
                    0 => {
                        let c2rust_fresh2 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh2 as usize] = '0' as ::core::ffi::c_char;
                    }
                    4 => {
                        let c2rust_fresh3 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh3 as usize] = '*' as ::core::ffi::c_char;
                    }
                    20 => {
                        let c2rust_fresh4 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh4 as usize] = '?' as ::core::ffi::c_char;
                    }
                    132 => {
                        let c2rust_fresh5 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh5 as usize] = '+' as ::core::ffi::c_char;
                    }
                    148 => {
                        let c2rust_fresh6 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh6 as usize] = '1' as ::core::ffi::c_char;
                    }
                    _ => {}
                }
                loop {
                    let c2rust_fresh7 = len;
                    len = len.wrapping_add(1);
                    (*IObuff.ptr())[c2rust_fresh7 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 5 as int64_t - over {
                        break;
                    }
                }
                if a & (EX_RANGE as uint32_t | EX_COUNT as uint32_t) != 0 {
                    if a & EX_COUNT as uint32_t != 0 {
                        let mut rc: ::core::ffi::c_int = snprintf(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).add(len),
                            (IOSIZE as size_t).wrapping_sub(len),
                            b"%ldc\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        );
                        debug_assert!(rc > 0 as ::core::ffi::c_int, "rc > 0");
                        len = len.wrapping_add(rc as size_t);
                    } else if a & EX_DFLALL as uint32_t != 0 {
                        let c2rust_fresh8 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh8 as usize] = '%' as ::core::ffi::c_char;
                    } else if (*cmd).uc_def >= 0 as int64_t {
                        let mut rc_0: ::core::ffi::c_int = snprintf(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).add(len),
                            (IOSIZE as size_t).wrapping_sub(len),
                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        );
                        debug_assert!(rc_0 > 0 as ::core::ffi::c_int, "rc > 0");
                        len = len.wrapping_add(rc_0 as size_t);
                    } else {
                        let c2rust_fresh9 = len;
                        len = len.wrapping_add(1);
                        (*IObuff.ptr())[c2rust_fresh9 as usize] = '.' as ::core::ffi::c_char;
                    }
                }
                loop {
                    let c2rust_fresh10 = len;
                    len = len.wrapping_add(1);
                    (*IObuff.ptr())[c2rust_fresh10 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 8 as int64_t - over {
                        break;
                    }
                }
                let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (*addr_type_complete.ptr())[j as usize].expand as ::core::ffi::c_uint
                    != ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if (*addr_type_complete.ptr())[j as usize].expand as ::core::ffi::c_uint
                        != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                        && (*addr_type_complete.ptr())[j as usize].expand as ::core::ffi::c_uint
                            == (*cmd).uc_addr_type as ::core::ffi::c_uint
                    {
                        let mut rc_1: ::core::ffi::c_int = snprintf(
                            (IObuff.ptr() as *mut ::core::ffi::c_char).add(len),
                            (IOSIZE as size_t).wrapping_sub(len),
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            (*addr_type_complete.ptr())[j as usize].shortname,
                        );
                        debug_assert!(rc_1 > 0 as ::core::ffi::c_int, "rc > 0");
                        len = len.wrapping_add(rc_1 as size_t);
                        break;
                    } else {
                        j += 1;
                    }
                }
                loop {
                    let c2rust_fresh11 = len;
                    len = len.wrapping_add(1);
                    (*IObuff.ptr())[c2rust_fresh11 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 13 as int64_t - over {
                        break;
                    }
                }
                let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete((*cmd).uc_compl);
                if !cmd_compl.is_null() {
                    let mut rc_2: ::core::ffi::c_int = snprintf(
                        (IObuff.ptr() as *mut ::core::ffi::c_char).add(len),
                        (IOSIZE as size_t).wrapping_sub(len),
                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                        get_command_complete((*cmd).uc_compl),
                    );
                    debug_assert!(rc_2 > 0 as ::core::ffi::c_int, "rc > 0");
                    len = len.wrapping_add(rc_2 as size_t);
                }
                loop {
                    let c2rust_fresh12 = len;
                    len = len.wrapping_add(1);
                    (*IObuff.ptr())[c2rust_fresh12 as usize] = ' ' as ::core::ffi::c_char;
                    if (len as int64_t) >= 25 as int64_t - over {
                        break;
                    }
                }
                (*IObuff.ptr())[len as usize] = NUL as ::core::ffi::c_char;
                msg_outtrans(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    0 as ::core::ffi::c_int,
                    false_0 != 0,
                );
                if (*cmd).uc_luaref != LUA_NOREF {
                    let mut fn_0: *mut ::core::ffi::c_char =
                        nlua_funcref_str((*cmd).uc_luaref, ::core::ptr::null_mut::<Arena>());
                    msg_puts_hl(fn_0, HLF_8, false_0 != 0);
                    xfree(fn_0 as *mut ::core::ffi::c_void);
                    if *(*cmd).uc_rep as ::core::ffi::c_int != NUL {
                        msg_puts(
                            b"\n                                               \0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                }
                msg_outtrans_special(
                    (*cmd).uc_rep,
                    false_0 != 0,
                    if name_len == 0 as size_t {
                        Columns.get() - 47 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
                if p_verbose.get() > 0 as OptInt {
                    last_set_msg((*cmd).uc_script_ctx);
                }
                line_breakcheck();
                if got_int.get() {
                    break;
                }
            }
            i += 1;
        }
        if gap == ucmds.ptr() as *const garray_T || i < (*gap).ga_len {
            break;
        }
        gap = ucmds.ptr();
    }
    if !found {
        msg(
            gettext(b"No user-defined commands found\0".as_ptr() as *const ::core::ffi::c_char),
            0 as ::core::ffi::c_int,
        );
    }
}
pub unsafe extern "C" fn parse_addr_type_arg(
    mut value: *mut ::core::ffi::c_char,
    mut vallen: ::core::ffi::c_int,
    mut addr_type_arg: *mut cmd_addr_T,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while (*addr_type_complete.ptr())[i as usize].expand as ::core::ffi::c_uint
        != ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut a: ::core::ffi::c_int = (strlen((*addr_type_complete.ptr())[i as usize].name)
            as ::core::ffi::c_int
            == vallen) as ::core::ffi::c_int;
        let mut b: ::core::ffi::c_int = (strncmp(
            value,
            (*addr_type_complete.ptr())[i as usize].name,
            vallen as size_t,
        ) == 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
        if a != 0 && b != 0 {
            *addr_type_arg = (*addr_type_complete.ptr())[i as usize].expand;
            break;
        } else {
            i += 1;
        }
    }
    if (*addr_type_complete.ptr())[i as usize].expand as ::core::ffi::c_uint
        == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut err: *mut ::core::ffi::c_char = value;
        i = 0 as ::core::ffi::c_int;
        while *err.offset(i as isize) as ::core::ffi::c_int != NUL
            && !ascii_iswhite(*err.offset(i as isize) as ::core::ffi::c_int)
        {
            i += 1;
        }
        *err.offset(i as isize) = NUL as ::core::ffi::c_char;
        semsg_c!(
            gettext(
                b"E180: Invalid address type value: %s\0".as_ptr() as *const ::core::ffi::c_char
            ),
            err,
        );
        return FAIL;
    }
    return OK;
}
pub unsafe extern "C" fn parse_compl_arg(
    mut value: *const ::core::ffi::c_char,
    mut vallen: ::core::ffi::c_int,
    mut complp: *mut ::core::ffi::c_int,
    mut argt: *mut uint32_t,
    mut compl_arg: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut arg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut arglen: size_t = 0 as size_t;
    let mut valend: ::core::ffi::c_int = vallen;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < vallen {
        if *value.offset(i as isize) as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            arg = value.offset((i + 1 as ::core::ffi::c_int) as isize) as *mut ::core::ffi::c_char;
            arglen = (vallen - i - 1 as ::core::ffi::c_int) as size_t;
            valend = i;
            break;
        } else {
            i += 1;
        }
    }
    let mut i_0: ::core::ffi::c_int = 0;
    i_0 = 0 as ::core::ffi::c_int;
    while i_0
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        if !get_command_complete(i_0).is_null() {
            if strlen((*command_complete.ptr())[i_0 as usize]) as ::core::ffi::c_int == valend
                && strncmp(
                    value,
                    (*command_complete.ptr())[i_0 as usize],
                    valend as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                *complp = i_0;
                if i_0 == EXPAND_BUFFERS as ::core::ffi::c_int {
                    *argt = (*argt as ::core::ffi::c_uint | EX_BUFNAME) as uint32_t;
                } else if i_0 == EXPAND_DIRECTORIES as ::core::ffi::c_int
                    || i_0 == EXPAND_FILES as ::core::ffi::c_int
                    || i_0 == EXPAND_SHELLCMDLINE as ::core::ffi::c_int
                {
                    *argt = (*argt as ::core::ffi::c_uint | EX_XFILE) as uint32_t;
                }
                break;
            }
        }
        i_0 += 1;
    }
    if i_0
        == ::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 64]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as ::core::ffi::c_int
    {
        semsg_c!(
            gettext(b"E180: Invalid complete value: %s\0".as_ptr() as *const ::core::ffi::c_char),
            value,
        );
        return FAIL;
    }
    if *complp != EXPAND_USER_DEFINED as ::core::ffi::c_int
        && *complp != EXPAND_USER_LIST as ::core::ffi::c_int
        && !arg.is_null()
    {
        emsg(gettext(
            b"E468: Completion argument only allowed for custom completion\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if (*complp == EXPAND_USER_DEFINED as ::core::ffi::c_int
        || *complp == EXPAND_USER_LIST as ::core::ffi::c_int)
        && arg.is_null()
    {
        emsg(gettext(
            b"E467: Custom completion requires a function argument\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if !arg.is_null() {
        *compl_arg = xstrnsave(arg, arglen);
    }
    return OK;
}
unsafe extern "C" fn uc_scan_attr(
    mut attr: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut argt: *mut uint32_t,
    mut def: *mut ::core::ffi::c_int,
    mut flags: *mut ::core::ffi::c_int,
    mut complp: *mut ::core::ffi::c_int,
    mut compl_arg: *mut *mut ::core::ffi::c_char,
    mut addr_type_arg: *mut cmd_addr_T,
) -> ::core::ffi::c_int {
    if len == 0 as size_t {
        emsg(gettext(
            b"E175: No attribute specified\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return FAIL;
    }
    if strncasecmp(
        attr,
        b"bang\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_BANG) as uint32_t;
    } else if strncasecmp(
        attr,
        b"buffer\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *flags |= UC_BUFFER as ::core::ffi::c_int;
    } else if strncasecmp(
        attr,
        b"register\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_REGSTR) as uint32_t;
    } else if strncasecmp(
        attr,
        b"keepscript\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_KEEPSCRIPT) as uint32_t;
    } else if strncasecmp(
        attr,
        b"bar\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        len,
    ) == 0 as ::core::ffi::c_int
    {
        *argt = (*argt as ::core::ffi::c_uint | EX_TRLBAR) as uint32_t;
    } else {
        let mut val: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut vallen: size_t = 0 as size_t;
        let mut attrlen: size_t = len;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < len as ::core::ffi::c_int {
            if *attr.offset(i as isize) as ::core::ffi::c_int == '=' as ::core::ffi::c_int {
                val = attr.offset((i + 1 as ::core::ffi::c_int) as isize);
                vallen = len.wrapping_sub(i as size_t).wrapping_sub(1 as size_t);
                attrlen = i as size_t;
                break;
            } else {
                i += 1;
            }
        }
        if strncasecmp(
            attr,
            b"nargs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            attrlen,
        ) == 0 as ::core::ffi::c_int
        {
            's_180: {
                '_wrong_nargs: {
                    if vallen == 1 as size_t {
                        if *val as ::core::ffi::c_int != '0' as ::core::ffi::c_int {
                            if *val as ::core::ffi::c_int == '1' as ::core::ffi::c_int {
                                *argt = (*argt as ::core::ffi::c_uint
                                    | (EX_EXTRA | EX_NOSPC | EX_NEEDARG))
                                    as uint32_t;
                            } else if *val as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                                *argt = (*argt as ::core::ffi::c_uint | EX_EXTRA) as uint32_t;
                            } else if *val as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                                *argt = (*argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC))
                                    as uint32_t;
                            } else if *val as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
                                *argt = (*argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NEEDARG))
                                    as uint32_t;
                            } else {
                                break '_wrong_nargs;
                            }
                        }
                        break 's_180;
                    }
                }
                emsg(gettext(
                    b"E176: Invalid number of arguments\0".as_ptr() as *const ::core::ffi::c_char
                ));
                return FAIL;
            }
        } else {
            's_409: {
                let mut p: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                '_two_count: {
                    '_invalid_count: {
                        if strncasecmp(
                            attr,
                            b"range\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            *argt = (*argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                            if vallen == 1 as size_t
                                && *val as ::core::ffi::c_int == '%' as ::core::ffi::c_int
                            {
                                *argt = (*argt as ::core::ffi::c_uint | EX_DFLALL) as uint32_t;
                            } else if !val.is_null() {
                                p = val;
                                if *def >= 0 as ::core::ffi::c_int {
                                    break '_two_count;
                                } else {
                                    *def = getdigits_int(
                                        &raw mut p,
                                        true_0 != 0,
                                        0 as ::core::ffi::c_int,
                                    );
                                    *argt = (*argt as ::core::ffi::c_uint | EX_ZEROR) as uint32_t;
                                    if p != val.add(vallen) || vallen == 0 as size_t {
                                        break '_invalid_count;
                                    }
                                }
                            }
                            if *addr_type_arg as ::core::ffi::c_uint
                                == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                *addr_type_arg = ADDR_LINES;
                            }
                        } else if strncasecmp(
                            attr,
                            b"count\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            *argt = (*argt as ::core::ffi::c_uint
                                | (EX_COUNT | EX_ZEROR | EX_RANGE))
                                as uint32_t;
                            if *addr_type_arg as ::core::ffi::c_uint
                                == ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                *addr_type_arg = ADDR_OTHER;
                            }
                            if !val.is_null() {
                                let mut p_0: *mut ::core::ffi::c_char = val;
                                if *def >= 0 as ::core::ffi::c_int {
                                    break '_two_count;
                                } else {
                                    *def = getdigits_int(
                                        &raw mut p_0,
                                        true_0 != 0,
                                        0 as ::core::ffi::c_int,
                                    );
                                    if p_0 != val.add(vallen) {
                                        break '_invalid_count;
                                    }
                                }
                            }
                            *def = if *def > 0 as ::core::ffi::c_int {
                                *def
                            } else {
                                0 as ::core::ffi::c_int
                            };
                        } else if strncasecmp(
                            attr,
                            b"complete\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            if val.is_null() {
                                semsg_c!(
                                    gettext(
                                        (e_argument_required_for_str.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    b"-complete\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                return FAIL;
                            }
                            if parse_compl_arg(
                                val,
                                vallen as ::core::ffi::c_int,
                                complp,
                                argt,
                                compl_arg,
                            ) == FAIL
                            {
                                return FAIL;
                            }
                        } else if strncasecmp(
                            attr,
                            b"addr\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            attrlen,
                        ) == 0 as ::core::ffi::c_int
                        {
                            *argt = (*argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                            if val.is_null() {
                                semsg_c!(
                                    gettext(
                                        (e_argument_required_for_str.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    b"-addr\0".as_ptr() as *const ::core::ffi::c_char,
                                );
                                return FAIL;
                            }
                            if parse_addr_type_arg(val, vallen as ::core::ffi::c_int, addr_type_arg)
                                == FAIL
                            {
                                return FAIL;
                            }
                            if *addr_type_arg as ::core::ffi::c_uint
                                != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                *argt = (*argt as ::core::ffi::c_uint | EX_ZEROR) as uint32_t;
                            }
                        } else {
                            let mut ch: ::core::ffi::c_char = *attr.add(len);
                            *attr.add(len) = NUL as ::core::ffi::c_char;
                            semsg_c!(
                                gettext(b"E181: Invalid attribute: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                attr,
                            );
                            *attr.add(len) = ch;
                            return FAIL;
                        }
                        break 's_409;
                    }
                    emsg(gettext(b"E178: Invalid default value for count\0".as_ptr()
                        as *const ::core::ffi::c_char));
                    return FAIL;
                }
                emsg(gettext(b"E177: Count cannot be specified twice\0".as_ptr()
                    as *const ::core::ffi::c_char));
                return FAIL;
            }
        }
    }
    return OK;
}
pub unsafe extern "C" fn uc_validate_name(
    mut name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if *name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
    {
        while *name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && *name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
            || *name as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
                && *name as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
            || ascii_isdigit(*name as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            name = name.offset(1);
        }
    }
    if ends_excmd(*name as ::core::ffi::c_int) == 0 && !ascii_iswhite(*name as ::core::ffi::c_int) {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return name;
}
pub unsafe extern "C" fn uc_add_command(
    mut name: *mut ::core::ffi::c_char,
    mut name_len: size_t,
    mut rep: *const ::core::ffi::c_char,
    mut argt: uint32_t,
    mut def: int64_t,
    mut flags: ::core::ffi::c_int,
    mut context: ::core::ffi::c_int,
    mut compl_arg: *mut ::core::ffi::c_char,
    mut compl_luaref: LuaRef,
    mut preview_luaref: LuaRef,
    mut addr_type: cmd_addr_T,
    mut luaref: LuaRef,
    mut force: bool,
) -> ::core::ffi::c_int {
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut cmp: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut rep_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    replace_termcodes(
        rep,
        strlen(rep),
        &raw mut rep_buf,
        0 as scid_T,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<bool>(),
        p_cpo.get(),
    );
    if rep_buf.is_null() {
        rep_buf = xstrdup(rep);
    }
    if flags & UC_BUFFER as ::core::ffi::c_int != 0 {
        gap = &raw mut (*curbuf.get()).b_ucmds;
        if (*gap).ga_itemsize == 0 as ::core::ffi::c_int {
            ga_init(
                gap,
                ::core::mem::size_of::<ucmd_T>() as ::core::ffi::c_int,
                4 as ::core::ffi::c_int,
            );
        }
    } else {
        gap = ucmds.ptr();
    }
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    '_fail: {
        while i < (*gap).ga_len {
            cmd = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            let mut len: size_t = strlen((*cmd).uc_name);
            cmp = strncmp(name, (*cmd).uc_name, name_len);
            if cmp == 0 as ::core::ffi::c_int {
                if name_len < len {
                    cmp = -1 as ::core::ffi::c_int;
                } else if name_len > len {
                    cmp = 1 as ::core::ffi::c_int;
                }
            }
            if cmp == 0 as ::core::ffi::c_int {
                if !force
                    && ((*cmd).uc_script_ctx.sc_sid != (*current_sctx.ptr()).sc_sid
                        || (*cmd).uc_script_ctx.sc_seq == (*current_sctx.ptr()).sc_seq)
                {
                    semsg_c!(
                        gettext(
                            b"E174: Command already exists: add ! to replace it: %s\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                        name,
                    );
                    break '_fail;
                } else {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut (*cmd).uc_rep as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                    let mut ptr__0: *mut *mut ::core::ffi::c_void =
                        &raw mut (*cmd).uc_compl_arg as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__0);
                    *ptr__0 = NULL;
                    let _ = *ptr__0;
                    if (*cmd).uc_luaref != LUA_NOREF {
                        api_free_luaref((*cmd).uc_luaref);
                        (*cmd).uc_luaref = LUA_NOREF as LuaRef;
                    }
                    if (*cmd).uc_compl_luaref != LUA_NOREF {
                        api_free_luaref((*cmd).uc_compl_luaref);
                        (*cmd).uc_compl_luaref = LUA_NOREF as LuaRef;
                    }
                    if (*cmd).uc_preview_luaref != LUA_NOREF {
                        api_free_luaref((*cmd).uc_preview_luaref);
                        (*cmd).uc_preview_luaref = LUA_NOREF as LuaRef;
                    }
                    break;
                }
            } else {
                if cmp < 0 as ::core::ffi::c_int {
                    break;
                }
                i += 1;
            }
        }
        if cmp != 0 as ::core::ffi::c_int {
            ga_grow(gap, 1 as ::core::ffi::c_int);
            let p: *mut ::core::ffi::c_char = xstrnsave(name, name_len);
            cmd = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            memmove(
                cmd.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                cmd as *const ::core::ffi::c_void,
                (((*gap).ga_len - i) as size_t).wrapping_mul(::core::mem::size_of::<ucmd_T>()),
            );
            (*gap).ga_len += 1;
            (*cmd).uc_name = p;
        }
        (*cmd).uc_rep = rep_buf;
        (*cmd).uc_argt = argt;
        (*cmd).uc_def = def;
        (*cmd).uc_compl = context;
        (*cmd).uc_script_ctx = current_sctx.get();
        (*cmd).uc_script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*cmd).uc_script_ctx);
        (*cmd).uc_compl_arg = compl_arg;
        (*cmd).uc_compl_luaref = compl_luaref;
        (*cmd).uc_preview_luaref = preview_luaref;
        (*cmd).uc_addr_type = addr_type;
        (*cmd).uc_luaref = luaref;
        return OK;
    }
    xfree(rep_buf as *mut ::core::ffi::c_void);
    xfree(compl_arg as *mut ::core::ffi::c_void);
    if luaref != LUA_NOREF {
        api_free_luaref(luaref);
        luaref = LUA_NOREF as LuaRef;
    }
    if compl_luaref != LUA_NOREF {
        api_free_luaref(compl_luaref);
        compl_luaref = LUA_NOREF as LuaRef;
    }
    if preview_luaref != LUA_NOREF {
        api_free_luaref(preview_luaref);
        preview_luaref = LUA_NOREF as LuaRef;
    }
    return FAIL;
}
pub unsafe fn ex_command(mut eap: *mut exarg_T) {
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name_len: size_t = 0;
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argt: uint32_t = 0 as uint32_t;
    let mut def: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
    let mut compl_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut addr_type_arg: cmd_addr_T = ADDR_NONE;
    let mut has_attr: ::core::ffi::c_int =
        (*(*eap).arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int) as ::core::ffi::c_int;
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    '_theend: {
        while *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            p = p.offset(1);
            end = skiptowhite(p);
            if uc_scan_attr(
                p,
                end.offset_from(p) as size_t,
                &raw mut argt,
                &raw mut def,
                &raw mut flags,
                &raw mut context,
                &raw mut compl_arg,
                &raw mut addr_type_arg,
            ) == FAIL
            {
                break '_theend;
            }
            p = skipwhite(end);
        }
        name = p;
        end = uc_validate_name(name);
        if end.is_null() {
            emsg(gettext(
                b"E182: Invalid command name\0".as_ptr() as *const ::core::ffi::c_char
            ));
        } else {
            name_len = end.offset_from(name) as size_t;
            p = skipwhite(end);
            if has_attr == 0 && ends_excmd(*p as ::core::ffi::c_int) != 0 {
                uc_list(name, name_len);
            } else if !(*name as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                && *name as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint)
            {
                emsg(gettext(
                    b"E183: User defined commands must start with an uppercase letter\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            } else if name_len <= 4 as size_t
                && strncmp(
                    name,
                    b"Next\0".as_ptr() as *const ::core::ffi::c_char,
                    name_len,
                ) == 0 as ::core::ffi::c_int
            {
                emsg(gettext(
                    b"E841: Reserved name, cannot be used for user defined command\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            } else if context > 0 as ::core::ffi::c_int
                && argt & EX_EXTRA as uint32_t == 0 as uint32_t
            {
                emsg(gettext(
                    (e_complete_used_without_allowing_arguments.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
            } else {
                uc_add_command(
                    name,
                    name_len,
                    p,
                    argt,
                    def as int64_t,
                    flags,
                    context,
                    compl_arg,
                    LUA_NOREF,
                    LUA_NOREF,
                    addr_type_arg,
                    LUA_NOREF,
                    (*eap).forceit != 0,
                );
                return;
            }
        }
    }
    xfree(compl_arg as *mut ::core::ffi::c_void);
}
pub unsafe fn ex_comclear(mut _eap: *mut exarg_T) {
    uc_clear(ucmds.ptr());
    if !(*curbuf.ptr()).is_null() {
        uc_clear(&raw mut (*curbuf.get()).b_ucmds);
    }
}
pub unsafe extern "C" fn free_ucmd(mut cmd: *mut ucmd_T) {
    xfree((*cmd).uc_name as *mut ::core::ffi::c_void);
    xfree((*cmd).uc_rep as *mut ::core::ffi::c_void);
    xfree((*cmd).uc_compl_arg as *mut ::core::ffi::c_void);
    if (*cmd).uc_compl_luaref != LUA_NOREF {
        api_free_luaref((*cmd).uc_compl_luaref);
        (*cmd).uc_compl_luaref = LUA_NOREF as LuaRef;
    }
    if (*cmd).uc_luaref != LUA_NOREF {
        api_free_luaref((*cmd).uc_luaref);
        (*cmd).uc_luaref = LUA_NOREF as LuaRef;
    }
    if (*cmd).uc_preview_luaref != LUA_NOREF {
        api_free_luaref((*cmd).uc_preview_luaref);
        (*cmd).uc_preview_luaref = LUA_NOREF as LuaRef;
    }
}
pub unsafe extern "C" fn uc_clear(mut gap: *mut garray_T) {
    let mut _gap: *mut garray_T = gap;
    if !(*_gap).ga_data.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*_gap).ga_len {
            let mut _item: *mut ucmd_T = ((*_gap).ga_data as *mut ucmd_T).offset(i as isize);
            free_ucmd(_item);
            i += 1;
        }
    }
    ga_clear(_gap);
}
pub unsafe fn ex_delcommand(mut eap: *mut exarg_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    let mut res: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut arg: *const ::core::ffi::c_char = (*eap).arg;
    let mut buffer_only: bool = false_0 != 0;
    if strncmp(
        arg,
        b"-buffer\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        buffer_only = true_0 != 0;
        arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
    }
    let mut gap: *mut garray_T = &raw mut (*curbuf.get()).b_ucmds;
    loop {
        i = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            cmd = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            res = strcmp(arg, (*cmd).uc_name);
            if res <= 0 as ::core::ffi::c_int {
                break;
            }
            i += 1;
        }
        if gap == ucmds.ptr()
            || res == 0 as ::core::ffi::c_int
            || buffer_only as ::core::ffi::c_int != 0
        {
            break;
        }
        gap = ucmds.ptr();
    }
    if res != 0 as ::core::ffi::c_int {
        semsg_c!(
            gettext(if buffer_only as ::core::ffi::c_int != 0 {
                (e_no_such_user_defined_command_in_current_buffer_str.ptr() as *const _)
                    as *const ::core::ffi::c_char
            } else {
                (e_no_such_user_defined_command_str.ptr() as *const _) as *const ::core::ffi::c_char
            }),
            arg,
        );
        return;
    }
    free_ucmd(cmd);
    (*gap).ga_len -= 1;
    if i < (*gap).ga_len {
        memmove(
            cmd as *mut ::core::ffi::c_void,
            cmd.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            (((*gap).ga_len - i) as size_t).wrapping_mul(::core::mem::size_of::<ucmd_T>()),
        );
    }
}
pub unsafe extern "C" fn uc_split_args_iter(
    mut arg: *const ::core::ffi::c_char,
    mut arglen: size_t,
    mut end: *mut size_t,
    mut buf: *mut ::core::ffi::c_char,
    mut len: *mut size_t,
) -> bool {
    if arglen == 0 {
        return true_0 != 0;
    }
    let mut pos: size_t = *end;
    while pos < arglen
        && ascii_iswhite(*arg.add(pos) as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        pos = pos.wrapping_add(1);
    }
    let mut l: size_t = 0 as size_t;
    while pos < arglen.wrapping_sub(1 as size_t) {
        if *arg.add(pos) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && (*arg.add(pos.wrapping_add(1 as size_t)) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                || ascii_iswhite(*arg.add(pos.wrapping_add(1 as size_t)) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            pos = pos.wrapping_add(1);
            let c2rust_fresh13 = l;
            l = l.wrapping_add(1);
            *buf.add(c2rust_fresh13) = *arg.add(pos);
        } else {
            let c2rust_fresh14 = l;
            l = l.wrapping_add(1);
            *buf.add(c2rust_fresh14) = *arg.add(pos);
        }
        if ascii_iswhite(*arg.add(pos.wrapping_add(1 as size_t)) as ::core::ffi::c_int) {
            *end = pos.wrapping_add(1 as size_t);
            *len = l;
            return false_0 != 0;
        }
        pos = pos.wrapping_add(1);
    }
    if pos < arglen && !ascii_iswhite(*arg.add(pos) as ::core::ffi::c_int) {
        let c2rust_fresh15 = l;
        l = l.wrapping_add(1);
        *buf.add(c2rust_fresh15) = *arg.add(pos);
    }
    *len = l;
    return true_0 != 0;
}
pub unsafe extern "C" fn uc_nargs_upper_bound(
    mut arg: *const ::core::ffi::c_char,
    mut arglen: size_t,
) -> size_t {
    let mut was_white: bool = true_0 != 0;
    let mut nargs: size_t = 0 as size_t;
    let mut i: size_t = 0 as size_t;
    while i < arglen {
        let mut is_white: bool = ascii_iswhite(*arg.add(i) as ::core::ffi::c_int);
        if was_white as ::core::ffi::c_int != 0 && !is_white {
            nargs = nargs.wrapping_add(1);
        }
        was_white = is_white;
        i = i.wrapping_add(1);
    }
    return nargs;
}
unsafe extern "C" fn uc_split_args(
    mut arg: *const ::core::ffi::c_char,
    mut args: *mut *mut ::core::ffi::c_char,
    mut arglens: *const size_t,
    mut argc: size_t,
    mut lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut len: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    if args.is_null() {
        let mut p: *const ::core::ffi::c_char = arg;
        while *p != 0 {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                len += 2 as ::core::ffi::c_int;
                p = p.offset(2 as ::core::ffi::c_int as isize);
            } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && ascii_iswhite(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                len += 1 as ::core::ffi::c_int;
                p = p.offset(2 as ::core::ffi::c_int as isize);
            } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
            {
                len += 2 as ::core::ffi::c_int;
                p = p.offset(1 as ::core::ffi::c_int as isize);
            } else if ascii_iswhite(*p as ::core::ffi::c_int) {
                p = skipwhite(p);
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
                len += 4 as ::core::ffi::c_int;
            } else {
                let charlen: ::core::ffi::c_int = utfc_ptr2len(p);
                len += charlen;
                p = p.offset(charlen as isize);
            }
        }
    } else {
        let mut i: size_t = 0 as size_t;
        while i < argc {
            let mut p_0: *const ::core::ffi::c_char = *args.add(i);
            let mut arg_end: *const ::core::ffi::c_char = (*args.add(i)).add(*arglens.add(i));
            while p_0 < arg_end {
                if *p_0 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p_0 as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                {
                    len += 2 as ::core::ffi::c_int;
                    p_0 = p_0.offset(1 as ::core::ffi::c_int as isize);
                } else {
                    let charlen_0: ::core::ffi::c_int = utfc_ptr2len(p_0);
                    len += charlen_0;
                    p_0 = p_0.offset(charlen_0 as isize);
                }
            }
            if i != argc.wrapping_sub(1 as size_t) {
                len += 4 as ::core::ffi::c_int;
            }
            i = i.wrapping_add(1);
        }
    }
    let mut buf: *mut ::core::ffi::c_char =
        xmalloc((len as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut q: *mut ::core::ffi::c_char = buf;
    let c2rust_fresh26 = q;
    q = q.offset(1);
    *c2rust_fresh26 = '"' as ::core::ffi::c_char;
    if args.is_null() {
        let mut p_1: *const ::core::ffi::c_char = arg;
        while *p_1 != 0 {
            if *p_1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p_1.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
            {
                let c2rust_fresh27 = q;
                q = q.offset(1);
                *c2rust_fresh27 = '\\' as ::core::ffi::c_char;
                let c2rust_fresh28 = q;
                q = q.offset(1);
                *c2rust_fresh28 = '\\' as ::core::ffi::c_char;
                p_1 = p_1.offset(2 as ::core::ffi::c_int as isize);
            } else if *p_1.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && ascii_iswhite(*p_1.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0
            {
                let c2rust_fresh29 = q;
                q = q.offset(1);
                *c2rust_fresh29 = *p_1.offset(1 as ::core::ffi::c_int as isize);
                p_1 = p_1.offset(2 as ::core::ffi::c_int as isize);
            } else if *p_1 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p_1 as ::core::ffi::c_int == '"' as ::core::ffi::c_int
            {
                let c2rust_fresh30 = q;
                q = q.offset(1);
                *c2rust_fresh30 = '\\' as ::core::ffi::c_char;
                let c2rust_fresh31 = p_1;
                p_1 = p_1.offset(1);
                let c2rust_fresh32 = q;
                q = q.offset(1);
                *c2rust_fresh32 = *c2rust_fresh31;
            } else if ascii_iswhite(*p_1 as ::core::ffi::c_int) {
                p_1 = skipwhite(p_1);
                if *p_1 as ::core::ffi::c_int == NUL {
                    break;
                }
                let c2rust_fresh33 = q;
                q = q.offset(1);
                *c2rust_fresh33 = '"' as ::core::ffi::c_char;
                let c2rust_fresh34 = q;
                q = q.offset(1);
                *c2rust_fresh34 = ',' as ::core::ffi::c_char;
                let c2rust_fresh35 = q;
                q = q.offset(1);
                *c2rust_fresh35 = ' ' as ::core::ffi::c_char;
                let c2rust_fresh36 = q;
                q = q.offset(1);
                *c2rust_fresh36 = '"' as ::core::ffi::c_char;
            } else {
                mb_copy_char(&raw mut p_1, &raw mut q);
            }
        }
    } else {
        let mut i_0: size_t = 0 as size_t;
        while i_0 < argc {
            let mut p_2: *const ::core::ffi::c_char = *args.add(i_0);
            let mut arg_end_0: *const ::core::ffi::c_char = (*args.add(i_0)).add(*arglens.add(i_0));
            while p_2 < arg_end_0 {
                if *p_2 as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    || *p_2 as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                {
                    let c2rust_fresh37 = q;
                    q = q.offset(1);
                    *c2rust_fresh37 = '\\' as ::core::ffi::c_char;
                    let c2rust_fresh38 = p_2;
                    p_2 = p_2.offset(1);
                    let c2rust_fresh39 = q;
                    q = q.offset(1);
                    *c2rust_fresh39 = *c2rust_fresh38;
                } else {
                    mb_copy_char(&raw mut p_2, &raw mut q);
                }
            }
            if i_0 != argc.wrapping_sub(1 as size_t) {
                let c2rust_fresh40 = q;
                q = q.offset(1);
                *c2rust_fresh40 = '"' as ::core::ffi::c_char;
                let c2rust_fresh41 = q;
                q = q.offset(1);
                *c2rust_fresh41 = ',' as ::core::ffi::c_char;
                let c2rust_fresh42 = q;
                q = q.offset(1);
                *c2rust_fresh42 = ' ' as ::core::ffi::c_char;
                let c2rust_fresh43 = q;
                q = q.offset(1);
                *c2rust_fresh43 = '"' as ::core::ffi::c_char;
            }
            i_0 = i_0.wrapping_add(1);
        }
    }
    let c2rust_fresh44 = q;
    q = q.offset(1);
    *c2rust_fresh44 = '"' as ::core::ffi::c_char;
    *q = 0 as ::core::ffi::c_char;
    *lenp = len as size_t;
    return buf;
}
unsafe extern "C" fn add_cmd_modifier(
    mut buf: *mut ::core::ffi::c_char,
    mut mod_str: *mut ::core::ffi::c_char,
    mut multi_mods: *mut bool,
) -> size_t {
    let mut result: size_t = strlen(mod_str);
    if *multi_mods {
        result = result.wrapping_add(1);
    }
    if !buf.is_null() {
        if *multi_mods {
            strcat(buf, b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
        strcat(buf, mod_str);
    }
    *multi_mods = true_0 != 0;
    return result;
}
pub unsafe extern "C" fn add_win_cmd_modifiers(
    mut buf: *mut ::core::ffi::c_char,
    mut cmod: *const cmdmod_T,
    mut multi_mods: *mut bool,
) -> size_t {
    let mut result: size_t = 0 as size_t;
    if (*cmod).cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            multi_mods,
        ));
    }
    if (*cmod).cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            b"belowright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            multi_mods,
        ));
    }
    if (*cmod).cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            b"botright\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            multi_mods,
        ));
    }
    if (*cmod).cmod_tab > 0 as ::core::ffi::c_int {
        let mut tabnr: ::core::ffi::c_int = (*cmod).cmod_tab - 1 as ::core::ffi::c_int;
        if tabnr == tabpage_index(curtab.get()) {
            result = result.wrapping_add(add_cmd_modifier(
                buf,
                b"tab\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                multi_mods,
            ));
        } else {
            let mut tab_buf: [::core::ffi::c_char; 68] = [0; 68];
            snprintf(
                &raw mut tab_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 68]>(),
                b"%dtab\0".as_ptr() as *const ::core::ffi::c_char,
                tabnr,
            );
            result = result.wrapping_add(add_cmd_modifier(
                buf,
                &raw mut tab_buf as *mut ::core::ffi::c_char,
                multi_mods,
            ));
        }
    }
    if (*cmod).cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            b"topleft\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            multi_mods,
        ));
    }
    if (*cmod).cmod_split & WSP_VERT as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            b"vertical\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            multi_mods,
        ));
    }
    if (*cmod).cmod_split & WSP_HOR as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            b"horizontal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            multi_mods,
        ));
    }
    return result;
}
pub unsafe extern "C" fn uc_mods(
    mut buf: *mut ::core::ffi::c_char,
    mut cmod: *const cmdmod_T,
    mut quote: bool,
) -> size_t {
    let mut result: size_t = 0 as size_t;
    let mut multi_mods: bool = false_0 != 0;
    static mod_entries: GlobalCell<[mod_entry_T; 12]> = GlobalCell::new([
        mod_entry_T {
            flag: CMOD_BROWSE as ::core::ffi::c_int,
            name: b"browse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_CONFIRM as ::core::ffi::c_int,
            name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_HIDE as ::core::ffi::c_int,
            name: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPALT as ::core::ffi::c_int,
            name: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPJUMPS as ::core::ffi::c_int,
            name: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPMARKS as ::core::ffi::c_int,
            name: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPPATTERNS as ::core::ffi::c_int,
            name: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_LOCKMARKS as ::core::ffi::c_int,
            name: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_NOSWAPFILE as ::core::ffi::c_int,
            name: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_UNSILENT as ::core::ffi::c_int,
            name: b"unsilent\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_NOAUTOCMD as ::core::ffi::c_int,
            name: b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_SANDBOX as ::core::ffi::c_int,
            name: b"sandbox\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
    ]);
    result = (if quote as ::core::ffi::c_int != 0 {
        2 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as size_t;
    if !buf.is_null() {
        if quote {
            let c2rust_fresh16 = buf;
            buf = buf.offset(1);
            *c2rust_fresh16 = '"' as ::core::ffi::c_char;
        }
        *buf = NUL as ::core::ffi::c_char;
    }
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[mod_entry_T; 12]>()
        .wrapping_div(::core::mem::size_of::<mod_entry_T>())
        .wrapping_div(
            (::core::mem::size_of::<[mod_entry_T; 12]>()
                .wrapping_rem(::core::mem::size_of::<mod_entry_T>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        if (*cmod).cmod_flags & (*mod_entries.ptr())[i as usize].flag != 0 {
            result = result.wrapping_add(add_cmd_modifier(
                buf,
                (*mod_entries.ptr())[i as usize].name,
                &raw mut multi_mods,
            ));
        }
        i = i.wrapping_add(1);
    }
    if (*cmod).cmod_flags & CMOD_SILENT as ::core::ffi::c_int != 0 {
        result = result.wrapping_add(add_cmd_modifier(
            buf,
            (if (*cmod).cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int != 0 {
                b"silent!\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"silent\0".as_ptr() as *const ::core::ffi::c_char
            }) as *mut ::core::ffi::c_char,
            &raw mut multi_mods,
        ));
    }
    if (*cmod).cmod_verbose > 0 as ::core::ffi::c_int {
        let mut verbose_value: ::core::ffi::c_int = (*cmod).cmod_verbose - 1 as ::core::ffi::c_int;
        if verbose_value == 1 as ::core::ffi::c_int {
            result = result.wrapping_add(add_cmd_modifier(
                buf,
                b"verbose\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                &raw mut multi_mods,
            ));
        } else {
            let mut verbose_buf: [::core::ffi::c_char; 65] = [0; 65];
            snprintf(
                &raw mut verbose_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                b"%dverbose\0".as_ptr() as *const ::core::ffi::c_char,
                verbose_value,
            );
            result = result.wrapping_add(add_cmd_modifier(
                buf,
                &raw mut verbose_buf as *mut ::core::ffi::c_char,
                &raw mut multi_mods,
            ));
        }
    }
    result = result.wrapping_add(add_win_cmd_modifiers(buf, cmod, &raw mut multi_mods));
    if quote as ::core::ffi::c_int != 0 && !buf.is_null() {
        buf = buf.add(result.wrapping_sub(2 as size_t));
        *buf = '"' as ::core::ffi::c_char;
    }
    return result;
}
unsafe extern "C" fn uc_check_code(
    mut code: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut buf: *mut ::core::ffi::c_char,
    mut cmd: *mut ucmd_T,
    mut eap: *mut exarg_T,
    mut split_buf: *mut *mut ::core::ffi::c_char,
    mut split_len: *mut size_t,
) -> size_t {
    let mut result: size_t = 0 as size_t;
    let mut p: *mut ::core::ffi::c_char = code.offset(1 as ::core::ffi::c_int as isize);
    let mut l: size_t = len.wrapping_sub(2 as size_t);
    let mut quote: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut type_0: C2Rust_Unnamed_21 = ct_NONE;
    if !vim_strchr(
        b"qQfF\0".as_ptr() as *const ::core::ffi::c_char,
        *p as uint8_t as ::core::ffi::c_int,
    )
    .is_null()
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
    {
        quote = if *p as ::core::ffi::c_int == 'q' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == 'Q' as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            2 as ::core::ffi::c_int
        };
        p = p.offset(2 as ::core::ffi::c_int as isize);
        l = l.wrapping_sub(2 as size_t);
    }
    l = l.wrapping_add(1);
    if l > 1 as size_t {
        if strncasecmp(
            p,
            b"args>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_ARGS;
        } else if strncasecmp(
            p,
            b"bang>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_BANG;
        } else if strncasecmp(
            p,
            b"count>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_COUNT;
        } else if strncasecmp(
            p,
            b"line1>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_LINE1;
        } else if strncasecmp(
            p,
            b"line2>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_LINE2;
        } else if strncasecmp(
            p,
            b"range>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_RANGE;
        } else if strncasecmp(
            p,
            b"lt>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_LT;
        } else if strncasecmp(
            p,
            b"reg>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
            || strncasecmp(
                p,
                b"register>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                l,
            ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_REGISTER;
        } else if strncasecmp(
            p,
            b"mods>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            l,
        ) == 0 as ::core::ffi::c_int
        {
            type_0 = ct_MODS;
        }
    }
    match type_0 as ::core::ffi::c_uint {
        0 => {
            if *(*eap).arg as ::core::ffi::c_int == NUL {
                if quote == 1 as ::core::ffi::c_int {
                    result = 2 as size_t;
                    if !buf.is_null() {
                        strcpy(
                            buf,
                            b"''\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                    }
                } else {
                    result = 0 as size_t;
                }
            } else {
                if (*eap).argt & EX_NOSPC as uint32_t != 0 && quote == 2 as ::core::ffi::c_int {
                    quote = 1 as ::core::ffi::c_int;
                }
                match quote {
                    0 => {
                        result = strlen((*eap).arg);
                        if !buf.is_null() {
                            strcpy(buf, (*eap).arg);
                        }
                    }
                    1 => {
                        result = strlen((*eap).arg).wrapping_add(2 as size_t);
                        p = (*eap).arg;
                        while *p != 0 {
                            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                || *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                            {
                                result = result.wrapping_add(1);
                            }
                            p = p.offset(1);
                        }
                        if !buf.is_null() {
                            let c2rust_fresh18 = buf;
                            buf = buf.offset(1);
                            *c2rust_fresh18 = '"' as ::core::ffi::c_char;
                            p = (*eap).arg;
                            while *p != 0 {
                                if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                                    || *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                                {
                                    let c2rust_fresh19 = buf;
                                    buf = buf.offset(1);
                                    *c2rust_fresh19 = '\\' as ::core::ffi::c_char;
                                }
                                let c2rust_fresh20 = buf;
                                buf = buf.offset(1);
                                *c2rust_fresh20 = *p;
                                p = p.offset(1);
                            }
                            *buf = '"' as ::core::ffi::c_char;
                        }
                    }
                    2 => {
                        if (*split_buf).is_null() {
                            *split_buf = uc_split_args(
                                (*eap).arg,
                                (*eap).args,
                                (*eap).arglens,
                                (*eap).argc,
                                split_len,
                            );
                        }
                        result = *split_len;
                        if !buf.is_null() && result != 0 as size_t {
                            strcpy(buf, *split_buf);
                        }
                    }
                    _ => {}
                }
            }
        }
        1 => {
            result = (if (*eap).forceit != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as size_t;
            if quote != 0 {
                result = result.wrapping_add(2 as size_t);
            }
            if !buf.is_null() {
                if quote != 0 {
                    let c2rust_fresh21 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh21 = '"' as ::core::ffi::c_char;
                }
                if (*eap).forceit != 0 {
                    let c2rust_fresh22 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh22 = '!' as ::core::ffi::c_char;
                }
                if quote != 0 {
                    *buf = '"' as ::core::ffi::c_char;
                }
            }
        }
        3 | 4 | 5 | 2 => {
            let mut num_buf: [::core::ffi::c_char; 20] = [0; 20];
            let mut num: int64_t = if type_0 as ::core::ffi::c_uint
                == ct_LINE1 as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*eap).line1 as int64_t
            } else if type_0 as ::core::ffi::c_uint
                == ct_LINE2 as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*eap).line2 as int64_t
            } else if type_0 as ::core::ffi::c_uint
                == ct_RANGE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*eap).addr_count as int64_t
            } else if (*eap).addr_count > 0 as ::core::ffi::c_int {
                (*eap).line2 as int64_t
            } else {
                (*cmd).uc_def
            };
            let mut num_len: size_t = 0;
            snprintf(
                &raw mut num_buf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                num,
            );
            num_len = strlen(&raw mut num_buf as *mut ::core::ffi::c_char);
            result = num_len;
            if quote != 0 {
                result = result.wrapping_add(2 as size_t);
            }
            if !buf.is_null() {
                if quote != 0 {
                    let c2rust_fresh23 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh23 = '"' as ::core::ffi::c_char;
                }
                strcpy(buf, &raw mut num_buf as *mut ::core::ffi::c_char);
                buf = buf.add(num_len);
                if quote != 0 {
                    *buf = '"' as ::core::ffi::c_char;
                }
            }
        }
        6 => {
            result = uc_mods(buf, cmdmod.ptr(), quote != 0);
        }
        7 => {
            result = (if (*eap).regname != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as size_t;
            if quote != 0 {
                result = result.wrapping_add(2 as size_t);
            }
            if !buf.is_null() {
                if quote != 0 {
                    let c2rust_fresh24 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh24 = '\'' as ::core::ffi::c_char;
                }
                if (*eap).regname != 0 {
                    let c2rust_fresh25 = buf;
                    buf = buf.offset(1);
                    *c2rust_fresh25 = (*eap).regname as ::core::ffi::c_char;
                }
                if quote != 0 {
                    *buf = '\'' as ::core::ffi::c_char;
                }
            }
        }
        8 => {
            result = 1 as size_t;
            if !buf.is_null() {
                *buf = '<' as ::core::ffi::c_char;
            }
        }
        _ => {
            result = -1 as ::core::ffi::c_int as size_t;
            if !buf.is_null() {
                *buf = '<' as ::core::ffi::c_char;
            }
        }
    }
    return result;
}
pub unsafe extern "C" fn do_ucmd(mut eap: *mut exarg_T, mut preview: bool) -> ::core::ffi::c_int {
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut split_len: size_t = 0 as size_t;
    let mut split_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut cmd: *mut ucmd_T = ::core::ptr::null_mut::<ucmd_T>();
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_USER as ::core::ffi::c_int {
        cmd = ((*ucmds.ptr()).ga_data as *mut ucmd_T).offset((*eap).useridx as isize);
    } else {
        cmd = ((*(*(prevwin_curwin as unsafe extern "C" fn() -> *mut win_T)()).w_buffer)
            .b_ucmds
            .ga_data as *mut ucmd_T)
            .offset((*eap).useridx as isize);
    }
    if preview {
        debug_assert!(
            (*cmd).uc_preview_luaref > 0 as ::core::ffi::c_int,
            "cmd->uc_preview_luaref > 0"
        );
        return nlua_do_ucmd(cmd, eap, true_0 != 0);
    }
    if (*cmd).uc_luaref > 0 as ::core::ffi::c_int {
        nlua_do_ucmd(cmd, eap, false_0 != 0);
        return 0 as ::core::ffi::c_int;
    }
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    loop {
        let mut p: *mut ::core::ffi::c_char = (*cmd).uc_rep;
        let mut q: *mut ::core::ffi::c_char = buf;
        let mut totlen: size_t = 0 as size_t;
        loop {
            let mut start: *mut ::core::ffi::c_char = vim_strchr(p, '<' as ::core::ffi::c_int);
            if !start.is_null() {
                end = vim_strchr(
                    start.offset(1 as ::core::ffi::c_int as isize),
                    '>' as ::core::ffi::c_int,
                );
            }
            if !buf.is_null() {
                let mut ksp: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                ksp = p;
                while *ksp as ::core::ffi::c_int != NUL
                    && *ksp as uint8_t as ::core::ffi::c_int != K_SPECIAL
                {
                    ksp = ksp.offset(1);
                }
                if *ksp as uint8_t as ::core::ffi::c_int == K_SPECIAL
                    && (start.is_null() || ksp < start || end.is_null())
                    && (*ksp.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                        as ::core::ffi::c_int
                        == KS_SPECIAL
                        && *ksp.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == KE_FILLER)
                {
                    let mut len: size_t = ksp.offset_from(p) as size_t;
                    if len > 0 as size_t {
                        memmove(
                            q as *mut ::core::ffi::c_void,
                            p as *const ::core::ffi::c_void,
                            len,
                        );
                        q = q.add(len);
                    }
                    let c2rust_fresh17 = q;
                    q = q.offset(1);
                    *c2rust_fresh17 = K_SPECIAL as ::core::ffi::c_char;
                    p = ksp.offset(3 as ::core::ffi::c_int as isize);
                    continue;
                }
            }
            if start.is_null() || end.is_null() {
                break;
            }
            end = end.offset(1);
            let mut len_0: size_t = start.offset_from(p) as size_t;
            if buf.is_null() {
                totlen = totlen.wrapping_add(len_0);
            } else {
                memmove(
                    q as *mut ::core::ffi::c_void,
                    p as *const ::core::ffi::c_void,
                    len_0,
                );
                q = q.add(len_0);
            }
            len_0 = uc_check_code(
                start,
                end.offset_from(start) as size_t,
                q,
                cmd,
                eap,
                &raw mut split_buf,
                &raw mut split_len,
            );
            if len_0 == -1 as ::core::ffi::c_int as size_t {
                p = start.offset(1 as ::core::ffi::c_int as isize);
                len_0 = 1 as size_t;
            } else {
                p = end;
            }
            if buf.is_null() {
                totlen = totlen.wrapping_add(len_0);
            } else {
                q = q.add(len_0);
            }
        }
        if !buf.is_null() {
            strcpy(q, p);
            break;
        } else {
            totlen = totlen.wrapping_add(strlen(p));
            buf = xmalloc(totlen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
    }
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut restore_current_sctx: bool = false_0 != 0;
    if (*cmd).uc_argt & EX_KEEPSCRIPT as uint32_t == 0 as uint32_t {
        restore_current_sctx = true_0 != 0;
        save_current_sctx = current_sctx.get();
        (*current_sctx.ptr()).sc_sid = (*cmd).uc_script_ctx.sc_sid;
    }
    do_cmdline(
        buf,
        (*eap).ea_getline,
        (*eap).cookie,
        DOCMD_VERBOSE as ::core::ffi::c_int
            | DOCMD_NOWAIT as ::core::ffi::c_int
            | DOCMD_KEYTYPED as ::core::ffi::c_int,
    );
    if restore_current_sctx {
        current_sctx.set(save_current_sctx);
    }
    xfree(buf as *mut ::core::ffi::c_void);
    xfree(split_buf as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn commands_array(mut buf: *mut buf_T, mut arena: *mut Arena) -> Dict {
    let mut gap: *mut garray_T = if buf.is_null() {
        ucmds.ptr()
    } else {
        &raw mut (*buf).b_ucmds
    };
    let mut rv: Dict = arena_dict(arena, (*gap).ga_len as size_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*gap).ga_len {
        let mut arg: [::core::ffi::c_char; 2] =
            [0 as ::core::ffi::c_char, 0 as ::core::ffi::c_char];
        let mut d: Dict = arena_dict(arena, 16 as size_t);
        let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
        let c2rust_fresh45 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh45) = key_value_pair {
            key: cstr_as_string(b"name\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*cmd).uc_name),
                },
            },
        };
        let c2rust_fresh46 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh46) = key_value_pair {
            key: cstr_as_string(b"definition\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_as_string((*cmd).uc_rep),
                },
            },
        };
        let c2rust_fresh47 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh47) = key_value_pair {
            key: cstr_as_string(b"script_id\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (*cmd).uc_script_ctx.sc_sid as Integer,
                },
            },
        };
        let c2rust_fresh48 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh48) = key_value_pair {
            key: cstr_as_string(b"bang\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x2 as uint32_t != 0,
                },
            },
        };
        let c2rust_fresh49 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh49) = key_value_pair {
            key: cstr_as_string(b"bar\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x100 as uint32_t != 0,
                },
            },
        };
        let c2rust_fresh50 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh50) = key_value_pair {
            key: cstr_as_string(b"register\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x200 as uint32_t != 0,
                },
            },
        };
        let c2rust_fresh51 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh51) = key_value_pair {
            key: cstr_as_string(b"keepscript\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed {
                    boolean: (*cmd).uc_argt & 0x4000000 as uint32_t != 0,
                },
            },
        };
        if (*cmd).uc_preview_luaref != LUA_NOREF {
            let c2rust_fresh52 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.add(c2rust_fresh52) = key_value_pair {
                key: cstr_as_string(b"preview\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*cmd).uc_preview_luaref),
                    },
                },
            };
        }
        if (*cmd).uc_luaref != LUA_NOREF {
            let c2rust_fresh53 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.add(c2rust_fresh53) = key_value_pair {
                key: cstr_as_string(b"callback\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*cmd).uc_luaref),
                    },
                },
            };
        }
        match (*cmd).uc_argt
            & (EX_EXTRA as uint32_t | EX_NOSPC as uint32_t | EX_NEEDARG as uint32_t)
        {
            0 => {
                arg[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
            }
            4 => {
                arg[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
            }
            20 => {
                arg[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
            }
            132 => {
                arg[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
            }
            148 => {
                arg[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
            }
            _ => {}
        }
        let c2rust_fresh54 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh54) = key_value_pair {
            key: cstr_as_string(b"nargs\0".as_ptr() as *const ::core::ffi::c_char),
            value: object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: arena_string(
                        arena,
                        cstr_as_string(&raw mut arg as *mut ::core::ffi::c_char),
                    ),
                },
            },
        };
        if (*cmd).uc_compl_luaref != LUA_NOREF {
            let c2rust_fresh55 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.add(c2rust_fresh55) = key_value_pair {
                key: cstr_as_string(b"complete\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed {
                        luaref: api_new_luaref((*cmd).uc_compl_luaref),
                    },
                },
            };
        } else {
            let mut cmd_compl: *mut ::core::ffi::c_char = get_command_complete((*cmd).uc_compl);
            let c2rust_fresh56 = d.size;
            d.size = d.size.wrapping_add(1);
            *d.items.add(c2rust_fresh56) = key_value_pair {
                key: cstr_as_string(b"complete\0".as_ptr() as *const ::core::ffi::c_char),
                value: if cmd_compl.is_null() {
                    object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    }
                } else {
                    object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed {
                            string: cstr_as_string(cmd_compl),
                        },
                    }
                },
            };
        }
        let c2rust_fresh57 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh57) = key_value_pair {
            key: cstr_as_string(b"complete_arg\0".as_ptr() as *const ::core::ffi::c_char),
            value: if (*cmd).uc_compl_arg.is_null() {
                object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed { boolean: false },
                }
            } else {
                object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string((*cmd).uc_compl_arg),
                    },
                }
            },
        };
        let mut obj: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if (*cmd).uc_argt & EX_COUNT as uint32_t != 0 {
            if (*cmd).uc_def >= 0 as int64_t {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_printf(
                            arena,
                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        ),
                    },
                };
            } else {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(b"0\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                };
            }
        }
        let c2rust_fresh58 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh58) = key_value_pair {
            key: cstr_as_string(b"count\0".as_ptr() as *const ::core::ffi::c_char),
            value: obj,
        };
        obj = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if (*cmd).uc_argt & EX_RANGE as uint32_t != 0 {
            if (*cmd).uc_argt & EX_DFLALL as uint32_t != 0 {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: b"%\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                };
            } else if (*cmd).uc_def >= 0 as int64_t {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_printf(
                            arena,
                            b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                            (*cmd).uc_def,
                        ),
                    },
                };
            } else {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: String_0 {
                            data: b".\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            size: ::core::mem::size_of::<[::core::ffi::c_char; 2]>()
                                .wrapping_sub(1 as size_t),
                        },
                    },
                };
            }
        }
        let c2rust_fresh59 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh59) = key_value_pair {
            key: cstr_as_string(b"range\0".as_ptr() as *const ::core::ffi::c_char),
            value: obj,
        };
        obj = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (*addr_type_complete.ptr())[j as usize].expand as ::core::ffi::c_uint
            != ADDR_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*addr_type_complete.ptr())[j as usize].expand as ::core::ffi::c_uint
                != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*addr_type_complete.ptr())[j as usize].expand as ::core::ffi::c_uint
                    == (*cmd).uc_addr_type as ::core::ffi::c_uint
            {
                obj = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string((*addr_type_complete.ptr())[j as usize].name),
                    },
                };
                break;
            } else {
                j += 1;
            }
        }
        let c2rust_fresh60 = d.size;
        d.size = d.size.wrapping_add(1);
        *d.items.add(c2rust_fresh60) = key_value_pair {
            key: cstr_as_string(b"addr\0".as_ptr() as *const ::core::ffi::c_char),
            value: obj,
        };
        let c2rust_fresh61 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.add(c2rust_fresh61) = key_value_pair {
            key: cstr_as_string((*cmd).uc_name),
            value: object {
                type_0: kObjectTypeDict,
                data: C2Rust_Unnamed { dict: d },
            },
        };
        i += 1;
    }
    return rv;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
