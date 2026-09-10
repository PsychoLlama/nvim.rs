#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::memory::ARENA_EMPTY;
use crate::types::{
    Array, ChannelPart, ChannelStreamType, Context, GArray, GRegFlags, LuaRetMode, MotionType,
    Object, ProcType, String_0, XDGVarType, size_t, uint64_t,
};

/// The generated builtin table: one row per builtin, plus the perfect-hash
/// lookup over their names. Regenerate with `just apigen`.
mod table;

/// The call frame a builtin body is handed.  Crate-visible because the
/// builtins are not all under this module: the fs family lives in
/// `eval::fs`, and there is no reason for it to grow a second `Args`.
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
pub const INTERNAL_CALL_MASK: uint64_t = 1_u64
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8_usize)
        .wrapping_sub(1_usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const VARNUMBER_MAX: ::core::ffi::c_long = INT64_MAX;
pub const VARNUMBER_MIN: ::core::ffi::c_long = INT64_MIN;
pub const GA_EMPTY_INIT_VALUE: GArray = GArray {
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
static e_string_list_or_blob_required: &::core::ffi::CStr = c"E1098: String, List or Blob required";
static e_missing_function_argument: &::core::ffi::CStr = c"E1132: Missing function argument";
static dummy_ap: GlobalCell<::core::ffi::VaList<'static>> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 24], ::core::ffi::VaList<'static>>([0u8; 24])
});
pub const TV_TRANSLATE: ::core::ffi::c_ulong = SIZE_MAX;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const SIGINT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;

