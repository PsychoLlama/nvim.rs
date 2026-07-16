extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn strncpy(
        __dest: *mut ::core::ffi::c_char,
        __src: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn abort() -> !;
}
pub type uint8_t = u8;
pub type size_t = usize;
pub type cc_t = ::core::ffi::c_uchar;
pub type speed_t = ::core::ffi::c_uint;
pub type tcflag_t = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct termios {
    pub c_iflag: tcflag_t,
    pub c_oflag: tcflag_t,
    pub c_cflag: tcflag_t,
    pub c_lflag: tcflag_t,
    pub c_line: cc_t,
    pub c_cc: [cc_t; 32],
    pub c_ispeed: speed_t,
    pub c_ospeed: speed_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminfoEntry {
    pub bce: bool,
    pub has_Tc_or_RGB: bool,
    pub Su: bool,
    pub max_colors: ::core::ffi::c_int,
    pub lines: ::core::ffi::c_int,
    pub columns: ::core::ffi::c_int,
    pub defs: [*const ::core::ffi::c_char; 49],
    pub keys: [[*const ::core::ffi::c_char; 2]; 16],
    pub f_keys: [*const ::core::ffi::c_char; 63],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKey {
    pub fd: ::core::ffi::c_int,
    pub flags: ::core::ffi::c_int,
    pub canonflags: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_uchar,
    pub buffstart: size_t,
    pub buffcount: size_t,
    pub buffsize: size_t,
    pub hightide: size_t,
    pub restore_termios: termios,
    pub restore_termios_valid: ::core::ffi::c_char,
    pub ti_getstr_hook: Option<TermKey_Terminfo_Getstr_Hook>,
    pub ti_getstr_hook_data: *mut ::core::ffi::c_void,
    pub waittime: ::core::ffi::c_int,
    pub is_closed: ::core::ffi::c_char,
    pub is_started: ::core::ffi::c_char,
    pub nkeynames: ::core::ffi::c_int,
    pub keynames: *mut *const ::core::ffi::c_char,
    pub c0: [keyinfo; 32],
    pub drivers: *mut TermKeyDriverNode,
    pub method: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed {
    pub emit_codepoint:
        Option<unsafe extern "C" fn(*mut TermKey, ::core::ffi::c_int, *mut TermKeyKey) -> ()>,
    pub peekkey_simple: Option<
        unsafe extern "C" fn(
            *mut TermKey,
            *mut TermKeyKey,
            ::core::ffi::c_int,
            *mut size_t,
        ) -> TermKeyResult,
    >,
    pub peekkey_mouse:
        Option<unsafe extern "C" fn(*mut TermKey, *mut TermKeyKey, *mut size_t) -> TermKeyResult>,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyKey {
    pub type_0: TermKeyType,
    pub code: C2Rust_Unnamed_0,
    pub modifiers: ::core::ffi::c_int,
    pub event: TermKeyEvent,
    pub utf8: [::core::ffi::c_char; 7],
}
pub type TermKeyEvent = ::core::ffi::c_uint;
pub const TERMKEY_EVENT_RELEASE: TermKeyEvent = 3;
pub const TERMKEY_EVENT_REPEAT: TermKeyEvent = 2;
pub const TERMKEY_EVENT_PRESS: TermKeyEvent = 1;
pub const TERMKEY_EVENT_UNKNOWN: TermKeyEvent = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub codepoint: ::core::ffi::c_int,
    pub number: ::core::ffi::c_int,
    pub sym: TermKeySym,
    pub mouse: [::core::ffi::c_char; 4],
}
pub type TermKeySym = ::core::ffi::c_int;
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
pub type TermKeyType = ::core::ffi::c_int;
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
pub type TermKeyResult = ::core::ffi::c_uint;
pub const TERMKEY_RES_ERROR: TermKeyResult = 4;
pub const TERMKEY_RES_AGAIN: TermKeyResult = 3;
pub const TERMKEY_RES_EOF: TermKeyResult = 2;
pub const TERMKEY_RES_KEY: TermKeyResult = 1;
pub const TERMKEY_RES_NONE: TermKeyResult = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyDriverNode {
    pub driver: *mut TermKeyDriver,
    pub info: *mut ::core::ffi::c_void,
    pub next: *mut TermKeyDriverNode,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyDriver {
    pub name: *const ::core::ffi::c_char,
    pub new_driver:
        Option<unsafe extern "C" fn(*mut TermKey, *mut TerminfoEntry) -> *mut ::core::ffi::c_void>,
    pub free_driver: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub start_driver:
        Option<unsafe extern "C" fn(*mut TermKey, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub stop_driver:
        Option<unsafe extern "C" fn(*mut TermKey, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub peekkey: Option<
        unsafe extern "C" fn(
            *mut TermKey,
            *mut ::core::ffi::c_void,
            *mut TermKeyKey,
            ::core::ffi::c_int,
            *mut size_t,
        ) -> TermKeyResult,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keyinfo {
    pub type_0: TermKeyType,
    pub sym: TermKeySym,
    pub modifier_mask: ::core::ffi::c_int,
    pub modifier_set: ::core::ffi::c_int,
}
pub type TermKey_Terminfo_Getstr_Hook = unsafe extern "C" fn(
    *const ::core::ffi::c_char,
    *const ::core::ffi::c_char,
    *mut ::core::ffi::c_void,
) -> *const ::core::ffi::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyCsi {
    pub tk: *mut TermKey,
    pub saved_string_id: ::core::ffi::c_int,
    pub saved_string: *mut ::core::ffi::c_char,
}
pub type TermKeyMouseEvent = ::core::ffi::c_uint;
pub const TERMKEY_MOUSE_RELEASE: TermKeyMouseEvent = 3;
pub const TERMKEY_MOUSE_DRAG: TermKeyMouseEvent = 2;
pub const TERMKEY_MOUSE_PRESS: TermKeyMouseEvent = 1;
pub const TERMKEY_MOUSE_UNKNOWN: TermKeyMouseEvent = 0;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub const TERMKEY_KEYMOD_CTRL: C2Rust_Unnamed_1 = 4;
pub const TERMKEY_KEYMOD_ALT: C2Rust_Unnamed_1 = 2;
pub const TERMKEY_KEYMOD_SHIFT: C2Rust_Unnamed_1 = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyCsiParam {
    pub param: *const ::core::ffi::c_uchar,
    pub length: size_t,
}
pub type C2Rust_Unnamed_2 = ::core::ffi::c_uint;
pub const TERMKEY_FLAG_KEEPC0: C2Rust_Unnamed_2 = 512;
pub const TERMKEY_FLAG_NOSTART: C2Rust_Unnamed_2 = 256;
pub const TERMKEY_FLAG_EINTR: C2Rust_Unnamed_2 = 128;
pub const TERMKEY_FLAG_CTRLC: C2Rust_Unnamed_2 = 64;
pub const TERMKEY_FLAG_SPACESYMBOL: C2Rust_Unnamed_2 = 32;
pub const TERMKEY_FLAG_NOTERMIOS: C2Rust_Unnamed_2 = 16;
pub const TERMKEY_FLAG_UTF8: C2Rust_Unnamed_2 = 8;
pub const TERMKEY_FLAG_RAW: C2Rust_Unnamed_2 = 4;
pub const TERMKEY_FLAG_CONVERTKP: C2Rust_Unnamed_2 = 2;
pub const TERMKEY_FLAG_NOINTERPRET: C2Rust_Unnamed_2 = 1;
pub type CsiHandler = unsafe extern "C" fn(
    *mut TermKey,
    *mut TermKeyKey,
    ::core::ffi::c_int,
    *mut TermKeyCsiParam,
    ::core::ffi::c_int,
) -> TermKeyResult;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 83] = unsafe {
    ::core::mem::transmute::<[u8; 83], [::core::ffi::c_char; 83]>(
        *b"TermKeyResult termkey_interpret_csi_param(TermKeyCsiParam, int *, int *, size_t *)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn termkey_key_get_linecol(
    mut key: *const TermKeyKey,
    mut line: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    if !col.is_null() {
        *col = (*key).code.mouse[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_uchar
            as ::core::ffi::c_int
            | ((*key).code.mouse[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_uchar
                as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                << 8 as ::core::ffi::c_int;
    }
    if !line.is_null() {
        *line = (*key).code.mouse[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_uchar
            as ::core::ffi::c_int
            | ((*key).code.mouse[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_uchar
                as ::core::ffi::c_int
                & 0x70 as ::core::ffi::c_int)
                << 4 as ::core::ffi::c_int;
    }
}
#[inline]
unsafe extern "C" fn termkey_key_set_linecol(
    mut key: *mut TermKeyKey,
    mut line: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    if line > 0xfff as ::core::ffi::c_int {
        line = 0xfff as ::core::ffi::c_int;
    }
    if col > 0x7ff as ::core::ffi::c_int {
        col = 0x7ff as ::core::ffi::c_int;
    }
    (*key).code.mouse[1 as ::core::ffi::c_int as usize] =
        (line & 0xff as ::core::ffi::c_int) as ::core::ffi::c_char;
    (*key).code.mouse[2 as ::core::ffi::c_int as usize] =
        (col & 0xff as ::core::ffi::c_int) as ::core::ffi::c_char;
    (*key).code.mouse[3 as ::core::ffi::c_int as usize] = ((line & 0xf00 as ::core::ffi::c_int)
        >> 8 as ::core::ffi::c_int
        | (col & 0x300 as ::core::ffi::c_int) >> 4 as ::core::ffi::c_int)
        as ::core::ffi::c_char;
}
static mut keyinfo_initialised: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut ss3s: [keyinfo; 64] = [keyinfo {
    type_0: TERMKEY_TYPE_UNICODE,
    sym: TERMKEY_SYM_NONE,
    modifier_mask: 0,
    modifier_set: 0,
}; 64];
static mut ss3_kpalts: [::core::ffi::c_char; 64] = [0; 64];
static mut csi_handlers: [Option<CsiHandler>; 64] = [None; 64];
static mut csi_ss3s: [keyinfo; 64] = [keyinfo {
    type_0: TERMKEY_TYPE_UNICODE,
    sym: TERMKEY_SYM_NONE,
    modifier_mask: 0,
    modifier_set: 0,
}; 64];
unsafe extern "C" fn handle_csi_ss3_full(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut cmd: ::core::ffi::c_int,
    mut params: *mut TermKeyCsiParam,
    mut nparams: ::core::ffi::c_int,
) -> TermKeyResult {
    let mut result: TermKeyResult = TERMKEY_RES_KEY;
    if nparams > 1 as ::core::ffi::c_int
        && !(*params.offset(1 as ::core::ffi::c_int as isize))
            .param
            .is_null()
    {
        let mut arg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut subparam: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut nsubparams: size_t = 1 as size_t;
        result = termkey_interpret_csi_param(
            *params.offset(1 as ::core::ffi::c_int as isize),
            &raw mut arg,
            &raw mut subparam,
            &raw mut nsubparams,
        );
        if result as ::core::ffi::c_uint
            != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return result;
        }
        if nsubparams > 0 as size_t {
            (*key).event = parse_key_event(subparam);
            if (*key).event as ::core::ffi::c_uint
                == TERMKEY_EVENT_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_NONE;
            }
        }
        (*key).modifiers = arg - 1 as ::core::ffi::c_int;
    } else {
        (*key).modifiers = 0 as ::core::ffi::c_int;
    }
    (*key).type_0 = csi_ss3s[(cmd - 0x40 as ::core::ffi::c_int) as usize].type_0;
    (*key).code.sym = csi_ss3s[(cmd - 0x40 as ::core::ffi::c_int) as usize].sym;
    (*key).modifiers &= !csi_ss3s[(cmd - 0x40 as ::core::ffi::c_int) as usize].modifier_mask;
    (*key).modifiers |= csi_ss3s[(cmd - 0x40 as ::core::ffi::c_int) as usize].modifier_set;
    if (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_UNKNOWN as ::core::ffi::c_int {
        result = TERMKEY_RES_NONE;
    }
    return result;
}
unsafe extern "C" fn register_csi_ss3_full(
    mut type_0: TermKeyType,
    mut sym: TermKeySym,
    mut modifier_set: ::core::ffi::c_int,
    mut modifier_mask: ::core::ffi::c_int,
    mut cmd: ::core::ffi::c_uchar,
) {
    if (cmd as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        || cmd as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
    {
        return;
    }
    csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].type_0 = type_0;
    csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].sym = sym;
    csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].modifier_set =
        modifier_set;
    csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].modifier_mask =
        modifier_mask;
    csi_handlers[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csi_ss3_full
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
}
unsafe extern "C" fn register_csi_ss3(
    mut type_0: TermKeyType,
    mut sym: TermKeySym,
    mut cmd: ::core::ffi::c_uchar,
) {
    register_csi_ss3_full(
        type_0,
        sym,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        cmd,
    );
}
unsafe extern "C" fn register_ss3kpalt(
    mut type_0: TermKeyType,
    mut sym: TermKeySym,
    mut cmd: ::core::ffi::c_uchar,
    mut kpalt: ::core::ffi::c_char,
) {
    if (cmd as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        || cmd as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
    {
        return;
    }
    ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].type_0 = type_0;
    ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].sym = sym;
    ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].modifier_set =
        0 as ::core::ffi::c_int;
    ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].modifier_mask =
        0 as ::core::ffi::c_int;
    ss3_kpalts[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = kpalt;
}
pub const NCSIFUNCS: ::core::ffi::c_int = 35 as ::core::ffi::c_int;
static mut csifuncs: [keyinfo; 35] = [keyinfo {
    type_0: TERMKEY_TYPE_UNICODE,
    sym: TERMKEY_SYM_NONE,
    modifier_mask: 0,
    modifier_set: 0,
}; 35];
unsafe extern "C" fn handle_csifunc(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut cmd: ::core::ffi::c_int,
    mut params: *mut TermKeyCsiParam,
    mut nparams: ::core::ffi::c_int,
) -> TermKeyResult {
    if nparams == 0 as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    let mut result: TermKeyResult = TERMKEY_RES_KEY;
    let mut args: [::core::ffi::c_int; 3] = [0; 3];
    if nparams > 1 as ::core::ffi::c_int
        && !(*params.offset(1 as ::core::ffi::c_int as isize))
            .param
            .is_null()
    {
        let mut subparam: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut nsubparams: size_t = 1 as size_t;
        result = termkey_interpret_csi_param(
            *params.offset(1 as ::core::ffi::c_int as isize),
            (&raw mut args as *mut ::core::ffi::c_int).offset(1 as ::core::ffi::c_int as isize),
            &raw mut subparam,
            &raw mut nsubparams,
        );
        if result as ::core::ffi::c_uint
            != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return result;
        }
        if nsubparams > 0 as size_t {
            (*key).event = parse_key_event(subparam);
            if (*key).event as ::core::ffi::c_uint
                == TERMKEY_EVENT_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_NONE;
            }
        }
        (*key).modifiers = args[1 as ::core::ffi::c_int as usize] - 1 as ::core::ffi::c_int;
    } else {
        (*key).modifiers = 0 as ::core::ffi::c_int;
    }
    (*key).type_0 = TERMKEY_TYPE_KEYSYM;
    result = termkey_interpret_csi_param(
        *params.offset(0 as ::core::ffi::c_int as isize),
        (&raw mut args as *mut ::core::ffi::c_int).offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<size_t>(),
    );
    if result as ::core::ffi::c_uint != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return result;
    }
    if args[0 as ::core::ffi::c_int as usize] == 27 as ::core::ffi::c_int
        && nparams > 2 as ::core::ffi::c_int
        && !(*params.offset(2 as ::core::ffi::c_int as isize))
            .param
            .is_null()
    {
        result = termkey_interpret_csi_param(
            *params.offset(2 as ::core::ffi::c_int as isize),
            (&raw mut args as *mut ::core::ffi::c_int).offset(2 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<size_t>(),
        );
        if result as ::core::ffi::c_uint
            != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return result;
        }
        let mut mod_0: ::core::ffi::c_int = (*key).modifiers;
        Some(
            (*tk)
                .method
                .emit_codepoint
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            tk, args[2 as ::core::ffi::c_int as usize], key
        );
        (*key).modifiers |= mod_0;
    } else if args[0 as ::core::ffi::c_int as usize] >= 0 as ::core::ffi::c_int
        && args[0 as ::core::ffi::c_int as usize] < NCSIFUNCS
    {
        (*key).type_0 = csifuncs[args[0 as ::core::ffi::c_int as usize] as usize].type_0;
        (*key).code.sym = csifuncs[args[0 as ::core::ffi::c_int as usize] as usize].sym;
        (*key).modifiers &=
            !csifuncs[args[0 as ::core::ffi::c_int as usize] as usize].modifier_mask;
        (*key).modifiers |= csifuncs[args[0 as ::core::ffi::c_int as usize] as usize].modifier_set;
    } else {
        (*key).code.sym = TERMKEY_SYM_UNKNOWN;
    }
    if (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_UNKNOWN as ::core::ffi::c_int {
        result = TERMKEY_RES_NONE;
    }
    return result;
}
unsafe extern "C" fn register_csifunc(
    mut type_0: TermKeyType,
    mut sym: TermKeySym,
    mut number: ::core::ffi::c_int,
) {
    if number >= NCSIFUNCS {
        return;
    }
    csifuncs[number as usize].type_0 = type_0;
    csifuncs[number as usize].sym = sym;
    csifuncs[number as usize].modifier_set = 0 as ::core::ffi::c_int;
    csifuncs[number as usize].modifier_mask = 0 as ::core::ffi::c_int;
    csi_handlers[('~' as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csifunc
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
}
unsafe extern "C" fn handle_csi_u(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut cmd: ::core::ffi::c_int,
    mut params: *mut TermKeyCsiParam,
    mut nparams: ::core::ffi::c_int,
) -> TermKeyResult {
    match cmd {
        117 => {
            let mut args: [::core::ffi::c_int; 2] = [0; 2];
            if nparams > 1 as ::core::ffi::c_int
                && !(*params.offset(1 as ::core::ffi::c_int as isize))
                    .param
                    .is_null()
            {
                let mut subparam: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut nsubparams: size_t = 1 as size_t;
                if termkey_interpret_csi_param(
                    *params.offset(1 as ::core::ffi::c_int as isize),
                    (&raw mut args as *mut ::core::ffi::c_int)
                        .offset(1 as ::core::ffi::c_int as isize),
                    &raw mut subparam,
                    &raw mut nsubparams,
                ) as ::core::ffi::c_uint
                    != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    return TERMKEY_RES_ERROR;
                }
                if nsubparams > 0 as size_t {
                    (*key).event = parse_key_event(subparam);
                    if (*key).event as ::core::ffi::c_uint
                        == TERMKEY_EVENT_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        return TERMKEY_RES_NONE;
                    }
                }
                (*key).modifiers = args[1 as ::core::ffi::c_int as usize] - 1 as ::core::ffi::c_int;
            } else {
                (*key).modifiers = 0 as ::core::ffi::c_int;
            }
            if termkey_interpret_csi_param(
                *params.offset(0 as ::core::ffi::c_int as isize),
                (&raw mut args as *mut ::core::ffi::c_int).offset(0 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<size_t>(),
            ) as ::core::ffi::c_uint
                != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_ERROR;
            }
            let mut mod_0: ::core::ffi::c_int = (*key).modifiers;
            (*key).type_0 = TERMKEY_TYPE_KEYSYM;
            Some(
                (*tk)
                    .method
                    .emit_codepoint
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(
                tk, args[0 as ::core::ffi::c_int as usize], key
            );
            (*key).modifiers |= mod_0;
            return TERMKEY_RES_KEY;
        }
        _ => return TERMKEY_RES_NONE,
    };
}
unsafe extern "C" fn handle_csi_m(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut cmd: ::core::ffi::c_int,
    mut params: *mut TermKeyCsiParam,
    mut nparams: ::core::ffi::c_int,
) -> TermKeyResult {
    let mut initial: ::core::ffi::c_int = cmd >> 8 as ::core::ffi::c_int;
    cmd &= 0xff as ::core::ffi::c_int;
    match cmd {
        77 | 109 => {}
        _ => return TERMKEY_RES_NONE,
    }
    if nparams < 3 as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    let mut args: [::core::ffi::c_int; 3] = [0; 3];
    let mut i: size_t = 0 as size_t;
    while i < 3 as size_t {
        if termkey_interpret_csi_param(
            *params.offset(i as isize),
            (&raw mut args as *mut ::core::ffi::c_int).offset(i as isize),
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<size_t>(),
        ) as ::core::ffi::c_uint
            != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return TERMKEY_RES_ERROR;
        }
        i = i.wrapping_add(1);
    }
    if initial == 0 {
        (*key).type_0 = TERMKEY_TYPE_MOUSE;
        (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
            args[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_char;
        (*key).modifiers = ((*key).code.mouse[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            & 0x1c as ::core::ffi::c_int)
            >> 2 as ::core::ffi::c_int;
        (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
            ((*key).code.mouse[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                & !(0x1c as ::core::ffi::c_int)) as ::core::ffi::c_char;
        termkey_key_set_linecol(
            key,
            args[1 as ::core::ffi::c_int as usize],
            args[2 as ::core::ffi::c_int as usize],
        );
        return TERMKEY_RES_KEY;
    }
    if initial == '<' as ::core::ffi::c_int {
        (*key).type_0 = TERMKEY_TYPE_MOUSE;
        (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
            args[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_char;
        (*key).modifiers = ((*key).code.mouse[0 as ::core::ffi::c_int as usize]
            as ::core::ffi::c_int
            & 0x1c as ::core::ffi::c_int)
            >> 2 as ::core::ffi::c_int;
        (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
            ((*key).code.mouse[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                & !(0x1c as ::core::ffi::c_int)) as ::core::ffi::c_char;
        termkey_key_set_linecol(
            key,
            args[1 as ::core::ffi::c_int as usize],
            args[2 as ::core::ffi::c_int as usize],
        );
        if cmd == 'm' as ::core::ffi::c_int {
            (*key).code.mouse[3 as ::core::ffi::c_int as usize] =
                ((*key).code.mouse[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    | 0x80 as ::core::ffi::c_int) as ::core::ffi::c_char;
        }
        return TERMKEY_RES_KEY;
    }
    return TERMKEY_RES_NONE;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_interpret_mouse(
    mut tk: *mut TermKey,
    mut key: *const TermKeyKey,
    mut event: *mut TermKeyMouseEvent,
    mut button: *mut ::core::ffi::c_int,
    mut line: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) -> TermKeyResult {
    if (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_MOUSE as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    if !button.is_null() {
        *button = 0 as ::core::ffi::c_int;
    }
    termkey_key_get_linecol(key, line, col);
    if event.is_null() {
        return TERMKEY_RES_KEY;
    }
    let mut btn: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut code: ::core::ffi::c_int = (*key).code.mouse[0 as ::core::ffi::c_int as usize]
        as ::core::ffi::c_uchar as ::core::ffi::c_int;
    let mut drag: ::core::ffi::c_int = code & 0x20 as ::core::ffi::c_int;
    code &= !(0x3c as ::core::ffi::c_int);
    match code {
        0 | 1 | 2 => {
            *event = (if drag != 0 {
                TERMKEY_MOUSE_DRAG as ::core::ffi::c_int
            } else {
                TERMKEY_MOUSE_PRESS as ::core::ffi::c_int
            }) as TermKeyMouseEvent;
            btn = code + 1 as ::core::ffi::c_int;
        }
        3 => {
            *event = TERMKEY_MOUSE_RELEASE;
        }
        64 | 65 | 66 | 67 => {
            *event = (if drag != 0 {
                TERMKEY_MOUSE_DRAG as ::core::ffi::c_int
            } else {
                TERMKEY_MOUSE_PRESS as ::core::ffi::c_int
            }) as TermKeyMouseEvent;
            btn = code + 4 as ::core::ffi::c_int - 64 as ::core::ffi::c_int;
        }
        128 | 129 => {
            *event = (if drag != 0 {
                TERMKEY_MOUSE_DRAG as ::core::ffi::c_int
            } else {
                TERMKEY_MOUSE_PRESS as ::core::ffi::c_int
            }) as TermKeyMouseEvent;
            btn = code + 8 as ::core::ffi::c_int - 128 as ::core::ffi::c_int;
        }
        _ => {
            *event = TERMKEY_MOUSE_UNKNOWN;
        }
    }
    if !button.is_null() {
        *button = btn;
    }
    if (*key).code.mouse[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
        & 0x80 as ::core::ffi::c_int
        != 0
    {
        *event = TERMKEY_MOUSE_RELEASE;
    }
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn handle_csi_R(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut cmd: ::core::ffi::c_int,
    mut params: *mut TermKeyCsiParam,
    mut nparams: ::core::ffi::c_int,
) -> TermKeyResult {
    let mut args: [::core::ffi::c_int; 2] = [0; 2];
    match cmd {
        16210 => {
            if nparams < 2 as ::core::ffi::c_int {
                return TERMKEY_RES_NONE;
            }
            args = [0; 2];
            if termkey_interpret_csi_param(
                *params.offset(0 as ::core::ffi::c_int as isize),
                (&raw mut args as *mut ::core::ffi::c_int).offset(0 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<size_t>(),
            ) as ::core::ffi::c_uint
                != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_ERROR;
            }
            if termkey_interpret_csi_param(
                *params.offset(1 as ::core::ffi::c_int as isize),
                (&raw mut args as *mut ::core::ffi::c_int).offset(1 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<size_t>(),
            ) as ::core::ffi::c_uint
                != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_ERROR;
            }
            (*key).type_0 = TERMKEY_TYPE_POSITION;
            termkey_key_set_linecol(
                key,
                args[1 as ::core::ffi::c_int as usize],
                args[0 as ::core::ffi::c_int as usize],
            );
            return TERMKEY_RES_KEY;
        }
        _ => return handle_csi_ss3_full(tk, key, cmd, params, nparams),
    };
}
#[no_mangle]
pub unsafe extern "C" fn termkey_interpret_position(
    mut tk: *mut TermKey,
    mut key: *const TermKeyKey,
    mut line: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) -> TermKeyResult {
    if (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_POSITION as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    termkey_key_get_linecol(key, line, col);
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn handle_csi_y(
    mut tk: *mut TermKey,
    mut key: *mut TermKeyKey,
    mut cmd: ::core::ffi::c_int,
    mut params: *mut TermKeyCsiParam,
    mut nparams: ::core::ffi::c_int,
) -> TermKeyResult {
    let mut args: [::core::ffi::c_int; 2] = [0; 2];
    match cmd {
        2359417 | 2375545 => {
            if nparams < 2 as ::core::ffi::c_int {
                return TERMKEY_RES_NONE;
            }
            args = [0; 2];
            if termkey_interpret_csi_param(
                *params.offset(0 as ::core::ffi::c_int as isize),
                (&raw mut args as *mut ::core::ffi::c_int).offset(0 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<size_t>(),
            ) as ::core::ffi::c_uint
                != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_ERROR;
            }
            if termkey_interpret_csi_param(
                *params.offset(1 as ::core::ffi::c_int as isize),
                (&raw mut args as *mut ::core::ffi::c_int).offset(1 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                ::core::ptr::null_mut::<size_t>(),
            ) as ::core::ffi::c_uint
                != TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return TERMKEY_RES_ERROR;
            }
            (*key).type_0 = TERMKEY_TYPE_MODEREPORT;
            (*key).code.mouse[0 as ::core::ffi::c_int as usize] =
                (cmd >> 8 as ::core::ffi::c_int) as ::core::ffi::c_char;
            (*key).code.mouse[1 as ::core::ffi::c_int as usize] =
                (args[0 as ::core::ffi::c_int as usize] >> 8 as ::core::ffi::c_int)
                    as ::core::ffi::c_char;
            (*key).code.mouse[2 as ::core::ffi::c_int as usize] =
                (args[0 as ::core::ffi::c_int as usize] & 0xff as ::core::ffi::c_int)
                    as ::core::ffi::c_char;
            (*key).code.mouse[3 as ::core::ffi::c_int as usize] =
                args[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_char;
            return TERMKEY_RES_KEY;
        }
        _ => return TERMKEY_RES_NONE,
    };
}
#[no_mangle]
pub unsafe extern "C" fn termkey_interpret_modereport(
    mut tk: *mut TermKey,
    mut key: *const TermKeyKey,
    mut initial: *mut ::core::ffi::c_int,
    mut mode: *mut ::core::ffi::c_int,
    mut value: *mut ::core::ffi::c_int,
) -> TermKeyResult {
    if (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_MODEREPORT as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    if !initial.is_null() {
        *initial = (*key).code.mouse[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_uchar
            as ::core::ffi::c_int;
    }
    if !mode.is_null() {
        *mode = ((*key).code.mouse[1 as ::core::ffi::c_int as usize] as uint8_t
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int
            | (*key).code.mouse[2 as ::core::ffi::c_int as usize] as uint8_t as ::core::ffi::c_int;
    }
    if !value.is_null() {
        *value = (*key).code.mouse[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_uchar
            as ::core::ffi::c_int;
    }
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn parse_key_event(mut n: ::core::ffi::c_int) -> TermKeyEvent {
    match n {
        1 => return TERMKEY_EVENT_PRESS,
        2 => return TERMKEY_EVENT_REPEAT,
        3 => return TERMKEY_EVENT_RELEASE,
        _ => return TERMKEY_EVENT_UNKNOWN,
    };
}
unsafe extern "C" fn parse_csi(
    mut tk: *mut TermKey,
    mut introlen: size_t,
    mut csi_len: *mut size_t,
    mut params: *mut TermKeyCsiParam,
    mut nargs: *mut size_t,
    mut commandp: *mut ::core::ffi::c_uint,
) -> TermKeyResult {
    let mut csi_end: size_t = introlen;
    while csi_end < (*tk).buffcount {
        if *(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(csi_end) as isize) as ::core::ffi::c_int
            >= 0x40 as ::core::ffi::c_int
            && (*(*tk)
                .buffer
                .offset((*tk).buffstart.wrapping_add(csi_end) as isize)
                as ::core::ffi::c_int)
                < 0x80 as ::core::ffi::c_int
        {
            break;
        }
        csi_end = csi_end.wrapping_add(1);
    }
    if csi_end >= (*tk).buffcount {
        return TERMKEY_RES_AGAIN;
    }
    let mut cmd: ::core::ffi::c_uchar = *(*tk)
        .buffer
        .offset((*tk).buffstart.wrapping_add(csi_end) as isize);
    *commandp = cmd as ::core::ffi::c_uint;
    let mut present: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
    let mut argi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut p: size_t = introlen;
    if *(*tk)
        .buffer
        .offset((*tk).buffstart.wrapping_add(p) as isize) as ::core::ffi::c_int
        >= '<' as ::core::ffi::c_int
        && *(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(p) as isize) as ::core::ffi::c_int
            <= '?' as ::core::ffi::c_int
    {
        *commandp |= ((*(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(p) as isize)
            as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as ::core::ffi::c_uint;
        p = p.wrapping_add(1);
    }
    while p < csi_end {
        let mut c: ::core::ffi::c_uchar = *(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(p) as isize);
        if c as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
            && (c as ::core::ffi::c_int) < ';' as ::core::ffi::c_int
        {
            if present == 0 {
                (*params.offset(argi as isize)).param = (*tk)
                    .buffer
                    .offset((*tk).buffstart.wrapping_add(p) as isize);
                present = 1 as ::core::ffi::c_char;
            }
        } else if c as ::core::ffi::c_int == ';' as ::core::ffi::c_int {
            if present == 0 {
                (*params.offset(argi as isize)).param = ::core::ptr::null::<::core::ffi::c_uchar>();
                (*params.offset(argi as isize)).length = 0 as size_t;
            } else {
                (*params.offset(argi as isize)).length = (*tk)
                    .buffer
                    .offset((*tk).buffstart.wrapping_add(p) as isize)
                    .offset_from((*params.offset(argi as isize)).param)
                    as size_t;
            }
            present = 0 as ::core::ffi::c_char;
            argi += 1;
            if argi >= 16 as ::core::ffi::c_int {
                break;
            }
        } else if c as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
            && c as ::core::ffi::c_int <= 0x2f as ::core::ffi::c_int
        {
            *commandp |=
                ((c as ::core::ffi::c_int) << 16 as ::core::ffi::c_int) as ::core::ffi::c_uint;
            break;
        }
        p = p.wrapping_add(1);
    }
    if present != 0 {
        (*params.offset(argi as isize)).length = (*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(p) as isize)
            .offset_from((*params.offset(argi as isize)).param)
            as size_t;
        argi += 1;
    }
    *nargs = argi as size_t;
    *csi_len = csi_end.wrapping_add(1 as size_t);
    return TERMKEY_RES_KEY;
}
#[no_mangle]
pub unsafe extern "C" fn termkey_interpret_csi(
    mut tk: *mut TermKey,
    mut key: *const TermKeyKey,
    mut params: *mut TermKeyCsiParam,
    mut nparams: *mut size_t,
    mut cmd: *mut ::core::ffi::c_uint,
) -> TermKeyResult {
    let mut dummy: size_t = 0;
    if (*tk).hightide == 0 as size_t {
        return TERMKEY_RES_NONE;
    }
    if (*key).type_0 as ::core::ffi::c_int != TERMKEY_TYPE_UNKNOWN_CSI as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    return parse_csi(tk, 0 as size_t, &raw mut dummy, params, nparams, cmd);
}
#[no_mangle]
pub unsafe extern "C" fn termkey_interpret_csi_param(
    mut param: TermKeyCsiParam,
    mut paramp: *mut ::core::ffi::c_int,
    mut subparams: *mut ::core::ffi::c_int,
    mut nsubparams: *mut size_t,
) -> TermKeyResult {
    if paramp.is_null() {
        return TERMKEY_RES_ERROR;
    }
    if param.param.is_null() {
        *paramp = -1 as ::core::ffi::c_int;
        if !nsubparams.is_null() {
            *nsubparams = 0 as size_t;
        }
        return TERMKEY_RES_KEY;
    }
    let mut arg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    let mut capacity: size_t = if !nsubparams.is_null() {
        *nsubparams
    } else {
        0 as size_t
    };
    let mut length: size_t = 0 as size_t;
    while i < param.length && length <= capacity {
        let mut c: ::core::ffi::c_uchar = *param.param.offset(i as isize);
        if c as ::core::ffi::c_int == ':' as ::core::ffi::c_int {
            if length == 0 as size_t {
                *paramp = arg;
            } else if !subparams.is_null() {
                *subparams.offset(length.wrapping_sub(1 as size_t) as isize) = arg;
            }
            arg = 0 as ::core::ffi::c_int;
            length = length.wrapping_add(1);
        } else {
            '_c2rust_label: {
                if c as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                    && c as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
                {
                } else {
                    __assert_fail(
                        b"c >= '0' && c <= '9'\0".as_ptr() as *const ::core::ffi::c_char,
                        b"/home/overlord/projects/neovim/neovim/src/nvim/tui/termkey/driver-csi.c\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        578 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            arg = 10 as ::core::ffi::c_int * arg
                + (c as ::core::ffi::c_int - '0' as ::core::ffi::c_int);
        }
        i = i.wrapping_add(1);
    }
    if length == 0 as size_t {
        *paramp = arg;
    } else if !subparams.is_null() {
        *subparams.offset(length.wrapping_sub(1 as size_t) as isize) = arg;
    }
    if !nsubparams.is_null() {
        *nsubparams = length;
    }
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn register_keys() -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 64 as ::core::ffi::c_int {
        csi_ss3s[i as usize].sym = TERMKEY_SYM_UNKNOWN;
        ss3s[i as usize].sym = TERMKEY_SYM_UNKNOWN;
        ss3_kpalts[i as usize] = 0 as ::core::ffi::c_char;
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < NCSIFUNCS {
        csifuncs[i as usize].sym = TERMKEY_SYM_UNKNOWN;
        i += 1;
    }
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_UP,
        'A' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_DOWN,
        'B' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_RIGHT,
        'C' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_LEFT,
        'D' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_BEGIN,
        'E' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_END,
        'F' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_HOME,
        'H' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_BACKSPACE,
        'P' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_TAB,
        'Q' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_ENTER,
        'R' as ::core::ffi::c_uchar,
    );
    register_csi_ss3(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_ESCAPE,
        'S' as ::core::ffi::c_uchar,
    );
    register_csi_ss3_full(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_TAB,
        TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int,
        TERMKEY_KEYMOD_SHIFT as ::core::ffi::c_int,
        'Z' as ::core::ffi::c_uchar,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPENTER,
        'M' as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPEQUALS,
        'X' as ::core::ffi::c_uchar,
        '=' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPMULT,
        'j' as ::core::ffi::c_uchar,
        '*' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPPLUS,
        'k' as ::core::ffi::c_uchar,
        '+' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPCOMMA,
        'l' as ::core::ffi::c_uchar,
        ',' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPMINUS,
        'm' as ::core::ffi::c_uchar,
        '-' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPPERIOD,
        'n' as ::core::ffi::c_uchar,
        '.' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KPDIV,
        'o' as ::core::ffi::c_uchar,
        '/' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP0,
        'p' as ::core::ffi::c_uchar,
        '0' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP1,
        'q' as ::core::ffi::c_uchar,
        '1' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP2,
        'r' as ::core::ffi::c_uchar,
        '2' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP3,
        's' as ::core::ffi::c_uchar,
        '3' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP4,
        't' as ::core::ffi::c_uchar,
        '4' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP5,
        'u' as ::core::ffi::c_uchar,
        '5' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP6,
        'v' as ::core::ffi::c_uchar,
        '6' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP7,
        'w' as ::core::ffi::c_uchar,
        '7' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP8,
        'x' as ::core::ffi::c_uchar,
        '8' as ::core::ffi::c_char,
    );
    register_ss3kpalt(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_KP9,
        'y' as ::core::ffi::c_uchar,
        '9' as ::core::ffi::c_char,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_FIND,
        1 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_INSERT,
        2 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_DELETE,
        3 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_SELECT,
        4 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_PAGEUP,
        5 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_PAGEDOWN,
        6 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_HOME,
        7 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_KEYSYM,
        TERMKEY_SYM_END,
        8 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_BACKSPACE,
        11 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_TAB,
        12 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_ENTER,
        13 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_ESCAPE,
        14 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_SPACE,
        15 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_DEL,
        17 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_UP,
        18 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_DOWN,
        19 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_LEFT,
        20 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_RIGHT,
        21 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_BEGIN,
        23 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_FIND,
        24 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_INSERT,
        25 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_DELETE,
        26 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_SELECT,
        28 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_PAGEUP,
        29 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_PAGEDOWN,
        31 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_HOME,
        32 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_END,
        33 as ::core::ffi::c_int,
    );
    register_csifunc(
        TERMKEY_TYPE_FUNCTION,
        TERMKEY_SYM_CANCEL,
        34 as ::core::ffi::c_int,
    );
    csi_handlers[('u' as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csi_u
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
    csi_handlers[('M' as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csi_m
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
    csi_handlers[('m' as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csi_m
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
    csi_handlers[('R' as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csi_R
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
    csi_handlers[('y' as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize] = Some(
        handle_csi_y
            as unsafe extern "C" fn(
                *mut TermKey,
                *mut TermKeyKey,
                ::core::ffi::c_int,
                *mut TermKeyCsiParam,
                ::core::ffi::c_int,
            ) -> TermKeyResult,
    )
        as Option<CsiHandler>;
    keyinfo_initialised = 1 as ::core::ffi::c_int;
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn new_driver_csi(
    mut tk: *mut TermKey,
    mut term: *mut TerminfoEntry,
) -> *mut ::core::ffi::c_void {
    if keyinfo_initialised == 0 {
        if register_keys() == 0 {
            return NULL;
        }
    }
    let mut csi: *mut TermKeyCsi = xmalloc(::core::mem::size_of::<TermKeyCsi>()) as *mut TermKeyCsi;
    (*csi).tk = tk;
    (*csi).saved_string_id = 0 as ::core::ffi::c_int;
    (*csi).saved_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    return csi as *mut ::core::ffi::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn free_driver_csi(mut info: *mut ::core::ffi::c_void) {
    let mut csi: *mut TermKeyCsi = info as *mut TermKeyCsi;
    if !(*csi).saved_string.is_null() {
        xfree((*csi).saved_string as *mut ::core::ffi::c_void);
    }
    xfree(csi as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn peekkey_csi_csi(
    mut tk: *mut TermKey,
    mut csi: *mut TermKeyCsi,
    mut introlen: size_t,
    mut key: *mut TermKeyKey,
    mut force: ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    let mut csi_len: size_t = 0;
    let mut nparams: size_t = 16 as size_t;
    let mut params: [TermKeyCsiParam; 16] = [TermKeyCsiParam {
        param: ::core::ptr::null::<::core::ffi::c_uchar>(),
        length: 0,
    }; 16];
    let mut cmd: ::core::ffi::c_uint = 0;
    let mut ret: TermKeyResult = parse_csi(
        tk,
        introlen,
        &raw mut csi_len,
        &raw mut params as *mut TermKeyCsiParam,
        &raw mut nparams,
        &raw mut cmd,
    );
    if ret as ::core::ffi::c_uint == TERMKEY_RES_AGAIN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if force == 0 {
            return TERMKEY_RES_AGAIN;
        }
        Some(
            (*tk)
                .method
                .emit_codepoint
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(tk, '[' as ::core::ffi::c_int, key);
        (*key).modifiers |= TERMKEY_KEYMOD_ALT as ::core::ffi::c_int;
        *nbytep = introlen;
        return TERMKEY_RES_KEY;
    }
    if cmd == 'M' as ::core::ffi::c_uint && nparams < 3 as size_t {
        (*tk).buffstart = (*tk).buffstart.wrapping_add(csi_len);
        (*tk).buffcount = (*tk).buffcount.wrapping_sub(csi_len);
        let mut mouse_result: TermKeyResult =
            Some(
                (*tk)
                    .method
                    .peekkey_mouse
                    .expect("non-null function pointer"),
            )
            .expect("non-null function pointer")(tk, key, nbytep);
        (*tk).buffstart = (*tk).buffstart.wrapping_sub(csi_len);
        (*tk).buffcount = (*tk).buffcount.wrapping_add(csi_len);
        if mouse_result as ::core::ffi::c_uint
            == TERMKEY_RES_KEY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            *nbytep = (*nbytep).wrapping_add(csi_len);
        }
        return mouse_result;
    }
    let mut result: TermKeyResult = TERMKEY_RES_NONE;
    if csi_handlers
        [(cmd & 0xff as ::core::ffi::c_uint).wrapping_sub(0x40 as ::core::ffi::c_uint) as usize]
        .is_some()
    {
        result = Some(
            (*(&raw mut csi_handlers as *mut Option<CsiHandler>).offset(
                (cmd & 0xff as ::core::ffi::c_uint).wrapping_sub(0x40 as ::core::ffi::c_uint)
                    as isize,
            ))
            .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(
            tk,
            key,
            cmd as ::core::ffi::c_int,
            &raw mut params as *mut TermKeyCsiParam,
            nparams as ::core::ffi::c_int,
        );
    }
    if result as ::core::ffi::c_uint
        == TERMKEY_RES_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*key).type_0 = TERMKEY_TYPE_UNKNOWN_CSI;
        (*key).code.number = cmd as ::core::ffi::c_int;
        (*key).modifiers = 0 as ::core::ffi::c_int;
        (*tk).hightide = csi_len.wrapping_sub(introlen);
        *nbytep = introlen;
        return TERMKEY_RES_KEY;
    }
    *nbytep = csi_len;
    return result;
}
unsafe extern "C" fn peekkey_ss3(
    mut tk: *mut TermKey,
    mut csi: *mut TermKeyCsi,
    mut introlen: size_t,
    mut key: *mut TermKeyKey,
    mut force: ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    if (*tk).buffcount < introlen.wrapping_add(1 as size_t) {
        if force == 0 {
            return TERMKEY_RES_AGAIN;
        }
        Some(
            (*tk)
                .method
                .emit_codepoint
                .expect("non-null function pointer"),
        )
        .expect("non-null function pointer")(tk, 'O' as ::core::ffi::c_int, key);
        (*key).modifiers |= TERMKEY_KEYMOD_ALT as ::core::ffi::c_int;
        *nbytep = (*tk).buffcount;
        return TERMKEY_RES_KEY;
    }
    let mut cmd: ::core::ffi::c_uchar = *(*tk)
        .buffer
        .offset((*tk).buffstart.wrapping_add(introlen) as isize);
    if (cmd as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        || cmd as ::core::ffi::c_int >= 0x80 as ::core::ffi::c_int
    {
        return TERMKEY_RES_NONE;
    }
    (*key).type_0 =
        csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].type_0;
    (*key).code.sym =
        csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].sym;
    (*key).modifiers =
        csi_ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].modifier_set;
    if (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_UNKNOWN as ::core::ffi::c_int {
        if (*tk).flags & TERMKEY_FLAG_CONVERTKP as ::core::ffi::c_int != 0
            && ss3_kpalts[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_int
                != 0
        {
            (*key).type_0 = TERMKEY_TYPE_UNICODE;
            (*key).code.codepoint = ss3_kpalts
                [(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize]
                as ::core::ffi::c_uchar as ::core::ffi::c_int;
            (*key).modifiers = 0 as ::core::ffi::c_int;
            (*key).utf8[0 as ::core::ffi::c_int as usize] =
                (*key).code.codepoint as ::core::ffi::c_char;
            (*key).utf8[1 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        } else {
            (*key).type_0 =
                ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].type_0;
            (*key).code.sym =
                ss3s[(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize].sym;
            (*key).modifiers = ss3s
                [(cmd as ::core::ffi::c_int - 0x40 as ::core::ffi::c_int) as usize]
                .modifier_set;
        }
    }
    if (*key).code.sym as ::core::ffi::c_int == TERMKEY_SYM_UNKNOWN as ::core::ffi::c_int {
        return TERMKEY_RES_NONE;
    }
    *nbytep = introlen.wrapping_add(1 as size_t);
    return TERMKEY_RES_KEY;
}
unsafe extern "C" fn peekkey_ctrlstring(
    mut tk: *mut TermKey,
    mut csi: *mut TermKeyCsi,
    mut introlen: size_t,
    mut key: *mut TermKeyKey,
    mut force: ::core::ffi::c_int,
    mut nbytep: *mut size_t,
) -> TermKeyResult {
    let mut str_end: size_t = introlen;
    while str_end < (*tk).buffcount {
        if *(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(str_end) as isize) as ::core::ffi::c_int
            == 0x7 as ::core::ffi::c_int
        {
            break;
        }
        if *(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(str_end) as isize) as ::core::ffi::c_int
            == 0x9c as ::core::ffi::c_int
        {
            break;
        }
        if *(*tk)
            .buffer
            .offset((*tk).buffstart.wrapping_add(str_end) as isize) as ::core::ffi::c_int
            == 0x1b as ::core::ffi::c_int
            && str_end.wrapping_add(1 as size_t) < (*tk).buffcount
            && *(*tk).buffer.offset(
                (*tk)
                    .buffstart
                    .wrapping_add(str_end.wrapping_add(1 as size_t)) as isize,
            ) as ::core::ffi::c_int
                == 0x5c as ::core::ffi::c_int
        {
            break;
        }
        str_end = str_end.wrapping_add(1);
    }
    if str_end >= (*tk).buffcount {
        return TERMKEY_RES_AGAIN;
    }
    *nbytep = str_end.wrapping_add(1 as size_t);
    if *(*tk)
        .buffer
        .offset((*tk).buffstart.wrapping_add(str_end) as isize) as ::core::ffi::c_int
        == 0x1b as ::core::ffi::c_int
    {
        *nbytep = (*nbytep).wrapping_add(1);
    }
    if !(*csi).saved_string.is_null() {
        xfree((*csi).saved_string as *mut ::core::ffi::c_void);
    }
    let mut len: size_t = str_end.wrapping_sub(introlen);
    (*csi).saved_string_id += 1;
    (*csi).saved_string = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    strncpy(
        (*csi).saved_string,
        ((*tk).buffer as *mut ::core::ffi::c_char)
            .offset((*tk).buffstart as isize)
            .offset(introlen as isize),
        len,
    );
    *(*csi).saved_string.offset(len as isize) = 0 as ::core::ffi::c_char;
    let mut type_0: ::core::ffi::c_char = (*(*tk).buffer.offset(
        (*tk)
            .buffstart
            .wrapping_add(introlen.wrapping_sub(1 as size_t)) as isize,
    ) as ::core::ffi::c_int
        & 0x1f as ::core::ffi::c_int)
        as ::core::ffi::c_char;
    match type_0 as ::core::ffi::c_int {
        16 => {
            (*key).type_0 = TERMKEY_TYPE_DCS;
        }
        29 => {
            (*key).type_0 = TERMKEY_TYPE_OSC;
        }
        31 => {
            (*key).type_0 = TERMKEY_TYPE_APC;
        }
        _ => {
            abort();
        }
    }
    (*key).code.number = (*csi).saved_string_id;
    (*key).modifiers = 0 as ::core::ffi::c_int;
    return TERMKEY_RES_KEY;
}
#[no_mangle]
pub unsafe extern "C" fn peekkey_csi(
    mut tk: *mut TermKey,
    mut info: *mut ::core::ffi::c_void,
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
    let mut csi: *mut TermKeyCsi = info as *mut TermKeyCsi;
    match *(*tk)
        .buffer
        .offset((*tk).buffstart.wrapping_add(0 as size_t) as isize) as ::core::ffi::c_int
    {
        27 => {
            if (*tk).buffcount < 2 as size_t {
                return TERMKEY_RES_NONE;
            }
            's_46: {
                match *(*tk)
                    .buffer
                    .offset((*tk).buffstart.wrapping_add(1 as size_t) as isize)
                    as ::core::ffi::c_int
                {
                    79 => return peekkey_ss3(tk, csi, 2 as size_t, key, force, nbytep),
                    80 => {}
                    93 | 95 => {}
                    91 => {
                        return peekkey_csi_csi(tk, csi, 2 as size_t, key, force, nbytep);
                    }
                    _ => {
                        break 's_46;
                    }
                }
                return peekkey_ctrlstring(tk, csi, 2 as size_t, key, force, nbytep);
            }
            return TERMKEY_RES_NONE;
        }
        143 => return peekkey_ss3(tk, csi, 1 as size_t, key, force, nbytep),
        144 | 157 => return peekkey_ctrlstring(tk, csi, 1 as size_t, key, force, nbytep),
        155 => return peekkey_csi_csi(tk, csi, 1 as size_t, key, force, nbytep),
        _ => {}
    }
    return TERMKEY_RES_NONE;
}
