//! Signs: the definition table, and placing them as extmarks.
//!
//! A sign has two halves. `:sign define` records a *definition* — a name,
//! up to two cells of text and up to four highlight groups — in the table
//! this module owns. `:sign place` then puts a *placement* in a buffer,
//! which is an extmark carrying a [`DecorSignHighlight`] copied out of the
//! definition; the drawing code never sees the definition again, which is
//! why redefining a placed sign has to walk the decoration store and patch
//! every copy.
//!
//! Sign *groups* are namespaces. The global group is namespace 0, a named
//! group is whatever `nvim_create_namespace` answers, and `"*"` means all
//! of them ([`ALL_GROUPS`]).
//!
//! The `:sign` command lives in [`command`], its completion in [`complete`]
//! and the `sign_*()` Vimscript functions in [`vimscript`].

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{CStr, c_char, c_int};
use core::ops::{Deref, DerefMut};
use core::slice;
use std::ffi::CString;

use crate::src::nvim::api::extmark::{describe_ns, nvim_create_namespace};
use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::buffer::{buflist_findname_exp, buflist_findnr};
use crate::src::nvim::charset::{
    backslash_halve, getdigits_int, skiptowhite, skiptowhite_esc, skipwhite, vim_isprintc,
};
use crate::src::nvim::cursor::check_cursor_lnum;
use crate::src::nvim::decoration::{
    DECOR_SIGN_HIGHLIGHT_INIT, SIGN_WIDTH, Sh, decor_find_sign, decor_item_count, decor_put_sh,
    kMTMetaSignHL, kMTMetaSignText, kSHIsSign, sign_item_cmp,
};
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_buf_later};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::funcs::get_buf_arg;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_check_for_opt_dict_arg, tv_check_for_string_arg,
    tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str, tv_dict_alloc, tv_dict_find,
    tv_dict_get_number, tv_dict_get_number_def, tv_dict_get_string, tv_get_lnum, tv_get_number_chk,
    tv_get_string, tv_get_string_chk, tv_list_alloc, tv_list_alloc_ret, tv_list_append_dict,
    tv_list_append_number, tv_list_first,
};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::extmark::{extmark_del, extmark_del_id, extmark_set};
use crate::src::nvim::fold::foldOpenCursor;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::schar_get;
use crate::src::nvim::highlight_group::{HLF_D, get_highlight_name_ext, syn_check_group};
use crate::src::nvim::main::{
    curwin, e_argreq, e_dictreq, e_invalid_buffer_name_str, e_invarg, e_invarg2, e_listreq,
    e_trailing_arg, firstbuf, got_int, namespace_ids,
};
use crate::src::nvim::map::mh_get_String;
use crate::src::nvim::marktree::cursor::{Cursor, lookup_ns, tree_of};
use crate::src::nvim::marktree::key::{
    MT_FLAG_DECOR_SIGNHL, MT_FLAG_DECOR_SIGNTEXT, mt_decor, mt_decor_sign, mt_end,
};
use crate::src::nvim::marktree::{marktree_itr_current, marktree_itr_get, marktree_itr_next};
use crate::src::nvim::mbyte::{MAX_SCHAR_SIZE, utf_ptr2cells, utfc_ptr2len, utfc_ptr2schar};
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{
    emsg, msg_outtrans, msg_putchar, msg_puts, msg_puts_hl, msg_puts_title,
};
use crate::src::nvim::os::libc::{atoi, gettext, snprintf, strcmp, strlen, strncmp};
use crate::src::nvim::strings::{vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    DecorExt, DecorInline, DecorInlineData, DecorPriority, DecorSignHighlight, DecorVirtText,
    Error, EvalFuncData, Integer, MTKey, MarkTree, MarkTreeIter, NS, SignItem, buf_T, dict_T,
    dictitem_T, exarg_T, expand_T, int64_t, linenr_T, list_T, listitem_T, ptrdiff_t, schar_T,
    sign_T, size_t, typval_T, uint16_t, uint32_t, varnumber_T,
};
use crate::src::nvim::window::buf_jump_open_win;
use crate::src::nvim::winlayer::{Buf, Win, buffers, windows};

mod command;
pub use self::command::*;
mod complete;
pub use self::complete::*;
mod vimscript;
pub use self::vimscript::*;

pub const EXPAND_SIGN: c_int = 34;
pub const EXPAND_HIGHLIGHT: c_int = 13;
pub const EXPAND_BUFFERS: c_int = 9;
pub const EXPAND_FILES: c_int = 2;
pub const EXPAND_NOTHING: c_int = 0;

