//! Turning a context into a match array.
//!
//! [`expand_from_context`] is the dispatcher: file-like contexts go to
//! `expand_wildcards`, everything else to a generator, and the answer is
//! sorted, deduplicated and escaped.  [`expand_generic`] is the generic
//! generator loop every `get_*_name` callback is driven by, and
//! [`map_wildopts_to_ewflags`] translates `'wildoptions'` into `EW_*`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::path::ExpandFlags;
use crate::types::{ExpandContext, Failed};
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

/// The bare function type behind [`CompleteListItemGetter`], for the one
/// place that compares a generator against a particular function.
pub(crate) type ItemGetter = unsafe fn(*mut expand_T, c_int) -> *mut c_char;

/// The `WILD_*` options that name an `EW_*` flag one-for-one.
const WILDOPT_TO_EW: [(WildOpts, ExpandFlags); 6] = [
    (WildOpts::LIST_NOTFOUND, ExpandFlags::NOTFOUND),
    (WildOpts::ADD_SLASH, ExpandFlags::ADDSLASH),
    (WildOpts::KEEP_ALL, ExpandFlags::KEEPALL),
    (WildOpts::SILENT, ExpandFlags::SILENT),
    (WildOpts::NOERROR, ExpandFlags::NOERROR),
    (WildOpts::ALLLINKS, ExpandFlags::ALLLINKS),
];

/// Translate the `WILD_*` options into the `EW_*` flags `expand_wildcards`
/// takes.  `ExpandFlags::DIR` — include directories — is always on.
pub(crate) fn map_wildopts_to_ewflags(options: WildOpts) -> ExpandFlags {
    WILDOPT_TO_EW
        .iter()
        .fold(ExpandFlags::DIR, |flags, &(wild, ew)| {
            if options.has(wild) { flags | ew } else { flags }
        })
}

/// A runtime-directory completion that wants nothing special: `'runtimepath'`
/// as it stands, no package trees and no `after/` filter. Upstream's bare `0`.
const RTP_ONLY: RuntimeOpts = RuntimeOpts::NONE;

