//! Running a user command: expanding the `<...>` codes of its replacement
//! text and handing the result to `do_cmdline()`.
//!
//! [`do_ucmd`] walks the replacement twice -- once with a null destination
//! to measure the result, once to write it -- and every helper below is
//! written to that shape: given a null buffer it answers the length it
//! *would* have written and touches nothing. [`uc_check_code`] is one
//! `<code>`, [`uc_mods`] the `<mods>` one (also called on its own, to
//! reproduce a command line's modifiers), and [`uc_split_args`] the
//! `<f-args>` splitter.
//!
//! The `q-`/`f-` prefixes are a quoting level rather than a code of their
//! own: `<q-args>` is the argument as one Vim string literal, `<f-args>` is
//! it split on unescaped whitespace into a comma-separated list of them.
//!
//! # Safety
//!
//! Everything here runs on the main thread from the Ex-command dispatcher,
//! with `eap` the command being executed and `cmd` its live [`ucmd_T`]. A
//! destination buffer is either null or has room for exactly what the
//! measuring pass answered. That is the contract the `unsafe fn`s share.
//!
//! Original: `src/nvim/usercmd.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    DOCMD_KEYTYPED, DOCMD_NOWAIT, DOCMD_VERBOSE, EX_KEEPSCRIPT, EX_NOSPC, NUL, Scope, ucmd_list,
};
use crate::ascii::ascii_iswhite;
use crate::charset::skipwhite;
use crate::ex_docmd::do_cmdline;
use crate::keycodes::{K_SPECIAL, KE_FILLER};
use crate::lua::executor::nlua_do_ucmd;
use crate::main::{cmdmod, current_sctx, curtab};
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmalloc};
use crate::os::cshim::memmove;
use crate::strings::vim_strchr;
use crate::types::{
    CMD_USER, CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS,
    CMOD_KEEPMARKS, CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, CMOD_NOAUTOCMD, CMOD_NOSWAPFILE,
    CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT, cmdmod_T, exarg_T, int64_t, size_t, ucmd_T,
};
use crate::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT, tabpage_index};
use ::libc::{strcat, strlen};
use core::ffi::{CStr, c_char, c_int};
use core::fmt::Write as _;
use core::{ptr, slice};

/// The second byte of an escaped `K_SPECIAL`.
const KS_SPECIAL: c_int = 254;

/// One argument of the whitespace-separated argument list, unescaped into
/// `buf`.
///
/// The iteration protocol is upstream's: `end` carries the position the
/// last call stopped at, and the answer is "that was the last one". Both
/// `nvim_parse_cmd()` and the Lua command callback split arguments with it.
///
/// # Safety
/// `arg` must be NUL-terminated with `arglen` bytes before the NUL, and
/// `buf` must have room for the longest single argument -- `arglen` always
/// is.
pub unsafe fn uc_split_args_iter(
    arg: *const c_char,
    arglen: size_t,
    end: *mut size_t,
    buf: *mut c_char,
    len: *mut size_t,
) -> bool {
    if arglen == 0 {
        return true;
    }
    // SAFETY: caller contract.
    let (arg, mut pos) = unsafe { (slice::from_raw_parts(arg.cast::<u8>(), arglen), *end) };
    // Upstream looks one byte past the argument, where the NUL is; nothing
    // past the end can be whitespace.
    let white_at = |i: usize| arg.get(i).is_some_and(|&b| ascii_iswhite(b as c_int));

    while pos < arglen && white_at(pos) {
        pos += 1;
    }

    let mut l = 0;
    while pos < arglen - 1 {
        // A backslash escapes a backslash or a separator, and nothing else.
        if arg[pos] == b'\\' && (arg[pos + 1] == b'\\' || white_at(pos + 1)) {
            pos += 1;
        }
        // SAFETY: caller contract; `l` never exceeds the bytes consumed.
        unsafe { *buf.add(l) = arg[pos] as c_char };
        l += 1;
        if white_at(pos + 1) {
            // SAFETY: caller contract.
            unsafe {
                *end = pos + 1;
                *len = l;
            }
            return false;
        }
        pos += 1;
    }
    if pos < arglen && !white_at(pos) {
        // SAFETY: as above.
        unsafe { *buf.add(l) = arg[pos] as c_char };
        l += 1;
    }
    // SAFETY: caller contract.
    unsafe { *len = l };
    true
}

