//! Turning what the user wrote into the name a `UserFunc` is stored under.
//!
//! [`trans_function_name`] is the whole of it: it resolves `s:`/`<SID>` to
//! the `<SNR>N_` mangling, evaluates a curly-brace name, follows a
//! dictionary subscript to a numbered function, and rejects the spellings
//! that are not names at all. It reads a slice of the command line and
//! answers how much of it the name took. [`fname_trans_sid`] is the smaller
//! mangling a call applies, and [`builtin_function`] is what decides a name
//! belongs to the builtin table instead.

#![forbid(unsafe_code)]

use crate::charset::skip;
use crate::eval::lval::{LValue, Slot as LvalSlot, Target};
use crate::eval::typval::{DictRef, PartialRef};
use crate::eval::vars::with_var;
use crate::mbyte::strnicmp_in;
use crate::memory::XString;
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::semsg;
use crate::tr_plural;
use core::ffi::{CStr, c_int};
use std::borrow::Cow;

use super::*;
use crate::keycodes::KE_SNR;

/// The bytes a script-local name is stored behind: `K_SPECIAL KS_EXTRA
/// KE_SNR`, which a listing shows as `<SNR>`.
const SNR: [u8; 3] = [0x80, 253, 82];
const _: () = assert!(SNR[0] as c_int == K_SPECIAL);
const _: () = assert!(SNR[1] as c_int == KS_EXTRA);
const _: () = assert!(SNR[2] as c_int == KE_SNR as c_int);

/// The byte at `i`, or a NUL past the end -- how the C read its string.
fn byte(text: &[u8], i: usize) -> u8 {
    text.get(i).copied().unwrap_or(0)
}

/// What a name, read as a variable, turned out to call.
pub(crate) struct Dereffed {
    /// The function a Funcref or a partial in the variable names; `None`
    /// when there is no such variable, or it holds something else.
    pub(crate) name: Option<XString>,
    /// The partial it came out of.
    pub(crate) partial: Option<PartialRef>,
    /// Whether a variable of that name exists at all.
    pub(crate) found_var: bool,
}

/// The function the variable `name` holds, when it holds one.
///
/// Both answers are the caller's own: a call may delete the variable they
/// were read from. Looking the *variable* up is also what autoloads
/// `pkg#name`'s package: `find_var` sources it on the way.
pub(crate) fn deref_func_name(name: &[u8], no_autoload: bool) -> Dereffed {
    with_var(name, no_autoload, |item| {
        let tv = &item.di_tv;
        // The function's own name; a missing one is the empty string.
        let callee = || XString::from_bytes(tv.callable_name().map_or(&b""[..], CStr::to_bytes));
        match tv {
            TypVal::Func(_) => Dereffed {
                name: Some(callee()),
                partial: None,
                found_var: true,
            },
            TypVal::Partial(partial) => Dereffed {
                name: Some(callee()),
                partial: (**partial).clone(),
                found_var: true,
            },
            _ => Dereffed {
                name: None,
                partial: None,
                found_var: true,
            },
        }
    })
    .unwrap_or(Dereffed {
        name: None,
        partial: None,
        found_var: false,
    })
}

/// [`deref_func_name`], answering `name` itself when it is not a function
/// variable.
pub(crate) fn deref_func_name_owned(
    name: &[u8],
    no_autoload: bool,
) -> (XString, Option<PartialRef>, bool) {
    let found = deref_func_name(name, no_autoload);
    let resolved = found.name.unwrap_or_else(|| XString::from_bytes(name));
    (resolved, found.partial, found.found_var)
}

/// Report `errmsg` about `name`, rendering the `<SNR>` mangling back into
/// something a user can read. `errmsg` is untranslated and has one `%s`.
pub(crate) fn emsg_funcname(errmsg: &'static CStr, name: &[u8]) {
    let shown: Cow<'_, [u8]> =
        if c_int::from(byte(name, 0)) == K_SPECIAL && byte(name, 1) != 0 && byte(name, 2) != 0 {
            let mut shown = b"<SNR>".to_vec();
            shown.extend_from_slice(&name[3..]);
            Cow::Owned(shown)
        } else {
            Cow::Borrowed(name)
        };
    emsg_text(tr_plural!(gettext(errmsg), msg_bytes(&shown)));
}

