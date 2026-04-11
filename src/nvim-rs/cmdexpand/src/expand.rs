//! Context-based expansion dispatch for command-line completion.
//!
//! Implements `ExpandFromContext` (main dispatch) and `ExpandOther` (table-driven
//! callback dispatch), migrated from `cmdexpand.c`.

use libc::{c_char, c_int, c_void};

use crate::{ExpandHandle, ExpandT};

// =============================================================================
// regmatch_T repr(C) struct (matches C layout exactly)
// =============================================================================

const NSUBEXP: usize = 10;

/// Structure matching `regmatch_T` for single-line matching.
#[repr(C)]
pub struct RegMatch {
    regprog: *mut c_void,
    startp: [*mut c_char; NSUBEXP],
    endp: [*mut c_char; NSUBEXP],
    rm_matchcol: c_int,
    rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: std::ptr::null_mut(),
            startp: [std::ptr::null_mut(); NSUBEXP],
            endp: [std::ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

// =============================================================================
// fuzmatch_str_T repr(C) struct
// =============================================================================

/// Matches `fuzmatch_str_T` from `fuzzy.h`.
/// Layout: `idx:i32@0`, `_pad:i32@4`, `str:*mut c_char@8`, `score:i32@16`, `_pad2:i32@20` = 24 bytes.
#[repr(C)]
struct FuzmatchStr {
    idx: c_int,
    _pad: c_int,
    str_: *mut c_char,
    score: c_int,
    _pad2: c_int,
}

// =============================================================================
// CompleteListItemGetter function pointer type
// =============================================================================

/// Type alias for `CompleteListItemGetter`: `char *(*)(expand_T *, int)`.
pub type CompleteListItemGetter =
    Option<unsafe extern "C" fn(xp: ExpandHandle, idx: c_int) -> *mut c_char>;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Regex (direct C functions)
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regfree(prog: *mut c_void);
    fn ignorecase(pat: *const c_char) -> c_int;
    fn nvim_cmdexpand_regmatch_set_rm_ic(rmp: *mut RegMatch, val: c_int);
    fn nvim_cmdexpand_regmatch_set_regprog(rmp: *mut RegMatch, prog: *mut c_void);
    fn nvim_cmdexpand_get_re_magic() -> c_int;

    // ExpandGeneric dispatch
    fn nvim_cmdexpand_expand_generic(
        pat: *const c_char,
        xp: *mut ExpandT,
        regmatch: *mut RegMatch,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
        func: CompleteListItemGetter,
        escaped: c_int,
    );

    // Individual expanders
    fn find_help_tags(
        arg: *const c_char,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
        keep_lang: bool,
    ) -> c_int;
    fn cleanup_help_tags(num_file: c_int, file: *mut *mut c_char);
    fn expand_shellcmd(
        filepat: *mut c_char,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
        flags: c_int,
    );
    fn ExpandOldSetting(num_matches: *mut c_int, matches: *mut *mut *mut c_char) -> c_int;
    fn rs_ExpandBufnames(
        pat: *mut c_char,
        num_file: *mut c_int,
        file: *mut *mut *mut c_char,
        options: c_int,
    ) -> c_int;
    fn nvim_cmdexpand_expand_rtdir(
        pat: *const c_char,
        flags: c_int,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
        directories: *mut *mut c_char,
    ) -> c_int;
    fn nvim_cmdexpand_expand_pack_add_dir(
        pat: *const c_char,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_cmdexpand_expand_runtime_cmd(
        pat: *const c_char,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn ExpandSettings(
        xp: *mut ExpandT,
        regmatch: *mut RegMatch,
        fuzzystr: *mut c_char,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
        can_fuzzy: bool,
    ) -> c_int;
    fn nvim_cmdexpand_expand_string_setting(
        xp: *mut ExpandT,
        regmatch: *mut RegMatch,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_cmdexpand_expand_mappings(
        pat: *const c_char,
        regmatch: *mut RegMatch,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_cmdexpand_expand_argopt(
        pat: *const c_char,
        xp: *mut ExpandT,
        regmatch: *mut RegMatch,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
    ) -> c_int;
    // nvim_cmdexpand_expand_user_defined -- replaced by crate::shell::rs_expand_user_defined
    // nvim_cmdexpand_expand_user_list -- replaced by crate::shell::rs_expand_user_list
    // nvim_cmdexpand_expand_user_lua -- replaced by crate::shell::rs_expand_user_lua
    fn nvim_cmdexpand_nlua_expand_get_matches(
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn nvim_cmdexpand_get_dip_start_opt() -> c_int;
    fn nvim_cmdexpand_magic_isset() -> c_int;

    // Function pointer accessors for ExpandOther dispatch table
    fn nvim_cmdexpand_get_fn_get_command_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_history_arg() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_commands() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_cmd_addr_type() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_cmd_flags() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_cmd_nargs() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_cmd_complete() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_var_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_function_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_user_func_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_expr_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_menu_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_menu_names() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_syntax_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_syntime_arg() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_highlight_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_expand_get_event_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_expand_get_augroup_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_sign_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_profile_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_lang_arg() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_locales() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_env_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_users() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_arglist_name() -> CompleteListItemGetter;
    fn nvim_cmdexpand_get_fn_get_healthcheck_names() -> CompleteListItemGetter;

    // Other Rust functions (called via C ABI)
    fn rs_expand_files_and_dirs(
        xp: *mut ExpandT,
        pat: *mut c_char,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
        flags: c_int,
        options: c_int,
    ) -> c_int;
    fn rs_expand_tags(
        tagnames: c_int,
        pat: *mut c_char,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn rs_expand_pattern_in_buf(
        pat: *const c_char,
        search_dir: c_int,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
    ) -> c_int;
    fn rs_expand_setting_subtract(
        xp: *mut ExpandT,
        regmatch: *mut RegMatch,
        num_matches: *mut c_int,
        matches: *mut *mut *mut c_char,
    ) -> c_int;
    fn rs_map_wildopts_to_ewflags(options: c_int) -> c_int;
    fn rs_get_filetypecmd_arg(xp: ExpandHandle, idx: c_int) -> *mut c_char;
    fn rs_get_mapclear_arg(xp: ExpandHandle, idx: c_int) -> *mut c_char;
    fn rs_get_messages_arg(xp: ExpandHandle, idx: c_int) -> *mut c_char;
    fn rs_get_breakadd_arg(xp: ExpandHandle, idx: c_int) -> *mut c_char;
    fn rs_get_scriptnames_arg(xp: ExpandHandle, idx: c_int) -> *mut c_char;
    fn rs_get_retab_arg(xp: ExpandHandle, idx: c_int) -> *mut c_char;
    fn rs_cmdline_fuzzy_completion_supported(context: c_int) -> c_int;

    // ExpandGeneric helpers
    fn vim_regexec(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> bool;
    fn fuzzy_match_str(str_: *mut c_char, pat: *const c_char) -> c_int;
    fn fuzzymatches_to_strmatches(
        fuzmatch: *mut FuzmatchStr,
        matches: *mut *mut *mut c_char,
        count: c_int,
        funcsort: bool,
    );
    fn sort_strings(files: *mut *mut c_char, count: c_int);
    fn vim_strsave_escaped(s: *const c_char, esc: *const c_char) -> *mut c_char;
    fn reset_expand_highlight();
    fn xstrdup(s: *const c_char) -> *mut c_char;

    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// Constants
// =============================================================================

use crate::context::ExpandContext;

const OK: c_int = 1;
const FAIL: c_int = 0;
const BUF_DIFF_FILTER: c_int = 0x2000;

// FUZZY_SCORE_NONE = INT_MIN
const FUZZY_SCORE_NONE: c_int = c_int::MIN;

// =============================================================================
// ExpandGeneric
// =============================================================================

/// Expand a list of names by calling `func` for each index.
///
/// Generic function for command line completion. Iterates `func(xp, i)`,
/// matches against `regmatch` (or fuzzy pattern `pat`), and collects results
/// into `matches`/`numMatches`. Mirrors `ExpandGeneric` in `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid. `func` must be a valid function pointer.
#[unsafe(export_name = "ExpandGeneric")]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_expand_generic(
    pat: *const c_char,
    xp: *mut ExpandT,
    regmatch: *mut RegMatch,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
    func: CompleteListItemGetter,
    escaped: bool,
) {
    let pat_str = std::ffi::CStr::from_ptr(pat).to_bytes();
    let pat_rust = std::str::from_utf8_unchecked(pat_str);
    let fuzzy = crate::cmdline_fuzzy_complete(pat_rust);

    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let mut str_matches: Vec<*mut c_char> = Vec::new();
    let mut fuz_matches: Vec<FuzmatchStr> = Vec::new();

    let get_menu_names_fn = nvim_cmdexpand_get_fn_get_menu_names();

    let mut i: c_int = 0;
    loop {
        let Some(func_fn) = func else {
            break;
        };
        let str_ = func_fn(xp, i);
        i += 1;
        if str_.is_null() {
            break;
        }
        if *str_ == 0 {
            continue;
        }

        let mut score: c_int = 0;
        let xp_pattern = (*xp).xp_pattern;
        let is_match = if !xp_pattern.is_null() && *xp_pattern != 0 {
            if fuzzy {
                score = fuzzy_match_str(str_, pat);
                score != FUZZY_SCORE_NONE
            } else {
                vim_regexec(regmatch, str_, 0)
            }
        } else {
            true
        };

        if !is_match {
            continue;
        }

        let owned_str = if escaped {
            vim_strsave_escaped(str_, c" \t\\.".as_ptr())
        } else {
            xstrdup(str_)
        };

        if fuzzy {
            fuz_matches.push(FuzmatchStr {
                idx: fuz_matches.len() as c_int,
                _pad: 0,
                str_: owned_str,
                score,
                _pad2: 0,
            });
        } else {
            str_matches.push(owned_str);
        }

        // Test for separator added by get_menu_names(): change '\001' to '.'
        if let Some(gmn) = get_menu_names_fn {
            if func_fn as usize == gmn as usize {
                let len = libc::strlen(owned_str);
                if len > 0 {
                    let last = owned_str.add(len - 1);
                    if *last == 0x01 {
                        *last = b'.' as c_char;
                    }
                }
            }
        }
    }

    let count = if fuzzy {
        fuz_matches.len()
    } else {
        str_matches.len()
    };

    if count == 0 {
        return;
    }

    let ctx = (*xp).xp_context;

    let sort_matches = !fuzzy
        && ctx != ExpandContext::Menunames.to_raw()
        && ctx != ExpandContext::StringSetting.to_raw()
        && ctx != ExpandContext::Menus.to_raw()
        && ctx != ExpandContext::Scriptnames.to_raw()
        && ctx != ExpandContext::Argopt.to_raw();

    let funcsort = ctx == ExpandContext::Expression.to_raw()
        || ctx == ExpandContext::Functions.to_raw()
        || ctx == ExpandContext::UserFunc.to_raw();

    if fuzzy {
        fuzzymatches_to_strmatches(
            fuz_matches.as_mut_ptr(),
            matches,
            fuz_matches.len() as c_int,
            funcsort,
        );
        *num_matches = fuz_matches.len() as c_int;
        std::mem::forget(fuz_matches);
    } else {
        if sort_matches {
            if funcsort {
                str_matches.sort_unstable_by(|a, b| {
                    let r = rs_sort_func_compare(
                        std::ptr::from_ref(a).cast::<c_void>(),
                        std::ptr::from_ref(b).cast::<c_void>(),
                    );
                    r.cmp(&0)
                });
            } else {
                sort_strings(str_matches.as_mut_ptr(), str_matches.len() as c_int);
            }
        }

        // Transfer into a C-heap array compatible with xfree.
        // SAFETY: xmalloc and libc malloc share the same allocator on Linux;
        // the result can be freed with xfree by the caller.
        let len = str_matches.len();
        let arr: *mut *mut c_char =
            libc::malloc(len * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
        for (j, p) in str_matches.iter().enumerate() {
            *arr.add(j) = *p;
        }
        std::mem::forget(str_matches);
        *matches = arr;
        *num_matches = len as c_int;
    }

    reset_expand_highlight();
}

extern "C" {
    fn rs_sort_func_compare(s1: *const c_void, s2: *const c_void) -> c_int;
}

// =============================================================================
// ExpandOther
// =============================================================================

/// Entry in the `ExpandOther` dispatch table.
struct ExpGen {
    context: c_int,
    func: CompleteListItemGetter,
    ic: bool,
    escaped: bool,
}

/// Do the expansion based on `xp->xp_context` using the table-driven dispatch.
///
/// Matches `ExpandOther` in `cmdexpand.c`.
///
/// # Safety
///
/// `pat`, `xp`, `rmp`, `matches`, and `num_matches` must all be valid pointers.
#[allow(clippy::too_many_lines)]
pub unsafe fn expand_other(
    pat: *const c_char,
    xp: *mut ExpandT,
    rmp: *mut RegMatch,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    let ctx = (*xp).xp_context;

    // Build the dispatch table inline (all function pointers obtained from C)
    let table = [
        ExpGen {
            context: ExpandContext::Commands.to_raw(),
            func: nvim_cmdexpand_get_fn_get_command_name(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Filetypecmd.to_raw(),
            func: Some(rs_get_filetypecmd_arg),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Mapclear.to_raw(),
            func: Some(rs_get_mapclear_arg),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Messages.to_raw(),
            func: Some(rs_get_messages_arg),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::History.to_raw(),
            func: nvim_cmdexpand_get_fn_get_history_arg(),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserCommands.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_commands(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserAddrType.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_cmd_addr_type(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserCmdFlags.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_cmd_flags(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserNargs.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_cmd_nargs(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserComplete.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_cmd_complete(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserVars.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_var_name(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Functions.to_raw(),
            func: nvim_cmdexpand_get_fn_get_function_name(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::UserFunc.to_raw(),
            func: nvim_cmdexpand_get_fn_get_user_func_name(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Expression.to_raw(),
            func: nvim_cmdexpand_get_fn_get_expr_name(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Menus.to_raw(),
            func: nvim_cmdexpand_get_fn_get_menu_name(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Menunames.to_raw(),
            func: nvim_cmdexpand_get_fn_get_menu_names(),
            ic: false,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Syntax.to_raw(),
            func: nvim_cmdexpand_get_fn_get_syntax_name(),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Syntime.to_raw(),
            func: nvim_cmdexpand_get_fn_get_syntime_arg(),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Highlight.to_raw(),
            func: nvim_cmdexpand_get_fn_get_highlight_name(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Events.to_raw(),
            func: nvim_cmdexpand_get_fn_expand_get_event_name(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Augroup.to_raw(),
            func: nvim_cmdexpand_get_fn_expand_get_augroup_name(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Sign.to_raw(),
            func: nvim_cmdexpand_get_fn_get_sign_name(),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Profile.to_raw(),
            func: nvim_cmdexpand_get_fn_get_profile_name(),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Language.to_raw(),
            func: nvim_cmdexpand_get_fn_get_lang_arg(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Locales.to_raw(),
            func: nvim_cmdexpand_get_fn_get_locales(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::EnvVars.to_raw(),
            func: nvim_cmdexpand_get_fn_get_env_name(),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::User.to_raw(),
            func: nvim_cmdexpand_get_fn_get_users(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Arglist.to_raw(),
            func: nvim_cmdexpand_get_fn_get_arglist_name(),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Breakpoint.to_raw(),
            func: Some(rs_get_breakadd_arg),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Scriptnames.to_raw(),
            func: Some(rs_get_scriptnames_arg),
            ic: true,
            escaped: false,
        },
        ExpGen {
            context: ExpandContext::Retab.to_raw(),
            func: Some(rs_get_retab_arg),
            ic: true,
            escaped: true,
        },
        ExpGen {
            context: ExpandContext::Checkhealth.to_raw(),
            func: nvim_cmdexpand_get_fn_get_healthcheck_names(),
            ic: true,
            escaped: false,
        },
    ];

    // Find a context in the table and call ExpandGeneric with the right function.
    for entry in &table {
        if ctx == entry.context {
            if entry.ic {
                nvim_cmdexpand_regmatch_set_rm_ic(rmp, 1);
            }
            nvim_cmdexpand_expand_generic(
                pat,
                xp,
                rmp,
                matches,
                num_matches,
                entry.func,
                c_int::from(entry.escaped),
            );
            return OK;
        }
    }

    FAIL
}

// =============================================================================
// ExpandFromContext
// =============================================================================

/// Do the expansion based on `xp->xp_context` and `pat`.
///
/// Mirrors `ExpandFromContext` in `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid. `xp` must be properly initialized.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "ExpandFromContext")]
pub unsafe extern "C" fn rs_expand_from_context(
    xp: *mut ExpandT,
    pat: *mut c_char,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
    options: c_int,
) -> c_int {
    let mut regmatch = RegMatch::default();
    let ctx = (*xp).xp_context;
    let flags = rs_map_wildopts_to_ewflags(options);
    let pat_str = std::ffi::CStr::from_ptr(pat).to_bytes();
    let pat_rust = std::str::from_utf8_unchecked(pat_str);
    let fuzzy =
        crate::cmdline_fuzzy_complete(pat_rust) && rs_cmdline_fuzzy_completion_supported(ctx) != 0;

    if ctx == ExpandContext::Files.to_raw()
        || ctx == ExpandContext::Directories.to_raw()
        || ctx == ExpandContext::FilesInPath.to_raw()
        || ctx == ExpandContext::Findfunc.to_raw()
        || ctx == ExpandContext::DirsInCdpath.to_raw()
    {
        return rs_expand_files_and_dirs(xp, pat, matches, num_matches, flags, options);
    }

    *matches = std::ptr::null_mut();
    *num_matches = 0;

    if ctx == ExpandContext::Help.to_raw() {
        let help_pat = if pat.is_null() || *pat == 0 {
            c"help".as_ptr()
        } else {
            pat.cast_const()
        };
        let ret = find_help_tags(help_pat, num_matches, matches, false);
        if ret == OK {
            cleanup_help_tags(*num_matches, *matches);
            return 1;
        }
        return 0;
    }

    if ctx == ExpandContext::Shellcmd.to_raw() {
        expand_shellcmd(pat, matches, num_matches, flags);
        return OK;
    }
    if ctx == ExpandContext::OldSetting.to_raw() {
        return ExpandOldSetting(num_matches, matches);
    }
    if ctx == ExpandContext::Buffers.to_raw() {
        return rs_ExpandBufnames(pat, num_matches, matches, options);
    }
    if ctx == ExpandContext::DiffBuffers.to_raw() {
        return rs_ExpandBufnames(pat, num_matches, matches, options | BUF_DIFF_FILTER);
    }
    if ctx == ExpandContext::Tags.to_raw() || ctx == ExpandContext::TagsListfiles.to_raw() {
        return rs_expand_tags(
            c_int::from(ctx == ExpandContext::Tags.to_raw()),
            pat,
            num_matches,
            matches,
        );
    }
    if ctx == ExpandContext::Colors.to_raw() {
        let mut dirs: [*mut c_char; 2] = [c"colors".as_ptr().cast_mut(), std::ptr::null_mut()];
        return nvim_cmdexpand_expand_rtdir(
            pat,
            nvim_cmdexpand_get_dip_start_opt(),
            num_matches,
            matches,
            dirs.as_mut_ptr(),
        );
    }
    if ctx == ExpandContext::Compiler.to_raw() {
        let mut dirs: [*mut c_char; 2] = [c"compiler".as_ptr().cast_mut(), std::ptr::null_mut()];
        return nvim_cmdexpand_expand_rtdir(pat, 0, num_matches, matches, dirs.as_mut_ptr());
    }
    if ctx == ExpandContext::Ownsyntax.to_raw() {
        let mut dirs: [*mut c_char; 2] = [c"syntax".as_ptr().cast_mut(), std::ptr::null_mut()];
        return nvim_cmdexpand_expand_rtdir(pat, 0, num_matches, matches, dirs.as_mut_ptr());
    }
    if ctx == ExpandContext::Filetype.to_raw() {
        let mut dirs: [*mut c_char; 4] = [
            c"syntax".as_ptr().cast_mut(),
            c"indent".as_ptr().cast_mut(),
            c"ftplugin".as_ptr().cast_mut(),
            std::ptr::null_mut(),
        ];
        return nvim_cmdexpand_expand_rtdir(pat, 0, num_matches, matches, dirs.as_mut_ptr());
    }
    if ctx == ExpandContext::Keymap.to_raw() {
        let mut dirs: [*mut c_char; 2] = [c"keymap".as_ptr().cast_mut(), std::ptr::null_mut()];
        return nvim_cmdexpand_expand_rtdir(pat, 0, num_matches, matches, dirs.as_mut_ptr());
    }
    if ctx == ExpandContext::UserList.to_raw() {
        return crate::shell::rs_expand_user_list(xp, matches, num_matches);
    }
    if ctx == ExpandContext::UserLua.to_raw() {
        return crate::shell::rs_expand_user_lua(xp, num_matches, matches);
    }
    if ctx == ExpandContext::Packadd.to_raw() {
        return nvim_cmdexpand_expand_pack_add_dir(pat, num_matches, matches);
    }
    if ctx == ExpandContext::Runtime.to_raw() {
        return nvim_cmdexpand_expand_runtime_cmd(pat, num_matches, matches);
    }
    if ctx == ExpandContext::PatternInBuf.to_raw() {
        return rs_expand_pattern_in_buf(pat, (*xp).xp_search_dir, matches, num_matches);
    }

    // When expanding a function name starting with s:, match the <SNR>nr_ prefix.
    let mut tofree: *mut c_char = std::ptr::null_mut();
    let effective_pat = if ctx == ExpandContext::UserFunc.to_raw() {
        let slice = std::ffi::CStr::from_ptr(pat).to_bytes();
        if slice.starts_with(b"^s:") {
            // Build "^<SNR>\d\+_<suffix>" pattern. pat+3 skips the "^s:" prefix.
            let suffix = std::ffi::CStr::from_ptr(pat.add(3));
            let suffix_str = suffix.to_string_lossy();
            let pattern = format!("^<SNR>\\d\\+_{suffix_str}\0");
            let boxed = pattern.into_bytes().into_boxed_slice();
            tofree = Box::into_raw(boxed).cast::<c_char>();
            tofree
        } else {
            pat
        }
    } else {
        pat
    };

    if ctx == ExpandContext::Lua.to_raw() {
        let ret = nvim_cmdexpand_nlua_expand_get_matches(num_matches, matches);
        xfree(tofree.cast());
        return ret;
    }

    if !fuzzy {
        let prog = vim_regcomp(
            effective_pat,
            if nvim_cmdexpand_magic_isset() != 0 {
                nvim_cmdexpand_get_re_magic()
            } else {
                0
            },
        );
        if prog.is_null() {
            xfree(tofree.cast());
            return FAIL;
        }
        nvim_cmdexpand_regmatch_set_regprog(&raw mut regmatch, prog);
        let ic = ignorecase(effective_pat);
        nvim_cmdexpand_regmatch_set_rm_ic(&raw mut regmatch, ic);
    }

    let ret =
        if ctx == ExpandContext::Settings.to_raw() || ctx == ExpandContext::BoolSettings.to_raw() {
            ExpandSettings(
                xp,
                &raw mut regmatch,
                effective_pat,
                num_matches,
                matches,
                fuzzy,
            )
        } else if ctx == ExpandContext::StringSetting.to_raw() {
            nvim_cmdexpand_expand_string_setting(xp, &raw mut regmatch, num_matches, matches)
        } else if ctx == ExpandContext::SettingSubtract.to_raw() {
            rs_expand_setting_subtract(xp, &raw mut regmatch, num_matches, matches)
        } else if ctx == ExpandContext::Mappings.to_raw() {
            nvim_cmdexpand_expand_mappings(effective_pat, &raw mut regmatch, num_matches, matches)
        } else if ctx == ExpandContext::Argopt.to_raw() {
            nvim_cmdexpand_expand_argopt(effective_pat, xp, &raw mut regmatch, matches, num_matches)
        } else if ctx == ExpandContext::UserDefined.to_raw() {
            crate::shell::rs_expand_user_defined(
                effective_pat,
                xp,
                &raw mut regmatch,
                matches,
                num_matches,
            )
        } else {
            expand_other(effective_pat, xp, &raw mut regmatch, matches, num_matches)
        };

    if !fuzzy {
        vim_regfree(regmatch.regprog);
    }
    xfree(tofree.cast());

    ret
}
