//! `eval/encode.c`: the shared half of the typval encoders.
//!
//! The four sinks upstream instantiates out of `typval_encode.c.h` live in the
//! three children beside this file — [`json`], [`msgpack`] and [`text`]
//! (`string()` and `:echo` are one `impl` there).  What stays here is what
//! they share, plus what belongs to no sink at all:
//!
//! - [`conv_error`], the failure path all three report through, which renders
//!   the walk's stack as `key foo, index 2, key bar`;
//! - [`convert_to_json_string`], JSON's string escaping — the one hook whose
//!   body is byte arithmetic rather than punctuation;
//! - [`encode_check_json_key`], the special-dictionary key test;
//! - the three `encode_tv2*` entry points; and
//! - the `readfile()`-style list codec ([`encode_list_write`],
//!   [`encode_read_from_list`], [`encode_vim_list_to_buf`]) that msgpack
//!   channels, `msgpackdump()` and `system()` read and write through.  Its
//!   one convention: a list item is a line, and a NUL inside a line is stored
//!   as a newline, because a Vimscript string cannot hold a newline.
//!
//! # Safety
//!
//! Every `unsafe fn` here forwards its caller's obligations; the `# Safety`
//! sections say which.  The recurring ones are that a `*mut typval_T` /
//! `*mut list_T` / `*const listitem_T` is live for the call and that nothing
//! removes an item from a list while one of these walks it — the encoders run
//! with no user code interleaved, which is what makes that hold.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::slice;

use crate::eval::typval::{
    GARRAY_EMPTY, tv_dict_find, tv_list_append_allocated_string, tv_list_first,
    tv_list_idx_of_item, tv_list_last, tv_list_len,
};
use crate::eval::typval_encode::{ConvPath, Flow, Frame, PartialStage};
use crate::eval::vars::eval_msgpack_type_lists;
use crate::garray::{Gap, ga_clear, ga_concat, ga_init};
use crate::global_cell::GlobalCell;
use crate::main::IObuff;
use crate::mbyte::{utf_char2len, utf_printable, utf_ptr2char, utf_ptr2len};
use crate::memory::{xfree, xmalloc, xmemdupz, xrealloc};
use crate::os::cshim::gettext;
use crate::semsg_c;
use crate::strings::vim_snprintf;
use crate::types::{
    ListReaderState, MessagePackType, VAR_DICT, VAR_FUNC, VAR_LIST, VAR_STRING, VAR_UNLOCKED,
    garray_T, list_T, listitem_T, ptrdiff_t, size_t, typval_T, typval_vval_union,
};
use ::libc::{abort, strlen};

// The sinks carved out of this module's `typval_encode.c.h` instantiations.
mod json;
use self::json::encode_vim_to_json;
mod msgpack;
pub use self::msgpack::*;
mod text;
use self::text::{encode_vim_to_echo, encode_vim_to_string};

pub const kMPString: MessagePackType = 4;
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NOTDONE: c_int = 2;
pub const IOSIZE: c_int = 1024 + 1;

/// The UTF-16 surrogate range, which a JSON `\u` escape has to spell a
/// character above the BMP with — and which a *string* may not contain.
pub const SURROGATE_HI_START: c_int = 0xd800;
pub const SURROGATE_HI_END: c_int = 0xdbff;
pub const SURROGATE_LO_START: c_int = 0xdc00;
pub const SURROGATE_LO_END: c_int = 0xdfff;
pub const SURROGATE_FIRST_CHAR: c_int = 0x10000;

pub static encode_bool_var_names: GlobalCell<[*const c_char; 2]> =
    GlobalCell::new([c"v:false".as_ptr(), c"v:true".as_ptr()]);
pub static encode_special_var_names: GlobalCell<[*const c_char; 1]> =
    GlobalCell::new([c"v:null".as_ptr()]);

/// Set once a `string()`/`echo` dump has reported a self-reference, so the
/// user is told once rather than once per cycle.
pub(crate) static did_echo_string_emsg: GlobalCell<bool> = GlobalCell::new(false);

/// `_()`: the translation of a message, which is always a literal here.
#[inline(always)]
fn tr(msg: &'static CStr) -> *const c_char {
    // SAFETY: `gettext` only reads the NUL-terminated string it is handed.
    unsafe { gettext(msg.as_ptr()) }
}

