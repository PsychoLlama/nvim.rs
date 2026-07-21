//! Clipboard-provider integration for the `*` and `+` registers: routing
//! register access to the provider, and batching provider updates across
//! script execution.
//!
//! Module state lives in one [`ClipboardState`] behind a [`GlobalCell`];
//! borrows are scoped so they never span a call into the evaluator
//! (`eval_has_provider`/`eval_call_provider` run user code that may
//! reenter this module).

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::cb_flags;
use crate::src::nvim::register::{
    kMTBlockWise, kMTCharWise, kMTLineWise, kMTUnknown, yankreg_T, AdditionalData, String_0,
    PLUS_REGISTER, STAR_REGISTER,
};
pub use crate::src::nvim::types::{
    blob_T, blobvar_S, dict_T, dictvar_S, float_T, funccall_S,
    funccall_S_fc_fixvar as C2Rust_Unnamed, funccall_T, garray_T, hash_T, hashitem_T, hashtab_T,
    int32_t, int64_t, linenr_T, list_T, listitem_S, listitem_T, listvar_S, listwatch_S,
    listwatch_T, partial_S, partial_T, proftime_T, ptrdiff_t, queue, scid_T, sctx_T, size_t,
    ssize_t, typval_T, typval_vval_union, ufunc_S, ufunc_T, uint32_t, uint64_t, uint8_t,
    varnumber_T, BoolVarValue, LuaRef, ScopeDictDictItem, ScopeType, SpecialVarValue, Timestamp,
    VarLockStatus, VarType, QUEUE,
};

extern "C" {
    fn strlen(__s: *const ::core::ffi::c_char) -> size_t;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
    fn cstr_to_string(str: *const ::core::ffi::c_char) -> String_0;
    fn eval_call_provider(
        provider: *mut ::core::ffi::c_char,
        method: *mut ::core::ffi::c_char,
        arguments: *mut list_T,
        discard: bool,
    ) -> typval_T;
    fn eval_has_provider(feat: *const ::core::ffi::c_char, throw_if_fast: bool) -> bool;
    fn msg(s: *const ::core::ffi::c_char, hl_id: ::core::ffi::c_int) -> bool;
    fn emsg(s: *const ::core::ffi::c_char) -> bool;
    fn redirecting() -> ::core::ffi::c_int;
    fn tv_list_alloc(len: ptrdiff_t) -> *mut list_T;
    fn tv_list_append_list(l: *mut list_T, itemlist: *mut list_T);
    fn tv_list_append_string(l: *mut list_T, str: *const ::core::ffi::c_char, len: ssize_t);
    fn get_y_register(reg: ::core::ffi::c_int) -> *mut yankreg_T;
    fn get_y_previous() -> *mut yankreg_T;
    fn update_yankreg_width(reg: *mut yankreg_T);
    fn free_register(reg: *mut yankreg_T);
}
pub const VAR_LIST: VarType = 4;
pub const VAR_STRING: VarType = 2;
pub const VAR_NUMBER: VarType = 1;

pub const kOptCbFlagUnnamed: ::core::ffi::c_uint = 1;
pub const kOptCbFlagUnnamedplus: ::core::ffi::c_uint = 2;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const Ctrl_V: ::core::ffi::c_int = 22;

