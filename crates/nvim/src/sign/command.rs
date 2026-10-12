//! The `:sign` Ex command.
//!
//! [`ex_sign`] picks a subcommand out of [`CMDS`], [`parse_sign_cmd_args`]
//! turns the rest of the line into the `line=`/`name=`/`group=`/
//! `priority=`/`file=`/`buffer=` tuple the three placement subcommands
//! share, and the `sign_*_cmd` functions diagnose the combinations that do
//! not make sense before handing off to the placement primitives in the
//! parent. [`sign_list_placed`] and [`sign_list_defined`] are the
//! `:sign place` / `:sign list` reports.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::find_buffer_by_name;
use crate::charset::getdigits_int_at;
use crate::charset::skip;
use crate::debugger::atoi;
use crate::message::{msg_bytes, msg_display_bytes};
use crate::message_fmt::msg_bytes as shown;
use crate::semsg;
use crate::smsg;
use crate::types::FAIL;
use crate::winlayer::Win;

/// The translated `$fmt` formatted with the arguments, cut to what a
/// [`MSG_BUF_LEN`] buffer holds.
///
/// Every message in this module is bounded that way upstream; `MSG_BUF_LEN`
/// is 480, and the format arguments (a file name, a group name, a sign name)
/// can all exceed it, so the truncation is load-bearing rather than
/// defensive.
macro_rules! msg_buf {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {
        $crate::formatted!(MSG_BUF_LEN as usize, gettext($fmt).to_bytes() $(, $arg)*)
    };
}

/// `text` past its leading spaces and tabs.
fn skip_white(text: &[u8]) -> &[u8] {
    &text[skip::white(text)..]
}

/// `text` up to its first space or tab, and what follows from there.
fn split_white(text: &[u8]) -> (&[u8], &[u8]) {
    text.split_at(skip::to_white(text))
}

/// A sign name with its leading zeroes stripped, so "099" and "99" name the
/// same sign -- but a bare "0" is kept.
fn strip_zeroes(mut name: &[u8]) -> &[u8] {
    while name.len() > 1 && name[0] == b'0' {
        name = &name[1..];
    }
    name
}

/// The `:sign place` report for `rbuf`, or for every buffer when it is `None`.
pub(crate) fn sign_list_placed(rbuf: Option<Buf>, group: Option<&[u8]>) {
    let ns = group_get_ns(group);
    msg_title(gettext(c"\n--- Signs ---"));

    let mut cur = match rbuf {
        Some(buf) => Some(buf),
        None => first_buffer(),
    };
    while let Some(cbuf) = cur {
        if got_int.get() {
            break;
        }
        if buf_has_signs(cbuf) {
            msg_putchar('\n' as c_int);
            let lbuf = msg_buf!(c"Signs for %s:", cbuf.name.shown());
            msg_bytes(&lbuf, HLF_D, false);
        }

        // A group that names no namespace matches nothing, but still prints
        // the per-buffer header above.
        if ns >= 0 {
            let mut signs = placed_signs(cbuf, 0, ns, |_| Keep::Yes);
            if !signs.is_empty() {
                sort_signs(&mut signs);
                report_signs(&signs);
            }
        }

        if rbuf.is_some() {
            return;
        }
        cur = cbuf.next();
    }
}

/// The `line=`/`id=`/`group=`/`name=`/`priority=` lines `:sign place`
/// prints for one buffer's signs, already sorted.
fn report_signs(signs: &[MTKey]) {
    msg_putchar('\n' as c_int);
    for (i, mark) in signs.iter().enumerate() {
        let sh = sign_of(*mark);
        let namebuf = if sh.sign_name_cstr().is_none() {
            Vec::new()
        } else {
            msg_buf!(c"  name=%s", placed_sign_name(&sh))
        };
        let groupbuf = if mark.ns == 0 {
            Vec::new()
        } else {
            msg_buf!(c"  group=%s", &describe_ns(mark.ns.cast_signed(), b"")[..])
        };
        let lbuf = msg_buf!(
            c"    line=%d  id=%u%s%s  priority=%d",
            mark.pos.row + 1,
            mark.id,
            &groupbuf[..],
            &namebuf[..],
            c_int::from(sh.priority),
        );
        msg_bytes(&lbuf, 0, false);
        if i + 1 < signs.len() {
            msg_putchar('\n' as c_int);
        }
    }
}

