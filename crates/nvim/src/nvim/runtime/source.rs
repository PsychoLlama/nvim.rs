//! Sourcing a script -- `:source`, `:runtime`'s callback, and `nvim_exec2`.
//!
//! `do_source_ext` is the whole of it, and it reads its lines from one of
//! three places (see [`Origin`]): a file, the current buffer, or a string the
//! API handed over.  Whichever it is, the shape is the same -- resolve a name,
//! find or create the `scriptitem_T`, push it on the execution stack, run it,
//! and unwind all of that whatever happens -- so [`source_bracket`] reads as
//! that sequence of named stages.  Everything else here is an entry point into
//! it, or an accessor `do_cmdline` calls back through to ask about the source
//! it is reading from.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr, slice};

/// `:source` with `fname`, or without it when `fname` is empty.
///
/// # Safety
/// `fname` is NUL-terminated; `eap` is null or the running command.
unsafe fn cmd_source(fname: *mut c_char, eap: *mut exarg_T) {
    // SAFETY: the caller's contract on both arguments.
    let (named, addr_count, forceit) = unsafe {
        (
            *fname as c_int != NUL,
            (!eap.is_null()).then(|| (*eap).addr_count).unwrap_or(0),
            !eap.is_null() && (*eap).forceit != 0,
        )
    };
    if named && !eap.is_null() && addr_count > 0 {
        // A range only makes sense when the lines come from a buffer.
        // SAFETY: a static message.
        unsafe { emsg(gettext(&raw const e_norange as *const c_char)) };
        return;
    }
    // SAFETY: as above; every callee below takes the command or the name.
    unsafe {
        if !eap.is_null() && !named {
            if forceit {
                emsg(gettext(&raw const e_argreq as *const c_char));
            } else {
                cmd_source_buffer(eap, false);
            }
        } else if !eap.is_null() && forceit {
            // `:source!` feeds the file to the editor as typed keys.
            let busy = global_busy.get() != 0
                || listcmd_busy.get()
                || !(*eap).nextcmd.is_null()
                || (*(*eap).cstack).cs_idx >= 0;
            openscript(fname, busy);
        } else if do_source(fname, false, DOSO_NONE, ptr::null_mut()) == FAIL {
            semsg_c!(gettext(&raw const e_notopen as *const c_char), fname);
        }
    }
}

/// `:source`.
///
/// # Safety
/// `eap` is the running command.
pub unsafe fn ex_source(eap: *mut exarg_T) {
    // SAFETY: the caller's contract.
    unsafe { cmd_source((*eap).arg, eap) };
}

/// `:options`, which is `:source` of the option window script with the
/// command modifiers passed along in the environment.
///
/// # Safety
/// Called as an Ex command implementation; `eap` is unused.
pub unsafe fn ex_options(_eap: *mut exarg_T) {
    let mut buf = [0 as c_char; 500];
    let mut multi_mods = false;
    // SAFETY: `buf` is the scratch the modifiers are rendered into, and
    // `SYS_OPTWIN_FILE` is a NUL-terminated constant.
    unsafe {
        add_win_cmd_modifiers(buf.as_mut_ptr(), cmdmod.ptr(), &raw mut multi_mods);
        os_setenv(c"OPTWIN_CMD".as_ptr(), buf.as_ptr(), 1);
        cmd_source(SYS_OPTWIN_FILE.as_ptr().cast_mut(), ptr::null_mut());
    }
}

/// The breakpoint line of the script `cookie` is reading, for the debugger.
///
/// # Safety
/// `cookie` is a live [`source_cookie_T`].
pub unsafe extern "C" fn source_breakpoint(cookie: *mut c_void) -> *mut linenr_T {
    // SAFETY: the caller's contract.
    unsafe { &raw mut (*cookie.cast::<source_cookie_T>()).breakpoint }
}

/// The `debug_tick` the script `cookie` is reading last saw.
///
/// # Safety
/// `cookie` is a live [`source_cookie_T`].
pub unsafe extern "C" fn source_dbg_tick(cookie: *mut c_void) -> *mut c_int {
    // SAFETY: the caller's contract.
    unsafe { &raw mut (*cookie.cast::<source_cookie_T>()).dbg_tick }
}

/// The `:if`/`:while` nesting level the script `cookie` is reading started at.
///
/// # Safety
/// `cookie` is a live [`source_cookie_T`].
pub unsafe extern "C" fn source_level(cookie: *mut c_void) -> c_int {
    // SAFETY: the caller's contract.
    unsafe { (*cookie.cast::<source_cookie_T>()).level }
}

