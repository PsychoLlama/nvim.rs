//! Changing directory, and telling autocommands about it.
//!
//! [`vim_chdir`] resolves a relative directory name along `'cdpath'` before
//! changing to it, [`vim_chdirfile`] changes to a file's own directory, and
//! [`do_autocmd_dirchanged`] fires `DirChangedPre`/`DirChanged` with the
//! `v:event` dictionary those events promise.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_autocmd_dirchanged(
    mut new_dir: *mut ::core::ffi::c_char,
    mut scope: CdScope,
    mut cause: CdCause,
    mut pre: bool,
) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut event: event_T = (if pre as ::core::ffi::c_int != 0 {
            EVENT_DIRCHANGEDPRE as ::core::ffi::c_int
        } else {
            EVENT_DIRCHANGED as ::core::ffi::c_int
        }) as event_T;
        if recursive.get() as ::core::ffi::c_int != 0 || !has_event(event) {
            return;
        }
        recursive.set(true_0 != 0);
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut dict: *mut dict_T = get_v_event(&raw mut save_v_event);
        let mut buf: [::core::ffi::c_char; 8] = [0; 8];
        match scope as ::core::ffi::c_int {
            2 => {
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                    b"global\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            1 => {
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                    b"tabpage\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            0 => {
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                    b"window\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            -1 => {
                abort();
            }
            _ => {}
        }
        if pre {
            tv_dict_add_str(
                dict,
                b"directory\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                new_dir,
            );
        } else {
            tv_dict_add_str(
                dict,
                b"cwd\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
                new_dir,
            );
        }
        tv_dict_add_str(
            dict,
            b"scope\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        tv_dict_add_bool(
            dict,
            b"changed_window\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
            (cause as ::core::ffi::c_int == kCdCauseWindow as ::core::ffi::c_int)
                as ::core::ffi::c_int as BoolVarValue,
        );
        tv_dict_set_keys_readonly(dict);
        match cause as ::core::ffi::c_int {
            2 => {
                snprintf(
                    &raw mut buf as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
                    b"auto\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            -1 => {
                abort();
            }
            0 | 1 | _ => {}
        }
        apply_autocmds(
            event,
            &raw mut buf as *mut ::core::ffi::c_char,
            new_dir,
            false_0 != 0,
            curbuf.get(),
        );
        restore_v_event(dict, &raw mut save_v_event);
        recursive.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn vim_chdirfile(
    mut fname: *mut ::core::ffi::c_char,
    mut cause: CdCause,
) -> ::core::ffi::c_int {
    unsafe {
        let mut dir: [::core::ffi::c_char; 4096] = [0; 4096];
        xstrlcpy(
            &raw mut dir as *mut ::core::ffi::c_char,
            fname,
            MAXPATHL as size_t,
        );
        *path_tail_with_sep(&raw mut dir as *mut ::core::ffi::c_char) = NUL as ::core::ffi::c_char;
        if os_dirname(
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
        ) != OK
        {
            (*NameBuff.ptr())[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        }
        if pathcmp(
            &raw mut dir as *mut ::core::ffi::c_char,
            NameBuff.ptr() as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
        ) == 0 as ::core::ffi::c_int
        {
            return OK;
        }
        if cause as ::core::ffi::c_int != kCdCauseOther as ::core::ffi::c_int {
            do_autocmd_dirchanged(
                &raw mut dir as *mut ::core::ffi::c_char,
                kCdScopeWindow,
                cause,
                true_0 != 0,
            );
        }
        if os_chdir(&raw mut dir as *mut ::core::ffi::c_char) != 0 as ::core::ffi::c_int {
            return FAIL;
        }
        if cause as ::core::ffi::c_int != kCdCauseOther as ::core::ffi::c_int {
            do_autocmd_dirchanged(
                &raw mut dir as *mut ::core::ffi::c_char,
                kCdScopeWindow,
                cause,
                false_0 != 0,
            );
        }
        return OK;
    }
}

pub unsafe extern "C" fn vim_chdir(mut new_dir: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut file_to_find: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut search_ctx: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dir_name: *mut ::core::ffi::c_char = find_directory_in_path(
            new_dir,
            strlen(new_dir),
            FNAME_MESS as ::core::ffi::c_int,
            (*curbuf.get()).b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        xfree(file_to_find as *mut ::core::ffi::c_void);
        vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
        if dir_name.is_null() {
            return -1 as ::core::ffi::c_int;
        }
        let mut r: ::core::ffi::c_int = os_chdir(dir_name);
        xfree(dir_name as *mut ::core::ffi::c_void);
        return r;
    }
}