/// The module's mutable state, all of it.
#[derive(Copy, Clone)]
struct ClipboardState {
    /// Depth of nested `start_batch_changes` scopes.
    batch_change_count: ::core::ffi::c_int,
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
pub unsafe fn adjust_clipboard_name(
    name: &mut ::core::ffi::c_int,
    quiet: bool,
    writing: bool,
) -> *mut yankreg_T {
    let explicit_cb_reg = *name == '*' as ::core::ffi::c_int || *name == '+' as ::core::ffi::c_int;
    let implicit_cb_reg =
        *name == NUL && cb_flags.get() & (kOptCbFlagUnnamed | kOptCbFlagUnnamedplus) != 0;
    if !explicit_cb_reg && !implicit_cb_reg {
        return ::core::ptr::null_mut();
    }

    if !eval_has_provider(b"clipboard\0".as_ptr() as *const ::core::ffi::c_char, false) {
        let warn = CLIPBOARD.with_mut(|st| {
            if st.batch_change_count <= 1
                && !quiet
                && (!st.didwarn || (explicit_cb_reg && redirecting() == 0))
            {
                st.didwarn = true;
                true
            } else {
                false
            }
        });
        if warn {
            // Do not use emsg here: it may interrupt other logic.
            msg(MSG_NO_CLIP.as_ptr(), 0);
        }
        return ::core::ptr::null_mut();
    }

    if explicit_cb_reg {
        let target = get_y_register(if *name == '*' as ::core::ffi::c_int {
            STAR_REGISTER as ::core::ffi::c_int
        } else {
            PLUS_REGISTER as ::core::ffi::c_int
        });
        let flag = if *name == '*' as ::core::ffi::c_int {
            kOptCbFlagUnnamed
        } else {
            kOptCbFlagUnnamedplus
        };
        if writing && cb_flags.get() & flag != 0 {
            CLIPBOARD.with_mut(|st| st.needs_update = false);
        }
        return target;
    }

    // Unnamed register with clipboard= routing to "* or "+.
    let st = CLIPBOARD.get();
    if writing && st.delay_update {
        CLIPBOARD.with_mut(|st| st.needs_update = true);
        return ::core::ptr::null_mut();
    }
    if !writing && st.needs_update {
        // The pending write hasn't reached the provider yet; read our own
        // register instead of stale provider contents.
        return ::core::ptr::null_mut();
    }
    if cb_flags.get() & kOptCbFlagUnnamedplus != 0 {
        *name = if cb_flags.get() & kOptCbFlagUnnamed != 0 && writing {
            '"' as ::core::ffi::c_int
        } else {
            '+' as ::core::ffi::c_int
        };
        get_y_register(PLUS_REGISTER as ::core::ffi::c_int)
    } else {
        *name = '*' as ::core::ffi::c_int;
        get_y_register(STAR_REGISTER as ::core::ffi::c_int)
    }
}

pub const MSG_NO_CLIP: [::core::ffi::c_char; 62] = unsafe {
    ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
        *b"clipboard: No provider. Try \":checkhealth\" or \":h clipboard\".\0",
    )
};

