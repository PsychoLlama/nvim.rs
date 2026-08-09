//! The precomputed 'runtimepath' search path, and the thread-safe copy of it.
//!
//! Walking 'runtimepath' *and* 'packpath' for every `:runtime` is O(entries)
//! filesystem calls, so the resolved, ordered, deduplicated list of directories
//! is built once (`runtime_search_path_build`) and reused until either option
//! changes.  `after/` directories sort last and `pack/*/start` entries are
//! spliced in where the pack was found, which is the ordering the whole plugin
//! ecosystem depends on and which nothing else in the tree recomputes.
//!
//! There are two copies.  The main one is invalidated by
//! `did_set_runtimepackpath` and rebuilt lazily by
//! `runtime_search_path_validate`; the second is a refcounted snapshot taken
//! under a mutex for the *thread* that serves `nvim_get_runtime_file` off the
//! main loop, which is why `copy_runtime_search_path` and
//! `runtime_search_path_unref` exist at all.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn runtime_search_path_get_cached(
    mut ref_0: *mut ::core::ffi::c_int,
) -> RuntimeSearchPath {
    unsafe {
        runtime_search_path_validate();
        *ref_0 = 0 as ::core::ffi::c_int;
        if (*runtime_search_path_ref.ptr()).is_null() {
            *ref_0 += 1;
            runtime_search_path_ref.set(ref_0);
        }
        return runtime_search_path.get();
    }
}