/// `fopen` for reading, with the descriptor kept out of child processes.
///
/// # Safety
/// `filename` is NUL-terminated.
unsafe fn fopen_noinh_readbin(filename: *mut c_char) -> *mut FILE {
    // SAFETY: the caller's contract; the descriptor is handed to `fdopen`,
    // which takes it over.
    unsafe {
        let fd = os_open(filename, O_RDONLY, 0);
        if fd < 0 {
            return ptr::null_mut();
        }
        os_set_cloexec(fd);
        fdopen(fd, READBIN.as_ptr())
    }
}

/// Append the continuation `p` to `ga`, answering whether the line was one.
///
/// A `"\ ` comment line is *not* a continuation but does not end one either,
/// so it answers true having appended nothing.
///
/// # Safety
/// `ga` is a garray the caller is collecting a script line in, and `p` names
/// `len` readable bytes.
pub(crate) unsafe extern "C" fn concat_continued_line(
    ga: *mut garray_T,
    init_growsize: c_int,
    p: *const c_char,
    len: size_t,
) -> bool {
    // SAFETY: the caller's contract; `skipwhite_len` stays within `len`.
    unsafe {
        let line = skipwhite_len(p, len);
        let len = len - line.offset_from(p) as size_t;
        if len >= 3 && strncmp(line, c"\"\\ ".as_ptr(), 3) == 0 {
            return true;
        }
        if len == 0 || *line as c_int != '\\' as c_int {
            return false;
        }
        // Grow by what has been collected so far (capped), so a line with
        // many continuations does not reallocate once per continuation.
        if (*ga).ga_len > init_growsize {
            ga_set_growsize(ga, (*ga).ga_len.min(8000));
        }
        ga_concat_len(ga, line.add(1), len - 1);
        true
    }
}

/// Allocate the next script ID and its `scriptitem_T`, taking ownership of
/// `name`.  IDs are never reused, so the registry is grown to cover every ID
/// up to the new one; the intervening items exist and are empty.
///
/// # Safety
/// `name` is owned memory or null; `sid_out` is null or writable.
pub unsafe extern "C" fn new_script_item(
    name: *mut c_char,
    sid_out: *mut scid_T,
) -> *mut scriptitem_T {
    /// The highest script ID handed out so far.
    static last_current_SID: GlobalCell<scid_T> = GlobalCell::new(0);

    let sid = last_current_SID.get() + 1;
    last_current_SID.set(sid);
    if !sid_out.is_null() {
        // SAFETY: the caller's out-parameter.
        unsafe { *sid_out = sid };
    }
    let ga = script_items.ptr();
    // SAFETY: `script_items` is this family's own garray of `scriptitem_T *`;
    // `ga_grow` makes room for every slot written below, and `xcalloc` zeroes
    // the item -- which is what upstream's `sn_name = NULL` and
    // `sn_prof_on = false` amount to.
    unsafe {
        ga_grow(ga, sid - (*ga).ga_len);
        while (*ga).ga_len < sid {
            let slot = (*ga)
                .ga_data
                .cast::<*mut scriptitem_T>()
                .add((*ga).ga_len as usize);
            slot.write(xcalloc(1, size_of::<scriptitem_T>()).cast());
            (*ga).ga_len += 1;
            new_script_vars((*ga).ga_len as scid_T);
        }
        let si = script_item(sid);
        (*si).sn_name = name;
        si
    }
}

/// Append an owned string to a garray of `char *`.
///
/// # Safety
/// `ga` holds `char *` items and `s` is owned memory.
unsafe fn ga_push_string(ga: *mut garray_T, s: *mut c_char) {
    // SAFETY: `ga_grow` leaves room for one more item at `ga_len`.
    unsafe {
        ga_grow(ga, 1);
        (*ga)
            .ga_data
            .cast::<*mut c_char>()
            .add((*ga).ga_len as usize)
            .write(s);
        (*ga).ga_len += 1;
    }
}