/// Whether a script-local prefix was written `s:` rather than `<SNR>` --
/// which decides whether the *current* script id has to be substituted in.
/// `name` starts with a prefix [`fname_script_len`] accepted.
fn fname_is_sid(name: &[u8]) -> bool {
    byte(name, 0) == b's' || byte(name, 2).eq_ignore_ascii_case(&b'I')
}

/// `name` with an `s:`/`<SID>`/`<SNR>` prefix rewritten into the `<SNR>N_`
/// byte sequence, or `name` itself when it has none; and `FCERR_SCRIPT`
/// when `s:` was written outside a script, which leaves the number out.
pub(crate) fn fname_trans_sid(name: &[u8]) -> (Cow<'_, [u8]>, c_int) {
    let lead = fname_script_len(name);
    if lead == 0 {
        // "name" doesn't start with "s:" or "<SID>".
        return (Cow::Borrowed(name), FCERR_NONE);
    }
    let script_name = &name[lead..];
    let mut error = FCERR_NONE;
    let mut fname = Vec::with_capacity(SNR.len() + 12 + script_name.len() + 1);
    fname.extend_from_slice(&SNR);
    if fname_is_sid(name) {
        let sid = current_sctx.get().sc_sid;
        if sid <= 0 {
            error = FCERR_SCRIPT;
        } else {
            fname.extend_from_slice(format!("{sid}_").as_bytes());
        }
    }
    // "<SNR>" keeps the digits it was written with.
    fname.extend_from_slice(script_name);
    (Cow::Owned(fname), error)
}

/// The function stored under `name`, or null.
pub(crate) fn find_func(name: &[u8]) -> *mut UserFunc {
    let hi = func_table().find_bytes(name);
    if hi.is_kept() {
        uf_from_name_ptr(hi.hi_key)
    } else {
        ptr::null_mut()
    }
}

/// Whether a function of this name is reference-counted: the numbered
/// dictionary functions and the lambdas, and nothing else.
pub(crate) fn func_name_refcount(name: &[u8]) -> bool {
    byte(name, 0).is_ascii_digit() || name.starts_with(b"<l")
}

/// Whether `name` names a builtin function: it starts lowercase, is not a
/// scoped name, and carries no `#` (which would make it an autoload name).
pub(crate) fn builtin_function(name: &[u8]) -> bool {
    builtin_function_in(name, name.len())
}

/// [`builtin_function`] of `text[..len]`. The scope test reads the byte
/// after the first even when the name is one byte long, as the C did,
/// which is why it gets the text the name is the start of.
fn builtin_function_in(text: &[u8], len: usize) -> bool {
    if !byte(text, 0).is_ascii_lowercase() || byte(text, 1) == b':' {
        return false;
    }
    let name = text.get(..len).unwrap_or(text);
    !name.contains(&b'#')
}

/// The dictionary entry a `dict.func` name selected: what `:function`
/// writes, `:delfunction` removes and `:call` binds `self` to.
///
/// The entry is found again by its key whenever it is used: reading a
/// function body or evaluating the arguments of a call runs code that may
/// remove it.
pub(crate) struct FuncDict {
    /// The dictionary, held across whatever runs before it is used.
    pub(crate) dict: DictRef,
    /// The key the function is, or is to be, under.
    pub(crate) key: Vec<u8>,
    /// The key was not in the dictionary when the name was read.
    pub(crate) new_key: bool,
}

/// What [`trans_function_name`] read.
#[derive(Default)]
pub(crate) struct FunctionName {
    /// The name the function is stored under; `None` when there is not one
    /// there, which has been reported unless the name was a `dict.key` that
    /// does not exist yet.
    pub(crate) name: Option<XString>,
    /// How much of the text the name took. It stays at 0 where the C left
    /// its cursor alone.
    pub(crate) end: usize,
    /// The dictionary entry a `dict.func` selected, for a caller that asked.
    pub(crate) dict: Option<FuncDict>,
    /// The partial the name selected, or held in the variable it named.
    pub(crate) partial: Option<PartialRef>,
}

