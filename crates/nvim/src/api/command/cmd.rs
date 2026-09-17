//! `nvim_cmd()`: executing a command given as a Dict.
//!
//! The inverse of [`super::parse`]: every field is validated against the
//! command's `argt` flags (which arguments it accepts, whether it takes a
//! range, a count, a register or a bang), the `mods` sub-keyset is unpacked
//! into a `CmdMod`, and the result is handed to `execute_cmd` -- with
//! the output captured when `opts.output` is set.
//!
//! The Dict is consumed in stages, in the order the command line itself
//! would be: resolve the name, collect the arguments, apply the address
//! (range, count, register, bang), unpack `magic` and `mods`, render the
//! whole thing back into a command line for `++opt` parsing, and only then
//! execute. Each stage answers "may we keep going?" -- `false`/`None` means
//! either `err` is set or the Dict asked for nothing executable, both of
//! which end the call without running anything.
//!
//! # What the references here promise
//!
//! Two contracts hold for every function below, and stating them once is what
//! lets the stages be ordinary safe code rather than a chain of `unsafe fn`s:
//!
//! - **A `&KeyDict_cmd` (or a sub-keyset reference) is the dispatcher's own
//!   decoded Dict.** Every `String` field in it is NUL-terminated and every
//!   `Array` is valid for its `size`, because `api_dict_to_keydict` is the
//!   only thing that ever fills one. Reading a field is therefore safe; only
//!   *dereferencing* the `data` pointer needs a block, and the note there
//!   names which key the byte belongs to.
//! - **This runs on the main thread inside the API dispatcher**, so the
//!   editor globals the `ex_docmd` entry points consult (`curbuf`, the
//!   command table, the register table) are live for the whole call.
//!
//! What is left `unsafe fn` is what those two do not cover: the raw `arena`,
//! and `excmd.arg` pointing into a command line only the caller can vouch for.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::validate::{err_bad_value, err_expected, err_required};
use crate::api_error;
use crate::cstr;
use crate::ex_docmd::is_user_cmd;
use crate::guard::Suppress;
use crate::message_fmt::{c_str, msg_bytes, msg_cstr};
use crate::types::CmdIdx;
use crate::types::{CmdLine, ExArgt, FieldHashfn, NUL};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

const EMPTY_ARRAY: Array = Array::EMPTY;

// Two locals over `api::private::validate`'s family: the ones whose argument
// is still a pointer into the caller's own text rather than a literal.

/// [`err_bad_value`] where the offending value is a pointer.
fn err_invalid_at(name: &CStr, value: *const c_char) -> Error {
    // SAFETY: `value` is null or a NUL-terminated string of the caller's.
    let value = unsafe { cstr::at_opt(value) };
    err_bad_value(name, value.unwrap_or(c""))
}

/// [`err_expected`] where what arrived is a pointer.
fn err_expected_at(name: &CStr, expected: &CStr, actual: *const c_char) -> Error {
    // SAFETY: `actual` is null or a NUL-terminated string of the caller's.
    let actual = unsafe { cstr::at_opt(actual) };
    err_expected(name, expected, actual)
}

/// Decode one of `cmd`'s sub-keyset Dicts (`magic`, `mods`, `mods.filter`)
/// into a fresh `K`.
///
/// `get_field` must be `K`'s own generated field lookup: the decoder writes
/// through the offsets it hands back, so pairing it with a different keyset
/// would write outside `K`.
fn sub_keyset<K: Default>(dict: &ApiDict, get_field: FieldHashfn) -> Result<K, Error> {
    // Every key unset, which is what the decoder expects to start from.
    let mut out = K::default();
    // The keyset takes its fields over, and the caller's dictionary is a
    // borrow of the outer keyset's -- so the copy is the sub-keyset's.
    // SAFETY: as above.
    unsafe { api_dict_to_keydict((&raw mut out).cast(), get_field, dict.clone()) }?;
    Ok(out)
}