/// At most how many arguments `arg` splits into -- the number of runs of
/// non-whitespace, which escaped separators can only reduce.
///
/// # Safety
/// `arg` must have `arglen` readable bytes.
pub unsafe fn uc_nargs_upper_bound(arg: *const c_char, arglen: size_t) -> size_t {
    // SAFETY: caller contract.
    let arg = unsafe { slice::from_raw_parts(arg.cast::<u8>(), arglen) };
    let mut was_white = true; // space before the first argument
    let mut nargs = 0;
    for &byte in arg {
        let is_white = ascii_iswhite(byte as c_int);
        if was_white && !is_white {
            nargs += 1;
        }
        was_white = is_white;
    }
    nargs
}

/// `<f-args>`: the arguments as a comma-separated list of Vim string
/// literals, allocated for the caller to free. `lenp` is given its length.
///
/// Upstream sizes the result in a first pass and fills it in a second; the
/// two have to agree byte for byte, so this builds the bytes once and takes
/// the length from them.
///
/// # Safety
/// Either `args`/`arglens` describe `argc` live arguments, or `args` is
/// null and `arg` is the NUL-terminated whole argument string.
unsafe fn uc_split_args(
    arg: *const c_char,
    args: *const *mut c_char,
    arglens: *const size_t,
    argc: size_t,
    lenp: *mut size_t,
) -> *mut c_char {
    let mut out = Vec::<u8>::new();
    out.push(b'"');
    if args.is_null() {
        // SAFETY: caller contract.
        unsafe { quote_line(arg, &mut out) };
    } else {
        for i in 0..argc {
            // SAFETY: caller contract.
            unsafe {
                let start = *args.add(i);
                quote_span(start, start.add(*arglens.add(i)), &mut out);
            }
            if i != argc - 1 {
                out.extend_from_slice(b"\", \"");
            }
        }
    }
    out.push(b'"');

    // SAFETY: the block is `out.len() + 1` bytes and is written whole.
    unsafe {
        let buf = xmalloc(out.len() + 1).cast::<c_char>();
        ptr::copy_nonoverlapping(out.as_ptr(), buf.cast::<u8>(), out.len());
        *buf.add(out.len()) = NUL;
        *lenp = out.len();
        buf
    }
}

/// The whole argument string, split on unescaped whitespace.
///
/// # Safety
/// `arg` must be NUL-terminated.
unsafe fn quote_line(arg: *const c_char, out: &mut Vec<u8>) {
    let mut p = arg;
    // SAFETY: caller contract; every step stays inside the string.
    unsafe {
        while *p != NUL {
            let (first, second) = (*p as u8, *p.add(1) as u8);
            if first == b'\\' && second == b'\\' {
                out.extend_from_slice(b"\\\\");
                p = p.add(2);
            } else if first == b'\\' && ascii_iswhite(second as c_int) {
                // An escaped separator stands for one literal separator.
                out.push(second);
                p = p.add(2);
            } else if first == b'\\' || first == b'"' {
                out.push(b'\\');
                out.push(first);
                p = p.add(1);
            } else if ascii_iswhite(first as c_int) {
                p = skipwhite(p);
                if *p == NUL {
                    break;
                }
                out.extend_from_slice(b"\", \"");
            } else {
                p = copy_char(p, out);
            }
        }
    }
}

/// One already-split argument, `start..end`, as the body of a literal.
///
/// # Safety
/// `start..end` must be a live range of one string.
unsafe fn quote_span(start: *const c_char, end: *const c_char, out: &mut Vec<u8>) {
    let mut p = start;
    // SAFETY: caller contract.
    unsafe {
        while p < end {
            if *p == b'\\' as c_char || *p == b'"' as c_char {
                out.push(b'\\');
                out.push(*p as u8);
                p = p.add(1);
            } else {
                p = copy_char(p, out);
            }
        }
    }
}