/// Collect `eap`'s range of the current buffer into `sp`, and answer the name
/// to show for those lines: the buffer's own file name, or a synthetic
/// `:source buffer=N` when it has none.
///
/// # Safety
/// `sp` is a cookie under construction and `eap` carries the range.
unsafe fn do_source_buffer_init(
    sp: &mut source_cookie_T,
    eap: *const exarg_T,
    ex_lua: bool,
) -> *mut c_char {
    let buf = curbuf.get();
    if buf.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: `buf` is the current buffer and `eap` the caller's command.
    let (ffname, handle, line1, line2) =
        unsafe { ((*buf).b_ffname, (*buf).handle, (*eap).line1, (*eap).line2) };
    let fname = if ffname.is_null() {
        let mut name = [0 as c_char; IOSIZE as usize];
        let fmt = if ex_lua {
            c":{range}lua buffer=%d"
        } else {
            c":source buffer=%d"
        };
        // SAFETY: `snprintf` NUL-terminates within `name`.
        unsafe {
            snprintf(name.as_mut_ptr(), IOSIZE as size_t, fmt.as_ptr(), handle);
            xstrdup(name.as_ptr())
        }
    } else {
        // SAFETY: the buffer's own file name.
        unsafe { xstrdup(ffname) }
    };
    let lines = &raw mut sp.buflines;
    // SAFETY: the cookie's own garray, and every line of the range is
    // readable through `ml_get`.
    unsafe {
        ga_init(lines, size_of::<*mut c_char>() as c_int, 100);
        for lnum in line1..=line2 {
            ga_push_string(lines, xstrdup(ml_get(lnum)));
        }
    }
    sp.buf_lnum = 0;
    sp.source_from_buf_or_str = true;
    // The first line the reader hands out is `line1`, so the counter starts
    // one below it.
    sp.sourcing_lnum = line1 - 1;
    fname
}

/// Split `str` into lines and collect them into `sp`.
///
/// # Safety
/// `sp` is a cookie under construction and `str` is NUL-terminated.
unsafe fn do_source_str_init(sp: &mut source_cookie_T, mut str: *const c_char) {
    let lines = &raw mut sp.buflines;
    // SAFETY: the cookie's own garray; `skip_to_newline` stops at the
    // terminator, so every span copied is within the string.
    unsafe {
        ga_init(lines, size_of::<*mut c_char>() as c_int, 100);
        while *str as c_int != NUL {
            let eol = skip_to_newline(str);
            let line = xmemdupz(str.cast(), eol.offset_from(str) as size_t);
            ga_push_string(lines, line.cast());
            // Step over the newline, unless this was the last line -- which
            // ends at the terminator instead.
            str = eol.add((*eol as c_int != NUL) as usize);
        }
    }
    sp.buf_lnum = 0;
    sp.source_from_buf_or_str = true;
}

/// Source the current buffer's lines, as Vimscript or (with `ex_lua`) as Lua.
///
/// # Safety
/// `eap` carries the range to run.
pub unsafe extern "C" fn cmd_source_buffer(eap: *const exarg_T, ex_lua: bool) {
    let req = SourceRequest::new(ptr::null_mut(), ptr::null(), eap, ex_lua);
    // SAFETY: the caller's contract.
    unsafe { do_source_ext(&req) };
}

/// Source `str` as Vimscript, under `traceback_name`.
///
/// The name is decorated with where the call came from, so an `nvim_exec2()`
/// nested inside a script says so.
///
/// # Safety
/// `str` is NUL-terminated and `traceback_name` names the caller.
pub unsafe extern "C" fn do_source_str(
    str: *const c_char,
    mut traceback_name: *mut c_char,
) -> c_int {
    let mut sname_buf = [0 as c_char; 256];
    let (name, lnum) = (estack::sourcing_name(), estack::sourcing_lnum());
    if !name.is_null() {
        let fmt = c"%s called at %s:%d".as_ptr();
        // SAFETY: `traceback_name` and `name` are NUL-terminated, and
        // `sname_buf` outlives the call below.
        unsafe {
            snprintf(
                sname_buf.as_mut_ptr(),
                sname_buf.len(),
                fmt,
                traceback_name,
                name,
                lnum,
            )
        };
        traceback_name = sname_buf.as_mut_ptr();
    }
    let req = SourceRequest::new(traceback_name, str, ptr::null(), false);
    // SAFETY: the caller's contract.
    unsafe { do_source_ext(&req) }
}

/// Where a `do_source_ext` call reads its lines from.
///
/// The C asks this with two null checks in seven separate places; naming it
/// is what makes the stages below readable.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    /// The current buffer, over `eap`'s range.
    Buffer,
    /// A string handed in by the API. `fname` is only a traceback name, and
    /// no `scriptitem_T` is allocated.
    Str,
    /// A file on disk named by `fname`.
    File,
}