/// # Safety
///
/// `cmd` must point at the `KeyDict_cmd` the dispatcher filled in, live for
/// the call. `opts` must point at the `KeyDict_cmd_opts` the dispatcher
/// filled in, live for the call. `arena` must point at a live arena, which
/// the memory this answers with is taken from and must outlive.
pub unsafe fn nvim_cmd(
    channel_id: uint64_t,
    cmd: *mut KeyDict_cmd,
    opts: *mut KeyDict_cmd_opts,
) -> Result<String_0, Error> {
    // SAFETY: the dispatcher decodes both keydicts onto its own frame and
    // keeps them alive across the call; neither is reachable from anything
    // this function runs, so a shared borrow of each holds throughout.
    let (cmd, opts) = unsafe { (&*cmd, &*opts) };
    let output = opts.output.unwrap_or(false);

    let mut excmd = ExArg::default();
    // SAFETY: a plain C aggregate whose all-zero state is the valid
    // "nothing parsed yet" one, as the C original's CLEAR_FIELD relies on.
    let mut cmdinfo: CmdParseInfo = unsafe { ::core::mem::zeroed() };

    prepare_cmd(cmd, &mut excmd, &mut cmdinfo).and_then(|prepared| match prepared {
        // SAFETY: `prepare_cmd` answering true means `excmd`/`cmdinfo`
        // describe a resolved, validated command.
        true => unsafe { run_cmd(channel_id, &mut excmd, &mut cmdinfo, output) },
        false => Ok(String_0::NULL),
    })
}

/// Turn the Dict into a resolved, validated `ExArg` plus its rendered
/// command line.
///
/// `Ok(false)` means stop with nothing executed: the Dict carried modifiers
/// and nothing else, which upstream treats as a silent no-op.
///
fn prepare_cmd(
    cmd: &KeyDict_cmd,
    excmd: &mut ExArg,
    cmdinfo: &mut CmdParseInfo,
) -> Result<bool, Error> {
    let Some(range_only) = resolve_command(cmd, excmd)? else {
        return Ok(false);
    };

    let mut args = EMPTY_ARRAY;
    let mut count_from_first_arg = false;
    if let Some(given) = cmd.args.as_ref() {
        count_from_first_arg = collect_args(given, excmd, &mut args)?;
    }

    if !range_only {
        // Only the first argument is ever consulted.
        // `args` was built above, so item 0 is in bounds when it is not
        // empty.
        let first = args.first().map_or(ptr::null_mut(), |arg| {
            arg.as_string()
                .expect("`collect_args` puts only Strings in the array")
                .data()
        });
        unsafe { set_cmd_addr_type(excmd, first) };
    }

    apply_range(cmd, excmd)?;
    apply_count(cmd, excmd, count_from_first_arg)?;
    apply_register(cmd, excmd)?;
    apply_bang(cmd, excmd)?;
    apply_magic(cmd, excmd, cmdinfo)?;
    apply_mods(cmd, excmd, cmdinfo)?;

    // Render the Dict back into a command line: `execute_cmd` and everything
    // under it read the line, not the Array.
    // SAFETY: `excmd` is resolved and `args` holds only Strings.
    unsafe { build_cmdline_str(excmd, cmdinfo, args) };
    apply_argopt(excmd)?;
    if excmd.argt.has(ExArgt::CMDARG) && !excmd.usefilter {
        // SAFETY: as above.
        excmd.do_ecmd_cmd = unsafe { excmd.with_arg_cursor(|cursor| getargcmd(cursor)) };
    }
    Ok(true)
}

