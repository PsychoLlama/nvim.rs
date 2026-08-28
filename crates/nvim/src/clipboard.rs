//! Clipboard-provider integration for the `*` and `+` registers: routing
//! register access to the provider, and batching provider updates across
//! script execution.
//!
//! Module state lives in one [`ClipboardState`] behind a [`GlobalCell`];
//! borrows are scoped so they never span a call into the evaluator
//! (`eval_has_provider`/`eval_call_provider` run user code that may
//! reenter this module).

#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::cstr_to_string;
use crate::eval::typval::{
    tv_list_alloc, tv_list_append_list, tv_list_append_string, tv_list_first, tv_list_last,
    tv_list_len,
};
use crate::eval::{eval_call_provider, eval_has_provider};
use crate::global_cell::GlobalCell;
use crate::main::cb_flags;
use crate::memory::{xcalloc, xfree};
use crate::message::{emsg, msg, redirecting};
use crate::options::{kOptCbFlagUnnamed, kOptCbFlagUnnamedplus};
use crate::register::{
    PLUS_REGISTER, STAR_REGISTER, free_register, get_y_previous, get_y_register, kMTBlockWise,
    kMTCharWise, kMTLineWise, kMTUnknown, update_yankreg_width,
};
use crate::types::{
    AdditionalData, NUL, String_0, VAR_LIST, VAR_NUMBER, VAR_STRING, ptrdiff_t, size_t, ssize_t,
    yankreg_T,
};
use ::libc::strlen;
use core::ffi::{c_char, c_int, c_void};

/// The message shown once when a clipboard register is used and no provider
/// answers.
const MSG_NO_CLIP: &core::ffi::CStr =
    c"clipboard: No provider. Try \":checkhealth\" or \":h clipboard\".";

/// The module's mutable state, all of it.
#[derive(Copy, Clone)]
struct ClipboardState {
    /// Depth of nested `start_batch_changes` scopes.
    batch_change_count: c_int,
    /// Defer provider "set" calls until the batch ends.
    delay_update: bool,
    /// A deferred update is pending.
    needs_update: bool,
    /// The "no provider" warning was already shown.
    didwarn: bool,
}

static CLIPBOARD: GlobalCell<ClipboardState> = GlobalCell::new(ClipboardState {
    batch_change_count: 0,
    delay_update: false,
    needs_update: false,
    didwarn: false,
});

/// Resolve register `*name` to a clipboard register, or null when the
/// clipboard is not involved (not a clipboard register, no provider, or
/// the access is deferred/satisfied by a pending update).
///
/// # Safety
///
/// Main-thread editor call; may run the provider-detection vimscript.
pub(crate) unsafe fn adjust_clipboard_name(
    name: &mut c_int,
    quiet: bool,
    writing: bool,
) -> *mut yankreg_T {
    let explicit_cb_reg = *name == '*' as c_int || *name == '+' as c_int;
    let implicit_cb_reg =
        *name == NUL && cb_flags.get() & (kOptCbFlagUnnamed | kOptCbFlagUnnamedplus) != 0;
    if !explicit_cb_reg && !implicit_cb_reg {
        return core::ptr::null_mut();
    }

    // SAFETY: main-thread editor call; the feature name is a literal. This
    // runs user code, so no borrow of CLIPBOARD may be held across it.
    if !unsafe { eval_has_provider(c"clipboard".as_ptr(), false) } {
        // Be silent inside a `:while`, a `:redir` and the like — but always
        // complain the first time. `redirecting` walks the message state,
        // so it is asked outside the cell borrow.
        let st = CLIPBOARD.get();
        // SAFETY: main-thread editor call.
        let warn = st.batch_change_count <= 1
            && !quiet
            && (!st.didwarn || (explicit_cb_reg && !unsafe { redirecting() }));
        if warn {
            CLIPBOARD.with_mut(|st| st.didwarn = true);
            // Do not use emsg here: it may interrupt other logic.
            // SAFETY: a NUL-terminated literal.
            unsafe { msg(MSG_NO_CLIP.as_ptr(), 0) };
        }
        return core::ptr::null_mut();
    }

    if explicit_cb_reg {
        let star = *name == '*' as c_int;
        let (reg, flag) = if star {
            (STAR_REGISTER as c_int, kOptCbFlagUnnamed)
        } else {
            (PLUS_REGISTER as c_int, kOptCbFlagUnnamedplus)
        };
        if writing && cb_flags.get() & flag != 0 {
            CLIPBOARD.with_mut(|st| st.needs_update = false);
        }
        // SAFETY: main-thread editor call; the register table is live.
        return unsafe { get_y_register(reg) };
    }

    // Unnamed register with clipboard= routing to "* or "+.
    let st = CLIPBOARD.get();
    if writing && st.delay_update {
        // For "set" (copy), defer the provider call.
        CLIPBOARD.with_mut(|st| st.needs_update = true);
        return core::ptr::null_mut();
    }
    if !writing && st.needs_update {
        // The pending write hasn't reached the provider yet; read our own
        // register instead of stale provider contents.
        return core::ptr::null_mut();
    }
    let reg = if cb_flags.get() & kOptCbFlagUnnamedplus != 0 {
        *name = if cb_flags.get() & kOptCbFlagUnnamed != 0 && writing {
            '"' as c_int
        } else {
            '+' as c_int
        };
        PLUS_REGISTER as c_int
    } else {
        *name = '*' as c_int;
        STAR_REGISTER as c_int
    };
    // SAFETY: main-thread editor call; the register table is live.
    unsafe { get_y_register(reg) }
}