/// One call into [`do_source_ext`], minus the cookie it fills in.
struct SourceRequest {
    origin: Origin,
    /// The name as the caller spelled it -- messages only; the resolved name
    /// is the cookie's.
    fname: *mut c_char,
    /// The script text, for [`Origin::Str`].
    str: *const c_char,
    /// Try the other spelling of the init file's name as well.
    check_other: bool,
    /// A `DOSO_` value: whether this is the user's init file.
    is_vimrc: c_int,
    /// Where to report the script ID, if the caller wants it.
    ret_sid: *mut c_int,
    /// The command that asked, for [`Origin::Buffer`]'s range.
    eap: *const exarg_T,
    /// Source a buffer as Lua regardless of what it looks like.
    ex_lua: bool,
}

impl SourceRequest {
    /// The request `fname`/`str` describe, with the init-file fields at their
    /// defaults; [`do_source`] is the only caller that sets those.
    fn new(fname: *mut c_char, str: *const c_char, eap: *const exarg_T, ex_lua: bool) -> Self {
        let origin = if fname.is_null() {
            debug_assert!(str.is_null(), "str == NULL");
            Origin::Buffer
        } else if str.is_null() {
            Origin::File
        } else {
            Origin::Str
        };
        SourceRequest {
            origin,
            fname,
            str,
            check_other: false,
            is_vimrc: DOSO_NONE,
            ret_sid: ptr::null_mut(),
            eap,
            ex_lua,
        }
    }

    fn is(&self, origin: Origin) -> bool {
        self.origin == origin
    }
}

/// Fill `cookie` from the buffer or the string, and answer the name this
/// source will be known by -- owned memory the caller frees.  `Err` is
/// `do_source_ext`'s whole return value: an empty buffer, a name that would
/// not expand, or a directory (the one case that reports itself).
///
/// # Safety
/// `cookie` is zeroed and the request's pointers are live.
unsafe fn source_name(
    req: &SourceRequest,
    cookie: &mut source_cookie_T,
) -> Result<*mut c_char, c_int> {
    match req.origin {
        Origin::Buffer => {
            // SAFETY: the caller's contract.
            let name = unsafe { do_source_buffer_init(cookie, req.eap, req.ex_lua) };
            if name.is_null() { Err(FAIL) } else { Ok(name) }
        }
        // SAFETY: the caller's contract.
        Origin::Str => unsafe {
            do_source_str_init(cookie, req.str);
            Ok(xstrdup(req.fname))
        },
        // SAFETY: the caller's contract; `expand_env_save` and `fix_fname`
        // answer owned memory or null.
        Origin::File => unsafe {
            let expanded = expand_env_save(req.fname);
            if expanded.is_null() {
                return Err(FAIL);
            }
            let name = fix_fname(expanded);
            xfree(expanded.cast());
            if name.is_null() {
                return Err(FAIL);
            }
            if !os_isdir(name) {
                return Ok(name);
            }
            let fmt = gettext(c"Cannot source a directory: \"%s\"".as_ptr());
            smsg_c!(0, fmt, req.fname);
            xfree(name.cast());
            Err(FAIL)
        },
    }
}

/// Run the `SourceCmd` and `SourcePre` autocommands.
///
/// `Some` when a `SourceCmd` handler took the file over, in which case
/// nothing else happens: the handler *is* the sourcing.
///
/// # Safety
/// `fname_exp` is the resolved name.
unsafe fn source_autocmds(fname_exp: *mut c_char) -> Option<c_int> {
    let buf = curbuf.get();
    // SAFETY: the caller's contract; the handlers may run arbitrary script,
    // which is why nothing is borrowed across them.
    unsafe {
        if has_autocmd(EVENT_SOURCECMD, fname_exp, ptr::null_mut())
            && apply_autocmds(EVENT_SOURCECMD, fname_exp, fname_exp, false, buf)
        {
            let retval = if aborting() { FAIL } else { OK };
            if retval == OK {
                apply_autocmds(EVENT_SOURCEPOST, fname_exp, fname_exp, false, curbuf.get());
            }
            return Some(retval);
        }
        apply_autocmds(EVENT_SOURCEPRE, fname_exp, fname_exp, false, curbuf.get());
    }
    None
}

/// Open the script file.
///
/// With `check_other` -- which only the init-file search sets -- a miss is
/// retried under the other spelling of the leading character, so `.nvimrc`
/// finds `_nvimrc` and `.exrc` finds `_exrc`.  The retry rewrites the name in
/// place, so later messages name the file that was actually opened.
///
/// # Safety
/// `fname_exp` is the resolved name, writable when `check_other` is set.
unsafe fn open_script(cookie: &mut source_cookie_T, fname_exp: *mut c_char, check_other: bool) {
    if !cookie.source_from_buf_or_str {
        // SAFETY: the caller's contract.
        cookie.fp = unsafe { fopen_noinh_readbin(fname_exp) };
    }
    if !cookie.fp.is_null() || !check_other {
        return;
    }
    // SAFETY: as above; `path_tail` answers a pointer inside `fname_exp`.
    unsafe {
        let tail = path_tail(fname_exp);
        let leader = *tail as c_int;
        if (leader == '.' as c_int || leader == '_' as c_int)
            && (strcasecmp(tail.add(1), c"nvimrc".as_ptr()) == 0
                || strcasecmp(tail.add(1), c"exrc".as_ptr()) == 0)
        {
            *tail = if leader == '_' as c_int { b'.' } else { b'_' } as c_char;
            cookie.fp = fopen_noinh_readbin(fname_exp);
        }
    }
}

