//! Clipboard-provider integration for the `*` and `+` registers: routing
//! register access to the provider, and batching provider updates across
//! script execution.
//!
//! Module state lives in one [`ClipboardState`] behind a [`GlobalCell`];
//! borrows are scoped so they never span a call into the evaluator
//! (`eval_has_provider`/`eval_call_provider` run user code that may
//! reenter this module).

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::eval_call_provider;
use crate::eval::eval_has_provider;
use crate::eval::typval::{list_iter, tv_list_alloc};
use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::message::{emsg, msg, redirecting};
use crate::option::vars::cb_flags;
use crate::options::{kOptCbFlagUnnamed, kOptCbFlagUnnamedplus};
use crate::register::{
    PLUS_REGISTER, Reg, STAR_REGISTER, get_y_previous, get_y_register, kMTBlockWise, kMTCharWise,
    kMTLineWise, kMTUnknown,
};
use crate::types::{NUL, RegLines, TypVal, VAR_LIST, VAR_NUMBER, VAR_STRING, YankReg, ptrdiff_t};
use core::ffi::{c_char, c_int};

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

/// Resolve register `*name` to a clipboard register's slot, or `None` when
/// the clipboard is not involved (not a clipboard register, no provider, or
/// the access is deferred/satisfied by a pending update).
///
/// May run the provider-detection Vimscript.
pub(crate) fn adjust_clipboard_name(name: &mut c_int, quiet: bool, writing: bool) -> Option<usize> {
    let explicit_cb_reg = *name == '*' as c_int || *name == '+' as c_int;
    let implicit_cb_reg =
        *name == NUL && cb_flags.get() & (kOptCbFlagUnnamed | kOptCbFlagUnnamedplus) != 0;
    if !explicit_cb_reg && !implicit_cb_reg {
        return None;
    }

    // This runs user code, so no borrow of CLIPBOARD may be held across it.
    if !eval_has_provider(c"clipboard", false) {
        // Be silent inside a `:while`, a `:redir` and the like — but always
        // complain the first time. `redirecting` walks the message state,
        // so it is asked outside the cell borrow.
        let st = CLIPBOARD.get();
        let warn = st.batch_change_count <= 1
            && !quiet
            && (!st.didwarn || (explicit_cb_reg && !redirecting()));
        if warn {
            CLIPBOARD.with_mut(|st| st.didwarn = true);
            // Do not use emsg here: it may interrupt other logic.
            msg(MSG_NO_CLIP, 0);
        }
        return None;
    }

    if explicit_cb_reg {
        let star = *name == '*' as c_int;
        let (reg, flag) = if star {
            (STAR_REGISTER, kOptCbFlagUnnamed)
        } else {
            (PLUS_REGISTER, kOptCbFlagUnnamedplus)
        };
        if writing && cb_flags.get() & flag != 0 {
            CLIPBOARD.with_mut(|st| st.needs_update = false);
        }
        return Some(reg as usize);
    }

    // Unnamed register with clipboard= routing to "* or "+.
    let st = CLIPBOARD.get();
    if writing && st.delay_update {
        // For "set" (copy), defer the provider call.
        CLIPBOARD.with_mut(|st| st.needs_update = true);
        return None;
    }
    if !writing && st.needs_update {
        // The pending write hasn't reached the provider yet; read our own
        // register instead of stale provider contents.
        return None;
    }
    let reg = if cb_flags.get() & kOptCbFlagUnnamedplus != 0 {
        *name = if cb_flags.get() & kOptCbFlagUnnamed != 0 && writing {
            '"' as c_int
        } else {
            '+' as c_int
        };
        PLUS_REGISTER
    } else {
        *name = '*' as c_int;
        STAR_REGISTER
    };
    Some(reg as usize)
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

/// What the provider answered, as a register: its lines and its type, or
/// `Err(lines read so far)` for an answer that is not a list of strings (and
/// an optional type).
///
/// The partial lines matter only as upstream's: its error path frees what it
/// had filled in, which leaves the register unset either way.
fn register_from_provider(result: &TypVal) -> Result<YankReg, bool> {
    let not_a_list = |result: &TypVal| {
        // A provider that answered 0 has said it failed: no message.
        !(result.v_type() == VAR_NUMBER && result.number_or_zero() == 0)
    };
    if result.v_type() != VAR_LIST {
        return Err(not_a_list(result));
    }
    let items = list_iter(result.list_ref()).as_slice();
    let (lines, kind) = match items {
        [lines, regtype] if lines.li_tv.v_type() == VAR_LIST => {
            if regtype.li_tv.v_type() != VAR_STRING {
                return Err(true);
            }
            let Some(regtype) = regtype.li_tv.string_ref() else {
                return Err(true);
            };
            if regtype.as_bytes().len() > 1 {
                return Err(true);
            }
            let Some(kind) = regtype_of(regtype.first()) else {
                return Err(true);
            };
            (lines.li_tv.list_ref(), kind)
        }
        // The provider did not specify a regtype; inferred below.
        _ => (result.list_ref(), kMTUnknown),
    };

    let mut reg = YankReg {
        lines: RegLines::NONE,
        kind,
        width: 0,
        // No timestamp: clipboard registers are not saved in the ShaDa file.
        timestamp: 0,
        extra: None,
    };
    let mut text = Vec::with_capacity(list_iter(lines).len());
    for li in list_iter(lines) {
        if li.li_tv.v_type() != VAR_STRING {
            return Err(true);
        }
        text.push(XString::from_cstr(li.li_tv.string_cstr().unwrap_or(c"")));
    }

    if text.last().is_some_and(|last| last.is_empty()) {
        // A known-to-be charwise yank might have a final linebreak, but
        // otherwise there is no line after the final newline.
        if reg.kind != kMTCharWise {
            text.pop();
            if reg.kind == kMTUnknown {
                reg.kind = kMTLineWise;
            }
        }
    } else if reg.kind == kMTUnknown {
        reg.kind = kMTCharWise;
    }
    // A provider that answered no lines still set the register: upstream's
    // array of none, which `@*` replays as nothing rather than failing.
    reg.lines = RegLines::new(text);
    reg.update_width();
    Ok(reg)
}

/// Fill the clipboard register `name` resolves to with the provider's
/// contents. `None` when the clipboard is not involved; otherwise the slot,
/// and false when the provider answered nothing usable -- the register is
/// then left unset.
///
/// Runs the clipboard provider.
pub(crate) fn get_clipboard(mut name: c_int, quiet: bool) -> Option<(usize, bool)> {
    let slot = adjust_clipboard_name(&mut name, quiet, false)?;
    get_y_register(slot as c_int).write(YankReg::clear);

    let args = tv_list_alloc(1);
    let regname = name as c_char;
    args.edit().push_bytes(Some(&[regname.cast_unsigned()]));
    let result = eval_call_provider(c"clipboard", c"get", Some(args), false);

    match register_from_provider(&result) {
        Ok(reg) => {
            get_y_register(slot as c_int).write(|slot| *slot = reg);
            Some((slot, true))
        }
        Err(errmsg) => {
            // Leave the register empty.
            get_y_register(slot as c_int).write(|reg| {
                reg.clear();
                reg.timestamp = 0;
            });
            if errmsg {
                emsg(c"clipboard: provider returned invalid data");
            }
            Some((slot, false))
        }
    }
}

/// Send register `reg` to the provider as register `name`.
///
/// Runs the clipboard provider. `reg`'s type must be known.
pub(crate) fn set_clipboard(mut name: c_int, reg: Reg) {
    if adjust_clipboard_name(&mut name, false, true).is_none() {
        return;
    }

    let (lines, regtype) = reg.read(|reg| {
        // A line-wise or block-wise register carries a trailing empty line.
        let trailing = reg.kind != kMTCharWise;
        let regtype: u8 = match reg.kind {
            kMTLineWise => b'V',
            kMTCharWise => b'v',
            kMTBlockWise => b'b',
            // kMTUnknown, which has no spelling the provider would accept.
            // Upstream aborts here and so does this; every register handed
            // over has been through `get_clipboard` or a yank, both of which
            // decide the type.
            _ => ::std::process::abort(),
        };
        let lines = tv_list_alloc(reg.lines.len() as ptrdiff_t + ptrdiff_t::from(trailing));
        for line in reg.lines.iter() {
            lines.edit().push_bytes(Some(line));
        }
        if trailing {
            lines.edit().push_bytes(None);
        }
        (lines, regtype)
    });

    let args = tv_list_alloc(3);
    args.edit().push_list(Some(lines));
    args.edit().push_bytes(Some(&[regtype]));
    let regname = name as c_char;
    args.edit().push_bytes(Some(&[regname.cast_unsigned()]));
    eval_call_provider(c"clipboard", c"set", Some(args), true);
}

/// Flush a pending update of the unnamed register to the provider.
fn flush_unnamed() {
    // A pending update means something was written, so `""` points at it.
    if let Some(reg) = get_y_previous() {
        set_clipboard(NUL, reg);
    }
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
        flush_unnamed();
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
        flush_unnamed();
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