/// The register type a provider's one-character `regtype` names, or `None`
/// for anything else (which is an error).
fn regtype_of(byte: u8) -> Option<c_int> {
    match byte {
        0 => Some(kMTUnknown),
        b'v' | b'c' => Some(kMTCharWise),
        b'V' | b'l' => Some(kMTLineWise),
        b'b' | 22 => Some(kMTBlockWise), // 22 == Ctrl_V
        _ => None,
    }
}

/// Fill `*target` with provider contents for register `name`. Returns
/// false (with the register emptied) when the clipboard is not involved
/// or the provider returned invalid data.
///
/// # Safety
///
/// Main-thread editor call; runs the clipboard provider.
pub(crate) unsafe fn get_clipboard(
    mut name: c_int,
    target: &mut *mut yankreg_T,
    quiet: bool,
) -> bool {
    // SAFETY: main-thread editor call.
    let reg = unsafe { adjust_clipboard_name(&mut name, quiet, false) };
    if reg.is_null() {
        return false;
    }
    // SAFETY: a live register, about to be refilled.
    unsafe { free_register(reg) };

    // SAFETY: main-thread editor call; `regname` outlives the append, and
    // the provider call owns `args` from here on.
    let args = unsafe { tv_list_alloc(1) };
    let regname = name as c_char;
    unsafe { tv_list_append_string(args, &raw const regname, 1) };
    let (provider, method) = (c"clipboard".as_ptr().cast_mut(), c"get".as_ptr().cast_mut());
    let result = unsafe { eval_call_provider(provider, method, args, false) };

    // Show a message on error unless the provider already indicated failure.
    let mut errmsg = true;
    // SAFETY: `result` is the provider's typval, whose list (if any) is
    // alive for this block, and `reg` is the live register being filled.
    'err: {
        if result.v_type != VAR_LIST {
            if result.v_type == VAR_NUMBER && unsafe { result.vval.v_number } == 0 {
                errmsg = false;
            }
            break 'err;
        }
        let res = unsafe { result.vval.v_list };
        let lines;
        if unsafe { tv_list_len(res) } == 2
            && unsafe { (*tv_list_first(res)).li_tv.v_type } == VAR_LIST
        {
            lines = unsafe { (*tv_list_first(res)).li_tv.vval.v_list };
            if unsafe { (*tv_list_last(res)).li_tv.v_type } != VAR_STRING {
                break 'err;
            }
            let regtype = unsafe { (*tv_list_last(res)).li_tv.vval.v_string };
            if regtype.is_null() || unsafe { strlen(regtype) } > 1 {
                break 'err;
            }
            match regtype_of(unsafe { *regtype } as u8) {
                Some(ty) => unsafe { (*reg).y_type = ty },
                None => break 'err,
            }
        } else {
            lines = res;
            // The provider did not specify a regtype; inferred below.
            unsafe { (*reg).y_type = kMTUnknown };
        }

        unsafe {
            (*reg).y_array =
                xcalloc(tv_list_len(lines) as size_t, size_of::<String_0>()) as *mut String_0
        };
        unsafe { (*reg).y_size = tv_list_len(lines) as size_t };
        unsafe { (*reg).y_width = 0 };
        unsafe { (*reg).additional_data = core::ptr::null_mut::<AdditionalData>() };
        // No timestamp: clipboard registers are not saved in the ShaDa file.
        unsafe { (*reg).timestamp = 0 };

        let mut tv_idx: size_t = 0;
        if !lines.is_null() {
            let mut li = unsafe { (*lines).lv_first };
            while !li.is_null() {
                if unsafe { (*li).li_tv.v_type } != VAR_STRING {
                    break 'err;
                }
                let s = unsafe { (*li).li_tv.vval.v_string };
                unsafe {
                    *(*reg).y_array.add(tv_idx) =
                        cstr_to_string(if !s.is_null() { s } else { c"".as_ptr() })
                };
                tv_idx += 1;
                li = unsafe { (*li).li_next };
            }
        }

        if unsafe { (*reg).y_size } > 0
            && unsafe { (*(*reg).y_array.add((*reg).y_size - 1)).is_empty() }
        {
            // A known-to-be charwise yank might have a final linebreak, but
            // otherwise there is no line after the final newline.
            if unsafe { (*reg).y_type } != kMTCharWise {
                unsafe { xfree((*(*reg).y_array.add((*reg).y_size - 1)).data() as *mut c_void) };
                unsafe { (*reg).y_size -= 1 };
                if unsafe { (*reg).y_type } == kMTUnknown {
                    unsafe { (*reg).y_type = kMTLineWise };
                }
            }
        } else if unsafe { (*reg).y_type } == kMTUnknown {
            unsafe { (*reg).y_type = kMTCharWise };
        }

        unsafe { update_yankreg_width(reg) };
        *target = reg;
        return true;
    }

    // Error path: leave the register empty.
    if !unsafe { (*reg).y_array }.is_null() {
        for i in 0..unsafe { (*reg).y_size } {
            unsafe { xfree((*(*reg).y_array.add(i)).data() as *mut c_void) };
        }
        unsafe { xfree((*reg).y_array as *mut c_void) };
    }
    unsafe { (*reg).y_array = core::ptr::null_mut() };
    unsafe { (*reg).y_size = 0 };
    unsafe { (*reg).additional_data = core::ptr::null_mut() };
    unsafe { (*reg).timestamp = 0 };
    if errmsg {
        unsafe { emsg(c"clipboard: provider returned invalid data".as_ptr()) };
    }
    *target = reg;
    false
}

