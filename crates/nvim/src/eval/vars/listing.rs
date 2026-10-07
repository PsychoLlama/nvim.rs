//! `:let` with no value: printing variables rather than setting them.
//!
//! [`list_arg_vars`] resolves each argument (including a bare scope name)
//! and [`list_one_var_a`] does the printing, padding the name to column 22
//! and prefixing the value with `#`, `*`, `[` or `{` by type.  That layout
//! is a contract: it is what a user sees.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::semsg;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_int};

use super::*;
use crate::eval::typval::DictRef;
use crate::types::IOSIZE;

/// The variables in `dict`, one per line, each name prefixed with `prefix`.
///
/// `empty` includes the variables holding the null string, which only the
/// scopes that can hold one want.  `:filter` is applied to the prefixed
/// name.
///
/// Each variable's name and value are copied out of the dictionary before
/// anything is printed: a message can `:redir` into a variable, which is a
/// write to a dictionary -- possibly this one.
pub(crate) fn list_dict_vars(dict: &DictRef, prefix: &CStr, empty: bool, first: &mut bool) {
    let mut cursor = DictCursor::new(dict);
    // Upstream re-reads `got_int` in the loop condition, so a `:let`
    // listing stops at the interrupt rather than at the end.
    while !got_int.get()
        && let Some(slot) = cursor.next(dict)
    {
        let Some((key, value)) = dict
            .item_at(slot)
            .map(|item| (item.key().to_vec(), item.di_tv.clone()))
        else {
            continue;
        };
        // What `:filter` matches: the prefixed name, cut where upstream's
        // `IOSIZE` buffer cut it.
        let mut name = prefix.to_bytes().to_vec();
        name.extend_from_slice(&key);
        name.truncate(IOSIZE as usize - 1);
        if cstr::with_terminated(&name, message_filtered) {
            continue;
        }
        if empty || value.v_type() != VAR_STRING || value.string_ref().is_some() {
            let text = encode_tv2echo(&value);
            list_one_var_a(prefix, &key, value.v_type(), text.as_bytes(), first);
        }
    }
}

/// The `g:` scope.
pub(crate) fn list_glob_vars(first: &mut bool) {
    list_dict_vars(&globvar_dict(), c"", true, first);
}

/// The current buffer's `b:` scope.
pub(crate) fn list_buf_vars(first: &mut bool) {
    if let Some(dict) = Buf::current().b_bufvar.di_tv.dict_handle() {
        list_dict_vars(&dict, c"b:", true, first);
    }
}

/// The current window's `w:` scope.
pub(crate) fn list_win_vars(first: &mut bool) {
    if let Some(dict) = Win::current().w_winvar.di_tv.dict_handle() {
        list_dict_vars(&dict, c"w:", true, first);
    }
}

/// The current tab page's `t:` scope.
pub(crate) fn list_tab_vars(first: &mut bool) {
    if let Some(dict) = TabPage::current().tp_winvar.di_tv.dict_handle() {
        list_dict_vars(&dict, c"t:", true, first);
    }
}

/// The `v:` scope.  `empty` is false: the `v:` variables that hold no string
/// are not listed.
pub(crate) fn list_vim_vars(first: &mut bool) {
    list_dict_vars(&vimvar_dict(), c"v:", false, first);
}

/// The current script's `s:` scope, if there is one.
pub(crate) fn list_script_vars(first: &mut bool) {
    let sid = current_sctx.get().sc_sid;
    if let Some(dict) = script_scope_dict(sid) {
        list_dict_vars(&dict, c"s:", false, first);
    }
}

