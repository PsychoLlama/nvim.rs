//! Resolving the left-hand side of an assignment.
//!
//! `get_lval` walks a name and its subscripts down to the container and key
//! that [`set_var_lval`](assign::set_var_lval) will write through. The two
//! halves communicate only through `LVal`, and which of its fields are set
//! is what says *what kind* of assignment this is:
//!
//! | `ll_tv` | `ll_blob` | `ll_newkey` | `ll_range` | the target |
//! | --- | --- | --- | --- | --- |
//! | null | null | — | — | a plain variable, by name |
//! | null | set | — | — | a Blob byte or byte range |
//! | set | — | null | false | an existing List or Dict item |
//! | set | — | set | false | a Dict key that does not exist yet |
//! | set | — | null | true | a List slice |

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

// The write half, which reads the record this one fills in.
mod assign;
pub use self::assign::*;

use crate::cstr;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::message_fmt::{c_str, c_str_len};
use crate::semsg;
use crate::strings::has_char;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ops::ControlFlow;
use core::ptr::null_mut;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::eval::EVALARG_EVALUATE;
use crate::eval::typval::DictTab;
use crate::eval::typval::{
    NumBuf, di_lock, tv_blob_alloc_ret, tv_blob_check_index, tv_blob_check_range, tv_blob_len,
    tv_check_str, tv_dict_alloc, tv_dict_find, tv_get_number, tv_list_alloc_ret,
    tv_list_check_range_index_one, tv_list_check_range_index_two, tv_list_items_mut,
};
use crate::eval::userfunc::get_funccal_args_ht;
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::vars::{
    find_var, get_vimvar_dict, valid_varname, var_check_lock, var_check_ro, var_wrong_func_name,
};
use crate::eval::{Cur, Lv, Tv};
use crate::eval::{
    FNE_INCL_BR, GLV_FAIL, GLV_NO_AUTOLOAD, GLV_OK, GLV_QUIET, GLV_READ_ONLY, GLV_STOP, GlvStatus,
    e_cannot_slice_dictionary, e_missbrac, eval_isnamec, eval_isnamec1, eval1, find_name_end,
    make_expanded_name, tv_is_luafunc,
};
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::aborting;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xfree, xmemdupz, xstrdup};
use crate::message::state::emsg_severe;
use crate::types::EvalArg;
use crate::types::{
    Dict, DictItem, Failed, LVal, List, NUL, TypVal, VAR_BLOB, VAR_DEF_SCOPE, VAR_DICT, VAR_LIST,
    VAR_UNKNOWN, VarNumber, kListLenUnknown, ptrdiff_t, size_t,
};

/// A freshly declared typval.
pub(super) const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// The namespace letters a `x:` prefix may use. A `:` anywhere else ends
/// the name.
const NAMESPACES: &core::ffi::CStr = c"bgstvw";

/// The end of the plain name starting at `arg`, or `arg` itself when it
/// does not start one. With `use_namespace`, a single leading `x:` from
/// `NAMESPACES` is part of the name rather than its end.
///
/// # Safety
/// `arg` must be NUL-terminated.
pub(crate) unsafe fn to_name_end(arg: *const c_char, use_namespace: bool) -> *const c_char {
    // SAFETY: the caller's promise -- `arg` is NUL-terminated, so its first byte is readable.
    let first = unsafe { *arg };
    if !eval_isnamec1(first as c_int) {
        return arg;
    }
    // SAFETY: a name character is not the terminator, so the byte after it is inside the string.
    let start = unsafe { arg.add(1) };
    let mut p = start;
    loop {
        // SAFETY: `p` walks the string and every step stops at the terminator.
        let c = unsafe { *p };
        if c as c_int == NUL || !eval_isnamec(c as c_int) {
            break;
        }
        if c == b':' as c_char {
            // A `:` continues the name only as the one namespace letter.
            let namespaced = use_namespace && p == start && has_char(NAMESPACES, first as c_int);
            if !namespaced {
                break;
            }
        }
        // SAFETY: `c` is not the terminator, so `p` is on a character.
        p = unsafe { p.offset(utfc_ptr2len(p as *mut c_char) as isize) };
    }
    p
}