/// Send register `reg` to the provider as register `name`.
///
/// # Safety
///
/// Main-thread editor call; runs the clipboard provider. `reg` must point
/// to a valid register whose y_type is known.
pub(crate) unsafe fn set_clipboard(mut name: c_int, reg: *mut yankreg_T) {
    // SAFETY: main-thread editor call.
    if unsafe { adjust_clipboard_name(&mut name, false, true) }.is_null() {
        return;
    }
    // SAFETY: the caller's live register.
    let reg = unsafe { &*reg };

    // A line-wise or block-wise register carries a trailing empty line.
    let trailing = reg.y_type != kMTCharWise;
    let regtype: c_char = match reg.y_type {
        kMTLineWise => b'V' as c_char,
        kMTCharWise => b'v' as c_char,
        kMTBlockWise => b'b' as c_char,
        // kMTUnknown, which has no spelling the provider would accept.
        // Upstream aborts here and so does this; every register handed over
        // has been through `get_clipboard` or a yank, both of which decide
        // the type.
        _ => ::std::process::abort(),
    };

    // SAFETY: main-thread editor call; `regtype`/`regname` outlive their
    // appends, and the provider call owns `args` from here on.
    let lines = unsafe { tv_list_alloc(reg.y_size as ptrdiff_t + trailing as ptrdiff_t) };
    for i in 0..reg.y_size {
        let line = unsafe { *reg.y_array.add(i) };
        unsafe { tv_list_append_string(lines, line.data(), line.len() as ssize_t) };
    }
    if trailing {
        unsafe { tv_list_append_string(lines, core::ptr::null(), 0) };
    }

    let args = unsafe { tv_list_alloc(3) };
    unsafe { tv_list_append_list(args, lines) };
    unsafe { tv_list_append_string(args, &raw const regtype, 1) };
    let regname = [name as c_char];
    unsafe { tv_list_append_string(args, regname.as_ptr(), 1) };
    let (provider, method) = (c"clipboard".as_ptr().cast_mut(), c"set".as_ptr().cast_mut());
    unsafe { eval_call_provider(provider, method, args, true) };
}

/// Start a batch: defer provider updates until the matching
/// [`end_batch_changes`]. Nests.
pub(crate) fn start_batch_changes() {
    CLIPBOARD.with_mut(|st| {
        st.batch_change_count += 1;
        if st.batch_change_count > 1 {
            return;
        }
        st.delay_update = true;
    });
}

/// End a batch; flush a pending update once the outermost batch closes.
pub(crate) fn end_batch_changes() {
    let update = CLIPBOARD.with_mut(|st| {
        st.batch_change_count -= 1;
        if st.batch_change_count > 0 {
            return false;
        }
        st.delay_update = false;
        core::mem::replace(&mut st.needs_update, false)
    });
    if update {
        // SAFETY: main-thread editor call, flushing the unnamed register.
        unsafe { set_clipboard(NUL, get_y_previous()) };
    }
}

/// Suspend batching (flushing any pending update); returns the depth for
/// [`restore_batch_count`].
pub(crate) fn save_batch_count() -> c_int {
    let (save_count, update) = CLIPBOARD.with_mut(|st| {
        let save = st.batch_change_count;
        st.batch_change_count = 0;
        st.delay_update = false;
        (save, core::mem::replace(&mut st.needs_update, false))
    });
    if update {
        // SAFETY: main-thread editor call, flushing the unnamed register.
        unsafe { set_clipboard(NUL, get_y_previous()) };
    }
    save_count
}

/// Resume batching at the depth returned by [`save_batch_count`].
pub(crate) fn restore_batch_count(save_count: c_int) {
    CLIPBOARD.with_mut(|st| {
        debug_assert!(st.batch_change_count == 0);
        st.batch_change_count = save_count;
        if st.batch_change_count > 0 {
            st.delay_update = true;
        }
    });
}
