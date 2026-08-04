#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::memory::ARENA_EMPTY;
use crate::src::nvim::types::{
    Array, BoolVarValue, CMD_index, Callback_data as C2Rust_Unnamed_22, ChannelPart,
    ChannelStreamType, Context, GRegFlags, ListLenSpecials, LuaRetMode, MotionType, Object,
    ProcType, SpecialVarValue, String_0, VarLockStatus, VarType, XDGVarType, cmd_addr_T, garray_T,
    object_data as C2Rust_Unnamed_16, size_t, uint64_t, uv__work, uv_loop_t, uv_req_type,
};

/// The generated builtin table: one row per builtin, plus the perfect-hash
/// lookup over their names. Regenerate with `just apigen`.
mod table;

/// The call frame a builtin body is handed.
mod args;
/// The dispatch layer and the wrappers whole groups of rows point at.
mod wrappers;

// One module per family of builtins. Each is rewritten and states its own
// imports; what is left in this file is the shared vocabulary they name.

mod call;
mod channel;
mod container;
mod context;
mod env;
mod input;
mod job;
mod marks;
mod math;
mod msgpack;
mod position;
mod reduce;
mod regexp;
mod region;
mod register;
mod runtime;
mod screen;
mod search;
mod strings;
mod timer;
mod variables;

pub use self::call::*;
pub use self::channel::*;
pub use self::container::*;
pub use self::context::*;
pub use self::env::*;
pub use self::input::*;
pub use self::job::*;
pub use self::marks::*;
pub use self::math::*;
pub use self::msgpack::*;
pub use self::position::*;
pub use self::reduce::*;
pub use self::regexp::*;
pub use self::region::*;
pub use self::register::*;
pub use self::runtime::*;
pub use self::screen::*;
pub use self::search::*;
pub use self::strings::*;
pub use self::timer::*;
pub use self::variables::*;
pub use self::wrappers::*;
unsafe extern "C" {
    fn uv_random(
        loop_0: *mut uv_loop_t,
        req: *mut uv_random_t,
        buf: *mut ::core::ffi::c_void,
        buflen: size_t,
        flags: ::core::ffi::c_uint,
        cb: uv_random_cb,
    ) -> ::core::ffi::c_int;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_random_s {
    pub data: *mut ::core::ffi::c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut ::core::ffi::c_void; 6],
    pub loop_0: *mut uv_loop_t,
    pub status: ::core::ffi::c_int,
    pub buf: *mut ::core::ffi::c_void,
    pub buflen: size_t,
    pub cb: uv_random_cb,
    pub work_req: uv__work,
}
pub type uv_random_cb = Option<
    unsafe extern "C" fn(
        *mut uv_random_t,
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
        size_t,
    ) -> (),