/// Resolve one `.key` or `[key]` subscript against the Dictionary in
/// `lval->ll_tv`. `key` is the text for a `.key`; for a `[key]` it is taken
/// from `var1` and `len` is -1.
///
/// Answers `GLV_STOP` when the key does not exist yet and may be added —
/// `ll_newkey` then holds it — and `GLV_FAIL` when it may not.
///
/// # Safety
/// `lval` must be valid with `ll_tv` a Dict; the rest as `get_lval`'s.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn get_lval_dict_item(
    lval: *mut LVal,
    name: *mut c_char,
    key: *mut c_char,
    len: c_int,
    key_end: *mut *mut c_char,
    var1: &mut TypVal,
    flags: c_int,
    unlet: bool,
    result: Option<&mut TypVal>,
) -> GlvStatus {
    let mut numbuf = NumBuf::new();
    let quiet = flags & GLV_QUIET as c_int != 0;
    // SAFETY: the caller's promise; `key_end` holds a cursor into `name`.
    let (mut lval, p) = unsafe { (Lv::new(lval), *key_end) };
    // SAFETY: the caller's promise: `ll_tv` holds a Dict, so `v_dict` is live.
    let mut container = unsafe { Tv::new(lval.ll_tv) };
    // "[key]": the key is `var1`'s string, which is a Number or String.
    let key = if len == -1 {
        // SAFETY: `var1` is the caller's, and `numbuf` outlives the string rendered into it.
        unsafe { numbuf.string(var1) as *mut c_char }
    } else {
        key
    };
    lval.ll_list = null_mut::<List>();

    // A null Dict is an empty Dict; allocate one now.
    // SAFETY: as above.
    if container.dict_or_null().is_null() {
        // SAFETY: the allocator is the editor's own; the typval takes the
        // handle over.
        container.write_dict(Some(tv_dict_alloc()));
    }
    lval.ll_dict = container.dict_or_null();
    // SAFETY: `ll_dict` is a live Dict, and `key` is NUL-terminated or `len` bytes long.
    lval.ll_di = unsafe { tv_dict_find(lval.ll_dict, key, len as ptrdiff_t) };
    // The one field this needs, read through the pointer.
    // SAFETY: `ll_dict` is the live Dict just resolved.
    let dv_scope = unsafe { (*lval.ll_dict).dv_scope };

    // Assigning into a scope dictionary: check that the name is a valid
    // variable name, and a valid *function* name too unless the scope is
    // `l:` or `g:`. Overwriting a builtin function is not allowed.
    if let Some(value) = result.as_deref()
        && dv_scope != 0
    {
        // The two checks want a NUL-terminated key, so a `.key` is
        // terminated in place and put back.
        // SAFETY: a `.key`'s `len` bytes are inside the writable `name`.
        let prevval = if len != -1 {
            unsafe { *key.offset(len as isize) }
        } else {
            0
        };
        if len != -1 {
            // SAFETY: as above.
            unsafe { *key.offset(len as isize) = NUL as c_char };
        }
        // SAFETY: `key` is NUL-terminated either way now.
        let existing = lval.ll_di.is_null();
        let wrong = (dv_scope == VAR_DEF_SCOPE
            && value.is_func()
            && unsafe { var_wrong_func_name(key, existing) })
            || !unsafe { valid_varname(key) };
        if len != -1 {
            // SAFETY: as above -- the byte cut out is put back.
            unsafe { *key.offset(len as isize) = prevval };
        }
        if wrong {
            return GLV_FAIL;
        }
    }

    // SAFETY: a non-null `ll_di` is a live dictionary item.
    let lua_key = !lval.ll_di.is_null()
        && unsafe { tv_is_luafunc(&mut (*lval.ll_di).di_tv) }
        && len == -1
        && result.is_none();
    if lua_key {
        let what = c"v:['lua']".as_ptr();
        // SAFETY: the format takes one NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E461: Illegal variable name: {what}");
        return GLV_FAIL;
    }

    if lval.ll_di.is_null() {
        // A "v:" or "a:" variable cannot be added.
        // SAFETY: naming a live Dict's hashtab reads nothing.
        let ht = unsafe { &raw mut (*lval.ll_dict).dv_hashtab };
        let args_ht = get_funccal_args_ht();
        if lval.ll_dict == get_vimvar_dict() || ht == args_ht {
            // SAFETY: the format takes one NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E461: Illegal variable name: {name}");
            return GLV_FAIL;
        }
        // The key does not exist. It may be added — unless something
        // follows it to subscript, or this is an `:unlet`.
        // SAFETY: `p` is a cursor into the NUL-terminated `name`.
        let after = unsafe { *p };
        if after == b'[' as c_char || after == b'.' as c_char || unlet {
            if !quiet {
                // SAFETY: the format takes one NUL-terminated string.
                let key = unsafe { c_str(key) };
                semsg!("E716: Key not present in Dictionary: \"{key}\"");
            }
            return GLV_FAIL;
        }
        // SAFETY: `key` is NUL-terminated when `len` is -1, and `len` bytes long otherwise.
        lval.ll_newkey = if len == -1 {
            unsafe { xstrdup(key) }
        } else {
            unsafe { xmemdupz(key as *const c_void, len as size_t) as *mut c_char }
        };
        // SAFETY: the caller's promise about `key_end`.
        unsafe { *key_end = p };
        return GLV_STOP;
    }

    // An existing item: check it may be changed.
    // SAFETY: `ll_di` is a live item, and `p` and `name` are cursors into the one string.
    let di_flags = unsafe { (*lval.ll_di).di_flags } as c_int;
    // SAFETY: as above.
    let name_len = unsafe { p.offset_from(name) } as size_t;
    let refused = flags & GLV_READ_ONLY as c_int == 0
        && (unsafe { var_check_ro(di_flags, name, name_len) }
            || unsafe { var_check_lock(di_flags, name, name_len) });
    if refused {
        return GLV_FAIL;
    }

    // SAFETY: `ll_di` is a live item, whose typval is the target.
    lval.ll_tv = unsafe { &raw mut (*lval.ll_di).di_tv };
    lval.ll_lock = di_lock(lval.ll_di);
    GLV_OK
}