/// Copy the whole character at `p`, combining marks and all, and answer
/// what follows it.
///
/// # Safety
/// `p` must point at a character of a live string.
unsafe fn copy_char(p: *const c_char, out: &mut Vec<u8>) -> *const c_char {
    // SAFETY: caller contract.
    unsafe {
        let len = utfc_ptr2len(p) as usize;
        out.extend_from_slice(slice::from_raw_parts(p.cast::<u8>(), len));
        p.add(len)
    }
}

/// A small fixed buffer that [`write!`] can format into, kept
/// NUL-terminated so a C callee can read it as a string.
struct Scratch {
    bytes: [u8; 24],
    len: usize,
}

impl Scratch {
    fn new() -> Self {
        Scratch {
            bytes: [0; 24],
            len: 0,
        }
    }

    fn as_ptr(&self) -> *const c_char {
        self.bytes.as_ptr().cast::<c_char>()
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl core::fmt::Write for Scratch {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        // One byte is always kept for the terminator.
        let end = self.len + text.len();
        if end >= self.bytes.len() {
            return Err(core::fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Append one modifier to `buf`, separated from the previous one, and
/// answer how many bytes that took. A null `buf` only measures.
///
/// # Safety
/// `buf` must be null or NUL-terminated with room for `mod_str`, a
/// separating space and the NUL; `mod_str` must be NUL-terminated.
unsafe fn add_cmd_modifier(
    buf: *mut c_char,
    mod_str: *const c_char,
    multi_mods: &mut bool,
) -> size_t {
    // SAFETY: caller contract.
    let mut result = unsafe { strlen(mod_str) };
    if *multi_mods {
        result += 1;
    }
    if !buf.is_null() {
        // SAFETY: caller contract.
        unsafe {
            if *multi_mods {
                strcat(buf, c" ".as_ptr());
            }
            strcat(buf, mod_str);
        }
    }
    *multi_mods = true;
    result
}

/// The window-placement modifiers of `cmod`, in the order upstream emits
/// them. Answers how many bytes were added.
///
/// # Safety
/// As [`add_cmd_modifier`]; `cmod` and `multi_mods` must be live.
pub unsafe fn add_win_cmd_modifiers(
    buf: *mut c_char,
    cmod: *const cmdmod_T,
    multi_mods: *mut bool,
) -> size_t {
    // SAFETY: caller contract.
    let (cmod, multi_mods) = unsafe { (&*cmod, &mut *multi_mods) };
    let mut result = 0;
    let mut add = |name: *const c_char, present: bool| {
        if present {
            // SAFETY: caller contract.
            result += unsafe { add_cmd_modifier(buf, name, multi_mods) };
        }
    };
    // `:aboveleft`/`:leftabove`, `:belowright`/`:rightbelow`, `:botright`.
    add(
        c"aboveleft".as_ptr(),
        cmod.cmod_split & WSP_ABOVE as c_int != 0,
    );
    add(
        c"belowright".as_ptr(),
        cmod.cmod_split & WSP_BELOW as c_int != 0,
    );
    add(
        c"botright".as_ptr(),
        cmod.cmod_split & WSP_BOT as c_int != 0,
    );

    if cmod.cmod_tab > 0 {
        let tabnr = cmod.cmod_tab - 1;
        let mut text = Scratch::new();
        // For compatibility, a tab number that is `:tab`'s own default is
        // left off.
        if tabnr == tabpage_index(curtab.get()) {
            add(c"tab".as_ptr(), true);
        } else {
            let _ = write!(text, "{tabnr}tab");
            add(text.as_ptr(), true);
        }
    }

    add(c"topleft".as_ptr(), cmod.cmod_split & WSP_TOP as c_int != 0);
    add(
        c"vertical".as_ptr(),
        cmod.cmod_split & WSP_VERT as c_int != 0,
    );
    add(
        c"horizontal".as_ptr(),
        cmod.cmod_split & WSP_HOR as c_int != 0,
    );
    result
}

/// The modifiers `cmod` carries, as the text that would produce them.
/// A null `buf` only measures; `quote` wraps the result in `"`.
///
/// # Safety
/// As [`add_cmd_modifier`]; `cmod` must be live.
pub unsafe fn uc_mods(buf: *mut c_char, cmod: *const cmdmod_T, quote: bool) -> size_t {
    /// The modifiers that are nothing but a flag.
    static MOD_ENTRIES: [(c_int, &CStr); 12] = [
        (CMOD_BROWSE as c_int, c"browse"),
        (CMOD_CONFIRM as c_int, c"confirm"),
        (CMOD_HIDE as c_int, c"hide"),
        (CMOD_KEEPALT as c_int, c"keepalt"),
        (CMOD_KEEPJUMPS as c_int, c"keepjumps"),
        (CMOD_KEEPMARKS as c_int, c"keepmarks"),
        (CMOD_KEEPPATTERNS as c_int, c"keeppatterns"),
        (CMOD_LOCKMARKS as c_int, c"lockmarks"),
        (CMOD_NOSWAPFILE as c_int, c"noswapfile"),
        (CMOD_UNSILENT as c_int, c"unsilent"),
        (CMOD_NOAUTOCMD as c_int, c"noautocmd"),
        (CMOD_SANDBOX as c_int, c"sandbox"),
    ];
    // SAFETY: caller contract.
    let flags = unsafe { (*cmod).cmod_flags };
    // SAFETY: caller contract.
    let verbose = unsafe { (*cmod).cmod_verbose };
    let mut multi_mods = false;
    let mut result: size_t = if quote { 2 } else { 0 };

    // The opening quote goes in now; the closing one afterwards, at the
    // offset the total length gives.
    let body = if buf.is_null() {
        buf
    } else {
        // SAFETY: caller contract.
        unsafe {
            let body = if quote {
                *buf = b'"' as c_char;
                buf.add(1)
            } else {
                buf
            };
            *body = NUL;
            body
        }
    };

    for &(flag, name) in &MOD_ENTRIES {
        if flags & flag != 0 {
            // SAFETY: caller contract.
            result += unsafe { add_cmd_modifier(body, name.as_ptr(), &mut multi_mods) };
        }
    }
    if flags & CMOD_SILENT as c_int != 0 {
        let name = if flags & CMOD_ERRSILENT as c_int != 0 {
            c"silent!"
        } else {
            c"silent"
        };
        // SAFETY: caller contract.
        result += unsafe { add_cmd_modifier(body, name.as_ptr(), &mut multi_mods) };
    }
    if verbose > 0 {
        let value = verbose - 1;
        let mut text = Scratch::new();
        let name = if value == 1 {
            c"verbose".as_ptr()
        } else {
            let _ = write!(text, "{value}verbose");
            text.as_ptr()
        };
        // SAFETY: caller contract.
        result += unsafe { add_cmd_modifier(body, name, &mut multi_mods) };
    }
    // SAFETY: caller contract.
    result += unsafe { add_win_cmd_modifiers(body, cmod, &raw mut multi_mods) };

    if quote && !buf.is_null() {
        // `result` counts both quotes, so the body ends at `result - 1`.
        // SAFETY: caller contract; that byte is the last one written.
        unsafe { *buf.add(result - 1) = b'"' as c_char };
    }
    result
}

/// The `<...>` codes a replacement string may contain.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Code {
    Args,
    Bang,
    Count,
    Line1,
    Line2,
    Range,
    Mods,
    Register,
    Lt,
    /// Not a code at all: only the `<` is copied.
    None,
}

impl Code {
    /// The code `name` spells -- the text between the `<` and, inclusive,
    /// the `>`.
    ///
    /// Upstream compares with `STRNICMP(p, "args>", l)`, a prefix test, but
    /// `l` counts through the `>` and no prefix of `args>` ends in one, so
    /// only the whole word can match. The case-blindness is real: `<Args>`
    /// is `<args>`.
    fn parse(name: &[u8]) -> Code {
        const CODES: [(&[u8], Code); 10] = [
            (b"args>", Code::Args),
            (b"bang>", Code::Bang),
            (b"count>", Code::Count),
            (b"line1>", Code::Line1),
            (b"line2>", Code::Line2),
            (b"range>", Code::Range),
            (b"lt>", Code::Lt),
            (b"reg>", Code::Register),
            (b"register>", Code::Register),
            (b"mods>", Code::Mods),
        ];
        for (text, code) in CODES {
            if name.len() == text.len() && name.eq_ignore_ascii_case(text) {
                return code;
            }
        }
        Code::None
    }
}

/// The replacement for one code: bytes written, or only counted when the
/// destination is null.
struct Replacement {
    buf: *mut c_char,
    len: size_t,
}

impl Replacement {
    /// # Safety
    /// The destination must have room for everything put into it.
    unsafe fn put(&mut self, bytes: &[u8]) {
        if !self.buf.is_null() {
            // SAFETY: caller contract.
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    self.buf.add(self.len).cast(),
                    bytes.len(),
                );
            }
        }
        self.len += bytes.len();
    }

    /// `body` between a pair of `quote` bytes, or `body` alone.
    ///
    /// # Safety
    /// As [`Replacement::put`].
    unsafe fn quoted(&mut self, quote: Option<u8>, body: &[u8]) {
        // SAFETY: caller contract.
        unsafe {
            if let Some(q) = quote {
                self.put(&[q]);
            }
            self.put(body);
            if let Some(q) = quote {
                self.put(&[q]);
            }
        }
    }
}

/// How a code wants its expansion quoted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Quote {
    /// `<args>`: as it stands.
    None,
    /// `<q-args>`: one string literal.
    One,
    /// `<f-args>`: split into a list of string literals.
    Split,
}

/// Expand one `<code>` into `buf`, answering its length -- or `-1` when it
/// is not a code, in which case only the `<` has been copied.
///
/// # Safety
/// Module contract; `code` must have `len` readable bytes, and `split_buf`
/// and `split_len` must be [`do_ucmd`]'s cache.
unsafe fn uc_check_code(
    code: *const c_char,
    len: size_t,
    buf: *mut c_char,
    cmd: &ucmd_T,
    eap: &exarg_T,
    split_buf: *mut *mut c_char,
    split_len: *mut size_t,
) -> size_t {
    // SAFETY: caller contract.
    let inner = &unsafe { slice::from_raw_parts(code.cast::<u8>(), len) }[1..];
    let (quote, name) = match inner {
        [b'q' | b'Q', b'-', rest @ ..] => (Quote::One, rest),
        [b'f' | b'F', b'-', rest @ ..] => (Quote::Split, rest),
        rest => (Quote::None, rest),
    };
    let quotes = |byte: u8| (quote != Quote::None).then_some(byte);
    let mut out = Replacement { buf, len: 0 };

    match Code::parse(name) {
        // SAFETY: module contract.
        Code::Args => unsafe { expand_args(&mut out, eap, quote, split_buf, split_len) },
        Code::Bang => {
            let body: &[u8] = if eap.forceit != 0 { b"!" } else { b"" };
            // SAFETY: caller contract.
            unsafe { out.quoted(quotes(b'"'), body) };
            out.len
        }
        code @ (Code::Line1 | Code::Line2 | Code::Range | Code::Count) => {
            let num: int64_t = match code {
                Code::Line1 => eap.line1 as int64_t,
                Code::Line2 => eap.line2 as int64_t,
                Code::Range => eap.addr_count as int64_t,
                // `<count>` is the range's end, or the command's default.
                _ if eap.addr_count > 0 => eap.line2 as int64_t,
                _ => cmd.uc_def,
            };
            let mut text = Scratch::new();
            let _ = write!(text, "{num}");
            // SAFETY: caller contract.
            unsafe { out.quoted(quotes(b'"'), text.as_bytes()) };
            out.len
        }
        // SAFETY: caller contract.
        Code::Mods => unsafe { uc_mods(buf, cmdmod.ptr(), quote != Quote::None) },
        Code::Register => {
            let register = [eap.regname as u8];
            let body: &[u8] = if eap.regname != 0 { &register } else { b"" };
            // SAFETY: caller contract.
            unsafe { out.quoted(quotes(b'\''), body) };
            out.len
        }
        Code::Lt => {
            // SAFETY: caller contract.
            unsafe { out.put(b"<") };
            out.len
        }
        Code::None => {
            // Not recognised: copy the `<` and say so.
            // SAFETY: caller contract.
            unsafe { out.put(b"<") };
            !0
        }
    }
}

/// `<args>`, `<q-args>` or `<f-args>`.
///
/// # Safety
/// Module contract; `split_buf`/`split_len` are [`do_ucmd`]'s cache.
unsafe fn expand_args(
    out: &mut Replacement,
    eap: &exarg_T,
    quote: Quote,
    split_buf: *mut *mut c_char,
    split_len: *mut size_t,
) -> size_t {
    // SAFETY: module contract.
    let arg = unsafe { CStr::from_ptr(eap.arg).to_bytes() };
    if arg.is_empty() {
        if quote == Quote::One {
            // SAFETY: caller contract.
            unsafe { out.put(b"''") };
        }
        return out.len;
    }
    // A command declared to take a single argument does not split it, so
    // that `:Cmd %` works when `%` stands for "a b c".
    let quote = if eap.argt & EX_NOSPC != 0 && quote == Quote::Split {
        Quote::One
    } else {
        quote
    };
    // SAFETY: caller contract.
    unsafe {
        match quote {
            Quote::None => out.put(arg),
            Quote::One => {
                out.put(b"\"");
                for &byte in arg {
                    if byte == b'\\' || byte == b'"' {
                        out.put(b"\\");
                    }
                    out.put(&[byte]);
                }
                out.put(b"\"");
            }
            // Splitting is expensive, so it is done once and cached.
            Quote::Split => {
                if (*split_buf).is_null() {
                    *split_buf = uc_split_args(eap.arg, eap.args, eap.arglens, eap.argc, split_len);
                }
                out.put(slice::from_raw_parts((*split_buf).cast::<u8>(), *split_len));
            }
        }
    }
    out.len
}

/// Run one user command: expand its replacement text and execute it.
///
/// `preview` runs the `'inccommand'` callback instead, which only a Lua
/// command can have.
///
/// # Safety
/// Module contract; `eap` must be the command being executed.
pub unsafe fn do_ucmd(eap: *mut exarg_T, preview: bool) -> c_int {
    // SAFETY: module contract; `useridx` was set by `find_ucmd`.
    let cmd = unsafe {
        let scope = if (*eap).cmdidx == CMD_USER {
            Scope::Global
        } else {
            Scope::Buffer
        };
        &ucmd_list(scope.table())[(*eap).useridx as usize]
    };

    if preview {
        debug_assert!(cmd.uc_preview_luaref > 0, "cmd->uc_preview_luaref > 0");
        // SAFETY: module contract.
        return unsafe { nlua_do_ucmd(ptr::from_ref(cmd).cast_mut(), eap, true) };
    }
    if cmd.uc_luaref > 0 {
        // SAFETY: module contract.
        unsafe { nlua_do_ucmd(ptr::from_ref(cmd).cast_mut(), eap, false) };
        return 0;
    }

    // SAFETY: module contract.
    let buf = unsafe { expand_replacement(cmd, &*eap) };

    // The command body runs with the defining script's id, unless it asked
    // to keep the caller's.
    let saved = (cmd.uc_argt & EX_KEEPSCRIPT == 0).then(|| {
        let saved = current_sctx.get();
        current_sctx.with_mut(|sctx| sctx.sc_sid = cmd.uc_script_ctx.sc_sid);
        saved
    });
    // Nothing may touch `cmd` past here: the body can define a command,
    // which reallocates the table it points into.
    // SAFETY: module contract.
    unsafe {
        do_cmdline(
            buf,
            (*eap).ea_getline,
            (*eap).cookie,
            (DOCMD_VERBOSE | DOCMD_NOWAIT | DOCMD_KEYTYPED) as c_int,
        );
    }
    if let Some(saved) = saved {
        current_sctx.set(saved);
    }
    // SAFETY: the block is this function's.
    unsafe { xfree(buf.cast()) };
    0
}

/// `cmd`'s replacement text with every `<code>` expanded, allocated.
///
/// # Safety
/// Module contract.
unsafe fn expand_replacement(cmd: &ucmd_T, eap: &exarg_T) -> *mut c_char {
    let mut split_len: size_t = 0;
    let mut split_buf: *mut c_char = ptr::null_mut();
    // First round: measure with a null destination. Second: fill it.
    let mut buf: *mut c_char = ptr::null_mut();
    loop {
        // SAFETY: module contract.
        let pass = unsafe { expand_pass(cmd, eap, buf, &raw mut split_buf, &raw mut split_len) };
        // SAFETY: `tail` points into `uc_rep`, which is NUL-terminated.
        let tail_len = unsafe { strlen(pass.tail) };
        if buf.is_null() {
            // SAFETY: `xmalloc` never answers null.
            buf = unsafe { xmalloc(pass.total + tail_len + 1) }.cast::<c_char>();
            continue;
        }
        // SAFETY: the measuring pass sized the block for exactly this.
        unsafe { ptr::copy_nonoverlapping(pass.tail, pass.out, tail_len + 1) };
        break;
    }
    // SAFETY: this function owns it.
    unsafe { xfree(split_buf.cast()) };
    buf
}

/// Where one pass over the replacement text stopped.
struct Pass {
    /// The rest of the replacement, which holds no more codes.
    tail: *mut c_char,
    /// Where `tail` is to be copied. Null while measuring.
    out: *mut c_char,
    /// Bytes accounted for so far. Only meaningful while measuring.
    total: size_t,
}

/// One pass over `cmd`'s replacement text, writing into `buf` when it is
/// not null and only measuring when it is.
///
/// # Safety
/// Module contract.
unsafe fn expand_pass(
    cmd: &ucmd_T,
    eap: &exarg_T,
    buf: *mut c_char,
    split_buf: *mut *mut c_char,
    split_len: *mut size_t,
) -> Pass {
    let mut p = cmd.uc_rep;
    let mut q = buf;
    let mut totlen: size_t = 0;
    // SAFETY: module contract; every pointer walks `uc_rep`.
    unsafe {
        loop {
            let start = vim_strchr(p, b'<' as c_int);
            let end = if start.is_null() {
                ptr::null_mut()
            } else {
                vim_strchr(start.add(1), b'>' as c_int)
            };

            if !buf.is_null() && unescape_k_special(&mut p, &mut q, start, end) {
                continue;
            }
            if start.is_null() || end.is_null() {
                break;
            }
            let end = end.add(1); // include the '>'

            // Everything up to the '<' is copied as it stands.
            let len = start.offset_from(p) as size_t;
            if buf.is_null() {
                totlen += len;
            } else {
                memmove(q.cast(), p.cast(), len);
                q = q.add(len);
            }

            let mut len = uc_check_code(
                start,
                end.offset_from(start) as size_t,
                q,
                cmd,
                eap,
                split_buf,
                split_len,
            );
            if len == !0 {
                // Not a code: carry on after the '<'.
                p = start.add(1);
                len = 1;
            } else {
                p = end;
            }
            if buf.is_null() {
                totlen += len;
            } else {
                q = q.add(len);
            }
        }
    }
    Pass {
        tail: p,
        out: q,
        total: totlen,
    }
}

/// `K_SPECIAL` is stored escaped, as for a mapping, but `do_cmdline()` does
/// not undo that -- so undo it here, copying everything up to it and the
/// byte itself. Answers whether one was found and handled.
///
/// # Safety
/// Module contract; only called on the writing pass, so `q` is live.
unsafe fn unescape_k_special(
    p: &mut *mut c_char,
    q: &mut *mut c_char,
    start: *const c_char,
    end: *const c_char,
) -> bool {
    // SAFETY: caller contract.
    unsafe {
        let mut ksp = *p;
        while *ksp != NUL && *ksp as u8 as c_int != K_SPECIAL {
            ksp = ksp.add(1);
        }
        if *ksp as u8 as c_int != K_SPECIAL
            || !(start.is_null() || ksp.cast_const() < start || end.is_null())
            || *ksp.add(1) as u8 as c_int != KS_SPECIAL
            || *ksp.add(2) as c_int != KE_FILLER
        {
            return false;
        }
        let len = ksp.offset_from(*p) as size_t;
        if len > 0 {
            memmove((*q).cast(), (*p).cast(), len);
            *q = (*q).add(len);
        }
        **q = K_SPECIAL as c_char;
        *q = (*q).add(1);
        *p = ksp.add(3);
        true
    }
}
