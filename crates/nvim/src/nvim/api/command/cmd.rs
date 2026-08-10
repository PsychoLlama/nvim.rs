//! `nvim_cmd()`: executing a command given as a Dict.
//!
//! The inverse of [`super::parse`]: every field is validated against the
//! command's `argt` flags (which arguments it accepts, whether it takes a
//! range, a count, a register or a bang), the `mods` sub-keyset is unpacked
//! into an `cmdmod_T`, and the result is handed to `execute_cmd` -- with
//! the output captured when `opts.output` is set.
//!
//! The Dict is consumed in stages, in the order the command line itself
//! would be: resolve the name, collect the arguments, apply the address
//! (range, count, register, bang), unpack `magic` and `mods`, render the
//! whole thing back into a command line for `++opt` parsing, and only then
//! execute. Each stage answers "may we keep going?" -- `false`/`None` means
//! either `err` is set or the Dict asked for nothing executable, both of
//! which end the call without running anything.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::api::private::helpers::{array_add, has_key};
use ::core::ffi::{c_char, c_int, c_uint};
use ::core::ptr;

const EMPTY_STRING: String_0 = String_0 {
    data: ptr::null_mut(),
    size: 0,
};

const EMPTY_ARRAY: Array = Array {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};

pub unsafe extern "C" fn nvim_cmd(
    channel_id: uint64_t,
    cmd: *mut KeyDict_cmd,
    opts: *mut KeyDict_cmd_opts,
    arena: *mut Arena,
    err: *mut Error,
) -> String_0 {
    // SAFETY: the dispatcher decodes both keydicts onto its own frame and
    // keeps them alive across the call; neither is reachable from anything
    // this function runs, so a shared borrow of each holds throughout.
    let (cmd, opts) = unsafe { (&*cmd, &*opts) };

    // SAFETY: `exarg_T` and `CmdParseInfo` are plain C aggregates whose
    // all-zero state is the valid "nothing parsed yet" one; the C original
    // clears both with CLEAR_FIELD.
    let mut ea: exarg_T = unsafe { ::core::mem::zeroed() };
    let mut cmdinfo: CmdParseInfo = unsafe { ::core::mem::zeroed() };

    // Owned here rather than in `prepare_cmd` because `ea.cmdlinep` points at
    // it for the whole of `execute_cmd`.
    let mut cmdline: *mut c_char = ptr::null_mut();

    let mut retv = EMPTY_STRING;
    // SAFETY: `arena` and `err` are the dispatcher's, live for the call.
    if unsafe { prepare_cmd(cmd, &mut ea, &mut cmdinfo, &mut cmdline, arena, err) } {
        // SAFETY: `prepare_cmd` returning true means `ea`/`cmdinfo` describe
        // a resolved, validated command.
        retv = unsafe { run_cmd(channel_id, &mut ea, &mut cmdinfo, opts.output, arena, err) };
    }

    // SAFETY: all three are heap blocks this call owns; `build_cmdline_str`
    // and `getargopt` are the only writers.
    unsafe {
        xfree(cmdline.cast());
        xfree(ea.args.cast());
        xfree(ea.arglens.cast());
    }
    retv
}