>;
pub type uv_random_t = uv_random_s;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MPACK_ERROR: C2Rust_Unnamed_13 = 2;
pub const MPACK_EOF: C2Rust_Unnamed_13 = 1;
pub const MPACK_OK: C2Rust_Unnamed_13 = 0;
pub const VAR_FIXED: VarLockStatus = 2;
pub const VAR_UNLOCKED: VarLockStatus = 0;
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const VAR_BLOB: VarType = 10;
pub const VAR_PARTIAL: VarType = 9;
pub const VAR_SPECIAL: VarType = 8;
pub const VAR_BOOL: VarType = 7;
pub const VAR_FLOAT: VarType = 6;
pub const VAR_DICT: VarType = 5;
pub const VAR_LIST: VarType = 4;
pub const VAR_FUNC: VarType = 3;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;
pub const VAR_UNKNOWN: VarType = 0;
pub const kProcTypePty: ProcType = 1;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const kListLenMayKnow: ListLenSpecials = -3;
pub const kListLenShouldKnow: ListLenSpecials = -2;
pub const kListLenUnknown: ListLenSpecials = -1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const VAR_TYPE_BLOB: C2Rust_Unnamed_33 = 10;
pub const VAR_TYPE_SPECIAL: C2Rust_Unnamed_33 = 7;
pub const VAR_TYPE_BOOL: C2Rust_Unnamed_33 = 6;
pub const VAR_TYPE_FLOAT: C2Rust_Unnamed_33 = 5;
pub const VAR_TYPE_DICT: C2Rust_Unnamed_33 = 4;
pub const VAR_TYPE_LIST: C2Rust_Unnamed_33 = 3;
pub const VAR_TYPE_FUNC: C2Rust_Unnamed_33 = 2;
pub const VAR_TYPE_STRING: C2Rust_Unnamed_33 = 1;
pub const VAR_TYPE_NUMBER: C2Rust_Unnamed_33 = 0;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const DI_FLAGS_LOCK: C2Rust_Unnamed_34 = 8;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_35 = 20;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_37 = 65;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_38 = 2;
pub type C2Rust_Unnamed_39 = ::core::ffi::c_uint;
pub const NSUBEXP: C2Rust_Unnamed_39 = 10;
pub const CMD_USER: CMD_index = -1;
pub const ADDR_LINES: cmd_addr_T = 0;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
pub type C2Rust_Unnamed_43 = ::core::ffi::c_uint;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_43 = 8;
pub const WILD_ALL: C2Rust_Unnamed_43 = 6;
pub type C2Rust_Unnamed_44 = ::core::ffi::c_uint;
pub const WILD_ICASE: C2Rust_Unnamed_44 = 256;
pub const WILD_SILENT: C2Rust_Unnamed_44 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_44 = 32;
pub const WILD_USE_NL: C2Rust_Unnamed_44 = 4;
pub const WILD_LIST_NOTFOUND: C2Rust_Unnamed_44 = 1;
pub type C2Rust_Unnamed_45 = ::core::ffi::c_uint;
pub const kCtxFuncs: C2Rust_Unnamed_45 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_45 = 16;
pub const kCtxGVars: C2Rust_Unnamed_45 = 8;
pub const kCtxBufs: C2Rust_Unnamed_45 = 4;
pub const kCtxJumps: C2Rust_Unnamed_45 = 2;
pub const kCtxRegs: C2Rust_Unnamed_45 = 1;
pub type C2Rust_Unnamed_46 = ::core::ffi::c_uint;
pub const BASE_LAST: C2Rust_Unnamed_46 = 255;
pub const BASE_NONE: C2Rust_Unnamed_46 = 0;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub const OP_NOP: C2Rust_Unnamed_62 = 0;
pub type SomeMatchType = ::core::ffi::c_uint;
pub const kSomeMatchStrPos: SomeMatchType = 4;
pub const kSomeMatchStr: SomeMatchType = 3;
pub const kSomeMatchList: SomeMatchType = 2;
pub const kSomeMatchEnd: SomeMatchType = 1;
pub const kSomeMatch: SomeMatchType = 0;
pub const VSE_NONE: C2Rust_Unnamed_57 = 0;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const MENU_ALL_MODES: C2Rust_Unnamed_58 = 127;
pub const CONV_NONE: C2Rust_Unnamed_53 = 0;
pub const GLV_READ_ONLY: C2Rust_Unnamed_67 = 16;
pub const GLV_NO_AUTOLOAD: C2Rust_Unnamed_67 = 4;
pub const TFN_NO_DEREF: C2Rust_Unnamed_66 = 8;
pub const TFN_NO_AUTOLOAD: C2Rust_Unnamed_66 = 4;
pub const TFN_QUIET: C2Rust_Unnamed_66 = 2;
pub const TFN_INT: C2Rust_Unnamed_66 = 1;
pub const VIM_GENERIC: C2Rust_Unnamed_54 = 0;
pub const VIM_WARNING: C2Rust_Unnamed_54 = 2;
pub const VIM_INFO: C2Rust_Unnamed_54 = 3;
pub const VIM_QUESTION: C2Rust_Unnamed_54 = 4;
pub const VIM_ERROR: C2Rust_Unnamed_54 = 1;
pub const DOCMD_KEYTYPED: C2Rust_Unnamed_56 = 8;
pub const DOCMD_REPEAT: C2Rust_Unnamed_56 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_56 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_56 = 2;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const YREG_YANK: C2Rust_Unnamed_61 = 1;
pub const FCERR_TOOMANY: C2Rust_Unnamed_55 = 1;
pub const FCERR_TOOFEW: C2Rust_Unnamed_55 = 2;
pub const FCERR_NONE: C2Rust_Unnamed_55 = 5;
pub const FCERR_UNKNOWN: C2Rust_Unnamed_55 = 0;
pub const FCERR_NOTMETHOD: C2Rust_Unnamed_55 = 8;
pub type C2Rust_Unnamed_53 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_54 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_55 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_56 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_57 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_58 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_61 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_62 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_66 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_67 = ::core::ffi::c_uint;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const VALID_VIRTCOL: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const CONTEXT_INIT: Context = Context {
    regs: STRING_INIT,
    jumps: STRING_INIT,
    bufs: STRING_INIT,
    gvars: STRING_INIT,
    funcs: ARRAY_DICT_INIT,
};
static e_string_list_or_blob_required: GlobalCell<[::core::ffi::c_char; 37]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 37], [::core::ffi::c_char; 37]>(
            *b"E1098: String, List or Blob required\0",
        )
    });
static e_missing_function_argument: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E1132: Missing function argument\0",
        )
    });
static dummy_ap: GlobalCell<::core::ffi::VaList<'static>> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], ::core::ffi::VaList<'static>>([0u8; 24])
});
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