/// Do the expansion based on `xp->xp_context` and `pat`.
///
/// `options` is a set of `WILD_*` flags.  Most contexts have a generator of
/// their own; the ones that do not fall through to [`expand_other`]'s table,
/// and all of those run against a compiled regexp (or, under
/// `'wildoptions'`=fuzzy, against `fuzzy_match_str`).
pub(crate) unsafe fn expand_from_context(
    xp: *mut expand_T,
    pat: *mut c_char,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    options: WildOpts,
) -> Result<(), Failed> {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let mut pat = pat;
    let flags = map_wildopts_to_ewflags(options);
    let fuzzy = unsafe { cmdline_fuzzy_complete(pat) }
        && unsafe { cmdline_fuzzy_completion_supported(xp.raw()) };
    let context = xp.xp_context;

    if matches!(
        context,
        ExpandContext::Files
            | ExpandContext::Directories
            | ExpandContext::FilesInPath
            | ExpandContext::Findfunc
            | ExpandContext::DirsInCdpath
    ) {
        return unsafe {
            expand_files_and_dirs(xp.raw(), pat, matches, numMatches, flags, options)
        };
    }

    unsafe { *matches = ptr::null_mut() };
    unsafe { *numMatches = 0 };

    // The contexts with a generator of their own.  Each `expand_runtime_dir`
    // arm builds the NULL-terminated `char *[]` it wants in this frame.
    match context {
        ExpandContext::Help => {
            // With an empty argument we would get all the help tags,
            // which is very slow.  Get matches for "help" instead.
            let arg = if unsafe { *pat } == 0 {
                c"help".as_ptr()
            } else {
                pat as *const c_char
            };
            if unsafe { find_help_tags(arg, numMatches, matches, false) }.is_err() {
                return Err(Failed);
            }
            unsafe { cleanup_help_tags(*numMatches, *matches) };
            return Ok(());
        }
        ExpandContext::ShellCmd => {
            unsafe { expand_shellcmd(pat, matches, numMatches, flags) };
            return Ok(());
        }
        ExpandContext::OldSetting => return unsafe { expand_old_setting(numMatches, matches) },
        ExpandContext::Buffers => {
            return unsafe { expand_buf_names(pat, numMatches, matches, options) };
        }
        ExpandContext::DiffBuffers => {
            return unsafe {
                expand_buf_names(pat, numMatches, matches, options | BUF_DIFF_FILTER)
            };
        }
        ExpandContext::Tags | ExpandContext::TagsListFiles => {
            return unsafe {
                expand_tags(context == ExpandContext::Tags, pat, numMatches, matches)
            };
        }
        ExpandContext::Colors => {
            let mut dirs = [c"colors".as_ptr() as *mut c_char, ptr::null_mut()];
            return unsafe {
                expand_runtime_dir(
                    pat,
                    RuntimeOpts::START | RuntimeOpts::OPT,
                    numMatches,
                    matches,
                    dirs.as_mut_ptr(),
                )
            };
        }
        ExpandContext::Compiler => {
            let mut dirs = [c"compiler".as_ptr() as *mut c_char, ptr::null_mut()];
            return unsafe {
                expand_runtime_dir(pat, RTP_ONLY, numMatches, matches, dirs.as_mut_ptr())
            };
        }
        ExpandContext::Ownsyntax => {
            let mut dirs = [c"syntax".as_ptr() as *mut c_char, ptr::null_mut()];
            return unsafe {
                expand_runtime_dir(pat, RTP_ONLY, numMatches, matches, dirs.as_mut_ptr())
            };
        }
        ExpandContext::Filetype => {
            let mut dirs = [
                c"syntax".as_ptr() as *mut c_char,
                c"indent".as_ptr() as *mut c_char,
                c"ftplugin".as_ptr() as *mut c_char,
                ptr::null_mut(),
            ];
            return unsafe {
                expand_runtime_dir(pat, RTP_ONLY, numMatches, matches, dirs.as_mut_ptr())
            };
        }
        ExpandContext::Keymap => {
            let mut dirs = [c"keymap".as_ptr() as *mut c_char, ptr::null_mut()];
            return unsafe {
                expand_runtime_dir(pat, RTP_ONLY, numMatches, matches, dirs.as_mut_ptr())
            };
        }
        ExpandContext::UserList => {
            return unsafe { expand_user_list(xp.raw(), matches, numMatches) };
        }
        ExpandContext::UserLua => return unsafe { expand_user_lua(xp.raw(), numMatches, matches) },
        ExpandContext::Packadd => return unsafe { expand_packadd_dir(pat, numMatches, matches) },
        ExpandContext::Runtime => return unsafe { expand_runtime_cmd(pat, numMatches, matches) },
        ExpandContext::PatternInBuf => {
            return unsafe { expand_pattern_in_buf(pat, xp.xp_search_dir, matches, numMatches) };
        }
        _ => {}
    }

    // When expanding a function name starting with s:, match the <SNR>nr_
    // prefix.
    let mut tofree = ptr::null_mut::<c_char>();
    if context == ExpandContext::UserFunc && unsafe { strncmp(pat, c"^s:".as_ptr(), 3) } == 0 {
        let len = unsafe { strlen(pat) } + 20;
        tofree = unsafe { xmalloc(len) } as *mut c_char;
        unsafe { snprintf(tofree, len, c"^<SNR>\\d\\+_%s".as_ptr(), pat.add(3)) };
        pat = tofree;
    }

    if context == ExpandContext::Lua {
        // `tofree` is still NULL here: only ExpandContext::UserFunc sets it.
        return unsafe { nlua_expand_get_matches(numMatches, matches) };
    }

    let mut regmatch = regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    if !fuzzy {
        regmatch.regprog = unsafe { vim_regcomp(pat, if magic_isset() { RE_MAGIC } else { 0 }) };
        if regmatch.regprog.is_null() {
            unsafe { xfree(tofree as *mut c_void) };
            return Err(Failed);
        }
        // Set ignore-case according to 'ignorecase', 'smartcase' and pat.
        regmatch.rm_ic = unsafe { ignorecase(pat) } != 0;
    }

    let ret = match context {
        ExpandContext::Settings | ExpandContext::BoolSettings => unsafe {
            expand_settings(xp.raw(), &raw mut regmatch, pat, numMatches, matches, fuzzy)
        },
        ExpandContext::StringSetting => unsafe {
            expand_string_setting(xp.raw(), &raw mut regmatch, numMatches, matches)
        },
        ExpandContext::SettingSubtract => unsafe {
            expand_setting_subtract(xp.raw(), &raw mut regmatch, numMatches, matches)
        },
        ExpandContext::Mappings => unsafe {
            expand_mappings(pat, &raw mut regmatch, numMatches, matches)
        },
        ExpandContext::Argopt => unsafe {
            expand_argopt(pat, xp.raw(), &raw mut regmatch, matches, numMatches)
        },
        ExpandContext::UserDefined => unsafe {
            expand_user_defined(pat, xp.raw(), &raw mut regmatch, matches, numMatches)
        },
        _ => unsafe { expand_other(pat, xp.raw(), &raw mut regmatch, matches, numMatches) },
    };

    if !fuzzy {
        unsafe { vim_regfree(regmatch.regprog) };
    }
    unsafe { xfree(tofree as *mut c_void) };
    ret
}

