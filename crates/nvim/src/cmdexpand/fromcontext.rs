//! Turning a context into a match array.
//!
//! [`ExpandFromContext`] is the dispatcher: file-like contexts go to
//! `expand_wildcards`, everything else to a generator, and the answer is
//! sorted, deduplicated and escaped.  [`ExpandGeneric`] is the generic
//! generator loop every `get_*_name` callback is driven by, and
//! [`map_wildopts_to_ewflags`] translates `'wildoptions'` into `EW_*`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cmdexpand::WildOpts;
use crate::path::ExpandFlags;
use crate::types::{FAIL, OK};
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

/// Do the expansion based on `xp->xp_context` and `pat`.
///
/// `options` is a set of `WILD_*` flags.  Most contexts have a generator of
/// their own; the ones that do not fall through to [`ExpandOther`]'s table,
/// and all of those run against a compiled regexp (or, under
/// `'wildoptions'`=fuzzy, against `fuzzy_match_str`).
pub(crate) unsafe fn ExpandFromContext(
    xp: *mut expand_T,
    pat: *mut c_char,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    options: WildOpts,
) -> c_int {
    unsafe {
        let mut pat = pat;
        let flags = map_wildopts_to_ewflags(options);
        let fuzzy = cmdline_fuzzy_complete(pat) && cmdline_fuzzy_completion_supported(xp);
        let context = (*xp).xp_context;

        if matches!(
            context,
            EXPAND_FILES
                | EXPAND_DIRECTORIES
                | EXPAND_FILES_IN_PATH
                | EXPAND_FINDFUNC
                | EXPAND_DIRS_IN_CDPATH
        ) {
            return expand_files_and_dirs(xp, pat, matches, numMatches, flags, options);
        }

        *matches = ptr::null_mut();
        *numMatches = 0;

        // The contexts with a generator of their own.  Each `ExpandRTDir`
        // arm builds the NULL-terminated `char *[]` it wants in this frame.
        match context {
            EXPAND_HELP => {
                // With an empty argument we would get all the help tags,
                // which is very slow.  Get matches for "help" instead.
                let arg = if *pat == 0 {
                    c"help".as_ptr()
                } else {
                    pat as *const c_char
                };
                if find_help_tags(arg, numMatches, matches, false) != OK {
                    return FAIL;
                }
                cleanup_help_tags(*numMatches, *matches);
                return OK;
            }
            EXPAND_SHELLCMD => {
                expand_shellcmd(pat, matches, numMatches, flags);
                return OK;
            }
            EXPAND_OLD_SETTING => return ExpandOldSetting(numMatches, matches),
            EXPAND_BUFFERS => return ExpandBufnames(pat, numMatches, matches, options),
            EXPAND_DIFF_BUFFERS => {
                return ExpandBufnames(pat, numMatches, matches, options | BUF_DIFF_FILTER);
            }
            EXPAND_TAGS | EXPAND_TAGS_LISTFILES => {
                return expand_tags(context == EXPAND_TAGS, pat, numMatches, matches);
            }
            EXPAND_COLORS => {
                let mut dirs = [c"colors".as_ptr() as *mut c_char, ptr::null_mut()];
                return ExpandRTDir(
                    pat,
                    (DIP_START + DIP_OPT) as c_int,
                    numMatches,
                    matches,
                    dirs.as_mut_ptr(),
                );
            }
            EXPAND_COMPILER => {
                let mut dirs = [c"compiler".as_ptr() as *mut c_char, ptr::null_mut()];
                return ExpandRTDir(pat, 0, numMatches, matches, dirs.as_mut_ptr());
            }
            EXPAND_OWNSYNTAX => {
                let mut dirs = [c"syntax".as_ptr() as *mut c_char, ptr::null_mut()];
                return ExpandRTDir(pat, 0, numMatches, matches, dirs.as_mut_ptr());
            }
            EXPAND_FILETYPE => {
                let mut dirs = [
                    c"syntax".as_ptr() as *mut c_char,
                    c"indent".as_ptr() as *mut c_char,
                    c"ftplugin".as_ptr() as *mut c_char,
                    ptr::null_mut(),
                ];
                return ExpandRTDir(pat, 0, numMatches, matches, dirs.as_mut_ptr());
            }
            EXPAND_KEYMAP => {
                let mut dirs = [c"keymap".as_ptr() as *mut c_char, ptr::null_mut()];
                return ExpandRTDir(pat, 0, numMatches, matches, dirs.as_mut_ptr());
            }
            EXPAND_USER_LIST => return ExpandUserList(xp, matches, numMatches),
            EXPAND_USER_LUA => return ExpandUserLua(xp, numMatches, matches),
            EXPAND_PACKADD => return ExpandPackAddDir(pat, numMatches, matches),
            EXPAND_RUNTIME => return expand_runtime_cmd(pat, numMatches, matches),
            EXPAND_PATTERN_IN_BUF => {
                return expand_pattern_in_buf(pat, (*xp).xp_search_dir, matches, numMatches);
            }
            _ => {}
        }

        // When expanding a function name starting with s:, match the <SNR>nr_
        // prefix.
        let mut tofree = ptr::null_mut::<c_char>();
        if context == EXPAND_USER_FUNC && strncmp(pat, c"^s:".as_ptr(), 3) == 0 {
            let len = strlen(pat) + 20;
            tofree = xmalloc(len) as *mut c_char;
            snprintf(tofree, len, c"^<SNR>\\d\\+_%s".as_ptr(), pat.add(3));
            pat = tofree;
        }

        if context == EXPAND_LUA {
            // `tofree` is still NULL here: only EXPAND_USER_FUNC sets it.
            return nlua_expand_get_matches(numMatches, matches);
        }

        let mut regmatch = regmatch_T {
            regprog: ptr::null_mut(),
            startp: [ptr::null_mut(); 10],
            endp: [ptr::null_mut(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        if !fuzzy {
            regmatch.regprog = vim_regcomp(pat, if magic_isset() { RE_MAGIC } else { 0 });
            if regmatch.regprog.is_null() {
                xfree(tofree as *mut c_void);
                return FAIL;
            }
            // Set ignore-case according to 'ignorecase', 'smartcase' and pat.
            regmatch.rm_ic = ignorecase(pat) != 0;
        }

        let ret = match context {
            EXPAND_SETTINGS | EXPAND_BOOL_SETTINGS => {
                ExpandSettings(xp, &raw mut regmatch, pat, numMatches, matches, fuzzy)
            }
            EXPAND_STRING_SETTING => {
                ExpandStringSetting(xp, &raw mut regmatch, numMatches, matches)
            }
            EXPAND_SETTING_SUBTRACT => {
                ExpandSettingSubtract(xp, &raw mut regmatch, numMatches, matches)
            }
            EXPAND_MAPPINGS => ExpandMappings(pat, &raw mut regmatch, numMatches, matches),
            EXPAND_ARGOPT => expand_argopt(pat, xp, &raw mut regmatch, matches, numMatches),
            EXPAND_USER_DEFINED => {
                ExpandUserDefined(pat, xp, &raw mut regmatch, matches, numMatches)
            }
            _ => ExpandOther(pat, xp, &raw mut regmatch, matches, numMatches),
        };

        if !fuzzy {
            vim_regfree(regmatch.regprog);
        }
        xfree(tofree as *mut c_void);
        ret
    }
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
pub unsafe fn ExpandGeneric(
    pat: *const c_char,
    xp: *mut expand_T,
    regmatch: *mut regmatch_T,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
    func: CompleteListItemGetter,
    escaped: bool,
) {
    unsafe {
        let get_item = func.expect("ExpandGeneric needs a generator");
        let fuzzy = cmdline_fuzzy_complete(pat);
        *matches = ptr::null_mut();
        *numMatches = 0;

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
        ga_init(&raw mut ga, itemsize as c_int, 30);

        for i in 0.. {
            let mut str = get_item(xp, i);
            if str.is_null() {
                break; // end of list
            }
            if *str == 0 {
                continue; // skip empty strings
            }

            // An empty pattern matches everything; otherwise every
            // candidate is tested, and under 'wildoptions'=fuzzy also scored.
            // `xp_pattern` is re-read each pass, as upstream does: the
            // generator is handed `xp` and a user-defined one can move it.
            let mut score = 0;
            let matched = if *(*xp).xp_pattern == 0 {
                true
            } else if fuzzy {
                score = fuzzy_match_str(str, pat);
                score != FUZZY_SCORE_NONE
            } else {
                vim_regexec(regmatch, str, 0)
            };
            if !matched {
                continue;
            }

            str = if escaped {
                vim_strsave_escaped(str, c" \t\\.".as_ptr())
            } else {
                xstrdup(str)
            };

            ga_grow(&raw mut ga, 1);
            if fuzzy {
                (ga.ga_data as *mut fuzmatch_str_T)
                    .offset(ga.ga_len as isize)
                    .write(fuzmatch_str_T {
                        idx: ga.ga_len,
                        str,
                        score,
                    });
            } else {
                (ga.ga_data as *mut *mut c_char)
                    .offset(ga.ga_len as isize)
                    .write(str);
            }
            ga.ga_len += 1;

            if ptr::fn_addr_eq(get_item, get_menu_names as ItemGetter) {
                // Undo the separator get_menu_names() added, in the copy that
                // is now in the array.
                let last = str.add(strlen(str) - 1);
                if *last == 1 {
                    *last = b'.' as c_char;
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
                (*xp).xp_context,
                EXPAND_MENUNAMES
                    | EXPAND_STRING_SETTING
                    | EXPAND_MENUS
                    | EXPAND_SCRIPTNAMES
                    | EXPAND_ARGOPT
            );
        // <SNR> functions should be sorted to the end.
        let funcsort = matches!(
            (*xp).xp_context,
            EXPAND_EXPRESSION | EXPAND_FUNCTIONS | EXPAND_USER_FUNC
        );

        if sort_matches {
            if funcsort {
                qsort(
                    ga.ga_data,
                    ga.ga_len as size_t,
                    size_of::<*mut c_char>(),
                    Some(sort_func_compare),
                );
            } else {
                sort_strings(ga.ga_data as *mut *mut c_char, ga.ga_len);
            }
        }

        if fuzzy {
            fuzzymatches_to_strmatches(
                ga.ga_data as *mut fuzmatch_str_T,
                matches,
                ga.ga_len,
                funcsort,
            );
        } else {
            *matches = ga.ga_data as *mut *mut c_char;
        }
        *numMatches = ga.ga_len;

        // Reset the variables used for special highlight names expansion, so
        // that they don't show up when getting normal highlight names by ID.
        reset_expand_highlight();
    }
}