/// Resolve a `[n]` or `[n:m]` subscript against the Blob in `lval.ll_tv`.
/// Leaves `ll_tv` null, which is what tells `set_var_lval` this is a Blob.
///
/// # Safety
/// `lval` must be valid with `ll_tv` a Blob; `var1`/`var2` valid.
pub(crate) unsafe fn get_lval_blob(
    lval: *mut LVal,
    var1: &mut TypVal,
    var2: &mut TypVal,
    empty1: bool,
    quiet: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise: `ll_tv` holds a Blob, so `v_blob` is live.
    let mut lval = unsafe { Lv::new(lval) };
    // SAFETY: as above.
    let bloblen = unsafe { tv_blob_len(Tv::new(lval.ll_tv).blob_or_null()) };
    lval.ll_n1 = if empty1 {
        0
    } else {
        tv_get_number(var1) as c_int
    };
    let n1 = lval.ll_n1 as VarNumber;
    tv_blob_check_index(bloblen, n1, quiet)?;
    if lval.ll_range && !lval.ll_empty2 {
        lval.ll_n2 = tv_get_number(var2) as c_int;
        let n2 = lval.ll_n2 as VarNumber;
        tv_blob_check_range(bloblen, n1, n2, quiet)?;
    }
    // SAFETY: as above -- the typval still holds the Blob.
    lval.ll_blob = unsafe { Tv::new(lval.ll_tv).blob_or_null() };
    lval.ll_tv = null_mut();
    lval.ll_lock = null_mut();
    Ok(())
}