/// Look `cmd.cmd` up in the command table, filling `excmd.cmdidx`/`excmd.argt`.
///
/// `Ok(Some(range_only))` on success -- a "range only" command such as `:1`
/// has no name at all. `Ok(None)` means stop with nothing executed, per
/// [`prepare_cmd`].
fn resolve_command(cmd: &KeyDict_cmd, excmd: &mut ExArg) -> Result<Option<bool>, Error> {
    let Some(name) = cmd.cmd.as_ref() else {
        return Err(err_required(c"cmd"));
    };

    // SAFETY: the key is set, so `name` is a NUL-terminated keydict String.
    let named = unsafe { *name.data() } as c_int != NUL;
    let has_range = cmd.range.as_ref().is_some_and(|range| !range.is_empty());
    let has_mods = cmd.mods.is_some();

    if !named && !has_range && !has_mods {
        return Err(err_expected_at(c"cmd", c"non-empty String", ptr::null()));
    }

    // `find_ex_command` reads the command word, so the name is the whole of
    // the line until `build_cmdline_str` renders the real one.
    excmd.line = CmdLine::from_bytes(name.as_bytes());
    let cmdname = excmd.line.ptr_at(0);
    let mut p = unsafe { find_ex_command(excmd, ptr::null_mut()) };

    // An unknown capitalised name plus a CmdUndefined autocommand is a lazily
    // defined user command: fire the event, then look again.
    if !p.is_null()
        && excmd.cmdidx == CmdIdx::SIZE
        && unsafe { *excmd.cmd_ptr() as u8 }.is_ascii_uppercase()
        && has_event(AutoEvent::CmdUndefined)
    {
        // SAFETY: as above.
        unsafe {
            p = name.data();
            let ret = apply_autocmds(AutoEvent::CmdUndefined, p, p, true, None);
            p = if ret as c_int != 0 && !aborting() {
                find_ex_command(excmd, ptr::null_mut())
            } else {
                excmd.cmd_ptr()
            };
        }
    }

    let unnamed_unknown = excmd.cmdidx == CmdIdx::SIZE && !named;
    let range_only = unnamed_unknown && has_range;

    // Modifiers and nothing else: upstream falls straight through to the
    // cleanup, with no error and nothing executed.
    if unnamed_unknown && !has_range && has_mods {
        return Ok(None);
    }

    if !(!p.is_null() && excmd.cmdidx != CmdIdx::SIZE) && !range_only {
        // SAFETY: `cmdname` is the caller's NUL-terminated name.
        let name = unsafe { c_str(cmdname) };
        return Err(api_error!(
            kErrorTypeValidation,
            "Command not found: {name}"
        ));
    }

    // SAFETY: `excmd.cmdidx` came out of `find_ex_command`.
    if !range_only && is_cmd_ni(excmd.cmdidx) {
        // SAFETY: `cmdname` is the caller's NUL-terminated name.
        let name = unsafe { c_str(cmdname) };
        return Err(api_error!(
            kErrorTypeValidation,
            "Command not implemented: {name}"
        ));
    }

    if !range_only {
        // The Dict may abbreviate the name; it still has to be a prefix.
        // SAFETY: both names are NUL-terminated.
        let matched = unsafe {
            let fullname = if is_user_cmd(excmd.cmdidx) {
                get_user_command_name(excmd.useridx, excmd.cmdidx)
            } else {
                get_command_name(ptr::null_mut(), excmd.cmdidx.code())
            };
            cstr::starts_with(fullname, cstr::bytes_at(cmdname))
        };
        if !matched {
            // SAFETY: `cmdname` is the caller's NUL-terminated name.
            let name = unsafe { c_str(cmdname) };
            return Err(api_error!(
                kErrorTypeValidation,
                "Invalid command: \"{name}\""
            ));
        }
    }

    if range_only {
        excmd.argt = ExArgt::RANGE | ExArgt::SBOXOK;
    } else if !is_user_cmd(excmd.cmdidx) {
        // A user command's flags already came out of `find_ex_command`.
        excmd.argt = excmd_get_argt(excmd.cmdidx);
    }

    Ok(Some(range_only))
}