/// Turn the Dict into a resolved, validated `exarg_T` plus its rendered
/// command line.
///
/// False means stop: either a stage set `err`, or the Dict carried modifiers
/// and nothing else, which upstream treats as a silent no-op.
unsafe fn prepare_cmd(
    cmd: &KeyDict_cmd,
    ea: &mut exarg_T,
    cmdinfo: &mut CmdParseInfo,
    cmdline: &mut *mut c_char,
    arena: *mut Arena,
    err: *mut Error,
) -> bool {
    // SAFETY (all): each stage takes the arena/error the caller was handed.
    let range_only = match unsafe { resolve_command(cmd, ea, arena, err) } {
        Some(range_only) => range_only,
        None => return false,
    };

    let mut args = EMPTY_ARRAY;
    let mut count_from_first_arg = false;
    if has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__args) {
        match unsafe { collect_args(cmd, ea, &mut args, arena, err) } {
            Some(from_first_arg) => count_from_first_arg = from_first_arg,
            None => return false,
        }
    }

    if !range_only {
        // Only the first argument is ever consulted.
        let first = if args.size > 0 {
            // SAFETY: `args` was built above; item 0 is in bounds and is a
            // String by construction.
            unsafe { (*args.items).data.string.data }
        } else {
            ptr::null_mut()
        };
        unsafe { set_cmd_addr_type(ea, first) };
    }

    let addressed = unsafe {
        apply_range(cmd, ea, err)
            && apply_count(cmd, ea, count_from_first_arg, err)
            && apply_register(cmd, ea, err)
            && apply_bang(cmd, ea, err)
            && apply_magic(cmd, ea, cmdinfo, err)
            && apply_mods(cmd, ea, cmdinfo, err)
    };
    if !addressed {
        return false;
    }

    // Render the Dict back into a command line: `execute_cmd` and everything
    // under it read `ea.arg`, not the Array.
    // SAFETY: `ea` is resolved and `args` holds only Strings.
    unsafe { build_cmdline_str(cmdline, ea, cmdinfo, args) };
    ea.cmdlinep = cmdline;
    // SAFETY: `ea.arg` now points into `*cmdline`.
    if !unsafe { apply_argopt(ea, err) } {
        return false;
    }
    if ea.argt & EX_CMDARG as uint32_t != 0 && ea.usefilter == 0 {
        // SAFETY: as above.
        ea.do_ecmd_cmd = unsafe { getargcmd(&raw mut ea.arg) };
    }
    true
}

/// Look `cmd.cmd` up in the command table, filling `ea.cmdidx`/`ea.argt`.
///
/// `Some(range_only)` on success -- a "range only" command such as `:1` has
/// no name at all. `None` means stop, per [`prepare_cmd`].
unsafe fn resolve_command(
    cmd: &KeyDict_cmd,
    ea: &mut exarg_T,
    arena: *mut Arena,
    err: *mut Error,
) -> Option<bool> {
    if !has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__cmd) {
        // SAFETY: `err` is the caller's.
        unsafe { api_err_required(err, c"cmd".as_ptr()) };
        return None;
    }

    // SAFETY: the key is set, so `cmd.cmd` is a NUL-terminated keydict String.
    let named = unsafe { *cmd.cmd.data } as c_int != NUL;
    let has_range = has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__range) && cmd.range.size > 0;
    let has_mods = has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__mods);

    if !named && !has_range && !has_mods {
        // SAFETY: `err` is the caller's.
        unsafe {
            api_err_exp(
                err,
                c"cmd".as_ptr(),
                c"non-empty String".as_ptr(),
                ptr::null(),
            )
        };
        return None;
    }

    // SAFETY: `arena` is the caller's; `find_ex_command` reads `ea.cmd`,
    // which the arena copy keeps alive for the whole call.
    let cmdname = unsafe { arena_string(arena, cmd.cmd) }.data;
    ea.cmd = cmdname;
    let mut p = unsafe { find_ex_command(ea, ptr::null_mut()) };

    // An unknown capitalised name plus a CmdUndefined autocommand is a lazily
    // defined user command: fire the event, then look again.
    if !p.is_null()
        && ea.cmdidx as c_int == CMD_SIZE as c_int
        && unsafe { *ea.cmd as u8 }.is_ascii_uppercase()
        && unsafe { has_event(EVENT_CMDUNDEFINED) }
    {
        // SAFETY: as above.
        unsafe {
            p = arena_string(arena, cmd.cmd).data;
            let ret = apply_autocmds(EVENT_CMDUNDEFINED, p, p, true, ptr::null_mut());
            p = if ret as c_int != 0 && !aborting() {
                find_ex_command(ea, ptr::null_mut())
            } else {
                ea.cmd
            };
        }
    }

    let unnamed_unknown = ea.cmdidx as c_int == CMD_SIZE as c_int && !named;
    let range_only = unnamed_unknown && has_range;

    // Modifiers and nothing else: upstream falls straight through to the
    // cleanup, with no error and nothing executed.
    if unnamed_unknown && !has_range && has_mods {
        return None;
    }

    if !(!p.is_null() && ea.cmdidx as c_int != CMD_SIZE as c_int) && !range_only {
        // SAFETY: `err` is the caller's, `cmdname` NUL-terminated.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Command not found: %s".as_ptr(),
                cmdname,
            )
        };
        return None;
    }

    if !range_only && unsafe { is_cmd_ni(ea.cmdidx) } {
        // SAFETY: as above.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Command not implemented: %s".as_ptr(),
                cmdname,
            )
        };
        return None;
    }

    if !range_only {
        // The Dict may abbreviate the name; it still has to be a prefix.
        // SAFETY: both names are NUL-terminated.
        let matched = unsafe {
            let fullname = if (ea.cmdidx as c_int) < 0 {
                get_user_command_name(ea.useridx, ea.cmdidx as c_int)
            } else {
                get_command_name(ptr::null_mut(), ea.cmdidx as c_int)
            };
            strncmp(fullname, cmdname, strlen(cmdname)) == 0
        };
        if !matched {
            // SAFETY: `err` is the caller's, `cmdname` NUL-terminated.
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Invalid command: \"%s\"".as_ptr(),
                    cmdname,
                )
            };
            return None;
        }
    }

    if range_only {
        ea.argt = (EX_RANGE | EX_SBOXOK) as uint32_t;
    } else if (ea.cmdidx as c_int) >= 0 {
        // A user command's flags already came out of `find_ex_command`.
        // SAFETY: `ea.cmdidx` is a valid index, checked just above.
        ea.argt = unsafe { excmd_get_argt(ea.cmdidx) };
    }

    Some(range_only)
}