/// Resolve a `[n]` or `[n:m]` subscript against the List in `lval.ll_tv`,
/// leaving `ll_tv` on the item it selected.
///
/// # Safety
/// `lval` must be valid with `ll_tv` a List; `var1`/`var2` valid.
pub(crate) unsafe fn get_lval_list(
    lval: *mut LVal,
    var1: &mut TypVal,
    var2: &mut TypVal,
    empty1: bool,
    _flags: c_int,
    quiet: bool,
) -> Result<(), Failed> {
    // The two range callees write back through the addresses of `ll_n1` and
    // `ll_n2`, so those addresses are live across the rest of this body.
    // Every field written below therefore goes through the pointer rather
    // than through `DerefMut`, which would borrow the whole record and pop
    // them — `winlayer::live`'s note, and the bug it names in
    // `set_buflocal_cpt_callbacks`.
    // SAFETY: the caller's promise: `lval` outlives the call with `ll_tv`
    // holding a List.
    let lval = unsafe { Lv::new(lval) };
    let (rec, n1, n2) = (
        lval.raw(),
        lval.field_ptr::<c_int>(offset_of!(LVal, ll_n1)),
        lval.field_ptr::<c_int>(offset_of!(LVal, ll_n2)),
    );
    let first = if empty1 {
        0
    } else {
        tv_get_number(var1) as c_int
    };
    // SAFETY: `VAR_LIST` says the value holds a List, and
    // `rec` is the caller's record.
    unsafe {
        *n1 = first;
        (*rec).ll_dict = null_mut::<Dict>();
        (*rec).ll_list = Tv::new((*rec).ll_tv).list_or_null();
    };
    // SAFETY: `ll_list` is the typval's List and `n1` is `lval`'s own field.
    let (list, at) = unsafe {
        let list = (*rec).ll_list;
        let at = tv_list_check_range_index_one(list, n1, quiet);
        (*rec).ll_li = at.unwrap_or(0);
        (list, at)
    };
    let Some(at) = at else {
        return Err(Failed);
    };
    // SAFETY: `rec` is the caller's record.
    let ranged = unsafe { (*rec).ll_range && !(*rec).ll_empty2 };
    if ranged {
        // SAFETY: `var2` is the caller's second index expression, and both
        // indexes are `lval`'s own fields.
        unsafe { *n2 = tv_get_number(var2) as c_int };
        // SAFETY: `at` is the index one selected.
        unsafe { tv_list_check_range_index_two(list, n1, at, n2, quiet) }?;
    }
    // SAFETY: `ll_li` is an index of the list, whose item is the target.
    let item = &raw mut unsafe { tv_list_items_mut(list) }[at];
    unsafe { (*rec).ll_tv = &raw mut (*item).li_tv };
    unsafe { (*rec).ll_lock = &raw mut (*item).li_lock };
    Ok(())
}

/// The subscript walk's per-call state: everything one turn of it needs.
///
/// The same shape as [`Live<T>`](crate::winlayer::Live), and for the same
/// reason -- **construction is the one unsafe step**. Whoever builds one
/// promises that `lval` outlives it with `ll_tv` on a live typval, that
/// `name` is a writable NUL-terminated string with `cursor` pointing into
/// it, and that `result` is the value about to be assigned; every method
/// below is ordinary checked code resting on that, and each union member it
/// reads is the one the `v_type` it just tested names.
struct Subscripts<'a> {
    /// The record being filled in. Its `ll_tv` names the container the next
    /// subscript selects into, and moves down to whatever that selected.
    lval: Lv,
    /// The whole left-hand side, for the messages that name it.
    name: *mut c_char,
    /// Where in `name` the walk has got to. The caller reads it back.
    cursor: Cur,
    /// The value about to be assigned, or `None` for an `:unlet`, which
    /// assigns nothing.
    result: Option<&'a mut TypVal>,
    unlet: bool,
    flags: c_int,
    /// `GLV_QUIET`: resolve, but report nothing.
    quiet: bool,
    /// This walk's own evaluation state, for the index expressions.
    evalarg: EvalArg,
}