/// Fill `*target` with provider contents for register `name`. Returns
/// false (with the register emptied) when the clipboard is not involved
/// or the provider returned invalid data.
///
/// # Safety
///
/// Main-thread editor call; runs the clipboard provider.
pub unsafe fn get_clipboard(
    mut name: ::core::ffi::c_int,
    target: &mut *mut yankreg_T,
    quiet: bool,
) -> bool {
    let reg = adjust_clipboard_name(&mut name, quiet, false);
    if reg.is_null() {
        return false;
    }
    free_register(reg);

    let args = tv_list_alloc(1);
    let regname = name as ::core::ffi::c_char;
    tv_list_append_string(args, &raw const regname, 1);
    let result = eval_call_provider(
        b"clipboard\0".as_ptr() as *mut ::core::ffi::c_char,
        b"get\0".as_ptr() as *mut ::core::ffi::c_char,
        args,
        false,
    );

    // Show a message on error unless the provider already indicated failure.
    let mut errmsg = true;
    'err: {
        if result.v_type != VAR_LIST {
            if result.v_type == VAR_NUMBER && result.vval.v_number == 0 {
                errmsg = false;
            }
            break 'err;
        }
        let res = result.vval.v_list;
        let lines;
        if tv_list_len(res) == 2 && (*tv_list_first(res)).li_tv.v_type == VAR_LIST {
            lines = (*tv_list_first(res)).li_tv.vval.v_list;
            if (*tv_list_last(res)).li_tv.v_type != VAR_STRING {
                break 'err;
            }
            let regtype = (*tv_list_last(res)).li_tv.vval.v_string;
            if regtype.is_null() || strlen(regtype) > 1 {
                break 'err;
            }
            (*reg).y_type = match *regtype as u8 {
                0 => kMTUnknown,
                b'v' | b'c' => kMTCharWise,
                b'V' | b'l' => kMTLineWise,
                b'b' | 22 => kMTBlockWise, // 22 == Ctrl_V
                _ => break 'err,
            };
        } else {
            lines = res;
            // The provider did not specify a regtype; inferred below.
            (*reg).y_type = kMTUnknown;
        }

        (*reg).y_array = xcalloc(
            tv_list_len(lines) as size_t,
            ::core::mem::size_of::<String_0>(),
        ) as *mut String_0;
        (*reg).y_size = tv_list_len(lines) as size_t;
        (*reg).y_width = 0;
        (*reg).additional_data = ::core::ptr::null_mut::<AdditionalData>();
        // No timestamp: clipboard registers are not saved in the ShaDa file.
        (*reg).timestamp = 0;

        let mut tv_idx: size_t = 0;
        if !lines.is_null() {
            let mut li = (*lines).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type != VAR_STRING {
                    break 'err;
                }
                let s = (*li).li_tv.vval.v_string;
                *(*reg).y_array.add(tv_idx) = cstr_to_string(if !s.is_null() {
                    s
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                });
                tv_idx += 1;
                li = (*li).li_next;
            }
        }

        if (*reg).y_size > 0 && (*(*reg).y_array.add((*reg).y_size - 1)).size == 0 {
            // A known-to-be charwise yank might have a final linebreak, but
            // otherwise there is no line after the final newline.
            if (*reg).y_type != kMTCharWise {
                xfree((*(*reg).y_array.add((*reg).y_size - 1)).data as *mut ::core::ffi::c_void);
                (*reg).y_size -= 1;
                if (*reg).y_type == kMTUnknown {
                    (*reg).y_type = kMTLineWise;
                }
            }
        } else if (*reg).y_type == kMTUnknown {
            (*reg).y_type = kMTCharWise;
        }

        update_yankreg_width(reg);
        *target = reg;
        return true;
    }

    // Error path: leave the register empty.
    if !(*reg).y_array.is_null() {
        for i in 0..(*reg).y_size {
            xfree((*(*reg).y_array.add(i)).data as *mut ::core::ffi::c_void);
        }
        xfree((*reg).y_array as *mut ::core::ffi::c_void);
    }
    (*reg).y_array = ::core::ptr::null_mut();
    (*reg).y_size = 0;
    (*reg).additional_data = ::core::ptr::null_mut();
    (*reg).timestamp = 0;
    if errmsg {
        emsg(b"clipboard: provider returned invalid data\0".as_ptr() as *const ::core::ffi::c_char);
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
pub unsafe fn set_clipboard(mut name: ::core::ffi::c_int, reg: *mut yankreg_T) {
    if adjust_clipboard_name(&mut name, false, true).is_null() {
        return;
    }

    let lines =
        tv_list_alloc((*reg).y_size as ptrdiff_t + ((*reg).y_type != kMTCharWise) as ptrdiff_t);
    for i in 0..(*reg).y_size {
        tv_list_append_string(
            lines,
            (*(*reg).y_array.add(i)).data,
            (*(*reg).y_array.add(i)).size as ssize_t,
        );
    }

    let regtype: ::core::ffi::c_char = match (*reg).y_type {
        kMTLineWise => {
            tv_list_append_string(lines, ::core::ptr::null(), 0);
            b'V' as ::core::ffi::c_char
        }
        kMTCharWise => b'v' as ::core::ffi::c_char,
        kMTBlockWise => {
            tv_list_append_string(lines, ::core::ptr::null(), 0);
            b'b' as ::core::ffi::c_char
        }
        _ => ::std::process::abort(), // kMTUnknown
    };

    let args = tv_list_alloc(3);
    tv_list_append_list(args, lines);
    tv_list_append_string(args, &raw const regtype, 1);
    let regname = [name as ::core::ffi::c_char];
    tv_list_append_string(args, regname.as_ptr(), 1);
    eval_call_provider(
        b"clipboard\0".as_ptr() as *mut ::core::ffi::c_char,
        b"set\0".as_ptr() as *mut ::core::ffi::c_char,
        args,
        true,
    );
}

/// Start a batch: defer provider updates until the matching
/// [`end_batch_changes`]. Nests.
pub fn start_batch_changes() {
    CLIPBOARD.with_mut(|st| {
        st.batch_change_count += 1;
        if st.batch_change_count > 1 {
            return;
        }
        st.delay_update = true;
    });
}

/// End a batch; flush a pending update once the outermost batch closes.
pub fn end_batch_changes() {
    let update = CLIPBOARD.with_mut(|st| {
        st.batch_change_count -= 1;
        if st.batch_change_count > 0 {
            return false;
        }
        st.delay_update = false;
        ::core::mem::replace(&mut st.needs_update, false)
    });
    if update {
        // SAFETY: main-thread editor call, flushing the unnamed register.
        unsafe { set_clipboard(NUL, get_y_previous()) };
    }
}

/// Suspend batching (flushing any pending update); returns the depth for
/// [`restore_batch_count`].
pub fn save_batch_count() -> ::core::ffi::c_int {
    let (save_count, update) = CLIPBOARD.with_mut(|st| {
        let save = st.batch_change_count;
        st.batch_change_count = 0;
        st.delay_update = false;
        (save, ::core::mem::replace(&mut st.needs_update, false))
    });
    if update {
        // SAFETY: main-thread editor call, flushing the unnamed register.
        unsafe { set_clipboard(NUL, get_y_previous()) };
    }
    save_count
}

/// Resume batching at the depth returned by [`save_batch_count`].
pub fn restore_batch_count(save_count: ::core::ffi::c_int) {
    CLIPBOARD.with_mut(|st| {
        assert!(st.batch_change_count == 0);
        st.batch_change_count = save_count;
        if st.batch_change_count > 0 {
            st.delay_update = true;
        }
    });
}

unsafe fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    if l.is_null() {
        return 0;
    }
    (*l).lv_len
}

unsafe fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut();
    }
    (*l).lv_first
}

unsafe fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    if l.is_null() {
        return ::core::ptr::null_mut();
    }
    (*l).lv_last
}