/// Expand a list of names.
///
/// The generic command-line completion loop: `func` is called with rising
/// indices until it answers NULL, each string is matched against `regmatch`
/// (or scored by `fuzzy_match_str`), and the survivors are copied into a new
/// array.
///
/// `escaped` asks for spaces, tabs, backslashes and dots to be escaped in
/// each match.
pub unsafe fn expand_generic(
    pat: *const c_char,
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    func: CompleteListItemGetter,
    escaped: bool,
) {
    // SAFETY: the caller's contract -- `xp` is the live expansion
    // context, which outlives this call.
    let mut xp = unsafe { Xp::new(xp) };
    let get_item = func.expect("expand_generic needs a generator");
    let fuzzy = unsafe { cmdline_fuzzy_complete(pat) };
    unsafe { *matches = ptr::null_mut() };
    unsafe { *numMatches = 0 };

    let mut ga = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    let itemsize = if fuzzy {
        size_of::<fuzmatch_str_T>()
    } else {
        size_of::<*mut c_char>()
    };
    unsafe { ga_init(&raw mut ga, itemsize as c_int, 30) };

    for i in 0.. {
        let mut str = unsafe { get_item(xp.raw(), i) };
        if str.is_null() {
            break; // end of list
        }
        if unsafe { *str } == 0 {
            continue; // skip empty strings
        }

        // An empty pattern matches everything; otherwise every
        // candidate is tested, and under 'wildoptions'=fuzzy also scored.
        // `xp_pattern` is re-read each pass, as upstream does: the
        // generator is handed `xp` and a user-defined one can move it.
        let mut score = 0;
        let matched = if unsafe { *xp.xp_pattern } == 0 {
            true
        } else if fuzzy {
            score = unsafe { fuzzy_match_str(str, pat) };
            score != FUZZY_SCORE_NONE
        } else {
            unsafe { vim_regexec(regmatch, str, 0) }
        };
        if !matched {
            continue;
        }

        str = if escaped {
            unsafe { vim_strsave_escaped(str, c" \t\\.".as_ptr()) }
        } else {
            unsafe { xstrdup(str) }
        };

        unsafe { ga_grow(&raw mut ga, 1) };
        if fuzzy {
            let scored = fuzmatch_str_T {
                idx: ga.ga_len,
                str,
                score,
            };
            let slot = (ga.ga_data as *mut fuzmatch_str_T).wrapping_offset(ga.ga_len as isize);
            // SAFETY: `ga_grow` above made room for one more entry.
            unsafe { slot.write(scored) };
        } else {
            unsafe {
                (ga.ga_data as *mut *mut c_char)
                    .offset(ga.ga_len as isize)
                    .write(str)
            };
        }
        ga.ga_len += 1;

        if ptr::fn_addr_eq(get_item, get_menu_names as ItemGetter) {
            // Undo the separator get_menu_names() added, in the copy that
            // is now in the array.
            let last = unsafe { str.add(strlen(str) - 1) };
            if unsafe { *last } == 1 {
                unsafe { *last = b'.' as c_char };
            }
        }
    }

    if ga.ga_len == 0 {
        return;
    }

    // Sort the matches when using regular expression matching and sorting
    // applies to the completion context.  Menus and scriptnames should be
    // kept in the order they were given in.
    let sort_matches = !fuzzy
        && !matches!(
            xp.xp_context,
            ExpandContext::Menunames
                | ExpandContext::StringSetting
                | ExpandContext::Menus
                | ExpandContext::Scriptnames
                | ExpandContext::Argopt
        );
    // <SNR> functions should be sorted to the end.
    let funcsort = matches!(
        xp.xp_context,
        ExpandContext::Expression | ExpandContext::Functions | ExpandContext::UserFunc
    );

    if sort_matches {
        if funcsort {
            unsafe {
                qsort(
                    ga.ga_data,
                    ga.ga_len as size_t,
                    size_of::<*mut c_char>(),
                    Some(sort_func_compare),
                )
            };
        } else {
            unsafe { sort_strings(ga.ga_data as *mut *mut c_char, ga.ga_len) };
        }
    }

    if fuzzy {
        unsafe {
            fuzzymatches_to_strmatches(
                ga.ga_data as *mut fuzmatch_str_T,
                matches,
                ga.ga_len,
                funcsort,
            )
        };
    } else {
        unsafe { *matches = ga.ga_data as *mut *mut c_char };
    }
    unsafe { *numMatches = ga.ga_len };

    // Reset the variables used for special highlight names expansion, so
    // that they don't show up when getting normal highlight names by ID.
    unsafe { reset_expand_highlight() };
}
