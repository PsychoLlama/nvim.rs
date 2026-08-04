//! The message history behind `:messages` and `'messagesopt'`.
//!
//! A doubly-linked list of [`MessageHistoryEntry`] capped at
//! `'messagesopt'`'s `history:` count. [`msg_hist_add`] appends and evicts,
//! [`ex_messages`] prints (or, under `ext_messages`, emits) the tail of it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn hl_msg_free(mut hl_msg: HlMessage) {
    unsafe {
        let mut i: size_t = 0 as size_t;
        while i < hl_msg.size {
            xfree((*hl_msg.items.offset(i as isize)).text.data as *mut ::core::ffi::c_void);
            i = i.wrapping_add(1);
        }
        xfree(hl_msg.items as *mut ::core::ffi::c_void);
        hl_msg.capacity = 0 as size_t;
        hl_msg.size = hl_msg.capacity;
        hl_msg.items = ::core::ptr::null_mut::<HlMessageChunk>();
    }
}

pub(crate) unsafe extern "C" fn msg_hist_add(
    mut s: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut hl_id: ::core::ffi::c_int,
) {
    unsafe {
        let mut text: String_0 = String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: if len < 0 as ::core::ffi::c_int {
                strlen(s)
            } else {
                len as size_t
            },
        };
        while text.size > 0 as size_t && *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
            text.size = text.size.wrapping_sub(1);
            s = s.offset(1);
        }
        while text.size > 0 as size_t
            && *s.offset(text.size.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                == '\n' as ::core::ffi::c_int
        {
            text.size = text.size.wrapping_sub(1);
        }
        if text.size == 0 as size_t {
            return;
        }
        text.data =
            xmemdupz(s as *const ::core::ffi::c_void, text.size) as *mut ::core::ffi::c_char;
        let mut msg_0: HlMessage = HlMessage {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<HlMessageChunk>(),
        };
        if msg_0.size == msg_0.capacity {
            msg_0.capacity = if msg_0.capacity != 0 {
                msg_0.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            msg_0.items = xrealloc(
                msg_0.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg_0.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh7 = msg_0.size;
        msg_0.size = msg_0.size.wrapping_add(1);
        *msg_0.items.offset(c2rust_fresh7 as isize) = HlMessageChunk {
            text: text,
            hl_id: hl_id,
        };
        msg_hist_add_multihl(msg_0, false_0 != 0, ::core::ptr::null_mut::<MessageData>());
    }
}

pub(crate) unsafe extern "C" fn msg_hist_add_multihl(
    mut msg_0: HlMessage,
    mut temp: bool,
    mut _msg_data: *mut MessageData,
) {
    unsafe {
        if do_clear_hist_temp.get() {
            msg_hist_clear_temp();
            do_clear_hist_temp.set(false_0 != 0);
        }
        if msg_hist_off.get() as ::core::ffi::c_int != 0
            || msg_silent.get() != 0 as ::core::ffi::c_int
        {
            hl_msg_free(msg_0);
            return;
        }
        let mut entry: *mut MessageHistoryEntry =
            xmalloc(::core::mem::size_of::<MessageHistoryEntry>()) as *mut MessageHistoryEntry;
        (*entry).msg = msg_0;
        (*entry).temp = temp;
        (*entry).kind = if !(*msg_ext_kind.ptr()).is_null() {
            xstrdup(msg_ext_kind.get())
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
        (*entry).prev = msg_hist_last.get() as *mut msg_hist;
        (*entry).next = ::core::ptr::null_mut::<msg_hist>();
        (*entry).append = msg_ext_append.get();
        if (*msg_hist_first.ptr()).is_null() {
            msg_hist_first.set(entry);
        }
        if !(*msg_hist_last.ptr()).is_null() {
            (*msg_hist_last.get()).next = entry as *mut msg_hist;
        }
        if (*msg_hist_temp.ptr()).is_null() {
            msg_hist_temp.set(entry);
        }
        (*msg_hist_len.ptr()) += !temp as ::core::ffi::c_int;
        msg_hist_last.set(entry);
        msg_ext_history.set(true_0 != 0);
        msg_hist_clear(msg_hist_max.get());
    }
}

pub(crate) unsafe extern "C" fn msg_hist_free_msg(mut entry: *mut MessageHistoryEntry) {
    unsafe {
        if (*entry).next.is_null() {
            msg_hist_last.set((*entry).prev as *mut MessageHistoryEntry);
        } else {
            (*(*entry).next).prev = (*entry).prev;
        }
        if (*entry).prev.is_null() {
            msg_hist_first.set((*entry).next as *mut MessageHistoryEntry);
        } else {
            (*(*entry).prev).next = (*entry).next;
        }
        if entry == msg_hist_temp.get() {
            msg_hist_temp.set((*entry).next as *mut MessageHistoryEntry);
        }
        hl_msg_free((*entry).msg);
        xfree((*entry).kind as *mut ::core::ffi::c_void);
        xfree(entry as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn msg_hist_clear(mut keep: ::core::ffi::c_int) {
    unsafe {
        while msg_hist_len.get() > keep
            || keep == 0 as ::core::ffi::c_int && !(*msg_hist_first.ptr()).is_null()
        {
            (*msg_hist_len.ptr()) -= !(*msg_hist_first.get()).temp as ::core::ffi::c_int;
            msg_hist_free_msg(msg_hist_first.get());
        }
    }
}

pub unsafe extern "C" fn msg_hist_clear_temp() {
    unsafe {
        while !(*msg_hist_temp.ptr()).is_null() {
            let mut next: *mut MessageHistoryEntry =
                (*msg_hist_temp.get()).next as *mut MessageHistoryEntry;
            if (*msg_hist_temp.get()).temp {
                msg_hist_free_msg(msg_hist_temp.get());
            }
            msg_hist_temp.set(next);
        }
    }
}

pub unsafe extern "C" fn messagesopt_changed() -> ::core::ffi::c_int {
    unsafe {
        let mut messages_flags_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut messages_wait_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut messages_history_new: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut progress_target_flag: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = p_mopt.get();
        while *p as ::core::ffi::c_int != NUL {
            if strnequal(
                p,
                b"hit-enter\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            ) {
                p = p.offset(
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                        as isize,
                );
                messages_flags_new |= kOptMoptFlagHitEnter as ::core::ffi::c_int;
            } else if strnequal(
                p,
                b"wait:\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            ) as ::core::ffi::c_int
                != 0
                && ascii_isdigit(*p.offset(
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                        as isize,
                ) as ::core::ffi::c_int) as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                        as isize,
                );
                messages_wait_new = getdigits_int(&raw mut p, false_0 != 0, INT_MAX);
                messages_flags_new |= kOptMoptFlagWait as ::core::ffi::c_int;
            } else if strnequal(
                p,
                b"history:\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            ) as ::core::ffi::c_int
                != 0
                && ascii_isdigit(*p.offset(
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                        as isize,
                ) as ::core::ffi::c_int) as ::core::ffi::c_int
                    != 0
            {
                p = p.offset(
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as usize)
                        as isize,
                );
                messages_history_new = getdigits_int(&raw mut p, false_0 != 0, INT_MAX);
                messages_flags_new |= kOptMoptFlagHistory as ::core::ffi::c_int;
            } else if strnequal(
                p,
                b"progress:\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
            ) {
                p = p.offset(
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as usize)
                        as isize,
                );
                messages_flags_new |= kOptMoptFlagProgress as ::core::ffi::c_int;
                if *p as ::core::ffi::c_int == 'c' as ::core::ffi::c_int {
                    progress_target_flag |= PROGRESS_TARGET_CMD;
                    p = p.offset(1);
                }
            }
            if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != NUL
            {
                return FAIL;
            }
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        }
        if messages_flags_new
            & (kOptMoptFlagHitEnter as ::core::ffi::c_int | kOptMoptFlagWait as ::core::ffi::c_int)
            == 0
        {
            return FAIL;
        }
        if messages_flags_new & kOptMoptFlagHistory as ::core::ffi::c_int == 0 {
            return FAIL;
        }
        '_c2rust_label: {
            if messages_history_new >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"messages_history_new >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1322 as ::core::ffi::c_uint,
                    b"int messagesopt_changed(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if messages_history_new > 10000 as ::core::ffi::c_int {
            return FAIL;
        }
        '_c2rust_label_0: {
            if messages_wait_new >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"messages_wait_new >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1328 as ::core::ffi::c_uint,
                    b"int messagesopt_changed(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if messages_wait_new > 10000 as ::core::ffi::c_int {
            return FAIL;
        }
        msg_flags.set(messages_flags_new);
        msg_wait.set(messages_wait_new);
        progress_msg_target.set(progress_target_flag);
        msg_hist_max.set(messages_history_new);
        msg_hist_clear(msg_hist_max.get());
        return OK;
    }
}

pub unsafe fn ex_messages(mut eap: *mut exarg_T) {
    unsafe {
        if strcmp(
            (*eap).arg,
            b"clear\0".as_ptr() as *const ::core::ffi::c_char,
        ) == 0 as ::core::ffi::c_int
        {
            msg_hist_clear(if (*eap).addr_count != 0 {
                (*eap).line2 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            });
            return;
        }
        if *(*eap).arg as ::core::ffi::c_int != NUL {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let mut entries: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut p: *mut MessageHistoryEntry = if (*eap).skip != 0 {
            msg_hist_temp.get()
        } else {
            msg_hist_first.get()
        };
        let mut skip: ::core::ffi::c_int = if (*eap).addr_count != 0 {
            msg_hist_len.get() - (*eap).line2 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        while !p.is_null() {
            if !((*p).temp as ::core::ffi::c_int != 0 && (*eap).skip == 0 || {
                let c2rust_fresh24 = skip;
                skip = skip - 1;
                c2rust_fresh24 > 0 as ::core::ffi::c_int
            }) {
                if ui_has(kUIMessages) as ::core::ffi::c_int != 0 && msg_silent.get() == 0 {
                    let mut entry: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    if entry.size == entry.capacity {
                        entry.capacity = if entry.capacity != 0 {
                            entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        entry.items = xrealloc(
                            entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh25 = entry.size;
                    entry.size = entry.size.wrapping_add(1);
                    *entry.items.offset(c2rust_fresh25 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_11 {
                            string: cstr_to_string((*p).kind),
                        },
                    };
                    let mut content: Array = Array {
                        size: 0 as size_t,
                        capacity: 0 as size_t,
                        items: ::core::ptr::null_mut::<Object>(),
                    };
                    let mut i: uint32_t = 0 as uint32_t;
                    while (i as size_t) < (*p).msg.size {
                        let mut chunk: HlMessageChunk = *(*p).msg.items.offset(i as isize);
                        let mut content_entry: Array = Array {
                            size: 0 as size_t,
                            capacity: 0 as size_t,
                            items: ::core::ptr::null_mut::<Object>(),
                        };
                        if content_entry.size == content_entry.capacity {
                            content_entry.capacity = if content_entry.capacity != 0 {
                                content_entry.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            content_entry.items = xrealloc(
                                content_entry.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>()
                                    .wrapping_mul(content_entry.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh26 = content_entry.size;
                        content_entry.size = content_entry.size.wrapping_add(1);
                        *content_entry.items.offset(c2rust_fresh26 as isize) = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed_11 {
                                integer: (if chunk.hl_id != 0 {
                                    syn_id2attr(chunk.hl_id)
                                } else {
                                    0 as ::core::ffi::c_int
                                }) as Integer,
                            },
                        };
                        if content_entry.size == content_entry.capacity {
                            content_entry.capacity = if content_entry.capacity != 0 {
                                content_entry.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            content_entry.items = xrealloc(
                                content_entry.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>()
                                    .wrapping_mul(content_entry.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh27 = content_entry.size;
                        content_entry.size = content_entry.size.wrapping_add(1);
                        *content_entry.items.offset(c2rust_fresh27 as isize) = object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed_11 {
                                string: copy_string(chunk.text, ::core::ptr::null_mut::<Arena>()),
                            },
                        };
                        if content_entry.size == content_entry.capacity {
                            content_entry.capacity = if content_entry.capacity != 0 {
                                content_entry.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            content_entry.items = xrealloc(
                                content_entry.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>()
                                    .wrapping_mul(content_entry.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh28 = content_entry.size;
                        content_entry.size = content_entry.size.wrapping_add(1);
                        *content_entry.items.offset(c2rust_fresh28 as isize) = object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed_11 {
                                integer: chunk.hl_id as Integer,
                            },
                        };
                        if content.size == content.capacity {
                            content.capacity = if content.capacity != 0 {
                                content.capacity << 1 as ::core::ffi::c_int
                            } else {
                                8 as size_t
                            };
                            content.items = xrealloc(
                                content.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<Object>().wrapping_mul(content.capacity),
                            ) as *mut Object;
                        } else {
                        };
                        let c2rust_fresh29 = content.size;
                        content.size = content.size.wrapping_add(1);
                        *content.items.offset(c2rust_fresh29 as isize) = object {
                            type_0: kObjectTypeArray,
                            data: C2Rust_Unnamed_11 {
                                array: content_entry,
                            },
                        };
                        i = i.wrapping_add(1);
                    }
                    if entry.size == entry.capacity {
                        entry.capacity = if entry.capacity != 0 {
                            entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        entry.items = xrealloc(
                            entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh30 = entry.size;
                    entry.size = entry.size.wrapping_add(1);
                    *entry.items.offset(c2rust_fresh30 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed_11 { array: content },
                    };
                    if entry.size == entry.capacity {
                        entry.capacity = if entry.capacity != 0 {
                            entry.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        entry.items = xrealloc(
                            entry.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(entry.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh31 = entry.size;
                    entry.size = entry.size.wrapping_add(1);
                    *entry.items.offset(c2rust_fresh31 as isize) = object {
                        type_0: kObjectTypeBoolean,
                        data: C2Rust_Unnamed_11 {
                            boolean: (*p).append,
                        },
                    };
                    if entries.size == entries.capacity {
                        entries.capacity = if entries.capacity != 0 {
                            entries.capacity << 1 as ::core::ffi::c_int
                        } else {
                            8 as size_t
                        };
                        entries.items = xrealloc(
                            entries.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<Object>().wrapping_mul(entries.capacity),
                        ) as *mut Object;
                    } else {
                    };
                    let c2rust_fresh32 = entries.size;
                    entries.size = entries.size.wrapping_add(1);
                    *entries.items.offset(c2rust_fresh32 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed_11 { array: entry },
                    };
                }
                if redirecting() || !ui_has(kUIMessages) {
                    (*msg_silent.ptr()) += ui_has(kUIMessages) as ::core::ffi::c_int;
                    let mut needs_clear: bool = false_0 != 0;
                    msg_multihl(
                        object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed_11 { boolean: false },
                        },
                        (*p).msg,
                        (*p).kind,
                        false_0 != 0,
                        false_0 != 0,
                        ::core::ptr::null_mut::<MessageData>(),
                        &raw mut needs_clear,
                    );
                    (*msg_silent.ptr()) -= ui_has(kUIMessages) as ::core::ffi::c_int;
                }
            }
            p = (*p).next as *mut MessageHistoryEntry;
        }
        if entries.size > 0 as size_t {
            ui_call_msg_history_show(entries, (*eap).skip != 0 as ::core::ffi::c_int);
            api_free_array(entries);
        }
    }
}
