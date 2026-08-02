//! A decoration as an API dictionary.
//!
//! [`decor_to_dict_legacy`] is what `nvim_buf_get_extmark*(details = true)`
//! answers with: one flat dictionary carrying whichever of the virt-text,
//! virt-lines, sign and highlight parts the decoration has. It assumes at
//! most one of each kind, which is not always true — the name says
//! "legacy" for that reason.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn decor_to_dict_legacy(
    mut dict: *mut Dict,
    mut decor: DecorInline,
    mut hl_name: bool,
    mut arena: *mut Arena,
) {
    unsafe {
        let mut sh_hl: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
        let mut sh_sign: DecorSignHighlight = DECOR_SIGN_HIGHLIGHT_INIT;
        let mut virt_text: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
        let mut virt_lines: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
        let mut priority: int32_t = -1 as int32_t;
        if decor.ext {
            let mut vt: *mut DecorVirtText = decor.data.ext.vt;
            while !vt.is_null() {
                if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                    virt_lines = vt;
                } else {
                    virt_text = vt;
                }
                vt = (*vt).next;
            }
            let mut idx: uint32_t = decor.data.ext.sh_idx;
            while idx != DECOR_ID_INVALID as uint32_t {
                let mut sh: *mut DecorSignHighlight =
                    (*decor_items.ptr()).items.offset(idx as isize);
                if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                    sh_sign = *sh;
                } else {
                    sh_hl = *sh;
                }
                idx = (*sh).next;
            }
        } else {
            sh_hl = decor_sh_from_inline(decor.data.hl);
        }
        if sh_hl.hl_id != 0 {
            let c2rust_fresh8 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh8 as isize) = key_value_pair {
                key: cstr_as_string(b"hl_group\0".as_ptr() as *const ::core::ffi::c_char),
                value: hl_group_name(sh_hl.hl_id, hl_name),
            };
            let c2rust_fresh9 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh9 as isize) = key_value_pair {
                key: cstr_as_string(b"hl_eol\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 {
                        boolean: sh_hl.flags as ::core::ffi::c_int & kSHHlEol as ::core::ffi::c_int
                            != 0,
                    },
                },
            };
            priority = sh_hl.priority as int32_t;
        }
        if sh_hl.flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int != 0 {
            let mut buf: [::core::ffi::c_char; 32] = [0; 32];
            schar_get(
                &raw mut buf as *mut ::core::ffi::c_char,
                sh_hl.text[0 as ::core::ffi::c_int as usize],
            );
            let c2rust_fresh10 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh10 as isize) = key_value_pair {
                key: cstr_as_string(b"conceal\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: arena_string(
                            arena,
                            cstr_as_string(&raw mut buf as *mut ::core::ffi::c_char),
                        ),
                    },
                },
            };
        }
        if sh_hl.flags as ::core::ffi::c_int & kSHConcealLines as ::core::ffi::c_int != 0 {
            let c2rust_fresh11 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh11 as isize) = key_value_pair {
                key: cstr_as_string(b"conceal_lines\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(b"\0".as_ptr() as *const ::core::ffi::c_char),
                    },
                },
            };
        }
        if sh_hl.flags as ::core::ffi::c_int & kSHSpellOn as ::core::ffi::c_int != 0 {
            let c2rust_fresh12 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh12 as isize) = key_value_pair {
                key: cstr_as_string(b"spell\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 { boolean: true },
                },
            };
        } else if sh_hl.flags as ::core::ffi::c_int & kSHSpellOff as ::core::ffi::c_int != 0 {
            let c2rust_fresh13 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh13 as isize) = key_value_pair {
                key: cstr_as_string(b"spell\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 { boolean: false },
                },
            };
        }
        if sh_hl.flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
            let c2rust_fresh14 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh14 as isize) = key_value_pair {
                key: cstr_as_string(b"ui_watched\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 { boolean: true },
                },
            };
        }
        if !sh_hl.url.is_null() {
            let c2rust_fresh15 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh15 as isize) = key_value_pair {
                key: cstr_as_string(b"url\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(sh_hl.url),
                    },
                },
            };
        }
        if !virt_text.is_null() {
            if (*virt_text).hl_mode != 0 {
                let c2rust_fresh16 = (*dict).size;
                (*dict).size = (*dict).size.wrapping_add(1);
                *(*dict).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                    key: cstr_as_string(b"hl_mode\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_14 {
                            string: cstr_as_string(
                                *(&raw const hl_mode_str as *const *const ::core::ffi::c_char)
                                    .offset((*virt_text).hl_mode as isize),
                            ),
                        },
                    },
                };
            }
            let mut chunks: Array = virt_text_to_array((*virt_text).data.virt_text, hl_name, arena);
            let c2rust_fresh17 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh17 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_text\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_14 { array: chunks },
                },
            };
            let c2rust_fresh18 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh18 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_text_hide\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 {
                        boolean: (*virt_text).flags as ::core::ffi::c_int
                            & kVTHide as ::core::ffi::c_int
                            != 0,
                    },
                },
            };
            let c2rust_fresh19 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh19 as isize) = key_value_pair {
                key: cstr_as_string(
                    b"virt_text_repeat_linebreak\0".as_ptr() as *const ::core::ffi::c_char
                ),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 {
                        boolean: (*virt_text).flags as ::core::ffi::c_int
                            & kVTRepeatLinebreak as ::core::ffi::c_int
                            != 0,
                    },
                },
            };
            if (*virt_text).pos as ::core::ffi::c_uint
                == kVPosWinCol as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let c2rust_fresh20 = (*dict).size;
                (*dict).size = (*dict).size.wrapping_add(1);
                *(*dict).items.offset(c2rust_fresh20 as isize) = key_value_pair {
                    key: cstr_as_string(
                        b"virt_text_win_col\0".as_ptr() as *const ::core::ffi::c_char
                    ),
                    value: object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed_14 {
                            integer: (*virt_text).col as Integer,
                        },
                    },
                };
            }
            let c2rust_fresh21 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh21 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_text_pos\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(
                            *(&raw const virt_text_pos_str as *const *const ::core::ffi::c_char)
                                .offset((*virt_text).pos as isize),
                        ),
                    },
                },
            };
            priority = (*virt_text).priority as int32_t;
        }
        if !virt_lines.is_null() {
            let mut all_chunks: Array = arena_array(arena, (*virt_lines).data.virt_lines.size);
            let mut virt_lines_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut i: size_t = 0 as size_t;
            while i < (*virt_lines).data.virt_lines.size {
                virt_lines_flags = (*(*virt_lines).data.virt_lines.items.offset(i as isize)).flags;
                let mut chunks_0: Array = virt_text_to_array(
                    (*(*virt_lines).data.virt_lines.items.offset(i as isize)).line,
                    hl_name,
                    arena,
                );
                if all_chunks.size == all_chunks.capacity {
                    all_chunks.capacity = if all_chunks.capacity != 0 {
                        all_chunks.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    all_chunks.items = xrealloc(
                        all_chunks.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(all_chunks.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh22 = all_chunks.size;
                all_chunks.size = all_chunks.size.wrapping_add(1);
                *all_chunks.items.offset(c2rust_fresh22 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_14 { array: chunks_0 },
                };
                i = i.wrapping_add(1);
            }
            let c2rust_fresh23 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh23 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_lines\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_14 { array: all_chunks },
                },
            };
            let c2rust_fresh24 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh24 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_lines_above\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 {
                        boolean: (*virt_lines).flags as ::core::ffi::c_int
                            & kVTLinesAbove as ::core::ffi::c_int
                            != 0,
                    },
                },
            };
            let c2rust_fresh25 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh25 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_lines_leftcol\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_14 {
                        boolean: virt_lines_flags & kVLLeftcol as ::core::ffi::c_int != 0,
                    },
                },
            };
            let c2rust_fresh26 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh26 as isize) = key_value_pair {
                key: cstr_as_string(b"virt_lines_overflow\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed_14 {
                        string: cstr_as_string(
                            if virt_lines_flags & kVLScroll as ::core::ffi::c_int != 0 {
                                b"scroll\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"trunc\0".as_ptr() as *const ::core::ffi::c_char
                            },
                        ),
                    },
                },
            };
            priority = (*virt_lines).priority as int32_t;
        }
        if sh_sign.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            if sh_sign.text[0 as ::core::ffi::c_int as usize] != 0 {
                let mut buf_0: [::core::ffi::c_char; 64] = [0; 64];
                describe_sign_text(
                    &raw mut buf_0 as *mut ::core::ffi::c_char,
                    &raw mut sh_sign.text as *mut schar_T,
                );
                let c2rust_fresh27 = (*dict).size;
                (*dict).size = (*dict).size.wrapping_add(1);
                *(*dict).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                    key: cstr_as_string(b"sign_text\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_14 {
                            string: arena_string(
                                arena,
                                cstr_as_string(&raw mut buf_0 as *mut ::core::ffi::c_char),
                            ),
                        },
                    },
                };
            }
            if !sh_sign.sign_name.is_null() {
                let c2rust_fresh28 = (*dict).size;
                (*dict).size = (*dict).size.wrapping_add(1);
                *(*dict).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                    key: cstr_as_string(b"sign_name\0".as_ptr() as *const ::core::ffi::c_char),
                    value: object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_14 {
                            string: cstr_as_string(sh_sign.sign_name),
                        },
                    },
                };
            }
            let mut hls: [C2Rust_Unnamed_28; 5] = [
                C2Rust_Unnamed_28 {
                    name: b"sign_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    val: sh_sign.hl_id,
                },
                C2Rust_Unnamed_28 {
                    name: b"number_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    val: sh_sign.number_hl_id,
                },
                C2Rust_Unnamed_28 {
                    name: b"line_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    val: sh_sign.line_hl_id,
                },
                C2Rust_Unnamed_28 {
                    name: b"cursorline_hl_group\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    val: sh_sign.cursorline_hl_id,
                },
                C2Rust_Unnamed_28 {
                    name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    val: 0 as ::core::ffi::c_int,
                },
            ];
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while !hls[j as usize].name.is_null() {
                if hls[j as usize].val != 0 {
                    let c2rust_fresh29 = (*dict).size;
                    (*dict).size = (*dict).size.wrapping_add(1);
                    *(*dict).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                        key: cstr_as_string(hls[j as usize].name),
                        value: hl_group_name(hls[j as usize].val, hl_name),
                    };
                }
                j += 1;
            }
            priority = sh_sign.priority as int32_t;
        }
        if priority != -1 as int32_t {
            let c2rust_fresh30 = (*dict).size;
            (*dict).size = (*dict).size.wrapping_add(1);
            *(*dict).items.offset(c2rust_fresh30 as isize) = key_value_pair {
                key: cstr_as_string(b"priority\0".as_ptr() as *const ::core::ffi::c_char),
                value: object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed_14 {
                        integer: priority as Integer,
                    },
                },
            };
        }
    }
}