/// What one subscript's index text came to.
struct Index {
    /// A `.key`'s text, or null when the key is `var1`'s string.
    key: *mut c_char,
    /// `key`'s length, or -1 when the key is `var1`'s string.
    len: c_int,
    /// Whether a `[:m]` left the first half out.
    empty1: bool,
}

impl Subscripts<'_> {
    /// Which subscript the cursor opens -- `.` for a key, `[` for an index
    /// -- or `None` when it opens none and the walk is over. `.=` and `..`
    /// are the concat operators, not a key.
    fn opener(&self) -> Option<u8> {
        let c = self.cursor.byte();
        let opens =
            c == b'[' || (c == b'.' && self.cursor.at(1) != b'=' && self.cursor.at(1) != b'.');
        opens.then_some(c)
    }

    /// The container `ll_tv` names, which every step reads afresh:
    /// evaluating an index expression runs user code, and that may have
    /// replaced what the name resolved to.
    fn container(&self) -> Tv {
        // SAFETY: `ll_tv` is the live container being subscripted.
        unsafe { Tv::new(self.lval.ll_tv) }
    }

    /// The checks a container has to pass before a subscript may select
    /// into it, and the empty List or Blob that a null one stands in for.
    ///
    /// `None` refuses, with the reason already reported.
    fn open_container(&mut self, opener: u8) -> Option<()> {
        let container = self.container();
        if opener == b'.' && container.v_type() != VAR_DICT {
            if !self.quiet {
                // SAFETY: a shared message, whose format takes one
                // NUL-terminated string.
                let name = unsafe { c_str(self.name) };
                semsg!("E1203: Dot can only be used on a dictionary: {name}");
            }
            return None;
        }
        if container.v_type() != VAR_LIST
            && container.v_type() != VAR_DICT
            && container.v_type() != VAR_BLOB
        {
            if !self.quiet {
                emsg_static(c"E689: Can only index a List, Dictionary or Blob");
            }
            return None;
        }

        // A null List or Blob works like an empty one; allocate now.
        // SAFETY: `ll_tv` is the live container being subscripted.
        let target = unsafe { &mut *self.lval.ll_tv };
        if container.v_type() == VAR_LIST && container.list_or_null().is_null() {
            tv_list_alloc_ret(target, kListLenUnknown as ptrdiff_t);
        } else if container.v_type() == VAR_BLOB && container.blob_or_null().is_null() {
            // SAFETY: as above.
            unsafe { tv_blob_alloc_ret(target) };
        }

        if self.lval.ll_range {
            if !self.quiet {
                emsg_static(c"E708: [:] must come last");
            }
            return None;
        }
        Some(())
    }

    /// The `.key` at the cursor, which is a run of name characters taken
    /// out of the command line where it stands.
    fn parse_key(&mut self) -> Option<Index> {
        // SAFETY, for every region below: the cursor is inside the
        // NUL-terminated name, so the byte after the `.` is inside it too,
        // and the walk stops at the first byte that is not a name
        // character -- the terminator included.
        let key = unsafe { self.cursor.get().add(1) };
        let mut len: c_int = 0;
        loop {
            let b = unsafe { *key.offset(len as isize) } as u8;
            if !(b.is_ascii_alphabetic() || ascii_isdigit(b.into()) || b == b'_') {
                break;
            }
            len += 1;
        }
        if len == 0 {
            if !self.quiet {
                emsg_static(c"E713: Cannot use empty key after .");
            }
            return None;
        }
        // SAFETY: as above.
        self.cursor.set(unsafe { key.offset(len as isize) });
        Some(Index {
            key,
            len,
            empty1: false,
        })
    }

    /// The `[expr]` or `[expr : expr]` at the cursor, evaluated into `var1`
    /// and `var2`. A range sets `ll_range`, and `ll_empty2` for a `[n:]`.
    fn parse_index(&mut self, var1: &mut TypVal, var2: &mut TypVal) -> Option<Index> {
        // `skip` steps past the `[` and then past the white space.
        self.cursor.skip(1);
        let empty1 = self.cursor.byte() == b':';
        if !empty1 {
            self.eval_into(var1)?;
            self.cursor.skip(0);
        }

        if self.cursor.byte() == b':' {
            self.parse_range(var2)?;
        } else {
            self.lval.ll_range = false;
        }

        if self.cursor.byte() != b']' {
            if !self.quiet {
                emsg_static(e_missbrac);
            }
            return None;
        }
        self.cursor.bump(1);
        Some(Index {
            key: null_mut(),
            len: -1,
            empty1,
        })
    }

    /// One index expression, evaluated at the cursor into `var` and checked
    /// for being usable as a string.
    fn eval_into(&mut self, var: &mut TypVal) -> Option<()> {
        // SAFETY: the cursor walks the NUL-terminated name, and `var` and
        // `evalarg` are this walk's own.
        let evaluated = unsafe { eval1(self.cursor.raw(), var, &raw mut self.evalarg) };
        (evaluated.is_ok() && tv_check_str(var)).then_some(())
    }

    /// The `: expr]` half of a `[n : m]`, with the cursor on the colon.
    fn parse_range(&mut self, var2: &mut TypVal) -> Option<()> {
        if self.container().v_type() == VAR_DICT {
            if !self.quiet {
                emsg_static(e_cannot_slice_dictionary);
            }
            return None;
        }
        // The value being assigned has to be sliceable too. No `result` is
        // `:unlet`, which assigns nothing.
        let sliceable = self.result.as_deref().is_none_or(|v| {
            (v.v_type() == VAR_LIST && !v.list_or_null().is_null())
                || (v.v_type() == VAR_BLOB && !v.blob_or_null().is_null())
        });
        if !sliceable {
            if !self.quiet {
                emsg_static(c"E709: [:] requires a List or Blob value");
            }
            return None;
        }
        // Past the `:` and the white space after it.
        self.cursor.skip(1);
        if self.cursor.byte() == b']' {
            self.lval.ll_empty2 = true;
        } else {
            self.lval.ll_empty2 = false;
            self.eval_into(var2)?;
        }
        self.lval.ll_range = true;
        Some(())
    }

    /// Select `index` out of the container `ll_tv` names, leaving `ll_tv`
    /// on what it selected. `Break` when there is nothing left to descend
    /// into: a Blob byte, or a Dictionary key that does not exist yet.
    fn descend(
        &mut self,
        index: &Index,
        var1: &mut TypVal,
        var2: &mut TypVal,
    ) -> Option<ControlFlow<()>> {
        let kind = self.container().v_type();
        let (rec, name) = (self.lval.raw(), self.name);
        let (flags, unlet, quiet) = (self.flags, self.unlet, self.quiet);
        let (key, len, empty1) = (index.key, index.len, index.empty1);
        if kind == VAR_DICT {
            let value = self.result.as_deref_mut();
            let end = self.cursor.raw();
            // SAFETY: the promise `Subscripts` records, and `end` names
            // this walk's own cursor.
            let status =
                unsafe { get_lval_dict_item(rec, name, key, len, end, var1, flags, unlet, value) };
            match status {
                GLV_FAIL => None,
                // The key is new: `ll_newkey` holds it and there is nothing
                // left to descend into.
                GLV_STOP => Some(ControlFlow::Break(())),
                _ => Some(ControlFlow::Continue(())),
            }
        } else if kind == VAR_BLOB {
            // SAFETY: as above.
            unsafe { get_lval_blob(rec, var1, var2, empty1, quiet) }.ok()?;
            // A Blob byte is never a container, so this is the end.
            Some(ControlFlow::Break(()))
        } else {
            // SAFETY: as above.
            unsafe { get_lval_list(rec, var1, var2, empty1, flags, quiet) }.ok()?;
            Some(ControlFlow::Continue(()))
        }
    }

    /// Every subscript at the cursor, one container at a time.
    ///
    /// `var1` and `var2` are the caller's, so that a refusal part-way
    /// through still leaves whichever was evaluated for it to release.
    fn walk(&mut self, var1: &mut TypVal, var2: &mut TypVal) -> Option<()> {
        while let Some(opener) = self.opener() {
            self.open_container(opener)?;
            let index = if opener == b'.' {
                self.parse_key()?
            } else {
                self.parse_index(var1, var2)?
            };
            if self.descend(&index, var1, var2)?.is_break() {
                break;
            }
            clear_local(var1);
            clear_local(var2);
            var1.write_empty(VAR_UNKNOWN);
            var2.write_empty(VAR_UNKNOWN);
        }
        Some(())
    }
}