/// A fresh byte `garray_T` grown 80 at a time: every text encoder's output.
fn text_garray() -> garray_T {
    let mut ga = GARRAY_EMPTY;
    // SAFETY: `ga` is a local array header and `ga_init` only writes it.
    unsafe { ga_init(&raw mut ga, size_of::<c_char>() as c_int, 80) };
    ga
}

/// The string `li` holds; a NULL one is an empty line.
///
/// # Safety
/// `li` must be a live list item whose value is a `VAR_STRING`.
#[inline(always)]
unsafe fn item_string(li: *const listitem_T) -> *mut c_char {
    unsafe { (*li).li_tv.vval.v_string }
}

/// `strlen` of [`item_string`], with a NULL string reading as zero.
///
/// # Safety
/// As [`item_string`].
#[inline(always)]
unsafe fn item_strlen(li: *const listitem_T) -> size_t {
    let s = unsafe { item_string(li) };
    if s.is_null() { 0 } else { unsafe { strlen(s) } }
}

/// The items of `list`, front to back.  A NULL list is an empty one.
///
/// # Safety
/// `list` must be live, and nothing may add to or remove from it while the
/// iterator is alive.
unsafe fn items(list: *const list_T) -> impl Iterator<Item = *const listitem_T> {
    let mut li = if list.is_null() {
        core::ptr::null()
    } else {
        unsafe { (*list).lv_first }
    };
    core::iter::from_fn(move || {
        let cur = li;
        if cur.is_null() {
            return None;
        }
        li = unsafe { (*cur).li_next };
        Some(cur)
    })
}

/// Store a line the way a `readfile()`-style list does: NUL bytes become
/// newlines, because a Vimscript string can hold the former and not the
/// latter.
fn store_nuls_as_newlines(line: &mut [u8]) {
    for byte in line {
        if *byte == 0 {
            *byte = b'\n';
        }
    }
}

/// Append `line` to the string `li` already holds, which grows in place.
///
/// # Safety
/// `li` must be a live list item whose value is a `VAR_STRING` this may take
/// ownership of and replace.
unsafe fn extend_item(li: *mut listitem_T, line: &[u8]) {
    unsafe {
        let old_len = item_strlen(li);
        let grown =
            xrealloc(item_string(li).cast::<c_void>(), old_len + line.len() + 1).cast::<c_char>();
        (*li).li_tv.vval.v_string = grown;
        let tail = slice::from_raw_parts_mut(grown.add(old_len).cast::<u8>(), line.len() + 1);
        tail[..line.len()].copy_from_slice(line);
        tail[line.len()] = 0;
        store_nuls_as_newlines(&mut tail[..line.len()]);
    }
}

/// `line` as a fresh NUL-terminated allocation the list takes over.
fn own_line(line: &[u8]) -> *mut c_char {
    // SAFETY: `line` is readable for its own length; `xmemdupz` allocates one
    // byte more and terminates.
    let owned = unsafe { xmemdupz(line.as_ptr().cast::<c_void>(), line.len()).cast::<c_char>() };
    // SAFETY: the allocation is `line.len()` bytes plus the terminator.
    let copied = unsafe { slice::from_raw_parts_mut(owned.cast::<u8>(), line.len()) };
    store_nuls_as_newlines(copied);
    owned
}