/// The index of the `:sign` subcommand `word` names, or [`SIGNCMD_LAST`]
/// for one that is not a subcommand.
pub(crate) fn sign_cmd_idx(word: &[u8]) -> c_int {
    CMDS.iter()
        .position(|cmd| cmd.to_bytes() == word)
        .map_or(SIGNCMD_LAST, |i| {
            c_int::try_from(i).expect("six subcommands")
        })
}

/// The `:sign list` report for one definition.
pub(crate) fn sign_list_defined(sign: &Sign) {
    smsg!(0, "sign {}", shown(sign.name.as_bytes()));
    if let Some(icon) = &sign.icon {
        msg_str(c" icon=");
        msg_display_bytes(icon, 0, false);
        msg_str(gettext(c" (not supported)"));
    }
    if sign.text[0] != 0 {
        msg_str(c" text=");
        msg_display_bytes(&sign_text_bytes(&sign.text), 0, false);
    }
    if sign.priority > 0 {
        let lbuf = msg_buf!(c" priority=%d", sign.priority);
        msg_bytes(&lbuf, 0, false);
    }
    let labels = [c" linehl=", c" texthl=", c" culhl=", c" numhl="];
    let ids = [sign.line_hl, sign.text_hl, sign.cul_hl, sign.num_hl];
    for (label, id) in labels.into_iter().zip(ids) {
        if id > 0 {
            msg_str(label);
            msg_str(get_highlight_name_ext(id - 1, false).unwrap_or(c"NONE"));
        }
    }
}

/// `:sign list {name}`.
fn sign_list_by_name(name: &[u8]) {
    match sign_find(name) {
        Some(sign) => sign_list_defined(&sign),
        None => {
            semsg!("E155: Unknown sign: {}", shown(name));
        }
    }
}

/// `:sign define {name} {args}...`.
///
/// The arguments are `key=value` pairs in any order, each running to the
/// next white space a backslash does not escape. An unrecognised key is
/// E475 and defines nothing.
fn sign_define_cmd(name: &[u8], args: &[u8]) {
    let mut attrs = SignAttrs {
        prio: -1,
        ..Default::default()
    };
    let mut rest = args;
    loop {
        let arg = skip_white(rest);
        if arg.is_empty() {
            break;
        }
        let end = skip::to_white_esc(arg);
        let value = |key: &[u8]| arg[..end].strip_prefix(key).map(<[u8]>::to_vec);
        if let Some(v) = value(b"icon=") {
            attrs.icon = Some(v);
        } else if let Some(v) = value(b"text=") {
            attrs.text = Some(v);
        } else if let Some(v) = value(b"linehl=") {
            attrs.linehl = Some(v);
        } else if let Some(v) = value(b"texthl=") {
            attrs.texthl = Some(v);
        } else if let Some(v) = value(b"culhl=") {
            attrs.culhl = Some(v);
        } else if let Some(v) = value(b"numhl=") {
            attrs.numhl = Some(v);
        } else if arg.starts_with(b"priority=") {
            // `atoi` reads on past the value's end, as upstream's does: the
            // value is only cut off after it is read.
            attrs.prio = atoi(&arg[b"priority=".len()..]);
        } else {
            // Upstream quotes the argument before cutting it off, so the
            // message runs to the end of the line.
            semsg!("E475: Invalid argument: {}", shown(arg));
            return;
        }
        // The next argument starts after the one white byte that ended this
        // one; skipping white space at the top of the loop covers it.
        rest = &arg[end..];
    }

    let _ = sign_define_by_name(name, attrs);
}