/// Convert `cmd.args` into the `String`-only array the command line is built
/// from, and check the count against `argt`.
///
/// `Some(true)` means the one argument was consumed as the command's count.
unsafe fn collect_args(
    cmd: &KeyDict_cmd,
    ea: &mut exarg_T,
    args: &mut Array,
    arena: *mut Arena,
    err: *mut Error,
) -> Option<bool> {
    // For a command that takes a count but no regular arguments, a lone
    // numeric argument *is* the count.
    if cmd.args.size == 1
        && ea.argt & EX_COUNT as uint32_t != 0
        && ea.argt & EX_EXTRA as uint32_t == 0
    {
        // SAFETY: the size is 1, so item 0 is in bounds; the union arm is
        // chosen by `type_0`.
        let count = unsafe {
            let first = *cmd.args.items;
            match first.type_0 {
                kObjectTypeInteger => Some(first.data.integer as int64_t),
                kObjectTypeString => {
                    let str = first.data.string;
                    let mut endptr: *mut c_char = ptr::null_mut();
                    let val = strtol(str.data, &raw mut endptr, 10);
                    // The whole string has to be the number.
                    (*endptr as c_int == NUL && str.size > 0).then_some(val as int64_t)
                }
                _ => None,
            }
        };
        if let Some(count) = count
            && count >= 0
        {
            ea.addr_count = 1;
            ea.line2 = count as linenr_T;
            ea.line1 = ea.line2;
            *args = arena_array(arena, 0);
            return Some(true);
        }
    }

    *args = arena_array(arena, cmd.args.size);
    for i in 0..cmd.args.size {
        // SAFETY: `i` is in bounds; the union arm is chosen by `type_0`, and
        // `arena_alloc` hands back a block of the size asked for.
        let elem: Object = unsafe { *cmd.args.items.add(i) };
        match elem.type_0 {
            // A boolean argument is spelled to the command as "0" or "1".
            kObjectTypeBoolean => unsafe {
                let data_str: *mut c_char = arena_alloc(arena, 2, false).cast();
                *data_str = if elem.data.boolean { b'1' } else { b'0' } as c_char;
                *data_str.add(1) = NUL as c_char;
                array_add(args, Object::string(cstr_as_string(data_str)));
            },
            // A handle is its id, like any integer.
            kObjectTypeBuffer | kObjectTypeWindow | kObjectTypeTabpage | kObjectTypeInteger => unsafe {
                let data_str: *mut c_char = arena_alloc(arena, NUMBUFLEN as size_t, false).cast();
                snprintf(
                    data_str,
                    NUMBUFLEN as size_t,
                    c"%ld".as_ptr(),
                    elem.data.integer,
                );
                array_add(args, Object::string(cstr_as_string(data_str)));
            },
            kObjectTypeString => {
                // An all-whitespace argument would vanish into the separators.
                // SAFETY: the union arm is a String, `err` is the caller's.
                if unsafe { string_iswhite(elem.data.string) } {
                    unsafe {
                        api_err_exp(
                            err,
                            c"command arg".as_ptr(),
                            c"non-whitespace".as_ptr(),
                            ptr::null(),
                        )
                    };
                    return None;
                }
                // SAFETY: `args` has room, reserved above.
                unsafe { array_add(args, elem) };
            }
            _ => {
                // SAFETY: `err` is the caller's.
                unsafe {
                    api_err_exp(
                        err,
                        c"command arg".as_ptr(),
                        c"valid type".as_ptr(),
                        api_typename(elem.type_0),
                    )
                };
                return None;
            }
        }
    }

    let argc_valid = match ea.argt & (EX_EXTRA | EX_NOSPC | EX_NEEDARG) as uint32_t {
        v if v == (EX_EXTRA | EX_NOSPC | EX_NEEDARG) as uint32_t => args.size == 1,
        v if v == (EX_EXTRA | EX_NOSPC) as uint32_t => args.size <= 1,
        v if v == (EX_EXTRA | EX_NEEDARG) as uint32_t => args.size >= 1,
        v if v == EX_EXTRA as uint32_t => true,
        _ => args.size == 0,
    };
    if !argc_valid {
        // SAFETY: `err` is the caller's.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Wrong number of arguments".as_ptr(),
            )
        };
        return None;
    }

    Some(false)
}