/// Msgpack callback for writing to a `readfile()`-style list.
///
/// Each newline in `buf` starts a new item; whatever came before the first
/// one continues the item already there.
///
/// # Safety
/// `data` must be a live `list_T *` and `buf` must be readable for `len`
/// bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn encode_list_write(data: *mut c_void, buf: *const c_char, len: size_t) {
    if len == 0 {
        return;
    }
    let list = data.cast::<list_T>();
    // SAFETY: the caller's promise about `buf` and `len`.
    let bytes = unsafe { slice::from_raw_parts(buf.cast::<u8>(), len) };

    /// The index just past the next newline, and the line before it.
    fn split(bytes: &[u8], from: usize) -> (&[u8], usize) {
        let rest = &bytes[from..];
        let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
        (&rest[..end], from + end + 1)
    }

    // SAFETY: `list` is the caller's, and nothing runs between these calls
    // that could touch it.
    let last = unsafe { tv_list_last(list) };
    let mut at = 0;
    if !last.is_null() {
        // Continue the last item, unless the write starts with a newline.
        let (line, next) = split(bytes, 0);
        if !line.is_empty() {
            // SAFETY: `last` is this list's own final item.
            unsafe { extend_item(last, line) };
        }
        at = next;
    }
    while at < len {
        let (line, next) = split(bytes, at);
        let owned = if line.is_empty() {
            core::ptr::null_mut()
        } else {
            own_line(line)
        };
        // SAFETY: `list` is live and takes over `owned`.
        unsafe { tv_list_append_allocated_string(list, owned) };
        at = next;
    }
    if at == len {
        // The write ended on a newline, so it opened one more empty item.
        // SAFETY: as above.
        unsafe { tv_list_append_allocated_string(list, core::ptr::null_mut()) };
    }
}

/// Report a failed dump, naming the path down to the value that failed.
///
/// `msg` must carry exactly two `%s`: the object being dumped, then the path
/// — "key foo, index 2, key bar" — which this builds out of the walk's stack.
/// Always answers [`Flow::Fail`], because that is all its callers do with it.
///
/// # Safety
/// `msg` must be a NUL-terminated format string of that shape, and `path`
/// must describe a walk that is still in progress.
pub(crate) unsafe fn conv_error(msg: *const c_char, path: &ConvPath) -> Flow {
    let idx_msg = tr(c"index %i");
    let partial_arg_msg = tr(c"partial");
    let partial_arg_i_msg = tr(c"argument %i");
    let partial_self_msg = tr(c"partial self dictionary");

    let mut msg_ga = text_garray();
    let iobuff = IObuff.ptr().cast::<c_char>();

    /// Everything the arms below share: format into `IObuff` and append it.
    ///
    /// # Safety
    /// `fmt` must match the arguments, and `msg_ga` must be a byte garray.
    macro_rules! append_formatted {
        ($fmt:expr $(, $arg:expr)*) => {
            // SAFETY: `iobuff` is the shared `IOSIZE`-byte scratch and the
            // format strings here are this function's own literals.
            unsafe {
                vim_snprintf(iobuff, IOSIZE as size_t, $fmt $(, $arg)*);
                ga_concat(&raw mut msg_ga, iobuff);
            }
        };
    }

    for (i, frame) in path.stack.iter().enumerate() {
        if i != 0 {
            Gap(&mut msg_ga).concat(b", ");
        }
        match frame.frame {
            Frame::Dict { dict, hi, .. } => {
                // The key most recently handed out, which is the slot before
                // the one the walk is now standing on.
                // SAFETY: the frame's dictionary is live and `hi` is either
                // NULL or one past a slot of its hash table.
                let key = unsafe {
                    let hi = if hi.is_null() {
                        (*dict).dv_hashtab.ht_array
                    } else {
                        hi.sub(1)
                    };
                    let mut key_tv = typval_T {
                        v_type: VAR_STRING,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union {
                            v_string: (*hi).hi_key,
                        },
                    };
                    encode_tv2string(&raw mut key_tv, core::ptr::null_mut())
                };
                append_formatted!(tr(c"key %s"), key);
                // SAFETY: `encode_tv2string` hands back an owned buffer.
                unsafe { xfree(key.cast::<c_void>()) };
            }
            Frame::List { list, li } | Frame::Pairs { list, li } => {
                // The item most recently handed out: one back from `li`, or
                // the last one once the walk has run off the end.
                // SAFETY: the frame's list is live and `li` is one of its
                // items or NULL.
                let (idx, li) = unsafe {
                    let idx = if li == tv_list_first(list) {
                        0
                    } else if li.is_null() {
                        tv_list_len(list) - 1
                    } else {
                        tv_list_idx_of_item(list, (*li).li_prev)
                    };
                    let li = if li.is_null() {
                        tv_list_last(list)
                    } else {
                        (*li).li_prev
                    };
                    (idx, li)
                };
                // SAFETY: `li` is an item of the frame's list, or NULL.
                let pair_key = unsafe {
                    let pairs = matches!(frame.frame, Frame::Pairs { .. });
                    if !pairs
                        || li.is_null()
                        || ((*li).li_tv.v_type != VAR_LIST
                            && tv_list_len((*li).li_tv.vval.v_list) <= 0)
                    {
                        None
                    } else {
                        // A special map's item is a [key, value] pair, so the
                        // path can name the key rather than the index.
                        let first_item = tv_list_first((*li).li_tv.vval.v_list);
                        let mut key_tv = (*first_item).li_tv;
                        Some(encode_tv2echo(&raw mut key_tv, core::ptr::null_mut()))
                    }
                };
                match pair_key {
                    None => append_formatted!(idx_msg, idx),
                    Some(key) => {
                        append_formatted!(tr(c"key %s at index %i from special map"), key, idx);
                        // SAFETY: `encode_tv2echo` hands back an owned buffer.
                        unsafe { xfree(key.cast::<c_void>()) };
                    }
                }
            }
            Frame::Partial { stage, .. } => {
                let text = match stage {
                    // The walk pushes a partial already past its arguments.
                    // SAFETY: unreachable; `abort` returns `!`.
                    PartialStage::Args => unsafe { abort() },
                    PartialStage::Self_ => partial_arg_msg,
                    PartialStage::End => partial_self_msg,
                };
                // SAFETY: both texts are NUL-terminated translations.
                unsafe { ga_concat(&raw mut msg_ga, text) };
            }
            Frame::PartialArgs { arg, argv, .. } => {
                // SAFETY: `arg` and `argv` point into one argument vector.
                let idx = unsafe { arg.offset_from(argv) } as c_int - 1;
                append_formatted!(partial_arg_i_msg, idx);
            }
        }
    }

    // SAFETY: `msg` is the caller's two-`%s` format; the path is either the
    // rendered stack or the literal below.
    unsafe {
        semsg_c!(
            msg,
            gettext(path.objname),
            if path.stack.is_empty() {
                tr(c"itself")
            } else {
                msg_ga.ga_data.cast::<c_char>()
            },
        );
        ga_clear(&raw mut msg_ga);
    }
    Flow::Fail
}