/// Walk every `[idx]` and `.key` following the name, descending `lval.ll_tv`
/// one container at a time. Answers the cursor after the last subscript, or
/// null on an error.
///
/// # Safety
/// `lval` must be valid with `ll_tv` set; `p` must point into the
/// NUL-terminated `name`.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn get_lval_subscript(
    lval: *mut LVal,
    mut p: *mut c_char,
    name: *mut c_char,
    result: Option<&mut TypVal>,
    _ht: *mut DictTab,
    _v: *mut DictItem,
    unlet: bool,
    flags: c_int,
) -> *mut c_char {
    let mut walk = Subscripts {
        // SAFETY: the caller's promise, which is the one `Subscripts`
        // records -- `lval` outlives the call with `ll_tv` on a live
        // typval, and `p` is a cursor into the writable NUL-terminated
        // left-hand side `name`.
        lval: unsafe { Lv::new(lval) },
        name,
        // SAFETY: as above -- `p` is this frame's own from here on.
        cursor: unsafe { Cur::new(&raw mut p) },
        result,
        unlet,
        flags,
        quiet: flags & GLV_QUIET as c_int != 0,
        evalarg: EVALARG_EVALUATE,
    };
    // The two index expressions. They outlive the walk, so a refusal
    // part-way through still releases whichever of them was evaluated.
    let mut var1 = UNSET_TV;
    let mut var2 = UNSET_TV;
    let walked = walk.walk(&mut var1, &mut var2);
    clear_local(&mut var1);
    clear_local(&mut var2);
    // `p` is where the walk's cursor left it.
    if walked.is_some() { p } else { null_mut() }
}