/// Apply `cmd.range`, then fall back to the command's default range.
unsafe fn apply_range(cmd: &KeyDict_cmd, ea: &mut exarg_T, err: *mut Error) -> bool {
    if has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__range) {
        if ea.argt & EX_RANGE as uint32_t == 0 {
            // SAFETY: `err` is the caller's, `cmd.cmd` NUL-terminated.
            unsafe { err_cannot_accept(err, c"range".as_ptr(), cmd) };
            return false;
        }
        if cmd.range.size > 2 {
            // SAFETY: `err` is the caller's.
            unsafe {
                api_err_exp(
                    err,
                    c"range".as_ptr(),
                    c"<=2 elements".as_ptr(),
                    ptr::null(),
                )
            };
            return false;
        }

        let range = cmd.range;
        ea.addr_count = range.size as c_int;
        for i in 0..range.size {
            // SAFETY: `i` is in bounds; the union arm is chosen by `type_0`.
            let elem: Object = unsafe { *range.items.add(i) };
            if elem.type_0 != kObjectTypeInteger || unsafe { elem.data.integer } < 0 {
                // SAFETY: `err` is the caller's.
                unsafe {
                    api_err_exp(
                        err,
                        c"range element".as_ptr(),
                        c"non-negative Integer".as_ptr(),
                        ptr::null(),
                    )
                };
                return false;
            }
        }
        // One element gives both bounds.
        if range.size > 0 {
            // SAFETY: both indices are in bounds and every item is an Integer,
            // checked above.
            unsafe {
                ea.line1 = (*range.items).data.integer as linenr_T;
                ea.line2 = (*range.items.add(range.size - 1)).data.integer as linenr_T;
            }
        }
        // SAFETY: `ea` is resolved.
        if !unsafe { invalid_range(ea) }.is_null() {
            // SAFETY: `err` is the caller's.
            unsafe { api_err_invalid(err, c"range".as_ptr(), c"".as_ptr(), 0, true) };
            return false;
        }
    }

    if ea.addr_count == 0 {
        // SAFETY: `ea` is resolved.
        unsafe {
            if ea.argt & EX_DFLALL as uint32_t != 0 {
                set_cmd_dflall_range(ea);
            } else {
                ea.line2 = get_cmd_default_range(ea);
                ea.line1 = ea.line2;
                if ea.addr_type as c_uint == ADDR_OTHER as c_uint {
                    ea.line2 = 1;
                }
            }
        }
    }

    true
}