pub const BL_WHITE: c_int = 1;

/// The priority a placement gets when neither the definition nor the
/// `:sign place` / `sign_place()` call names one.
pub const SIGN_DEF_PRIO: c_int = 10;

pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NUL: c_int = '\0' as c_int;
pub const MSG_BUF_LEN: c_int = 480;

/// Room for [`describe_sign_text`]'s answer: SIGN_WIDTH cells of up to
/// `MAX_SCHAR_SIZE` bytes each, the last of which carries the NUL
/// `schar_get` writes.
pub(crate) const SIGN_TEXT_BUF: usize = SIGN_WIDTH as usize * MAX_SCHAR_SIZE as usize;

/// [`group_get_ns`]'s answer for the group `"*"`: every namespace, the
/// global one included.
pub(crate) const ALL_GROUPS: int64_t = u32::MAX as int64_t;

/// [`group_get_ns`]'s answer for a group that names no namespace.
const NO_SUCH_GROUP: int64_t = -1;

pub const SIGNCMD_DEFINE: c_int = 0;
pub const SIGNCMD_UNDEFINE: c_int = 1;
pub const SIGNCMD_LIST: c_int = 2;
pub const SIGNCMD_PLACE: c_int = 3;
pub const SIGNCMD_UNPLACE: c_int = 4;
pub const SIGNCMD_JUMP: c_int = 5;
/// One past the last subcommand: [`sign_cmd_idx`]'s "not a subcommand".
pub const SIGNCMD_LAST: c_int = 6;

/// The `:sign` subcommands, in the order the `SIGNCMD_*` constants number
/// them. Also the completion list for `:sign <Tab>`.
pub(crate) const CMDS: [&CStr; SIGNCMD_LAST as usize] = [
    c"define",
    c"undefine",
    c"list",
    c"place",
    c"unplace",
    c"jump",
];

// --------------------------------------------------------- the definitions

/// One `:sign define` entry.
struct SignEntry {
    /// Owns the string `def.sn_name` points at. In the same box as `def`, so
    /// that pointer stays valid for the entry's whole life.
    name: CString,
    def: sign_T,
}

/// A sign definition the caller has promised is live. Definitions are boxed
/// (see [`SIGNS`]), so one stays put until its box is dropped.
#[derive(Clone, Copy)]
struct Sign(*mut sign_T);

impl Deref for Sign {
    type Target = sign_T;
    fn deref(&self) -> &sign_T {
        // SAFETY: the constructor's promise — a live definition.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Sign {
    fn deref_mut(&mut self) -> &mut sign_T {
        // SAFETY: as above.
        unsafe { &mut *self.0 }
    }
}

/// Every defined sign, in definition order.
///
/// Boxed because a definition's address escapes: `sign_place` hands a
/// `*mut sign_T` to `buf_set_sign`, and `sign_list_defined` holds one across
/// `msg_puts`. Deleting swap-removes, which is what the `Map(cstr_t, ptr_t)`
/// upstream uses does to its dense key array — and that order is observable
/// in `:sign list`, `sign_getdefined()` and `:sign` completion.
static SIGNS: GlobalCell<Vec<Box<SignEntry>>> = GlobalCell::new(Vec::new());

/// The namespaces `:sign place group=` has created, in creation order — the
/// list `:sign` completion offers as group names.
///
/// Groups are never removed, so a group whose signs have all been unplaced
/// is still offered. That is upstream's behaviour.
static SIGN_GROUPS: GlobalCell<Vec<Integer>> = GlobalCell::new(Vec::new());

/// Runs `f` over the definition named `name`, if there is one.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn with_sign<R>(name: *const c_char, f: impl FnOnce(&mut Box<SignEntry>) -> R) -> Option<R> {
    // SAFETY: the caller's name.
    let key = unsafe { CStr::from_ptr(name) };
    SIGNS.with_mut(|signs| signs.iter_mut().find(|e| e.name.as_c_str() == key).map(f))
}

/// The definition `:sign define` recorded under `name`, or null.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn sign_find(name: *const c_char) -> *mut sign_T {
    // SAFETY: the caller's name. The answer stays valid because each entry
    // is boxed and only `sign_undefine_by_name` ever drops one.
    unsafe { with_sign(name, |e| &raw mut e.def).unwrap_or(::core::ptr::null_mut()) }
}

/// Whether a sign is still defined under `name`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn sign_is_defined(name: *const c_char) -> bool {
    // SAFETY: the caller's name.
    unsafe { with_sign(name, |_| ()).is_some() }
}