/// Convert a `readfile()`-style list to a buffer with length.
///
/// The buffer is **not** NUL-terminated: it is exactly `*ret_len` bytes, and
/// the caller frees it.  Answers false — writing neither output — when any
/// item is not a string.
///
/// # Safety
/// `list` must be live, and `ret_len`/`ret_buf` must be writable.
pub unsafe fn encode_vim_list_to_buf(
    list: *const list_T,
    ret_len: *mut size_t,
    ret_buf: *mut *mut c_char,
) -> bool {
    let mut len: size_t = 0;
    // SAFETY: the caller's promise about `list`.
    for li in unsafe { items(list) } {
        // SAFETY: `li` is one of the list's items.
        if unsafe { (*li).li_tv.v_type } != VAR_STRING {
            return false;
        }
        // One separator per item, so the total is one too many.
        len += 1 + unsafe { item_strlen(li) };
    }
    len = len.saturating_sub(1);
    // SAFETY: the caller's promise about the two out parameters.
    unsafe { *ret_len = len };
    if len == 0 {
        // SAFETY: as above.
        unsafe { *ret_buf = core::ptr::null_mut() };
        return true;
    }
    // SAFETY: `list` is live and non-empty, so it has a first item.
    let mut lrstate = unsafe { encode_init_lrstate(list) };
    let buf = unsafe { xmalloc(len).cast::<c_char>() };
    let mut read_bytes: size_t = 0;
    // SAFETY: `buf` is `len` writable bytes and `lrstate` walks `list`.
    let ret = unsafe { encode_read_from_list(&raw mut lrstate, buf, len, &raw mut read_bytes) };
    if ret != OK {
        // Every item was checked above, so the reader cannot refuse one.
        // SAFETY: unreachable.
        unsafe { abort() };
    }
    debug_assert!(len == read_bytes, "len == read_bytes");
    // SAFETY: the caller's promise about `ret_buf`.
    unsafe { *ret_buf = buf };
    true
}