pub unsafe extern "C" fn decor_type_flags(mut decor: DecorInline) -> uint16_t {
    unsafe {
        if decor.ext {
            let mut type_flags: uint16_t = kExtmarkNone as ::core::ffi::c_int as uint16_t;
            let mut vt: *mut DecorVirtText = decor.data.ext.vt;
            while !vt.is_null() {
                type_flags = (type_flags as ::core::ffi::c_int
                    | if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                        kExtmarkVirtLines as ::core::ffi::c_int
                    } else {
                        kExtmarkVirtText as ::core::ffi::c_int
                    }) as uint16_t;
                vt = (*vt).next;
            }
            let mut idx: uint32_t = decor.data.ext.sh_idx;
            while idx != DECOR_ID_INVALID as uint32_t {
                let mut sh: *mut DecorSignHighlight =
                    (*decor_items.ptr()).items.offset(idx as isize);
                type_flags = (type_flags as ::core::ffi::c_int
                    | if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                        kExtmarkSign as ::core::ffi::c_int
                    } else {
                        kExtmarkHighlight as ::core::ffi::c_int
                    }) as uint16_t;
                idx = (*sh).next;
            }
            return type_flags;
        } else {
            return (if decor.data.hl.flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int
                != 0
            {
                kExtmarkSign as ::core::ffi::c_int
            } else {
                kExtmarkHighlight as ::core::ffi::c_int
            }) as uint16_t;
        };
    }
}

pub unsafe extern "C" fn hl_group_name(mut hl_id: ::core::ffi::c_int, mut hl_name: bool) -> Object {
    unsafe {
        if hl_name {
            return object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_14 {
                    string: cstr_as_string(syn_id2name(hl_id)),
                },
            };
        } else {
            return object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed_14 {
                    integer: hl_id as Integer,
                },
            };
        };
    }
}