/// Resolve the left-hand side of an assignment or an `:unlet` into `lval`,
/// and answer the cursor after it.
///
/// # Safety
/// `name` must be a writable, NUL-terminated string; `lval` must be valid;
/// `result` null or the value about to be assigned.
pub unsafe fn get_lval(
    name: *mut c_char,
    result: Option<&mut TypVal>,
    lval: *mut LVal,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    let quiet = flags & GLV_QUIET as c_int != 0;
    // SAFETY: the caller's promise; every field is written before it is read.
    let mut lval = unsafe { Lv::new(lval) };
    // SAFETY: as above -- the whole record is the caller's.
    unsafe { lval.raw().cast::<u8>().write_bytes(0, size_of::<LVal>()) };

    if skip {
        // Only the name matters; nothing is resolved.
        lval.ll_name = name;
        let fne = FNE_INCL_BR | fne_flags;
        // SAFETY: `name` is NUL-terminated and the walk wants no braces.
        return unsafe { find_name_end(name, null_mut(), null_mut(), fne) } as *mut c_char;
    }

    // `find_name_end` writes `*const` and `make_expanded_name` wants
    // `*mut`; the two spell the same bytes of `name`, which is writable.
    let mut expr_start = null_mut::<c_char>();
    let mut expr_end = null_mut::<c_char>();
    let (starts, ends) = (
        (&raw mut expr_start).cast::<*const c_char>(),
        (&raw mut expr_end).cast::<*const c_char>(),
    );
    // SAFETY: `name` is NUL-terminated and the two out-parameters are this frame's.
    let mut p = unsafe { find_name_end(name, starts, ends, fne_flags) } as *mut c_char;

    if !expr_start.is_null() {
        // A curly-braces name: expand it.
        // SAFETY: `p` is a cursor into the NUL-terminated `name`.
        let after = unsafe { *p };
        if unlet
            && !ascii_iswhite(after as c_int)
            && ends_excmd(after as c_int) == 0
            && after != b'[' as c_char
            && after != b'.' as c_char
        {
            // SAFETY: the format takes one NUL-terminated string.
            let p = unsafe { c_str(p) };
            semsg!("E488: Trailing characters: {p}");
            return null_mut();
        }
        // SAFETY: all four cursors are into the one writable string.
        lval.ll_exp_name = unsafe { make_expanded_name(name, expr_start, expr_end, p) };
        lval.ll_name = lval.ll_exp_name;
        if lval.ll_exp_name.is_null() {
            if !aborting() && !quiet {
                emsg_severe.set(true);
                // SAFETY: the format takes one NUL-terminated string.
                let name = unsafe { c_str(name) };
                semsg!("E475: Invalid argument: {name}");
                return null_mut();
            }
            lval.ll_name_len = 0 as size_t;
        } else {
            // SAFETY: the expansion is NUL-terminated.
            lval.ll_name_len = unsafe { cstr::bytes_at(lval.ll_name) }.len();
        }
    } else {
        lval.ll_name = name;
        // SAFETY: `p` and the name are cursors into the one string.
        lval.ll_name_len = unsafe { p.offset_from(lval.ll_name) } as size_t;
    }

    // Nothing is subscripted: the name is the whole left-hand side.
    // SAFETY: `p` is a cursor into the NUL-terminated `name`.
    let after = unsafe { *p };
    if (after != b'[' as c_char && after != b'.' as c_char) || lval.ll_name.is_null() {
        return p;
    }

    let mut ht: *mut DictTab = null_mut();
    let htp = if flags & GLV_READ_ONLY as c_int != 0 {
        null_mut()
    } else {
        &raw mut ht
    };
    let no_autoload = flags & GLV_NO_AUTOLOAD as c_int != 0;
    // SAFETY: the name is NUL-terminated and `ht` is this frame's.
    let v = unsafe { find_var(lval.ll_name, lval.ll_name_len, htp, no_autoload) };
    if v.is_null() {
        if !quiet {
            let (n, s) = (lval.ll_name_len as c_int, lval.ll_name);
            // SAFETY: as above.
            let s = unsafe { c_str_len(s, n as usize) };
            semsg!("E121: Undefined variable: {s}");
        }
        return null_mut();
    }

    // SAFETY: `v` is the live dictionary item the name resolved to.
    lval.ll_tv = unsafe { &raw mut (*v).di_tv };
    lval.ll_lock = di_lock(v);
    // SAFETY: `ll_tv` is that item's typval.
    if unsafe { tv_is_luafunc(&mut *lval.ll_tv) } {
        return p;
    }

    // SAFETY: `lval` has `ll_tv` set, `p` points into `name`, and `ht` and `v` are this frame's.
    p = unsafe { get_lval_subscript(lval.raw(), p, name, result, ht, v, unlet, flags) };
    if p.is_null() {
        return null_mut();
    }
    // SAFETY: `p` and the name are cursors into the one string.
    lval.ll_name_len = unsafe { p.offset_from(lval.ll_name) } as size_t;
    p
}

/// Release what `get_lval` allocated into `lval`.
///
/// # Safety
/// `lval` must be valid.
pub unsafe fn clear_lval(lval: *mut LVal) {
    // SAFETY: the caller's promise; both strings are `get_lval`'s own.
    let lval = unsafe { Lv::new(lval) };
    // SAFETY: as above -- both are owned, and null is fine for `xfree`.
    unsafe { xfree(lval.ll_exp_name as *mut c_void) };
    // SAFETY: as above.
    unsafe { xfree(lval.ll_newkey as *mut c_void) };
}