/// Apply `cmd.count`.
unsafe fn apply_count(
    cmd: &KeyDict_cmd,
    ea: &mut exarg_T,
    count_from_first_arg: bool,
    err: *mut Error,
) -> bool {
    if !has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__count) {
        return true;
    }
    if count_from_first_arg {
        // SAFETY: `err` is the caller's.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Cannot specify both 'count' and numeric argument".as_ptr(),
            )
        };
        return false;
    }
    if ea.argt & EX_COUNT as uint32_t == 0 {
        // SAFETY: `err` is the caller's, `cmd.cmd` NUL-terminated.
        unsafe { err_cannot_accept(err, c"count".as_ptr(), cmd) };
        return false;
    }
    if cmd.count < 0 as Integer {
        // SAFETY: `err` is the caller's.
        unsafe {
            api_err_exp(
                err,
                c"count".as_ptr(),
                c"non-negative Integer".as_ptr(),
                ptr::null(),
            )
        };
        return false;
    }
    // SAFETY: `ea` is resolved.
    unsafe { set_cmd_count(ea, cmd.count as linenr_T, true) };
    true
}

/// Apply `cmd.reg`.
unsafe fn apply_register(cmd: &KeyDict_cmd, ea: &mut exarg_T, err: *mut Error) -> bool {
    if !has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__reg) {
        return true;
    }
    if ea.argt & EX_REGSTR as uint32_t == 0 {
        // SAFETY: `err` is the caller's, `cmd.cmd` NUL-terminated.
        unsafe { err_cannot_accept(err, c"register".as_ptr(), cmd) };
        return false;
    }
    if cmd.reg.size != 1 {
        // SAFETY: `err` is the caller's; `cmd.reg` is NUL-terminated.
        unsafe {
            api_err_exp(
                err,
                c"reg".as_ptr(),
                c"single character".as_ptr(),
                cmd.reg.data,
            )
        };
        return false;
    }

    // SAFETY: the size is 1, so byte 0 is in bounds.
    let regname = unsafe { *cmd.reg.data };
    if regname as c_int == '=' as c_int {
        // SAFETY: `err` is the caller's.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Cannot use register \"=".as_ptr(),
            )
        };
        return false;
    }
    // `:put`/`:iput` read the register, everything else writes it.
    let writing = (ea.cmdidx as c_int) >= 0
        && ea.cmdidx as c_int != CMD_put as c_int
        && ea.cmdidx as c_int != CMD_iput as c_int;
    // SAFETY: `err` is the caller's.
    if !unsafe { valid_yank_reg(regname as c_int, writing) } {
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Invalid register: \"%c".as_ptr(),
                regname as c_int,
            )
        };
        return false;
    }
    ea.regname = regname as uint8_t as c_int;
    true
}

/// Apply `cmd.bang`.
unsafe fn apply_bang(cmd: &KeyDict_cmd, ea: &mut exarg_T, err: *mut Error) -> bool {
    ea.forceit = cmd.bang as c_int;
    if ea.forceit != 0 && ea.argt & EX_BANG as uint32_t == 0 {
        // SAFETY: `err` is the caller's, `cmd.cmd` NUL-terminated.
        unsafe { err_cannot_accept(err, c"bang".as_ptr(), cmd) };
        return false;
    }
    true
}