/// Every defined sign, in definition order.
///
/// A snapshot rather than an iterator: `:sign list` and `sign_getdefined()`
/// format each entry as they walk, and formatting can re-enter.
pub(crate) fn sign_defs() -> Vec<*mut sign_T> {
    SIGNS.with_mut(|signs| signs.iter_mut().map(|e| &raw mut e.def).collect())
}

/// The name of the `idx`'th defined sign, or null past the end — the
/// `:sign list` / `:sign undefine` completion source.
pub(crate) fn sign_nth_name(idx: usize) -> *mut c_char {
    SIGNS.with(|signs| {
        signs
            .get(idx)
            .map_or(::core::ptr::null_mut(), |e| e.name.as_ptr().cast_mut())
    })
}

/// The namespace of the `idx`'th sign group, or `None` past the end — the
/// `group=` completion source.
pub(crate) fn sign_nth_group(idx: usize) -> Option<Integer> {
    SIGN_GROUPS.with(|groups| groups.get(idx).copied())
}

/// The namespace id `group` names, or 0 when it names none.
///
/// # Safety
/// `group` must be a NUL-terminated string.
unsafe fn namespace_id(group: *const c_char) -> c_int {
    let map = namespace_ids.ptr();
    // SAFETY: the caller's group name, and the editor's own namespace table.
    let k = unsafe { mh_get_String(&raw mut (*map).set, cstr_as_string(group)) };
    // SAFETY: `k` is an index that table just answered with.
    if k == u32::MAX {
        0
    } else {
        unsafe { *(*map).values.add(k as usize) }
    }
}

/// The namespace filter `group` asks for: 0 for the global group,
/// [`ALL_GROUPS`] for `"*"`, [`NO_SUCH_GROUP`] for a group that does not
/// exist, and otherwise the group's own namespace.
///
/// # Safety
/// `group` must be null or a NUL-terminated string.
pub(crate) unsafe fn group_get_ns(group: *const c_char) -> int64_t {
    if group.is_null() {
        return 0;
    }
    // SAFETY: the caller's group name.
    if unsafe { strcmp(group, c"*".as_ptr()) } == 0 {
        return ALL_GROUPS;
    }
    // SAFETY: as above.
    match unsafe { namespace_id(group) } {
        0 => NO_SUCH_GROUP,
        ns => ns as int64_t,
    }
}

/// The name to report for a placed sign: the definition's name while it is
/// still defined, `"[Deleted]"` once it is not, and `""` for a sign placed
/// through `nvim_buf_set_extmark` rather than `:sign`.
///
/// # Safety
/// `sh` must be a live sign decoration.
pub(crate) unsafe fn sign_get_name(sh: *mut DecorSignHighlight) -> *const c_char {
    // SAFETY: the caller's decoration.
    let name = unsafe { Sh::new(sh) }.sign_name;
    if name.is_null() {
        return c"".as_ptr();
    }
    // SAFETY: a sign decoration's name is a NUL-terminated string it owns.
    if unsafe { sign_is_defined(name) } {
        name
    } else {
        c"[Deleted]".as_ptr()
    }
}

// -------------------------------------------------------------- the text

/// Renders `sign_text` back into `buf` and answers how many bytes it wrote.
/// No extra `+ 1` is needed on [`SIGN_TEXT_BUF`]: a cell that renders empty
/// stops the walk, and `schar_get` has already written its NUL.
///
/// # Safety
/// `buf` must have room for [`SIGN_TEXT_BUF`] bytes and `sign_text` for
/// `SIGN_WIDTH` cells.
pub unsafe fn describe_sign_text(buf: *mut c_char, sign_text: *mut schar_T) -> size_t {
    // SAFETY: `sign_text` holds SIGN_WIDTH cells, per the caller.
    let cells = unsafe { slice::from_raw_parts(sign_text, SIGN_WIDTH as usize) };
    let mut at = 0;
    for &cell in cells {
        // SAFETY: `buf` has room for SIGN_TEXT_BUF bytes, and `at` is how
        // many of them the cells so far have used.
        let len = unsafe {
            schar_get(buf.add(at), cell);
            strlen(buf.add(at))
        };
        if len == 0 {
            break;
        }
        at += len;
    }
    at
}