/// One of the four `'verbose' > 1` sourcing traces.
///
/// `plain` is used at the top level and `numbered` -- which takes the line
/// number first -- when something else is already executing.
///
/// # Safety
/// Both formats take a `%s` for `fname`, and `numbered` a leading `%ld`.
unsafe fn verbose_source_msg(plain: &CStr, numbered: &CStr, fname: *const c_char) {
    let name = estack::sourcing_name();
    let lnum = estack::sourcing_lnum() as int64_t;
    // SAFETY: the caller's contract on the two formats.
    unsafe {
        verbose_enter();
        if name.is_null() {
            smsg_c!(0, gettext(plain.as_ptr()), fname);
        } else {
            smsg_c!(0, gettext(numbered.as_ptr()), lnum, fname);
        }
        verbose_leave();
    }
}

/// Find or create the `scriptitem_T` this source runs under.  A brand-new
/// item takes ownership of `*fname_exp` and the caller gets a copy back,
/// because the `SourcePost` autocommand at the end still needs a name to fire
/// with.  Sourcing a string allocates no item at all.
///
/// # Safety
/// `sid` and `fname_exp` are the caller's locals.
unsafe fn register_script(
    req: &SourceRequest,
    sid: &mut c_int,
    fname_exp: &mut *mut c_char,
) -> *mut scriptitem_T {
    if *sid > 0 {
        // Loading the same script again.
        // SAFETY: a positive `sid` names a registered script.
        return unsafe { script_item(*sid) };
    }
    if req.is(Origin::Str) {
        return ptr::null_mut();
    }
    // SAFETY: `new_script_item` takes ownership of the name and answers a
    // live item; the copy replaces it for the caller.
    unsafe {
        let si = new_script_item(*fname_exp, sid);
        (*si).sn_lua = path_with_extension(*fname_exp, c"lua".as_ptr());
        *fname_exp = xstrdup((*si).sn_name);
        if !req.ret_sid.is_null() {
            *req.ret_sid = *sid;
        }
        si
    }
}

/// Arm and start this script's profile timer.
///
/// # Safety
/// `si` is the script's live registry item.
unsafe fn profile_script_start(si: *mut scriptitem_T) {
    let mut forceit = false;
    // SAFETY: the caller's contract.
    unsafe {
        if !(*si).sn_prof_on && has_profiling(true, (*si).sn_name, &raw mut forceit) {
            profile_init(si);
            (*si).sn_pr_force = forceit;
        }
        if (*si).sn_prof_on {
            (*si).sn_pr_count += 1;
            (*si).sn_pr_start = profile_start();
            (*si).sn_pr_children = profile_zero();
        }
    }
}

/// Fold the elapsed time into the script's profile totals.
///
/// The item is looked up again rather than carried across the run: sourcing
/// can register more scripts, which reallocates the registry.
///
/// # Safety
/// A script is on the execution stack and `current_sctx` still names it.
unsafe fn profile_script_stop(wait_start: proftime_T) {
    // SAFETY: the caller's contract.
    unsafe {
        let si = script_item(current_sctx.get().sc_sid);
        if (*si).sn_prof_on {
            (*si).sn_pr_start = profile_end((*si).sn_pr_start);
            (*si).sn_pr_start = profile_sub_wait(wait_start, (*si).sn_pr_start);
            (*si).sn_pr_total = profile_add((*si).sn_pr_total, (*si).sn_pr_start);
            let children = (*si).sn_pr_children;
            (*si).sn_pr_self = profile_self((*si).sn_pr_self, (*si).sn_pr_start, children);
        }
    }
}

/// Whether the current buffer is Lua by 'filetype' or by file name.
///
/// # Safety
/// There is a current buffer.
unsafe fn curbuf_is_lua() -> bool {
    let buf = curbuf.get();
    // SAFETY: the caller's contract.
    unsafe {
        strequal((*buf).b_p_ft, c"lua".as_ptr())
            || (!(*buf).b_fname.is_null() && path_with_extension((*buf).b_fname, c"lua".as_ptr()))
    }
}