/// "Command cannot accept `what`: `name`" -- the shape four of the stages
/// above raise when a field contradicts the command's `argt`.
///
/// # Safety
/// `err` must be live and `cmd.cmd` NUL-terminated.
unsafe fn err_cannot_accept(err: *mut Error, what: *const c_char, cmd: &KeyDict_cmd) {
    unsafe {
        api_set_error(
            err,
            kErrorTypeValidation,
            c"Command cannot accept %s: %s".as_ptr(),
            what,
            cmd.cmd.data,
        )
    };
}

/// Unpack the `magic` sub-keyset, defaulting each half to what `argt` says.
unsafe fn apply_magic(
    cmd: &KeyDict_cmd,
    ea: &mut exarg_T,
    cmdinfo: &mut CmdParseInfo,
    err: *mut Error,
) -> bool {
    let argt_file = ea.argt & EX_XFILE as uint32_t != 0;
    let argt_bar = ea.argt & EX_TRLBAR as uint32_t != 0;

    if !has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__magic) {
        cmdinfo.magic.file = argt_file;
        cmdinfo.magic.bar = argt_bar;
        return true;
    }

    // SAFETY: `magic` is a plain C aggregate whose zero state means "no key
    // set"; `api_dict_to_keydict` fills it from `cmd.magic`.
    let mut magic: KeyDict_cmd_magic = unsafe { ::core::mem::zeroed() };
    if !unsafe {
        api_dict_to_keydict(
            (&raw mut magic).cast(),
            Some(KeyDict_cmd_magic_get_field as _),
            cmd.magic,
            err,
        )
    } {
        return false;
    }

    cmdinfo.magic.file = if has_key(magic.is_set__cmd_magic_, KEYSET_OPTIDX_cmd_magic__file) {
        magic.file
    } else {
        argt_file
    };
    cmdinfo.magic.bar = if has_key(magic.is_set__cmd_magic_, KEYSET_OPTIDX_cmd_magic__bar) {
        magic.bar
    } else {
        argt_bar
    };

    // `magic.file` overrides EX_XFILE for the expansion `execute_cmd` does.
    if cmdinfo.magic.file {
        ea.argt |= EX_XFILE as uint32_t;
    } else {
        ea.argt &= !(EX_XFILE as uint32_t);
    }
    true
}