/// Read bytes out of a `readfile()`-style list into `buf`.
///
/// `state` is advanced to where reading stopped.  Answers [`OK`] when the
/// list ran out, [`NOTDONE`] when the buffer did, and [`FAIL`] on an item
/// that is not a string — the stored newlines turning back into NULs on the
/// way, which is what [`encode_list_write`] wrote them for.
///
/// # Safety
/// `state` must describe a position in a live list, `buf` must be writable
/// for `nbuf` bytes and `read_bytes` must be writable.
pub unsafe fn encode_read_from_list(
    state: *mut ListReaderState,
    buf: *mut c_char,
    nbuf: size_t,
    read_bytes: *mut size_t,
) -> c_int {
    // SAFETY: the caller's promises about `buf`/`nbuf` and `state`.
    let out = unsafe { slice::from_raw_parts_mut(buf.cast::<u8>(), nbuf) };
    let state = unsafe { &mut *state };
    let mut p = 0;
    while p < nbuf {
        debug_assert!(
            state.li_length == 0 || !unsafe { item_string(state.li) }.is_null(),
            "state->li_length == 0 || TV_LIST_ITEM_TV(state->li)->vval.v_string != NULL"
        );
        // `i` and `state.offset` step together; upstream keeps both because
        // the loop it wrote reads one and advances the other.
        let mut i = state.offset;
        while i < state.li_length && p < nbuf {
            // SAFETY: the item holds at least `li_length` bytes and `offset`
            // is below that.
            let ch = unsafe { *item_string(state.li).add(state.offset) } as u8;
            state.offset += 1;
            out[p] = if ch == b'\n' { 0 } else { ch };
            p += 1;
            i += 1;
        }
        if p < nbuf {
            // SAFETY: `state.li` is a live item of the walked list.
            state.li = unsafe { (*state.li).li_next };
            if state.li.is_null() {
                // SAFETY: the caller's promise about `read_bytes`.
                unsafe { *read_bytes = p };
                return OK;
            }
            out[p] = b'\n';
            p += 1;
            // SAFETY: as above.
            if unsafe { (*state.li).li_tv.v_type } != VAR_STRING {
                unsafe { *read_bytes = p };
                return FAIL;
            }
            state.offset = 0;
            // SAFETY: the item was just checked to hold a string.
            state.li_length = unsafe { item_strlen(state.li) };
        }
    }
    // SAFETY: the caller's promise about `read_bytes`.
    unsafe { *read_bytes = nbuf };
    // SAFETY: `state.li` is a live item.
    if state.offset < state.li_length || !unsafe { (*state.li).li_next }.is_null() {
        NOTDONE
    } else {
        OK
    }
}

/// Start reading a `readfile()`-style list from its first item.
///
/// # Safety
/// `list` must be live and must have at least one item.
pub unsafe fn encode_init_lrstate(list: *const list_T) -> ListReaderState {
    // SAFETY: the caller's promise; the first item holds a string or NULL.
    let li = unsafe { tv_list_first(list) };
    ListReaderState {
        list,
        li,
        offset: 0,
        li_length: unsafe { item_strlen(li) },
    }
}

const E474_BAD_UTF8: &CStr =
    c"E474: String \"%.*s\" contains byte that does not start any UTF-8 character";
const E474_SURROGATE: &CStr =
    c"E474: UTF-8 string contains code point which belongs to a surrogate pair: %.*s";

/// The hexadecimal digits a `\uNNNN` escape is spelled with.
const XDIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// Upstream's `escapes[]`: the two-character escape for every character that
/// has one, indexed by the character itself.  A zero first byte means none.
static JSON_ESCAPES: [[u8; 2]; 0x5d] = {
    let mut table = [[0u8; 2]; 0x5d];
    table[8] = *b"\\b";
    table[9] = *b"\\t";
    table[10] = *b"\\n";
    table[12] = *b"\\f";
    table[13] = *b"\\r";
    table[b'"' as usize] = *b"\\\"";
    table[b'\\' as usize] = *b"\\\\";
    table
};

/// The two-character escape JSON spells `ch` with, if it has one.
#[inline(always)]
fn json_escape_of(ch: c_int) -> Option<&'static [u8; 2]> {
    // A negative `ch` wraps to a huge index and misses, as it should.
    let escape = JSON_ESCAPES.get(ch as usize)?;
    (escape[0] != 0).then_some(escape)
}