/// Build the stored name out of a resolved lvalue: strip the scope prefix,
/// prepend the `<SNR>` mangling when the name is script-local, and reject
/// the two spellings that cannot be function names.
///
/// `name_len` is the lvalue's name length, measured in `expanded` when the
/// name had curly braces and in `text` from `start` otherwise; `end` is
/// where the name ends in `text`. `lead` is [`fname_script_len`]'s answer
/// (0, 2 or 5).
#[allow(clippy::too_many_arguments)]
fn mangle_function_name(
    text: &[u8],
    expanded: Option<&[u8]>,
    mut name_len: usize,
    start: usize,
    end: usize,
    mut lead: usize,
    skip: bool,
    flags: c_int,
) -> Option<XString> {
    let source = expanded.unwrap_or(text);
    let mut at;
    let len;
    if let Some(expanded) = expanded {
        at = 0;
        let mut whole = expanded.len();
        if lead <= 2 && name_len >= 2 && expanded.starts_with(b"s:") {
            // When there was "s:" already, or the name expanded to get a
            // leading "s:", remove it.
            at = 2;
            name_len -= 2;
            whole -= 2;
            lead = 2;
        }
        len = whole;
    } else {
        at = start;
        // Skip over "s:" and "g:". In skip mode the length can be shorter
        // than the prefix, which nothing then reads (`skip` forces `lead`
        // to 0 and gates the E884 check).
        if lead == 2 || (byte(text, at) == b'g' && byte(text, at + 1) == b':') {
            at += 2;
            name_len = name_len.saturating_sub(2);
        }
        len = end.saturating_sub(at);
    }

    // Accept <SID>name() inside a script, translated into <SNR>123_name();
    // accept <SNR>123_name() outside one.
    let mut sid = Vec::new();
    if skip {
        lead = 0; // do nothing
    } else if lead > 0 {
        lead = SNR.len();
        if expanded.is_some_and(fname_is_sid) || fname_is_sid(text) {
            // It's "s:" or "<SID>".
            let sc_sid = current_sctx.get().sc_sid;
            if sc_sid <= 0 {
                emsg(gettext(e_usingsid));
                return None;
            }
            sid = format!("{sc_sid}_").into_bytes();
        }
    } else if flags & TFN_INT == 0 && builtin_function_in(&source[at..], name_len) {
        let start = msg_bytes(&text[start..]);
        semsg!("E128: Function name must start with a capital or \"s:\": {start}");
        return None;
    }

    if !skip && flags & TFN_QUIET == 0 && flags & TFN_NO_DEREF == 0 {
        // Upstream also asks that the colon be before `end`, comparing a
        // pointer into a curly-brace name's expansion with one into the
        // command line: two unrelated objects. Every colon in the name is
        // inside it, so the extra test adds nothing but the coin flip
        // (O-B14-12).
        let name = source.get(at..at + name_len).unwrap_or_default();
        if name.contains(&b':') {
            let start = msg_bytes(&text[start..]);
            semsg!("E884: Function name cannot contain a colon: {start}");
            return None;
        }
    }

    let mut name = Vec::with_capacity(lead + sid.len() + len + 1);
    if !skip && lead > 0 {
        name.extend_from_slice(&SNR);
        // It's "<SID>", so the script id goes in as well.
        name.extend_from_slice(&sid);
    }
    name.extend_from_slice(source.get(at..at + len).unwrap_or_default());
    Some(XString::from_bytes(&name))
}