/// Convert `cmd.args` into the `String`-only array the command line is built
/// from, and check the count against `argt`.
///
/// `Ok(true)` means the one argument was consumed as the command's count.
fn collect_args(given: &Array, excmd: &mut ExArg, args: &mut Array) -> Result<bool, Error> {
    // For a command that takes a count but no regular arguments, a lone
    // numeric argument *is* the count.
    if given.len() == 1 && excmd.argt.has(ExArgt::COUNT) && !excmd.argt.has(ExArgt::EXTRA) {
        let count = match &given[0] {
            Object::Integer(n) => Some(*n as int64_t),
            Object::String(str) => {
                let mut endptr: *mut c_char = ptr::null_mut();
                // SAFETY: the argument is NUL-terminated and `endptr` is this
                // frame's own.
                let val = unsafe { strtol(str.data(), &raw mut endptr, 10) };
                // The whole string has to be the number.
                // SAFETY: `strtol` leaves `endptr` within the string it parsed.
                (unsafe { *endptr } as c_int == NUL && !str.is_empty()).then_some(val as int64_t)
            }
            _ => None,
        };
        if let Some(count) = count
            && count >= 0
        {
            excmd.addr_count = 1;
            excmd.line2 = count as LineNr;
            excmd.line1 = excmd.line2;
            *args = Array::with_capacity(0);
            return Ok(true);
        }
    }

    *args = Array::with_capacity(given.len());
    for elem in given {
        match elem {
            // A boolean argument is spelled to the command as "0" or "1".
            Object::Boolean(b) => {
                let digit = if *b { c"1" } else { c"0" };
                args.push(Object::string(String_0::from_cstr(digit)));
            }
            // A handle is its id, like any integer.
            Object::Buffer(n) | Object::Window(n) | Object::Tabpage(n) | Object::Integer(n) => {
                let mut buf = [0 as c_char; NUMBUFLEN as usize];
                // SAFETY: the buffer is `NUMBUFLEN` writable bytes and the
                // verb matches the argument.
                let rendered = unsafe {
                    let (room, fmt) = (NUMBUFLEN as size_t, c"%ld".as_ptr());
                    snprintf(buf.as_mut_ptr(), room, fmt, *n);
                    cstr::bytes_at(buf.as_ptr())
                };
                args.push(Object::string(String_0::from_bytes(rendered)));
            }
            Object::String(s) => {
                // An all-whitespace argument would vanish into the separators.
                if string_iswhite(s) {
                    return Err(err_expected_at(
                        c"command arg",
                        c"non-whitespace",
                        ptr::null(),
                    ));
                }
                args.push(Object::string(String_0::clone(s)));
            }
            _ => {
                let got = api_typename(elem.kind());
                return Err(err_expected(c"command arg", c"valid type", Some(got)));
            }
        }
    }

    let arity = ExArgt::EXTRA | ExArgt::NOSPC | ExArgt::NEEDARG;
    let argc_valid = match excmd.argt.masked(arity) {
        v if v == arity => args.len() == 1,
        v if v == ExArgt::EXTRA | ExArgt::NOSPC => args.len() <= 1,
        v if v == ExArgt::EXTRA | ExArgt::NEEDARG => !args.is_empty(),
        v if v == ExArgt::EXTRA => true,
        _ => args.is_empty(),
    };
    if !argc_valid {
        return Err(Error::validation(c"Wrong number of arguments"));
    }

    Ok(false)
}

/// Apply `cmd.range`, then fall back to the command's default range.
fn apply_range(cmd: &KeyDict_cmd, excmd: &mut ExArg) -> Result<(), Error> {
    if let Some(range) = cmd.range.as_ref() {
        if !excmd.argt.has(ExArgt::RANGE) {
            return Err(err_cannot_accept(c"range", cmd));
        }
        if range.len() > 2 {
            return Err(err_expected_at(c"range", c"<=2 elements", ptr::null()));
        }

        excmd.addr_count = range.len() as c_int;
        for i in 0..range.len() {
            let bound = &range[i];
            if bound.as_integer().is_none_or(|n| n < 0) {
                return Err(err_expected_at(
                    c"range element",
                    c"non-negative Integer",
                    ptr::null(),
                ));
            }
        }
        // One element gives both bounds.
        if !range.is_empty() {
            let last_idx = range.len() - 1;
            let (first, last) = (&range[0], &range[last_idx]);
            // Every item is an Integer, checked above.
            let expect = "the loop above rejected everything but Integers";
            let (first, last) = (
                first.as_integer().expect(expect),
                last.as_integer().expect(expect),
            );
            excmd.line1 = first as LineNr;
            excmd.line2 = last as LineNr;
        }
        if invalid_range(excmd).is_some() {
            return Err(err_bad_value(c"range", c""));
        }
    }

    if excmd.addr_count == 0 {
        if excmd.argt.has(ExArgt::DFLALL) {
            set_cmd_dflall_range(excmd);
        } else {
            excmd.line2 = get_cmd_default_range(excmd);
            excmd.line1 = excmd.line2;
            if excmd.addr_type == CmdAddr::Other {
                excmd.line2 = 1;
            }
        }
    }

    Ok(())
}