/// `:let name …`: print each named variable in `text`, or the whole of a
/// scope named on its own. `skip` only checks that the names parse. Answers
/// where in `text` it stopped.
pub(crate) fn list_arg_vars(text: &[u8], skip: bool, first: &mut bool) -> usize {
    let mut error = false;
    let mut arg = 0;
    let rest = |at: usize| text.get(at..).unwrap_or_default();
    while ends_excmd(c_int::from(cstr::byte_at(text, arg))) == 0 && !got_int.get() {
        if error || skip {
            // Nothing is being printed any more; just check that what is
            // left parses as names.
            let flags = FNE_INCL_BR | FNE_CHECK_START;
            arg += name_end(rest(arg), flags).end;
            let c = c_int::from(cstr::byte_at(text, arg));
            if !ascii_iswhite(c) && ends_excmd(c) == 0 {
                emsg_severe.set(true);
                let shown = crate::message_fmt::msg_bytes(rest(arg));
                semsg!("E488: Trailing characters: {shown}");
                break;
            }
            arg += skip::white(rest(arg));
            continue;
        }

        let name_start = arg;
        // A `{curly}` name is expanded into `tofree`.
        let mut cursor = Cursor::new(rest(arg));
        let (len, tofree) = get_name_len(&mut cursor, true, true);
        arg += cursor.offset();
        'done: {
            if len <= 0 {
                if len < 0 && !aborting() {
                    emsg_severe.set(true);
                    let shown = crate::message_fmt::msg_bytes(rest(arg));
                    semsg!("E475: Invalid argument: {shown}");
                    return arg;
                }
                error = true;
                break 'done;
            }
            let len = usize::try_from(len).expect("a positive length");
            let name: &[u8] = match &tofree {
                Some(expanded) => {
                    let expanded: &[u8] = expanded;
                    expanded.get(..len).unwrap_or(expanded)
                }
                None => &text[name_start..name_start + len],
            };

            let mut tv = TV_INITIAL_VALUE;
            if eval_variable(name, Some(&mut tv), true, false).is_err() {
                error = true;
                break 'done;
            }
            // The subscript is read from the same text as the name.
            let arg_subsc = arg;
            let subscripted = handle_subscript(&mut cursor, &mut tv, true, true);
            arg = name_start + cursor.offset();
            if subscripted.is_err() {
                error = true;
                break 'done;
            }

            if arg == arg_subsc && len == 2 && name[1] == b':' {
                // A bare scope name lists the whole scope.
                let lister: Option<ScopeLister> = match name[0] {
                    b'g' => Some(list_glob_vars),
                    b'b' => Some(list_buf_vars),
                    b'w' => Some(list_win_vars),
                    b't' => Some(list_tab_vars),
                    b'v' => Some(list_vim_vars),
                    b's' => Some(list_script_vars),
                    b'l' => Some(list_func_vars),
                    _ => None,
                };
                match lister {
                    Some(lister) => lister(first),
                    None => {
                        let name = crate::message_fmt::msg_bytes(name);
                        semsg!("E738: Can't list variables for {name}");
                    }
                }
            } else {
                let rendered = encode_tv2echo(&tv);
                // Without a subscript the expanded name is what was
                // looked up; with one, the command line's own text is
                // what should be shown.
                let shown: &[u8] = match &tofree {
                    Some(expanded) if arg == arg_subsc => expanded,
                    _ => &text[name_start..arg],
                };
                list_one_var_a(c"", shown, tv.v_type(), rendered.as_bytes(), first);
            }
            clear_local(&mut tv);
        }
        drop(tofree);
        arg += skip::white(rest(arg));
    }
    arg
}

/// Print one `name  <sigil><value>` line.
///
/// The name is padded to column 22 and the sigil says what the type is:
/// `#` a Number, `*` a Funcref, `[` a List, `{` a Dict, a space anything
/// else.  For a List or a Dict the sigil replaces the bracket the rendered
/// value already starts with.
///
/// `first` clears the rest of the screen on the first line and is set false.
fn list_one_var_a(prefix: &CStr, name: &[u8], value_type: VarType, text: &[u8], first: &mut bool) {
    if *first {
        msg_ext_set_kind(c"list_cmd");
        msg_start();
    } else {
        msg_putchar(c_int::from(b'\n'));
    }
    // Not `msg()`, which would overwrite "v:statusmsg".
    if !prefix.is_empty() {
        msg_str(prefix);
    }
    msg_bytes(name, 0, false);
    msg_putchar(c_int::from(b' '));
    msg_advance(22);

    // The sigil, and the bracket it stands in for.
    let sigil: u8 = match value_type {
        VAR_NUMBER => b'#',
        VAR_FUNC | VAR_PARTIAL => b'*',
        VAR_LIST => b'[',
        VAR_DICT => b'{',
        _ => b' ',
    };
    msg_putchar(c_int::from(sigil));
    let text = match text.split_first() {
        Some((&lead, rest))
            if (value_type == VAR_LIST || value_type == VAR_DICT) && lead == sigil =>
        {
            rest
        }
        _ => text,
    };

    msg_display_bytes(text, 0, false);

    if value_type == VAR_FUNC || value_type == VAR_PARTIAL {
        msg_str(c"()");
    }
    if *first {
        msg_clr_eos();
        *first = false;
    }
}
