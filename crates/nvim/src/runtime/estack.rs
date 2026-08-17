//! The execution stack -- what `<sfile>`, `<stack>` and every error message's
//! "line N of ..." prefix are read from.
//!
//! `exestack` is a stack of [`estack_T`] entries, one per nested thing being
//! executed: a sourced script, a user function, an autocommand.  [`estack_push`]
//! and [`estack_pop`] bracket each of them, and [`estack_sfile`] renders the
//! stack the three ways vimscript can ask for it -- `<sfile>` (the innermost
//! name), `<stack>` (the whole chain, `..`-joined and carrying line numbers),
//! and the `ESTACK_SCRIPT` form that answers "which script *defined* the frame
//! I am in".  [`stacktrace_create`] and [`f_getstacktrace`] are the same data
//! as a list of dicts.
//!
//! The stack lives in a `garray_T`, so [`entries`] hands the rest of the file a
//! plain slice and every walk below is checked code; only the pushes, the FFI
//! calls and the `es_info` union reach for a pointer.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

/// Slack [`estack_sfile`] reserves per entry on top of the name and its type
/// prefix: enough for the `[%d]` line number and the `..` separator.
const SFILE_ENTRY_SLACK: size_t = 26;

/// The execution stack, outermost frame first.
///
/// # Safety
///
/// The slice borrows `exestack`'s buffer, which [`estack_push`] reallocates.
/// No caller may hold it across a push.
unsafe fn entries<'a>() -> &'a [estack_T] {
    let ga = exestack.get();
    if ga.ga_data.is_null() {
        return &[];
    }
    // SAFETY: `exestack` is a garray of `estack_T` whose first `ga_len`
    // elements are live entries, and the caller promises not to hold the
    // borrow across a push.
    unsafe { slice::from_raw_parts(ga.ga_data.cast::<estack_T>(), ga.ga_len as usize) }
}

/// A stack entry with no `es_info` payload yet; the pushers that have one fill
/// it in through the returned pointer.
fn entry_for(es_type: etype_T, name: *mut c_char, lnum: linenr_T) -> estack_T {
    estack_T {
        es_lnum: lnum,
        es_name: name,
        es_type,
        es_info: estack_T_es_info {
            ufunc: ptr::null_mut(),
        },
    }
}

/// Append `entry` to the execution stack and hand back the slot it landed in.
fn push_entry(entry: estack_T) -> *mut estack_T {
    let ga = exestack.ptr();
    // SAFETY: `exestack` is this family's own garray of `estack_T`; `ga_grow`
    // makes room for one more element and leaves `ga_data` non-null, so the
    // slot at `ga_len` is ours to write.
    unsafe {
        ga_grow(ga, 1);
        let slot = (*ga).ga_data.cast::<estack_T>().add((*ga).ga_len as usize);
        slot.write(entry);
        (*ga).ga_len += 1;
        slot
    }
}

/// Push the bottom frame, the one that stands for "not executing anything".
pub unsafe extern "C" fn estack_init() {
    // SAFETY: the pre-grow is upstream's hint that ten frames of nesting are
    // the common case; `exestack` is empty at this point.
    unsafe { ga_grow(exestack.ptr(), 10) };
    push_entry(entry_for(ETYPE_TOP, ptr::null_mut(), 0));
}

/// Add an item to the execution stack.
///
/// Returns the new entry, so the caller can fill in its `es_info`.
pub unsafe extern "C" fn estack_push(
    es_type: etype_T,
    name: *mut c_char,
    lnum: linenr_T,
) -> *mut estack_T {
    push_entry(entry_for(es_type, name, lnum))
}

