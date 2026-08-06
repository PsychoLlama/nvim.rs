//! `nvim_parse_expression()`: an expression as an AST.
//!
//! The parser's whole output rendered as data: the length consumed, the error
//! if the parse failed, the highlight chunks when `hl` is set, and the AST
//! itself -- walked iteratively with an explicit stack rather than recursively,
//! because an expression nests arbitrarily deep.  `ast_conv_stack_grow` is that
//! stack's growth step, which c2rust had expanded inline at each of its three
//! push sites.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_parse_expression(
    mut expr: String_0,
    mut flags: String_0,
    mut hl: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Dict {
    unsafe {
        let mut pflags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: size_t = 0 as size_t;
        while i < flags.size {
            match *flags.data.offset(i as isize) as ::core::ffi::c_int {
                109 => {
                    pflags |= kExprFlagsMulti as ::core::ffi::c_int;
                }
                69 => {
                    pflags |= kExprFlagsDisallowEOC as ::core::ffi::c_int;
                }
                108 => {
                    pflags |= kExprFlagsParseLet as ::core::ffi::c_int;
                }
                NUL => {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"Invalid flag: '\\0' (%u)\0".as_ptr() as *const ::core::ffi::c_char,
                        *flags.data.offset(i as isize) as ::core::ffi::c_uint,
                    );
                    return ARRAY_DICT_INIT;
                }
                _ => {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"Invalid flag: '%c' (%u)".as_ptr(),
                        *flags.data.offset(i as isize) as ::core::ffi::c_int,
                        *flags.data.offset(i as isize) as ::core::ffi::c_uint,
                    );
                    return ARRAY_DICT_INIT;
                }
            }
            i = i.wrapping_add(1);
        }
        let mut parser_lines: [ParserLine; 2] = [
            ParserLine {
                data: expr.data,
                size: expr.size,
                allocated: false,
            },
            ParserLine {
                data: ::core::ptr::null::<::core::ffi::c_char>(),
                size: 0 as size_t,
                allocated: false,
            },
        ];
        let mut plines_p: *mut ParserLine = &raw mut parser_lines as *mut ParserLine;
        let mut colors: ParserHighlight = ParserHighlight {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ParserHighlightChunk>(),
            init_array: [ParserHighlightChunk {
                start: ParserPosition { line: 0, col: 0 },
                end_col: 0,
                group: ::core::ptr::null::<::core::ffi::c_char>(),
            }; 16],
        };
        colors.capacity = colors.init_array.len();
        colors.items = colors.init_array.as_mut_ptr();
        let colors_p: *mut ParserHighlight = if hl as ::core::ffi::c_int != 0 {
            &raw mut colors
        } else {
            ::core::ptr::null_mut::<ParserHighlight>()
        };
        let mut pstate: ParserState = ::core::mem::zeroed();
        viml_parser_init(
            &raw mut pstate,
            Some(parser_simple_get_line),
            &raw mut plines_p as *mut ::core::ffi::c_void,
            colors_p,
        );
        let mut east: ExprAST = viml_pexpr_parse(&raw mut pstate, pflags);
        let ret_size: size_t = (2 as size_t)
            .wrapping_add(!east.err.msg.is_null() as ::core::ffi::c_int as size_t)
            .wrapping_add(hl as size_t)
            .wrapping_add(0 as size_t);
        let mut ret: Dict = arena_dict(arena, ret_size);
        let c2rust_fresh1 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh1 as isize) = key_value_pair {
            key: cstr_as_string(c"len".as_ptr()),
            value: object {
                type_0: kObjectTypeInteger,
                data: C2Rust_Unnamed {
                    integer: (if pstate.pos.line == 1 as size_t {
                        parser_lines[0 as ::core::ffi::c_int as usize].size
                    } else {
                        pstate.pos.col
                    }) as Integer,
                },
            },
        };
        if !east.err.msg.is_null() {
            let mut err_dict: Dict = arena_dict(arena, 2 as size_t);
            let c2rust_fresh2 = err_dict.size;
            err_dict.size = err_dict.size.wrapping_add(1);
            *err_dict.items.offset(c2rust_fresh2 as isize) = key_value_pair {
                key: cstr_as_string(c"message".as_ptr()),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_string(arena, cstr_as_string(east.err.msg)),
                    },
                },
            };
            let c2rust_fresh3 = err_dict.size;
            err_dict.size = err_dict.size.wrapping_add(1);
            *err_dict.items.offset(c2rust_fresh3 as isize) = key_value_pair {
                key: cstr_as_string(c"arg".as_ptr()),
                value: object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: arena_string(
                            arena,
                            String_0 {
                                data: east.err.arg as *mut ::core::ffi::c_char,
                                size: east.err.arg_len as size_t,
                            },
                        ),
                    },
                },
            };
            let c2rust_fresh4 = ret.size;
            ret.size = ret.size.wrapping_add(1);
            *ret.items.offset(c2rust_fresh4 as isize) = key_value_pair {
                key: cstr_as_string(c"error".as_ptr()),
                value: object {
                    type_0: kObjectTypeDict,
                    data: C2Rust_Unnamed { dict: err_dict },
                },
            };
        }
        if hl {
            let mut hl_arr: Array = arena_array(arena, colors.size);
            let mut i_0: size_t = 0 as size_t;
            while i_0 < colors.size {
                let chunk: ParserHighlightChunk = *colors.items.offset(i_0 as isize);
                let mut chunk_arr: Array = arena_array(arena, 4 as size_t);
                let c2rust_fresh5 = chunk_arr.size;
                chunk_arr.size = chunk_arr.size.wrapping_add(1);
                *chunk_arr.items.offset(c2rust_fresh5 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: chunk.start.line as Integer,
                    },
                };
                let c2rust_fresh6 = chunk_arr.size;
                chunk_arr.size = chunk_arr.size.wrapping_add(1);
                *chunk_arr.items.offset(c2rust_fresh6 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: chunk.start.col as Integer,
                    },
                };
                let c2rust_fresh7 = chunk_arr.size;
                chunk_arr.size = chunk_arr.size.wrapping_add(1);
                *chunk_arr.items.offset(c2rust_fresh7 as isize) = object {
                    type_0: kObjectTypeInteger,
                    data: C2Rust_Unnamed {
                        integer: chunk.end_col as Integer,
                    },
                };
                let c2rust_fresh8 = chunk_arr.size;
                chunk_arr.size = chunk_arr.size.wrapping_add(1);
                *chunk_arr.items.offset(c2rust_fresh8 as isize) = object {
                    type_0: kObjectTypeString,
                    data: C2Rust_Unnamed {
                        string: cstr_as_string(chunk.group),
                    },
                };
                let c2rust_fresh9 = hl_arr.size;
                hl_arr.size = hl_arr.size.wrapping_add(1);
                *hl_arr.items.offset(c2rust_fresh9 as isize) = object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: chunk_arr },
                };
                i_0 = i_0.wrapping_add(1);
            }
            let c2rust_fresh10 = ret.size;
            ret.size = ret.size.wrapping_add(1);
            *ret.items.offset(c2rust_fresh10 as isize) = key_value_pair {
                key: cstr_as_string(c"highlight".as_ptr()),
                value: object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed { array: hl_arr },
                },
            };
        }
        if colors.items != &raw mut colors.init_array as *mut ParserHighlightChunk {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut colors.items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        }
        let mut ast_conv_stack: ExprASTConvStack = ExprASTConvStack {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<ExprASTConvStackItem>(),
            init_array: [ExprASTConvStackItem {
                node_p: ::core::ptr::null_mut::<*mut ExprASTNode>(),
                ret_node_p: ::core::ptr::null_mut::<Object>(),
            }; 16],
        };
        ast_conv_stack.capacity = ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
            .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
            .wrapping_div(
                (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                    .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        ast_conv_stack.size = 0 as size_t;
        ast_conv_stack.items = &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem;
        let mut ast: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        ast_conv_stack_grow(&raw mut ast_conv_stack);
        let c2rust_fresh11 = ast_conv_stack.size;
        ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
        *ast_conv_stack.items.offset(c2rust_fresh11 as isize) = ExprASTConvStackItem {
            node_p: &raw mut east.root,
            ret_node_p: &raw mut ast,
        };
        while ast_conv_stack.size != 0 {
            let mut cur_item: ExprASTConvStackItem = *ast_conv_stack.items.offset(
                ast_conv_stack
                    .size
                    .wrapping_sub(0 as size_t)
                    .wrapping_sub(1 as size_t) as isize,
            );
            let node: *mut ExprASTNode = *cur_item.node_p;
            if node.is_null() {
                '_c2rust_label: {
                    if ast_conv_stack.size == 1 as size_t {
                    } else {
                        __assert_fail(
                        c"kv_size(ast_conv_stack) == 1".as_ptr(),
                        c"src/nvim/api/vimscript.rs".as_ptr(),
                        511 as ::core::ffi::c_uint,
                        c"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)"
                            .as_ptr(),
                    );
                    }
                };
                ast_conv_stack.size = ast_conv_stack.size.wrapping_sub(1 as size_t);
            } else {
                if (*cur_item.ret_node_p).type_0 as ::core::ffi::c_uint
                    == kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut items_size: size_t = (3 as ::core::ffi::c_int
                        + !(*node).children.is_null() as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeOption as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*node).type_0 as ::core::ffi::c_uint
                                == kExprNodePlainIdentifier as ::core::ffi::c_int
                                    as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeOption as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*node).type_0 as ::core::ffi::c_uint
                                == kExprNodePlainIdentifier as ::core::ffi::c_int
                                    as ::core::ffi::c_uint
                            || (*node).type_0 as ::core::ffi::c_uint
                                == kExprNodePlainKey as ::core::ffi::c_int as ::core::ffi::c_uint
                            || (*node).type_0 as ::core::ffi::c_uint
                                == kExprNodeEnvironment as ::core::ffi::c_int
                                    as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeRegister as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + 3 as ::core::ffi::c_int
                            * ((*node).type_0 as ::core::ffi::c_uint
                                == kExprNodeComparison as ::core::ffi::c_int as ::core::ffi::c_uint)
                                as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeInteger as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeFloat as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeDoubleQuotedString as ::core::ffi::c_int
                                as ::core::ffi::c_uint
                            || (*node).type_0 as ::core::ffi::c_uint
                                == kExprNodeSingleQuotedString as ::core::ffi::c_int
                                    as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + ((*node).type_0 as ::core::ffi::c_uint
                            == kExprNodeAssignment as ::core::ffi::c_int as ::core::ffi::c_uint)
                            as ::core::ffi::c_int
                        + 0 as ::core::ffi::c_int)
                        as size_t;
                    let mut ret_node: Dict = arena_dict(arena, items_size);
                    *cur_item.ret_node_p = object {
                        type_0: kObjectTypeDict,
                        data: C2Rust_Unnamed { dict: ret_node },
                    };
                }
                let mut ret_node_0: *mut Dict = &raw mut (*cur_item.ret_node_p).data.dict;
                if !(*node).children.is_null() {
                    let num_children: size_t = (1 as ::core::ffi::c_int
                        + !(*(*node).children).next.is_null() as ::core::ffi::c_int)
                        as size_t;
                    let mut children_array: Array = arena_array(arena, num_children);
                    let mut i_1: size_t = 0 as size_t;
                    while i_1 < num_children {
                        let c2rust_fresh12 = children_array.size;
                        children_array.size = children_array.size.wrapping_add(1);
                        *children_array.items.offset(c2rust_fresh12 as isize) = object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed { boolean: false },
                        };
                        i_1 = i_1.wrapping_add(1);
                    }
                    let c2rust_fresh13 = (*ret_node_0).size;
                    (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                    *(*ret_node_0).items.offset(c2rust_fresh13 as isize) = key_value_pair {
                        key: cstr_as_string(c"children".as_ptr()),
                        value: object {
                            type_0: kObjectTypeArray,
                            data: C2Rust_Unnamed {
                                array: children_array,
                            },
                        },
                    };
                    ast_conv_stack_grow(&raw mut ast_conv_stack);
                    let c2rust_fresh14 = ast_conv_stack.size;
                    ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
                    *ast_conv_stack.items.offset(c2rust_fresh14 as isize) = ExprASTConvStackItem {
                        node_p: &raw mut (*node).children,
                        ret_node_p: children_array
                            .items
                            .offset(0 as ::core::ffi::c_int as isize),
                    };
                } else if !(*node).next.is_null() {
                    ast_conv_stack_grow(&raw mut ast_conv_stack);
                    let c2rust_fresh15 = ast_conv_stack.size;
                    ast_conv_stack.size = ast_conv_stack.size.wrapping_add(1);
                    *ast_conv_stack.items.offset(c2rust_fresh15 as isize) = ExprASTConvStackItem {
                        node_p: &raw mut (*node).next,
                        ret_node_p: cur_item.ret_node_p.offset(1 as ::core::ffi::c_int as isize),
                    };
                } else {
                    ast_conv_stack.size = ast_conv_stack.size.wrapping_sub(1 as size_t);
                    let c2rust_fresh16 = (*ret_node_0).size;
                    (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                    *(*ret_node_0).items.offset(c2rust_fresh16 as isize) = key_value_pair {
                        key: cstr_as_string(c"type".as_ptr()),
                        value: object {
                            type_0: kObjectTypeString,
                            data: C2Rust_Unnamed {
                                string: cstr_as_string(
                                    *(&raw const east_node_type_tab
                                        as *const *const ::core::ffi::c_char)
                                        .offset((*node).type_0 as isize),
                                ),
                            },
                        },
                    };
                    let mut start_array: Array = arena_array(arena, 2 as size_t);
                    let c2rust_fresh17 = start_array.size;
                    start_array.size = start_array.size.wrapping_add(1);
                    *start_array.items.offset(c2rust_fresh17 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*node).start.line as Integer,
                        },
                    };
                    let c2rust_fresh18 = start_array.size;
                    start_array.size = start_array.size.wrapping_add(1);
                    *start_array.items.offset(c2rust_fresh18 as isize) = object {
                        type_0: kObjectTypeInteger,
                        data: C2Rust_Unnamed {
                            integer: (*node).start.col as Integer,
                        },
                    };
                    let c2rust_fresh19 = (*ret_node_0).size;
                    (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                    *(*ret_node_0).items.offset(c2rust_fresh19 as isize) = key_value_pair {
                        key: cstr_as_string(c"start".as_ptr()),
                        value: object {
                            type_0: kObjectTypeArray,
                            data: C2Rust_Unnamed { array: start_array },
                        },
                    };
                    let c2rust_fresh20 = (*ret_node_0).size;
                    (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                    *(*ret_node_0).items.offset(c2rust_fresh20 as isize) = key_value_pair {
                        key: cstr_as_string(c"len".as_ptr()),
                        value: object {
                            type_0: kObjectTypeInteger,
                            data: C2Rust_Unnamed {
                                integer: (*node).len as Integer,
                            },
                        },
                    };
                    match (*node).type_0 as ::core::ffi::c_uint {
                        27 | 26 => {
                            let mut str: Object = object {
                                type_0: kObjectTypeString,
                                data: C2Rust_Unnamed {
                                    string: arena_string(
                                        arena,
                                        String_0 {
                                            data: (*node).data.str.value,
                                            size: (*node).data.str.size,
                                        },
                                    ),
                                },
                            };
                            let c2rust_fresh21 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh21 as isize) = key_value_pair {
                                key: cstr_as_string(c"svalue".as_ptr()),
                                value: str,
                            };
                            xfree((*node).data.str.value as *mut ::core::ffi::c_void);
                        }
                        36 => {
                            let c2rust_fresh22 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh22 as isize) = key_value_pair {
                                key: cstr_as_string(c"scope".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeInteger,
                                    data: C2Rust_Unnamed {
                                        integer: (*node).data.opt.scope as Integer,
                                    },
                                },
                            };
                            let c2rust_fresh23 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh23 as isize) = key_value_pair {
                                key: cstr_as_string(c"ident".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: arena_string(
                                            arena,
                                            String_0 {
                                                data: (*node).data.opt.ident
                                                    as *mut ::core::ffi::c_char,
                                                size: (*node).data.opt.ident_len,
                                            },
                                        ),
                                    },
                                },
                            };
                        }
                        11 => {
                            let c2rust_fresh24 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh24 as isize) = key_value_pair {
                                key: cstr_as_string(c"scope".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeInteger,
                                    data: C2Rust_Unnamed {
                                        integer: (*node).data.var.scope as Integer,
                                    },
                                },
                            };
                            let c2rust_fresh25 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh25 as isize) = key_value_pair {
                                key: cstr_as_string(c"ident".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: arena_string(
                                            arena,
                                            String_0 {
                                                data: (*node).data.var.ident
                                                    as *mut ::core::ffi::c_char,
                                                size: (*node).data.var.ident_len,
                                            },
                                        ),
                                    },
                                },
                            };
                        }
                        12 => {
                            let c2rust_fresh26 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh26 as isize) = key_value_pair {
                                key: cstr_as_string(c"ident".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: arena_string(
                                            arena,
                                            String_0 {
                                                data: (*node).data.var.ident
                                                    as *mut ::core::ffi::c_char,
                                                size: (*node).data.var.ident_len,
                                            },
                                        ),
                                    },
                                },
                            };
                        }
                        37 => {
                            let c2rust_fresh27 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh27 as isize) = key_value_pair {
                                key: cstr_as_string(c"ident".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: arena_string(
                                            arena,
                                            String_0 {
                                                data: (*node).data.env.ident
                                                    as *mut ::core::ffi::c_char,
                                                size: (*node).data.env.ident_len,
                                            },
                                        ),
                                    },
                                },
                            };
                        }
                        4 => {
                            let c2rust_fresh28 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh28 as isize) = key_value_pair {
                                key: cstr_as_string(c"name".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeInteger,
                                    data: C2Rust_Unnamed {
                                        integer: (*node).data.reg.name as Integer,
                                    },
                                },
                            };
                        }
                        21 => {
                            let c2rust_fresh29 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh29 as isize) = key_value_pair {
                                key: cstr_as_string(c"cmp_type".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: cstr_as_string(
                                            *(&raw const eltkn_cmp_type_tab
                                                as *const *const ::core::ffi::c_char)
                                                .offset((*node).data.cmp.type_0 as isize),
                                        ),
                                    },
                                },
                            };
                            let c2rust_fresh30 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh30 as isize) = key_value_pair {
                                key: cstr_as_string(c"ccs_strategy".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed {
                                        string: cstr_as_string(
                                            *(&raw const ccs_tab
                                                as *const *const ::core::ffi::c_char)
                                                .offset((*node).data.cmp.ccs as isize),
                                        ),
                                    },
                                },
                            };
                            let c2rust_fresh31 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh31 as isize) = key_value_pair {
                                key: cstr_as_string(c"invert".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeBoolean,
                                    data: C2Rust_Unnamed {
                                        boolean: (*node).data.cmp.inv,
                                    },
                                },
                            };
                        }
                        25 => {
                            let c2rust_fresh32 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh32 as isize) = key_value_pair {
                                key: cstr_as_string(c"fvalue".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeFloat,
                                    data: C2Rust_Unnamed {
                                        floating: (*node).data.flt.value,
                                    },
                                },
                            };
                        }
                        24 => {
                            let c2rust_fresh33 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh33 as isize) = key_value_pair {
                                key: cstr_as_string(c"ivalue".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeInteger,
                                    data: C2Rust_Unnamed {
                                        integer: if (*node).data.num.value
                                            > 9223372036854775807 as uvarnumber_T
                                        {
                                            9223372036854775807 as Integer
                                        } else {
                                            (*node).data.num.value as Integer
                                        },
                                    },
                                },
                            };
                        }
                        38 => {
                            let asgn_type: ExprAssignmentType = (*node).data.ass.type_0;
                            let mut str_0: String_0 = if asgn_type as ::core::ffi::c_uint
                                == kExprAsgnPlain as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                STRING_INIT
                            } else {
                                cstr_as_string(
                                    *(&raw const expr_asgn_type_tab
                                        as *const *const ::core::ffi::c_char)
                                        .offset(asgn_type as isize),
                                )
                            };
                            let c2rust_fresh34 = (*ret_node_0).size;
                            (*ret_node_0).size = (*ret_node_0).size.wrapping_add(1);
                            *(*ret_node_0).items.offset(c2rust_fresh34 as isize) = key_value_pair {
                                key: cstr_as_string(c"augmentation".as_ptr()),
                                value: object {
                                    type_0: kObjectTypeString,
                                    data: C2Rust_Unnamed { string: str_0 },
                                },
                            };
                        }
                        0 | 1 | 2 | 3 | 5 | 6 | 7 | 8 | 9 | 10 | 13 | 14 | 15 | 16 | 17 | 18
                        | 19 | 20 | 22 | 23 | 28 | 29 | 30 | 31 | 32 | 33 | 34 | 35 | _ => {}
                    }
                    '_c2rust_label_0: {
                        if (*cur_item.ret_node_p).data.dict.size
                            == (*cur_item.ret_node_p).data.dict.capacity
                        {
                        } else {
                            __assert_fail(
                            c"cur_item.ret_node_p->data.dict.size == cur_item.ret_node_p->data.dict.capacity".as_ptr(),
                            c"src/nvim/api/vimscript.rs".as_ptr(),
                            640 as ::core::ffi::c_uint,
                            c"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)".as_ptr(),
                        );
                        }
                    };
                    xfree(*cur_item.node_p as *mut ::core::ffi::c_void);
                    *cur_item.node_p = ::core::ptr::null_mut::<ExprASTNode>();
                }
            }
        }
        if ast_conv_stack.items != &raw mut ast_conv_stack.init_array as *mut ExprASTConvStackItem {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut ast_conv_stack.items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
        }
        let c2rust_fresh35 = ret.size;
        ret.size = ret.size.wrapping_add(1);
        *ret.items.offset(c2rust_fresh35 as isize) = key_value_pair {
            key: cstr_as_string(c"ast".as_ptr()),
            value: ast,
        };
        '_c2rust_label_1: {
            if ret.size == ret.capacity {
            } else {
                __assert_fail(
                    c"ret.size == ret.capacity".as_ptr(),
                    c"src/nvim/api/vimscript.rs".as_ptr(),
                    649 as ::core::ffi::c_uint,
                    c"Dict nvim_parse_expression(String, String, Boolean, Arena *, Error *)"
                        .as_ptr(),
                );
            }
        };
        viml_pexpr_free_ast(east);
        viml_parser_destroy(&mut pstate);
        return ret;
    }
}