/// Unpack the `mods` sub-keyset into `cmdinfo.cmdmod`.
unsafe fn apply_mods(
    cmd: &KeyDict_cmd,
    ea: &exarg_T,
    cmdinfo: &mut CmdParseInfo,
    err: *mut Error,
) -> bool {
    if !has_key(cmd.is_set__cmd_, KEYSET_OPTIDX_cmd__mods) {
        return true;
    }

    // SAFETY: as in `apply_magic`.
    let mut mods: KeyDict_cmd_mods = unsafe { ::core::mem::zeroed() };
    if !unsafe {
        api_dict_to_keydict(
            (&raw mut mods).cast(),
            Some(KeyDict_cmd_mods_get_field as _),
            cmd.mods,
            err,
        )
    } {
        return false;
    }
    let mods = &mods;

    if has_key(mods.is_set__cmd_mods_, KEYSET_OPTIDX_cmd_mods__filter)
        // SAFETY: `err` is the caller's.
        && !unsafe { apply_filter_mod(mods, cmdinfo, err) }
    {
        return false;
    }

    // Saturating: both are caller Integers, so INT_MAX would otherwise end
    // the process here. C wraps.
    if has_key(mods.is_set__cmd_mods_, KEYSET_OPTIDX_cmd_mods__tab) && mods.tab >= 0 {
        cmdinfo.cmdmod.cmod_tab = (mods.tab as c_int).saturating_add(1);
    }
    if has_key(mods.is_set__cmd_mods_, KEYSET_OPTIDX_cmd_mods__verbose) && mods.verbose >= 0 {
        cmdinfo.cmdmod.cmod_verbose = (mods.verbose as c_int).saturating_add(1);
    }

    if mods.vertical {
        cmdinfo.cmdmod.cmod_split |= WSP_VERT as c_int;
    }
    if mods.horizontal {
        cmdinfo.cmdmod.cmod_split |= WSP_HOR as c_int;
    }
    if has_key(mods.is_set__cmd_mods_, KEYSET_OPTIDX_cmd_mods__split) {
        // SAFETY: `mods.split` is a NUL-terminated keydict String.
        match unsafe { split_direction(mods.split.data) } {
            Some(Some(bit)) => cmdinfo.cmdmod.cmod_split |= bit,
            // The empty string is "no direction", not a bad one.
            Some(None) => {}
            None => {
                // SAFETY: `err` is the caller's.
                unsafe { api_err_invalid(err, c"mods.split".as_ptr(), c"".as_ptr(), 0, true) };
                return false;
            }
        }
    }

    for (set, bit) in [
        (mods.silent, CMOD_SILENT),
        (mods.emsg_silent, CMOD_ERRSILENT),
        (mods.unsilent, CMOD_UNSILENT),
        (mods.sandbox, CMOD_SANDBOX),
        (mods.noautocmd, CMOD_NOAUTOCMD),
        (mods.browse, CMOD_BROWSE),
        (mods.confirm, CMOD_CONFIRM),
        (mods.hide, CMOD_HIDE),
        (mods.keepalt, CMOD_KEEPALT),
        (mods.keepjumps, CMOD_KEEPJUMPS),
        (mods.keepmarks, CMOD_KEEPMARKS),
        (mods.keeppatterns, CMOD_KEEPPATTERNS),
        (mods.lockmarks, CMOD_LOCKMARKS),
        (mods.noswapfile, CMOD_NOSWAPFILE),
    ] {
        if set {
            cmdinfo.cmdmod.cmod_flags |= bit as c_int;
        }
    }
    if cmdinfo.cmdmod.cmod_flags & CMOD_ERRSILENT as c_int != 0 {
        cmdinfo.cmdmod.cmod_flags |= CMOD_SILENT as c_int;
    }

    if cmdinfo.cmdmod.cmod_flags & CMOD_SANDBOX as c_int != 0
        && ea.argt & EX_SBOXOK as uint32_t == 0
    {
        // SAFETY: `err` is the caller's.
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"Command cannot be run in sandbox".as_ptr(),
            )
        };
        return false;
    }

    true
}

/// `Some(Some(bit))` for a known split direction, `Some(None)` for the empty
/// string, `None` for a name that is neither.
///
/// # Safety
/// `name` must be NUL-terminated.
unsafe fn split_direction(name: *const c_char) -> Option<Option<c_int>> {
    unsafe {
        if *name as c_int == NUL {
            return Some(None);
        }
        let is = |lit: *const c_char| strcmp(name, lit) == 0;
        let bit = if is(c"aboveleft".as_ptr()) || is(c"leftabove".as_ptr()) {
            WSP_ABOVE
        } else if is(c"belowright".as_ptr()) || is(c"rightbelow".as_ptr()) {
            WSP_BELOW
        } else if is(c"topleft".as_ptr()) {
            WSP_TOP
        } else if is(c"botright".as_ptr()) {
            WSP_BOT
        } else {
            return None;
        };
        Some(Some(bit as c_int))
    }
}