/// `:sign place`, which both places a sign and — with no id — lists them.
fn sign_place_cmd(args: &SignCmdArgs) {
    let empty_group = args.group.as_deref().is_some_and(<[u8]>::is_empty);
    if args.id <= 0 {
        // The listing forms: `:sign place [group=X] [file=Y|buffer=N]`.
        // A `line=` or a `name=` means a placement was intended.
        if args.lnum >= 0 || args.name.is_some() || empty_group {
            emsg(gettext(e_invarg));
        } else {
            sign_list_placed(args.buf, args.group.as_deref());
        }
        return;
    }
    let (Some(name), Some(buffer), false) = (&args.name, args.buf, empty_group) else {
        emsg(gettext(e_invarg));
        return;
    };
    let mut uid = args.id.cast_unsigned();
    let _ = sign_place(
        &mut uid,
        args.group.as_deref(),
        name,
        buffer,
        args.lnum,
        args.prio,
    );
}

/// `:sign unplace`.
///
/// With no id at all it removes the highest-priority sign on the *cursor*
/// line of the current buffer — the only spelling that reaches
/// `buf_delete_signs`' single-line form, since an explicit `line=` is E474.
fn sign_unplace_cmd(args: &SignCmdArgs) {
    let empty_group = args.group.as_deref().is_some_and(<[u8]>::is_empty);
    if args.lnum >= 0 || args.name.is_some() || empty_group {
        emsg(gettext(e_invarg));
        return;
    }

    let (buf, lnum) = if args.id == -1 {
        (Some(Win::current().buffer()), Win::current().w_cursor.lnum)
    } else {
        (args.buf, args.lnum)
    };

    if sign_unplace(buf, args.id.max(0), args.group.as_deref(), lnum) == FAIL && lnum > 0 {
        emsg(gettext(c"E159: Missing sign number"));
    }
}

/// `:sign jump {id} [group={group}] file={fname}|buffer={nr}`.
fn sign_jump_cmd(args: &SignCmdArgs) {
    if args.name.is_none() && args.group.is_none() && args.id == -1 {
        emsg(gettext(e_argreq));
        return;
    }
    // No buffer, an empty group, or a `line=`/`name=` that jumping has
    // no use for.
    let empty_group = args.group.as_deref().is_some_and(<[u8]>::is_empty);
    let bad = empty_group || args.lnum >= 0 || args.name.is_some();
    let (Some(buffer), false) = (args.buf, bad) else {
        emsg(gettext(e_invarg));
        return;
    };
    sign_jump(args.id, args.group.as_deref(), buffer);
}

/// What [`parse_sign_cmd_args`] read off a `:sign place`/`unplace`/`jump`
/// line.
///
/// The "absent" values are what the three `sign_*_cmd` functions test
/// against, and they are not all zero: `id` and `lnum` are −1 so that an
/// explicit `0` can be told from no argument at all.
struct SignCmdArgs {
    /// Leading zeroes already stripped.
    name: Option<Vec<u8>>,
    id: c_int,
    /// `Some` of an empty group for a `group=` with no value.
    group: Option<Vec<u8>>,
    prio: c_int,
    /// Found while parsing and used by the command straight after.
    buf: Option<Buf>,
    lnum: LineNr,
}

impl Default for SignCmdArgs {
    fn default() -> Self {
        Self {
            name: None,
            id: -1,
            group: None,
            prio: -1,
            buf: None,
            lnum: -1,
        }
    }
}

