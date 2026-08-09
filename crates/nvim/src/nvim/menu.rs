use crate::semsg_c;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::autocmd::{EVENT_MENUPOPUP, apply_autocmds};
use crate::src::nvim::charset::{getdigits_int, skipwhite};
use crate::src::nvim::cursor::{check_cursor, gchar_cursor};
use crate::src::nvim::eval::typval::tv_dict_len;
use crate::src::nvim::eval::typval::{
    tv_dict_add_allocated_str, tv_dict_add_bool, tv_dict_add_dict, tv_dict_add_list,
    tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_alloc_ret, tv_get_string_chk,
    tv_list_alloc, tv_list_append_dict, tv_list_append_string,
};
use crate::src::nvim::eval::vars::del_menutrans_vars;
use crate::src::nvim::ex_docmd::{
    ends_excmd, exec_normal_cmd, restore_current_state, save_current_state,
};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::src::nvim::getchar::ins_typebuf;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::highlight_group::{HLF_8, HLF_D};
use crate::src::nvim::keycodes::{Ctrl_BSL, Ctrl_C, Ctrl_G, Ctrl_O, Ctrl_V, replace_termcodes};
use crate::src::nvim::main::{
    State, VIsual, VIsual_active, VIsual_mode, VIsual_reselect, VIsual_select, curbuf,
    current_sctx, curwin, e_cannot_change_menus_while_listing, e_invarg, e_invarg2,
    e_menu_only_exists_in_another_mode, e_trailing_arg, ex_normal_busy, finish_op, got_int, p_cpo,
    p_sel, restart_edit, root_menu, sys_menu,
};
use crate::src::nvim::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xmemdupz, xstrdup, xstrlcpy};
use crate::src::nvim::message::{
    emsg, msg_outnum, msg_outtrans, msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, str2special_save,
};
use crate::src::nvim::os::libc::{
    gettext, memmove, strcasecmp, strcat, strcmp, strcpy, strlen, strncasecmp, strncmp,
};
use crate::src::nvim::popupmenu::pum_show_popupmenu;
use crate::src::nvim::pos::MAXCOL;
use crate::src::nvim::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_TERMINAL, MODE_VISUAL, get_real_state,
};
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
use crate::src::nvim::types::{
    AlignTextPos, BoolVarValue, EvalFuncData, RemapValues, String_0, TriState, VAR_UNKNOWN,
    WinSplit, WinStyle, buffblock, buffblock_T, buffheader_T, colnr_T, dict_T, exarg_T, expand_T,
    garray_T, kFalse, kListLenMayKnow, kNone, kTrue, linenr_T, list_T, pos_T, ptrdiff_t,
    save_state_T, scid_T, size_t, ssize_t, tasave_T, typebuf_T, typval_T, uint8_t, varnumber_T,
    vimmenu_T,
};
use crate::src::nvim::ui::ui_call_update_menu;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const MENU_MODES: C2Rust_Unnamed_16 = 8;
pub const MENU_INDEX_TIP: C2Rust_Unnamed_16 = 7;
pub const MENU_INDEX_TERMINAL: C2Rust_Unnamed_16 = 6;
pub const MENU_INDEX_CMDLINE: C2Rust_Unnamed_16 = 5;
pub const MENU_INDEX_INSERT: C2Rust_Unnamed_16 = 4;
pub const MENU_INDEX_OP_PENDING: C2Rust_Unnamed_16 = 3;
pub const MENU_INDEX_SELECT: C2Rust_Unnamed_16 = 2;
pub const MENU_INDEX_VISUAL: C2Rust_Unnamed_16 = 1;
pub const MENU_INDEX_NORMAL: C2Rust_Unnamed_16 = 0;
pub const MENU_INDEX_INVALID: C2Rust_Unnamed_16 = -1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MENU_ALL_MODES: C2Rust_Unnamed_17 = 127;
pub const MENU_TIP_MODE: C2Rust_Unnamed_17 = 128;
pub const MENU_TERMINAL_MODE: C2Rust_Unnamed_17 = 64;
pub const MENU_CMDLINE_MODE: C2Rust_Unnamed_17 = 32;
pub const MENU_INSERT_MODE: C2Rust_Unnamed_17 = 16;
pub const MENU_OP_PENDING_MODE: C2Rust_Unnamed_17 = 8;
pub const MENU_SELECT_MODE: C2Rust_Unnamed_17 = 4;
pub const MENU_VISUAL_MODE: C2Rust_Unnamed_17 = 2;
pub const MENU_NORMAL_MODE: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const REPTERM_DO_LT: C2Rust_Unnamed_19 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menutrans_T {
    pub from: *mut ::core::ffi::c_char,
    pub from_noamp: *mut ::core::ffi::c_char,
    pub to: *mut ::core::ffi::c_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MNU_HIDDEN_CHAR: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const MENUDEPTH: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
static menus_locked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static menu_mode_chars: GlobalCell<[*mut ::core::ffi::c_char; 8]> = GlobalCell::new([
    b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"v\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"s\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"o\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"i\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"tl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    b"t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
]);
static e_notsubmenu: GlobalCell<[::core::ffi::c_char; 45]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 45], [::core::ffi::c_char; 45]>(
        *b"E327: Part of menu-item path is not sub-menu\0",
    )
});
static e_nomenu: GlobalCell<[::core::ffi::c_char; 19]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 19], [::core::ffi::c_char; 19]>(*b"E329: No menu \"%s\"\0")
});
unsafe extern "C" fn menu_is_winbar(name: *const ::core::ffi::c_char) -> bool {
    return strncmp(
        name,
        b"WinBar\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn get_root_menu(_name: *const ::core::ffi::c_char) -> *mut *mut vimmenu_T {
    return root_menu.ptr();
}
unsafe extern "C" fn is_menus_locked() -> ::core::ffi::c_int {
    if menus_locked.get() > 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_cannot_change_menus_while_listing as *const ::core::ffi::c_char,
        ));
        return true_0;
    }
    return false_0;
}
pub unsafe fn ex_menu(mut eap: *mut exarg_T) {
    let mut map_to: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut noremap: ::core::ffi::c_int = 0;
    let mut silent: bool = false_0 != 0;
    let mut unmenu: bool = false;
    let mut map_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0;
    let mut pri_tab: [::core::ffi::c_int; 11] = [0; 11];
    let mut enable: TriState = kNone;
    let mut menuarg: vimmenu_T = vimmenu_T {
        modes: 0,
        enabled: 0,
        name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        dname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        en_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        en_dname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        mnemonic: 0,
        actext: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        priority: 0,
        strings: [::core::ptr::null_mut::<::core::ffi::c_char>(); 8],
        noremap: [0; 8],
        silent: [false; 8],
        children: ::core::ptr::null_mut::<vimmenu_T>(),
        parent: ::core::ptr::null_mut::<vimmenu_T>(),
        next: ::core::ptr::null_mut::<vimmenu_T>(),
    };
    let mut modes: ::core::ffi::c_int = get_menu_cmd_modes(
        (*eap).cmd,
        (*eap).forceit != 0,
        &raw mut noremap,
        &raw mut unmenu,
    );
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    loop {
        if strncmp(
            arg,
            b"<script>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            noremap = REMAP_SCRIPT as ::core::ffi::c_int;
            arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
        } else if strncmp(
            arg,
            b"<silent>\0".as_ptr() as *const ::core::ffi::c_char,
            8 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            silent = true_0 != 0;
            arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
        } else {
            if strncmp(
                arg,
                b"<special>\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) != 0 as ::core::ffi::c_int
            {
                break;
            }
            arg = skipwhite(arg.offset(9 as ::core::ffi::c_int as isize));
        }
    }
    if strncmp(
        arg,
        b"icon=\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        arg = arg.offset(5 as ::core::ffi::c_int as isize);
        while *arg as ::core::ffi::c_int != NUL
            && *arg as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
        {
            if *arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                memmove(
                    arg as *mut ::core::ffi::c_void,
                    arg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(arg.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            arg = arg.offset(utfc_ptr2len(arg) as isize);
        }
        if *arg as ::core::ffi::c_int != NUL {
            let c2rust_fresh0 = arg;
            arg = arg.offset(1);
            *c2rust_fresh0 = NUL as ::core::ffi::c_char;
            arg = skipwhite(arg);
        }
    }
    p = arg;
    while *p != 0 {
        if !ascii_isdigit(*p as ::core::ffi::c_int)
            && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if ascii_iswhite(*p as ::core::ffi::c_int) {
        i = 0 as ::core::ffi::c_int;
        while i < MENUDEPTH && !ascii_iswhite(*arg as ::core::ffi::c_int) {
            pri_tab[i as usize] =
                getdigits_int(&raw mut arg, false_0 != 0, 0 as ::core::ffi::c_int);
            if pri_tab[i as usize] == 0 as ::core::ffi::c_int {
                pri_tab[i as usize] = 500 as ::core::ffi::c_int;
            }
            if *arg as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                arg = arg.offset(1);
            }
            i += 1;
        }
        arg = skipwhite(arg);
    } else if (*eap).addr_count != 0 && (*eap).line2 != 0 as linenr_T {
        pri_tab[0 as ::core::ffi::c_int as usize] = (*eap).line2 as ::core::ffi::c_int;
        i = 1 as ::core::ffi::c_int;
    } else {
        i = 0 as ::core::ffi::c_int;
    }
    while i < MENUDEPTH {
        let c2rust_fresh1 = i;
        i = i + 1;
        pri_tab[c2rust_fresh1 as usize] = 500 as ::core::ffi::c_int;
    }
    pri_tab[MENUDEPTH as usize] = -1 as ::core::ffi::c_int;
    if strncmp(
        arg,
        b"enable\0".as_ptr() as *const ::core::ffi::c_char,
        6 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        enable = kTrue;
        arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
    } else if strncmp(
        arg,
        b"disable\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ascii_iswhite(*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        enable = kFalse;
        arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
    }
    if *arg as ::core::ffi::c_int == NUL {
        show_menus(arg, modes);
        return;
    }
    let mut menu_path: *mut ::core::ffi::c_char = arg;
    's_573: {
        if *menu_path as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            semsg_c!(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                menu_path,
            );
        } else {
            map_to = menu_translate_tab_and_shift(arg);
            if *map_to as ::core::ffi::c_int == NUL
                && !unmenu
                && enable as ::core::ffi::c_int == kNone as ::core::ffi::c_int
            {
                show_menus(menu_path, modes);
            } else if *map_to as ::core::ffi::c_int != NUL
                && (unmenu as ::core::ffi::c_int != 0
                    || enable as ::core::ffi::c_int != kNone as ::core::ffi::c_int)
            {
                semsg_c!(
                    gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                    map_to,
                );
            } else {
                let mut root_menu_ptr: *mut *mut vimmenu_T = get_root_menu(menu_path);
                if enable as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                    if strcmp(menu_path, b"*\0".as_ptr() as *const ::core::ffi::c_char)
                        == 0 as ::core::ffi::c_int
                    {
                        menu_path = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                    }
                    if menu_is_popup(menu_path) {
                        i = 0 as ::core::ffi::c_int;
                        while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                            if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                p = popup_mode_name(menu_path, i);
                                menu_enable_recurse(
                                    *root_menu_ptr,
                                    p,
                                    MENU_ALL_MODES as ::core::ffi::c_int,
                                    enable as ::core::ffi::c_int,
                                );
                                xfree(p as *mut ::core::ffi::c_void);
                            }
                            i += 1;
                        }
                    }
                    menu_enable_recurse(
                        *root_menu_ptr,
                        menu_path,
                        modes,
                        enable as ::core::ffi::c_int,
                    );
                } else if unmenu {
                    if is_menus_locked() != 0 {
                        break 's_573;
                    } else {
                        if strcmp(menu_path, b"*\0".as_ptr() as *const ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                        {
                            menu_path = b"\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char;
                        }
                        if menu_is_popup(menu_path) {
                            i = 0 as ::core::ffi::c_int;
                            while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                                if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                    p = popup_mode_name(menu_path, i);
                                    remove_menu(
                                        root_menu_ptr,
                                        p,
                                        MENU_ALL_MODES as ::core::ffi::c_int,
                                        true_0 != 0,
                                    );
                                    xfree(p as *mut ::core::ffi::c_void);
                                }
                                i += 1;
                            }
                        }
                        remove_menu(root_menu_ptr, menu_path, modes, false_0 != 0);
                    }
                } else if is_menus_locked() != 0 {
                    break 's_573;
                } else {
                    if strcasecmp(
                        map_to,
                        b"<nop>\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    ) == 0 as ::core::ffi::c_int
                    {
                        map_to = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                        map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    } else {
                        map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        map_to = replace_termcodes(
                            map_to,
                            strlen(map_to),
                            &raw mut map_buf,
                            0 as scid_T,
                            REPTERM_DO_LT as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<bool>(),
                            p_cpo.get(),
                        );
                    }
                    menuarg.modes = modes;
                    menuarg.noremap[0 as ::core::ffi::c_int as usize] = noremap;
                    menuarg.silent[0 as ::core::ffi::c_int as usize] = silent;
                    add_menu_path(
                        menu_path,
                        &raw mut menuarg,
                        &raw mut pri_tab as *mut ::core::ffi::c_int,
                        map_to,
                    );
                    if menu_is_popup(menu_path) {
                        i = 0 as ::core::ffi::c_int;
                        while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                            if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                p = popup_mode_name(menu_path, i);
                                menuarg.modes = modes;
                                add_menu_path(
                                    p,
                                    &raw mut menuarg,
                                    &raw mut pri_tab as *mut ::core::ffi::c_int,
                                    map_to,
                                );
                                xfree(p as *mut ::core::ffi::c_void);
                            }
                            i += 1;
                        }
                    }
                    xfree(map_buf as *mut ::core::ffi::c_void);
                }
                ui_call_update_menu();
            }
        }
    };
}
unsafe extern "C" fn add_menu_path(
    menu_path: *const ::core::ffi::c_char,
    mut menuarg: *mut vimmenu_T,
    pri_tab: *const ::core::ffi::c_int,
    call_data: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut amenu: ::core::ffi::c_int = 0;
    let mut modes: ::core::ffi::c_int = (*menuarg).modes;
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    let mut lower_pri: *mut *mut vimmenu_T = ::core::ptr::null_mut::<*mut vimmenu_T>();
    let mut dname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut pri_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut old_modes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut en_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path_name: *mut ::core::ffi::c_char = xstrdup(menu_path);
    let mut root_menu_ptr: *mut *mut vimmenu_T = get_root_menu(menu_path);
    let mut menup: *mut *mut vimmenu_T = root_menu_ptr;
    let mut parent: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    let mut name: *mut ::core::ffi::c_char = path_name;
    '_erret: {
        while *name != 0 {
            let mut next_name: *mut ::core::ffi::c_char = menu_name_skip(name);
            let mut map_to: *mut ::core::ffi::c_char =
                menutrans_lookup(name, strlen(name) as ::core::ffi::c_int);
            if !map_to.is_null() {
                en_name = name;
                name = map_to;
            } else {
                en_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            dname = menu_text(
                name,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            if *dname as ::core::ffi::c_int == NUL {
                emsg(gettext(
                    b"E792: Empty menu name\0".as_ptr() as *const ::core::ffi::c_char
                ));
                break '_erret;
            } else {
                lower_pri = menup;
                menu = *menup;
                while !menu.is_null() {
                    if menu_name_equal(name, menu) as ::core::ffi::c_int != 0
                        || menu_name_equal(dname, menu) as ::core::ffi::c_int != 0
                    {
                        if *next_name as ::core::ffi::c_int == NUL && !(*menu).children.is_null() {
                            if !sys_menu.get() {
                                emsg(gettext(
                                    b"E330: Menu path must not lead to a sub-menu\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            }
                            break '_erret;
                        } else {
                            if !(*next_name as ::core::ffi::c_int != NUL
                                && (*menu).children.is_null())
                            {
                                break;
                            }
                            if !sys_menu.get() {
                                emsg(gettext(
                                    (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                                ));
                            }
                            break '_erret;
                        }
                    } else {
                        menup = &raw mut (*menu).next;
                        if !parent.is_null()
                            || menu_is_menubar((*menu).name) as ::core::ffi::c_int != 0
                        {
                            if (*menu).priority <= *pri_tab.offset(pri_idx as isize) {
                                lower_pri = menup;
                            }
                        }
                        menu = (*menu).next;
                    }
                }
                if menu.is_null() {
                    if *next_name as ::core::ffi::c_int == NUL && parent.is_null() {
                        emsg(gettext(
                            b"E331: Must not add menu items directly to menu bar\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        break '_erret;
                    } else if menu_is_separator(dname) as ::core::ffi::c_int != 0
                        && *next_name as ::core::ffi::c_int != NUL
                    {
                        emsg(gettext(
                            b"E332: Separator cannot be part of a menu path\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ));
                        break '_erret;
                    } else {
                        menu = xcalloc(1 as size_t, ::core::mem::size_of::<vimmenu_T>())
                            as *mut vimmenu_T;
                        (*menu).modes = modes;
                        (*menu).enabled = MENU_ALL_MODES as ::core::ffi::c_int;
                        (*menu).name = xstrdup(name);
                        (*menu).dname =
                            menu_text(name, &raw mut (*menu).mnemonic, &raw mut (*menu).actext);
                        if !en_name.is_null() {
                            (*menu).en_name = xstrdup(en_name);
                            (*menu).en_dname = menu_text(
                                en_name,
                                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                            );
                        } else {
                            (*menu).en_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            (*menu).en_dname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        (*menu).priority = *pri_tab.offset(pri_idx as isize);
                        (*menu).parent = parent;
                        (*menu).next = *lower_pri;
                        *lower_pri = menu;
                        old_modes = 0 as ::core::ffi::c_int;
                    }
                } else {
                    old_modes = (*menu).modes;
                    (*menu).modes |= modes;
                    (*menu).enabled |= modes;
                }
                menup = &raw mut (*menu).children;
                parent = menu;
                name = next_name;
                let mut ptr_: *mut *mut ::core::ffi::c_void =
                    &raw mut dname as *mut *mut ::core::ffi::c_void;
                xfree(*ptr_);
                *ptr_ = NULL;
                let _ = *ptr_;
                if *pri_tab.offset((pri_idx + 1 as ::core::ffi::c_int) as isize)
                    != -1 as ::core::ffi::c_int
                {
                    pri_idx += 1;
                }
            }
        }
        xfree(path_name as *mut ::core::ffi::c_void);
        amenu = (modes
            & (MENU_NORMAL_MODE as ::core::ffi::c_int | MENU_INSERT_MODE as ::core::ffi::c_int)
            == MENU_NORMAL_MODE as ::core::ffi::c_int | MENU_INSERT_MODE as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        if sys_menu.get() {
            modes &= !old_modes;
        }
        if !menu.is_null() && modes != 0 {
            let mut p: *mut ::core::ffi::c_char = if call_data.is_null() {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            } else {
                xstrdup(call_data)
            };
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < MENU_MODES as ::core::ffi::c_int {
                if modes & (1 as ::core::ffi::c_int) << i != 0 {
                    free_menu_string(menu, i);
                    let mut c: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                    let mut d: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                    if amenu != 0 && !call_data.is_null() && *call_data as ::core::ffi::c_int != NUL
                    {
                        match (1 as ::core::ffi::c_int) << i {
                            2 | 4 | 8 | 32 => {
                                c = Ctrl_C as ::core::ffi::c_char;
                            }
                            16 => {
                                c = Ctrl_BSL as ::core::ffi::c_char;
                                d = Ctrl_O as ::core::ffi::c_char;
                            }
                            _ => {}
                        }
                    }
                    if c as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                        (*menu).strings[i as usize] =
                            xmalloc(strlen(call_data).wrapping_add(5 as size_t))
                                as *mut ::core::ffi::c_char;
                        *(*menu).strings[i as usize].offset(0 as ::core::ffi::c_int as isize) = c;
                        if d as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                            strcpy(
                                (*menu).strings[i as usize]
                                    .offset(1 as ::core::ffi::c_int as isize),
                                call_data as *mut ::core::ffi::c_char,
                            );
                        } else {
                            *(*menu).strings[i as usize].offset(1 as ::core::ffi::c_int as isize) =
                                d;
                            strcpy(
                                (*menu).strings[i as usize]
                                    .offset(2 as ::core::ffi::c_int as isize),
                                call_data as *mut ::core::ffi::c_char,
                            );
                        }
                        if c as ::core::ffi::c_int == Ctrl_C {
                            let mut len: ::core::ffi::c_int =
                                strlen((*menu).strings[i as usize]) as ::core::ffi::c_int;
                            *(*menu).strings[i as usize].offset(len as isize) =
                                Ctrl_BSL as ::core::ffi::c_char;
                            *(*menu).strings[i as usize]
                                .offset((len + 1 as ::core::ffi::c_int) as isize) =
                                Ctrl_G as ::core::ffi::c_char;
                            *(*menu).strings[i as usize]
                                .offset((len + 2 as ::core::ffi::c_int) as isize) =
                                NUL as ::core::ffi::c_char;
                        }
                    } else {
                        (*menu).strings[i as usize] = p;
                    }
                    (*menu).noremap[i as usize] =
                        (*menuarg).noremap[0 as ::core::ffi::c_int as usize];
                    (*menu).silent[i as usize] =
                        (*menuarg).silent[0 as ::core::ffi::c_int as usize];
                }
                i += 1;
            }
        }
        return OK;
    }
    xfree(path_name as *mut ::core::ffi::c_void);
    xfree(dname as *mut ::core::ffi::c_void);
    while !parent.is_null() && (*parent).children.is_null() {
        if (*parent).parent.is_null() {
            menup = root_menu_ptr;
        } else {
            menup = &raw mut (*(*parent).parent).children;
        }
        while !(*menup).is_null() && *menup != parent {
            menup = &raw mut (**menup).next;
        }
        if (*menup).is_null() {
            break;
        }
        parent = (*parent).parent;
        free_menu(menup);
    }
    return FAIL;
}
unsafe extern "C" fn menu_enable_recurse(
    mut menu: *mut vimmenu_T,
    mut name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut enable: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if menu.is_null() {
        return OK;
    }
    let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
    while !menu.is_null() {
        if *name as ::core::ffi::c_int == NUL
            || *name as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || menu_name_equal(name, menu) as ::core::ffi::c_int != 0
        {
            if *p as ::core::ffi::c_int != NUL {
                if (*menu).children.is_null() {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    return FAIL;
                }
                if menu_enable_recurse((*menu).children, p, modes, enable) == FAIL {
                    return FAIL;
                }
            } else if enable != 0 {
                (*menu).enabled |= modes;
            } else {
                (*menu).enabled &= !modes;
            }
            if *name as ::core::ffi::c_int != NUL
                && *name as ::core::ffi::c_int != '*' as ::core::ffi::c_int
            {
                break;
            }
        }
        menu = (*menu).next;
    }
    if *name as ::core::ffi::c_int != NUL
        && *name as ::core::ffi::c_int != '*' as ::core::ffi::c_int
        && menu.is_null()
    {
        semsg_c!(
            gettext((e_nomenu.ptr() as *const _) as *const ::core::ffi::c_char),
            name,
        );
        return FAIL;
    }
    return OK;
}
unsafe extern "C" fn remove_menu(
    mut menup: *mut *mut vimmenu_T,
    mut name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut silent: bool,
) -> ::core::ffi::c_int {
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    if (*menup).is_null() {
        return OK;
    }
    let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
    loop {
        menu = *menup;
        if menu.is_null() {
            break;
        }
        if *name as ::core::ffi::c_int == NUL
            || menu_name_equal(name, menu) as ::core::ffi::c_int != 0
        {
            if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                if !silent {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                }
                return FAIL;
            }
            if (*menu).modes & modes != 0 as ::core::ffi::c_int {
                if remove_menu(&raw mut (*menu).children, p, modes, silent) == FAIL {
                    return FAIL;
                }
            } else if *name as ::core::ffi::c_int != NUL {
                if !silent {
                    emsg(gettext(
                        &raw const e_menu_only_exists_in_another_mode as *const ::core::ffi::c_char,
                    ));
                }
                return FAIL;
            }
            if *name as ::core::ffi::c_int != NUL {
                break;
            }
            (*menu).modes &= !modes;
            if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                free_menu_string(menu, MENU_INDEX_TIP as ::core::ffi::c_int);
            }
            if (*menu).modes & MENU_ALL_MODES as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                free_menu(menup);
            } else {
                menup = &raw mut (*menu).next;
            }
        } else {
            menup = &raw mut (*menu).next;
        }
    }
    if *name as ::core::ffi::c_int != NUL {
        if menu.is_null() {
            if !silent {
                semsg_c!(
                    gettext((e_nomenu.ptr() as *const _) as *const ::core::ffi::c_char),
                    name,
                );
            }
            return FAIL;
        }
        (*menu).modes &= !modes;
        let mut child: *mut vimmenu_T = (*menu).children;
        while !child.is_null() {
            (*menu).modes |= (*child).modes;
            child = (*child).next;
        }
        if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
            free_menu_string(menu, MENU_INDEX_TIP as ::core::ffi::c_int);
        }
        if (*menu).modes & MENU_ALL_MODES as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            *menup = menu;
            free_menu(menup);
        }
    }
    return OK;
}
unsafe extern "C" fn free_menu(mut menup: *mut *mut vimmenu_T) {
    let mut menu: *mut vimmenu_T = *menup;
    *menup = (*menu).next;
    xfree((*menu).name as *mut ::core::ffi::c_void);
    xfree((*menu).dname as *mut ::core::ffi::c_void);
    xfree((*menu).en_name as *mut ::core::ffi::c_void);
    xfree((*menu).en_dname as *mut ::core::ffi::c_void);
    xfree((*menu).actext as *mut ::core::ffi::c_void);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < MENU_MODES as ::core::ffi::c_int {
        free_menu_string(menu, i);
        i += 1;
    }
    xfree(menu as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_menu_string(mut menu: *mut vimmenu_T, mut idx: ::core::ffi::c_int) {
    let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < MENU_MODES as ::core::ffi::c_int {
        if (*menu).strings[i as usize] == (*menu).strings[idx as usize] {
            count += 1;
        }
        i += 1;
    }
    if count == 1 as ::core::ffi::c_int {
        xfree((*menu).strings[idx as usize] as *mut ::core::ffi::c_void);
    }
    (*menu).strings[idx as usize] = ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn menu_get_recursive(
    mut menu: *const vimmenu_T,
    mut modes: ::core::ffi::c_int,
) -> *mut dict_T {
    if menu.is_null() || (*menu).modes & modes == 0 as ::core::ffi::c_int {
        return ::core::ptr::null_mut::<dict_T>();
    }
    let mut dict: *mut dict_T = tv_dict_alloc();
    tv_dict_add_str(
        dict,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*menu).dname,
    );
    tv_dict_add_nr(
        dict,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*menu).priority as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"hidden\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        menu_is_hidden((*menu).dname) as varnumber_T,
    );
    if (*menu).mnemonic != 0 {
        let mut buf: [::core::ffi::c_char; 7] = [0 as ::core::ffi::c_char, 0, 0, 0, 0, 0, 0];
        utf_char2bytes((*menu).mnemonic, &raw mut buf as *mut ::core::ffi::c_char);
        tv_dict_add_str(
            dict,
            b"shortcut\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
    }
    if !(*menu).actext.is_null() {
        tv_dict_add_str(
            dict,
            b"actext\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            (*menu).actext,
        );
    }
    if (*menu).modes & MENU_TIP_MODE as ::core::ffi::c_int != 0
        && !(*menu).strings[MENU_INDEX_TIP as ::core::ffi::c_int as usize].is_null()
    {
        tv_dict_add_str(
            dict,
            b"tooltip\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            (*menu).strings[MENU_INDEX_TIP as ::core::ffi::c_int as usize],
        );
    }
    if (*menu).children.is_null() {
        let mut commands: *mut dict_T = tv_dict_alloc();
        tv_dict_add_dict(
            dict,
            b"mappings\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            commands,
        );
        let mut bit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while bit < MENU_MODES as ::core::ffi::c_int {
            if (*menu).modes & modes & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int {
                let mut impl_0: *mut dict_T = tv_dict_alloc();
                tv_dict_add_allocated_str(
                    impl_0,
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    str2special_save((*menu).strings[bit as usize], false_0 != 0, false_0 != 0),
                );
                tv_dict_add_nr(
                    impl_0,
                    b"silent\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                    (*menu).silent[bit as usize] as varnumber_T,
                );
                tv_dict_add_nr(
                    impl_0,
                    b"enabled\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    (if (*menu).enabled & (1 as ::core::ffi::c_int) << bit != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_nr(
                    impl_0,
                    b"noremap\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    (if (*menu).noremap[bit as usize] & REMAP_NONE as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_nr(
                    impl_0,
                    b"sid\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    (if (*menu).noremap[bit as usize] & REMAP_SCRIPT as ::core::ffi::c_int != 0 {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }) as varnumber_T,
                );
                tv_dict_add_dict(
                    commands,
                    (*menu_mode_chars.ptr())[bit as usize],
                    1 as size_t,
                    impl_0,
                );
            }
            bit += 1;
        }
    } else {
        let children_list: *mut list_T =
            tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        menu = (*menu).children;
        while !menu.is_null() {
            let mut d: *mut dict_T = menu_get_recursive(menu, modes);
            if tv_dict_len(d) > 0 as ::core::ffi::c_long {
                tv_list_append_dict(children_list, d);
            }
            menu = (*menu).next;
        }
        tv_dict_add_list(
            dict,
            b"submenus\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            children_list,
        );
    }
    return dict;
}
pub unsafe extern "C" fn menu_get(
    path_name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut list: *mut list_T,
) -> bool {
    let mut menu: *mut vimmenu_T = *get_root_menu(path_name);
    if *path_name as ::core::ffi::c_int != NUL {
        menu = find_menu(menu, path_name, modes);
        if menu.is_null() {
            return false_0 != 0;
        }
    }
    while !menu.is_null() {
        let mut d: *mut dict_T = menu_get_recursive(menu, modes);
        if !d.is_null() && tv_dict_len(d) > 0 as ::core::ffi::c_long {
            tv_list_append_dict(list, d);
        }
        if *path_name as ::core::ffi::c_int != NUL {
            break;
        }
        menu = (*menu).next;
    }
    return true_0 != 0;
}
unsafe extern "C" fn find_menu(
    mut menu: *mut vimmenu_T,
    mut path_name: *const ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
) -> *mut vimmenu_T {
    debug_assert!(*path_name != 0, "*path_name");
    let saved_name: *mut ::core::ffi::c_char = xstrdup(path_name);
    let mut name: *mut ::core::ffi::c_char = saved_name;
    '_theend: while *name != 0 {
        let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
        while !menu.is_null() {
            if menu_name_equal(name, menu) {
                if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                    break '_theend;
                } else if (*menu).modes & modes == 0 as ::core::ffi::c_int {
                    emsg(gettext(
                        &raw const e_menu_only_exists_in_another_mode as *const ::core::ffi::c_char,
                    ));
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                    break '_theend;
                } else if *p as ::core::ffi::c_int == NUL {
                    break '_theend;
                } else {
                    break;
                }
            } else {
                menu = (*menu).next;
            }
        }
        if menu.is_null() {
            semsg_c!(
                gettext((e_nomenu.ptr() as *const _) as *const ::core::ffi::c_char),
                name,
            );
            break;
        } else {
            name = p;
            debug_assert!(*name != 0, "*name");
            menu = (*menu).children;
        }
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    return menu;
}
unsafe extern "C" fn show_menus(
    path_name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    if *path_name as ::core::ffi::c_int != NUL {
        menu = find_menu(*get_root_menu(path_name), path_name, modes);
        if menu.is_null() {
            return FAIL;
        }
    }
    (*menus_locked.ptr()) += 1;
    msg_puts_title(gettext(
        b"\n--- Menus ---\0".as_ptr() as *const ::core::ffi::c_char
    ));
    show_menus_recursive(menu, modes, 0 as ::core::ffi::c_int);
    (*menus_locked.ptr()) -= 1;
    return OK;
}
unsafe extern "C" fn show_menus_recursive(
    mut menu: *mut vimmenu_T,
    mut modes: ::core::ffi::c_int,
    mut depth: ::core::ffi::c_int,
) {
    if !menu.is_null() && (*menu).modes & modes == 0 as ::core::ffi::c_int {
        return;
    }
    if !menu.is_null() {
        msg_putchar('\n' as ::core::ffi::c_int);
        if got_int.get() {
            return;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < depth {
            msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
            i += 1;
        }
        if (*menu).priority != 0 {
            msg_outnum((*menu).priority);
            msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_outtrans((*menu).name, HLF_D, false_0 != 0);
    }
    if !menu.is_null() && (*menu).children.is_null() {
        let mut bit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while bit < MENU_MODES as ::core::ffi::c_int {
            if (*menu).modes & modes & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int {
                msg_putchar('\n' as ::core::ffi::c_int);
                if got_int.get() {
                    return;
                }
                let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i_0 < depth + 2 as ::core::ffi::c_int {
                    msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                    i_0 += 1;
                }
                msg_puts((*menu_mode_chars.ptr())[bit as usize]);
                if (*menu).noremap[bit as usize] == REMAP_NONE as ::core::ffi::c_int {
                    msg_putchar('*' as ::core::ffi::c_int);
                } else if (*menu).noremap[bit as usize] == REMAP_SCRIPT as ::core::ffi::c_int {
                    msg_putchar('&' as ::core::ffi::c_int);
                } else {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if (*menu).silent[bit as usize] {
                    msg_putchar('s' as ::core::ffi::c_int);
                } else {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if (*menu).modes & (*menu).enabled & (1 as ::core::ffi::c_int) << bit
                    == 0 as ::core::ffi::c_int
                {
                    msg_putchar('-' as ::core::ffi::c_int);
                } else {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                msg_puts(b" \0".as_ptr() as *const ::core::ffi::c_char);
                if *(*menu).strings[bit as usize] as ::core::ffi::c_int == NUL {
                    msg_puts_hl(
                        b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char,
                        HLF_8,
                        false_0 != 0,
                    );
                } else {
                    msg_outtrans_special(
                        (*menu).strings[bit as usize],
                        false_0 != 0,
                        0 as ::core::ffi::c_int,
                    );
                }
            }
            bit += 1;
        }
    } else {
        if menu.is_null() {
            menu = root_menu.get();
            depth -= 1;
        } else {
            menu = (*menu).children;
        }
        while !menu.is_null() && !got_int.get() {
            if !menu_is_hidden((*menu).dname) {
                show_menus_recursive(menu, modes, depth + 1 as ::core::ffi::c_int);
            }
            menu = (*menu).next;
        }
    };
}
static expand_menu: GlobalCell<*mut vimmenu_T> =
    GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
static expand_modes: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static expand_emenu: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
pub unsafe extern "C" fn set_context_in_menu_cmd(
    mut xp: *mut expand_T,
    mut cmd: *const ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: bool,
) -> *mut ::core::ffi::c_char {
    let mut after_dot: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut path_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut unmenu: bool = false;
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
    p = arg;
    while *p != 0 {
        if !ascii_isdigit(*p as ::core::ffi::c_int)
            && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            break;
        }
        p = p.offset(1);
    }
    if !ascii_iswhite(*p as ::core::ffi::c_int) {
        if strncmp(
            arg,
            b"enable\0".as_ptr() as *const ::core::ffi::c_char,
            6 as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || ascii_iswhite(*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            p = arg.offset(6 as ::core::ffi::c_int as isize);
        } else if strncmp(
            arg,
            b"disable\0".as_ptr() as *const ::core::ffi::c_char,
            7 as size_t,
        ) == 0 as ::core::ffi::c_int
            && (*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                || ascii_iswhite(*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
        {
            p = arg.offset(7 as ::core::ffi::c_int as isize);
        } else {
            p = arg;
        }
    }
    while *p as ::core::ffi::c_int != NUL
        && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
    {
        p = p.offset(1);
    }
    after_dot = p;
    arg = after_dot;
    while *p as ::core::ffi::c_int != 0 && !ascii_iswhite(*p as ::core::ffi::c_int) {
        if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V)
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        } else if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
            after_dot = p.offset(1 as ::core::ffi::c_int as isize);
        }
        p = p.offset(1);
    }
    let mut expand_menus: ::core::ffi::c_int = !(*cmd as ::core::ffi::c_int
        == 't' as ::core::ffi::c_int
        && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'e' as ::core::ffi::c_int
        || *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int)
        as ::core::ffi::c_int;
    expand_emenu
        .set((*cmd as ::core::ffi::c_int == 'e' as ::core::ffi::c_int) as ::core::ffi::c_int);
    if expand_menus != 0 && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if *p as ::core::ffi::c_int == NUL {
        expand_modes.set(get_menu_cmd_modes(
            cmd,
            forceit,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut unmenu,
        ));
        if !unmenu {
            expand_modes.set(MENU_ALL_MODES as ::core::ffi::c_int);
        }
        menu = root_menu.get();
        if after_dot > arg {
            let mut path_len: size_t = after_dot.offset_from(arg) as size_t;
            path_name = xmalloc(path_len) as *mut ::core::ffi::c_char;
            xstrlcpy(path_name, arg, path_len);
        }
        let mut name: *mut ::core::ffi::c_char = path_name;
        while !name.is_null() && *name as ::core::ffi::c_int != 0 {
            p = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null()
                        || (*menu).modes & expand_modes.get() == 0 as ::core::ffi::c_int
                    {
                        xfree(path_name as *mut ::core::ffi::c_void);
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                    break;
                } else {
                    menu = (*menu).next;
                }
            }
            if menu.is_null() {
                xfree(path_name as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            name = p;
            menu = (*menu).children;
        }
        xfree(path_name as *mut ::core::ffi::c_void);
        (*xp).xp_context = if expand_menus != 0 {
            EXPAND_MENUNAMES as ::core::ffi::c_int
        } else {
            EXPAND_MENUS as ::core::ffi::c_int
        };
        (*xp).xp_pattern = after_dot;
        expand_menu.set(menu);
    } else {
        (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe extern "C" fn get_menu_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static menu: GlobalCell<*mut vimmenu_T> = GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static should_advance: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if idx == 0 as ::core::ffi::c_int {
        menu.set(expand_menu.get());
        should_advance.set(false_0 != 0);
    }
    while !(*menu.ptr()).is_null()
        && (menu_is_hidden((*menu.get()).dname) as ::core::ffi::c_int != 0
            || menu_is_separator((*menu.get()).dname) as ::core::ffi::c_int != 0
            || (*menu.get()).children.is_null())
    {
        menu.set((*menu.get()).next);
    }
    if (*menu.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*menu.get()).modes & expand_modes.get() != 0 {
        if should_advance.get() {
            str = (*menu.get()).en_dname;
        } else {
            str = (*menu.get()).dname;
            if (*menu.get()).en_dname.is_null() {
                should_advance.set(true_0 != 0);
            }
        }
    } else {
        str = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if should_advance.get() {
        menu.set((*menu.get()).next);
    }
    should_advance.set(!should_advance.get());
    return str;
}
pub unsafe extern "C" fn get_menu_names(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static menu: GlobalCell<*mut vimmenu_T> = GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
    static tbuffer: GlobalCell<[::core::ffi::c_char; 256]> = GlobalCell::new([0; 256]);
    let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    static should_advance: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if idx == 0 as ::core::ffi::c_int {
        menu.set(expand_menu.get());
        should_advance.set(false_0 != 0);
    }
    while !(*menu.ptr()).is_null()
        && (menu_is_hidden((*menu.get()).dname) as ::core::ffi::c_int != 0
            || expand_emenu.get() != 0
                && menu_is_separator((*menu.get()).dname) as ::core::ffi::c_int != 0
            || *(*menu.get())
                .dname
                .offset(strlen((*menu.get()).dname).wrapping_sub(1 as size_t) as isize)
                as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int)
    {
        menu.set((*menu.get()).next);
    }
    if (*menu.ptr()).is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if (*menu.get()).modes & expand_modes.get() != 0 {
        if !(*menu.get()).children.is_null() {
            if should_advance.get() {
                xstrlcpy(
                    tbuffer.ptr() as *mut ::core::ffi::c_char,
                    (*menu.get()).en_dname,
                    TBUFFER_LEN as size_t,
                );
            } else {
                xstrlcpy(
                    tbuffer.ptr() as *mut ::core::ffi::c_char,
                    (*menu.get()).dname,
                    TBUFFER_LEN as size_t,
                );
                if (*menu.get()).en_dname.is_null() {
                    should_advance.set(true_0 != 0);
                }
            }
            strcat(
                tbuffer.ptr() as *mut ::core::ffi::c_char,
                b"\x01\0".as_ptr() as *const ::core::ffi::c_char,
            );
            str = tbuffer.ptr() as *mut ::core::ffi::c_char;
        } else if should_advance.get() {
            str = (*menu.get()).en_dname;
        } else {
            str = (*menu.get()).dname;
            if (*menu.get()).en_dname.is_null() {
                should_advance.set(true_0 != 0);
            }
        }
    } else {
        str = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if should_advance.get() {
        menu.set((*menu.get()).next);
    }
    should_advance.set(!should_advance.get());
    return str;
}
pub const TBUFFER_LEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C" fn menu_name_skip(name: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    p = name;
    while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V
        {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if *p != 0 {
        let c2rust_fresh2 = p;
        p = p.offset(1);
        *c2rust_fresh2 = NUL as ::core::ffi::c_char;
    }
    return p;
}
unsafe extern "C" fn menu_name_equal(
    name: *const ::core::ffi::c_char,
    menu: *const vimmenu_T,
) -> bool {
    if !(*menu).en_name.is_null()
        && (menu_namecmp(name, (*menu).en_name) as ::core::ffi::c_int != 0
            || menu_namecmp(name, (*menu).en_dname) as ::core::ffi::c_int != 0)
    {
        return true_0 != 0;
    }
    return menu_namecmp(name, (*menu).name) as ::core::ffi::c_int != 0
        || menu_namecmp(name, (*menu).dname) as ::core::ffi::c_int != 0;
}
unsafe extern "C" fn menu_namecmp(
    name: *const ::core::ffi::c_char,
    mname: *const ::core::ffi::c_char,
) -> bool {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while *name.offset(i as isize) as ::core::ffi::c_int != NUL
        && *name.offset(i as isize) as ::core::ffi::c_int != TAB
    {
        if *name.offset(i as isize) as ::core::ffi::c_int
            != *mname.offset(i as isize) as ::core::ffi::c_int
        {
            break;
        }
        i += 1;
    }
    return (*name.offset(i as isize) as ::core::ffi::c_int == NUL
        || *name.offset(i as isize) as ::core::ffi::c_int == TAB)
        && (*mname.offset(i as isize) as ::core::ffi::c_int == NUL
            || *mname.offset(i as isize) as ::core::ffi::c_int == TAB);
}
pub unsafe extern "C" fn get_menu_cmd_modes(
    mut cmd: *const ::core::ffi::c_char,
    mut forceit: bool,
    mut noremap: *mut ::core::ffi::c_int,
    mut unmenu: *mut bool,
) -> ::core::ffi::c_int {
    let mut modes: ::core::ffi::c_int = 0;
    's_121: {
        let c2rust_fresh3 = cmd;
        cmd = cmd.offset(1);
        match *c2rust_fresh3 as ::core::ffi::c_int {
            118 => {
                modes =
                    MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            120 => {
                modes = MENU_VISUAL_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            115 => {
                modes = MENU_SELECT_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            111 => {
                modes = MENU_OP_PENDING_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            105 => {
                modes = MENU_INSERT_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            116 => {
                if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                    modes = MENU_TERMINAL_MODE as ::core::ffi::c_int;
                    cmd = cmd.offset(1);
                    break 's_121;
                } else {
                    modes = MENU_TIP_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
            }
            99 => {
                modes = MENU_CMDLINE_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            97 => {
                modes = MENU_INSERT_MODE as ::core::ffi::c_int
                    | MENU_CMDLINE_MODE as ::core::ffi::c_int
                    | MENU_NORMAL_MODE as ::core::ffi::c_int
                    | MENU_VISUAL_MODE as ::core::ffi::c_int
                    | MENU_SELECT_MODE as ::core::ffi::c_int
                    | MENU_OP_PENDING_MODE as ::core::ffi::c_int;
                break 's_121;
            }
            110 => {
                if *cmd as ::core::ffi::c_int != 'o' as ::core::ffi::c_int {
                    modes = MENU_NORMAL_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
            }
            _ => {}
        }
        cmd = cmd.offset(-1);
        if forceit {
            modes =
                MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int;
        } else {
            modes = MENU_NORMAL_MODE as ::core::ffi::c_int
                | MENU_VISUAL_MODE as ::core::ffi::c_int
                | MENU_SELECT_MODE as ::core::ffi::c_int
                | MENU_OP_PENDING_MODE as ::core::ffi::c_int;
        }
    }
    if !noremap.is_null() {
        *noremap = if *cmd as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
            REMAP_NONE as ::core::ffi::c_int
        } else {
            REMAP_YES as ::core::ffi::c_int
        };
    }
    if !unmenu.is_null() {
        *unmenu = *cmd as ::core::ffi::c_int == 'u' as ::core::ffi::c_int;
    }
    return modes;
}
unsafe extern "C" fn get_menu_mode_str(mut modes: ::core::ffi::c_int) -> *mut ::core::ffi::c_char {
    if modes
        & (MENU_INSERT_MODE as ::core::ffi::c_int
            | MENU_CMDLINE_MODE as ::core::ffi::c_int
            | MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int)
        == MENU_INSERT_MODE as ::core::ffi::c_int
            | MENU_CMDLINE_MODE as ::core::ffi::c_int
            | MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int
    {
        return b"a\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes
        & (MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int)
        == MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int
    {
        return b" \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & (MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int)
        == MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int
    {
        return b"!\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & (MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int)
        == MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int
    {
        return b"v\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_VISUAL_MODE as ::core::ffi::c_int != 0 {
        return b"x\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_SELECT_MODE as ::core::ffi::c_int != 0 {
        return b"s\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_OP_PENDING_MODE as ::core::ffi::c_int != 0 {
        return b"o\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_INSERT_MODE as ::core::ffi::c_int != 0 {
        return b"i\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_TERMINAL_MODE as ::core::ffi::c_int != 0 {
        return b"tl\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_CMDLINE_MODE as ::core::ffi::c_int != 0 {
        return b"c\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_NORMAL_MODE as ::core::ffi::c_int != 0 {
        return b"n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
        return b"t\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    }
    return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
}
unsafe extern "C" fn popup_mode_name(
    mut name: *mut ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut len: size_t = strlen(name);
    debug_assert!(len >= 4 as size_t, "len >= 4");
    let mut mode_chars: *mut ::core::ffi::c_char = (*menu_mode_chars.ptr())[idx as usize];
    let mut mode_chars_len: size_t = strlen(mode_chars);
    let mut p: *mut ::core::ffi::c_char = xstrnsave(name, len.wrapping_add(mode_chars_len));
    memmove(
        p.offset(5 as ::core::ffi::c_int as isize)
            .offset(mode_chars_len as isize) as *mut ::core::ffi::c_void,
        p.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
        len.wrapping_sub(4 as size_t),
    );
    let mut i: size_t = 0 as size_t;
    while i < mode_chars_len {
        *p.offset((5 as size_t).wrapping_add(i) as isize) =
            *(*menu_mode_chars.ptr())[idx as usize].offset(i as isize);
        i = i.wrapping_add(1);
    }
    return p;
}
unsafe extern "C" fn menu_text(
    mut str: *const ::core::ffi::c_char,
    mut mnemonic: *mut ::core::ffi::c_int,
    mut actext: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut p: *mut ::core::ffi::c_char = vim_strchr(str, TAB);
    if !p.is_null() {
        if !actext.is_null() {
            *actext = xstrdup(p.offset(1 as ::core::ffi::c_int as isize));
        }
        debug_assert!(p >= str as *mut ::core::ffi::c_char, "p >= str");
        text = xmemdupz(
            str as *const ::core::ffi::c_void,
            p.offset_from(str) as size_t,
        ) as *mut ::core::ffi::c_char;
    } else {
        text = xstrdup(str);
    }
    p = text;
    while !p.is_null() {
        p = vim_strchr(p, '&' as ::core::ffi::c_int);
        if p.is_null() {
            continue;
        }
        if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
            break;
        }
        if !mnemonic.is_null()
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '&' as ::core::ffi::c_int
        {
            *mnemonic =
                *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
        }
        memmove(
            p as *mut ::core::ffi::c_void,
            p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
        );
        p = p.offset(1 as ::core::ffi::c_int as isize);
    }
    return text;
}
pub unsafe extern "C" fn menu_is_menubar(name: *const ::core::ffi::c_char) -> bool {
    return !menu_is_popup(name)
        && !menu_is_toolbar(name)
        && !menu_is_winbar(name)
        && *name as ::core::ffi::c_int != MNU_HIDDEN_CHAR;
}
pub unsafe extern "C" fn menu_is_popup(name: *const ::core::ffi::c_char) -> bool {
    return strncmp(
        name,
        b"PopUp\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn menu_is_toolbar(name: *const ::core::ffi::c_char) -> bool {
    return strncmp(
        name,
        b"ToolBar\0".as_ptr() as *const ::core::ffi::c_char,
        7 as size_t,
    ) == 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn menu_is_separator(mut name: *mut ::core::ffi::c_char) -> bool {
    return *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '-' as ::core::ffi::c_int
        && *name.offset(strlen(name).wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int;
}
unsafe extern "C" fn menu_is_hidden(mut name: *mut ::core::ffi::c_char) -> bool {
    return *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == MNU_HIDDEN_CHAR
        || menu_is_popup(name) as ::core::ffi::c_int != 0
            && *name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
}
unsafe extern "C" fn get_menu_mode() -> ::core::ffi::c_int {
    if State.get() & MODE_TERMINAL != 0 {
        return MENU_INDEX_TERMINAL as ::core::ffi::c_int;
    }
    if VIsual_active.get() {
        if VIsual_select.get() {
            return MENU_INDEX_SELECT as ::core::ffi::c_int;
        }
        return MENU_INDEX_VISUAL as ::core::ffi::c_int;
    }
    if State.get() & MODE_INSERT != 0 {
        return MENU_INDEX_INSERT as ::core::ffi::c_int;
    }
    if State.get() & MODE_CMDLINE != 0
        || State.get() == MODE_ASKMORE
        || State.get() == MODE_HITRETURN
    {
        return MENU_INDEX_CMDLINE as ::core::ffi::c_int;
    }
    if finish_op.get() {
        return MENU_INDEX_OP_PENDING as ::core::ffi::c_int;
    }
    if State.get() & MODE_NORMAL != 0 {
        return MENU_INDEX_NORMAL as ::core::ffi::c_int;
    }
    if State.get() & MODE_LANGMAP != 0 {
        return MENU_INDEX_INSERT as ::core::ffi::c_int;
    }
    return MENU_INDEX_INVALID as ::core::ffi::c_int;
}
pub unsafe extern "C" fn get_menu_mode_flag() -> ::core::ffi::c_int {
    let mut mode: ::core::ffi::c_int = get_menu_mode();
    if mode == MENU_INDEX_INVALID as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    return (1 as ::core::ffi::c_int) << mode;
}
pub unsafe extern "C" fn show_popupmenu() {
    let mut menu_mode: ::core::ffi::c_int = get_menu_mode();
    if menu_mode == MENU_INDEX_INVALID as ::core::ffi::c_int {
        return;
    }
    let mut mode: *mut ::core::ffi::c_char = (*menu_mode_chars.ptr())[menu_mode as usize];
    let mut mode_len: size_t = strlen(mode);
    apply_autocmds(
        EVENT_MENUPOPUP,
        mode,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        curbuf.get(),
    );
    let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
    menu = root_menu.get();
    while !menu.is_null() {
        if strncmp(
            b"PopUp\0".as_ptr() as *const ::core::ffi::c_char,
            (*menu).name,
            5 as size_t,
        ) == 0 as ::core::ffi::c_int
            && strncmp(
                (*menu).name.offset(5 as ::core::ffi::c_int as isize),
                mode,
                mode_len,
            ) == 0 as ::core::ffi::c_int
        {
            break;
        }
        menu = (*menu).next;
    }
    if menu.is_null() || (*menu).children.is_null() {
        return;
    }
    pum_show_popupmenu(menu);
}
pub unsafe extern "C" fn execute_menu(
    mut eap: *const exarg_T,
    mut menu: *mut vimmenu_T,
    mut mode_idx: ::core::ffi::c_int,
) {
    let mut idx: ::core::ffi::c_int = mode_idx;
    if idx < 0 as ::core::ffi::c_int {
        if State.get() & MODE_TERMINAL != 0 {
            idx = MENU_INDEX_TERMINAL as ::core::ffi::c_int;
        } else if State.get() & MODE_CMDLINE != 0 {
            idx = MENU_INDEX_CMDLINE as ::core::ffi::c_int;
        } else if get_real_state() & MODE_VISUAL != 0 {
            idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
        } else if (State.get() & MODE_INSERT != 0 || restart_edit.get() != 0)
            && (*current_sctx.ptr()).sc_sid == 0 as ::core::ffi::c_int
        {
            idx = MENU_INDEX_INSERT as ::core::ffi::c_int;
        } else if !eap.is_null() && (*eap).addr_count != 0 {
            let mut tpos: pos_T = pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            };
            idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
            if (*curbuf.get()).b_visual.vi_start.lnum == (*eap).line1
                && (*curbuf.get()).b_visual.vi_end.lnum == (*eap).line2
            {
                VIsual_mode.set((*curbuf.get()).b_visual.vi_mode);
                tpos = (*curbuf.get()).b_visual.vi_end;
                (*curwin.get()).w_cursor = (*curbuf.get()).b_visual.vi_start;
                (*curwin.get()).w_curswant = (*curbuf.get()).b_visual.vi_curswant;
            } else {
                VIsual_mode.set('V' as ::core::ffi::c_int);
                (*curwin.get()).w_cursor.lnum = (*eap).line1;
                (*curwin.get()).w_cursor.col = 1 as ::core::ffi::c_int as colnr_T;
                tpos.lnum = (*eap).line2;
                tpos.col = MAXCOL as ::core::ffi::c_int as colnr_T;
                tpos.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
            VIsual_active.set(true_0 != 0);
            VIsual_reselect.set(true_0);
            check_cursor(curwin.get());
            VIsual.set((*curwin.get()).w_cursor);
            (*curwin.get()).w_cursor = tpos;
            check_cursor(curwin.get());
            if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                && gchar_cursor() != NUL
            {
                (*curwin.get()).w_cursor.col += 1;
            }
        }
    }
    if idx == MENU_INDEX_INVALID as ::core::ffi::c_int || eap.is_null() {
        idx = MENU_INDEX_NORMAL as ::core::ffi::c_int;
    }
    if !(*menu).strings[idx as usize].is_null()
        && (*menu).modes & (1 as ::core::ffi::c_int) << idx != 0
    {
        if eap.is_null() || (*current_sctx.ptr()).sc_sid != 0 as ::core::ffi::c_int {
            let mut save_state: save_state_T = save_state_T {
                save_msg_scroll: 0,
                save_restart_edit: 0,
                save_msg_didout: false,
                save_State: 0,
                save_finish_op: false,
                save_opcount: 0,
                save_reg_executing: 0,
                save_pending_end_reg_executing: false,
                tabuf: tasave_T {
                    save_typebuf: typebuf_T {
                        tb_buf: ::core::ptr::null_mut::<uint8_t>(),
                        tb_noremap: ::core::ptr::null_mut::<uint8_t>(),
                        tb_buflen: 0,
                        tb_off: 0,
                        tb_len: 0,
                        tb_maplen: 0,
                        tb_silent: 0,
                        tb_no_abbr_cnt: 0,
                        tb_change_cnt: 0,
                    },
                    typebuf_valid: false,
                    old_char: 0,
                    old_mod_mask: 0,
                    save_readbuf1: buffheader_T {
                        bh_first: buffblock_T {
                            b_next: ::core::ptr::null_mut::<buffblock>(),
                            b_strlen: 0,
                            b_str: [0; 1],
                        },
                        bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                        bh_index: 0,
                        bh_space: 0,
                        bh_create_newblock: false,
                    },
                    save_readbuf2: buffheader_T {
                        bh_first: buffblock_T {
                            b_next: ::core::ptr::null_mut::<buffblock>(),
                            b_strlen: 0,
                            b_str: [0; 1],
                        },
                        bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
                        bh_index: 0,
                        bh_space: 0,
                        bh_create_newblock: false,
                    },
                    save_inputbuf: String_0 {
                        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        size: 0,
                    },
                },
            };
            (*ex_normal_busy.ptr()) += 1;
            if save_current_state(&raw mut save_state) {
                exec_normal_cmd(
                    (*menu).strings[idx as usize],
                    (*menu).noremap[idx as usize],
                    (*menu).silent[idx as usize],
                );
            }
            restore_current_state(&raw mut save_state);
            (*ex_normal_busy.ptr()) -= 1;
        } else {
            ins_typebuf(
                (*menu).strings[idx as usize],
                (*menu).noremap[idx as usize],
                0 as ::core::ffi::c_int,
                true_0 != 0,
                (*menu).silent[idx as usize],
            );
        }
    } else if !eap.is_null() {
        let mut mode: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        match idx {
            1 => {
                mode =
                    b"Visual\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            2 => {
                mode =
                    b"Select\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            3 => {
                mode = b"Op-pending\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            6 => {
                mode = b"Terminal\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char;
            }
            4 => {
                mode =
                    b"Insert\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            5 => {
                mode =
                    b"Cmdline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            _ => {
                mode =
                    b"Normal\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
        semsg_c!(
            gettext(b"E335: Menu not defined for %s mode\0".as_ptr() as *const ::core::ffi::c_char),
            mode,
        );
    }
}
unsafe extern "C" fn menu_getbyname(mut name_arg: *mut ::core::ffi::c_char) -> *mut vimmenu_T {
    let mut saved_name: *mut ::core::ffi::c_char = xstrdup(name_arg);
    let mut menu: *mut vimmenu_T = *get_root_menu(saved_name);
    let mut name: *mut ::core::ffi::c_char = saved_name;
    let mut gave_emsg: bool = false_0 != 0;
    while *name != 0 {
        let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
        while !menu.is_null() {
            if menu_name_equal(name, menu) {
                if *p as ::core::ffi::c_int == NUL && !(*menu).children.is_null() {
                    emsg(gettext(
                        b"E333: Menu path must lead to a menu item\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    gave_emsg = true_0 != 0;
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                } else if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                    emsg(gettext(
                        (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                    ));
                    menu = ::core::ptr::null_mut::<vimmenu_T>();
                }
                break;
            } else {
                menu = (*menu).next;
            }
        }
        if menu.is_null() || *p as ::core::ffi::c_int == NUL {
            break;
        }
        menu = (*menu).children;
        name = p;
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    if menu.is_null() {
        if !gave_emsg {
            semsg_c!(
                gettext(b"E334: Menu not found: %s\0".as_ptr() as *const ::core::ffi::c_char),
                name_arg,
            );
        }
        return ::core::ptr::null_mut::<vimmenu_T>();
    }
    return menu;
}
pub unsafe fn ex_emenu(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut mode_idx: ::core::ffi::c_int = MENU_INDEX_INVALID as ::core::ffi::c_int;
    if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
        && ascii_iswhite(*arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            as ::core::ffi::c_int
            != 0
    {
        match *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
            110 => {
                mode_idx = MENU_INDEX_NORMAL as ::core::ffi::c_int;
            }
            118 => {
                mode_idx = MENU_INDEX_VISUAL as ::core::ffi::c_int;
            }
            115 => {
                mode_idx = MENU_INDEX_SELECT as ::core::ffi::c_int;
            }
            111 => {
                mode_idx = MENU_INDEX_OP_PENDING as ::core::ffi::c_int;
            }
            116 => {
                mode_idx = MENU_INDEX_TERMINAL as ::core::ffi::c_int;
            }
            105 => {
                mode_idx = MENU_INDEX_INSERT as ::core::ffi::c_int;
            }
            99 => {
                mode_idx = MENU_INDEX_CMDLINE as ::core::ffi::c_int;
            }
            _ => {
                semsg_c!(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    arg,
                );
                return;
            }
        }
        arg = skipwhite(arg.offset(2 as ::core::ffi::c_int as isize));
    }
    let mut menu: *mut vimmenu_T = menu_getbyname(arg);
    if menu.is_null() {
        return;
    }
    execute_menu(eap, menu, mode_idx);
}
pub unsafe extern "C" fn menu_find(mut path_name: *const ::core::ffi::c_char) -> *mut vimmenu_T {
    let mut menu: *mut vimmenu_T = *get_root_menu(path_name);
    let mut saved_name: *mut ::core::ffi::c_char = xstrdup(path_name);
    let mut name: *mut ::core::ffi::c_char = saved_name;
    '_theend: {
        while *name != 0 {
            let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    if (*menu).children.is_null() {
                        if *p as ::core::ffi::c_int == NUL {
                            emsg(gettext(
                                b"E336: Menu path must lead to a sub-menu\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        } else {
                            emsg(gettext(
                                (e_notsubmenu.ptr() as *const _) as *const ::core::ffi::c_char,
                            ));
                        }
                        menu = ::core::ptr::null_mut::<vimmenu_T>();
                        break '_theend;
                    } else if *p as ::core::ffi::c_int == NUL {
                        break '_theend;
                    } else {
                        break;
                    }
                } else {
                    menu = (*menu).next;
                }
            }
            if menu.is_null() {
                break;
            }
            menu = (*menu).children;
            name = p;
        }
        if menu.is_null() {
            emsg(gettext(
                b"E337: Menu not found - check menu names\0".as_ptr() as *const ::core::ffi::c_char,
            ));
        }
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    return menu;
}
static menutrans_ga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
pub unsafe fn ex_menutranslate(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    if (*menutrans_ga.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
        ga_init(
            menutrans_ga.ptr(),
            ::core::mem::size_of::<menutrans_T>() as ::core::ffi::c_int,
            5 as ::core::ffi::c_int,
        );
    }
    if strncmp(
        arg,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0 as ::core::ffi::c_int
        && ends_excmd(*skipwhite(arg.offset(5 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int)
            != 0
    {
        let mut _gap: *mut garray_T = menutrans_ga.ptr();
        if !(*_gap).ga_data.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*_gap).ga_len {
                let mut _item: *mut menutrans_T =
                    ((*_gap).ga_data as *mut menutrans_T).offset(i as isize);
                let mut _mt: *mut menutrans_T = _item;
                xfree((*_mt).from as *mut ::core::ffi::c_void);
                xfree((*_mt).from_noamp as *mut ::core::ffi::c_void);
                xfree((*_mt).to as *mut ::core::ffi::c_void);
                i += 1;
            }
        }
        ga_clear(_gap);
        del_menutrans_vars();
    } else {
        let mut from: *mut ::core::ffi::c_char = arg;
        arg = menu_skip_part(arg);
        let mut to: *mut ::core::ffi::c_char = skipwhite(arg);
        *arg = NUL as ::core::ffi::c_char;
        arg = menu_skip_part(to);
        if arg == to {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            from = xstrdup(from);
            let mut from_noamp: *mut ::core::ffi::c_char = menu_text(
                from,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            );
            debug_assert!(arg >= to, "arg >= to");
            to = xmemdupz(
                to as *const ::core::ffi::c_void,
                arg.offset_from(to) as size_t,
            ) as *mut ::core::ffi::c_char;
            menu_translate_tab_and_shift(from);
            menu_translate_tab_and_shift(to);
            menu_unescape_name(from);
            menu_unescape_name(to);
            let mut tp: *mut menutrans_T =
                ga_append_via_ptr(menutrans_ga.ptr(), ::core::mem::size_of::<menutrans_T>())
                    as *mut menutrans_T;
            (*tp).from = from;
            (*tp).from_noamp = from_noamp;
            (*tp).to = to;
        }
    };
}
unsafe extern "C" fn menu_skip_part(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    while *p as ::core::ffi::c_int != NUL
        && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        && !ascii_iswhite(*p as ::core::ffi::c_int)
    {
        if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *p as ::core::ffi::c_int == Ctrl_V)
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            p = p.offset(1);
        }
        p = p.offset(1);
    }
    return p;
}
unsafe extern "C" fn menutrans_lookup(
    mut name: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut tp: *mut menutrans_T = (*menutrans_ga.ptr()).ga_data as *mut menutrans_T;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*menutrans_ga.ptr()).ga_len {
        if strncasecmp(name, (*tp.offset(i as isize)).from, len as size_t)
            == 0 as ::core::ffi::c_int
            && *(*tp.offset(i as isize)).from.offset(len as isize) as ::core::ffi::c_int == NUL
        {
            return (*tp.offset(i as isize)).to;
        }
        i += 1;
    }
    let mut c: ::core::ffi::c_char = *name.offset(len as isize);
    *name.offset(len as isize) = NUL as ::core::ffi::c_char;
    let mut dname: *mut ::core::ffi::c_char = menu_text(
        name,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
    *name.offset(len as isize) = c;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < (*menutrans_ga.ptr()).ga_len {
        if strcasecmp(dname, (*tp.offset(i_0 as isize)).from_noamp) == 0 as ::core::ffi::c_int {
            xfree(dname as *mut ::core::ffi::c_void);
            return (*tp.offset(i_0 as isize)).to;
        }
        i_0 += 1;
    }
    xfree(dname as *mut ::core::ffi::c_void);
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
unsafe extern "C" fn menu_unescape_name(mut name: *mut ::core::ffi::c_char) {
    let mut p: *mut ::core::ffi::c_char = name;
    while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int {
        if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
}
unsafe extern "C" fn menu_translate_tab_and_shift(
    mut arg_start: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut arg: *mut ::core::ffi::c_char = arg_start;
    while *arg as ::core::ffi::c_int != 0 && !ascii_iswhite(*arg as ::core::ffi::c_int) {
        if (*arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == Ctrl_V)
            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            arg = arg.offset(1);
        } else if strncasecmp(
            arg,
            b"<TAB>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            5 as ::core::ffi::c_int as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            *arg = TAB as ::core::ffi::c_char;
            memmove(
                arg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                arg.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(arg.offset(5 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
        }
        arg = arg.offset(1);
    }
    if *arg as ::core::ffi::c_int != NUL {
        let c2rust_fresh4 = arg;
        arg = arg.offset(1);
        *c2rust_fresh4 = NUL as ::core::ffi::c_char;
    }
    arg = skipwhite(arg);
    return arg;
}
unsafe extern "C" fn menuitem_getinfo(
    mut menu_name: *const ::core::ffi::c_char,
    mut menu: *const vimmenu_T,
    mut modes: ::core::ffi::c_int,
    mut dict: *mut dict_T,
) {
    if *menu_name as ::core::ffi::c_int == NUL {
        let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        tv_dict_add_list(
            dict,
            b"submenus\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            l,
        );
        let mut topmenu: *const vimmenu_T = menu;
        while !topmenu.is_null() {
            if !menu_is_hidden((*topmenu).dname) {
                tv_list_append_string(l, (*topmenu).dname, -1 as ssize_t);
            }
            topmenu = (*topmenu).next;
        }
        return;
    }
    tv_dict_add_str(
        dict,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        (*menu).name,
    );
    tv_dict_add_str(
        dict,
        b"display\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        (*menu).dname,
    );
    if !(*menu).actext.is_null() {
        tv_dict_add_str(
            dict,
            b"accel\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            (*menu).actext,
        );
    }
    tv_dict_add_nr(
        dict,
        b"priority\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        (*menu).priority as varnumber_T,
    );
    tv_dict_add_str(
        dict,
        b"modes\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        get_menu_mode_str((*menu).modes),
    );
    let mut buf: [::core::ffi::c_char; 65] = [0; 65];
    buf[utf_char2bytes((*menu).mnemonic, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
        NUL as ::core::ffi::c_char;
    tv_dict_add_str(
        dict,
        b"shortcut\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
        &raw mut buf as *mut ::core::ffi::c_char,
    );
    if (*menu).children.is_null() {
        let mut bit: ::core::ffi::c_int = 0;
        bit = 0 as ::core::ffi::c_int;
        while bit < MENU_MODES as ::core::ffi::c_int
            && (1 as ::core::ffi::c_int) << bit & modes == 0
        {
            bit += 1;
        }
        if bit < MENU_MODES as ::core::ffi::c_int {
            if !(*menu).strings[bit as usize].is_null() {
                tv_dict_add_allocated_str(
                    dict,
                    b"rhs\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                    if *(*menu).strings[bit as usize] as ::core::ffi::c_int == NUL {
                        xstrdup(b"<Nop>\0".as_ptr() as *const ::core::ffi::c_char)
                    } else {
                        str2special_save((*menu).strings[bit as usize], false_0 != 0, false_0 != 0)
                    },
                );
            }
            tv_dict_add_bool(
                dict,
                b"noremenu\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                ((*menu).noremap[bit as usize] == REMAP_NONE as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue,
            );
            tv_dict_add_bool(
                dict,
                b"script\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                ((*menu).noremap[bit as usize] == REMAP_SCRIPT as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue,
            );
            tv_dict_add_bool(
                dict,
                b"silent\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*menu).silent[bit as usize] as BoolVarValue,
            );
            tv_dict_add_bool(
                dict,
                b"enabled\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                ((*menu).enabled & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int)
                    as ::core::ffi::c_int as BoolVarValue,
            );
        }
    } else {
        let l_0: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
        tv_dict_add_list(
            dict,
            b"submenus\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            l_0,
        );
        let mut child: *const vimmenu_T = (*menu).children;
        while !child.is_null() {
            tv_list_append_string(l_0, (*child).dname, -1 as ssize_t);
            child = (*child).next;
        }
    };
}
pub unsafe extern "C" fn f_menu_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let retdict: *mut dict_T = (*rettv).vval.v_dict;
    let menu_name: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if menu_name.is_null() {
        return;
    }
    let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        which = tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
    } else {
        which = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if which.is_null() {
        return;
    }
    let modes: ::core::ffi::c_int = get_menu_cmd_modes(
        which,
        *which as ::core::ffi::c_int == '!' as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<bool>(),
    );
    let mut menu: *const vimmenu_T = *get_root_menu(menu_name);
    let saved_name: *mut ::core::ffi::c_char = xstrdup(menu_name);
    if *saved_name as ::core::ffi::c_int != NUL {
        let mut name: *mut ::core::ffi::c_char = saved_name;
        while *name != 0 {
            let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    break;
                }
                menu = (*menu).next;
            }
            if menu.is_null() || *p as ::core::ffi::c_int == NUL {
                break;
            }
            menu = (*menu).children;
            name = p;
        }
    }
    xfree(saved_name as *mut ::core::ffi::c_void);
    if menu.is_null() {
        return;
    }
    if (*menu).modes & modes != 0 {
        menuitem_getinfo(menu_name, menu, modes, retdict);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