/// The function a name that selects a value names -- `dict.func`,
/// `list[i]`, `v:lua.name` -- with what it records of the selection in
/// `answer`. `end` is where the name `lv` was resolved from ends in `text`.
fn trans_selected(
    lv: &mut LValue<'_>,
    text: &[u8],
    end: usize,
    skip: bool,
    flags: c_int,
    want_dict: bool,
    answer: &mut FunctionName,
) {
    let new_key = matches!(lv.target, Target::NewKey { .. });
    let selected_dict = match &lv.target {
        Target::Slot {
            slot: LvalSlot::Key { dict, key },
            ..
        }
        | Target::NewKey { dict, key } => Some(FuncDict {
            dict: dict.clone(),
            key: key.clone(),
            new_key,
        }),
        _ => None,
    };
    let has_dict = selected_dict.is_some();
    if want_dict {
        answer.dict = selected_dict;
    }

    /// What the selected value is, as far as a function name goes.
    enum Selected {
        Func(XString),
        Partial(PartialRef, XString),
        Other,
    }
    let selected = lv
        .with_slot(|tv, _| {
            let tv = &*tv;
            // The function's own name; a missing one is the empty string.
            let callee =
                || XString::from_bytes(tv.callable_name().map_or(&b""[..], CStr::to_bytes));
            match tv {
                TypVal::Func(_) if !tv.func_name_or_null().is_null() => Selected::Func(callee()),
                TypVal::Partial(partial) => match &**partial {
                    Some(partial) => Selected::Partial(partial.clone(), callee()),
                    None => Selected::Other,
                },
                _ => Selected::Other,
            }
        })
        .unwrap_or(Selected::Other);
    match selected {
        Selected::Func(func) => {
            answer.name = Some(func);
            answer.end = end;
        }
        Selected::Partial(partial, name) => {
            if is_luafunc(partial.as_ptr()) && byte(text, end) == b'.' {
                let len = check_luafunc_name(&text[end + 1..], true);
                if len == 0 {
                    let arg0 = "v:lua";
                    semsg!("E15: Invalid expression: \"{arg0}\"");
                    return;
                }
                answer.name = Some(XString::from_bytes(&text[end + 1..end + 1 + len]));
                answer.end = end + 1 + len;
            } else {
                answer.name = Some(name);
                answer.end = end;
            }
            answer.partial = Some(partial);
        }
        Selected::Other => {
            if !skip && flags & TFN_QUIET == 0 && !(want_dict && has_dict && new_key) {
                emsg(gettext(E_FUNCREF));
            } else {
                answer.end = end;
            }
        }
    }
}

/// Read a function name at the start of `text` -- a command line from the
/// name to its end -- and answer the name it is stored under and how much
/// of the text it took.
///
/// `want_dict` asks for the dictionary entry a `dict.func` selects, which
/// is also what lets a key the dictionary does not have yet pass quietly.
pub(crate) fn trans_function_name(
    text: &[u8],
    skip: bool,
    flags: c_int,
    want_dict: bool,
) -> FunctionName {
    let mut answer = FunctionName::default();

    // A hard-coded <SNR> is an already translated function id, from a
    // user command.
    if text.starts_with(&SNR) {
        let id = id_len(&text[SNR.len()..]);
        let past = SNR.len() + id;
        answer.end = if id > 0 {
            past + skip::white(&text[past..])
        } else {
            SNR.len()
        };
        answer.name = Some(XString::from_bytes(&text[..past]));
        return answer;
    }

    // A name starting with "<SID>" or "<SNR>" is local to a script. But
    // don't skip over "s:", `get_lval` needs it for "s:dict.func".
    let lead = fname_script_len(text);
    let start = if lead > 2 { lead } else { 0 };

    // The TFN_ flags use the same values as the GLV_ ones.
    let glv = flags | GLV_READ_ONLY;
    let fne = if lead > 2 { 0 } else { FNE_CHECK_START };
    let (mut lv, end_at) = get_lval(&text[start..], None, false, skip, glv, fne);
    // Upstream's `ll_tv != NULL`: the name selected a value rather than
    // naming a whole variable.
    let selects = matches!(lv.target, Target::Slot { .. } | Target::NewKey { .. });
    let range = matches!(lv.target, Target::Slot { span, .. } if span.range);

    if end_at == Some(0) {
        if !skip {
            emsg(gettext(c"E129: Function name required"));
        }
        return answer;
    }
    let end = match end_at {
        Some(end) if !(selects && (lead > 2 || range)) => start + end,
        _ => {
            // Report an invalid expression in braces, unless the
            // evaluation was cancelled by an aborting error, an interrupt
            // or an exception.
            if !aborting() {
                if end_at.is_some() {
                    let start = msg_bytes(&text[start..]);
                    semsg!("E475: Invalid argument: {start}");
                }
            } else {
                answer.end = start + name_end(&text[start..], FNE_INCL_BR).end;
            }
            return answer;
        }
    };

    if selects {
        trans_selected(&mut lv, text, end, skip, flags, want_dict, &mut answer);
        return answer;
    }

    if !lv.has_name() {
        // Error found, but carry on after the function name.
        answer.end = end;
        return answer;
    }

    // Check whether the name is a funcref; if so, use its value. A
    // curly-brace name is always looked up; a plain one unless the caller
    // asked for the name as written.
    let no_autoload = flags & TFN_NO_AUTOLOAD != 0;
    let expanded = lv.expanded().map(|expanded| &expanded[..]);
    let dereffed = match expanded {
        Some(expanded) => Some(deref_func_name(expanded, no_autoload)),
        None if flags & TFN_NO_DEREF == 0 => Some(deref_func_name(&text[..end], no_autoload)),
        None => None,
    };
    if let Some(dereffed) = dereffed {
        answer.partial = dereffed.partial;
        if let Some(name) = dereffed.name {
            answer.end = end;
            // Change "<SNR>" to the byte sequence.
            answer.name = Some(match name.strip_prefix(b"<SNR>") {
                Some(rest) => {
                    let mut name = SNR.to_vec();
                    name.extend_from_slice(rest);
                    XString::from_bytes(&name)
                }
                None => name,
            });
            return answer;
        }
    }

    let name_len = lv.name().len();
    answer.name = mangle_function_name(text, expanded, name_len, start, end, lead, skip, flags);
    if answer.name.is_some() {
        answer.end = end;
    }
    answer
}