/// Whether treesitter parses `eap`'s range of the current buffer as Lua --
/// which is what makes a fenced Lua block inside a help file `:source`able.
///
/// # Safety
/// `eap` is null or the running command.
unsafe fn range_is_lua(eap: *const exarg_T) -> bool {
    if eap.is_null() {
        return false;
    }
    // SAFETY: the caller's command, and the current buffer.
    let (handle, line1, line2) = unsafe { ((*curbuf.get()).handle, (*eap).line1, (*eap).line2) };
    let mut items = [
        integer_obj(handle as Integer),
        integer_obj(line1 as Integer),
        integer_obj(line2 as Integer),
    ];
    let args = Array {
        size: items.len(),
        capacity: items.len(),
        items: items.as_mut_ptr(),
    };
    let mut err = Error {
        type_0: kErrorTypeNone,
        msg: ptr::null_mut(),
    };
    let src = c"return require('vim._core.util').source_is_lua(...)";
    let script = String_0 {
        data: src.as_ptr().cast_mut(),
        size: src.count_bytes(),
    };
    // SAFETY: `items` and `err` live on this frame and outlive the call,
    // which retains neither; the result's union is read under its own tag.
    unsafe {
        let nil = ptr::null_mut();
        let result = nlua_exec(script, ptr::null(), args, kRetNilBool, nil, &raw mut err);
        let is_lua = err.type_0 == kErrorTypeNone
            && result.type_0 == kObjectTypeBoolean as ObjectType
            && result.data.boolean;
        api_clear_error(&raw mut err);
        is_lua
    }
}

/// Strip a UTF-8 BOM off the first line, setting up the conversion it implies.
///
/// # Safety
/// `conv` is the cookie's converter and `firstline` is null or owned memory.
unsafe fn strip_bom(conv: *mut vimconv_T, firstline: *mut c_char) -> *mut c_char {
    // SAFETY: the caller's contract; the length check is what makes the
    // three-byte read in bounds.
    unsafe {
        if firstline.is_null() || strlen(firstline) < 3 {
            return firstline;
        }
        if slice::from_raw_parts(firstline.cast::<u8>(), 3) != b"\xef\xbb\xbf" {
            return firstline;
        }
        convert_setup(conv, c"utf-8".as_ptr().cast_mut(), p_enc.get());
        let rest = firstline.add(3);
        let mut recoded = string_convert(conv, rest, ptr::null_mut());
        if recoded.is_null() {
            recoded = xstrdup(rest);
        }
        xfree(firstline.cast());
        recoded
    }
}

/// Run the script's lines, in whichever language they turn out to be.  Answers
/// the first line when it was read here -- the Vimscript path reads it up
/// front so the BOM can be sniffed off it -- for the caller to free.
///
/// # Safety
/// The cookie is loaded, `si` is this script's item or null, and `fname_exp`
/// is the resolved name.
unsafe fn execute_source(
    req: &SourceRequest,
    cookie: &mut source_cookie_T,
    si: *mut scriptitem_T,
    fname_exp: *mut c_char,
) -> *mut c_char {
    // SAFETY: the caller's contract; both executors read the cookie's lines
    // or the file, not the cookie itself.
    unsafe {
        if req.is(Origin::Buffer) && (req.ex_lua || curbuf_is_lua() || range_is_lua(req.eap)) {
            nlua_exec_ga(&raw mut cookie.buflines, fname_exp);
            return ptr::null_mut();
        }
        if !si.is_null() && (*si).sn_lua {
            nlua_exec_file(fname_exp);
            return ptr::null_mut();
        }
    }
    // SAFETY: `getsourceline` reads the cookie back through the pointer it is
    // handed, which is derived afresh from the borrow for each call and stays
    // valid for its duration.
    unsafe {
        let first = getsourceline(0, ptr::from_mut(cookie).cast(), 0, true);
        let firstline = strip_bom(&raw mut cookie.conv, first);
        let flags = DOCMD_VERBOSE | DOCMD_NOWAIT | DOCMD_REPEAT;
        let reader = Some(getsourceline as LineGetterFn);
        do_cmdline(firstline, reader, ptr::from_mut(cookie).cast(), flags);
        firstline
    }
}