/// Parses a sign's `text=` into `sign_text`; `FAIL` when it does not fit.
///
/// `from_define` distinguishes the `:sign define` / `sign_define()` caller,
/// which unescapes backslashes (so `text=\ x` can carry a space) and
/// diagnoses a bad value, from `nvim_buf_set_extmark`, which does neither.
/// The unescaping happens **in place**, in the caller's own buffer.
///
/// A one-cell text is padded to two with a space, and a two-cell character
/// blanks the second cell so the drawing code knows not to emit it.
///
/// # Safety
/// `text` must be a writable NUL-terminated string and `sign_text` must have
/// room for `SIGN_WIDTH` cells.
pub unsafe fn init_sign_text(
    text: *mut c_char,
    sign_text: *mut schar_T,
    from_define: bool,
) -> c_int {
    // SAFETY: the caller's text, NUL-terminated.
    let mut endp = unsafe { text.add(strlen(text)) };
    // SAFETY: the caller's cell array, `SIGN_WIDTH` wide.
    let out = unsafe { slice::from_raw_parts_mut(sign_text, SIGN_WIDTH as usize) };

    if from_define {
        let mut s = text;
        // SAFETY: `s` walks the text, which stays NUL-terminated: the shift
        // below copies the NUL along with the rest. The last byte is never
        // examined, as upstream does not examine it either.
        while unsafe { s.add(1) } < endp {
            // SAFETY: as above.
            unsafe {
                if *s == b'\\' as c_char {
                    ::core::ptr::copy(s.add(1), s, strlen(s.add(1)) + 1);
                    endp = endp.sub(1);
                }
                s = s.add(1);
            };
        }
    }

    // Count display cells, stopping at the first unprintable character.
    let mut cells = 0;
    let mut s = text;
    while s < endp {
        // SAFETY: `s` points into the text, which is NUL-terminated.
        unsafe {
            let mut c: c_int = 0;
            let sc = utfc_ptr2schar(s, &raw mut c);
            // `sign_text` holds SIGN_WIDTH cells but this walk runs to the
            // end of `text` and only tests the width afterwards, so upstream
            // (v0.12.4) overruns the array for anything wider: on the heap
            // via `:sign define x text=xxx`, on the STACK via
            // nvim_buf_set_extmark{sign_text=..}. Dropping the out-of-range
            // stores is unobservable — every path that gets here with
            // `cells >= SIGN_WIDTH` goes on to fail and discard the array.
            if cells < SIGN_WIDTH {
                out[cells as usize] = sc;
            }
            if !vim_isprintc(c) {
                break;
            }
            let width = utf_ptr2cells(s);
            if width == 2 && cells + 1 < SIGN_WIDTH {
                out[cells as usize + 1] = 0;
            }
            cells += width;
            s = s.add(utfc_ptr2len(s) as usize);
        };
    }

    // Must be empty, one cell or two; `s != endp` means the walk stopped on
    // an unprintable character.
    if s != endp || cells > SIGN_WIDTH {
        if from_define {
            // SAFETY: the caller's text, and a format the message takes.
            unsafe { semsg_c!(gettext(c"E239: Invalid sign text: %s".as_ptr()), text) };
        }
        return FAIL;
    }

    if cells < 1 {
        out[0] = 0;
    } else if cells == 1 {
        out[1] = b' ' as schar_T;
    }
    OK
}

// ------------------------------------------------------- define / undefine

/// Defines a sign, or updates the one already defined under `name`.
///
/// Every argument but `name` and `prio` is optional: a null leaves that
/// property alone, which is what makes `:sign define X texthl=Y` an update
/// rather than a redefinition. `prio` is always written, `-1` meaning
/// [`SIGN_DEF_PRIO`].
///
/// # Safety
/// Every non-null argument must be a NUL-terminated string; `text` must
/// additionally be writable ([`init_sign_text`] unescapes it in place).
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn sign_define_by_name(
    name: *mut c_char,
    icon: *mut c_char,
    text: *mut c_char,
    linehl: *mut c_char,
    texthl: *mut c_char,
    culhl: *mut c_char,
    numhl: *mut c_char,
    prio: c_int,
) -> c_int {
    // SAFETY: the caller's name.
    let mut sp = unsafe { sign_find(name) };
    let new_sign = sp.is_null();
    if new_sign {
        // SAFETY: as above.
        let owned = unsafe { CStr::from_ptr(name) }.to_owned();
        let mut entry = Box::new(SignEntry {
            def: sign_T {
                sn_name: owned.as_ptr().cast_mut(),
                ..Default::default()
            },
            name: owned,
        });
        sp = &raw mut entry.def;
        SIGNS.with_mut(|signs| signs.push(entry));
    }
    // SAFETY: `sp` is a boxed definition, either found or just pushed.
    let mut def = Sign(sp);

    if !icon.is_null() {
        // SAFETY: the old icon is this module's own `xstrdup` and the new
        // one the caller's NUL-terminated path.
        def.sn_icon = unsafe {
            xfree(def.sn_icon.cast());
            let owned = xstrdup(icon);
            backslash_halve(owned);
            owned
        };
    }

    // SAFETY: the caller's text, writable and NUL-terminated.
    let text_ok = text.is_null()
        || unsafe { init_sign_text(text, (&raw mut def.sn_text).cast(), true) } == OK;
    if !text_ok {
        return FAIL;
    }

    def.sn_priority = prio;

    for (which, arg) in [linehl, texthl, culhl, numhl].into_iter().enumerate() {
        if arg.is_null() {
            continue;
        }
        // SAFETY: the caller's highlight group name, NUL-terminated.
        let hl = unsafe {
            if *arg != 0 {
                syn_check_group(arg, strlen(arg))
            } else {
                0
            }
        };
        match which {
            0 => def.sn_line_hl = hl,
            1 => def.sn_text_hl = hl,
            2 => def.sn_cul_hl = hl,
            _ => def.sn_num_hl = hl,
        }
    }

    if !new_sign {
        // SAFETY: the caller's name and the definition above.
        unsafe { update_placements(name, sp) };
    }
    OK
}