/// Upstream's `ENCODE_RAW`: may `ch` go into the output as itself?
///
/// Everything else becomes `\uNNNN`, so that a JSON value stays displayable
/// outside Neovim.  0x7F is caught by `utf_printable`, not by the range.
#[inline(always)]
fn json_encode_raw(ch: c_int) -> bool {
    ch >= 0x20 && utf_printable(ch)
}

/// `\uNNNN` for a code unit.
#[inline(always)]
fn json_unicode_escape(unit: c_int) -> [u8; 6] {
    let digit = |shift: u32| XDIGITS[((unit >> (4 * shift)) & 0xf) as usize];
    [b'\\', b'u', digit(3), digit(2), digit(1), digit(0)]
}

/// The UTF-16 surrogate pair for a character above the BMP.
///
/// The low half is upstream's: it counts up from `SURROGATE_LO_END`, not from
/// `SURROGATE_LO_START`, so `U+10000` would come out as `𐏿` rather
/// than `𐀀`.  Kept as it is because the arm is **unreachable**:
/// reaching it needs a character above the BMP that `utf_printable` refuses,
/// and its table stops at `U+FFFF`.  Should that table ever grow, this is
/// where to look.
#[inline(always)]
fn json_surrogate_pair(ch: c_int) -> (c_int, c_int) {
    let tmp = ch - SURROGATE_FIRST_CHAR;
    (
        SURROGATE_HI_START + ((tmp >> 10) & 0x3ff),
        SURROGATE_LO_END + (tmp & 0x3ff),
    )
}

/// `semsg(_(msg), (int)tail.len(), tail)` — the two `%.*s` refusals below.
fn err_tail(msg: &'static CStr, tail: &[u8]) {
    // SAFETY: `%.*s` reads exactly the length it is given, and `tail` is
    // readable for its own.
    unsafe { semsg_c!(tr(msg), tail.len() as c_int, tail.as_ptr()) };
}

/// The bytes being escaped into a JSON string.
///
/// Deliberately **not** a slice.  Upstream measures each character with
/// `utf_ptr2char`/`utf_ptr2len`, which read as many bytes as the lead byte
/// promises and so read *past* `len` when the last character is a truncated
/// multi-byte sequence.  For a `VAR_STRING` that byte is the terminating NUL
/// and nothing comes of it, but a special `{'_TYPE': string}` value arrives
/// in a buffer [`encode_vim_list_to_buf`] sized exactly, and there the
/// over-read is real.  It is upstream's behaviour, it is reproduced rather
/// than fixed, and it is why the three accessors are `unsafe`.
struct Utf8 {
    at: *const u8,
    len: usize,
}

impl Utf8 {
    /// The code point at `i`, or the byte's own value where no complete
    /// sequence starts there.
    ///
    /// # Safety
    /// `i` must be below `len`, and the bytes the lead byte at `i` promises
    /// must be readable — see the type's own note.
    #[inline(always)]
    unsafe fn char_at(&self, i: usize) -> c_int {
        unsafe { utf_ptr2char(self.at.add(i).cast::<c_char>()) }
    }

    /// How many bytes the character at `i` occupies.
    ///
    /// # Safety
    /// As [`Self::char_at`].
    #[inline(always)]
    unsafe fn len_at(&self, i: usize) -> usize {
        unsafe { utf_ptr2len(self.at.add(i).cast::<c_char>()) as usize }
    }

    /// `n` bytes from `i`.
    ///
    /// # Safety
    /// As [`Self::char_at`]: `n` is a measured character length, which may
    /// reach past `len`.
    #[inline(always)]
    unsafe fn run(&self, i: usize, n: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.at.add(i), n) }
    }

    /// Everything from `i` to the end, for an error message.
    ///
    /// # Safety
    /// `i` must be below `len`.
    #[inline(always)]
    unsafe fn tail(&self, i: usize) -> &[u8] {
        unsafe { slice::from_raw_parts(self.at.add(i), self.len - i) }
    }
}