/// Expand `s:`/`<SID>` at the front of `name` into `<SNR>N_`. `None` when
/// there is no such prefix, or no script to take the id from (reported).
pub(crate) fn scriptlocal_funcname(name: &[u8]) -> Option<XString> {
    let off = if name.starts_with(b"s:") {
        2
    } else if name.starts_with(b"<SID>") {
        5
    } else {
        // The function name does not have a script-local prefix.
        return None;
    };
    let sid = current_sctx.get().sc_sid;
    if !script_id_valid(sid) {
        emsg(gettext(e_usingsid));
        return None;
    }
    let mut local = format!("<SNR>{sid}_").into_bytes();
    local.extend_from_slice(&name[off..]);
    Some(XString::from_bytes(&local))
}

/// How many bytes `strtoimax` consumes at the start of `text`: blanks, a
/// sign and the digits after it, or nothing at all when there are none.
fn strtoimax_len(text: &[u8]) -> usize {
    let blanks = text
        .iter()
        .take_while(|&&b| matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r'))
        .count();
    let sign = usize::from(matches!(byte(text, blanks), b'+' | b'-'));
    let digits = skip::digits(&text[(blanks + sign).min(text.len())..]);
    if digits == 0 {
        0
    } else {
        blanks + sign + digits
    }
}

/// [`trans_function_name`], except that a `<lambda>N` is taken as-is.
pub(crate) fn save_function_name(
    text: &[u8],
    skip: bool,
    flags: c_int,
    want_dict: bool,
) -> FunctionName {
    if let Some(number) = text.strip_prefix(b"<lambda>") {
        let end = b"<lambda>".len() + strtoimax_len(number);
        return FunctionName {
            name: Some(XString::from_bytes(&text[..end])),
            end,
            ..FunctionName::default()
        };
    }
    trans_function_name(text, skip, flags, want_dict)
}

/// How long the script-local prefix `text` starts with is: 5 for
/// `<SID>`/`<SNR>`, 2 for `s:`, 0 for neither.
pub(crate) fn fname_script_len(text: &[u8]) -> usize {
    // Writing `s:` instead of `<SID>` is allowed, and `<SNR>` is what a
    // name that has already been translated looks like. The comparison
    // folds case the way the rest of the name lookup does.
    if let [b'<', rest @ ..] = text {
        let rest = &rest[..rest.len().min(4)];
        if strnicmp_in(rest, b"SID>") == 0 || strnicmp_in(rest, b"SNR>") == 0 {
            return 5;
        }
    }
    if text.starts_with(b"s:") {
        return 2;
    }
    0
}
