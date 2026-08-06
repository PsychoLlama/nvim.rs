//! User commands: `:command` from the API.
//!
//! `create_user_command` is the shared implementation -- it validates the
//! name, decodes the `nargs`/`range`/`count`/`addr`/`complete` keyset into
//! the `uc_add_command` flags, and accepts either a command string or a
//! `LuaRef` -- and the four `nvim_*_user_command` entry points differ only
//! in whether they are buffer-local.  The two `get_commands` spellings
//! render the table back.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    unsafe {
        create_user_command(channel_id, name, cmd, opts, 0 as ::core::ffi::c_int, err);
    }
}

pub unsafe extern "C" fn nvim_del_user_command(mut name: String_0, mut err: *mut Error) {
    unsafe {
        nvim_buf_del_user_command(-1 as Buffer, name, err);
    }
}

pub unsafe extern "C" fn nvim_buf_create_user_command(
    mut channel_id: uint64_t,
    mut buf: Buffer,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut err: *mut Error,
) {
    unsafe {
        let mut target_buf: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return;
        }
        let mut save_curbuf: *mut buf_T = curbuf.get();
        curbuf.set(target_buf);
        create_user_command(
            channel_id,
            name,
            cmd,
            opts,
            UC_BUFFER as ::core::ffi::c_int,
            err,
        );
        curbuf.set(save_curbuf);
    }
}

pub unsafe extern "C" fn nvim_buf_del_user_command(
    mut buf: Buffer,
    mut name: String_0,
    mut err: *mut Error,
) {
    unsafe {
        let mut gap: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
        if buf == -1 as ::core::ffi::c_int {
            gap = ucmds.ptr();
        } else {
            let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return;
            }
            gap = &raw mut (*b).b_ucmds;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*gap).ga_len {
            let mut cmd: *mut ucmd_T = ((*gap).ga_data as *mut ucmd_T).offset(i as isize);
            if strcmp(name.data, (*cmd).uc_name) == 0 {
                free_ucmd(cmd);
                (*gap).ga_len -= 1 as ::core::ffi::c_int;
                if i < (*gap).ga_len {
                    memmove(
                        cmd as *mut ::core::ffi::c_void,
                        cmd.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        (((*gap).ga_len - i) as size_t)
                            .wrapping_mul(::core::mem::size_of::<ucmd_T>()),
                    );
                }
                return;
            }
            i += 1;
        }
        api_set_error(
            err,
            kErrorTypeException,
            c"Invalid command (not found): %s".as_ptr(),
            name.data,
        );
    }
}