/// How long the escaped form of `text` will be, or `None` once the refusal
/// has been reported.
///
/// This is upstream's first pass: the one that decides whether the string can
/// be JSON at all.
///
/// # Safety
/// As [`Utf8`].
#[inline(always)]
unsafe fn json_escaped_len(text: &Utf8) -> Option<usize> {
    let mut str_len = 0;
    let mut i = 0;
    while i < text.len {
        let ch = unsafe { text.char_at(i) };
        let shift = if ch == 0 {
            1
        } else {
            unsafe { text.len_at(i) }
        };
        debug_assert!(shift > 0, "shift > 0");
        i += shift;
        if json_escape_of(ch).is_some() {
            str_len += 2;
        } else if ch > 0x7f && shift == 1 {
            err_tail(E474_BAD_UTF8, unsafe { text.tail(i - shift) });
            return None;
        } else if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&ch)
            || (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&ch)
        {
            err_tail(E474_SURROGATE, unsafe { text.tail(i - shift) });
            return None;
        } else if json_encode_raw(ch) {
            str_len += shift;
        } else {
            // Six bytes per `\uNNNN`, and twice that for a surrogate pair.
            str_len += 6 * (1 + usize::from(ch >= SURROGATE_FIRST_CHAR));
        }
    }
    Some(str_len)
}

/// Convert a string to a JSON string literal, quotes included.
///
/// Two passes, exactly as upstream: the first sizes the result and is where
/// the refusals happen, the second writes it.  A NULL buffer is `""`.
///
/// # Safety
/// `gap` must be a live byte garray, and `buf` must be NULL or readable for
/// `len` bytes — with the over-read [`Utf8`] describes.
#[inline(always)]
pub(crate) unsafe fn convert_to_json_string(
    gap: *mut garray_T,
    buf: *const c_char,
    len: size_t,
) -> c_int {
    // SAFETY: the caller's promise about `gap`.
    let mut gap = Gap(unsafe { &mut *gap });
    if buf.is_null() {
        gap.concat(b"\"\"");
        return OK;
    }
    let text = Utf8 {
        at: buf.cast::<u8>(),
        len,
    };
    // SAFETY: forwarded to the caller's promise about `buf`.
    let Some(str_len) = (unsafe { json_escaped_len(&text) }) else {
        return FAIL;
    };
    gap.append(b'"');
    gap.grow(str_len as c_int);
    let mut i = 0;
    while i < text.len {
        let ch = unsafe { text.char_at(i) };
        // The write pass measures the *character*, not the bytes; the two
        // agree except at a NUL, which is one byte and not one character.
        let shift = if ch == 0 {
            1
        } else {
            utf_char2len(ch) as usize
        };
        debug_assert!(shift > 0, "shift > 0");
        debug_assert!(
            ch == 0 || shift == unsafe { text.len_at(i) },
            "ch == 0 || shift == ((size_t)utf_ptr2len(utf_buf + i))"
        );
        if let Some(escape) = json_escape_of(ch) {
            gap.concat(escape);
        } else if json_encode_raw(ch) {
            gap.concat(unsafe { text.run(i, shift) });
        } else if ch < SURROGATE_FIRST_CHAR {
            gap.concat(&json_unicode_escape(ch));
        } else {
            let (hi, lo) = json_surrogate_pair(ch);
            gap.concat(&json_unicode_escape(hi));
            gap.concat(&json_unicode_escape(lo));
        }
        i += shift;
    }
    gap.append(b'"');
    OK
}