/// Copies a redefined sign's text and highlights into every placement of it,
/// and redraws the windows showing one.
///
/// Placements carry their own copy of the definition, so this is the only
/// thing that makes a `:sign define` of an already-placed sign visible.
///
/// # Safety
/// `name` must be NUL-terminated and `sp` a live definition.
unsafe fn update_placements(name: *const c_char, sp: *const sign_T) {
    // SAFETY: the caller's definition, copied out so the store below cannot
    // move it underneath the walk.
    let def = unsafe { *sp };
    let mut did_redraw = false;
    for i in 0..decor_item_count() {
        let mut sh = Sh::at(i as uint32_t);
        // SAFETY: the caller's name, and a store item's own name string.
        if sh.sign_name.is_null() || unsafe { strcmp(sh.sign_name, name) } != 0 {
            continue;
        }
        sh.text = def.sn_text;
        sh.hl_id = def.sn_text_hl;
        sh.line_hl_id = def.sn_line_hl;
        sh.number_hl_id = def.sn_num_hl;
        sh.cursorline_hl_id = def.sn_cul_hl;
        if !did_redraw {
            for wp in windows() {
                let buf = wp.buffer();
                // SAFETY: a live window's buffer is live.
                if unsafe { buf_has_signs(buf.raw()) } {
                    // SAFETY: as above.
                    unsafe { redraw_buf_later(buf.raw(), UPD_NOT_VALID) };
                }
            }
            did_redraw = true;
        }
    }
}

/// Forgets the definition named `name`, or answers `FAIL` with E155.
///
/// Placements survive: they carry their own copy, and [`sign_get_name`]
/// starts reporting them as `[Deleted]`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn sign_undefine_by_name(name: *const c_char) -> c_int {
    // SAFETY: the caller's name.
    let key = unsafe { CStr::from_ptr(name) };
    let entry = SIGNS.with_mut(|signs| {
        signs
            .iter()
            .position(|e| e.name.as_c_str() == key)
            // Swap-remove, which is what the map upstream uses does to its
            // dense key array; the resulting order is observable.
            .map(|i| signs.swap_remove(i))
    });
    let Some(entry) = entry else {
        // SAFETY: the caller's name, and a format the message takes.
        unsafe { semsg_c!(gettext(c"E155: Unknown sign: %s".as_ptr()), name) };
        return FAIL;
    };
    // SAFETY: the icon is this module's own `xstrdup`.
    unsafe { xfree(entry.def.sn_icon.cast()) };
    OK
}

/// Forgets every definition — `sign_undefine()` with no argument.
pub fn free_signs() {
    for entry in SIGNS.with_mut(::core::mem::take) {
        // SAFETY: the icon is this module's own `xstrdup` and nothing else
        // holds it.
        unsafe { xfree(entry.def.sn_icon.cast()) };
    }
}

// ------------------------------------------------------------- placements