unsafe extern "C" fn copy_runtime_search_path(src: RuntimeSearchPath) -> RuntimeSearchPath {
    unsafe {
        let mut dst: RuntimeSearchPath = RuntimeSearchPath {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<SearchPathItem>(),
        };
        let mut j: size_t = 0 as size_t;
        while j < src.size {
            let mut item: SearchPathItem = *src.items.add(j);
            if dst.size == dst.capacity {
                dst.capacity = if dst.capacity != 0 {
                    dst.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                dst.items = xrealloc(
                    dst.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<SearchPathItem>().wrapping_mul(dst.capacity),
                ) as *mut SearchPathItem;
            } else {
            };
            let c2rust_fresh4 = dst.size;
            dst.size = dst.size.wrapping_add(1);
            *dst.items.add(c2rust_fresh4) = SearchPathItem {
                path: xstrdup(item.path),
                after: item.after,
                pack_inserted: item.pack_inserted,
                has_lua: item.has_lua,
                pos_in_rtp: item.pos_in_rtp,
            };
            j = j.wrapping_add(1);
        }
        return dst;
    }
}

pub(crate) unsafe extern "C" fn runtime_search_path_unref(
    mut path: RuntimeSearchPath,
    mut ref_0: *const ::core::ffi::c_int,
) {
    unsafe {
        if *ref_0 != 0 {
            if runtime_search_path_ref.get() == ref_0 as *mut ::core::ffi::c_int {
                runtime_search_path_ref.set(::core::ptr::null_mut::<::core::ffi::c_int>());
            } else {
                runtime_search_path_free(path);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn do_in_cached_path(
    mut name: *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
    mut callback: DoInRuntimepathCB,
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut did_one: bool = false_0 != 0;
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        if p_verbose.get() > 10 as OptInt && !name.is_null() {
            verbose_enter();
            smsg_c!(
                0 as ::core::ffi::c_int,
                gettext(c"Searching for \"%s\" in runtime path".as_ptr()),
                name,
            );
            verbose_leave();
        }
        let mut ref_0: ::core::ffi::c_int = 0;
        let mut path: RuntimeSearchPath = runtime_search_path_get_cached(&raw mut ref_0);
        let mut do_all: bool = flags & DIP_ALL as ::core::ffi::c_int != 0 as ::core::ffi::c_int;
        let mut j: size_t = 0 as size_t;
        while j < path.size {
            let mut item: SearchPathItem = *path.items.add(j);
            let mut buflen: size_t = strlen(item.path);
            's_32: {
                if flags & (DIP_NOAFTER as ::core::ffi::c_int | DIP_AFTER as ::core::ffi::c_int)
                    != 0
                {
                    if item.after as ::core::ffi::c_int != 0
                        && flags & DIP_NOAFTER as ::core::ffi::c_int != 0
                        || !item.after && flags & DIP_AFTER as ::core::ffi::c_int != 0
                    {
                        break 's_32;
                    }
                }
                if name.is_null() {
                    Some(callback.expect("non-null function pointer"))
                        .expect("non-null function pointer")(
                        1 as ::core::ffi::c_int,
                        &raw mut item.path,
                        do_all,
                        cookie,
                    );
                } else if buflen.wrapping_add(strlen(name)).wrapping_add(2 as size_t)
                    < MAXPATHL as size_t
                {
                    strcpy(&raw mut buf as *mut ::core::ffi::c_char, item.path);
                    add_pathsep(&raw mut buf as *mut ::core::ffi::c_char);
                    let mut tail: *mut ::core::ffi::c_char = (&raw mut buf
                        as *mut ::core::ffi::c_char)
                        .add(strlen(&raw mut buf as *mut ::core::ffi::c_char));
                    let mut np: *mut ::core::ffi::c_char = name;
                    while *np as ::core::ffi::c_int != NUL
                        && (do_all as ::core::ffi::c_int != 0 || !did_one)
                    {
                        debug_assert!(
                            4096_isize
                                >= tail.offset_from(&raw mut buf as *mut ::core::ffi::c_char),
                            "MAXPATHL >= (tail - buf)"
                        );
                        copy_option_part(
                            &raw mut np,
                            tail,
                            (MAXPATHL as isize
                                - tail.offset_from(&raw mut buf as *mut ::core::ffi::c_char))
                                as size_t,
                            c"\t ".as_ptr() as *mut ::core::ffi::c_char,
                        );
                        if p_verbose.get() > 10 as OptInt {
                            verbose_enter();
                            smsg_c!(
                                0 as ::core::ffi::c_int,
                                gettext(c"Searching for \"%s\"".as_ptr()),
                                &raw mut buf as *mut ::core::ffi::c_char,
                            );
                            verbose_leave();
                        }
                        let mut ew_flags: ::core::ffi::c_int =
                            (if flags & DIP_DIR as ::core::ffi::c_int != 0 {
                                EW_DIR
                            } else {
                                EW_FILE
                            }) | (if flags & DIP_DIRFILE as ::core::ffi::c_int != 0 {
                                EW_DIR | EW_FILE
                            } else {
                                0 as ::core::ffi::c_int
                            }) | EW_NOBREAK;
                        let mut pat: [*mut ::core::ffi::c_char; 1] =
                            [&raw mut buf as *mut ::core::ffi::c_char];
                        did_one = did_one as ::core::ffi::c_int
                            | (gen_expand_wildcards_and_cb(
                                1 as ::core::ffi::c_int,
                                &raw mut pat as *mut *mut ::core::ffi::c_char,
                                ew_flags,
                                do_all,
                                callback,
                                cookie,
                            ) == OK) as ::core::ffi::c_int
                            != 0;
                    }
                }
            }
            j = j.wrapping_add(1);
        }
        if !did_one && !name.is_null() {
            if flags & DIP_ERR as ::core::ffi::c_int != 0 {
                semsg_c!(
                    gettext(&raw const e_dirnotf as *const ::core::ffi::c_char),
                    c"runtime path".as_ptr(),
                    name,
                );
            } else if p_verbose.get() > 1 as OptInt {
                verbose_enter();
                smsg_c!(
                    0 as ::core::ffi::c_int,
                    gettext(c"not found in runtime path: \"%s\"".as_ptr()),
                    name,
                );
                verbose_leave();
            }
        }
        runtime_search_path_unref(path, &raw mut ref_0);
        return if did_one as ::core::ffi::c_int != 0 {
            OK
        } else {
            FAIL
        };
    }
}

unsafe extern "C" fn push_path(
    mut search_path: *mut RuntimeSearchPath,
    mut rtp_used: *mut Set_String,
    mut entry: *mut ::core::ffi::c_char,
    mut after: bool,
    mut pos_in_rtp: size_t,
) -> bool {
    unsafe {
        let mut key_alloc: *mut String_0 = ::core::ptr::null_mut::<String_0>();
        if set_put_String(rtp_used, cstr_as_string(entry), &raw mut key_alloc) {
            *key_alloc = cstr_to_string(entry);
            if (*search_path).size == (*search_path).capacity {
                (*search_path).capacity = if (*search_path).capacity != 0 {
                    (*search_path).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*search_path).items = xrealloc(
                    (*search_path).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<SearchPathItem>().wrapping_mul((*search_path).capacity),
                ) as *mut SearchPathItem;
            } else {
            };
            let c2rust_fresh6 = (*search_path).size;
            (*search_path).size = (*search_path).size.wrapping_add(1);
            *(*search_path).items.add(c2rust_fresh6) = SearchPathItem {
                path: (*key_alloc).data,
                after: after,
                pack_inserted: false,
                has_lua: kNone,
                pos_in_rtp: pos_in_rtp,
            };
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

unsafe extern "C" fn expand_rtp_entry(
    mut search_path: *mut RuntimeSearchPath,
    mut rtp_used: *mut Set_String,
    mut entry: *mut ::core::ffi::c_char,
    mut after: bool,
    mut pos_in_rtp: size_t,
) {
    unsafe {
        if set_has_String(rtp_used, cstr_as_string(entry)) {
            return;
        }
        if *entry == 0 {
            push_path(search_path, rtp_used, entry, after, pos_in_rtp);
        }
        let mut num_files: ::core::ffi::c_int = 0;
        let mut files: *mut *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        let mut pat: [*mut ::core::ffi::c_char; 1] = [entry as *mut ::core::ffi::c_char];
        if gen_expand_wildcards(
            1 as ::core::ffi::c_int,
            &raw mut pat as *mut *mut ::core::ffi::c_char,
            &raw mut num_files,
            &raw mut files,
            EW_DIR | EW_NOBREAK,
        ) == OK
        {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < num_files {
                push_path(
                    search_path,
                    rtp_used,
                    *files.offset(i as isize),
                    after,
                    pos_in_rtp,
                );
                i += 1;
            }
            FreeWild(num_files, files);
        }
    }
}

unsafe extern "C" fn expand_pack_entry(
    mut search_path: *mut RuntimeSearchPath,
    mut rtp_used: *mut Set_String,
    mut after_path: *mut CharVec,
    mut pack_entry: *mut ::core::ffi::c_char,
    mut pack_entry_len: size_t,
    mut pos_in_rtp: size_t,
) {
    unsafe {
        static buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
        let mut start_pat: [*mut ::core::ffi::c_char; 2] = [
            c"/pack/*/start/*".as_ptr() as *mut ::core::ffi::c_char,
            c"/start/*".as_ptr() as *mut ::core::ffi::c_char,
        ];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 2 as ::core::ffi::c_int {
            if pack_entry_len
                .wrapping_add(strlen(start_pat[i as usize] as *const ::core::ffi::c_char))
                .wrapping_add(1 as size_t)
                <= ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
            {
                xstrlcpy(
                    buf.ptr() as *mut ::core::ffi::c_char,
                    pack_entry,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                xstrlcpy(
                    (buf.ptr() as *mut ::core::ffi::c_char).add(pack_entry_len),
                    start_pat[i as usize] as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
                        .wrapping_sub(pack_entry_len),
                );
                expand_rtp_entry(
                    search_path,
                    rtp_used,
                    buf.ptr() as *mut ::core::ffi::c_char,
                    false_0 != 0,
                    pos_in_rtp,
                );
                let mut after_size: size_t =
                    strlen(buf.ptr() as *mut ::core::ffi::c_char).wrapping_add(7 as size_t);
                let mut after: *mut ::core::ffi::c_char =
                    xmallocz(after_size) as *mut ::core::ffi::c_char;
                xstrlcpy(after, buf.ptr() as *mut ::core::ffi::c_char, after_size);
                xstrlcat(after, c"/after".as_ptr(), after_size);
                if (*after_path).size == (*after_path).capacity {
                    (*after_path).capacity = if (*after_path).capacity != 0 {
                        (*after_path).capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    (*after_path).items = xrealloc(
                        (*after_path).items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                            .wrapping_mul((*after_path).capacity),
                    ) as *mut *mut ::core::ffi::c_char;
                } else {
                };
                let c2rust_fresh7 = (*after_path).size;
                (*after_path).size = (*after_path).size.wrapping_add(1);
                let c2rust_lvalue_ptr = &raw mut *(*after_path).items.add(c2rust_fresh7);
                *c2rust_lvalue_ptr = after;
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn path_is_after(
    mut buf: *mut ::core::ffi::c_char,
    mut buflen: size_t,
) -> bool {
    unsafe {
        return buflen >= 5 as size_t
            && (!(buflen >= 6 as size_t)
                || vim_ispathsep(*buf.add(buflen.wrapping_sub(6 as size_t)) as ::core::ffi::c_int)
                    as ::core::ffi::c_int
                    != 0)
            && strcmp(
                buf.add(buflen).offset(-(5 as ::core::ffi::c_int as isize)),
                c"after".as_ptr(),
            ) == 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn runtime_search_path_build() -> RuntimeSearchPath {
    unsafe {
        let mut pack_entries: StringVec = StringVec {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<String_0>(),
        };
        let mut pack_used: Map_String_int = MAP_INIT;
        let mut rtp_used: Set_String = SET_INIT;
        let mut search_path: RuntimeSearchPath = RuntimeSearchPath {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<SearchPathItem>(),
        };
        let mut after_path: CharVec = CharVec {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        };
        static buf: GlobalCell<[::core::ffi::c_char; 4096]> = GlobalCell::new([0; 4096]);
        let mut entry: *mut ::core::ffi::c_char = p_pp.get();
        while *entry as ::core::ffi::c_int != NUL {
            let mut cur_entry: *mut ::core::ffi::c_char = entry;
            let mut buflen: size_t = copy_option_part(
                &raw mut entry,
                buf.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                c",".as_ptr() as *mut ::core::ffi::c_char,
            );
            let mut the_entry: String_0 = String_0 {
                data: cur_entry,
                size: buflen,
            };
            if pack_entries.size == pack_entries.capacity {
                pack_entries.capacity = if pack_entries.capacity != 0 {
                    pack_entries.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                pack_entries.items = xrealloc(
                    pack_entries.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<String_0>().wrapping_mul(pack_entries.capacity),
                ) as *mut String_0;
            } else {
            };
            let c2rust_fresh5 = pack_entries.size;
            pack_entries.size = pack_entries.size.wrapping_add(1);
            *pack_entries.items.add(c2rust_fresh5) = the_entry;
            map_put_String_int(&raw mut pack_used, the_entry, 0 as ::core::ffi::c_int);
        }
        let mut rtp_entry: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        rtp_entry = p_rtp.get();
        while *rtp_entry as ::core::ffi::c_int != NUL {
            let mut cur_entry_0: *mut ::core::ffi::c_char = rtp_entry;
            let mut buflen_0: size_t = copy_option_part(
                &raw mut rtp_entry,
                buf.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                c",".as_ptr() as *mut ::core::ffi::c_char,
            );
            if path_is_after(buf.ptr() as *mut ::core::ffi::c_char, buflen_0) {
                rtp_entry = cur_entry_0;
                break;
            } else {
                let mut pos_in_rtp: size_t = cur_entry_0.offset_from(p_rtp.get()) as size_t;
                expand_rtp_entry(
                    &raw mut search_path,
                    &raw mut rtp_used,
                    buf.ptr() as *mut ::core::ffi::c_char,
                    false_0 != 0,
                    pos_in_rtp,
                );
                let mut h: *mut handle_T = map_ref_String_int(
                    &raw mut pack_used,
                    cstr_as_string(buf.ptr() as *mut ::core::ffi::c_char),
                    ::core::ptr::null_mut::<*mut String_0>(),
                ) as *mut handle_T;
                if !h.is_null() {
                    *h += 1;
                    expand_pack_entry(
                        &raw mut search_path,
                        &raw mut rtp_used,
                        &raw mut after_path,
                        buf.ptr() as *mut ::core::ffi::c_char,
                        buflen_0,
                        pos_in_rtp,
                    );
                }
            }
        }
        let mut sentinel_pos_in_rtp: size_t = rtp_entry.offset_from(p_rtp.get()) as size_t;
        sentinel_pos_in_rtp = sentinel_pos_in_rtp.wrapping_sub(
            (if sentinel_pos_in_rtp > 0 as size_t {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as size_t,
        );
        let mut i: size_t = 0 as size_t;
        while i < pack_entries.size {
            let mut item: String_0 = *pack_entries.items.add(i);
            let mut h_0: handle_T = map_get_String_int(&raw mut pack_used, item);
            if h_0 == 0 as ::core::ffi::c_int {
                expand_pack_entry(
                    &raw mut search_path,
                    &raw mut rtp_used,
                    &raw mut after_path,
                    item.data,
                    item.size,
                    sentinel_pos_in_rtp,
                );
            }
            i = i.wrapping_add(1);
        }
        let mut i_0: size_t = 0 as size_t;
        while i_0 < after_path.size {
            expand_rtp_entry(
                &raw mut search_path,
                &raw mut rtp_used,
                *after_path.items.add(i_0),
                true_0 != 0,
                sentinel_pos_in_rtp,
            );
            xfree(*after_path.items.add(i_0) as *mut ::core::ffi::c_void);
            i_0 = i_0.wrapping_add(1);
        }
        while *rtp_entry as ::core::ffi::c_int != NUL {
            let mut cur_entry_1: *mut ::core::ffi::c_char = rtp_entry;
            let mut buflen_1: size_t = copy_option_part(
                &raw mut rtp_entry,
                buf.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                c",".as_ptr() as *mut ::core::ffi::c_char,
            );
            let mut pos_in_rtp_0: size_t = cur_entry_1.offset_from(p_rtp.get()) as size_t;
            expand_rtp_entry(
                &raw mut search_path,
                &raw mut rtp_used,
                buf.ptr() as *mut ::core::ffi::c_char,
                path_is_after(buf.ptr() as *mut ::core::ffi::c_char, buflen_1),
                pos_in_rtp_0,
            );
        }
        xfree(pack_entries.items as *mut ::core::ffi::c_void);
        pack_entries.capacity = 0 as size_t;
        pack_entries.size = pack_entries.capacity;
        pack_entries.items = ::core::ptr::null_mut::<String_0>();
        xfree(after_path.items as *mut ::core::ffi::c_void);
        after_path.capacity = 0 as size_t;
        after_path.size = after_path.capacity;
        after_path.items = ::core::ptr::null_mut::<*mut ::core::ffi::c_char>();
        xfree(pack_used.set.keys as *mut ::core::ffi::c_void);
        xfree(pack_used.set.h.hash as *mut ::core::ffi::c_void);
        pack_used.set = SET_INIT;
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut pack_used.values as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL_0;
        let _ = *ptr_;
        xfree(rtp_used.keys as *mut ::core::ffi::c_void);
        xfree(rtp_used.h.hash as *mut ::core::ffi::c_void);
        rtp_used = SET_INIT;
        return search_path;
    }
}

pub unsafe extern "C" fn did_set_runtimepackpath(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    runtime_search_path_valid.set(false_0 != 0);
    return ::core::ptr::null::<::core::ffi::c_char>();
}

unsafe extern "C" fn runtime_search_path_free(mut path: RuntimeSearchPath) {
    unsafe {
        let mut j: size_t = 0 as size_t;
        while j < path.size {
            let mut item: SearchPathItem = *path.items.add(j);
            xfree(item.path as *mut ::core::ffi::c_void);
            j = j.wrapping_add(1);
        }
        xfree(path.items as *mut ::core::ffi::c_void);
        path.capacity = 0 as size_t;
        path.size = path.capacity;
        path.items = ::core::ptr::null_mut::<SearchPathItem>();
    }
}

pub unsafe extern "C" fn runtime_search_path_validate() {
    unsafe {
        if !nlua_is_deferred_safe() {
            return;
        }
        if !runtime_search_path_valid.get() {
            if (*runtime_search_path_ref.ptr()).is_null() {
                msg_ext_ui_flush();
                runtime_search_path_free(runtime_search_path.get());
            }
            runtime_search_path.set(runtime_search_path_build());
            runtime_search_path_valid.set(true_0 != 0);
            runtime_search_path_ref.set(::core::ptr::null_mut::<::core::ffi::c_int>());
            update_runtime_search_path_thread(true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn update_runtime_search_path_thread(mut force: bool) {
    unsafe {
        if !force
            && !(runtime_search_path_valid.get() as ::core::ffi::c_int != 0
                && !runtime_search_path_valid_thread.get())
        {
            return;
        }
        uv_mutex_lock(runtime_search_path_mutex.ptr());
        runtime_search_path_free(runtime_search_path_thread.get());
        runtime_search_path_thread.set(copy_runtime_search_path(runtime_search_path.get()));
        uv_mutex_unlock(runtime_search_path_mutex.ptr());
        runtime_search_path_valid_thread.set(true_0 != 0);
    }
}