/// May `tv` be a key in `json_encode()`'s output?
///
/// A plain string may.  So may a `{'_TYPE': v:msgpack_types.string, '_VAL':
/// [...]}` special dictionary, provided every part of its `_VAL` is a string
/// — that is how a key holding a NUL is spelled.
///
/// # Safety
/// `tv` must be live, as must anything it points at.
pub unsafe fn encode_check_json_key(tv: *const typval_T) -> bool {
    // SAFETY: the caller's promise about `tv`.
    let tv = unsafe { &*tv };
    if tv.v_type == VAR_STRING {
        return true;
    }
    if tv.v_type != VAR_DICT {
        return false;
    }
    // SAFETY: a `VAR_DICT` holds a live dictionary.
    let spdict = unsafe { tv.vval.v_dict };
    if unsafe { (*spdict).dv_hashtab.ht_used } != 2 {
        return false;
    }
    // SAFETY: `spdict` is live and both keys are NUL-terminated literals.
    let (type_di, val_di) = unsafe {
        (
            tv_dict_find(spdict, c"_TYPE".as_ptr(), 5 as ptrdiff_t),
            tv_dict_find(spdict, c"_VAL".as_ptr(), 4 as ptrdiff_t),
        )
    };
    if type_di.is_null() {
        return false;
    }
    // SAFETY: a non-NULL find answers a live item of `spdict`.
    let type_tv = unsafe { &(*type_di).di_tv };
    if type_tv.v_type != VAR_LIST
        || unsafe { type_tv.vval.v_list }
            != eval_msgpack_type_lists.get()[kMPString as usize] as *mut list_T
        || val_di.is_null()
    {
        return false;
    }
    // SAFETY: as `type_di`.
    let val_tv = unsafe { &(*val_di).di_tv };
    if val_tv.v_type != VAR_LIST {
        return false;
    }
    // SAFETY: a `VAR_LIST` holds a live list or NULL, and nothing runs
    // between the items.
    for li in unsafe { items(val_tv.vval.v_list) } {
        if unsafe { (*li).li_tv.v_type } != VAR_STRING {
            return false;
        }
    }
    true
}

/// Finish one of the three `encode_tv2*` entry points: report the length if
/// asked, terminate, and hand the buffer over for the caller to free.
///
/// # Safety
/// `len` must be NULL or writable, and `ga` must be a byte garray.
unsafe fn finish_tv2(mut ga: garray_T, len: *mut size_t) -> *mut c_char {
    if !len.is_null() {
        // SAFETY: the caller's promise about `len`.
        unsafe { *len = ga.ga_len as size_t };
    }
    Gap(&mut ga).append(0);
    ga.ga_data.cast::<c_char>()
}

/// The string representation of `tv`, quoted so `eval()` can read it back.
///
/// # Safety
/// `tv` must be live; `len` must be NULL or writable.
pub unsafe fn encode_tv2string(tv: *mut typval_T, len: *mut size_t) -> *mut c_char {
    let mut ga = text_garray();
    // SAFETY: the caller's promise about `tv`; `string()` never refuses.
    let evs_ret =
        unsafe { encode_vim_to_string(&raw mut ga, tv, c"encode_tv2string() argument".as_ptr()) };
    debug_assert!(evs_ret);
    did_echo_string_emsg.set(false);
    // SAFETY: the caller's promise about `len`.
    unsafe { finish_tv2(ga, len) }
}

/// The string representation of `tv` as `:echo` displays it — no quotes.
///
/// # Safety
/// As [`encode_tv2string`].
pub unsafe fn encode_tv2echo(tv: *mut typval_T, len: *mut size_t) -> *mut c_char {
    let mut ga = text_garray();
    // SAFETY: the caller's promise about `tv`.
    unsafe {
        // A string or function reference echoes as its own bytes, which is
        // the whole difference between `:echo` and `string()` at the top
        // level; below it, the sink says it again.
        if (*tv).v_type == VAR_STRING || (*tv).v_type == VAR_FUNC {
            if !(*tv).vval.v_string.is_null() {
                ga_concat(&raw mut ga, (*tv).vval.v_string);
            }
        } else {
            let eve_ret = encode_vim_to_echo(&raw mut ga, tv, c":echo argument".as_ptr());
            debug_assert!(eve_ret);
        }
        finish_tv2(ga, len)
    }
}

/// `tv` as JSON, or an empty buffer once the refusal has been reported.
///
/// # Safety
/// As [`encode_tv2string`].
pub unsafe fn encode_tv2json(tv: *mut typval_T, len: *mut size_t) -> *mut c_char {
    let mut ga = text_garray();
    // SAFETY: the caller's promise about `tv`.
    let evj_ret =
        unsafe { encode_vim_to_json(&raw mut ga, tv, c"encode_tv2json() argument".as_ptr()) };
    if !evj_ret {
        // SAFETY: `ga` is this function's own array.
        unsafe { ga_clear(&raw mut ga) };
    }
    did_echo_string_emsg.set(false);
    // SAFETY: the caller's promise about `len`.
    unsafe { finish_tv2(ga, len) }
}