/// Unpack `mods.filter` and compile its pattern.
unsafe fn apply_filter_mod(
    mods: &KeyDict_cmd_mods,
    cmdinfo: &mut CmdParseInfo,
    err: *mut Error,
) -> bool {
    // SAFETY: as in `apply_magic`.
    let mut filter: KeyDict_cmd_mods_filter = unsafe { ::core::mem::zeroed() };
    if !unsafe {
        api_dict_to_keydict(
            (&raw mut filter).cast(),
            Some(KeyDict_cmd_mods_filter_get_field as _),
            mods.filter,
            err,
        )
    } {
        return false;
    }
    if !has_key(
        filter.is_set__cmd_mods_filter_,
        KEYSET_OPTIDX_cmd_mods_filter__pattern,
    ) {
        return true;
    }

    cmdinfo.cmdmod.cmod_filter_force = filter.force;
    // A bare `filter!` with an empty pattern still inverts the match.
    // SAFETY: `filter.pattern` is a NUL-terminated keydict String.
    if unsafe { *filter.pattern.data } as c_int != NUL || cmdinfo.cmdmod.cmod_filter_force {
        // SAFETY: the pattern outlives the compiled program, which
        // `undo_cmdmod` frees.
        unsafe {
            cmdinfo.cmdmod.cmod_filter_pat = string_to_cstr(filter.pattern);
            cmdinfo.cmdmod.cmod_filter_regmatch.regprog =
                vim_regcomp(cmdinfo.cmdmod.cmod_filter_pat, RE_MAGIC);
        }
    }
    true
}

/// Consume any leading `++opt` arguments off the rendered command line.
unsafe fn apply_argopt(ea: &mut exarg_T, err: *mut Error) -> bool {
    if ea.argt & EX_ARGOPT as uint32_t == 0 {
        return true;
    }
    // SAFETY: `ea.arg` points into the NUL-terminated command line
    // `build_cmdline_str` rendered, and `getargopt` only advances it.
    unsafe {
        while *ea.arg as c_int == '+' as c_int && *ea.arg.add(1) as c_int == '+' as c_int {
            let orig_arg = ea.arg;
            if getargopt(ea) == 0 && !is_cmd_ni(ea.cmdidx) {
                api_err_invalid(err, c"argument ".as_ptr(), orig_arg, 0, true);
                return false;
            }
        }
    }
    true
}

/// Run the prepared command, capturing its messages when asked.
unsafe fn run_cmd(
    channel_id: uint64_t,
    ea: &mut exarg_T,
    cmdinfo: &mut CmdParseInfo,
    capture: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> String_0 {
    let mut capture_local = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    let save_msg_silent = msg_silent.get();
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
    if capture {
        // Captured output must not also reach the message grid.
        msg_silent.set(msg_silent.get() + 1);
        redir_off.set(false);
        msg_col.set(0);
    }

    let save_current_sctx = api_set_sctx(channel_id);
    // SAFETY: `ea`/`cmdinfo` are fully prepared; this is the call the whole
    // function exists to make.
    unsafe { execute_cmd(ea, cmdinfo, false) };
    current_sctx.set(save_current_sctx);

    if capture {
        capture_ga.set(save_capture_ga);
        msg_silent.set(save_msg_silent);
        redir_off.set(save_redir_off);
        msg_col.set(save_msg_col);
    }
    // SAFETY: paired with the `try_enter` above; `err` is the caller's.
    unsafe { try_leave(&raw mut tstate, err) };

    let mut retv = EMPTY_STRING;
    // SAFETY: `err` is the caller's.
    let failed = unsafe { (*err).type_0 as c_int != kErrorTypeNone as c_int };
    if !failed && capture && capture_local.ga_len > 1 {
        // SAFETY: the garray holds `ga_len` bytes of message text.
        retv = unsafe {
            arena_string(
                arena,
                String_0 {
                    data: capture_local.ga_data.cast(),
                    size: capture_local.ga_len as size_t,
                },
            )
        };
        // Messages open with a newline the caller did not ask for.
        // SAFETY: the arena copy is non-empty and NUL-terminated.
        if unsafe { *retv.data } as c_int == '\n' as c_int {
            retv.data = unsafe { retv.data.add(1) };
            retv.size -= 1;
        }
    }
    if capture {
        // SAFETY: initialised above under the same condition.
        unsafe { ga_clear(&raw mut capture_local) };
    }
    retv
}