/// Places or replaces the sign extmark `*id` in `buf` at `lnum`.
///
/// Writes `*id` back when it was zero: `extmark_set` allocates one.
///
/// # Safety
/// `buf` and `sp` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_set_sign(
    buf: *mut buf_T,
    id: *mut uint32_t,
    group: *mut c_char,
    prio: c_int,
    lnum: linenr_T,
    sp: *mut sign_T,
) {
    // SAFETY: the caller's buffer, and the definition copied out — the store
    // and the extmark below can both move the entry `sp` points into.
    let (buf, def) = unsafe { (Buf::new(buf), *sp) };
    // SAFETY: the caller's group name.
    let ns = if group.is_null() {
        0
    } else {
        unsafe { namespace_of(group) as uint32_t }
    };

    let mut sign = DECOR_SIGN_HIGHLIGHT_INIT;
    sign.flags |= kSHIsSign;
    sign.text = def.sn_text;
    // SAFETY: a definition's name is a NUL-terminated string it owns.
    sign.sign_name = unsafe { xstrdup(def.sn_name) };
    sign.hl_id = def.sn_text_hl;
    sign.line_hl_id = def.sn_line_hl;
    sign.number_hl_id = def.sn_num_hl;
    sign.cursorline_hl_id = def.sn_cul_hl;
    sign.priority = prio as DecorPriority;

    let has_hl = def.sn_line_hl != 0 || def.sn_num_hl != 0 || def.sn_cul_hl != 0;
    let text_flag = if def.sn_text[0] != 0 {
        MT_FLAG_DECOR_SIGNTEXT
    } else {
        0
    };
    let hl_flag = if has_hl { MT_FLAG_DECOR_SIGNHL } else { 0 };

    let decor = DecorInline {
        ext: true,
        data: DecorInlineData {
            ext: DecorExt {
                sh_idx: decor_put_sh(sign),
                vt: ::core::ptr::null_mut::<DecorVirtText>(),
            },
        },
    };
    let row = buf.line_count().min(lnum) - 1;
    // SAFETY: a live buffer, and `id` is the caller's writable out-parameter.
    unsafe {
        extmark_set(
            buf.raw(),
            ns,
            id,
            row,
            0,
            -1,
            -1,
            decor,
            (text_flag | hl_flag) as uint16_t,
            true,
            false,
            true,
            true,
            ::core::ptr::null_mut::<Error>(),
        )
    };
}

/// The namespace `group` names, creating it — and remembering it for
/// completion — the first time a sign is placed in it.
///
/// # Safety
/// `group` must be NUL-terminated.
unsafe fn namespace_of(group: *mut c_char) -> Integer {
    // SAFETY: the caller's group name.
    let known = unsafe { namespace_id(group) } != 0;
    // SAFETY: as above.
    let ns = unsafe { nvim_create_namespace(cstr_as_string(group)) };
    if !known {
        SIGN_GROUPS.with_mut(|groups| groups.push(ns));
    }
    ns
}

/// Re-places the existing sign `*id` where it already is, so that a
/// `:sign place {id} name=...` with no `line=` changes its type or priority.
///
/// Answers its line number, or zero when there is no such sign.
///
/// # Safety
/// `buf` and `sp` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_mod_sign(
    buf: *mut buf_T,
    id: *mut uint32_t,
    group: *mut c_char,
    prio: c_int,
    sp: *mut sign_T,
) -> linenr_T {
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    if ns < 0 || (!group.is_null() && ns == 0) {
        return 0;
    }
    // SAFETY: the caller's buffer and out-parameter.
    let mark = unsafe { lookup_ns(Buf::new(buf), ns as uint32_t, *id, false) };
    if mark.pos.row >= 0 {
        // SAFETY: the caller's buffer, group and definition.
        unsafe { buf_set_sign(buf, id, group, prio, mark.pos.row + 1, sp) };
    }
    mark.pos.row + 1
}

/// The line the sign `id` sits on in `group`, or zero when there is none.
///
/// Zero rather than an error, so that `:sign jump` still loads the file.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_findsign(buf: *mut buf_T, id: c_int, group: *mut c_char) -> c_int {
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    if ns < 0 || (!group.is_null() && ns == 0) {
        return 0;
    }
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };
    lookup_ns(buf, ns as uint32_t, id as uint32_t, false)
        .pos
        .row
        + 1
}

/// Orders marks the way `:sign` reports them and removes them: by row, then
/// by [`sign_item_cmp`] — priority, then mark id, then placement serial, all
/// newest first.
///
/// A stable sort is provably the permutation the `qsort` upstream uses
/// produced: `buf_put_decor_sh` hands every placed sign a distinct
/// `sign_add_id`, so the comparator is a total order and no two entries tie.
///
/// # Safety
/// Every mark must carry a live sign decoration.
pub(crate) unsafe fn sort_signs(signs: &mut [MTKey]) {
    signs.sort_by(|a, b| {
        if a.pos.row != b.pos.row {
            return a.pos.row.cmp(&b.pos.row);
        }
        let (sh1, sh2) = (decor_find_sign(mt_decor(*a)), decor_find_sign(mt_decor(*b)));
        assert!(!sh1.is_null() && !sh2.is_null(), "sign mark without a sign");
        let (ia, ib) = (
            SignItem { sh: sh1, id: a.id },
            SignItem { sh: sh2, id: b.id },
        );
        // SAFETY: the caller's marks, whose sign items the store just named.
        unsafe { sign_item_cmp(&ia, &ib) }.cmp(&0)
    });
}