/// Add a user function to the execution stack.
pub unsafe extern "C" fn estack_push_ufunc(ufunc: *mut ufunc_T, lnum: linenr_T) {
    // SAFETY: `ufunc` is a live user function. `uf_name_exp` is the
    // `<SNR>`-expanded name when one was built; otherwise the name is the
    // struct's trailing inline buffer.
    let name = unsafe {
        if (*ufunc).uf_name_exp.is_null() {
            &raw mut (*ufunc).uf_name as *mut c_char
        } else {
            (*ufunc).uf_name_exp
        }
    };
    let entry = push_entry(entry_for(ETYPE_UFUNC, name, lnum));
    // SAFETY: the slot `push_entry` just wrote to. (Upstream guards this with
    // a null check, but `ga_grow` aborts rather than returning short.)
    unsafe { (*entry).es_info.ufunc = ufunc };
}

/// Take an item off of the execution stack. The bottom frame stays.
pub unsafe extern "C" fn estack_pop() {
    exestack.with_mut(|ga| {
        if ga.ga_len > 1 {
            ga.ga_len -= 1;
        }
    });
}

/// The frame being executed, or `None` when nothing is.
///
/// Upstream cannot express the `None`: `SOURCING_NAME`/`SOURCING_LNUM` index
/// the stack unconditionally and rely on [`estack_init`] having run.
fn innermost() -> Option<estack_T> {
    // SAFETY: the borrow ends with this expression.
    unsafe { entries() }.last().copied()
}

/// The line number the innermost frame is on -- upstream's `SOURCING_LNUM`.
pub(crate) fn sourcing_lnum() -> linenr_T {
    innermost().map_or(0, |entry| entry.es_lnum)
}

/// The name of the innermost frame -- upstream's `SOURCING_NAME`. Null when
/// nothing is executing, which is also what the bottom frame holds.
pub(crate) fn sourcing_name() -> *mut c_char {
    innermost().map_or(ptr::null_mut(), |entry| entry.es_name)
}

/// Move the innermost frame to `lnum`.
pub(crate) fn set_sourcing_lnum(lnum: linenr_T) {
    let ga = exestack.get();
    if ga.ga_len > 0 {
        // SAFETY: the top entry of a non-empty `exestack`.
        unsafe { (*ga.ga_data.cast::<estack_T>().add(ga.ga_len as usize - 1)).es_lnum = lnum };
    }
}

/// The current value for `<sfile>`, `<stack>` or `<script>`, in allocated
/// memory.
///
/// `which` is `ESTACK_SFILE` for `<sfile>`, `ESTACK_STACK` for `<stack>` or
/// `ESTACK_SCRIPT` for `<script>`.
pub unsafe extern "C" fn estack_sfile(which: estack_arg_T) -> *mut c_char {
    // SAFETY: nothing reached from here pushes onto the stack, so the borrow
    // outlives every use below.
    let stack = unsafe { entries() };
    let Some(innermost) = stack.last() else {
        return ptr::null_mut();
    };

    if which == ESTACK_SFILE && innermost.es_type != ETYPE_UFUNC {
        if innermost.es_name.is_null() {
            return ptr::null_mut();
        }
        // SAFETY: a non-null `es_name` is the frame's NUL-terminated name.
        return unsafe { xstrdup(innermost.es_name) };
    }

    // Evaluated in a function or an autocommand: report the script that
    // *defined* it. At script level the current script's path is the answer.
    if which == ESTACK_SCRIPT {
        // SAFETY: same borrow; `defining_script` allocates but never pushes.
        return unsafe { defining_script(stack) };
    }

    // SAFETY: same borrow, same reason.
    unsafe { render_stack(stack, which) }
}