/// Release everything the cookie owns.
///
/// # Safety
/// The cookie is done being read from, and `firstline` is null or owned.
unsafe fn finish_source(cookie: &mut source_cookie_T, firstline: *mut c_char) {
    // SAFETY: the caller's contract.
    unsafe {
        if !cookie.fp.is_null() {
            fclose(cookie.fp);
        }
        if cookie.source_from_buf_or_str {
            ga_clear_strings(&raw mut cookie.buflines);
        }
        xfree(cookie.nextline.cast());
        xfree(firstline.cast());
        convert_setup(&raw mut cookie.conv, ptr::null_mut(), ptr::null_mut());
    }
}

/// Everything from opening the script to closing it, in the order it has to
/// come back down: the profile timers, the funccal stack, `current_sctx` and
/// the execution stack all outlive the run and are restored here.
///
/// # Safety
/// The cookie is loaded, `fname_exp` names the resolved script (the callee
/// may replace it with a copy it also owns), and `save_debug_break_level` was
/// read before any of this ran.
unsafe fn source_bracket(
    req: &SourceRequest,
    cookie: &mut source_cookie_T,
    fname_exp: &mut *mut c_char,
    save_debug_break_level: c_int,
) -> c_int {
    let mut sid = if req.is(Origin::Str) {
        SID_STR
    } else {
        // SAFETY: the resolved name.
        unsafe { find_script_by_name(*fname_exp) }
    };
    if sid > 0 && !req.ret_sid.is_null() {
        // Already loaded, and the caller only wanted the ID.
        // SAFETY: the caller's out-parameter.
        unsafe { *req.ret_sid = sid };
        return OK;
    }
    if !req.is(Origin::Str) {
        // SAFETY: the resolved name.
        if let Some(retval) = unsafe { source_autocmds(*fname_exp) } {
            return retval;
        }
    }

    // SAFETY: as above.
    unsafe { open_script(cookie, *fname_exp, req.check_other) };
    if cookie.fp.is_null() && !cookie.source_from_buf_or_str {
        if p_verbose.get() > 1 {
            let numbered = c"line %ld: could not source \"%s\"";
            // SAFETY: both formats take the name, `numbered` after the line.
            unsafe { verbose_source_msg(c"could not source \"%s\"", numbered, req.fname) };
        }
        return FAIL;
    }

    // The file exists.  Everything set up from here has to come back down
    // before this function returns.
    if p_verbose.get() > 1 {
        let numbered = c"line %ld: sourcing \"%s\"";
        // SAFETY: as above.
        unsafe { verbose_source_msg(c"sourcing \"%s\"", numbered, req.fname) };
    }
    if req.is_vimrc == DOSO_VIMRC {
        // SAFETY: the resolved name.
        unsafe { vimrc_found(*fname_exp, c"MYVIMRC".as_ptr().cast_mut()) };
    }

    // SAFETY: as above.
    cookie.breakpoint = unsafe { dbg_find_breakpoint(true, *fname_exp, 0) };
    cookie.fname = *fname_exp;
    cookie.dbg_tick = debug_tick.get();
    cookie.level = ex_nesting_level.get();

    // Start measuring load time, if --startuptime opened the log.
    let time_log = time_fd.get();
    let (rel_time, mut start_time) = if time_log.is_null() {
        (0, 0)
    } else {
        time_push()
    };
    let profiling = do_profiling.get() == PROF_YES;
    // SAFETY: paired with the `prof_child_exit` below.
    let wait_start = if profiling {
        unsafe { prof_child_enter() }
    } else {
        0
    };

    // Don't use the calling function's local variables.
    let mut funccalp_entry = funccal_entry_T {
        top_funccal: ptr::null_mut(),
        next: ptr::null_mut(),
    };
    // SAFETY: the entry lives on this frame until `restore_funccal` below.
    unsafe { save_funccal(&raw mut funccalp_entry) };
    let save_current_sctx = current_sctx.get();

    // Always use a new sequence number.
    let seq = last_current_SID_seq.get() + 1;
    last_current_SID_seq.set(seq);
    current_sctx.with_mut(|sctx| sctx.sc_seq = seq);

    // SAFETY: the resolved name, which a new item takes over.
    let si = unsafe { register_script(req, &mut sid, fname_exp) };
    debug_assert!(
        si.is_null() == req.is(Origin::Str),
        "(si != NULL) == (str == NULL)"
    );

    // Sourcing a string from a Lua script keeps the Lua script's SID, so
    // `:verbose` still names something useful.
    // SAFETY: `script_is_lua` only reads the registry.
    if !req.is(Origin::Str) || !unsafe { script_is_lua(current_sctx.get().sc_sid) } {
        current_sctx.with_mut(|sctx| {
            sctx.sc_sid = sid;
            sctx.sc_lnum = 0;
        });
    }

    // Keep the sourcing name and line number, for recursive calls.
    // SAFETY: `si` is this script's item, and the name outlives the frame it
    // is pushed onto.
    unsafe {
        let name = if si.is_null() {
            *fname_exp
        } else {
            (*si).sn_name
        };
        estack_push(ETYPE_SCRIPT, name, 0);
        if profiling && !si.is_null() {
            profile_script_start(si);
        }
    }
    cookie.conv.vc_type = CONV_NONE;

    // SAFETY: the loaded cookie and this script's item.
    let firstline = unsafe { execute_source(req, cookie, si, *fname_exp) };

    // SAFETY: `si` is still this script's, though the registry may have moved.
    unsafe {
        if profiling && !si.is_null() {
            profile_script_stop(wait_start);
        }
        if got_int.get() {
            emsg(gettext(&raw const e_interr as *const c_char));
        }
        estack_pop();
    }
    if p_verbose.get() > 1 {
        let resumed = estack::sourcing_name();
        // SAFETY: both messages take a NUL-terminated name.
        unsafe {
            verbose_enter();
            smsg_c!(0, gettext(c"finished sourcing %s".as_ptr()), req.fname);
            if !resumed.is_null() {
                smsg_c!(0, gettext(c"continuing in %s".as_ptr()), resumed);
            }
            verbose_leave();
        }
    }
    if !time_log.is_null() {
        let mut label = [0 as c_char; IOSIZE as usize];
        let buf = label.as_mut_ptr();
        // SAFETY: `label` outlives all three calls.
        unsafe {
            vim_snprintf(buf, IOSIZE as size_t, c"sourcing %s".as_ptr(), req.fname);
            time_msg(buf, &raw mut start_time);
            time_pop(rel_time);
        }
    }
    let trigger_source_post = !got_int.get();

    // After a `:finish` in debug mode, break at the first command of the next
    // sourced file.
    if save_debug_break_level > ex_nesting_level.get()
        && debug_break_level.get() == ex_nesting_level.get()
    {
        debug_break_level.set(debug_break_level.get() + 1);
    }
    current_sctx.set(save_current_sctx);

    // SAFETY: paired with the `save_funccal`/`prof_child_enter` above; the
    // cookie is done being read from.
    unsafe {
        restore_funccal();
        if profiling {
            prof_child_exit(wait_start);
        }
        finish_source(cookie, firstline);
    }
    if !req.is(Origin::Str) && trigger_source_post {
        let (name, buf) = (*fname_exp, curbuf.get());
        // SAFETY: the resolved name, still owned by this frame.
        unsafe { apply_autocmds(EVENT_SOURCEPOST, name, name, false, buf) };
    }
    OK
}

