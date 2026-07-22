use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::mbyte::{utf_char2bytes, utf_char2len};
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc};
use crate::src::nvim::os::libc::{
    __ctype_b_loc, __errno_location, memcpy, memmove, snprintf, strlen, strncmp, tcgetattr,
    tcsetattr, tolower,
};
use crate::src::nvim::tui::termkey::driver_csi::{
    free_driver_csi, new_driver_csi, peekkey_csi, termkey_interpret_modereport,
    termkey_interpret_mouse,
};
use crate::src::nvim::tui::termkey::driver_ti::{
    free_driver_ti, new_driver_ti, peekkey_ti, start_driver_ti, stop_driver_ti,
};
pub use crate::src::nvim::types::{
    cc_t, keyinfo, size_t, speed_t, tcflag_t, termios, TermKey, TermKeyCsi, TermKeyDriver,
    TermKeyDriverNode, TermKeyEvent, TermKeyFormat, TermKeyKey,
    TermKeyKey_code as C2Rust_Unnamed_2, TermKeyMouseEvent, TermKeyResult, TermKeySym, TermKeyType,
    TermKey_Terminfo_Getstr_Hook, TermKey_method as C2Rust_Unnamed_1, TerminfoEntry,
};
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
pub const UNICODE_INVALID: C2Rust_Unnamed_0 = 65533;
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
pub const TERMKEY_MOUSE_RELEASE: TermKeyMouseEvent = 3;
pub const TERMKEY_MOUSE_DRAG: TermKeyMouseEvent = 2;
pub const TERMKEY_MOUSE_PRESS: TermKeyMouseEvent = 1;
pub const TERMKEY_MOUSE_UNKNOWN: TermKeyMouseEvent = 0;
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const TERMKEY_KEYMOD_CTRL: C2Rust_Unnamed_3 = 4;
pub const TERMKEY_KEYMOD_ALT: C2Rust_Unnamed_3 = 2;
pub const TERMKEY_KEYMOD_SHIFT: C2Rust_Unnamed_3 = 1;
pub type C2Rust_Unnamed_4 = ::core::ffi::c_uint;
pub const TERMKEY_FLAG_KEEPC0: C2Rust_Unnamed_4 = 512;
pub const TERMKEY_FLAG_NOSTART: C2Rust_Unnamed_4 = 256;
pub const TERMKEY_FLAG_EINTR: C2Rust_Unnamed_4 = 128;
pub const TERMKEY_FLAG_CTRLC: C2Rust_Unnamed_4 = 64;
pub const TERMKEY_FLAG_SPACESYMBOL: C2Rust_Unnamed_4 = 32;
pub const TERMKEY_FLAG_NOTERMIOS: C2Rust_Unnamed_4 = 16;
pub const TERMKEY_FLAG_UTF8: C2Rust_Unnamed_4 = 8;
pub const TERMKEY_FLAG_RAW: C2Rust_Unnamed_4 = 4;
pub const TERMKEY_FLAG_CONVERTKP: C2Rust_Unnamed_4 = 2;
pub const TERMKEY_FLAG_NOINTERPRET: C2Rust_Unnamed_4 = 1;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_uint;
pub const TERMKEY_CANON_DELBS: C2Rust_Unnamed_5 = 2;
pub const TERMKEY_CANON_SPACESYMBOL: C2Rust_Unnamed_5 = 1;
pub const TERMKEY_FORMAT_MOUSE_POS: TermKeyFormat = 256;
pub const TERMKEY_FORMAT_LOWERSPACE: TermKeyFormat = 64;
pub const TERMKEY_FORMAT_LOWERMOD: TermKeyFormat = 32;
pub const TERMKEY_FORMAT_SPACEMOD: TermKeyFormat = 16;
pub const TERMKEY_FORMAT_WRAPBRACKET: TermKeyFormat = 8;
pub const TERMKEY_FORMAT_ALTISMETA: TermKeyFormat = 4;
pub const TERMKEY_FORMAT_CARETCTRL: TermKeyFormat = 2;
pub const TERMKEY_FORMAT_LONGMOD: TermKeyFormat = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_6 {
    pub sym: TermKeySym,
    pub name: *const ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct modnames {
    pub shift: *const ::core::ffi::c_char,
    pub alt: *const ::core::ffi::c_char,
    pub ctrl: *const ::core::ffi::c_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub static termkey_driver_ti: GlobalCell<TermKeyDriver> = GlobalCell::new(TermKeyDriver {
    name: b"terminfo\0".as_ptr() as *const ::core::ffi::c_char,
    new_driver: Some(
        new_driver_ti
            as unsafe extern "C" fn(*mut TermKey, *mut TerminfoEntry) -> *mut ::core::ffi::c_void,
    ),
    free_driver: Some(free_driver_ti as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    start_driver: Some(
        start_driver_ti
            as unsafe extern "C" fn(*mut TermKey, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    ),
    stop_driver: Some(
        stop_driver_ti
            as unsafe extern "C" fn(*mut TermKey, *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    ),
    peekkey: Some(
        peekkey_ti
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut ::core::ffi::c_void,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut size_t,
            ) -> TermKeyResult,
    ),
});
pub static termkey_driver_csi: GlobalCell<TermKeyDriver> = GlobalCell::new(TermKeyDriver {
    name: b"CSI\0".as_ptr() as *const ::core::ffi::c_char,
    new_driver: Some(
        new_driver_csi
            as unsafe extern "C" fn(*mut TermKey, *mut TerminfoEntry) -> *mut ::core::ffi::c_void,
    ),
    free_driver: Some(free_driver_csi as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
    start_driver: None,
    stop_driver: None,
    peekkey: Some(
        peekkey_csi
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut ::core::ffi::c_void,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut size_t,
            ) -> TermKeyResult,
    ),
});
static drivers: GlobalCell<[*mut TermKeyDriver; 3]> = GlobalCell::new([
    (termkey_driver_ti.as_raw() as *const _) as *mut TermKeyDriver,
    (termkey_driver_csi.as_raw() as *const _) as *mut TermKeyDriver,
    ::core::ptr::null_mut::<TermKeyDriver>(),
]);
static keynames: GlobalCell<[C2Rust_Unnamed_6; 61]> = GlobalCell::new([
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_NONE,
        name: b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_BACKSPACE,
        name: b"Backspace\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_TAB,
        name: b"Tab\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_ENTER,
        name: b"Enter\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_ESCAPE,
        name: b"Escape\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_SPACE,
        name: b"Space\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_DEL,
        name: b"DEL\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_UP,
        name: b"Up\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_DOWN,
        name: b"Down\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_LEFT,
        name: b"Left\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_RIGHT,
        name: b"Right\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_BEGIN,
        name: b"Begin\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_FIND,
        name: b"Find\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_INSERT,
        name: b"Insert\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_DELETE,
        name: b"Delete\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_SELECT,
        name: b"Select\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_PAGEUP,
        name: b"PageUp\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_PAGEDOWN,
        name: b"PageDown\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_HOME,
        name: b"Home\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_END,
        name: b"End\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_CANCEL,
        name: b"Cancel\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_CLEAR,
        name: b"Clear\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_CLOSE,
        name: b"Close\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_COMMAND,
        name: b"Command\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_COPY,
        name: b"Copy\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_EXIT,
        name: b"Exit\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_HELP,
        name: b"Help\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_MARK,
        name: b"Mark\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_MESSAGE,
        name: b"Message\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_MOVE,
        name: b"Move\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_OPEN,
        name: b"Open\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_OPTIONS,
        name: b"Options\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_PRINT,
        name: b"Print\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_REDO,
        name: b"Redo\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_REFERENCE,
        name: b"Reference\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_REFRESH,
        name: b"Refresh\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_REPLACE,
        name: b"Replace\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_RESTART,
        name: b"Restart\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_RESUME,
        name: b"Resume\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_SAVE,
        name: b"Save\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_SUSPEND,
        name: b"Suspend\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_UNDO,
        name: b"Undo\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP0,
        name: b"KP0\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP1,
        name: b"KP1\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP2,
        name: b"KP2\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP3,
        name: b"KP3\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP4,
        name: b"KP4\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP5,
        name: b"KP5\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP6,
        name: b"KP6\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP7,
        name: b"KP7\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP8,
        name: b"KP8\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KP9,
        name: b"KP9\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPENTER,
        name: b"KPEnter\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPPLUS,
        name: b"KPPlus\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPMINUS,
        name: b"KPMinus\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPMULT,
        name: b"KPMult\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPDIV,
        name: b"KPDiv\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPCOMMA,
        name: b"KPComma\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPPERIOD,
        name: b"KPPeriod\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_KPEQUALS,
        name: b"KPEquals\0".as_ptr() as *const ::core::ffi::c_char,
    },
    C2Rust_Unnamed_6 {
        sym: TERMKEY_SYM_NONE,
        name: ::core::ptr::null::<::core::ffi::c_char>(),
    },
]);
static evnames: GlobalCell<[*const ::core::ffi::c_char; 4]> = GlobalCell::new([
    b"Unknown\0".as_ptr() as *const ::core::ffi::c_char,
    b"Press\0".as_ptr() as *const ::core::ffi::c_char,
    b"Drag\0".as_ptr() as *const ::core::ffi::c_char,
    b"Release\0".as_ptr() as *const ::core::ffi::c_char,
]);
#[no_mangle]
pub unsafe extern "C" fn termkey_interpret_string(
    mut tk: *mut TermKey,
    mut key: *const TermKeyKey,
    mut strp: *mut *const ::core::ffi::c_char,
) -> TermKeyResult {
    let mut p: *mut TermKeyDriverNode = ::core::ptr::null_mut::<TermKeyDriverNode>();
    p = (*tk).drivers;
    while !p.is_null() {
        if (*p).driver == termkey_driver_csi.ptr() {
            break;
        }
        p = (*p).next;
    }
    if p.is_null() {
        return TERMKEY_RES_NONE;
    }
    if (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_DCS as ::core::ffi::c_int
        && (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_OSC as ::core::ffi::c_int
        && (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_APC as ::core::ffi::c_int
    {
        return TERMKEY_RES_NONE;
    }
    let mut csi: *mut TermKeyCsi = (*p).info as *mut TermKeyCsi;
    if (*csi).saved_string_id != (*key).code.number {
        return TERMKEY_RES_NONE;
    }
    *strp = (*csi).saved_string;
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn snprint_cameltospaces(
    mut str: *mut ::core::ffi::c_char,
    mut size: size_t,
    mut src: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut prev_lower: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut l: size_t = 0 as size_t;
    while *src as ::core::ffi::c_int != 0 && l < size.wrapping_sub(1 as size_t) {
        if *(*__ctype_b_loc()).offset(*src as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
            && prev_lower != 0
        {
            if !str.is_null() {
                let c2rust_fresh0 = l;
                l = l.wrapping_add(1);
                *str.offset(c2rust_fresh0 as isize) = ' ' as ::core::ffi::c_char;
            }
            if l >= size.wrapping_sub(1 as size_t) {
                break;
            }
        }
        prev_lower = *(*__ctype_b_loc()).offset(*src as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
        let c2rust_fresh1 = src;
        src = src.offset(1);
        let c2rust_fresh2 = l;
        l = l.wrapping_add(1);
        *str.offset(c2rust_fresh2 as isize) =
            tolower(*c2rust_fresh1 as ::core::ffi::c_int) as ::core::ffi::c_char;
    }
    *str.offset(l as isize) = 0 as ::core::ffi::c_char;
    while *src != 0 {
        if *(*__ctype_b_loc()).offset(*src as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
            && prev_lower != 0
        {
            l = l.wrapping_add(1);
        }
        prev_lower = *(*__ctype_b_loc()).offset(*src as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
        src = src.offset(1);
        l = l.wrapping_add(1);
    }
    return l as ::core::ffi::c_int;
}
unsafe extern "C" fn strpncmp_camel(
    mut strp: *mut *const ::core::ffi::c_char,
    mut strcamelp: *mut *const ::core::ffi::c_char,
    mut n: size_t,
) -> ::core::ffi::c_int {
    let mut str: *const ::core::ffi::c_char = *strp;
    let mut strcamel: *const ::core::ffi::c_char = *strcamelp;
    let mut prev_lower: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (*str as ::core::ffi::c_int != 0 || *strcamel as ::core::ffi::c_int != 0) && n != 0 {
        let mut b: ::core::ffi::c_char =
            tolower(*strcamel as ::core::ffi::c_int) as ::core::ffi::c_char;
        if *(*__ctype_b_loc()).offset(*strcamel as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISupper as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
            && prev_lower != 0
        {
            if *str as ::core::ffi::c_int != ' ' as ::core::ffi::c_int {
                break;
            }
            str = str.offset(1);
            if *str as ::core::ffi::c_int != b as ::core::ffi::c_int {
                break;
            }
        } else if *str as ::core::ffi::c_int != b as ::core::ffi::c_int {
            break;
        }
        prev_lower = *(*__ctype_b_loc()).offset(*strcamel as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISlower as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int;
        str = str.offset(1);
        strcamel = strcamel.offset(1);
        n = n.wrapping_sub(1);
    }
    *strp = str;
    *strcamelp = strcamel;
    return *str as ::core::ffi::c_int - *strcamel as ::core::ffi::c_int;
}
unsafe extern "C" fn termkey_alloc() -> *mut TermKey {
    let mut tk: *mut TermKey = xmalloc(::core::mem::size_of::<TermKey>()) as *mut TermKey;
    (*tk).fd = -1 as ::core::ffi::c_int;
    (*tk).flags = 0 as ::core::ffi::c_int;
    (*tk).canonflags = 0 as ::core::ffi::c_int;
    (*tk).buffer = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    (*tk).buffstart = 0 as size_t;
    (*tk).buffcount = 0 as size_t;
    (*tk).buffsize = 256 as size_t;
    (*tk).hightide = 0 as size_t;
    (*tk).restore_termios_valid = 0 as ::core::ffi::c_char;
    (*tk).ti_getstr_hook = None;
    (*tk).ti_getstr_hook_data = NULL;
    (*tk).waittime = 50 as ::core::ffi::c_int;
    (*tk).is_closed = 0 as ::core::ffi::c_char;
    (*tk).is_started = 0 as ::core::ffi::c_char;
    (*tk).nkeynames = 64 as ::core::ffi::c_int;
    (*tk).keynames = ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 32 as ::core::ffi::c_int {
        (*tk).c0[i as usize].sym = TERMKEY_SYM_NONE;
        i += 1;
    }
    (*tk).drivers = ::core::ptr::null_mut::<TermKeyDriverNode>();
    (*tk).method.emit_codepoint = Some(
        emit_codepoint
            as unsafe extern "C" fn(*mut TermKey, ::core::ffi::c_int, *mut TermKeyKey) -> (),
    )
        as Option<unsafe extern "C" fn(*mut TermKey, ::core::ffi::c_int, *mut TermKeyKey) -> ()>;
    (*tk).method.peekkey_simple = Some(
        peekkey_simple
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut size_t,
            ) -> TermKeyResult,
    )
        as Option<
            unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut size_t,
            ) -> TermKeyResult,
        >;
    (*tk).method.peekkey_mouse = Some(
        peekkey_mouse
            as unsafe extern "C" fn(*mut TermKey, *mut TermKeyKey, *mut size_t) -> TermKeyResult,
    )
        as Option<
            unsafe extern "C" fn(*mut TermKey, *mut TermKeyKey, *mut size_t) -> TermKeyResult,
        >;
    return tk;
}
unsafe extern "C" fn termkey_init(
    mut tk: *mut TermKey,
    mut term: *mut TerminfoEntry,
) -> ::core::ffi::c_int {
    let mut tail: *mut TermKeyDriverNode = ::core::ptr::null_mut::<TermKeyDriverNode>();
    (*tk).buffer = xmalloc((*tk).buffsize) as *mut ::core::ffi::c_uchar;
    (*tk).keynames = xmalloc(
        ::core::mem::size_of::<*const ::core::ffi::c_char>()
            .wrapping_mul((*tk).nkeynames as size_t),
    ) as *mut *const ::core::ffi::c_char;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < (*tk).nkeynames {
        *(*tk).keynames.offset(i as isize) = ::core::ptr::null::<::core::ffi::c_char>();
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    '_abort_free_keynames: {
        while !(*keynames.ptr())[i as usize].name.is_null() {
            if termkey_register_keyname(
                tk,
                (*keynames.ptr())[i as usize].sym,
                (*keynames.ptr())[i as usize].name,
            ) as ::core::ffi::c_int
                == -1 as ::core::ffi::c_int
            {
                break '_abort_free_keynames;
            }
            i += 1;
        }
        register_c0(
            tk,
            TERMKEY_SYM_TAB,
            0x9 as ::core::ffi::c_uchar,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        register_c0(
            tk,
            TERMKEY_SYM_ENTER,
            0xd as ::core::ffi::c_uchar,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        register_c0(
            tk,
            TERMKEY_SYM_ESCAPE,
            0x1b as ::core::ffi::c_uchar,
            ::core::ptr::null::<::core::ffi::c_char>(),
        );
        tail = ::core::ptr::null_mut::<TermKeyDriverNode>();
        i = 0 as ::core::ffi::c_int;
        '_abort_free_drivers: {
            while !(*drivers.ptr())[i as usize].is_null() {
                let mut info: *mut ::core::ffi::c_void =
                    Some(
                        (**(drivers.ptr() as *mut *mut TermKeyDriver).offset(i as isize))
                            .new_driver
                            .expect("non-null function pointer"),
                    )
                    .expect("non-null function pointer")(tk, term);
                if !info.is_null() {
                    let mut thisdrv: *mut TermKeyDriverNode =
                        xmalloc(::core::mem::size_of::<TermKeyDriverNode>())
                            as *mut TermKeyDriverNode;
                    if thisdrv.is_null() {
                        break '_abort_free_drivers;
                    }
                    (*thisdrv).driver = (*drivers.ptr())[i as usize];
                    (*thisdrv).info = info;
                    (*thisdrv).next = ::core::ptr::null_mut::<TermKeyDriverNode>();
                    if tail.is_null() {
                        (*tk).drivers = thisdrv;
                    } else {
                        (*tail).next = thisdrv;
                    }
                    tail = thisdrv;
                }
                i += 1;
            }
            if (*tk).drivers.is_null() {
                *__errno_location() = ENOENT;
                break '_abort_free_keynames;
            } else {
                return 1 as ::core::ffi::c_int;
            }
        }
        let mut p: *mut TermKeyDriverNode = (*tk).drivers;
        while !p.is_null() {
            Some(
                (*(*p).driver)
                    .free_driver
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")((*p).info);
            let mut next: *mut TermKeyDriverNode = (*p).next;
            xfree(p as *mut ::core::ffi::c_void);
            p = next;
        }
    }
    xfree((*tk).keynames as *mut ::core::ffi::c_void);
    xfree((*tk).buffer as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_new_abstract(
    mut term: *mut TerminfoEntry,
    mut flags: ::core::ffi::c_int,
) -> *mut TermKey {
    let mut tk: *mut TermKey = termkey_alloc();
    if tk.is_null() {
        return ::core::ptr::null_mut::<TermKey>();
    }
    (*tk).fd = -1 as ::core::ffi::c_int;
    termkey_set_flags(tk, flags);
    if termkey_init(tk, term) == 0 {
        xfree(tk as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<TermKey>();
    }
    if flags & TERMKEY_FLAG_NOSTART as ::core::ffi::c_int == 0 && termkey_start(tk) == 0 {
        xfree(tk as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<TermKey>();
    } else {
        return tk;
    };
}
pub unsafe extern "C" fn termkey_free(mut tk: *mut TermKey) {
    xfree((*tk).buffer as *mut ::core::ffi::c_void);
    (*tk).buffer = ::core::ptr::null_mut::<::core::ffi::c_uchar>();
    xfree((*tk).keynames as *mut ::core::ffi::c_void);
    (*tk).keynames = ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    let mut p: *mut TermKeyDriverNode = ::core::ptr::null_mut::<TermKeyDriverNode>();
    p = (*tk).drivers;
    while !p.is_null() {
        Some(
            (*(*p).driver)
                .free_driver
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")((*p).info);
        let mut next: *mut TermKeyDriverNode = (*p).next;
        xfree(p as *mut ::core::ffi::c_void);
        p = next;
    }
    xfree(tk as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn termkey_destroy(mut tk: *mut TermKey) {
    if (*tk).is_started != 0 {
        termkey_stop(tk);
    }
    termkey_free(tk);
}
pub unsafe extern "C" fn termkey_hook_terminfo_getstr(
    mut tk: *mut TermKey,
    mut hookfn: Option<TermKey_Terminfo_Getstr_Hook>,
    mut data: *mut ::core::ffi::c_void,
) {
    (*tk).ti_getstr_hook = hookfn;
    (*tk).ti_getstr_hook_data = data;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_start(mut tk: *mut TermKey) -> ::core::ffi::c_int {
    if (*tk).is_started != 0 {
        return 1 as ::core::ffi::c_int;
    }
    if (*tk).fd != -1 as ::core::ffi::c_int
        && (*tk).flags & TERMKEY_FLAG_NOTERMIOS as ::core::ffi::c_int == 0
    {
        let mut termios: termios = termios {
            c_iflag: 0,
            c_oflag: 0,
            c_cflag: 0,
            c_lflag: 0,
            c_line: 0,
            c_cc: [0; 32],
            c_ispeed: 0,
            c_ospeed: 0,
        };
        if tcgetattr((*tk).fd, &raw mut termios) == 0 as ::core::ffi::c_int {
            (*tk).restore_termios = termios;
            (*tk).restore_termios_valid = 1 as ::core::ffi::c_char;
            termios.c_iflag &= !(IXON | INLCR | ICRNL) as tcflag_t;
            termios.c_lflag &= !(ICANON | ECHO | IEXTEN) as tcflag_t;
            termios.c_cc[VMIN as usize] = 1 as cc_t;
            termios.c_cc[VTIME as usize] = 0 as cc_t;
            if (*tk).flags & TERMKEY_FLAG_CTRLC as ::core::ffi::c_int != 0 {
                termios.c_lflag &= !ISIG as tcflag_t;
            } else {
                termios.c_cc[VQUIT as usize] = _POSIX_VDISABLE as cc_t;
                termios.c_cc[VSUSP as usize] = _POSIX_VDISABLE as cc_t;
            }
            tcsetattr((*tk).fd, TCSANOW, &raw mut termios);
        }
    }
    let mut p: *mut TermKeyDriverNode = ::core::ptr::null_mut::<TermKeyDriverNode>();
    p = (*tk).drivers;
    while !p.is_null() {
        if (*(*p).driver).start_driver.is_some() {
            if Some(
                (*(*p).driver)
                    .start_driver
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(tk, (*p).info)
                == 0
            {
                return 0 as ::core::ffi::c_int;
            }
        }
        p = (*p).next;
    }
    (*tk).is_started = 1 as ::core::ffi::c_char;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_stop(mut tk: *mut TermKey) -> ::core::ffi::c_int {
    if (*tk).is_started == 0 {
        return 1 as ::core::ffi::c_int;
    }
    let mut p: *mut TermKeyDriverNode = ::core::ptr::null_mut::<TermKeyDriverNode>();
    p = (*tk).drivers;
    while !p.is_null() {
        if (*(*p).driver).stop_driver.is_some() {
            Some(
                (*(*p).driver)
                    .stop_driver
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(tk, (*p).info);
        }
        p = (*p).next;
    }
    if (*tk).restore_termios_valid != 0 {
        tcsetattr((*tk).fd, TCSANOW, &raw mut (*tk).restore_termios);
    }
    (*tk).is_started = 0 as ::core::ffi::c_char;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_set_flags(mut tk: *mut TermKey, mut newflags: ::core::ffi::c_int) {
    (*tk).flags = newflags;
    if (*tk).flags & TERMKEY_FLAG_SPACESYMBOL as ::core::ffi::c_int != 0 {
        (*tk).canonflags |= TERMKEY_CANON_SPACESYMBOL as ::core::ffi::c_int;
    } else {
        (*tk).canonflags &= !(TERMKEY_CANON_SPACESYMBOL as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn termkey_get_canonflags(mut tk: *mut TermKey) -> ::core::ffi::c_int {
    return (*tk).canonflags;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_set_canonflags(
    mut tk: *mut TermKey,
    mut flags: ::core::ffi::c_int,
) {
    (*tk).canonflags = flags;
    if (*tk).canonflags & TERMKEY_CANON_SPACESYMBOL as ::core::ffi::c_int != 0 {
        (*tk).flags |= TERMKEY_FLAG_SPACESYMBOL as ::core::ffi::c_int;
    } else {
        (*tk).flags &= !(TERMKEY_FLAG_SPACESYMBOL as ::core::ffi::c_int);
    };
}
#[no_mangle]
pub unsafe extern "C" fn termkey_get_buffer_size(mut tk: *mut TermKey) -> size_t {
    return (*tk).buffsize;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_set_buffer_size(
    mut tk: *mut TermKey,
    mut size: size_t,
) -> ::core::ffi::c_int {
    let mut buffer: *mut ::core::ffi::c_uchar =
        xrealloc((*tk).buffer as *mut ::core::ffi::c_void, size) as *mut ::core::ffi::c_uchar;
    (*tk).buffer = buffer;
    (*tk).buffsize = size;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_get_buffer_remaining(mut tk: *mut TermKey) -> size_t {
    return (*tk).buffsize.wrapping_sub((*tk).buffcount);
}
unsafe extern "C" fn eat_bytes(mut tk: *mut TermKey, mut count: size_t) {
    if count >= (*tk).buffcount {
        (*tk).buffstart = 0 as size_t;
        (*tk).buffcount = 0 as size_t;
        return;
    }
    (*tk).buffstart = (*tk).buffstart.wrapping_add(count);
    (*tk).buffcount = (*tk).buffcount.wrapping_sub(count);
}
pub unsafe extern "C" fn fill_utf8(
    mut codepoint: ::core::ffi::c_int,
    mut str: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut nbytes: ::core::ffi::c_int = utf_char2bytes(codepoint, str);
    *str.offset(nbytes as isize) = 0 as ::core::ffi::c_char;
    return nbytes;
}
unsafe extern "C" fn parse_utf8(
    mut bytes: *const ::core::ffi::c_uchar,
    mut len: size_t,
    mut cp: *mut ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    let mut nbytes: ::core::ffi::c_uint = 0;
    let mut b0: ::core::ffi::c_uchar = *bytes.offset(0 as ::core::ffi::c_int as isize);
    if (b0 as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
        *cp = b0 as ::core::ffi::c_int;
        *nbytep = 1 as size_t;
        return TERMKEY_RES_KEY;
    } else if (b0 as ::core::ffi::c_int) < 0xc0 as ::core::ffi::c_int {
        *cp = UNICODE_INVALID as ::core::ffi::c_int;
        *nbytep = 1 as size_t;
        return TERMKEY_RES_KEY;
    } else if (b0 as ::core::ffi::c_int) < 0xe0 as ::core::ffi::c_int {
        nbytes = 2 as ::core::ffi::c_uint;
        *cp = b0 as ::core::ffi::c_int & 0x1f as ::core::ffi::c_int;
    } else if (b0 as ::core::ffi::c_int) < 0xf0 as ::core::ffi::c_int {
        nbytes = 3 as ::core::ffi::c_uint;
        *cp = b0 as ::core::ffi::c_int & 0xf as ::core::ffi::c_int;
    } else if (b0 as ::core::ffi::c_int) < 0xf8 as ::core::ffi::c_int {
        nbytes = 4 as ::core::ffi::c_uint;
        *cp = b0 as ::core::ffi::c_int & 0x7 as ::core::ffi::c_int;
    } else if (b0 as ::core::ffi::c_int) < 0xfc as ::core::ffi::c_int {
        nbytes = 5 as ::core::ffi::c_uint;
        *cp = b0 as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int;
    } else if (b0 as ::core::ffi::c_int) < 0xfe as ::core::ffi::c_int {
        nbytes = 6 as ::core::ffi::c_uint;
        *cp = b0 as ::core::ffi::c_int & 0x1 as ::core::ffi::c_int;
    } else {
        *cp = UNICODE_INVALID as ::core::ffi::c_int;
        *nbytep = 1 as size_t;
        return TERMKEY_RES_KEY;
    }
    let mut b: ::core::ffi::c_uint = 1 as ::core::ffi::c_uint;
    while b < nbytes {
        let mut cb: ::core::ffi::c_uchar = 0;
        if b as size_t >= len {
            return TERMKEY_RES_AGAIN;
        }
        cb = *bytes.offset(b as isize);
        if (cb as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int
            || cb as ::core::ffi::c_int >= 0xc0 as ::core::ffi::c_int
        {
            *cp = UNICODE_INVALID as ::core::ffi::c_int;
            *nbytep = b as size_t;
            return TERMKEY_RES_KEY;
        }
        *cp <<= 6 as ::core::ffi::c_int;
        *cp |= cb as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int;
        b = b.wrapping_add(1);
    }
    if nbytes as ::core::ffi::c_int > utf_char2len(*cp) {
        *cp = UNICODE_INVALID as ::core::ffi::c_int;
    }
    if *cp >= 0xd800 as ::core::ffi::c_int && *cp <= 0xdfff as ::core::ffi::c_int
        || *cp == 0xfffe as ::core::ffi::c_int
        || *cp == 0xffff as ::core::ffi::c_int
    {
        *cp = UNICODE_INVALID as ::core::ffi::c_int;
    }
    *nbytep = nbytes as size_t;
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn emit_codepoint(
    mut tk: *mut TermKey,
    mut codepoint: ::core::ffi::c_int,
    mut key: *mut TermKeyKey,
) {
    if codepoint == 0 as ::core::ffi::c_int {
        (*key).type_0 = TERMKEY_TYPE_KEYSYM;
        (*key).code.sym = TERMKEY_SYM_SPACE;
        (*key).modifiers = TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int;
    } else if codepoint < 0x20 as ::core::ffi::c_int
        && (*tk).flags & TERMKEY_FLAG_KEEPC0 as ::core::ffi::c_int == 0
    {
        (*key).code.codepoint = 0 as ::core::ffi::c_int;
        (*key).modifiers = 0 as ::core::ffi::c_int;
        if (*tk).flags & TERMKEY_FLAG_NOINTERPRET as ::core::ffi::c_int == 0
            && (*tk).c0[codepoint as usize].sym as ::core::ffi::c_int
                != TERMKEY_SYM_UNKNOWN as ::core::ffi::c_int
        {
            (*key).code.sym = (*tk).c0[codepoint as usize].sym;
            (*key).modifiers |= (*tk).c0[codepoint as usize].modifier_set;
        }
        if (*key).code.sym as u64 == 0 {
            (*key).type_0 = TERMKEY_TYPE_UNICODE;
            if codepoint + 0x40 as ::core::ffi::c_int >= 'A' as ::core::ffi::c_int
                && codepoint + 0x40 as ::core::ffi::c_int <= 'Z' as ::core::ffi::c_int
            {
                (*key).code.codepoint = codepoint + 0x60 as ::core::ffi::c_int;
            } else {
                (*key).code.codepoint = codepoint + 0x40 as ::core::ffi::c_int;
            }
            (*key).modifiers = TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int;
        } else {
            (*key).type_0 = TERMKEY_TYPE_KEYSYM;
        }
    } else if codepoint == 0x7f as ::core::ffi::c_int
        && (*tk).flags & TERMKEY_FLAG_NOINTERPRET as ::core::ffi::c_int == 0
    {
        (*key).type_0 = TERMKEY_TYPE_KEYSYM;
        (*key).code.sym = TERMKEY_SYM_DEL;
        (*key).modifiers = 0 as ::core::ffi::c_int;
    } else if codepoint > 0 as ::core::ffi::c_int && codepoint < 0x80 as ::core::ffi::c_int {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code.codepoint = codepoint;
        (*key).modifiers = 0 as ::core::ffi::c_int;
    } else if codepoint >= 0x80 as ::core::ffi::c_int && codepoint < 0xa0 as ::core::ffi::c_int {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code.codepoint = codepoint - 0x40 as ::core::ffi::c_int;
        (*key).modifiers =
            TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int | TERMKEY_KEYMOD_ALT as ::core::ffi::c_int;
    } else {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code.codepoint = codepoint;
        (*key).modifiers = 0 as ::core::ffi::c_int;
    }
    termkey_canonicalise(tk, key);
    if (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_UNICODE as ::core::ffi::c_int {
        fill_utf8(
            (*key).code.codepoint,
            &raw mut (*key).utf8 as *mut ::core::ffi::c_char,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn termkey_canonicalise(mut tk: *mut TermKey, mut key: *mut TermKeyKey) {
    let mut flags: ::core::ffi::c_int = (*tk).canonflags;
    if flags & TERMKEY_CANON_SPACESYMBOL as ::core::ffi::c_int != 0 {
        if (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_UNICODE as ::core::ffi::c_int
            && (*key).code.codepoint == 0x20 as ::core::ffi::c_int
        {
            (*key).type_0 = TERMKEY_TYPE_KEYSYM;
            (*key).code.sym = TERMKEY_SYM_SPACE;
        }
    } else if (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_KEYSYM as ::core::ffi::c_int
        && (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_SPACE as ::core::ffi::c_int
    {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code.codepoint = 0x20 as ::core::ffi::c_int;
        fill_utf8(
            (*key).code.codepoint,
            &raw mut (*key).utf8 as *mut ::core::ffi::c_char,
        );
    }
    if flags & TERMKEY_CANON_DELBS as ::core::ffi::c_int != 0 {
        if (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_KEYSYM as ::core::ffi::c_int
            && (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_DEL as ::core::ffi::c_int
        {
            (*key).code.sym = TERMKEY_SYM_BACKSPACE;
        }
    }
}
unsafe extern "C" fn peekkey(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut force: ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    let mut again: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*tk).is_started == 0 {
        *__errno_location() = EINVAL;
        return TERMKEY_RES_ERROR;
    }
    (*key).event = TERMKEY_EVENT_PRESS;
    if (*tk).hightide != 0 {
        (*tk).buffstart = (*tk).buffstart.wrapping_add((*tk).hightide);
        (*tk).buffcount = (*tk).buffcount.wrapping_sub((*tk).hightide);
        (*tk).hightide = 0 as size_t;
    }
    let mut ret: TermKeyResult = TERMKEY_RES_NONE;
    let mut p: *mut TermKeyDriverNode = ::core::ptr::null_mut::<TermKeyDriverNode>();
    p = (*tk).drivers;
    while !p.is_null() {
        ret = (*(*p).driver).peekkey.expect("non-null function pointer")(
            tk,
            (*p).info,
            key,
            force,
            nbytep,
        );
        's_115: {
            match ret as ::core::ffi::c_uint {
                1 => {
                    let mut halfsize: size_t = (*tk).buffsize.wrapping_div(2 as size_t);
                    if (*tk).buffstart > halfsize {
                        memcpy(
                            (*tk).buffer as *mut ::core::ffi::c_void,
                            (*tk).buffer.offset(halfsize as isize) as *const ::core::ffi::c_void,
                            halfsize,
                        );
                        (*tk).buffstart = (*tk).buffstart.wrapping_sub(halfsize);
                    }
                }
                2 | 4 => {}
                3 => {
                    if force == 0 {
                        again = 1 as ::core::ffi::c_int;
                    }
                    break 's_115;
                }
                0 | _ => {
                    break 's_115;
                }
            }
            return ret;
        }
        p = (*p).next;
    }
    if again != 0 {
        return TERMKEY_RES_AGAIN;
    }
    ret = peekkey_simple(tk, key, force, nbytep);
    return ret;
}
unsafe extern "C" fn peekkey_simple(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut force: ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).buffcount == 0 as size_t {
        return (if (*tk).is_closed as ::core::ffi::c_int != 0 {
            TERMKEY_RES_EOF as ::core::ffi::c_int
        } else {
            TERMKEY_RES_NONE as ::core::ffi::c_int
        }) as TermKeyResult;
    }
    let mut b0: ::core::ffi::c_uchar = *(*tk)
        .buffer
        .offset((*tk).buffstart.wrapping_add(0 as size_t) as isize);
    if b0 as ::core::ffi::c_int == 0x1b as ::core::ffi::c_int {
        if (*tk).buffcount == 1 as size_t {
            if force == 0 {
                return TERMKEY_RES_AGAIN;
            }
            Some(
                (*tk)
                    .method
                    .emit_codepoint
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(tk, b0 as ::core::ffi::c_int, key);
            *nbytep = 1 as size_t;
            return TERMKEY_RES_KEY;
        }
        (*tk).buffstart = (*tk).buffstart.wrapping_add(1);
        (*tk).buffcount = (*tk).buffcount.wrapping_sub(1);
        let mut metakey_result: TermKeyResult = peekkey(tk, key, force, nbytep);
        (*tk).buffstart = (*tk).buffstart.wrapping_sub(1);
        (*tk).buffcount = (*tk).buffcount.wrapping_add(1);
        match metakey_result as ::core::ffi::c_uint {
            1 => {
                (*key).modifiers |= TERMKEY_KEYMOD_ALT as ::core::ffi::c_int;
                *nbytep = (*nbytep).wrapping_add(1);
            }
            0 | 2 | 3 | 4 | _ => {}
        }
        return metakey_result;
    } else if (b0 as ::core::ffi::c_int) < 0xa0 as ::core::ffi::c_int {
        Some(
            (*tk)
                .method
                .emit_codepoint
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(tk, b0 as ::core::ffi::c_int, key);
        *nbytep = 1 as size_t;
        return TERMKEY_RES_KEY;
    } else if (*tk).flags & TERMKEY_FLAG_UTF8 as ::core::ffi::c_int != 0 {
        let mut codepoint: ::core::ffi::c_int = 0;
        let mut res: TermKeyResult = parse_utf8(
            (*tk).buffer.offset((*tk).buffstart as isize),
            (*tk).buffcount,
            &raw mut codepoint,
            nbytep,
        );
        if res as ::core::ffi::c_uint
            == TERMKEY_RES_AGAIN as ::core::ffi::c_int as ::core::ffi::c_uint
            && force != 0
        {
            codepoint = UNICODE_INVALID as ::core::ffi::c_int;
            *nbytep = (*tk).buffcount;
            res = TERMKEY_RES_KEY;
        }
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).modifiers = 0 as ::core::ffi::c_int;
        Some(
            (*tk)
                .method
                .emit_codepoint
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(tk, codepoint, key);
        return res;
    } else {
        (*key).type_0 = TERMKEY_TYPE_UNICODE;
        (*key).code.codepoint = b0 as ::core::ffi::c_int;
        (*key).modifiers = 0 as ::core::ffi::c_int;
        (*key).utf8[0 as ::core::ffi::c_int as usize] =
            (*key).code.codepoint as ::core::ffi::c_char;
        (*key).utf8[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        *nbytep = 1 as size_t;
        return TERMKEY_RES_KEY;
    };
}
unsafe extern "C" fn peekkey_mouse(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).buffcount < 3 as size_t {
        return TERMKEY_RES_AGAIN;
    }
    (*key).type_0 = TERMKEY_TYPE_MOUSE;
    (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
        (*(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(0 as size_t) as isize)
            as ::core::ffi::c_char as ::core::ffi::c_int
            - 0x20 as ::core::ffi::c_int) as ::core::ffi::c_char;
    (*key).code.mouse[1 as ::core::ffi::c_int as usize] =
        (*(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(1 as size_t) as isize)
            as ::core::ffi::c_char as ::core::ffi::c_int
            - 0x20 as ::core::ffi::c_int) as ::core::ffi::c_char;
    (*key).code.mouse[2 as ::core::ffi::c_int as usize] =
        (*(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(2 as size_t) as isize)
            as ::core::ffi::c_char as ::core::ffi::c_int
            - 0x20 as ::core::ffi::c_int) as ::core::ffi::c_char;
    (*key).code.mouse[3 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
    (*key).modifiers = ((*key).code.mouse[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        & 0x1c as ::core::ffi::c_int)
        >> 2 as ::core::ffi::c_int;
    (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
        ((*key).code.mouse[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            & !(0x1c as ::core::ffi::c_int)) as ::core::ffi::c_char;
    *nbytep = 3 as size_t;
    return TERMKEY_RES_KEY;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_getkey(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
) -> TermKeyResult {
    let mut nbytes: size_t = 0 as size_t;
    let mut ret: TermKeyResult = peekkey(tk, key, 0 as ::core::ffi::c_int, &raw mut nbytes);
    if ret as ::core::ffi::c_uint == TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint {
        eat_bytes(tk, nbytes);
    }
    if ret as ::core::ffi::c_uint == TERMKEY_RES_AGAIN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        peekkey(tk, key, 1 as ::core::ffi::c_int, &raw mut nbytes);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_getkey_force(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
) -> TermKeyResult {
    let mut nbytes: size_t = 0 as size_t;
    let mut ret: TermKeyResult = peekkey(tk, key, 1 as ::core::ffi::c_int, &raw mut nbytes);
    if ret as ::core::ffi::c_uint == TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint {
        eat_bytes(tk, nbytes);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_push_bytes(
    mut tk: *mut TermKey,
    mut bytes: *const ::core::ffi::c_char,
    mut len: size_t,
) -> size_t {
    if (*tk).buffstart != 0 {
        memmove(
            (*tk).buffer as *mut ::core::ffi::c_void,
            (*tk).buffer.offset((*tk).buffstart as isize) as *const ::core::ffi::c_void,
            (*tk).buffcount,
        );
        (*tk).buffstart = 0 as size_t;
    }
    if (*tk).buffcount >= (*tk).buffsize {
        *__errno_location() = ENOMEM;
        return -1 as ::core::ffi::c_int as size_t;
    }
    if len > (*tk).buffsize.wrapping_sub((*tk).buffcount) {
        len = (*tk).buffsize.wrapping_sub((*tk).buffcount);
    }
    memcpy(
        (*tk).buffer.offset((*tk).buffcount as isize) as *mut ::core::ffi::c_void,
        bytes as *const ::core::ffi::c_void,
        len,
    );
    (*tk).buffcount = (*tk).buffcount.wrapping_add(len);
    return len;
}
pub unsafe extern "C" fn termkey_register_keyname(
    mut tk: *mut TermKey,
    mut sym: TermKeySym,
    mut name: *const ::core::ffi::c_char,
) -> TermKeySym {
    if sym as u64 == 0 {
        sym = (*tk).nkeynames as TermKeySym;
    }
    if sym as ::core::ffi::c_int >= (*tk).nkeynames {
        let mut new_keynames: *mut *const ::core::ffi::c_char = xrealloc(
            (*tk).keynames as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<*const ::core::ffi::c_char>()
                .wrapping_mul((sym as size_t).wrapping_add(1 as size_t)),
        )
            as *mut *const ::core::ffi::c_char;
        (*tk).keynames = new_keynames;
        let mut i: ::core::ffi::c_int = (*tk).nkeynames;
        while i < sym as ::core::ffi::c_int {
            *(*tk).keynames.offset(i as isize) = ::core::ptr::null::<::core::ffi::c_char>();
            i += 1;
        }
        (*tk).nkeynames = sym as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
    }
    *(*tk).keynames.offset(sym as isize) = name;
    return sym;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_get_keyname(
    mut tk: *mut TermKey,
    mut sym: TermKeySym,
) -> *const ::core::ffi::c_char {
    if sym as ::core::ffi::c_int == TERMKEY_SYM_UNKNOWN as ::core::ffi::c_int {
        return b"UNKNOWN\0".as_ptr() as *const ::core::ffi::c_char;
    }
    if (sym as ::core::ffi::c_int) < (*tk).nkeynames {
        return *(*tk).keynames.offset(sym as isize);
    }
    return b"UNKNOWN\0".as_ptr() as *const ::core::ffi::c_char;
}
unsafe extern "C" fn termkey_lookup_keyname_format(
    mut tk: *mut TermKey,
    mut str: *const ::core::ffi::c_char,
    mut sym: *mut TermKeySym,
    mut format: TermKeyFormat,
) -> *const ::core::ffi::c_char {
    *sym = TERMKEY_SYM_NONE;
    while (*sym as ::core::ffi::c_int) < (*tk).nkeynames {
        let mut thiskey: *const ::core::ffi::c_char = *(*tk).keynames.offset(*sym as isize);
        if !thiskey.is_null() {
            let mut len: size_t = strlen(thiskey);
            if format as ::core::ffi::c_uint
                & TERMKEY_FORMAT_LOWERSPACE as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                let mut thisstr: *const ::core::ffi::c_char = str;
                if strpncmp_camel(&raw mut thisstr, &raw mut thiskey, len)
                    == 0 as ::core::ffi::c_int
                {
                    return thisstr;
                }
            } else if strncmp(str, thiskey, len) == 0 as ::core::ffi::c_int {
                return (str as *mut ::core::ffi::c_char).offset(len as isize);
            }
        }
        *sym += 1;
    }
    return ::core::ptr::null::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn termkey_lookup_keyname(
    mut tk: *mut TermKey,
    mut str: *const ::core::ffi::c_char,
    mut sym: *mut TermKeySym,
) -> *const ::core::ffi::c_char {
    return termkey_lookup_keyname_format(tk, str, sym, 0 as TermKeyFormat);
}
unsafe extern "C" fn register_c0(
    mut tk: *mut TermKey,
    mut sym: TermKeySym,
    mut ctrl: ::core::ffi::c_uchar,
    mut name: *const ::core::ffi::c_char,
) -> TermKeySym {
    return register_c0_full(
        tk,
        sym,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        ctrl,
        name,
    );
}
unsafe extern "C" fn register_c0_full(
    mut tk: *mut TermKey,
    mut sym: TermKeySym,
    mut modifier_set: ::core::ffi::c_int,
    mut modifier_mask: ::core::ffi::c_int,
    mut ctrl: ::core::ffi::c_uchar,
    mut name: *const ::core::ffi::c_char,
) -> TermKeySym {
    if ctrl as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int {
        *__errno_location() = EINVAL;
        return TERMKEY_SYM_UNKNOWN;
    }
    if !name.is_null() {
        sym = termkey_register_keyname(tk, sym, name);
    }
    (*tk).c0[ctrl as usize].sym = sym;
    (*tk).c0[ctrl as usize].modifier_set = modifier_set;
    (*tk).c0[ctrl as usize].modifier_mask = modifier_mask;
    return sym;
}
static modnames: GlobalCell<[modnames; 8]> = GlobalCell::new([
    modnames {
        shift: b"S\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"A\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"C\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"Shift\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"Alt\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"Ctrl\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"S\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"M\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"C\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"Shift\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"Meta\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"Ctrl\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"s\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"a\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"c\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"shift\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"alt\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"ctrl\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"s\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"m\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"c\0".as_ptr() as *const ::core::ffi::c_char,
    },
    modnames {
        shift: b"shift\0".as_ptr() as *const ::core::ffi::c_char,
        alt: b"meta\0".as_ptr() as *const ::core::ffi::c_char,
        ctrl: b"ctrl\0".as_ptr() as *const ::core::ffi::c_char,
    },
]);
#[no_mangle]
pub unsafe extern "C" fn termkey_strfkey(
    mut tk: *mut TermKey,
    mut buffer: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut key: *mut TermKeyKey,
    mut format: TermKeyFormat,
) -> size_t {
    let mut pos: size_t = 0 as size_t;
    let mut l: size_t = 0 as size_t;
    let mut mods: *mut modnames = (modnames.ptr() as *mut modnames).offset(
        ((format as ::core::ffi::c_uint
            & TERMKEY_FORMAT_LONGMOD as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0) as ::core::ffi::c_int
            + (format as ::core::ffi::c_uint
                & TERMKEY_FORMAT_ALTISMETA as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0) as ::core::ffi::c_int
                * 2 as ::core::ffi::c_int
            + (format as ::core::ffi::c_uint
                & TERMKEY_FORMAT_LOWERMOD as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0) as ::core::ffi::c_int
                * 4 as ::core::ffi::c_int) as isize,
    ) as *mut modnames;
    let mut wrapbracket: ::core::ffi::c_int = (format as ::core::ffi::c_uint
        & TERMKEY_FORMAT_WRAPBRACKET as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
        && ((*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_UNICODE as ::core::ffi::c_int
            || (*key).modifiers != 0 as ::core::ffi::c_int))
        as ::core::ffi::c_int;
    let mut sep: ::core::ffi::c_char = (if format as ::core::ffi::c_uint
        & TERMKEY_FORMAT_SPACEMOD as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        ' ' as ::core::ffi::c_int
    } else {
        '-' as ::core::ffi::c_int
    }) as ::core::ffi::c_char;
    if format as ::core::ffi::c_uint
        & TERMKEY_FORMAT_CARETCTRL as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
        && (*key).type_0 as ::core::ffi::c_int == TERMKEY_TYPE_UNICODE as ::core::ffi::c_int
        && (*key).modifiers == TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int
    {
        let mut codepoint: ::core::ffi::c_long = (*key).code.codepoint as ::core::ffi::c_long;
        if codepoint >= 'a' as ::core::ffi::c_long && codepoint <= 'z' as ::core::ffi::c_long {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                if wrapbracket != 0 {
                    b"<^%c>\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"^%c\0".as_ptr() as *const ::core::ffi::c_char
                },
                codepoint as ::core::ffi::c_char as ::core::ffi::c_int - 0x20 as ::core::ffi::c_int,
            ) as size_t;
            if l <= 0 as size_t {
                return pos;
            }
            pos = pos.wrapping_add(l);
            return pos;
        } else if codepoint >= '@' as ::core::ffi::c_long && codepoint < 'A' as ::core::ffi::c_long
            || codepoint > 'Z' as ::core::ffi::c_long && codepoint <= '_' as ::core::ffi::c_long
        {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                if wrapbracket != 0 {
                    b"<^%c>\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"^%c\0".as_ptr() as *const ::core::ffi::c_char
                },
                codepoint as ::core::ffi::c_char as ::core::ffi::c_int,
            ) as size_t;
            if l <= 0 as size_t {
                return pos;
            }
            pos = pos.wrapping_add(l);
            return pos;
        }
    }
    if wrapbracket != 0 {
        l = snprintf(
            buffer.offset(pos as isize),
            len.wrapping_sub(pos),
            b"<\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t;
        if l <= 0 as size_t {
            return pos;
        }
        pos = pos.wrapping_add(l);
    }
    if (*key).modifiers & TERMKEY_KEYMOD_ALT as ::core::ffi::c_int != 0 {
        l = snprintf(
            buffer.offset(pos as isize),
            len.wrapping_sub(pos),
            b"%s%c\0".as_ptr() as *const ::core::ffi::c_char,
            (*mods).alt,
            sep as ::core::ffi::c_int,
        ) as size_t;
        if l <= 0 as size_t {
            return pos;
        }
        pos = pos.wrapping_add(l);
    }
    if (*key).modifiers & TERMKEY_KEYMOD_CTRL as ::core::ffi::c_int != 0 {
        l = snprintf(
            buffer.offset(pos as isize),
            len.wrapping_sub(pos),
            b"%s%c\0".as_ptr() as *const ::core::ffi::c_char,
            (*mods).ctrl,
            sep as ::core::ffi::c_int,
        ) as size_t;
        if l <= 0 as size_t {
            return pos;
        }
        pos = pos.wrapping_add(l);
    }
    if (*key).modifiers & TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int != 0 {
        l = snprintf(
            buffer.offset(pos as isize),
            len.wrapping_sub(pos),
            b"%s%c\0".as_ptr() as *const ::core::ffi::c_char,
            (*mods).shift,
            sep as ::core::ffi::c_int,
        ) as size_t;
        if l <= 0 as size_t {
            return pos;
        }
        pos = pos.wrapping_add(l);
    }
    match (*key).type_0 as ::core::ffi::c_int {
        0 => {
            if (*key).utf8[0 as ::core::ffi::c_int as usize] == 0 {
                fill_utf8(
                    (*key).code.codepoint,
                    &raw mut (*key).utf8 as *mut ::core::ffi::c_char,
                );
            }
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut (*key).utf8 as *mut ::core::ffi::c_char,
            ) as size_t;
        }
        2 => {
            let mut name: *const ::core::ffi::c_char = termkey_get_keyname(tk, (*key).code.sym);
            if format as ::core::ffi::c_uint
                & TERMKEY_FORMAT_LOWERSPACE as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                l = snprint_cameltospaces(buffer.offset(pos as isize), len.wrapping_sub(pos), name)
                    as size_t;
            } else {
                l = snprintf(
                    buffer.offset(pos as isize),
                    len.wrapping_sub(pos),
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    name,
                ) as size_t;
            }
        }
        1 => {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"%c%d\0".as_ptr() as *const ::core::ffi::c_char,
                if format as ::core::ffi::c_uint
                    & TERMKEY_FORMAT_LOWERSPACE as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                {
                    'f' as ::core::ffi::c_int
                } else {
                    'F' as ::core::ffi::c_int
                },
                (*key).code.number,
            ) as size_t;
        }
        3 => {
            let mut ev: TermKeyMouseEvent = TERMKEY_MOUSE_UNKNOWN;
            let mut button: ::core::ffi::c_int = 0;
            let mut line: ::core::ffi::c_int = 0;
            let mut col: ::core::ffi::c_int = 0;
            termkey_interpret_mouse(
                tk,
                key,
                &raw mut ev,
                &raw mut button,
                &raw mut line,
                &raw mut col,
            );
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"Mouse%s(%d)\0".as_ptr() as *const ::core::ffi::c_char,
                (*evnames.ptr())[ev as usize],
                button,
            ) as size_t;
            if format as ::core::ffi::c_uint
                & TERMKEY_FORMAT_MOUSE_POS as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
            {
                if l <= 0 as size_t {
                    return pos;
                }
                pos = pos.wrapping_add(l);
                l = snprintf(
                    buffer.offset(pos as isize),
                    len.wrapping_sub(pos),
                    b" @ (%u,%u)\0".as_ptr() as *const ::core::ffi::c_char,
                    col,
                    line,
                ) as size_t;
            }
        }
        4 => {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"Position\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t;
        }
        5 => {
            let mut initial: ::core::ffi::c_int = 0;
            let mut mode: ::core::ffi::c_int = 0;
            let mut value: ::core::ffi::c_int = 0;
            termkey_interpret_modereport(tk, key, &raw mut initial, &raw mut mode, &raw mut value);
            if initial != 0 {
                l = snprintf(
                    buffer.offset(pos as isize),
                    len.wrapping_sub(pos),
                    b"Mode(%c%d=%d)\0".as_ptr() as *const ::core::ffi::c_char,
                    initial,
                    mode,
                    value,
                ) as size_t;
            } else {
                l = snprintf(
                    buffer.offset(pos as isize),
                    len.wrapping_sub(pos),
                    b"Mode(%d=%d)\0".as_ptr() as *const ::core::ffi::c_char,
                    mode,
                    value,
                ) as size_t;
            }
        }
        6 => {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"DCS\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t;
        }
        7 => {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"OSC\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t;
        }
        8 => {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"APC\0".as_ptr() as *const ::core::ffi::c_char,
            ) as size_t;
        }
        -1 => {
            l = snprintf(
                buffer.offset(pos as isize),
                len.wrapping_sub(pos),
                b"CSI %c\0".as_ptr() as *const ::core::ffi::c_char,
                (*key).code.number & 0xff as ::core::ffi::c_int,
            ) as size_t;
        }
        _ => {}
    }
    if l <= 0 as size_t {
        return pos;
    }
    pos = pos.wrapping_add(l);
    if wrapbracket != 0 {
        l = snprintf(
            buffer.offset(pos as isize),
            len.wrapping_sub(pos),
            b">\0".as_ptr() as *const ::core::ffi::c_char,
        ) as size_t;
        if l <= 0 as size_t {
            return pos;
        }
        pos = pos.wrapping_add(l);
    }
    return pos;
}
pub const ENOENT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENOMEM: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const VQUIT: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const VTIME: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const VMIN: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const VSUSP: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const INLCR: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const ICRNL: ::core::ffi::c_int = 0o400 as ::core::ffi::c_int;
pub const IXON: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const ISIG: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const ICANON: ::core::ffi::c_int = 0o2 as ::core::ffi::c_int;
pub const ECHO: ::core::ffi::c_int = 0o10 as ::core::ffi::c_int;
pub const IEXTEN: ::core::ffi::c_int = 0o100000 as ::core::ffi::c_int;
pub const TCSANOW: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const _POSIX_VDISABLE: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