/// Walk out from the innermost frame until something says which script we are
/// running under, and return a copy of that script's path.
///
/// # Safety
///
/// `stack` must be a live borrow of `exestack`.
unsafe fn defining_script(stack: &[estack_T]) -> *mut c_char {
    for entry in stack.iter().rev() {
        match entry.es_type {
            ETYPE_UFUNC | ETYPE_AUCMD => {
                // SAFETY: `es_info`'s live union member is keyed by `es_type`,
                // and both payloads outlive the frame that names them.
                let def_ctx = unsafe {
                    if entry.es_type == ETYPE_UFUNC {
                        (*entry.es_info.ufunc).uf_script_ctx
                    } else {
                        (*entry.es_info.aucmd).script_ctx
                    }
                };
                if def_ctx.sc_sid <= 0 {
                    return ptr::null_mut();
                }
                // SAFETY: a positive `sc_sid` is an index into `script_items`.
                return unsafe { xstrdup((*script_item(def_ctx.sc_sid)).sn_name) };
            }
            // SAFETY: a script frame's `es_name` is its path.
            ETYPE_SCRIPT => return unsafe { xstrdup(entry.es_name) },
            _ => {}
        }
    }
    ptr::null_mut()
}

/// Compose the whole stack up to the root, the way it has always been spelled:
/// `"function One[123]..Two[456]..Three"`.
///
/// Returns allocated memory, or null when no frame had a name.
///
/// # Safety
///
/// `stack` must be a live borrow of `exestack`.
unsafe fn render_stack(stack: &[estack_T], which: estack_arg_T) -> *mut c_char {
    let mut ga = GA_EMPTY_INIT_VALUE;
    // SAFETY: `ga` is a local garray; `ga_init` only fills in its fields.
    unsafe { ga_init(&raw mut ga, size_of::<c_char>() as c_int, 100) };

    let innermost = stack.len() - 1;
    let mut last_type: etype_T = ETYPE_SCRIPT;
    for (idx, entry) in stack.iter().enumerate() {
        if entry.es_name.is_null() {
            continue;
        }
        let mut type_name: &CStr = c"";
        if entry.es_type != last_type {
            type_name = match entry.es_type {
                ETYPE_SCRIPT => c"script ",
                ETYPE_UFUNC => c"function ",
                _ => c"",
            };
            last_type = entry.es_type;
        }
        // The bottom entry of `<sfile>` leaves its line number out -- that is
        // what `<slnum>` is for. So does any entry whose number is unset.
        let lnum = if idx == innermost && which != ESTACK_STACK {
            0
        } else {
            entry.es_lnum
        };

        // SAFETY: `es_name` is NUL-terminated, and the grow reserves room for
        // both concatenations plus the `[%d]` and `..` that may follow.
        let name_len = unsafe { strlen(entry.es_name) };
        unsafe {
            ga_grow(
                &raw mut ga,
                (name_len + type_name.count_bytes() + SFILE_ENTRY_SLACK) as c_int,
            );
            ga_concat_len(&raw mut ga, type_name.as_ptr(), type_name.count_bytes());
            ga_concat_len(&raw mut ga, entry.es_name, name_len);
        }
        if lnum != 0 {
            // SAFETY: the grow above left `ga_maxlen - ga_len` writable bytes.
            ga.ga_len += unsafe {
                vim_snprintf_safelen(
                    ga.ga_data.cast::<c_char>().add(ga.ga_len as usize),
                    (ga.ga_maxlen - ga.ga_len) as size_t,
                    c"[%d]".as_ptr(),
                    lnum,
                )
            } as c_int;
        }
        if idx != innermost {
            // SAFETY: as above.
            unsafe { ga_concat_len(&raw mut ga, c"..".as_ptr(), 2) };
        }
    }

    // Only NUL-terminate when not returning null.
    if !ga.ga_data.is_null() {
        // SAFETY: `ga` holds the string built above.
        unsafe { ga_append(&raw mut ga, NUL as uint8_t) };
    }
    ga.ga_data.cast::<c_char>()
}

/// `tv_dict_add_*` take the key and its length separately; upstream spells that
/// pair `S_LEN(key)`.
unsafe fn dict_add_str(d: *mut dict_T, key: &CStr, val: *const c_char) {
    unsafe { tv_dict_add_str(d, key.as_ptr(), key.count_bytes(), val) };
}