#[cfg(test)]
mod arity_audit {
    //! The check the compiler cannot make: an `args[i]` past a builtin's
    //! *minimum* arity is a panic, not a type error, and the suites only
    //! reach the builtins some test happens to call.
    //!
    //! A source-level audit rather than a static one, because the row and
    //! the body are connected by a function *pointer*: the table's text is
    //! where the name, the arity and the body's identifier meet.

    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// The crate's `src` directory.
    fn src_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
    }

    /// `text` with comments and string, char and byte-string literals blanked
    /// to spaces, so a brace or an `args[` inside prose is not code.
    fn masked(text: &str) -> String {
        let b = text.as_bytes();
        let mut out = vec![b' '; b.len()];
        let (mut i, copy) = (0, |out: &mut Vec<u8>, a: usize, z: usize| {
            out[a..z].copy_from_slice(&b[a..z]);
        });
        while i < b.len() {
            match b[i] {
                b'/' if b.get(i + 1) == Some(&b'/') => {
                    while i < b.len() && b[i] != b'\n' {
                        out[i] = if b[i] == b'\r' { b'\r' } else { b' ' };
                        i += 1;
                    }
                }
                b'/' if b.get(i + 1) == Some(&b'*') => {
                    i += 2;
                    while i < b.len() && !(b[i] == b'*' && b.get(i + 1) == Some(&b'/')) {
                        i += 1;
                    }
                    i = (i + 2).min(b.len());
                }
                q @ (b'"' | b'\'') => {
                    i += 1;
                    while i < b.len() && b[i] != q {
                        i += if b[i] == b'\\' { 2 } else { 1 };
                    }
                    i += 1;
                }
                _ => {
                    // At least one byte, so that a `/` that opens no comment
                    // -- a division, a doc link -- cannot stall the walk.
                    let start = i;
                    i += 1;
                    while i < b.len() && !matches!(b[i], b'/' | b'"' | b'\'') {
                        i += 1;
                    }
                    copy(&mut out, start, i);
                }
            }
        }
        // Newlines are kept so that a failure can be located by eye.
        for (o, s) in out.iter_mut().zip(b) {
            if *s == b'\n' {
                *o = b'\n';
            }
        }
        String::from_utf8(out).expect("masking keeps the byte count")
    }

    /// The body of the `fn` whose header starts at `at`, braces included.
    fn body_at(masked: &str, at: usize) -> &str {
        let Some(open) = masked[at..].find('{').map(|i| at + i) else {
            return "";
        };
        let (mut depth, mut end) = (0usize, open);
        for (i, c) in masked[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = open + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        &masked[open..end]
    }

    /// Every `args[<literal>]` in `body`.
    fn indices(body: &str) -> Vec<usize> {
        let mut found = Vec::new();
        for (at, _) in body.match_indices("args[") {
            let rest = &body[at + "args[".len()..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() && rest[digits.len()..].starts_with(']') {
                found.push(digits.parse().expect("a run of digits"));
            }
        }
        found
    }

    /// Every builtin's body identifier and the smallest argument count the
    /// table lets a call have, read out of the generated rows.
    ///
    /// A row states its arity and then names its function, in that order and
    /// with nothing between them, however rustfmt wrapped it; the rows that
    /// state no arity are the `float()` shorthand, whose one argument no
    /// `args[i]` reads.
    fn rows() -> BTreeMap<String, usize> {
        let mut out = BTreeMap::new();
        for chunk in 1..=3 {
            let path = src_dir().join(format!("eval/funcs/table/table_{chunk}.rs"));
            let text = fs::read_to_string(&path).expect("the generated table");
            for (at, _) in text.match_indices("Arity::") {
                let rest = &text[at + "Arity::".len()..];
                let min: String = rest
                    .split_once('(')
                    .expect("an arity takes its bound in parentheses")
                    .1
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect();
                let row = &rest[..rest.find("Arity::").unwrap_or(rest.len())];
                let Some((_, tail)) = row.split_once("Some(") else {
                    continue;
                };
                let ident: String = tail.chars().take_while(|c| *c != ')').collect();
                out.insert(ident, min.parse().expect("a run of digits"));
            }
        }
        out
    }

    /// Every `.rs` under `src`, masked once.
    fn masked_sources() -> Vec<String> {
        fn walk(dir: &Path, into: &mut Vec<String>) {
            for entry in fs::read_dir(dir).expect("a readable directory") {
                let path = entry.expect("a directory entry").path();
                if path.is_dir() {
                    walk(&path, into);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    into.push(masked(&fs::read_to_string(&path).expect("a source file")));
                }
            }
        }
        let mut out = Vec::new();
        walk(&src_dir(), &mut out);
        out
    }

    /// A direct `args[i]` reaches past the minimum arity only where the body
    /// has already asked how many arguments there are; anywhere else it is a
    /// panic waiting for the first call that omits the optional argument.
    ///
    /// Not under Miri: this reads and masks every source file in the crate,
    /// which the interpreter turns from half a second into hours, and there
    /// is no memory to be unsafe about -- it is a text audit.
    #[cfg_attr(miri, ignore = "a source-text audit, and far too slow to interpret")]
    #[test]
    fn no_builtin_indexes_past_its_minimum_arity() {
        let rows = rows();
        let mut wrong = Vec::new();
        let mut seen = 0usize;
        // One pass per source, not per row: the table has some 570 of them.
        for text in masked_sources() {
            for (at, _) in text.match_indices("fn f") {
                let ident: String = text[at + 3..]
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !text[at + 3 + ident.len()..].starts_with('(') {
                    continue;
                }
                let Some(&min) = rows.get(&ident) else {
                    continue;
                };
                seen += 1;
                let body = body_at(&text, at);
                // "Has the body asked how many arguments there are?" -- the
                // three spellings a guard takes. A body that asks nowhere and
                // still indexes past the minimum is the bug this looks for.
                let guarded = ["args.len()", "args.is_empty()", "args.get(", "args.first()"]
                    .iter()
                    .any(|ask| body.contains(ask));
                for i in indices(body) {
                    if i >= min && !guarded {
                        wrong.push(format!("{ident}: args[{i}], minimum arity {min}"));
                    }
                }
            }
        }
        assert!(seen > 400, "only {seen} of the table's bodies were found");
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
}