/// Parses the arguments `:sign place`, `:sign unplace` and `:sign jump`
/// share: an optional leading id, then `line=`, `name=`, `group=`,
/// `priority=` and one of `file=`/`buffer=`, which ends the line.
///
/// Answers `None` after diagnosing; `Some` otherwise, including when
/// nothing was given.
fn parse_sign_cmd_args(cmd: c_int, line: &[u8]) -> Option<SignCmdArgs> {
    let mut out = SignCmdArgs::default();
    let mut arg = line;
    // Whether a `file=` or `buffer=` was given.
    let mut filename: Option<&[u8]> = None;
    let mut lnum_arg = false;

    // A leading number is the sign id — but only if a separator follows,
    // so that `:sign unplace 3name=x` is not read as id 3.
    if arg.first().is_some_and(u8::is_ascii_digit) {
        let (id, past) = getdigits_int_at(arg, 0, true, 0);
        if arg
            .get(past)
            .is_some_and(|&b| !ascii_iswhite(c_int::from(b)))
        {
            out.id = -1;
        } else {
            out.id = id;
            arg = skip_white(&arg[past..]);
        }
    }

    while !arg.is_empty() {
        if let Some(v) = arg.strip_prefix(b"line=") {
            out.lnum = atoi(v);
            arg = split_white(v).1;
            lnum_arg = true;
        } else if cmd == SIGNCMD_UNPLACE && arg[0] == b'*' {
            // `:sign unplace *`: every sign, and not with an id too.
            if out.id != -1 {
                emsg(gettext(e_invarg));
                return None;
            }
            out.id = -2;
            arg = split_white(&arg[1..]).1;
        } else if let Some(v) = arg.strip_prefix(b"name=") {
            let (name, after) = split_white(v);
            out.name = Some(strip_zeroes(name).to_vec());
            arg = after;
        } else if let Some(v) = arg.strip_prefix(b"group=") {
            let (group, after) = split_white(v);
            out.group = Some(group.to_vec());
            arg = after;
        } else if let Some(v) = arg.strip_prefix(b"priority=") {
            out.prio = atoi(v);
            arg = split_white(v).1;
        } else if let Some(v) = arg.strip_prefix(b"file=") {
            // The rest of the line, white space and all, is the name.
            filename = Some(v);
            out.buf = CString::new(v)
                .ok()
                .and_then(|name| find_buffer_by_name(&name));
            break;
        } else if let Some(v) = arg.strip_prefix(b"buffer=") {
            filename = Some(v);
            let (nr, past) = getdigits_int_at(v, 0, true, 0);
            out.buf = find_buf(nr);
            // Diagnosed but not fatal, which is why this still breaks
            // out with whatever buffer it found.
            if !skip_white(&v[past..]).is_empty() {
                semsg!("E488: Trailing characters: {}", shown(&v[past..]));
            }
            break;
        } else {
            emsg(gettext(e_invarg));
            return None;
        }
        arg = skip_white(arg);
    }

    if let Some(filename) = filename
        && out.buf.is_none()
    {
        semsg!("E158: Invalid buffer name: {}", shown(filename));
        return None;
    }

    // `:sign place line=N` and `:sign jump` default to the current
    // buffer; `:sign unplace` deliberately does not.
    if filename.is_none() && ((cmd == SIGNCMD_PLACE && lnum_arg) || cmd == SIGNCMD_JUMP) {
        out.buf = Win::current().buffer_or_none();
    }
    Some(out)
}

/// `:sign`.
pub(crate) fn ex_sign(excmd: &mut ExArg) {
    // A copy: `:sign jump` runs autocommands.
    let line = excmd.line.arg().to_vec();
    let (word, after) = split_white(&line);
    let idx = sign_cmd_idx(word);
    if idx == SIGNCMD_LAST {
        semsg!("E160: Unknown sign command: {}", shown(&line));
        return;
    }
    let arg = skip_white(after);

    if idx > SIGNCMD_LIST {
        // Place, unplace or jump: a shared argument parser first.
        let Some(args) = parse_sign_cmd_args(idx, arg) else {
            return;
        };
        match idx {
            SIGNCMD_PLACE => sign_place_cmd(&args),
            SIGNCMD_UNPLACE => sign_unplace_cmd(&args),
            SIGNCMD_JUMP => sign_jump_cmd(&args),
            _ => {}
        }
        return;
    }

    // Define, undefine or list.
    if idx == SIGNCMD_LIST && arg.is_empty() {
        for sign in sign_defs() {
            sign_list_defined(&sign);
        }
        return;
    }
    if arg.is_empty() {
        emsg(gettext(c"E156: Missing sign name"));
        return;
    }

    // Isolate the sign name; the arguments start after the white byte that
    // ends it.
    let (name, rest) = split_white(arg);
    let name = strip_zeroes(name);
    match idx {
        SIGNCMD_DEFINE => sign_define_cmd(name, rest.get(1..).unwrap_or_default()),
        SIGNCMD_LIST => sign_list_by_name(name),
        _ => {
            let _ = sign_undefine_by_name(name);
        }
    }
}