/// Deletes signs from `buf`.
///
/// `id` of zero means any id and `group` selects a namespace (see
/// [`group_get_ns`]). `atlnum` above zero narrows to one line — where,
/// unlike every other combination, only the **highest priority** sign goes.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn buf_delete_signs(
    buf: *mut buf_T,
    group: *mut c_char,
    id: c_int,
    atlnum: linenr_T,
) -> c_int {
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    if ns < 0 {
        return FAIL;
    }
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };

    let mut itr = MarkTreeIter::default();
    let row = if atlnum > 0 { atlnum - 1 } else { 0 };
    let mut signs: Vec<MTKey> = Vec::new();

    // The walk continues below as a bare iterator: `extmark_del` steps it
    // itself, which a `Cursor` cannot express.
    {
        let mut walk = Cursor::in_buffer(buf, &mut itr);
        if atlnum > 0 {
            // Signs that *started* above this row but still cover it.
            if !walk.seek_overlap(row, 0) {
                return FAIL;
            }
            while let Some(pair) = walk.step_overlap() {
                if (ns == ALL_GROUPS || ns == pair.start.ns as int64_t) && mt_decor_sign(pair.start)
                {
                    signs.push(pair.start);
                }
            }
        } else {
            walk.seek(0, 0);
        }
    }

    let tree = tree_of(buf);
    while !itr.x.is_null() {
        // SAFETY: the iterator is positioned in this buffer's live tree.
        let mark = unsafe { marktree_itr_current(&mut itr) };
        if row != 0 && mark.pos.row > row {
            break;
        }
        let wanted = !mt_end(mark)
            && mt_decor_sign(mark)
            && (id == 0 || mark.id as c_int == id)
            && (ns == ALL_GROUPS || ns == mark.ns as int64_t);
        if wanted && atlnum <= 0 {
            // `extmark_del` advances the iterator itself.
            // SAFETY: as above, plus a live buffer.
            unsafe { extmark_del(buf.raw(), &raw mut itr, mark, true) };
            continue;
        }
        if wanted {
            signs.push(mark);
        }
        // SAFETY: as above.
        unsafe { marktree_itr_next(&mut *tree, &mut itr) };
    }

    if signs.is_empty() {
        // Only the single-line form treats "nothing matched" as failure; the
        // sweeping forms are content to have deleted nothing.
        return if atlnum > 0 { FAIL } else { OK };
    }
    // SAFETY: every mark collected above carries a live sign decoration.
    unsafe { sort_signs(&mut signs) };
    // SAFETY: a live buffer.
    unsafe { extmark_del_id(buf.raw(), signs[0].ns, signs[0].id) };
    OK
}

/// Whether `buf` carries any sign at all — text or highlight.
///
/// # Safety
/// `buf` must be live.
pub unsafe fn buf_has_signs(buf: *const buf_T) -> bool {
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf.cast_mut()) };
    buf.meta_total(kMTMetaSignHL) + buf.meta_total(kMTMetaSignText) != 0
}

/// Places the sign `name` in `buf`, or changes the existing sign `*id`.
///
/// `lnum` above zero places; zero re-places the existing sign where it is,
/// which is how `:sign place {id} name=X buffer=N` changes a sign's type.
/// `prio` of `-1` takes the definition's, or [`SIGN_DEF_PRIO`].
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated; `name` must
/// be NUL-terminated.
pub(crate) unsafe fn sign_place(
    id: *mut uint32_t,
    group: *mut c_char,
    name: *mut c_char,
    buf: *mut buf_T,
    lnum: linenr_T,
    prio: c_int,
) -> c_int {
    // `*` is the "all groups" filter, not a group one can place into.
    // SAFETY: the caller's group name, null or NUL-terminated.
    if !group.is_null() && unsafe { *group == b'*' as c_char || *group == 0 } {
        return FAIL;
    }

    // SAFETY: the caller's name.
    let sp = unsafe { sign_find(name) };
    if sp.is_null() {
        // SAFETY: as above.
        unsafe { semsg_c!(gettext(c"E155: Unknown sign: %s".as_ptr()), name) };
        return FAIL;
    }
    // SAFETY: `sign_find` answered a live definition.
    let prio = match (prio, unsafe { (*sp).sn_priority }) {
        (-1, -1) => SIGN_DEF_PRIO,
        (-1, def) => def,
        (given, _) => given,
    };

    // SAFETY: the caller's buffer, group and the definition above.
    let lnum = unsafe {
        if lnum > 0 {
            buf_set_sign(buf, id, group, prio, lnum, sp);
            lnum
        } else {
            buf_mod_sign(buf, id, group, prio, sp)
        }
    };
    if lnum <= 0 {
        // SAFETY: the caller's name.
        unsafe {
            semsg_c!(
                gettext(c"E885: Not possible to change sign %s".as_ptr()),
                name,
            )
        };
        return FAIL;
    }
    OK
}