unsafe fn dict_add_nr(d: *mut dict_T, key: &CStr, nr: varnumber_T) {
    unsafe { tv_dict_add_nr(d, key.as_ptr(), key.count_bytes(), nr) };
}

/// Append one `getstacktrace()` frame to `l`.
///
/// Exactly one of `fp` (a user function) and `event` (an autocommand event
/// name) is set; a script frame has neither.
unsafe fn stacktrace_push_item(
    l: *mut list_T,
    fp: *mut ufunc_T,
    event: *const c_char,
    lnum: linenr_T,
    filepath: *mut c_char,
) {
    // SAFETY: `l` is the caller's list, and the dict below is freshly
    // allocated, so every `tv_dict_add_*` writes into memory we own until the
    // final append hands the dict to the list.
    unsafe {
        let d = tv_dict_alloc_lock(VAR_FIXED);
        let mut tv = typval_T {
            v_type: VAR_DICT,
            v_lock: VAR_LOCKED,
            vval: typval_vval_union { v_dict: d },
        };
        if !fp.is_null() {
            tv_dict_add_func(d, c"funcref".as_ptr(), c"funcref".count_bytes(), fp);
        }
        if !event.is_null() {
            dict_add_str(d, c"event", event);
        }
        dict_add_nr(d, c"lnum", lnum as varnumber_T);
        dict_add_str(d, c"filepath", filepath);
        tv_list_append_tv(l, &raw mut tv);
    }
}

/// The execution stack as `getstacktrace()` reports it: one dict per frame,
/// outermost first.
pub unsafe extern "C" fn stacktrace_create() -> *mut list_T {
    // SAFETY: nothing below pushes onto the execution stack.
    let stack = unsafe { entries() };
    // SAFETY: a fresh list sized for the frames about to go into it.
    let l = unsafe { tv_list_alloc(stack.len() as ptrdiff_t) };

    for entry in stack {
        match entry.es_type {
            // SAFETY: a script frame's `es_name` is its path.
            ETYPE_SCRIPT => unsafe {
                stacktrace_push_item(
                    l,
                    ptr::null_mut(),
                    ptr::null(),
                    entry.es_lnum,
                    entry.es_name,
                );
            },
            ETYPE_UFUNC => {
                let fp = unsafe { entry.es_info.ufunc };
                // SAFETY: the frame's function outlives the frame.
                let sctx = unsafe { (*fp).uf_script_ctx };
                // SAFETY: `l` and the path below are ours.
                unsafe {
                    stacktrace_push_item(
                        l,
                        fp,
                        ptr::null(),
                        entry.es_lnum + sctx.sc_lnum,
                        script_path(sctx),
                    );
                }
            }
            ETYPE_AUCMD => {
                // SAFETY: the frame's `AutoPatCmd` outlives the frame.
                let sctx = unsafe { (*entry.es_info.aucmd).script_ctx };
                // SAFETY: `l` and the path below are ours.
                unsafe {
                    stacktrace_push_item(
                        l,
                        ptr::null_mut(),
                        entry.es_name,
                        entry.es_lnum + sctx.sc_lnum,
                        script_path(sctx),
                    );
                }
            }
            _ => {}
        }
    }
    l
}

/// The path a frame's defining script was read from, or `""` when it has none
/// (a `-c` argument, a modeline, Lua, ...).
///
/// # Safety
///
/// `sctx` must name a live script context.
unsafe fn script_path(sctx: sctx_T) -> *mut c_char {
    if sctx.sc_sid <= 0 {
        return c"".as_ptr().cast_mut();
    }
    // SAFETY: a positive `sc_sid` indexes `script_items`; passing null for
    // `should_free` asks for the borrowed name.
    unsafe { get_scriptname(sctx, ptr::null_mut()) }
}

/// `getstacktrace()` function
pub unsafe extern "C" fn f_getstacktrace(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: `rettv` is the caller's return slot.
    unsafe { tv_list_set_ret(rettv, stacktrace_create()) };
}
