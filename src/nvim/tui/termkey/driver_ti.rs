use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::os::libc::{__assert_fail, abort, fprintf, sprintf, stderr, strlen, write};
pub use crate::src::nvim::types::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __gid_t, __mode_t, __off64_t, __off_t,
    __time_t, __uid_t, cc_t, keyinfo, size_t, speed_t, ssize_t, tcflag_t, termios, TermKey,
    TermKeyDriver, TermKeyDriverNode, TermKeyEvent, TermKeyKey,
    TermKeyKey_code as C2Rust_Unnamed_1, TermKeyResult, TermKeySym, TermKeyType,
    TermKey_Terminfo_Getstr_Hook, TermKey_method as C2Rust_Unnamed_0, TerminfoEntry, _IO_FILE,
    FILE,
};
extern "C" {
    fn fstat(__fd: ::core::ffi::c_int, __buf: *mut stat) -> ::core::ffi::c_int;
}
pub type __dev_t = ::core::ffi::c_ulong;
pub type __ino_t = ::core::ffi::c_ulong;
pub type __nlink_t = ::core::ffi::c_ulong;
pub type __blksize_t = ::core::ffi::c_long;
pub type __blkcnt_t = ::core::ffi::c_long;
pub type __syscall_slong_t = ::core::ffi::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct stat {
    pub st_dev: __dev_t,
    pub st_ino: __ino_t,
    pub st_nlink: __nlink_t,
    pub st_mode: __mode_t,
    pub st_uid: __uid_t,
    pub st_gid: __gid_t,
    pub __pad0: ::core::ffi::c_int,
    pub st_rdev: __dev_t,
    pub st_size: __off_t,
    pub st_blksize: __blksize_t,
    pub st_blocks: __blkcnt_t,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    pub __glibc_reserved: [__syscall_slong_t; 3],
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const kTermCount: C2Rust_Unnamed = 49;
pub const kTerm_set_underline_style: C2Rust_Unnamed = 48;
pub const kTerm_reset_cursor_color: C2Rust_Unnamed = 47;
pub const kTerm_set_cursor_color: C2Rust_Unnamed = 46;
pub const kTerm_set_rgb_background: C2Rust_Unnamed = 45;
pub const kTerm_set_rgb_foreground: C2Rust_Unnamed = 44;
pub const kTerm_enter_strikethrough_mode: C2Rust_Unnamed = 43;
pub const kTerm_set_cursor_style: C2Rust_Unnamed = 42;
pub const kTerm_reset_cursor_style: C2Rust_Unnamed = 41;
pub const kTerm_to_status_line: C2Rust_Unnamed = 40;
pub const kTerm_set_lr_margin: C2Rust_Unnamed = 39;
pub const kTerm_set_attributes: C2Rust_Unnamed = 38;
pub const kTerm_set_a_foreground: C2Rust_Unnamed = 37;
pub const kTerm_set_a_background: C2Rust_Unnamed = 36;
pub const kTerm_parm_up_cursor: C2Rust_Unnamed = 35;
pub const kTerm_parm_right_cursor: C2Rust_Unnamed = 34;
pub const kTerm_parm_left_cursor: C2Rust_Unnamed = 33;
pub const kTerm_parm_insert_line: C2Rust_Unnamed = 32;
pub const kTerm_parm_down_cursor: C2Rust_Unnamed = 31;
pub const kTerm_parm_delete_line: C2Rust_Unnamed = 30;
pub const kTerm_keypad_xmit: C2Rust_Unnamed = 29;
pub const kTerm_keypad_local: C2Rust_Unnamed = 28;
pub const kTerm_insert_line: C2Rust_Unnamed = 27;
pub const kTerm_from_status_line: C2Rust_Unnamed = 26;
pub const kTerm_exit_ca_mode: C2Rust_Unnamed = 25;
pub const kTerm_exit_attribute_mode: C2Rust_Unnamed = 24;
pub const kTerm_erase_chars: C2Rust_Unnamed = 23;
pub const kTerm_enter_underline_mode: C2Rust_Unnamed = 22;
pub const kTerm_enter_standout_mode: C2Rust_Unnamed = 21;
pub const kTerm_enter_secure_mode: C2Rust_Unnamed = 20;
pub const kTerm_enter_reverse_mode: C2Rust_Unnamed = 19;
pub const kTerm_enter_italics_mode: C2Rust_Unnamed = 18;
pub const kTerm_enter_dim_mode: C2Rust_Unnamed = 17;
pub const kTerm_enter_ca_mode: C2Rust_Unnamed = 16;
pub const kTerm_enter_bold_mode: C2Rust_Unnamed = 15;
pub const kTerm_enter_blink_mode: C2Rust_Unnamed = 14;
pub const kTerm_delete_line: C2Rust_Unnamed = 13;
pub const kTerm_cursor_right: C2Rust_Unnamed = 12;
pub const kTerm_cursor_up: C2Rust_Unnamed = 11;
pub const kTerm_cursor_normal: C2Rust_Unnamed = 10;
pub const kTerm_cursor_home: C2Rust_Unnamed = 9;
pub const kTerm_cursor_left: C2Rust_Unnamed = 8;
pub const kTerm_cursor_invisible: C2Rust_Unnamed = 7;
pub const kTerm_cursor_down: C2Rust_Unnamed = 6;
pub const kTerm_cursor_address: C2Rust_Unnamed = 5;
pub const kTerm_clr_eos: C2Rust_Unnamed = 4;
pub const kTerm_clr_eol: C2Rust_Unnamed = 3;
pub const kTerm_clear_screen: C2Rust_Unnamed = 2;
pub const kTerm_change_scroll_region: C2Rust_Unnamed = 1;
pub const kTerm_carriage_return: C2Rust_Unnamed = 0;
pub type TerminfoKey = ::core::ffi::c_uint;
pub const kTermKeyCount: TerminfoKey = 16;
pub const kTermKey_right: TerminfoKey = 15;
pub const kTermKey_left: TerminfoKey = 14;
pub const kTermKey_undo: TerminfoKey = 13;
pub const kTermKey_suspend: TerminfoKey = 12;
pub const kTermKey_select: TerminfoKey = 11;
pub const kTermKey_ppage: TerminfoKey = 10;
pub const kTermKey_npage: TerminfoKey = 9;
pub const kTermKey_ic: TerminfoKey = 8;
pub const kTermKey_home: TerminfoKey = 7;
pub const kTermKey_find: TerminfoKey = 6;
pub const kTermKey_end: TerminfoKey = 5;
pub const kTermKey_dc: TerminfoKey = 4;
pub const kTermKey_clear: TerminfoKey = 3;
pub const kTermKey_btab: TerminfoKey = 2;
pub const kTermKey_beg: TerminfoKey = 1;
pub const kTermKey_backspace: TerminfoKey = 0;
pub const TERMKEY_EVENT_RELEASE: TermKeyEvent = 3;
pub const TERMKEY_EVENT_REPEAT: TermKeyEvent = 2;
pub const TERMKEY_EVENT_PRESS: TermKeyEvent = 1;
pub const TERMKEY_EVENT_UNKNOWN: TermKeyEvent = 0;
pub const TERMKEY_N_SYMS: TermKeySym = 60;
pub const TERMKEY_SYM_KPEQUALS: TermKeySym = 59;
pub const TERMKEY_SYM_KPPERIOD: TermKeySym = 58;
pub const TERMKEY_SYM_KPCOMMA: TermKeySym = 57;
pub const TERMKEY_SYM_KPDIV: TermKeySym = 56;
pub const TERMKEY_SYM_KPMULT: TermKeySym = 55;
pub const TERMKEY_SYM_KPMINUS: TermKeySym = 54;
pub const TERMKEY_SYM_KPPLUS: TermKeySym = 53;
pub const TERMKEY_SYM_KPENTER: TermKeySym = 52;
pub const TERMKEY_SYM_KP9: TermKeySym = 51;
pub const TERMKEY_SYM_KP8: TermKeySym = 50;
pub const TERMKEY_SYM_KP7: TermKeySym = 49;
pub const TERMKEY_SYM_KP6: TermKeySym = 48;
pub const TERMKEY_SYM_KP5: TermKeySym = 47;
pub const TERMKEY_SYM_KP4: TermKeySym = 46;
pub const TERMKEY_SYM_KP3: TermKeySym = 45;
pub const TERMKEY_SYM_KP2: TermKeySym = 44;
pub const TERMKEY_SYM_KP1: TermKeySym = 43;
pub const TERMKEY_SYM_KP0: TermKeySym = 42;
pub const TERMKEY_SYM_UNDO: TermKeySym = 41;
pub const TERMKEY_SYM_SUSPEND: TermKeySym = 40;
pub const TERMKEY_SYM_SAVE: TermKeySym = 39;
pub const TERMKEY_SYM_RESUME: TermKeySym = 38;
pub const TERMKEY_SYM_RESTART: TermKeySym = 37;
pub const TERMKEY_SYM_REPLACE: TermKeySym = 36;
pub const TERMKEY_SYM_REFRESH: TermKeySym = 35;
pub const TERMKEY_SYM_REFERENCE: TermKeySym = 34;
pub const TERMKEY_SYM_REDO: TermKeySym = 33;
pub const TERMKEY_SYM_PRINT: TermKeySym = 32;
pub const TERMKEY_SYM_OPTIONS: TermKeySym = 31;
pub const TERMKEY_SYM_OPEN: TermKeySym = 30;
pub const TERMKEY_SYM_MOVE: TermKeySym = 29;
pub const TERMKEY_SYM_MESSAGE: TermKeySym = 28;
pub const TERMKEY_SYM_MARK: TermKeySym = 27;
pub const TERMKEY_SYM_HELP: TermKeySym = 26;
pub const TERMKEY_SYM_EXIT: TermKeySym = 25;
pub const TERMKEY_SYM_COPY: TermKeySym = 24;
pub const TERMKEY_SYM_COMMAND: TermKeySym = 23;
pub const TERMKEY_SYM_CLOSE: TermKeySym = 22;
pub const TERMKEY_SYM_CLEAR: TermKeySym = 21;
pub const TERMKEY_SYM_CANCEL: TermKeySym = 20;
pub const TERMKEY_SYM_END: TermKeySym = 19;
pub const TERMKEY_SYM_HOME: TermKeySym = 18;
pub const TERMKEY_SYM_PAGEDOWN: TermKeySym = 17;
pub const TERMKEY_SYM_PAGEUP: TermKeySym = 16;
pub const TERMKEY_SYM_SELECT: TermKeySym = 15;
pub const TERMKEY_SYM_DELETE: TermKeySym = 14;
pub const TERMKEY_SYM_INSERT: TermKeySym = 13;
pub const TERMKEY_SYM_FIND: TermKeySym = 12;
pub const TERMKEY_SYM_BEGIN: TermKeySym = 11;
pub const TERMKEY_SYM_RIGHT: TermKeySym = 10;
pub const TERMKEY_SYM_LEFT: TermKeySym = 9;
pub const TERMKEY_SYM_DOWN: TermKeySym = 8;
pub const TERMKEY_SYM_UP: TermKeySym = 7;
pub const TERMKEY_SYM_DEL: TermKeySym = 6;
pub const TERMKEY_SYM_SPACE: TermKeySym = 5;
pub const TERMKEY_SYM_ESCAPE: TermKeySym = 4;
pub const TERMKEY_SYM_ENTER: TermKeySym = 3;
pub const TERMKEY_SYM_TAB: TermKeySym = 2;
pub const TERMKEY_SYM_BACKSPACE: TermKeySym = 1;
pub const TERMKEY_SYM_NONE: TermKeySym = 0;
pub const TERMKEY_SYM_UNKNOWN: TermKeySym = -1;
pub const TERMKEY_TYPE_UNKNOWN_CSI: TermKeyType = -1;
pub const TERMKEY_TYPE_APC: TermKeyType = 8;
pub const TERMKEY_TYPE_OSC: TermKeyType = 7;
pub const TERMKEY_TYPE_DCS: TermKeyType = 6;
pub const TERMKEY_TYPE_MODEREPORT: TermKeyType = 5;
pub const TERMKEY_TYPE_POSITION: TermKeyType = 4;
pub const TERMKEY_TYPE_MOUSE: TermKeyType = 3;
pub const TERMKEY_TYPE_KEYSYM: TermKeyType = 2;
pub const TERMKEY_TYPE_FUNCTION: TermKeyType = 1;
pub const TERMKEY_TYPE_UNICODE: TermKeyType = 0;
pub const TERMKEY_RES_ERROR: TermKeyResult = 4;
pub const TERMKEY_RES_AGAIN: TermKeyResult = 3;
pub const TERMKEY_RES_EOF: TermKeyResult = 2;
pub const TERMKEY_RES_KEY: TermKeyResult = 1;
pub const TERMKEY_RES_NONE: TermKeyResult = 0;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const TERMKEY_KEYMOD_CTRL: C2Rust_Unnamed_2 = 4;
pub const TERMKEY_KEYMOD_ALT: C2Rust_Unnamed_2 = 2;
pub const TERMKEY_KEYMOD_SHIFT: C2Rust_Unnamed_2 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyTI {
    pub tk: *mut TermKey,
    pub ti: *mut TerminfoEntry,
    pub root: *mut trie_node,
    pub start_string: *mut ::core::ffi::c_char,
    pub stop_string: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct trie_node {
    pub type_0: trie_nodetype,
}
pub type trie_nodetype = ::core::ffi::c_uint;
pub const TYPE_ARR: trie_nodetype = 1;
pub const TYPE_KEY: trie_nodetype = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct trie_node_arr {
    pub type_0: trie_nodetype,
    pub min: ::core::ffi::c_uchar,
    pub max: ::core::ffi::c_uchar,
    pub arr: [*mut trie_node; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct trie_node_key {
    pub type_0: trie_nodetype,
    pub key: keyinfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub ti_key: TerminfoKey,
    pub funcname: *const ::core::ffi::c_char,
    pub type_0: TermKeyType,
    pub sym: TermKeySym,
    pub mods: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const kTerminfoFuncKeyMax: ::core::ffi::c_int = 63 as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 92] = unsafe {
    ::core::mem::transmute::<
        [u8; 92],
        [::core::ffi::c_char; 92],
    >(
        *b"_Bool try_load_terminfo_key(TermKeyTI *, _Bool, int, _Bool, const char *, struct keyinfo *)\0",
    )
};
static funcs: GlobalCell<[C2Rust_Unnamed_3; 17]> = GlobalCell::new([
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_backspace,
        funcname: b"backspace\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_BACKSPACE,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_beg,
        funcname: b"beg\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_BEGIN,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_btab,
        funcname: b"btab\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_TAB,
        mods: TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_clear,
        funcname: b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_CLEAR,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_dc,
        funcname: b"dc\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_DELETE,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_end,
        funcname: b"end\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_END,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_find,
        funcname: b"find\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_FIND,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_home,
        funcname: b"home\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_HOME,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_ic,
        funcname: b"ic\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_INSERT,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_left,
        funcname: b"left\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_LEFT,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_npage,
        funcname: b"npage\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_PAGEDOWN,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_ppage,
        funcname: b"ppage\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_PAGEUP,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_right,
        funcname: b"right\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_RIGHT,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_select,
        funcname: b"select\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_SELECT,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_suspend,
        funcname: b"suspend\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_SUSPEND,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_undo,
        funcname: b"undo\0".as_ptr() as *const ::core::ffi::c_char,
        type_0: TERMKEY_TYPE_KEYSYM,
        sym: TERMKEY_SYM_UNDO,
        mods: 0 as ::core::ffi::c_int,
    },
    C2Rust_Unnamed_3 {
        ti_key: kTermKey_backspace,
        funcname: ::core::ptr::null::<::core::ffi::c_char>(),
        type_0: TERMKEY_TYPE_UNICODE,
        sym: TERMKEY_SYM_NONE,
        mods: 0 as ::core::ffi::c_int,
    },
]);
unsafe extern "C" fn new_node_key(
    mut type_0: TermKeyType,
    mut sym: TermKeySym,
    mut modmask: ::core::ffi::c_int,
    mut modset: ::core::ffi::c_int,
) -> *mut trie_node {
    let mut n: *mut trie_node_key =
        xmalloc(::core::mem::size_of::<trie_node_key>()) as *mut trie_node_key;
    (*n).type_0 = TYPE_KEY;
    (*n).key.type_0 = type_0;
    (*n).key.sym = sym;
    (*n).key.modifier_mask = modmask;
    (*n).key.modifier_set = modset;
    return n as *mut trie_node;
}
unsafe extern "C" fn new_node_arr(
    mut min: ::core::ffi::c_uchar,
    mut max: ::core::ffi::c_uchar,
) -> *mut trie_node {
    let mut n: *mut trie_node_arr = xmalloc(
        ::core::mem::size_of::<trie_node_arr>().wrapping_add(
            ((max as ::core::ffi::c_int - min as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut trie_node>()),
        ),
    ) as *mut trie_node_arr;
    (*n).type_0 = TYPE_ARR;
    (*n).min = min;
    (*n).max = max;
    let mut i: ::core::ffi::c_int = 0;
    i = min as ::core::ffi::c_int;
    while i <= max as ::core::ffi::c_int {
        *(&raw mut (*n).arr as *mut *mut trie_node)
            .offset((i - min as ::core::ffi::c_int) as isize) =
            ::core::ptr::null_mut::<trie_node>();
        i += 1;
    }
    return n as *mut trie_node;
}
unsafe extern "C" fn lookup_next(
    mut n: *mut trie_node,
    mut b: ::core::ffi::c_uchar,
) -> *mut trie_node {
    match (*n).type_0 as ::core::ffi::c_uint {
        0 => {
            fprintf(
                stderr,
                b"ABORT: lookup_next within a TYPE_KEY node\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            abort();
        }
        1 => {
            let mut nar: *mut trie_node_arr = n as *mut trie_node_arr;
            if (b as ::core::ffi::c_int) < (*nar).min as ::core::ffi::c_int
                || b as ::core::ffi::c_int > (*nar).max as ::core::ffi::c_int
            {
                return ::core::ptr::null_mut::<trie_node>();
            }
            return *(&raw mut (*nar).arr as *mut *mut trie_node)
                .offset((b as ::core::ffi::c_int - (*nar).min as ::core::ffi::c_int) as isize);
        }
        _ => {}
    }
    return ::core::ptr::null_mut::<trie_node>();
}
unsafe extern "C" fn free_trie(mut n: *mut trie_node) {
    match (*n).type_0 as ::core::ffi::c_uint {
        1 => {
            let mut nar: *mut trie_node_arr = n as *mut trie_node_arr;
            let mut i: ::core::ffi::c_int = 0;
            i = (*nar).min as ::core::ffi::c_int;
            while i <= (*nar).max as ::core::ffi::c_int {
                if !(*(&raw mut (*nar).arr as *mut *mut trie_node)
                    .offset((i - (*nar).min as ::core::ffi::c_int) as isize))
                .is_null()
                {
                    free_trie(
                        *(&raw mut (*nar).arr as *mut *mut trie_node)
                            .offset((i - (*nar).min as ::core::ffi::c_int) as isize),
                    );
                }
                i += 1;
            }
        }
        0 | _ => {}
    }
    xfree(n as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn compress_trie(mut n: *mut trie_node) -> *mut trie_node {
    if n.is_null() {
        return ::core::ptr::null_mut::<trie_node>();
    }
    match (*n).type_0 as ::core::ffi::c_uint {
        0 => return n,
        1 => {
            let mut nar: *mut trie_node_arr = n as *mut trie_node_arr;
            let mut min: ::core::ffi::c_uchar = 0;
            let mut max: ::core::ffi::c_uchar = 0;
            min = 0 as ::core::ffi::c_uchar;
            while (*(&raw mut (*nar).arr as *mut *mut trie_node).offset(min as isize)).is_null() {
                if min as ::core::ffi::c_int == 255 as ::core::ffi::c_int
                    && (*(&raw mut (*nar).arr as *mut *mut trie_node).offset(min as isize))
                        .is_null()
                {
                    xfree(nar as *mut ::core::ffi::c_void);
                    return new_node_arr(1 as ::core::ffi::c_uchar, 0 as ::core::ffi::c_uchar);
                }
                min = min.wrapping_add(1);
            }
            max = 0xff as ::core::ffi::c_uchar;
            while (*(&raw mut (*nar).arr as *mut *mut trie_node).offset(max as isize)).is_null() {
                max = max.wrapping_sub(1);
            }
            let mut new: *mut trie_node_arr = new_node_arr(min, max) as *mut trie_node_arr;
            let mut i: ::core::ffi::c_int = 0;
            i = min as ::core::ffi::c_int;
            while i <= max as ::core::ffi::c_int {
                *(&raw mut (*new).arr as *mut *mut trie_node)
                    .offset((i - min as ::core::ffi::c_int) as isize) =
                    compress_trie(*(&raw mut (*nar).arr as *mut *mut trie_node).offset(i as isize));
                i += 1;
            }
            xfree(nar as *mut ::core::ffi::c_void);
            return new as *mut trie_node;
        }
        _ => {}
    }
    return n;
}
unsafe extern "C" fn try_load_terminfo_key(
    mut ti: *mut TermKeyTI,
    mut fn_nr: bool,
    mut key: ::core::ffi::c_int,
    mut shift: bool,
    mut name: *const ::core::ffi::c_char,
    mut info: *mut keyinfo,
) -> bool {
    let mut value: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    if !(*ti).ti.is_null() {
        if !fn_nr {
            value = (*(*ti).ti).keys[key as usize][(if shift as ::core::ffi::c_int != 0 {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as usize];
        } else {
            '_c2rust_label: {
                if !shift {
                } else {
                    __assert_fail(
                        b"!shift\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/tui/termkey/driver_ti.rs\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        220 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            value = (*(*ti).ti).f_keys[key as usize];
        }
    }
    if (*(*ti).tk).ti_getstr_hook.is_some() {
        value = (*(*ti).tk)
            .ti_getstr_hook
            .expect("non-null function pointer")(
            name, value, (*(*ti).tk).ti_getstr_hook_data
        );
    }
    if value.is_null()
        || value
            == ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_char>(
                -1 as ::core::ffi::c_int as usize,
            ) as *const ::core::ffi::c_char
        || *value.offset(0 as ::core::ffi::c_int as isize) == 0
    {
        return false_0 != 0;
    }
    let mut node: *mut trie_node = new_node_key(
        (*info).type_0,
        (*info).sym,
        (*info).modifier_mask,
        (*info).modifier_set,
    );
    insert_seq(ti, value, node);
    return true_0 != 0;
}
unsafe extern "C" fn load_terminfo(mut ti: *mut TermKeyTI) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    (*ti).root =
        new_node_arr(0 as ::core::ffi::c_uchar, 0xff as ::core::ffi::c_uchar) as *mut trie_node;
    if (*ti).root.is_null() {
        return 0 as ::core::ffi::c_int;
    }
    i = 0 as ::core::ffi::c_int;
    while !(*funcs.ptr())[i as usize].funcname.is_null() {
        let mut name: [::core::ffi::c_char; 15] = [0; 15];
        sprintf(
            &raw mut name as *mut ::core::ffi::c_char,
            b"key_%s\0".as_ptr() as *const ::core::ffi::c_char,
            (*funcs.ptr())[i as usize].funcname,
        );
        let mut c2rust_lvalue: keyinfo = keyinfo {
            type_0: (*funcs.ptr())[i as usize].type_0,
            sym: (*funcs.ptr())[i as usize].sym,
            modifier_mask: (*funcs.ptr())[i as usize].mods,
            modifier_set: (*funcs.ptr())[i as usize].mods,
        };
        if try_load_terminfo_key(
            ti,
            false_0 != 0,
            (*funcs.ptr())[i as usize].ti_key as ::core::ffi::c_int,
            false_0 != 0,
            &raw mut name as *mut ::core::ffi::c_char,
            &raw mut c2rust_lvalue,
        ) {
            sprintf(
                &raw mut name as *mut ::core::ffi::c_char,
                b"key_s%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*funcs.ptr())[i as usize].funcname,
            );
            let mut c2rust_lvalue_0: keyinfo = keyinfo {
                type_0: (*funcs.ptr())[i as usize].type_0,
                sym: (*funcs.ptr())[i as usize].sym,
                modifier_mask: (*funcs.ptr())[i as usize].mods
                    | TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int,
                modifier_set: (*funcs.ptr())[i as usize].mods
                    | TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int,
            };
            try_load_terminfo_key(
                ti,
                false_0 != 0,
                (*funcs.ptr())[i as usize].ti_key as ::core::ffi::c_int,
                true_0 != 0,
                &raw mut name as *mut ::core::ffi::c_char,
                &raw mut c2rust_lvalue_0,
            );
        }
        i += 1;
    }
    i = 1 as ::core::ffi::c_int;
    while i <= kTerminfoFuncKeyMax {
        let mut name_0: [::core::ffi::c_char; 9] = [0; 9];
        sprintf(
            &raw mut name_0 as *mut ::core::ffi::c_char,
            b"key_f%d\0".as_ptr() as *const ::core::ffi::c_char,
            i,
        );
        let mut c2rust_lvalue_1: keyinfo = keyinfo {
            type_0: TERMKEY_TYPE_FUNCTION,
            sym: i as TermKeySym,
            modifier_mask: 0 as ::core::ffi::c_int,
            modifier_set: 0 as ::core::ffi::c_int,
        };
        if !try_load_terminfo_key(
            ti,
            true_0 != 0,
            i - 1 as ::core::ffi::c_int,
            false_0 != 0,
            &raw mut name_0 as *mut ::core::ffi::c_char,
            &raw mut c2rust_lvalue_1,
        ) {
            break;
        }
        i += 1;
    }
    let mut keypad_xmit: *const ::core::ffi::c_char = if !(*ti).ti.is_null() {
        (*(*ti).ti).defs[kTerm_keypad_xmit as ::core::ffi::c_int as usize]
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
    if !keypad_xmit.is_null() {
        (*ti).start_string = xstrdup(keypad_xmit);
    } else {
        (*ti).start_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut keypad_local: *const ::core::ffi::c_char = if !(*ti).ti.is_null() {
        (*(*ti).ti).defs[kTerm_keypad_local as ::core::ffi::c_int as usize]
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
    if !keypad_local.is_null() {
        (*ti).stop_string = xstrdup(keypad_local);
    } else {
        (*ti).stop_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    (*ti).root = compress_trie((*ti).root as *mut trie_node) as *mut trie_node;
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn new_driver_ti(
    mut tk: *mut TermKey,
    mut term: *mut TerminfoEntry,
) -> *mut ::core::ffi::c_void {
    let mut ti: *mut TermKeyTI = xmalloc(::core::mem::size_of::<TermKeyTI>()) as *mut TermKeyTI;
    (*ti).tk = tk;
    (*ti).root = ::core::ptr::null_mut::<trie_node>();
    (*ti).start_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ti).stop_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*ti).ti = term;
    return ti as *mut ::core::ffi::c_void;
}
pub unsafe extern "C" fn start_driver_ti(
    mut tk: *mut TermKey,
    mut info: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut ti: *mut TermKeyTI = info as *mut TermKeyTI;
    let mut statbuf: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    let mut start_string: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    if (*ti).root.is_null() {
        load_terminfo(ti);
    }
    start_string = (*ti).start_string;
    if (*tk).fd == -1 as ::core::ffi::c_int || start_string.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if fstat((*tk).fd, &raw mut statbuf) == -1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if statbuf.st_mode & __S_IFMT as __mode_t == 0o10000 as __mode_t {
        return 1 as ::core::ffi::c_int;
    }
    len = strlen(start_string);
    while len != 0 {
        let mut result: ssize_t = write(
            (*tk).fd,
            start_string as *const ::core::ffi::c_void,
            len as ::core::ffi::c_uint as size_t,
        );
        if result < 0 as ssize_t {
            return 0 as ::core::ffi::c_int;
        }
        let mut written: size_t = result as size_t;
        start_string = start_string.offset(written as isize);
        len = len.wrapping_sub(written);
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn stop_driver_ti(
    mut tk: *mut TermKey,
    mut info: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut ti: *mut TermKeyTI = info as *mut TermKeyTI;
    let mut statbuf: stat = stat {
        st_dev: 0,
        st_ino: 0,
        st_nlink: 0,
        st_mode: 0,
        st_uid: 0,
        st_gid: 0,
        __pad0: 0,
        st_rdev: 0,
        st_size: 0,
        st_blksize: 0,
        st_blocks: 0,
        st_atim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_mtim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        st_ctim: timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        __glibc_reserved: [0; 3],
    };
    let mut stop_string: *mut ::core::ffi::c_char = (*ti).stop_string;
    let mut len: size_t = 0;
    if (*tk).fd == -1 as ::core::ffi::c_int || stop_string.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if fstat((*tk).fd, &raw mut statbuf) == -1 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if statbuf.st_mode & __S_IFMT as __mode_t == 0o10000 as __mode_t {
        return 1 as ::core::ffi::c_int;
    }
    len = strlen(stop_string);
    while len != 0 {
        let mut result: ssize_t = write(
            (*tk).fd,
            stop_string as *const ::core::ffi::c_void,
            len as ::core::ffi::c_uint as size_t,
        );
        if result < 0 as ssize_t {
            return 0 as ::core::ffi::c_int;
        }
        let mut written: size_t = result as size_t;
        stop_string = stop_string.offset(written as isize);
        len = len.wrapping_sub(written);
    }
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn free_driver_ti(mut info: *mut ::core::ffi::c_void) {
    let mut ti: *mut TermKeyTI = info as *mut TermKeyTI;
    free_trie((*ti).root as *mut trie_node);
    if !(*ti).start_string.is_null() {
        xfree((*ti).start_string as *mut ::core::ffi::c_void);
    }
    if !(*ti).stop_string.is_null() {
        xfree((*ti).stop_string as *mut ::core::ffi::c_void);
    }
    xfree(ti as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn peekkey_ti(
    mut tk: *mut TermKey,
    mut info: *mut ::core::ffi::c_void,
    mut key: *mut TermKeyKey,
    mut force: ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    let mut ti: *mut TermKeyTI = info as *mut TermKeyTI;
    if (*tk).buffcount == 0 as size_t {
        return (if (*tk).is_closed as ::core::ffi::c_int != 0 {
            TERMKEY_RES_EOF as ::core::ffi::c_int
        } else {
            TERMKEY_RES_NONE as ::core::ffi::c_int
        }) as TermKeyResult;
    }
    let mut p: *mut trie_node = (*ti).root as *mut trie_node;
    let mut pos: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    while (pos as size_t) < (*tk).buffcount {
        p = lookup_next(
            p,
            *(*tk)
                .buffer
                .offset((*tk).buffstart.wrapping_add(pos as size_t) as isize),
        );
        if p.is_null() {
            break;
        }
        pos = pos.wrapping_add(1);
        if (*p).type_0 as ::core::ffi::c_uint
            != TYPE_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            continue;
        }
        let mut nk: *mut trie_node_key = p as *mut trie_node_key;
        if (*nk).key.type_0 as ::core::ffi::c_int == TERMKEY_TYPE_MOUSE as ::core::ffi::c_int {
            (*tk).buffstart = (*tk).buffstart.wrapping_add(pos as size_t);
            (*tk).buffcount = (*tk).buffcount.wrapping_sub(pos as size_t);
            let mut mouse_result: TermKeyResult =
                Some(
                    (*tk)
                        .method
                        .peekkey_mouse
                        .expect("non-null function pointer"),
                )
                .expect("non-null function pointer")(tk, key, nbytep);
            (*tk).buffstart = (*tk).buffstart.wrapping_sub(pos as size_t);
            (*tk).buffcount = (*tk).buffcount.wrapping_add(pos as size_t);
            if mouse_result as ::core::ffi::c_uint
                == TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *nbytep = (*nbytep).wrapping_add(pos as size_t);
            }
            return mouse_result;
        }
        (*key).type_0 = (*nk).key.type_0;
        (*key).code.sym = (*nk).key.sym;
        (*key).modifiers = (*nk).key.modifier_set;
        *nbytep = pos as size_t;
        return TERMKEY_RES_KEY;
    }
    if !p.is_null() && force == 0 {
        return TERMKEY_RES_AGAIN;
    }
    return TERMKEY_RES_NONE;
}
unsafe extern "C" fn insert_seq(
    mut ti: *mut TermKeyTI,
    mut seq: *const ::core::ffi::c_char,
    mut node: *mut trie_node,
) -> ::core::ffi::c_int {
    let mut pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: *mut trie_node = (*ti).root as *mut trie_node;
    let mut b: ::core::ffi::c_uchar = 0;
    loop {
        b = *seq.offset(pos as isize) as ::core::ffi::c_uchar;
        if b == 0 {
            break;
        }
        let mut next: *mut trie_node = lookup_next(p, b);
        if next.is_null() {
            break;
        }
        p = next;
        pos += 1;
    }
    loop {
        b = *seq.offset(pos as isize) as ::core::ffi::c_uchar;
        if b == 0 {
            break;
        }
        let mut next_0: *mut trie_node = ::core::ptr::null_mut::<trie_node>();
        if *seq.offset((pos + 1 as ::core::ffi::c_int) as isize) != 0 {
            next_0 = new_node_arr(0 as ::core::ffi::c_uchar, 0xff as ::core::ffi::c_uchar);
        } else {
            next_0 = node;
        }
        if next_0.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        match (*p).type_0 as ::core::ffi::c_uint {
            1 => {
                let mut nar: *mut trie_node_arr = p as *mut trie_node_arr;
                if (b as ::core::ffi::c_int) < (*nar).min as ::core::ffi::c_int
                    || b as ::core::ffi::c_int > (*nar).max as ::core::ffi::c_int
                {
                    fprintf(
                        stderr,
                        b"ASSERT FAIL: Trie insert at 0x%02x is outside of extent bounds (0x%02x..0x%02x)\n\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        b as ::core::ffi::c_int,
                        (*nar).min as ::core::ffi::c_int,
                        (*nar).max as ::core::ffi::c_int,
                    );
                    abort();
                }
                *(&raw mut (*nar).arr as *mut *mut trie_node).offset(
                    (b as ::core::ffi::c_int - (*nar).min as ::core::ffi::c_int) as isize,
                ) = next_0;
                p = next_0;
            }
            0 => {
                fprintf(
                    stderr,
                    b"ASSERT FAIL: Tried to insert child node in TYPE_KEY\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                abort();
            }
            _ => {}
        }
        pos += 1;
    }
    return 1 as ::core::ffi::c_int;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
