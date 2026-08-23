#![deny(unsafe_op_in_unsafe_fn)]

use crate::global_cell::GlobalCell;
use crate::main::c_bytes;
use crate::memory::ARENA_EMPTY;
use crate::types::{
    Array, Callback_data, ChannelPart, ChannelStreamType, Context, GRegFlags, LuaRetMode,
    MotionType, Object, ProcType, String_0, XDGVarType, garray_T, object_data, size_t, uint64_t,
};

/// The generated builtin table: one row per builtin, plus the perfect-hash
/// lookup over their names. Regenerate with `just apigen`.
mod table;

/// The call frame a builtin body is handed.  Crate-visible because the
/// builtins are not all under this module: the fs family lives in
/// `eval::fs`, and there is no reason for it to grow a second `Args`.
pub(crate) mod args;
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
pub const MPACK_ERROR: ::core::ffi::c_uint = 2;
pub const MPACK_EOF: ::core::ffi::c_uint = 1;
pub const MPACK_OK: ::core::ffi::c_uint = 0;
pub const kProcTypePty: ProcType = 1;
pub const DI_FLAGS_LOCK: ::core::ffi::c_uint = 8;
pub const MAX_FUNC_ARGS: ::core::ffi::c_uint = 20;
pub const NUMBUFLEN: ::core::ffi::c_uint = 65;
pub const NSUBEXP: ::core::ffi::c_uint = 10;
pub const kChannelStreamProc: ChannelStreamType = 0;
pub const kChannelPartAll: ChannelPart = 4;
pub const kChannelPartRpc: ChannelPart = 3;
pub const kChannelPartStderr: ChannelPart = 2;
pub const kChannelPartStdout: ChannelPart = 1;
pub const kChannelPartStdin: ChannelPart = 0;
pub const kCtxFuncs: ::core::ffi::c_uint = 32;
pub const kCtxSFuncs: ::core::ffi::c_uint = 16;
pub const kCtxGVars: ::core::ffi::c_uint = 8;
pub const kCtxBufs: ::core::ffi::c_uint = 4;
pub const kCtxJumps: ::core::ffi::c_uint = 2;
pub const kCtxRegs: ::core::ffi::c_uint = 1;
pub const BASE_LAST: ::core::ffi::c_uint = 255;
pub const BASE_NONE: ::core::ffi::c_uint = 0;
pub const kMTUnknown: MotionType = -1;
pub const kMTBlockWise: MotionType = 2;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type SomeMatchType = ::core::ffi::c_uint;
pub const kSomeMatchStrPos: SomeMatchType = 4;
pub const kSomeMatchStr: SomeMatchType = 3;
pub const kSomeMatchList: SomeMatchType = 2;
pub const kSomeMatchEnd: SomeMatchType = 1;
pub const kSomeMatch: SomeMatchType = 0;
pub const VSE_NONE: ::core::ffi::c_uint = 0;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
pub const kGRegList: GRegFlags = 4;
pub const kGRegExprSrc: GRegFlags = 2;
pub const MENU_ALL_MODES: ::core::ffi::c_uint = 127;
pub const GLV_READ_ONLY: ::core::ffi::c_uint = 16;
pub const GLV_NO_AUTOLOAD: ::core::ffi::c_uint = 4;
pub const TFN_NO_DEREF: ::core::ffi::c_uint = 8;
pub const TFN_NO_AUTOLOAD: ::core::ffi::c_uint = 4;
pub const TFN_QUIET: ::core::ffi::c_uint = 2;
pub const TFN_INT: ::core::ffi::c_uint = 1;
pub const VIM_GENERIC: ::core::ffi::c_uint = 0;
pub const VIM_WARNING: ::core::ffi::c_uint = 2;
pub const VIM_INFO: ::core::ffi::c_uint = 3;
pub const VIM_QUESTION: ::core::ffi::c_uint = 4;
pub const VIM_ERROR: ::core::ffi::c_uint = 1;
pub const kXDGDataDirs: XDGVarType = 6;
pub const kXDGConfigDirs: XDGVarType = 5;
pub const kXDGRuntimeDir: XDGVarType = 4;
pub const kXDGStateHome: XDGVarType = 3;
pub const kXDGCacheHome: XDGVarType = 2;
pub const kXDGDataHome: XDGVarType = 1;
pub const kXDGConfigHome: XDGVarType = 0;
pub const YREG_YANK: ::core::ffi::c_uint = 1;
pub const FCERR_TOOMANY: ::core::ffi::c_uint = 1;
pub const FCERR_TOOFEW: ::core::ffi::c_uint = 2;
pub const FCERR_NONE: ::core::ffi::c_uint = 5;
pub const FCERR_UNKNOWN: ::core::ffi::c_uint = 0;
pub const FCERR_NOTMETHOD: ::core::ffi::c_uint = 8;
pub const INT64_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const ARENA_BLOCK_SIZE: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8_usize)
        .wrapping_sub(1_usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL_0,
};
pub const CONTEXT_INIT: Context = Context {
    regs: String_0::NULL,
    jumps: String_0::NULL,
    bufs: String_0::NULL,
    gvars: String_0::NULL,
    funcs: ARRAY_DICT_INIT,
};
static e_string_list_or_blob_required: [::core::ffi::c_char; 37] =
    c_bytes(b"E1098: String, List or Blob required\0");
static e_missing_function_argument: [::core::ffi::c_char; 33] =
    c_bytes(b"E1132: Missing function argument\0");
static dummy_ap: GlobalCell<::core::ffi::VaList<'static>> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], ::core::ffi::VaList<'static>>([0u8; 24])
});
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