/// Apply `cmd.count`.
fn apply_count(
    cmd: &KeyDict_cmd,
    excmd: &mut ExArg,
    count_from_first_arg: bool,
) -> Result<(), Error> {
    let Some(count) = cmd.count else {
        return Ok(());
    };
    if count_from_first_arg {
        let why = c"Cannot specify both 'count' and numeric argument";
        return Err(Error::validation(why));
    }
    if !excmd.argt.has(ExArgt::COUNT) {
        return Err(err_cannot_accept(c"count", cmd));
    }
    if count < 0 as Integer {
        return Err(err_expected_at(
            c"count",
            c"non-negative Integer",
            ptr::null(),
        ));
    }
    set_cmd_count(excmd, count as LineNr, true);
    Ok(())
}

/// Apply `cmd.reg`.
fn apply_register(cmd: &KeyDict_cmd, excmd: &mut ExArg) -> Result<(), Error> {
    let Some(reg) = cmd.reg.as_ref() else {
        return Ok(());
    };
    if !excmd.argt.has(ExArgt::REGSTR) {
        return Err(err_cannot_accept(c"register", cmd));
    }
    if reg.len() != 1 {
        return Err(err_expected_at(c"reg", c"single character", reg.data()));
    }

    // SAFETY: the size is 1, so byte 0 is in bounds.
    let regname = unsafe { *reg.data() };
    if regname as c_int == '=' as c_int {
        return Err(Error::validation(c"Cannot use register \"="));
    }
    // `:put`/`:iput` read the register, everything else writes it.
    let writing =
        !is_user_cmd(excmd.cmdidx) && excmd.cmdidx != CmdIdx::put && excmd.cmdidx != CmdIdx::iput;
    if !valid_yank_reg(regname as c_int, writing) {
        // `%c` wrote the one byte, whatever it was.
        let byte = regname as u8;
        let reg = msg_bytes(core::slice::from_ref(&byte));
        return Err(api_error!(
            kErrorTypeValidation,
            "Invalid register: \"{reg}"
        ));
    }
    excmd.regname = regname as uint8_t as c_int;
    Ok(())
}

/// Apply `cmd.bang`.
fn apply_bang(cmd: &KeyDict_cmd, excmd: &mut ExArg) -> Result<(), Error> {
    excmd.forceit = cmd.bang.unwrap_or(false);
    if excmd.forceit && !excmd.argt.has(ExArgt::BANG) {
        return Err(err_cannot_accept(c"bang", cmd));
    }
    Ok(())
}

/// "Command cannot accept `what`: `name`" -- the shape four of the stages
/// above raise when a field contradicts the command's `argt`.
fn err_cannot_accept(what: &CStr, cmd: &KeyDict_cmd) -> Error {
    let what = msg_cstr(what);
    let name = cmd
        .cmd
        .as_ref()
        .map_or(c"", |cmd| cmd.as_cstr())
        .to_string_lossy();
    api_error!(kErrorTypeValidation, "Command cannot accept {what}: {name}")
}

/// Unpack the `magic` sub-keyset, defaulting each half to what `argt` says.
fn apply_magic(
    cmd: &KeyDict_cmd,
    excmd: &mut ExArg,
    cmdinfo: &mut CmdParseInfo,
) -> Result<(), Error> {
    let argt_file = excmd.argt.has(ExArgt::XFILE);
    let argt_bar = excmd.argt.has(ExArgt::TRLBAR);

    let Some(given) = cmd.magic.as_ref() else {
        cmdinfo.magic.file = argt_file;
        cmdinfo.magic.bar = argt_bar;
        return Ok(());
    };

    let get_field = Some(key_dict_cmd_magic_get_field as _);
    let magic = sub_keyset::<KeyDict_cmd_magic>(given, get_field)?;

    cmdinfo.magic.file = magic.file.unwrap_or(argt_file);
    cmdinfo.magic.bar = magic.bar.unwrap_or(argt_bar);

    // `magic.file` overrides `XFILE` for the expansion `execute_cmd` does.
    if cmdinfo.magic.file {
        excmd.argt |= ExArgt::XFILE;
    } else {
        excmd.argt.clear(ExArgt::XFILE);
    }
    Ok(())
}

