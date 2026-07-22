use crate::src::nvim::charset::kv_transstr;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::memory::{strequal, xrealloc};
use crate::src::nvim::os::libc::{__ctype_b_loc, memcmp, memset, snprintf, strlen};
use crate::src::nvim::strings::kv_do_printf;
use crate::src::nvim::tui::unibi;
pub use crate::src::nvim::types::{
    size_t, ssize_t, Arena, StringBuilder, String_0, TerminfoEntry, TPVAR,
};
extern "C" {
    fn arena_strdup(arena: *mut Arena, str: *const ::core::ffi::c_char)
        -> *mut ::core::ffi::c_char;
}
pub type unibi_boolean = ::core::ffi::c_uint;
pub const unibi_back_color_erase: unibi_boolean = 29;
pub type unibi_numeric = ::core::ffi::c_uint;
pub const unibi_max_colors: unibi_numeric = 59;
pub const unibi_lines: unibi_numeric = 48;
pub const unibi_columns: unibi_numeric = 46;
pub type unibi_string = ::core::ffi::c_uint;
pub const unibi_set_lr_margin: unibi_string = 454;
pub const unibi_set_a_background: unibi_string = 446;
pub const unibi_set_a_foreground: unibi_string = 445;
pub const unibi_enter_italics_mode: unibi_string = 397;
pub const unibi_key_f63: unibi_string = 354;
pub const unibi_key_f62: unibi_string = 353;
pub const unibi_key_f61: unibi_string = 352;
pub const unibi_key_f60: unibi_string = 351;
pub const unibi_key_f59: unibi_string = 350;
pub const unibi_key_f58: unibi_string = 349;
pub const unibi_key_f57: unibi_string = 348;
pub const unibi_key_f56: unibi_string = 347;
pub const unibi_key_f55: unibi_string = 346;
pub const unibi_key_f54: unibi_string = 345;
pub const unibi_key_f53: unibi_string = 344;
pub const unibi_key_f52: unibi_string = 343;
pub const unibi_key_f51: unibi_string = 342;
pub const unibi_key_f50: unibi_string = 341;
pub const unibi_key_f49: unibi_string = 340;
pub const unibi_key_f48: unibi_string = 339;
pub const unibi_key_f47: unibi_string = 338;
pub const unibi_key_f46: unibi_string = 337;
pub const unibi_key_f45: unibi_string = 336;
pub const unibi_key_f44: unibi_string = 335;
pub const unibi_key_f43: unibi_string = 334;
pub const unibi_key_f42: unibi_string = 333;
pub const unibi_key_f41: unibi_string = 332;
pub const unibi_key_f40: unibi_string = 331;
pub const unibi_key_f39: unibi_string = 330;
pub const unibi_key_f38: unibi_string = 329;
pub const unibi_key_f37: unibi_string = 328;
pub const unibi_key_f36: unibi_string = 327;
pub const unibi_key_f35: unibi_string = 326;
pub const unibi_key_f34: unibi_string = 325;
pub const unibi_key_f33: unibi_string = 324;
pub const unibi_key_f32: unibi_string = 323;
pub const unibi_key_f31: unibi_string = 322;
pub const unibi_key_f30: unibi_string = 321;
pub const unibi_key_f29: unibi_string = 320;
pub const unibi_key_f28: unibi_string = 319;
pub const unibi_key_f27: unibi_string = 318;
pub const unibi_key_f26: unibi_string = 317;
pub const unibi_key_f25: unibi_string = 316;
pub const unibi_key_f24: unibi_string = 315;
pub const unibi_key_f23: unibi_string = 314;
pub const unibi_key_f22: unibi_string = 313;
pub const unibi_key_f21: unibi_string = 312;
pub const unibi_key_f20: unibi_string = 311;
pub const unibi_key_f19: unibi_string = 310;
pub const unibi_key_f18: unibi_string = 309;
pub const unibi_key_f17: unibi_string = 308;
pub const unibi_key_f16: unibi_string = 307;
pub const unibi_key_f15: unibi_string = 306;
pub const unibi_key_f14: unibi_string = 305;
pub const unibi_key_f13: unibi_string = 304;
pub const unibi_key_f12: unibi_string = 303;
pub const unibi_key_f11: unibi_string = 302;
pub const unibi_key_sundo: unibi_string = 300;
pub const unibi_key_ssuspend: unibi_string = 299;
pub const unibi_key_sright: unibi_string = 296;
pub const unibi_key_sleft: unibi_string = 287;
pub const unibi_key_sic: unibi_string = 286;
pub const unibi_key_shome: unibi_string = 285;
pub const unibi_key_sfind: unibi_string = 283;
pub const unibi_key_send: unibi_string = 280;
pub const unibi_key_select: unibi_string = 279;
pub const unibi_key_sdc: unibi_string = 277;
pub const unibi_key_sbeg: unibi_string = 272;
pub const unibi_key_undo: unibi_string = 271;
pub const unibi_key_suspend: unibi_string = 270;
pub const unibi_key_find: unibi_string = 253;
pub const unibi_key_end: unibi_string = 250;
pub const unibi_key_beg: unibi_string = 244;
pub const unibi_key_btab: unibi_string = 234;
pub const unibi_to_status_line: unibi_string = 221;
pub const unibi_set_attributes: unibi_string = 217;
pub const unibi_parm_up_cursor: unibi_string = 200;
pub const unibi_parm_right_cursor: unibi_string = 198;
pub const unibi_parm_left_cursor: unibi_string = 197;
pub const unibi_parm_insert_line: unibi_string = 196;
pub const unibi_parm_down_cursor: unibi_string = 193;
pub const unibi_parm_delete_line: unibi_string = 192;
pub const unibi_keypad_xmit: unibi_string = 175;
pub const unibi_keypad_local: unibi_string = 174;
pub const unibi_key_right: unibi_string = 169;
pub const unibi_key_ppage: unibi_string = 168;
pub const unibi_key_npage: unibi_string = 167;
pub const unibi_key_left: unibi_string = 165;
pub const unibi_key_ic: unibi_string = 163;
pub const unibi_key_home: unibi_string = 162;
pub const unibi_key_f9: unibi_string = 161;
pub const unibi_key_f8: unibi_string = 160;
pub const unibi_key_f7: unibi_string = 159;
pub const unibi_key_f6: unibi_string = 158;
pub const unibi_key_f5: unibi_string = 157;
pub const unibi_key_f4: unibi_string = 156;
pub const unibi_key_f3: unibi_string = 155;
pub const unibi_key_f2: unibi_string = 154;
pub const unibi_key_f10: unibi_string = 153;
pub const unibi_key_f1: unibi_string = 152;
pub const unibi_key_dc: unibi_string = 145;
pub const unibi_key_clear: unibi_string = 143;
pub const unibi_key_backspace: unibi_string = 141;
pub const unibi_insert_line: unibi_string = 139;
pub const unibi_from_status_line: unibi_string = 133;
pub const unibi_exit_ca_mode: unibi_string = 126;
pub const unibi_exit_attribute_mode: unibi_string = 125;
pub const unibi_erase_chars: unibi_string = 123;
pub const unibi_enter_underline_mode: unibi_string = 122;
pub const unibi_enter_standout_mode: unibi_string = 121;
pub const unibi_enter_reverse_mode: unibi_string = 120;
pub const unibi_enter_secure_mode: unibi_string = 118;
pub const unibi_enter_dim_mode: unibi_string = 116;
pub const unibi_enter_ca_mode: unibi_string = 114;
pub const unibi_enter_bold_mode: unibi_string = 113;
pub const unibi_enter_blink_mode: unibi_string = 112;
pub const unibi_delete_line: unibi_string = 108;
pub const unibi_cursor_up: unibi_string = 105;
pub const unibi_cursor_right: unibi_string = 103;
pub const unibi_cursor_normal: unibi_string = 102;
pub const unibi_cursor_left: unibi_string = 100;
pub const unibi_cursor_invisible: unibi_string = 99;
pub const unibi_cursor_home: unibi_string = 98;
pub const unibi_cursor_down: unibi_string = 97;
pub const unibi_cursor_address: unibi_string = 96;
pub const unibi_clr_eos: unibi_string = 93;
pub const unibi_clr_eol: unibi_string = 92;
pub const unibi_clear_screen: unibi_string = 91;
pub const unibi_change_scroll_region: unibi_string = 89;
pub const unibi_carriage_return: unibi_string = 88;
pub const unibi_string_begin_: unibi_string = 85;
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISalnum: C2Rust_Unnamed = 8;
pub const _ISpunct: C2Rust_Unnamed = 4;
pub const _IScntrl: C2Rust_Unnamed = 2;
pub const _ISblank: C2Rust_Unnamed = 1;
pub const _ISgraph: C2Rust_Unnamed = 32768;
pub const _ISprint: C2Rust_Unnamed = 16384;
pub const _ISspace: C2Rust_Unnamed = 8192;
pub const _ISxdigit: C2Rust_Unnamed = 4096;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub const _ISalpha: C2Rust_Unnamed = 1024;
pub const _ISlower: C2Rust_Unnamed = 512;
pub const _ISupper: C2Rust_Unnamed = 256;
pub type C2Rust_Unnamed_0 = ::core::ffi::c_uint;
pub const kTermCount: C2Rust_Unnamed_0 = 49;
pub const kTerm_set_underline_style: C2Rust_Unnamed_0 = 48;
pub const kTerm_reset_cursor_color: C2Rust_Unnamed_0 = 47;
pub const kTerm_set_cursor_color: C2Rust_Unnamed_0 = 46;
pub const kTerm_set_rgb_background: C2Rust_Unnamed_0 = 45;
pub const kTerm_set_rgb_foreground: C2Rust_Unnamed_0 = 44;
pub const kTerm_enter_strikethrough_mode: C2Rust_Unnamed_0 = 43;
pub const kTerm_set_cursor_style: C2Rust_Unnamed_0 = 42;
pub const kTerm_reset_cursor_style: C2Rust_Unnamed_0 = 41;
pub const kTerm_to_status_line: C2Rust_Unnamed_0 = 40;
pub const kTerm_set_lr_margin: C2Rust_Unnamed_0 = 39;
pub const kTerm_set_attributes: C2Rust_Unnamed_0 = 38;
pub const kTerm_set_a_foreground: C2Rust_Unnamed_0 = 37;
pub const kTerm_set_a_background: C2Rust_Unnamed_0 = 36;
pub const kTerm_parm_up_cursor: C2Rust_Unnamed_0 = 35;
pub const kTerm_parm_right_cursor: C2Rust_Unnamed_0 = 34;
pub const kTerm_parm_left_cursor: C2Rust_Unnamed_0 = 33;
pub const kTerm_parm_insert_line: C2Rust_Unnamed_0 = 32;
pub const kTerm_parm_down_cursor: C2Rust_Unnamed_0 = 31;
pub const kTerm_parm_delete_line: C2Rust_Unnamed_0 = 30;
pub const kTerm_keypad_xmit: C2Rust_Unnamed_0 = 29;
pub const kTerm_keypad_local: C2Rust_Unnamed_0 = 28;
pub const kTerm_insert_line: C2Rust_Unnamed_0 = 27;
pub const kTerm_from_status_line: C2Rust_Unnamed_0 = 26;
pub const kTerm_exit_ca_mode: C2Rust_Unnamed_0 = 25;
pub const kTerm_exit_attribute_mode: C2Rust_Unnamed_0 = 24;
pub const kTerm_erase_chars: C2Rust_Unnamed_0 = 23;
pub const kTerm_enter_underline_mode: C2Rust_Unnamed_0 = 22;
pub const kTerm_enter_standout_mode: C2Rust_Unnamed_0 = 21;
pub const kTerm_enter_secure_mode: C2Rust_Unnamed_0 = 20;
pub const kTerm_enter_reverse_mode: C2Rust_Unnamed_0 = 19;
pub const kTerm_enter_italics_mode: C2Rust_Unnamed_0 = 18;
pub const kTerm_enter_dim_mode: C2Rust_Unnamed_0 = 17;
pub const kTerm_enter_ca_mode: C2Rust_Unnamed_0 = 16;
pub const kTerm_enter_bold_mode: C2Rust_Unnamed_0 = 15;
pub const kTerm_enter_blink_mode: C2Rust_Unnamed_0 = 14;
pub const kTerm_delete_line: C2Rust_Unnamed_0 = 13;
pub const kTerm_cursor_right: C2Rust_Unnamed_0 = 12;
pub const kTerm_cursor_up: C2Rust_Unnamed_0 = 11;
pub const kTerm_cursor_normal: C2Rust_Unnamed_0 = 10;
pub const kTerm_cursor_home: C2Rust_Unnamed_0 = 9;
pub const kTerm_cursor_left: C2Rust_Unnamed_0 = 8;
pub const kTerm_cursor_invisible: C2Rust_Unnamed_0 = 7;
pub const kTerm_cursor_down: C2Rust_Unnamed_0 = 6;
pub const kTerm_cursor_address: C2Rust_Unnamed_0 = 5;
pub const kTerm_clr_eos: C2Rust_Unnamed_0 = 4;
pub const kTerm_clr_eol: C2Rust_Unnamed_0 = 3;
pub const kTerm_clear_screen: C2Rust_Unnamed_0 = 2;
pub const kTerm_change_scroll_region: C2Rust_Unnamed_0 = 1;
pub const kTerm_carriage_return: C2Rust_Unnamed_0 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TPSTACK {
    pub nums: [::core::ffi::c_long; 20],
    pub strings: [*mut ::core::ffi::c_char; 20],
    pub offset: size_t,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: StringBuilder = StringBuilder {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub unsafe extern "C" fn terminfo_is_term_family(
    mut term: *const ::core::ffi::c_char,
    mut family: *const ::core::ffi::c_char,
) -> bool {
    if term.is_null() {
        return false_0 != 0;
    }
    let mut tlen: size_t = strlen(term);
    let mut flen: size_t = strlen(family);
    return tlen >= flen
        && 0 as ::core::ffi::c_int
            == memcmp(
                term as *const ::core::ffi::c_void,
                family as *const ::core::ffi::c_void,
                flen,
            )
        && (NUL == *term.offset(flen as isize) as ::core::ffi::c_int
            || '-' as ::core::ffi::c_int == *term.offset(flen as isize) as ::core::ffi::c_int
            || '.' as ::core::ffi::c_int == *term.offset(flen as isize) as ::core::ffi::c_int);
}
pub unsafe extern "C" fn terminfo_is_bsd_console(mut _term: *const ::core::ffi::c_char) -> bool {
    return false_0 != 0;
}
pub unsafe extern "C" fn terminfo_from_builtin(
    mut term: *const ::core::ffi::c_char,
    mut termname: *mut *mut ::core::ffi::c_char,
) -> *const TerminfoEntry {
    if strequal(term, b"ghostty\0".as_ptr() as *const ::core::ffi::c_char) as ::core::ffi::c_int
        != 0
        || strequal(
            term,
            b"xterm-ghostty\0".as_ptr() as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
    {
        *termname = b"ghostty\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return ghostty_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"xterm\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"xterm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return xterm_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"screen\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"screen\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return screen_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"tmux\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"tmux\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return tmux_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"rxvt\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"rxvt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return rxvt_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"putty\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"putty\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return putty_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"linux\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"linux\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return linux_16colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"interix\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"interix\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return interix_8colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"iterm\0".as_ptr() as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
        || terminfo_is_term_family(term, b"iterm2\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
        || terminfo_is_term_family(term, b"iTerm.app\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
        || terminfo_is_term_family(term, b"iTerm2.app\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
    {
        *termname = b"iterm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return iterm_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"st\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"st\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return st_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"gnome\0".as_ptr() as *const ::core::ffi::c_char)
        as ::core::ffi::c_int
        != 0
        || terminfo_is_term_family(term, b"vte\0".as_ptr() as *const ::core::ffi::c_char)
            as ::core::ffi::c_int
            != 0
    {
        *termname = b"vte\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return vte_256colour_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"cygwin\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"cygwin\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return cygwin_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"win32con\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname =
            b"win32con\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return win32con_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"conemu\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"conemu\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return conemu_terminfo.ptr() as *const _;
    } else if terminfo_is_term_family(term, b"vtpcon\0".as_ptr() as *const ::core::ffi::c_char) {
        *termname = b"vtpcon\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return vtpcon_terminfo.ptr() as *const _;
    } else {
        *termname = b"ansi\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        return ansi_terminfo.ptr() as *const _;
    };
}
pub unsafe extern "C" fn terminfo_from_database(
    ti: *mut TerminfoEntry,
    termname: *mut ::core::ffi::c_char,
    arena: *mut Arena,
) -> bool {
    let Some(term) = unibi::from_term(::std::ffi::CStr::from_ptr(termname)) else {
        return false;
    };
    let dup = |val: Option<&::std::ffi::CStr>| match val {
        Some(s) => arena_strdup(arena, s.as_ptr()),
        None => ::core::ptr::null_mut(),
    };

    (*ti).bce = term.get_bool(unibi_back_color_erase);
    (*ti).max_colors = term.get_num(unibi_max_colors);
    (*ti).lines = term.get_num(unibi_lines);
    (*ti).columns = term.get_num(unibi_columns);
    (*ti).has_Tc_or_RGB = false;
    (*ti).Su = false;
    for name in term.ext_bool_names() {
        match name.to_bytes() {
            b"Tc" | b"RGB" => (*ti).has_Tc_or_RGB = true,
            b"Su" => (*ti).Su = true,
            _ => {}
        }
    }

    // The TerminfoEntry.defs slots in order: kTerm_carriage_return ..
    // kTerm_to_status_line.
    const UNI_IDS: [unibi_string; 41] = [
        unibi_carriage_return,
        unibi_change_scroll_region,
        unibi_clear_screen,
        unibi_clr_eol,
        unibi_clr_eos,
        unibi_cursor_address,
        unibi_cursor_down,
        unibi_cursor_invisible,
        unibi_cursor_left,
        unibi_cursor_home,
        unibi_cursor_normal,
        unibi_cursor_up,
        unibi_cursor_right,
        unibi_delete_line,
        unibi_enter_blink_mode,
        unibi_enter_bold_mode,
        unibi_enter_ca_mode,
        unibi_enter_dim_mode,
        unibi_enter_italics_mode,
        unibi_enter_reverse_mode,
        unibi_enter_secure_mode,
        unibi_enter_standout_mode,
        unibi_enter_underline_mode,
        unibi_erase_chars,
        unibi_exit_attribute_mode,
        unibi_exit_ca_mode,
        unibi_from_status_line,
        unibi_insert_line,
        unibi_keypad_local,
        unibi_keypad_xmit,
        unibi_parm_delete_line,
        unibi_parm_down_cursor,
        unibi_parm_insert_line,
        unibi_parm_left_cursor,
        unibi_parm_right_cursor,
        unibi_parm_up_cursor,
        unibi_set_a_background,
        unibi_set_a_foreground,
        unibi_set_attributes,
        unibi_set_lr_margin,
        unibi_to_status_line,
    ];
    for (i, &cap) in UNI_IDS.iter().enumerate() {
        (*ti).defs[i] = dup(term.get_str(cap));
    }

    // Extended string capabilities fill the defs slots from
    // kTerm_reset_cursor_style on. A name the description doesn't define
    // leaves its slot untouched.
    const UNI_EXT: [&[u8]; 8] = [
        b"Se", b"Ss", b"smxx", b"setrgbf", b"setrgbb", b"Cs", b"Cr", b"Smulx",
    ];
    for (i, want) in UNI_EXT.iter().enumerate() {
        if let Some((_, val)) = term.ext_strs().find(|(name, _)| name.to_bytes() == *want) {
            (*ti).defs[kTerm_reset_cursor_style as usize + i] = dup(val);
        }
    }

    // Special keys paired with their shifted variant where terminfo defines
    // one (unibi_string_begin_ marks "no shifted form"). The shifted slot is
    // only looked up when the unshifted key exists.
    const UNI_KEYS: [[unibi_string; 2]; 16] = [
        [unibi_key_backspace, unibi_string_begin_],
        [unibi_key_beg, unibi_key_sbeg],
        [unibi_key_btab, unibi_string_begin_],
        [unibi_key_clear, unibi_string_begin_],
        [unibi_key_dc, unibi_key_sdc],
        [unibi_key_end, unibi_key_send],
        [unibi_key_find, unibi_key_sfind],
        [unibi_key_home, unibi_key_shome],
        [unibi_key_ic, unibi_key_sic],
        [unibi_key_npage, unibi_string_begin_],
        [unibi_key_ppage, unibi_string_begin_],
        [unibi_key_select, unibi_string_begin_],
        [unibi_key_suspend, unibi_key_ssuspend],
        [unibi_key_undo, unibi_key_sundo],
        [unibi_key_left, unibi_key_sleft],
        [unibi_key_right, unibi_key_sright],
    ];
    for (i, &[key, skey]) in UNI_KEYS.iter().enumerate() {
        if let Some(val) = term.get_str(key) {
            (*ti).keys[i][0] = arena_strdup(arena, val.as_ptr());
            if skey != unibi_string_begin_ {
                (*ti).keys[i][1] = dup(term.get_str(skey));
            }
        }
    }

    const UNI_FKEYS: [unibi_string; 63] = [
        unibi_key_f1,
        unibi_key_f2,
        unibi_key_f3,
        unibi_key_f4,
        unibi_key_f5,
        unibi_key_f6,
        unibi_key_f7,
        unibi_key_f8,
        unibi_key_f9,
        unibi_key_f10,
        unibi_key_f11,
        unibi_key_f12,
        unibi_key_f13,
        unibi_key_f14,
        unibi_key_f15,
        unibi_key_f16,
        unibi_key_f17,
        unibi_key_f18,
        unibi_key_f19,
        unibi_key_f20,
        unibi_key_f21,
        unibi_key_f22,
        unibi_key_f23,
        unibi_key_f24,
        unibi_key_f25,
        unibi_key_f26,
        unibi_key_f27,
        unibi_key_f28,
        unibi_key_f29,
        unibi_key_f30,
        unibi_key_f31,
        unibi_key_f32,
        unibi_key_f33,
        unibi_key_f34,
        unibi_key_f35,
        unibi_key_f36,
        unibi_key_f37,
        unibi_key_f38,
        unibi_key_f39,
        unibi_key_f40,
        unibi_key_f41,
        unibi_key_f42,
        unibi_key_f43,
        unibi_key_f44,
        unibi_key_f45,
        unibi_key_f46,
        unibi_key_f47,
        unibi_key_f48,
        unibi_key_f49,
        unibi_key_f50,
        unibi_key_f51,
        unibi_key_f52,
        unibi_key_f53,
        unibi_key_f54,
        unibi_key_f55,
        unibi_key_f56,
        unibi_key_f57,
        unibi_key_f58,
        unibi_key_f59,
        unibi_key_f60,
        unibi_key_f61,
        unibi_key_f62,
        unibi_key_f63,
    ];
    for (i, &cap) in UNI_FKEYS.iter().enumerate() {
        (*ti).f_keys[i] = dup(term.get_str(cap));
    }
    true
}
unsafe extern "C" fn fmt(mut val: bool) -> *const ::core::ffi::c_char {
    return if val as ::core::ffi::c_int != 0 {
        b"true\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"false\0".as_ptr() as *const ::core::ffi::c_char
    };
}
pub unsafe extern "C" fn terminfo_info_msg(
    mut ti: *const TerminfoEntry,
    mut termname: *const ::core::ffi::c_char,
    mut from_db: bool,
) -> String_0 {
    let mut data: StringBuilder = KV_INITIAL_VALUE;
    kv_do_printf(
        &raw mut data,
        b"&term: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        termname,
    );
    if from_db {
        kv_do_printf(
            &raw mut data,
            b"using terminfo database\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else {
        kv_do_printf(
            &raw mut data,
            b"using builtin terminfo\n\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    kv_do_printf(
        &raw mut data,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"Boolean capabilities:\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"  back_color_erase: %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        fmt((*ti).bce),
    );
    kv_do_printf(
        &raw mut data,
        b"  truecolor ('Tc' or 'RGB'): %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        fmt((*ti).has_Tc_or_RGB),
    );
    kv_do_printf(
        &raw mut data,
        b"  extended underline ('Su'): %s\n\0".as_ptr() as *const ::core::ffi::c_char,
        fmt((*ti).Su),
    );
    kv_do_printf(
        &raw mut data,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"Numeric capabilities: (-1 for unknown)\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"  lines: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*ti).lines,
    );
    kv_do_printf(
        &raw mut data,
        b"  columns: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*ti).columns,
    );
    kv_do_printf(
        &raw mut data,
        b"  max_colors: %d\n\0".as_ptr() as *const ::core::ffi::c_char,
        (*ti).columns,
    );
    kv_do_printf(
        &raw mut data,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    kv_do_printf(
        &raw mut data,
        b"String capabilities:\n\0".as_ptr() as *const ::core::ffi::c_char,
    );
    static string_names: GlobalCell<[*const ::core::ffi::c_char; 49]> = GlobalCell::new([
        b"carriage_return\0".as_ptr() as *const ::core::ffi::c_char,
        b"change_scroll_region\0".as_ptr() as *const ::core::ffi::c_char,
        b"clear_screen\0".as_ptr() as *const ::core::ffi::c_char,
        b"clr_eol\0".as_ptr() as *const ::core::ffi::c_char,
        b"clr_eos\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_address\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_down\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_invisible\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_left\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_home\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_normal\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_up\0".as_ptr() as *const ::core::ffi::c_char,
        b"cursor_right\0".as_ptr() as *const ::core::ffi::c_char,
        b"delete_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_blink_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_bold_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_ca_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_dim_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_italics_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_reverse_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_secure_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_standout_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_underline_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"erase_chars\0".as_ptr() as *const ::core::ffi::c_char,
        b"exit_attribute_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"exit_ca_mode\0".as_ptr() as *const ::core::ffi::c_char,
        b"from_status_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"insert_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"keypad_local\0".as_ptr() as *const ::core::ffi::c_char,
        b"keypad_xmit\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_delete_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_down_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_insert_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_left_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_right_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"parm_up_cursor\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_a_background\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_a_foreground\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_attributes\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_lr_margin\0".as_ptr() as *const ::core::ffi::c_char,
        b"to_status_line\0".as_ptr() as *const ::core::ffi::c_char,
        b"reset_cursor_style (Se)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_cursor_style (Ss)\0".as_ptr() as *const ::core::ffi::c_char,
        b"enter_strikethrough_mode (smxx)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_rgb_foreground (setrgbf)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_rgb_background (setrgbb)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_cursor_color (Cs)\0".as_ptr() as *const ::core::ffi::c_char,
        b"reset_cursor_color (Cr)\0".as_ptr() as *const ::core::ffi::c_char,
        b"set_underline_style (Smulx)\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[*const ::core::ffi::c_char; 49]>()
        .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
        .wrapping_div(
            (::core::mem::size_of::<[*const ::core::ffi::c_char; 49]>()
                .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut s: *const ::core::ffi::c_char = (*ti).defs[i as usize];
        if !s.is_null() {
            kv_do_printf(
                &raw mut data,
                b"  %-31s = \0".as_ptr() as *const ::core::ffi::c_char,
                (*string_names.ptr())[i as usize],
            );
            kv_transstr(&raw mut data, s, false_0 != 0);
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh0 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh0 as isize) = '\n' as ::core::ffi::c_char;
        }
        i = i.wrapping_add(1);
    }
    static key_names: GlobalCell<[*const ::core::ffi::c_char; 16]> = GlobalCell::new([
        b"backspace\0".as_ptr() as *const ::core::ffi::c_char,
        b"beg\0".as_ptr() as *const ::core::ffi::c_char,
        b"btab\0".as_ptr() as *const ::core::ffi::c_char,
        b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        b"dc\0".as_ptr() as *const ::core::ffi::c_char,
        b"end\0".as_ptr() as *const ::core::ffi::c_char,
        b"find\0".as_ptr() as *const ::core::ffi::c_char,
        b"home\0".as_ptr() as *const ::core::ffi::c_char,
        b"ic\0".as_ptr() as *const ::core::ffi::c_char,
        b"npage\0".as_ptr() as *const ::core::ffi::c_char,
        b"ppage\0".as_ptr() as *const ::core::ffi::c_char,
        b"select\0".as_ptr() as *const ::core::ffi::c_char,
        b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
        b"undo\0".as_ptr() as *const ::core::ffi::c_char,
        b"left\0".as_ptr() as *const ::core::ffi::c_char,
        b"right\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut i_0: size_t = (0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
    while i_0
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 16]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 16]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut s_0: *const ::core::ffi::c_char =
            (*ti).keys[i_0 as usize][0 as ::core::ffi::c_int as usize];
        if !s_0.is_null() {
            kv_do_printf(
                &raw mut data,
                b"  key_%-27s = \0".as_ptr() as *const ::core::ffi::c_char,
                (*key_names.ptr())[i_0 as usize],
            );
            kv_transstr(&raw mut data, s_0, false_0 != 0);
            let mut ss: *const ::core::ffi::c_char =
                (*ti).keys[i_0 as usize][1 as ::core::ffi::c_int as usize];
            if !ss.is_null() {
                kv_do_printf(
                    &raw mut data,
                    b", key_s%s = \0".as_ptr() as *const ::core::ffi::c_char,
                    (*key_names.ptr())[i_0 as usize],
                );
                kv_transstr(&raw mut data, ss, false_0 != 0);
            }
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh1 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh1 as isize) = '\n' as ::core::ffi::c_char;
        }
        i_0 = i_0.wrapping_add(1);
    }
    static fkey_names: GlobalCell<[*const ::core::ffi::c_char; 63]> = GlobalCell::new([
        b"f1\0".as_ptr() as *const ::core::ffi::c_char,
        b"f2\0".as_ptr() as *const ::core::ffi::c_char,
        b"f3\0".as_ptr() as *const ::core::ffi::c_char,
        b"f4\0".as_ptr() as *const ::core::ffi::c_char,
        b"f5\0".as_ptr() as *const ::core::ffi::c_char,
        b"f6\0".as_ptr() as *const ::core::ffi::c_char,
        b"f7\0".as_ptr() as *const ::core::ffi::c_char,
        b"f8\0".as_ptr() as *const ::core::ffi::c_char,
        b"f9\0".as_ptr() as *const ::core::ffi::c_char,
        b"f10\0".as_ptr() as *const ::core::ffi::c_char,
        b"f11\0".as_ptr() as *const ::core::ffi::c_char,
        b"f12\0".as_ptr() as *const ::core::ffi::c_char,
        b"f13\0".as_ptr() as *const ::core::ffi::c_char,
        b"f14\0".as_ptr() as *const ::core::ffi::c_char,
        b"f15\0".as_ptr() as *const ::core::ffi::c_char,
        b"f16\0".as_ptr() as *const ::core::ffi::c_char,
        b"f17\0".as_ptr() as *const ::core::ffi::c_char,
        b"f18\0".as_ptr() as *const ::core::ffi::c_char,
        b"f19\0".as_ptr() as *const ::core::ffi::c_char,
        b"f20\0".as_ptr() as *const ::core::ffi::c_char,
        b"f21\0".as_ptr() as *const ::core::ffi::c_char,
        b"f22\0".as_ptr() as *const ::core::ffi::c_char,
        b"f23\0".as_ptr() as *const ::core::ffi::c_char,
        b"f24\0".as_ptr() as *const ::core::ffi::c_char,
        b"f25\0".as_ptr() as *const ::core::ffi::c_char,
        b"f26\0".as_ptr() as *const ::core::ffi::c_char,
        b"f27\0".as_ptr() as *const ::core::ffi::c_char,
        b"f28\0".as_ptr() as *const ::core::ffi::c_char,
        b"f29\0".as_ptr() as *const ::core::ffi::c_char,
        b"f30\0".as_ptr() as *const ::core::ffi::c_char,
        b"f31\0".as_ptr() as *const ::core::ffi::c_char,
        b"f32\0".as_ptr() as *const ::core::ffi::c_char,
        b"f33\0".as_ptr() as *const ::core::ffi::c_char,
        b"f34\0".as_ptr() as *const ::core::ffi::c_char,
        b"f35\0".as_ptr() as *const ::core::ffi::c_char,
        b"f36\0".as_ptr() as *const ::core::ffi::c_char,
        b"f37\0".as_ptr() as *const ::core::ffi::c_char,
        b"f38\0".as_ptr() as *const ::core::ffi::c_char,
        b"f39\0".as_ptr() as *const ::core::ffi::c_char,
        b"f40\0".as_ptr() as *const ::core::ffi::c_char,
        b"f41\0".as_ptr() as *const ::core::ffi::c_char,
        b"f42\0".as_ptr() as *const ::core::ffi::c_char,
        b"f43\0".as_ptr() as *const ::core::ffi::c_char,
        b"f44\0".as_ptr() as *const ::core::ffi::c_char,
        b"f45\0".as_ptr() as *const ::core::ffi::c_char,
        b"f46\0".as_ptr() as *const ::core::ffi::c_char,
        b"f47\0".as_ptr() as *const ::core::ffi::c_char,
        b"f48\0".as_ptr() as *const ::core::ffi::c_char,
        b"f49\0".as_ptr() as *const ::core::ffi::c_char,
        b"f50\0".as_ptr() as *const ::core::ffi::c_char,
        b"f51\0".as_ptr() as *const ::core::ffi::c_char,
        b"f52\0".as_ptr() as *const ::core::ffi::c_char,
        b"f53\0".as_ptr() as *const ::core::ffi::c_char,
        b"f54\0".as_ptr() as *const ::core::ffi::c_char,
        b"f55\0".as_ptr() as *const ::core::ffi::c_char,
        b"f56\0".as_ptr() as *const ::core::ffi::c_char,
        b"f57\0".as_ptr() as *const ::core::ffi::c_char,
        b"f58\0".as_ptr() as *const ::core::ffi::c_char,
        b"f59\0".as_ptr() as *const ::core::ffi::c_char,
        b"f60\0".as_ptr() as *const ::core::ffi::c_char,
        b"f61\0".as_ptr() as *const ::core::ffi::c_char,
        b"f62\0".as_ptr() as *const ::core::ffi::c_char,
        b"f63\0".as_ptr() as *const ::core::ffi::c_char,
    ]);
    let mut i_1: size_t = (0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as size_t;
    while i_1
        < ::core::mem::size_of::<[*const ::core::ffi::c_char; 63]>()
            .wrapping_div(::core::mem::size_of::<*const ::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[*const ::core::ffi::c_char; 63]>()
                    .wrapping_rem(::core::mem::size_of::<*const ::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        let mut s_1: *const ::core::ffi::c_char = (*ti).f_keys[i_1 as usize];
        if !s_1.is_null() {
            kv_do_printf(
                &raw mut data,
                b"  key_%-27s = \0".as_ptr() as *const ::core::ffi::c_char,
                (*fkey_names.ptr())[i_1 as usize],
            );
            kv_transstr(&raw mut data, s_1, false_0 != 0);
            if data.size == data.capacity {
                data.capacity = if data.capacity != 0 {
                    data.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                data.items = xrealloc(
                    data.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh2 = data.size;
            data.size = data.size.wrapping_add(1);
            *data.items.offset(c2rust_fresh2 as isize) = '\n' as ::core::ffi::c_char;
        }
        i_1 = i_1.wrapping_add(1);
    }
    if data.size == data.capacity {
        data.capacity = if data.capacity != 0 {
            data.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        data.items = xrealloc(
            data.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(data.capacity),
        ) as *mut ::core::ffi::c_char;
    } else {
    };
    let c2rust_fresh3 = data.size;
    data.size = data.size.wrapping_add(1);
    *data.items.offset(c2rust_fresh3 as isize) = '\0' as ::core::ffi::c_char;
    return String_0 {
        data: data.items,
        size: data.size.wrapping_sub(1 as size_t),
    };
}
unsafe extern "C" fn push(
    mut num: ::core::ffi::c_long,
    mut string: *mut ::core::ffi::c_char,
    mut stack: *mut TPSTACK,
) -> ::core::ffi::c_int {
    if (*stack).offset
        >= ::core::mem::size_of::<[::core::ffi::c_long; 20]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_long>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_long; 20]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_long>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        return -1 as ::core::ffi::c_int;
    }
    (*stack).nums[(*stack).offset as usize] = num;
    (*stack).strings[(*stack).offset as usize] = string;
    (*stack).offset = (*stack).offset.wrapping_add(1);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn pop(
    mut num: *mut ::core::ffi::c_long,
    mut string: *mut *mut ::core::ffi::c_char,
    mut stack: *mut TPSTACK,
) -> ::core::ffi::c_int {
    if (*stack).offset == 0 as size_t {
        if !num.is_null() {
            *num = 0 as ::core::ffi::c_long;
        }
        if !string.is_null() {
            *string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        return -1 as ::core::ffi::c_int;
    }
    (*stack).offset = (*stack).offset.wrapping_sub(1);
    if !num.is_null() {
        *num = (*stack).nums[(*stack).offset as usize];
    }
    if !string.is_null() {
        *string = (*stack).strings[(*stack).offset as usize];
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn ochar(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buf_end: *const ::core::ffi::c_char,
    mut c: ::core::ffi::c_int,
) -> bool {
    if c == 0 as ::core::ffi::c_int {
        c = 0o200 as ::core::ffi::c_int;
    }
    if buf_end.offset_from(*buf) < 2 as isize {
        return false;
    }
    let c2rust_fresh18 = *buf;
    *buf = (*buf).offset(1);
    *c2rust_fresh18 = c as ::core::ffi::c_char;
    return true;
}
unsafe extern "C" fn onum(
    mut buf: *mut *mut ::core::ffi::c_char,
    mut buf_end: *const ::core::ffi::c_char,
    mut fmt_0: *const ::core::ffi::c_char,
    mut num: ::core::ffi::c_int,
    mut len: size_t,
) -> bool {
    let LONG_STR_MAX: size_t = 21 as size_t;
    len = if len > LONG_STR_MAX {
        len
    } else {
        LONG_STR_MAX
    };
    if buf_end.offset_from(*buf) < len.wrapping_add(2 as size_t) as isize {
        return false;
    }
    let mut l: ::core::ffi::c_int = snprintf(*buf, len.wrapping_add(2 as size_t), fmt_0, num);
    if l == -1 as ::core::ffi::c_int {
        return false;
    }
    *buf = (*buf).offset(l as isize);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn terminfo_fmt(
    mut buf_start: *mut ::core::ffi::c_char,
    mut buf_end: *mut ::core::ffi::c_char,
    mut str: *const ::core::ffi::c_char,
    mut params: *mut TPVAR,
) -> size_t {
    let mut c: ::core::ffi::c_char = 0;
    let mut fmt_0: [::core::ffi::c_char; 64] = [0; 64];
    let mut fp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ostr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut val: ::core::ffi::c_long = 0;
    let mut val2: ::core::ffi::c_long = 0;
    let mut dnums: [::core::ffi::c_long; 26] = [0; 26];
    let mut snums: [::core::ffi::c_long; 26] = [0; 26];
    memset(
        &raw mut dnums as *mut ::core::ffi::c_long as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_long; 26]>(),
    );
    memset(
        &raw mut snums as *mut ::core::ffi::c_long as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[::core::ffi::c_long; 26]>(),
    );
    let mut buf: *mut ::core::ffi::c_char = buf_start;
    let mut l: size_t = 0;
    let mut width: size_t = 0;
    let mut precision: size_t = 0;
    let mut olen: size_t = 0;
    let mut stack: TPSTACK = TPSTACK {
        nums: [0; 20],
        strings: [::core::ptr::null_mut::<::core::ffi::c_char>(); 20],
        offset: 0,
    };
    let mut done: ::core::ffi::c_uint = 0;
    let mut dot: ::core::ffi::c_uint = 0;
    let mut minus: ::core::ffi::c_uint = 0;
    memset(
        &raw mut stack as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<TPSTACK>(),
    );
    loop {
        let c2rust_fresh4 = str;
        str = str.offset(1);
        c = *c2rust_fresh4;
        if c as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
            break;
        }
        if c as ::core::ffi::c_int != '%' as ::core::ffi::c_int || {
            let c2rust_fresh5 = str;
            str = str.offset(1);
            c = *c2rust_fresh5;
            c as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        } {
            if c as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
                break;
            }
            if !ochar(&raw mut buf, buf_end, c as ::core::ffi::c_int) {
                return false_0 as size_t;
            }
        } else {
            fp = &raw mut fmt_0 as *mut ::core::ffi::c_char;
            let c2rust_fresh6 = fp;
            fp = fp.offset(1);
            *c2rust_fresh6 = '%' as ::core::ffi::c_char;
            minus = 0 as ::core::ffi::c_uint;
            dot = minus;
            done = dot;
            precision = 0 as size_t;
            width = precision;
            val = 0 as ::core::ffi::c_long;
            while done == 0 as ::core::ffi::c_uint
                && (fp.offset_from(&raw mut fmt_0 as *mut ::core::ffi::c_char) as size_t)
                    < ::core::mem::size_of::<[::core::ffi::c_char; 64]>()
            {
                match c as ::core::ffi::c_int {
                    99 | 115 => {
                        let c2rust_fresh7 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh7 = c;
                        done = 1 as ::core::ffi::c_uint;
                    }
                    100 | 111 | 120 | 88 => {
                        let c2rust_fresh8 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh8 = 'l' as ::core::ffi::c_char;
                        let c2rust_fresh9 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh9 = c;
                        done = 1 as ::core::ffi::c_uint;
                    }
                    35 | 32 => {
                        let c2rust_fresh10 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh10 = c;
                    }
                    46 => {
                        let c2rust_fresh11 = fp;
                        fp = fp.offset(1);
                        *c2rust_fresh11 = c;
                        if dot == 0 as ::core::ffi::c_uint {
                            dot = 1 as ::core::ffi::c_uint;
                            width = val as size_t;
                        } else {
                            done = 2 as ::core::ffi::c_uint;
                        }
                        val = 0 as ::core::ffi::c_long;
                    }
                    58 => {
                        minus = 1 as ::core::ffi::c_uint;
                    }
                    45 => {
                        if minus != 0 {
                            let c2rust_fresh12 = fp;
                            fp = fp.offset(1);
                            *c2rust_fresh12 = c;
                        } else {
                            done = 1 as ::core::ffi::c_uint;
                        }
                    }
                    _ => {
                        if *(*__ctype_b_loc())
                            .offset(c as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                                as ::core::ffi::c_int
                            != 0
                        {
                            val = val * 10 as ::core::ffi::c_long
                                + (c as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
                                    as ::core::ffi::c_long;
                            if val > 10000 as ::core::ffi::c_long {
                                done = 2 as ::core::ffi::c_uint;
                            } else {
                                let c2rust_fresh13 = fp;
                                fp = fp.offset(1);
                                *c2rust_fresh13 = c;
                            }
                        } else {
                            done = 1 as ::core::ffi::c_uint;
                        }
                    }
                }
                if done == 0 as ::core::ffi::c_uint {
                    let c2rust_fresh14 = str;
                    str = str.offset(1);
                    c = *c2rust_fresh14;
                }
            }
            if done == 2 as ::core::ffi::c_uint {
                fp = (&raw mut fmt_0 as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize);
                *fp = *str;
                olen = 0 as size_t;
            } else {
                if dot == 0 as ::core::ffi::c_uint {
                    width = val as size_t;
                } else {
                    precision = val as size_t;
                }
                olen = if width > precision { width } else { precision };
            }
            let c2rust_fresh15 = fp;
            fp = fp.offset(1);
            *c2rust_fresh15 = '\0' as ::core::ffi::c_char;
            match c as ::core::ffi::c_int {
                99 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if !ochar(
                        &raw mut buf,
                        buf_end,
                        val as ::core::ffi::c_uchar as ::core::ffi::c_int,
                    ) {
                        return false_0 as size_t;
                    }
                }
                115 => {
                    pop(
                        ::core::ptr::null_mut::<::core::ffi::c_long>(),
                        &raw mut ostr,
                        &raw mut stack,
                    );
                    if !ostr.is_null() {
                        let mut r: ::core::ffi::c_int = 0;
                        l = strlen(ostr);
                        if l < olen {
                            l = olen;
                        }
                        if (buf_end.offset_from(buf) as size_t) < l.wrapping_add(1 as size_t) {
                            return false_0 as size_t;
                        }
                        r = snprintf(
                            buf,
                            l.wrapping_add(1 as size_t),
                            &raw mut fmt_0 as *mut ::core::ffi::c_char,
                            ostr,
                        );
                        if r != -1 as ::core::ffi::c_int {
                            buf = buf.offset(r as size_t as isize);
                        }
                    }
                }
                108 => {
                    pop(
                        ::core::ptr::null_mut::<::core::ffi::c_long>(),
                        &raw mut ostr,
                        &raw mut stack,
                    );
                    if ostr.is_null() {
                        l = 0 as size_t;
                    } else {
                        l = strlen(ostr);
                    }
                    push(
                        l as ::core::ffi::c_long,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                }
                100 | 111 | 120 | 88 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if onum(
                        &raw mut buf,
                        buf_end,
                        &raw mut fmt_0 as *mut ::core::ffi::c_char,
                        val as ::core::ffi::c_int,
                        olen,
                    ) as ::core::ffi::c_int
                        == 0 as ::core::ffi::c_int
                    {
                        return 0 as size_t;
                    }
                }
                112 => {
                    if !((*str as ::core::ffi::c_int) < '1' as ::core::ffi::c_int
                        || *str as ::core::ffi::c_int > '9' as ::core::ffi::c_int)
                    {
                        let c2rust_fresh16 = str;
                        str = str.offset(1);
                        l = (*c2rust_fresh16 as ::core::ffi::c_int - '1' as ::core::ffi::c_int)
                            as size_t;
                        if push(
                            (*params.offset(l as isize)).num,
                            (*params.offset(l as isize)).string,
                            &raw mut stack,
                        ) != 0
                        {
                            return 0 as size_t;
                        }
                    }
                }
                80 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if *str as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
                    {
                        dnums[(*str as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize] =
                            val;
                    } else if *str as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
                    {
                        snums[(*str as ::core::ffi::c_int - 'A' as ::core::ffi::c_int) as usize] =
                            val;
                    }
                }
                103 => {
                    if *str as ::core::ffi::c_int >= 'a' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'z' as ::core::ffi::c_int
                    {
                        if push(
                            dnums
                                [(*str as ::core::ffi::c_int - 'a' as ::core::ffi::c_int) as usize],
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut stack,
                        ) != 0
                        {
                            return 0 as size_t;
                        }
                    } else if *str as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
                    {
                        if push(
                            snums
                                [(*str as ::core::ffi::c_int - 'A' as ::core::ffi::c_int) as usize],
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut stack,
                        ) != 0
                        {
                            return 0 as size_t;
                        }
                    }
                }
                105 => {
                    (*params.offset(0 as ::core::ffi::c_int as isize)).num += 1;
                    (*params.offset(1 as ::core::ffi::c_int as isize)).num += 1;
                }
                39 => {
                    let c2rust_fresh17 = str;
                    str = str.offset(1);
                    if push(
                        *c2rust_fresh17 as ::core::ffi::c_uchar as ::core::ffi::c_long,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                    while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int != '\'' as ::core::ffi::c_int
                    {
                        str = str.offset(1);
                    }
                    if *str as ::core::ffi::c_int == '\'' as ::core::ffi::c_int {
                        str = str.offset(1);
                    }
                }
                123 => {
                    val = 0 as ::core::ffi::c_long;
                    while *(*__ctype_b_loc())
                        .offset(*str as ::core::ffi::c_uchar as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort
                            as ::core::ffi::c_int
                        != 0
                    {
                        val = val * 10 as ::core::ffi::c_long
                            + (*str as ::core::ffi::c_int - '0' as ::core::ffi::c_int)
                                as ::core::ffi::c_long;
                        str = str.offset(1);
                    }
                    if push(
                        val,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                    while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                        && *str as ::core::ffi::c_int != '}' as ::core::ffi::c_int
                    {
                        str = str.offset(1);
                    }
                    if *str as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                        str = str.offset(1);
                    }
                }
                43 | 45 | 42 | 47 | 109 | 65 | 79 | 38 | 124 | 94 | 61 | 60 | 62 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    pop(
                        &raw mut val2,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    match c as ::core::ffi::c_int {
                        43 => {
                            val = val + val2;
                        }
                        45 => {
                            val = val2 - val;
                        }
                        42 => {
                            val = val * val2;
                        }
                        47 => {
                            val = if val != 0 {
                                val2 / val
                            } else {
                                0 as ::core::ffi::c_long
                            };
                        }
                        109 => {
                            val = if val != 0 {
                                val2 % val
                            } else {
                                0 as ::core::ffi::c_long
                            };
                        }
                        65 => {
                            val = (val != 0 && val2 != 0) as ::core::ffi::c_int
                                as ::core::ffi::c_long;
                        }
                        79 => {
                            val = (val != 0 || val2 != 0) as ::core::ffi::c_int
                                as ::core::ffi::c_long;
                        }
                        38 => {
                            val = val & val2;
                        }
                        124 => {
                            val = val | val2;
                        }
                        94 => {
                            val = val ^ val2;
                        }
                        61 => {
                            val = (val == val2) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        60 => {
                            val = (val2 < val) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        62 => {
                            val = (val2 > val) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        _ => {}
                    }
                    if push(
                        val,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                }
                33 | 126 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    match c as ::core::ffi::c_int {
                        33 => {
                            val = (val == 0) as ::core::ffi::c_int as ::core::ffi::c_long;
                        }
                        126 => {
                            val = !val;
                        }
                        _ => {}
                    }
                    if push(
                        val,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut stack,
                    ) != 0
                    {
                        return 0 as size_t;
                    }
                }
                63 => {}
                116 => {
                    pop(
                        &raw mut val,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                        &raw mut stack,
                    );
                    if val == 0 as ::core::ffi::c_long {
                        l = 0 as size_t;
                        while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                            if *str as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                                str = str.offset(1);
                                if *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                                    l = l.wrapping_add(1);
                                } else if *str as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                                    if l > 0 as size_t {
                                        l = l.wrapping_sub(1);
                                    } else {
                                        str = str.offset(1);
                                        break;
                                    }
                                } else if *str as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                                    && l == 0 as size_t
                                {
                                    str = str.offset(1);
                                    break;
                                }
                            }
                            str = str.offset(1);
                        }
                    }
                }
                101 => {
                    l = 0 as size_t;
                    while *str as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                        if *str as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                            str = str.offset(1);
                            if *str as ::core::ffi::c_int == '?' as ::core::ffi::c_int {
                                l = l.wrapping_add(1);
                            } else if *str as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
                                if l > 0 as size_t {
                                    l = l.wrapping_sub(1);
                                } else {
                                    str = str.offset(1);
                                    break;
                                }
                            }
                        }
                        str = str.offset(1);
                    }
                }
                59 | _ => {}
            }
        }
    }
    return buf.offset_from(buf_start) as size_t;
}
static ansi_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;%?%p6%t;1%;%?%p7%t;8%;%?%p9%t;11%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static ghostty_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?12l\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B(B\x1B[m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"%?%p9%t\x1B(0%e\x1B(B%;\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p5%t;2%;%?%p7%t;8%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?69h\x1B[%i%p1%d;%p2%ds\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]2;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static interix_8colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: 25 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[s\x1B[1b\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2b\x1B[u\r\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[U\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[T\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[S\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1BF^\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1BF$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BF1\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF2\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF3\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF4\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF5\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF6\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF7\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF8\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BF9\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFE\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFF\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFG\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFI\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFJ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFK\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFN\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFO\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFT\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFU\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFV\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFW\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFY\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFZ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFa\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFb\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFc\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFd\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFe\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFf\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFg\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFh\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFi\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFj\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFk\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFn\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFo\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFp\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFq\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFs\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFt\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFu\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFv\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFw\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFx\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFy\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BFz\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static iterm_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\x1B[22;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\x1B[23;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p5%t;2%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]2;\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static linux_16colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\x1B[?1c\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\x1B[?0c\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;%?%p5%t;2%;%?%p6%t;1%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B^I\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static putty_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%p6%|%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;m%?%p9%t^N%e\x0F%;\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1B[11~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static rxvt_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B7\x1B[?47h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2J\x1B[?47l\x1B8\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;m%?%p9%t^N%e\x0F%;\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[8~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[8$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[7~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[7$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2$\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[d\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[c\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[11~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static screen_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34h\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p1%t;3%;%?%p2%t;4%;%?%p3%t;7%;%?%p4%t;5%;%?%p5%t;2%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static st_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"%?%p9%t\x1B(0%e\x1B(B%;\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p5%t;2%;%?%p7%t;8%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3;5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static tmux_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34h\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p5%t;2%;%?%p7%t;8%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static vte_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\x1B[22;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\x0F\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\x1B[23;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p6%t;1%;%?%p2%t;4%;%?%p4%t;5%;%?%p5%t;2%;%?%p7%t;8%;%?%p1%p3%|%t;7%;m%?%p9%t^N%e\x0F%;\0"
            .as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4:%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
    ],
    keys: [
        [
            b"\x7F\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static xterm_256colour_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\n\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?12l\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[5m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\x1B[22;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B(B\x1B[m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\x1B[23;0;0t\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1l\x1B>\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1h\x1B=\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"%?%p9%t\x1B(0%e\x1B(B%;\x1B[0%?%p6%t;1%;%?%p5%t;2%;%?%p2%t;4%;%?%p1%p3%|%t;7%;%?%p4%t;5%;%?%p7%t;8%;m\0"
            .as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?69h\x1B[%i%p1%d;%p2%ds\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]12;%p1%s\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B]112\x07\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOF\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2F\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOH\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2H\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOD\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1BOC\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1BOP\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOQ\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOR\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1BOS\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;2S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;2~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;5S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;5~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;6~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3R\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;3S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24;3~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4P\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4Q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;4R\0".as_ptr() as *const ::core::ffi::c_char,
    ],
});
static cygwin_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B7\x1B[?47h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[8m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0;10m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2J\x1B[?47l\x1B8\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0;10%?%p1%t;7%;%?%p2%t;4%;%?%p3%t;7%;%?%p6%t;1%;%?%p7%t;8%;%?%p9%t;11%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B];\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static win32con_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: false_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 8 as ::core::ffi::c_int,
    lines: -1 as ::core::ffi::c_int,
    columns: -1 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[H\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B7\x1B[?47h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2J\x1B[?47l\x1B8\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[3%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%t;7%;%?%p3%t;7%;%?%p6%t;1%;m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[0 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[4;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"^Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static conemu_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[48;5;%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[38;5;%p1%dm\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%p3%|%t;7%;%?%p2%t;4%;%?%p6%t;1%;m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[4;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
static vtpcon_terminfo: GlobalCell<TerminfoEntry> = GlobalCell::new(TerminfoEntry {
    bce: true_0 != 0,
    has_Tc_or_RGB: false_0 != 0,
    Su: false_0 != 0,
    max_colors: 0x100 as ::core::ffi::c_int,
    lines: 24 as ::core::ffi::c_int,
    columns: 80 as ::core::ffi::c_int,
    defs: [
        b"\r\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dr\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\x1B[2J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[K\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[J\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%i%p1%d;%p2%dH\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?25l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[H\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?12l\x1B[?25h\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[M\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[1m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049h\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[3m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[7m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[4m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dX\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[0m\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[?1049l\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x07\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[L\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B[%p1%dM\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dB\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dL\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dD\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dC\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%dA\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t4%p1%d%e%p1%{16}%<%t10%p1%{8}%-%d%e48;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[%?%p1%{8}%<%t3%p1%d%e%p1%{16}%<%t9%p1%{8}%-%d%e38;5;%p1%d%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        b"\x1B[0%?%p1%p3%|%t;7%;%?%p2%t;4%;%?%p6%t;1%;%?%p9%t;9%;m\0".as_ptr()
            as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        b"\x1B]0;\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[2 q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[%p1%d q\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[9m\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
    keys: [
        [
            b"\x08\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1BOE\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[Z\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[3~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[3;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[4~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[4;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[1~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[2~\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[2;2~\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[6~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[5~\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            ::core::ptr::null::<::core::ffi::c_char>(),
            ::core::ptr::null::<::core::ffi::c_char>(),
        ],
        [
            b"\x1B[D\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2D\0".as_ptr() as *const ::core::ffi::c_char,
        ],
        [
            b"\x1B[C\0".as_ptr() as *const ::core::ffi::c_char,
            b"\x1B[1;2C\0".as_ptr() as *const ::core::ffi::c_char,
        ],
    ],
    f_keys: [
        b"\x1B[[A\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[B\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[C\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[D\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[[E\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34~\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24$\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[11^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[12^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[13^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[14^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[15^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[17^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[18^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[19^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[20^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[21^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[25^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[26^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[28^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[29^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[31^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[1;6S\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[32^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[33^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[34^\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[23@\0".as_ptr() as *const ::core::ffi::c_char,
        b"\x1B[24@\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ],
});
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