/// Read a script and run it.
///
/// Answers FAIL when the file could not be opened, OK otherwise.  When a
/// `scriptitem_T` was found or created, `ret_sid` -- if given -- gets its ID,
/// and a script that has one already is *not* run again.
///
/// # Safety
/// The request's pointers are live for the call.
unsafe fn do_source_ext(req: &SourceRequest) -> c_int {
    let save_debug_break_level = debug_break_level.get();
    // SAFETY: every field of the cookie is a pointer, an integer or a bool,
    // for all of which all-zero is the value C's `CLEAR_FIELD` leaves behind.
    let mut cookie: source_cookie_T = unsafe { mem::zeroed() };
    // SAFETY: the caller's contract.
    let mut fname_exp = match unsafe { source_name(req, &mut cookie) } {
        Ok(name) => name,
        Err(retval) => return retval,
    };
    // SAFETY: the cookie is loaded and `fname_exp` is owned by this frame.
    let retval = unsafe {
        let retval = source_bracket(req, &mut cookie, &mut fname_exp, save_debug_break_level);
        xfree(fname_exp.cast());
        retval
    };
    retval
}

/// [`do_source_ext`] for a file: the spelling every caller outside this module
/// uses.  `check_other` also tries the other init-file name; `is_vimrc` is a
/// `DOSO_` value.
///
/// # Safety
/// `fname` is NUL-terminated and `ret_sid` is null or writable.
pub unsafe extern "C" fn do_source(
    fname: *mut c_char,
    check_other: bool,
    is_vimrc: c_int,
    ret_sid: *mut c_int,
) -> c_int {
    let req = SourceRequest {
        check_other,
        is_vimrc,
        ret_sid,
        ..SourceRequest::new(fname, ptr::null(), ptr::null(), false)
    };
    // SAFETY: the caller's contract.
    unsafe { do_source_ext(&req) }
}