/// Unpack the `mods` sub-keyset into `cmdinfo.cmdmod`.
fn apply_mods(cmd: &KeyDict_cmd, excmd: &ExArg, cmdinfo: &mut CmdParseInfo) -> Result<(), Error> {
    let Some(given) = cmd.mods.as_ref() else {
        return Ok(());
    };

    let get_field = Some(key_dict_cmd_mods_get_field as _);
    let mods = sub_keyset::<KeyDict_cmd_mods>(given, get_field)?;
    let mods = &mods;

    if mods.filter.is_some() {
        apply_filter_mod(mods, cmdinfo)?;
    }

    // Saturating: both are caller Integers, so INT_MAX would otherwise end
    // the process here. C wraps.
    if let Some(tab) = mods.tab.filter(|&tab| tab >= 0) {
        cmdinfo.cmdmod.cmod_tab = (tab as c_int).saturating_add(1);
    }
    if let Some(verbose) = mods.verbose.filter(|&verbose| verbose >= 0) {
        cmdinfo.cmdmod.cmod_verbose = (verbose as c_int).saturating_add(1);
    }

    if mods.vertical.unwrap_or(false) {
        cmdinfo.cmdmod.cmod_split |= WSP_VERT as c_int;
    }
    if mods.horizontal.unwrap_or(false) {
        cmdinfo.cmdmod.cmod_split |= WSP_HOR as c_int;
    }
    if let Some(named) = mods.split.as_ref() {
        // SAFETY: `mods.split` is a NUL-terminated keydict String.
        let split = unsafe { CStr::from_ptr(named.data()) };
        match split_direction(split) {
            Some(Some(bit)) => cmdinfo.cmdmod.cmod_split |= bit,
            // The empty string is "no direction", not a bad one.
            Some(None) => {}
            None => return Err(err_bad_value(c"mods.split", c"")),
        }
    }

    for (set, bit) in [
        (mods.silent, CmdModFlags::SILENT),
        (mods.emsg_silent, CmdModFlags::ERRSILENT),
        (mods.unsilent, CmdModFlags::UNSILENT),
        (mods.sandbox, CmdModFlags::SANDBOX),
        (mods.noautocmd, CmdModFlags::NOAUTOCMD),
        (mods.browse, CmdModFlags::BROWSE),
        (mods.confirm, CmdModFlags::CONFIRM),
        (mods.hide, CmdModFlags::HIDE),
        (mods.keepalt, CmdModFlags::KEEPALT),
        (mods.keepjumps, CmdModFlags::KEEPJUMPS),
        (mods.keepmarks, CmdModFlags::KEEPMARKS),
        (mods.keeppatterns, CmdModFlags::KEEPPATTERNS),
        (mods.lockmarks, CmdModFlags::LOCKMARKS),
        (mods.noswapfile, CmdModFlags::NOSWAPFILE),
    ] {
        if set.unwrap_or(false) {
            cmdinfo.cmdmod.cmod_flags |= bit;
        }
    }
    if cmdinfo.cmdmod.cmod_flags.has(CmdModFlags::ERRSILENT) {
        cmdinfo.cmdmod.cmod_flags |= CmdModFlags::SILENT;
    }

    if cmdinfo.cmdmod.cmod_flags.has(CmdModFlags::SANDBOX) && !excmd.argt.has(ExArgt::SBOXOK) {
        return Err(Error::validation(c"Command cannot be run in sandbox"));
    }

    Ok(())
}

/// `Some(Some(bit))` for a known split direction, `Some(None)` for the empty
/// string, `None` for a name that is neither.
fn split_direction(name: &CStr) -> Option<Option<c_int>> {
    if name.is_empty() {
        return Some(None);
    }
    let bit = match name.to_bytes() {
        b"aboveleft" | b"leftabove" => WSP_ABOVE,
        b"belowright" | b"rightbelow" => WSP_BELOW,
        b"topleft" => WSP_TOP,
        b"botright" => WSP_BOT,
        _ => return None,
    };
    Some(Some(bit as c_int))
}