/// One `kvi_push` growth step for a `ExprASTConvStack`, which c2rust expanded
/// inline at each of its 3 call sites.
unsafe fn ast_conv_stack_grow(kv: *mut ExprASTConvStack) {
    unsafe {
        if (*kv).size == (*kv).capacity {
            (*kv).capacity = if (*kv).capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                    .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*kv).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                    .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*kv).items = (if (*kv).capacity
                == ::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                    .wrapping_div(::core::mem::size_of::<ExprASTConvStackItem>())
                    .wrapping_div(
                        (::core::mem::size_of::<[ExprASTConvStackItem; 16]>()
                            .wrapping_rem(::core::mem::size_of::<ExprASTConvStackItem>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*kv).items == &raw mut (*kv).init_array as *mut ExprASTConvStackItem {
                    (*kv).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*kv).init_array as *mut ExprASTConvStackItem
                            as *mut ::core::ffi::c_void,
                        (*kv).items as *mut ::core::ffi::c_void,
                        (*kv)
                            .size
                            .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                    )
                }
            } else {
                if (*kv).items == &raw mut (*kv).init_array as *mut ExprASTConvStackItem {
                    memcpy(
                        xmalloc(
                            (*kv)
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                        ),
                        (*kv).items as *const ::core::ffi::c_void,
                        (*kv)
                            .size
                            .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                    )
                } else {
                    xrealloc(
                        (*kv).items as *mut ::core::ffi::c_void,
                        (*kv)
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<ExprASTConvStackItem>()),
                    )
                }
            }) as *mut ExprASTConvStackItem;
        } else {
        };
    }
}