pub unsafe extern "C" fn create_user_command(
    mut channel_id: uint64_t,
    mut name: String_0,
    mut cmd: Object,
    mut opts: *mut KeyDict_user_command,
    mut flags: ::core::ffi::c_int,
    mut err: *mut Error,
) {
    unsafe {
        let mut force: bool = false;
        let mut argt: uint32_t = 0 as uint32_t;
        let mut def: int64_t = -1 as int64_t;
        let mut addr_type_arg: cmd_addr_T = ADDR_NONE;
        let mut context: ::core::ffi::c_int = EXPAND_NOTHING as ::core::ffi::c_int;
        let mut compl_arg: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut rep: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut luaref: LuaRef = LUA_NOREF;
        let mut compl_luaref: LuaRef = LUA_NOREF;
        let mut preview_luaref: LuaRef = LUA_NOREF;
        '_err: {
            if uc_validate_name(name.data).is_null() {
                api_err_invalid(
                    err,
                    c"command name".as_ptr(),
                    name.data,
                    0 as int64_t,
                    true_0 != 0,
                );
            } else if mb_islower(
                *name.data.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            ) {
                api_err_invalid(
                    err,
                    c"command name (must start with uppercase)".as_ptr(),
                    name.data,
                    0 as int64_t,
                    true_0 != 0,
                );
            } else if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                & (1 as ::core::ffi::c_ulonglong) << 8 as ::core::ffi::c_int
                != 0 as ::core::ffi::c_ulonglong)
                || !((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 5 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong))
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"Cannot use both 'range' and 'count'".as_ptr(),
                );
            } else {
                if (*opts).nargs.type_0 as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    match (*opts).nargs.data.integer {
                        0 => {}
                        1 => {
                            argt = (argt as ::core::ffi::c_uint
                                | (EX_EXTRA | EX_NOSPC | EX_NEEDARG))
                                as uint32_t;
                        }
                        _ => {
                            if true {
                                api_err_invalid(
                                    err,
                                    c"nargs".as_ptr(),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                    (*opts).nargs.data.integer,
                                    false_0 != 0,
                                );
                                break '_err;
                            }
                        }
                    }
                } else if (*opts).nargs.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    if !((*opts).nargs.data.string.size <= 1 as size_t) {
                        api_err_invalid(
                            err,
                            c"nargs".as_ptr(),
                            (*opts).nargs.data.string.data,
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    } else {
                        match *(*opts)
                            .nargs
                            .data
                            .string
                            .data
                            .offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                        {
                            42 => {
                                argt = (argt as ::core::ffi::c_uint | EX_EXTRA) as uint32_t;
                            }
                            63 => {
                                argt = (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NOSPC))
                                    as uint32_t;
                            }
                            43 => {
                                argt = (argt as ::core::ffi::c_uint | (EX_EXTRA | EX_NEEDARG))
                                    as uint32_t;
                            }
                            _ => {
                                if true {
                                    api_err_invalid(
                                        err,
                                        c"nargs".as_ptr(),
                                        (*opts).nargs.data.string.data,
                                        0 as int64_t,
                                        true_0 != 0,
                                    );
                                    break '_err;
                                }
                            }
                        }
                    }
                } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__nargs
                    != 0 as ::core::ffi::c_ulonglong
                {
                    if true {
                        api_err_invalid(
                            err,
                            c"nargs".as_ptr(),
                            c"".as_ptr(),
                            0 as int64_t,
                            true_0 != 0,
                        );
                        break '_err;
                    }
                }
                if !(!((*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                    & (1 as ::core::ffi::c_ulonglong) << 10 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_ulonglong)
                    || argt != 0)
                {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"%s".as_ptr(),
                        c"'complete' used without 'nargs'".as_ptr(),
                    );
                } else {
                    if (*opts).range.type_0 as ::core::ffi::c_uint
                        == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if (*opts).range.data.boolean {
                            argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                            addr_type_arg = ADDR_LINES;
                        }
                    } else if (*opts).range.type_0 as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if !(*(*opts)
                            .range
                            .data
                            .string
                            .data
                            .offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == '%' as ::core::ffi::c_int
                            && (*opts).range.data.string.size == 1 as size_t)
                        {
                            api_err_invalid(
                                err,
                                c"range".as_ptr(),
                                c"".as_ptr(),
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        } else {
                            argt =
                                (argt as ::core::ffi::c_uint | (EX_RANGE | EX_DFLALL)) as uint32_t;
                            addr_type_arg = ADDR_LINES;
                        }
                    } else if (*opts).range.type_0 as ::core::ffi::c_uint
                        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        argt = (argt as ::core::ffi::c_uint | (EX_RANGE | EX_ZEROR)) as uint32_t;
                        def = (*opts).range.data.integer as int64_t;
                        addr_type_arg = ADDR_LINES;
                    } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__range
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if true {
                            api_err_invalid(
                                err,
                                c"range".as_ptr(),
                                c"".as_ptr(),
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        }
                    }
                    if (*opts).count.type_0 as ::core::ffi::c_uint
                        == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if (*opts).count.data.boolean {
                            argt = (argt as ::core::ffi::c_uint | (EX_COUNT | EX_ZEROR | EX_RANGE))
                                as uint32_t;
                            addr_type_arg = ADDR_OTHER;
                            def = 0 as int64_t;
                        }
                    } else if (*opts).count.type_0 as ::core::ffi::c_uint
                        == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        argt = (argt as ::core::ffi::c_uint | (EX_COUNT | EX_ZEROR | EX_RANGE))
                            as uint32_t;
                        addr_type_arg = ADDR_OTHER;
                        def = (*opts).count.data.integer as int64_t;
                    } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__count
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if true {
                            api_err_invalid(
                                err,
                                c"count".as_ptr(),
                                c"".as_ptr(),
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        }
                    }
                    if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__addr
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                            != (*opts).addr.type_0 as ::core::ffi::c_uint
                        {
                            api_err_exp(
                                err,
                                c"addr".as_ptr(),
                                api_typename(kObjectTypeString),
                                api_typename((*opts).addr.type_0),
                            );
                            break '_err;
                        } else if !(1 as ::core::ffi::c_int
                            == parse_addr_type_arg(
                                (*opts).addr.data.string.data,
                                (*opts).addr.data.string.size as ::core::ffi::c_int,
                                &raw mut addr_type_arg,
                            ))
                        {
                            api_err_invalid(
                                err,
                                c"addr".as_ptr(),
                                (*opts).addr.data.string.data,
                                0 as int64_t,
                                true_0 != 0,
                            );
                            break '_err;
                        } else {
                            argt = (argt as ::core::ffi::c_uint | EX_RANGE) as uint32_t;
                            if addr_type_arg as ::core::ffi::c_uint
                                != ADDR_LINES as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                argt = (argt as ::core::ffi::c_uint | EX_ZEROR) as uint32_t;
                            }
                        }
                    }
                    if (*opts).bang {
                        argt = (argt as ::core::ffi::c_uint | EX_BANG) as uint32_t;
                    }
                    if (*opts).bar {
                        argt = (argt as ::core::ffi::c_uint | EX_TRLBAR) as uint32_t;
                    }
                    if (*opts).register_ {
                        argt = (argt as ::core::ffi::c_uint | EX_REGSTR) as uint32_t;
                    }
                    if (*opts).keepscript {
                        argt = (argt as ::core::ffi::c_uint | EX_KEEPSCRIPT) as uint32_t;
                    }
                    force = if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__force
                        != 0 as ::core::ffi::c_ulonglong
                    {
                        (*opts).force as ::core::ffi::c_int
                    } else {
                        true_0
                    } != 0;
                    if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                        if (*opts).complete.type_0 as ::core::ffi::c_uint
                            == kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            context = EXPAND_USER_LUA as ::core::ffi::c_int;
                            compl_luaref = (*opts).complete.data.luaref;
                            (*opts).complete.data.luaref = LUA_NOREF as LuaRef;
                        } else if (*opts).complete.type_0 as ::core::ffi::c_uint
                            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if !(1 as ::core::ffi::c_int
                                == parse_compl_arg(
                                    (*opts).complete.data.string.data,
                                    (*opts).complete.data.string.size as ::core::ffi::c_int,
                                    &raw mut context,
                                    &raw mut argt,
                                    &raw mut compl_arg,
                                ))
                            {
                                api_err_invalid(
                                    err,
                                    c"complete".as_ptr(),
                                    (*opts).complete.data.string.data,
                                    0 as int64_t,
                                    true_0 != 0,
                                );
                                break '_err;
                            }
                        } else if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong)
                                << KEYSET_OPTIDX_user_command__complete
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            if true {
                                api_err_exp(
                                    err,
                                    c"complete".as_ptr(),
                                    c"Function or String".as_ptr(),
                                    ::core::ptr::null::<::core::ffi::c_char>(),
                                );
                                break '_err;
                            }
                        }
                        if (*opts).is_set__user_command_ as ::core::ffi::c_ulonglong
                            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_user_command__preview
                            != 0 as ::core::ffi::c_ulonglong
                        {
                            if kObjectTypeLuaRef as ::core::ffi::c_int as ::core::ffi::c_uint
                                != (*opts).preview.type_0 as ::core::ffi::c_uint
                            {
                                api_err_exp(
                                    err,
                                    c"preview".as_ptr(),
                                    api_typename(kObjectTypeLuaRef),
                                    api_typename((*opts).preview.type_0),
                                );
                                break '_err;
                            } else {
                                argt = (argt as ::core::ffi::c_uint | EX_PREVIEW) as uint32_t;
                                preview_luaref = (*opts).preview.data.luaref;
                                (*opts).preview.data.luaref = LUA_NOREF as LuaRef;
                            }
                        }
                        match cmd.type_0 as ::core::ffi::c_uint {
                            7 => {
                                luaref = api_new_luaref(cmd.data.luaref);
                                if (*opts).desc.type_0 as ::core::ffi::c_uint
                                    == kObjectTypeString as ::core::ffi::c_int
                                        as ::core::ffi::c_uint
                                {
                                    rep = (*opts).desc.data.string.data;
                                } else {
                                    rep = c"".as_ptr();
                                }
                            }
                            4 => {
                                rep = cmd.data.string.data;
                            }
                            _ => {
                                if true {
                                    api_err_exp(
                                        err,
                                        c"command".as_ptr(),
                                        c"Function or String".as_ptr(),
                                        ::core::ptr::null::<::core::ffi::c_char>(),
                                    );
                                    break '_err;
                                }
                            }
                        }
                        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
                        if uc_add_command(
                            name.data,
                            name.size,
                            rep,
                            argt,
                            def,
                            flags,
                            context,
                            compl_arg,
                            compl_luaref,
                            preview_luaref,
                            addr_type_arg,
                            luaref,
                            force,
                        ) != 1 as ::core::ffi::c_int
                        {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"Failed to create user command".as_ptr(),
                            );
                        }
                        current_sctx.set(save_current_sctx);
                        return;
                    }
                }
            }
        }
        if luaref != LUA_NOREF {
            api_free_luaref(luaref);
            luaref = LUA_NOREF as LuaRef;
        }
        if compl_luaref != LUA_NOREF {
            api_free_luaref(compl_luaref);
            compl_luaref = LUA_NOREF as LuaRef;
        }
        if preview_luaref != LUA_NOREF {
            api_free_luaref(preview_luaref);
            preview_luaref = LUA_NOREF as LuaRef;
        }
        xfree(compl_arg as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn nvim_get_commands(
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        return nvim_buf_get_commands(-1 as Buffer, opts, arena, err);
    }
}

pub unsafe extern "C" fn nvim_buf_get_commands(
    mut buf: Buffer,
    mut opts: *mut KeyDict_get_commands,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut global: bool = buf == -1 as ::core::ffi::c_int;
        if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        if global {
            if (*opts).builtin {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"builtin=true not implemented".as_ptr(),
                );
                return Dict {
                    size: 0 as size_t,
                    capacity: 0 as size_t,
                    items: ::core::ptr::null_mut::<KeyValuePair>(),
                };
            }
            return commands_array(::core::ptr::null_mut::<buf_T>(), arena);
        }
        let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
        if (*opts).builtin as ::core::ffi::c_int != 0 || b.is_null() {
            return Dict {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<KeyValuePair>(),
            };
        }
        return commands_array(b, arena);
    }
}