/// Unpack `mods.filter` and compile its pattern.
fn apply_filter_mod(mods: &KeyDict_cmd_mods, cmdinfo: &mut CmdParseInfo) -> Result<(), Error> {
    let get_field = Some(key_dict_cmd_mods_filter_get_field as _);
    let filter = sub_keyset::<KeyDict_cmd_mods_filter>(
        mods.filter.as_ref().unwrap_or(&ApiDict::EMPTY),
        get_field,
    )?;
    let Some(pattern) = filter.pattern.as_ref() else {
        return Ok(());
    };

    cmdinfo.cmdmod.cmod_filter_force = filter.force.unwrap_or(false);
    // A bare `filter!` with an empty pattern still inverts the match.
    // SAFETY: `pattern` is a NUL-terminated keydict String.
    if unsafe { *pattern.data() } as c_int != NUL || cmdinfo.cmdmod.cmod_filter_force {
        let pat = string_to_cstr(pattern);
        cmdinfo.cmdmod.cmod_filter_pat = pat;
        // SAFETY: as above.
        cmdinfo.cmdmod.cmod_filter_regmatch.regprog = unsafe { vim_regcomp(pat, RE_MAGIC) };
    }
    Ok(())
}

/// Consume any leading `++opt` arguments off the rendered command line.
fn apply_argopt(excmd: &mut ExArg) -> Result<(), Error> {
    if !excmd.argt.has(ExArgt::ARGOPT) {
        return Ok(());
    }
    loop {
        // SAFETY: caller contract; `getargopt` only ever advances `excmd.arg`
        // within the same line, so the two bytes stay readable.
        let opt = unsafe {
            *excmd.arg_ptr() as c_int == '+' as c_int
                && *excmd.arg_ptr().add(1) as c_int == '+' as c_int
        };
        if !opt {
            return Ok(());
        }
        let orig_arg = excmd.arg_ptr();
        if getargopt(excmd).is_err() && !is_cmd_ni(excmd.cmdidx) {
            return Err(err_invalid_at(c"argument ", orig_arg));
        }
    }
}

/// Run the prepared command, capturing its messages when asked.
///
/// # Safety
/// The answer's storage is the api's own: the caller frees whatever this hands
/// back.
unsafe fn run_cmd(
    channel_id: uint64_t,
    excmd: &mut ExArg,
    cmdinfo: &mut CmdParseInfo,
    capture: bool,
) -> Result<String_0, Error> {
    let mut capture_local = GArray {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    let save_redir_off = redir_off.get();
    let save_capture_ga = capture_ga.get();
    let save_msg_col = msg_col.get();
    if capture {
        // SAFETY: `capture_local` outlives `execute_cmd`, which is the only
        // thing that can reach `capture_ga`.
        unsafe { ga_init(&raw mut capture_local, 1, 80) };
        capture_ga.set(&raw mut capture_local);
    }

    let mut tstate = TryState {
        current_exception: ptr::null_mut(),
        private_msg_list: ptr::null_mut(),
        msg_list: ptr::null(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    // SAFETY: `tstate` is paired with the `try_leave` below.
    unsafe { try_enter(&raw mut tstate) };
    // Captured output must not also reach the message grid.
    let silenced = capture.then(Suppress::messages_saved);
    if capture {
        redir_off.set(false);
        msg_col.set(0);
    }

    let sctx = api_set_sctx(channel_id);
    // SAFETY: `excmd`/`cmdinfo` are fully prepared; this is the call the whole
    // function exists to make.
    unsafe { execute_cmd(excmd, cmdinfo, false) };
    drop(sctx);

    drop(silenced);
    if capture {
        capture_ga.set(save_capture_ga);
        redir_off.set(save_redir_off);
        msg_col.set(save_msg_col);
    }
    // SAFETY: paired with the `try_enter` above.
    let caught = unsafe { try_leave(&raw mut tstate) };

    let mut retv = String_0::NULL;
    if caught.is_ok() && capture && capture_local.ga_len > 1 {
        // SAFETY: the garray holds `ga_len` bytes of message text.
        let captured = unsafe {
            core::slice::from_raw_parts(
                capture_local.ga_data.cast::<u8>(),
                capture_local.ga_len as size_t,
            )
        };
        // Messages open with a newline the caller did not ask for.
        let skip = usize::from(captured[0] == b'\n');
        retv = String_0::from_bytes(&captured[skip..]);
    }
    if capture {
        // SAFETY: initialised above under the same condition.
        unsafe { ga_clear(&raw mut capture_local) };
    }
    caught.map(|()| retv)
}