/// [`sign_unplace`] for one buffer.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
unsafe fn sign_unplace_inner(
    buf: *mut buf_T,
    id: c_int,
    group: *mut c_char,
    atlnum: linenr_T,
) -> c_int {
    // SAFETY: the caller's buffer.
    if !unsafe { buf_has_signs(buf) } {
        return FAIL;
    }
    // SAFETY: the caller's group name, null or NUL-terminated.
    let all_groups = !group.is_null() && unsafe { *group } == b'*' as c_char;
    if id == 0 || atlnum > 0 || all_groups {
        // SAFETY: the caller's buffer and group.
        return unsafe { buf_delete_signs(buf, group, id, atlnum) };
    }
    // SAFETY: the caller's group name.
    let ns = unsafe { group_get_ns(group) };
    // SAFETY: the caller's buffer.
    if ns < 0 || !unsafe { extmark_del_id(buf, ns as uint32_t, id as uint32_t) } {
        return FAIL;
    }
    OK
}

/// Removes signs from `buf`, or from every buffer when `buf` is null.
///
/// # Safety
/// `buf` must be null or live; `group` must be null or NUL-terminated.
pub(crate) unsafe fn sign_unplace(
    buf: *mut buf_T,
    id: c_int,
    group: *mut c_char,
    atlnum: linenr_T,
) -> c_int {
    if !buf.is_null() {
        // SAFETY: the caller's buffer and group.
        return unsafe { sign_unplace_inner(buf, id, group, atlnum) };
    }
    let mut retval = OK;
    for cbuf in buffers() {
        // SAFETY: a live buffer from the editor's own list, and the caller's
        // group name.
        if unsafe { sign_unplace_inner(cbuf.raw(), id, group, atlnum) } == FAIL {
            retval = FAIL;
        }
    }
    retval
}

/// Moves the cursor to the sign `id`, opening `buf` if no window shows it.
///
/// Answers the line jumped to, or `-1`.
///
/// # Safety
/// `buf` must be live; `group` must be null or NUL-terminated.
pub(crate) unsafe fn sign_jump(id: c_int, group: *mut c_char, buf: *mut buf_T) -> linenr_T {
    // SAFETY: the caller's buffer and group.
    let lnum = unsafe { buf_findsign(buf, id, group) };
    if lnum <= 0 {
        // SAFETY: a format the message takes.
        unsafe { semsg_c!(gettext(c"E157: Invalid sign ID: %d".as_ptr()), id) };
        return -1;
    }
    // SAFETY: the caller's buffer.
    let buf = unsafe { Buf::new(buf) };

    // SAFETY: a live buffer.
    if !unsafe { buf_jump_open_win(buf.raw()) }.is_null() {
        // SAFETY: `curwin` is live from startup to exit.
        let mut win = unsafe { Win::current() };
        win.w_cursor.lnum = lnum;
        // SAFETY: as above.
        unsafe {
            check_cursor_lnum(win.raw());
            beginline(BL_WHITE);
        };
    } else {
        if buf.b_fname.is_null() {
            // SAFETY: a static message.
            unsafe {
                emsg(gettext(
                    c"E934: Cannot jump to a buffer that does not have a name".as_ptr(),
                ))
            };
            return -1;
        }
        // SAFETY: a live buffer's name is a NUL-terminated string it owns.
        let cmdlen = unsafe { strlen(buf.b_fname) } + 24;
        let mut cmd = vec![0 as c_char; cmdlen + 1];
        // SAFETY: as above; `cmd` has room for `cmdlen` bytes plus the NUL.
        unsafe {
            snprintf(
                cmd.as_mut_ptr(),
                cmdlen,
                c"e +%ld %s".as_ptr(),
                lnum as int64_t,
                buf.b_fname,
            );
            do_cmdline_cmd(cmd.as_mut_ptr());
        };
    }

    // SAFETY: the editor's own fold state.
    unsafe { foldOpenCursor() };
    lnum
}
