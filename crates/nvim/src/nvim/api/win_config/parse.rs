//! Decoding the config keyset into a `WinConfig`.
//!
//! `parse_win_config` is the whole validation surface: which keys may appear
//! together, which are floats-only, which need a window or buffer handle, and
//! what each one's range is.  The small `parse_*` helpers are the individual
//! enumerated fields -- the anchor, the `relative` kind, the split direction,
//! the `bufpos` pair and the border title/footer with its position.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn parse_float_anchor(mut anchor: String_0, mut out: *mut FloatAnchor) -> bool {
    unsafe {
        if anchor.size == 0 as size_t {
            *out = 0 as ::core::ffi::c_int;
        }
        let mut str: *mut ::core::ffi::c_char = anchor.data;
        if striequal(str, c"NW".as_ptr()) {
            *out = 0 as ::core::ffi::c_int as FloatAnchor;
        } else if striequal(str, c"NE".as_ptr()) {
            *out = kFloatAnchorEast as ::core::ffi::c_int as FloatAnchor;
        } else if striequal(str, c"SW".as_ptr()) {
            *out = kFloatAnchorSouth as ::core::ffi::c_int as FloatAnchor;
        } else if striequal(str, c"SE".as_ptr()) {
            *out = (kFloatAnchorSouth as ::core::ffi::c_int
                | kFloatAnchorEast as ::core::ffi::c_int) as FloatAnchor;
        } else {
            return false;
        }
        return true;
    }
}

unsafe extern "C" fn parse_float_relative(
    mut relative: String_0,
    mut out: *mut FloatRelative,
) -> bool {
    unsafe {
        let mut str: *mut ::core::ffi::c_char = relative.data;
        if striequal(str, c"editor".as_ptr()) {
            *out = kFloatRelativeEditor;
        } else if striequal(str, c"win".as_ptr()) {
            *out = kFloatRelativeWindow;
        } else if striequal(str, c"cursor".as_ptr()) {
            *out = kFloatRelativeCursor;
        } else if striequal(str, c"mouse".as_ptr()) {
            *out = kFloatRelativeMouse;
        } else if striequal(str, c"tabline".as_ptr()) {
            *out = kFloatRelativeTabline;
        } else if striequal(str, c"laststatus".as_ptr()) {
            *out = kFloatRelativeLaststatus;
        } else {
            return false;
        }
        return true;
    }
}

unsafe extern "C" fn parse_config_split(mut split: String_0, mut out: *mut WinSplit) -> bool {
    unsafe {
        let mut str: *mut ::core::ffi::c_char = split.data;
        if striequal(str, c"left".as_ptr()) {
            *out = kWinSplitLeft;
        } else if striequal(str, c"right".as_ptr()) {
            *out = kWinSplitRight;
        } else if striequal(str, c"above".as_ptr()) {
            *out = kWinSplitAbove;
        } else if striequal(str, c"below".as_ptr()) {
            *out = kWinSplitBelow;
        } else {
            return false;
        }
        return true;
    }
}

unsafe extern "C" fn parse_float_bufpos(mut bufpos: Array, mut out: *mut lpos_T) -> bool {
    unsafe {
        if bufpos.size != 2 as size_t
            || (*bufpos.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                as ::core::ffi::c_uint
                != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*bufpos.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                as ::core::ffi::c_uint
                != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return false;
        }
        (*out).lnum = (*bufpos.items.offset(0 as ::core::ffi::c_int as isize))
            .data
            .integer as linenr_T;
        (*out).col = (*bufpos.items.offset(1 as ::core::ffi::c_int as isize))
            .data
            .integer as colnr_T;
        return true;
    }
}

unsafe extern "C" fn parse_bordertext(
    mut bordertext: Object,
    mut bordertext_type: BorderTextType,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) {
    unsafe {
        if bordertext.type_0 as ::core::ffi::c_uint
            != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
            && bordertext.type_0 as ::core::ffi::c_uint
                != kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            api_err_exp(
                err,
                c"title/footer".as_ptr(),
                c"String or Array".as_ptr(),
                api_typename(bordertext.type_0),
            );
            return;
        }
        if bordertext.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
            && bordertext.data.array.size == 0 as size_t
        {
            api_err_exp(
                err,
                c"title/footer".as_ptr(),
                c"non-empty Array".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
            );
            return;
        }
        let mut is_present: *mut bool = ::core::ptr::null_mut::<bool>();
        let mut chunks: *mut VirtText = ::core::ptr::null_mut::<VirtText>();
        let mut width: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
        match bordertext_type as ::core::ffi::c_uint {
            0 => {
                is_present = &raw mut (*fconfig).title;
                chunks = &raw mut (*fconfig).title_chunks;
                width = &raw mut (*fconfig).title_width;
            }
            1 => {
                is_present = &raw mut (*fconfig).footer;
                chunks = &raw mut (*fconfig).footer_chunks;
                width = &raw mut (*fconfig).footer_width;
            }
            _ => {}
        }
        if bordertext.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if bordertext.data.string.size == 0 as size_t {
                *is_present = false;
                return;
            }
            (*chunks).capacity = 0 as size_t;
            (*chunks).size = (*chunks).capacity;
            (*chunks).items = ::core::ptr::null_mut::<VirtTextChunk>();
            if (*chunks).size == (*chunks).capacity {
                (*chunks).capacity = if (*chunks).capacity != 0 {
                    (*chunks).capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*chunks).items = xrealloc(
                    (*chunks).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<VirtTextChunk>().wrapping_mul((*chunks).capacity),
                ) as *mut VirtTextChunk;
            } else {
            };
            let c2rust_fresh1 = (*chunks).size;
            (*chunks).size = (*chunks).size.wrapping_add(1);
            *(*chunks).items.offset(c2rust_fresh1 as isize) = VirtTextChunk {
                text: xstrdup(bordertext.data.string.data),
                hl_id: -1 as ::core::ffi::c_int,
            };
            *width = mb_string2cells(bordertext.data.string.data) as ::core::ffi::c_int;
            *is_present = true;
            return;
        }
        *width = 0 as ::core::ffi::c_int;
        *chunks = parse_virt_text(bordertext.data.array, err, width);
        *is_present = true;
    }
}

unsafe extern "C" fn parse_bordertext_pos(
    mut wp: *mut win_T,
    mut bordertext_pos: String_0,
    mut bordertext_type: BorderTextType,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) -> bool {
    unsafe {
        let mut align: *mut AlignTextPos = ::core::ptr::null_mut::<AlignTextPos>();
        match bordertext_type as ::core::ffi::c_uint {
            0 => {
                align = &raw mut (*fconfig).title_pos;
            }
            1 => {
                align = &raw mut (*fconfig).footer_pos;
            }
            _ => {}
        }
        if bordertext_pos.size == 0 as size_t {
            if wp.is_null() {
                *align = kAlignLeft;
            }
            return true;
        }
        let mut pos: *mut ::core::ffi::c_char = bordertext_pos.data;
        if strequal(pos, c"left".as_ptr()) {
            *align = kAlignLeft;
        } else if strequal(pos, c"center".as_ptr()) {
            *align = kAlignCenter;
        } else if strequal(pos, c"right".as_ptr()) {
            *align = kAlignRight;
        } else if true {
            api_err_invalid(
                err,
                if bordertext_type as ::core::ffi::c_uint
                    == kBorderTextTitle as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    c"title_pos".as_ptr()
                } else {
                    c"footer_pos".as_ptr()
                },
                pos,
                0 as int64_t,
                true,
            );
            return false;
        }
        return true;
    }
}

pub(crate) unsafe extern "C" fn parse_win_config(
    mut wp: *mut win_T,
    mut config: *mut KeyDict_win_config,
    mut fconfig: *mut WinConfig,
    mut reconf: bool,
    mut err: *mut Error,
) -> bool {
    unsafe {
        let mut border_style: Object = Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        let mut has_relative: bool = false;
        let mut relative_is_win: bool = false;
        let mut is_split: bool = false;
        '_fail: {
            if (*config).relative.size > 0 as size_t {
                if !parse_float_relative((*config).relative, &raw mut (*fconfig).relative) {
                    api_err_invalid(
                        err,
                        c"relative".as_ptr(),
                        (*config).relative.data,
                        0 as int64_t,
                        true,
                    );
                    break '_fail;
                } else if (*config).relative.size > 0 as size_t
                    && !(has_key((*config).is_set__win_config_, 2 as ::core::ffi::c_int)
                        && has_key((*config).is_set__win_config_, 1 as ::core::ffi::c_int))
                    && !(has_key((*config).is_set__win_config_, 12 as ::core::ffi::c_int))
                {
                    api_err_required(err, c"'relative' requires 'row'/'col' or 'bufpos'".as_ptr());
                    break '_fail;
                }
                has_relative = true;
                (*fconfig).external = false;
                if (*fconfig).relative as ::core::ffi::c_uint
                    == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    relative_is_win = true;
                    (*fconfig).bufpos.lnum = -1 as ::core::ffi::c_int as linenr_T;
                }
            } else if !(*config).external {
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__vertical,
                ) || has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__split,
                ) {
                    is_split = true;
                    (*fconfig).external = false;
                } else if wp.is_null() {
                    if true {
                        api_err_required(
                            err,
                            c"'relative' or 'external' when creating a float".as_ptr(),
                        );
                        break '_fail;
                    }
                }
            }
            if has_key((*config).is_set__win_config_, 19 as ::core::ffi::c_int) && !is_split {
                api_err_conflict(err, c"vertical".as_ptr(), c"floating windows".as_ptr());
            } else if has_key((*config).is_set__win_config_, 6 as ::core::ffi::c_int) && !is_split {
                api_err_conflict(err, c"split".as_ptr(), c"floating windows".as_ptr());
            } else {
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__split,
                ) {
                    if !is_split {
                        api_err_conflict(err, c"split".as_ptr(), c"floating windows".as_ptr());
                        break '_fail;
                    } else if !parse_config_split((*config).split, &raw mut (*fconfig).split) {
                        api_err_invalid(
                            err,
                            c"split".as_ptr(),
                            (*config).split.data,
                            0 as int64_t,
                            true,
                        );
                        break '_fail;
                    }
                }
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__anchor,
                ) {
                    if !parse_float_anchor((*config).anchor, &raw mut (*fconfig).anchor) {
                        api_err_invalid(
                            err,
                            c"anchor".as_ptr(),
                            (*config).anchor.data,
                            0 as int64_t,
                            true,
                        );
                        break '_fail;
                    }
                }
                if has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__row) {
                    if !has_relative || is_split as ::core::ffi::c_int != 0 {
                        generate_api_error(wp, c"row".as_ptr(), err);
                        break '_fail;
                    }
                    (*fconfig).row = (*config).row as ::core::ffi::c_double;
                }
                if has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__col) {
                    if !has_relative || is_split as ::core::ffi::c_int != 0 {
                        generate_api_error(wp, c"col".as_ptr(), err);
                        break '_fail;
                    }
                    (*fconfig).col = (*config).col as ::core::ffi::c_double;
                }
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__bufpos,
                ) {
                    if !has_relative || is_split as ::core::ffi::c_int != 0 {
                        generate_api_error(wp, c"bufpos".as_ptr(), err);
                        break '_fail;
                    } else if !parse_float_bufpos((*config).bufpos, &raw mut (*fconfig).bufpos) {
                        api_err_exp(
                            err,
                            c"bufpos".as_ptr(),
                            c"[row, col] array".as_ptr(),
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_fail;
                    }
                    if !(has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__row)) {
                        (*fconfig).row = (if (*fconfig).anchor as ::core::ffi::c_int
                            & kFloatAnchorSouth as ::core::ffi::c_int
                            != 0
                        {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        }) as ::core::ffi::c_double;
                    }
                    if !(has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__col)) {
                        (*fconfig).col = 0 as ::core::ffi::c_int as ::core::ffi::c_double;
                    }
                }
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__width,
                ) {
                    if !((*config).width > 0 as Integer) {
                        api_err_exp(
                            err,
                            c"width".as_ptr(),
                            c"positive Integer".as_ptr(),
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_fail;
                    }
                    (*fconfig).width = (*config).width as ::core::ffi::c_int;
                } else if !reconf && !is_split {
                    if true {
                        api_err_required(err, c"width".as_ptr());
                        break '_fail;
                    }
                }
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__height,
                ) {
                    if !((*config).height > 0 as Integer) {
                        api_err_exp(
                            err,
                            c"height".as_ptr(),
                            c"positive Integer".as_ptr(),
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        break '_fail;
                    }
                    (*fconfig).height = (*config).height as ::core::ffi::c_int;
                } else if !reconf && !is_split {
                    if true {
                        api_err_required(err, c"height".as_ptr());
                        break '_fail;
                    }
                }
                if has_key(
                    (*config).is_set__win_config_,
                    KEYSET_OPTIDX_win_config__external,
                ) {
                    (*fconfig).external = (*config).external as bool;
                    if has_relative as ::core::ffi::c_int != 0
                        && (*fconfig).external as ::core::ffi::c_int != 0
                    {
                        api_err_conflict(err, c"relative".as_ptr(), c"external".as_ptr());
                        break '_fail;
                    } else if (*fconfig).external as ::core::ffi::c_int != 0
                        && !ui_has(kUIMultigrid)
                    {
                        api_set_error(
                            err,
                            kErrorTypeValidation,
                            c"UI doesn't support external windows".as_ptr(),
                        );
                        break '_fail;
                    }
                }
                if has_key((*config).is_set__win_config_, 3 as ::core::ffi::c_int)
                    && (*fconfig).external as ::core::ffi::c_int != 0
                {
                    api_err_conflict(err, c"win".as_ptr(), c"external window".as_ptr());
                } else {
                    if relative_is_win as ::core::ffi::c_int != 0
                        || has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__win)
                            && !is_split
                            && !wp.is_null()
                            && (*wp).w_floating as ::core::ffi::c_int != 0
                            && (*fconfig).relative as ::core::ffi::c_uint
                                == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut target_win: *mut win_T = find_window_by_handle((*config).win, err);
                        if target_win.is_null() {
                            break '_fail;
                        } else if target_win == wp {
                            api_set_error(
                                err,
                                kErrorTypeException,
                                c"floating window cannot be relative to itself".as_ptr(),
                            );
                            break '_fail;
                        }
                        (*fconfig).window = (*target_win).handle as Window;
                    } else {
                        if has_key((*config).is_set__win_config_, KEYSET_OPTIDX_win_config__win) {
                            if !is_split && !has_relative && (wp.is_null() || !(*wp).w_floating) {
                                api_err_required(
                                    err,
                                    c"non-float with 'win' requires 'split' or 'vertical'".as_ptr(),
                                );
                                break '_fail;
                            }
                            (*fconfig).window = (*config).win;
                        }
                        if (*fconfig).window == 0 as ::core::ffi::c_int {
                            (*fconfig).window = (*curwin.get()).handle as Window;
                        }
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__focusable,
                    ) {
                        (*fconfig).focusable = (*config).focusable as bool;
                        (*fconfig).mouse = (*config).focusable as bool;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__mouse,
                    ) {
                        (*fconfig).mouse = (*config).mouse as bool;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__zindex,
                    ) {
                        if is_split {
                            api_err_conflict(err, c"zindex".as_ptr(), c"non-float window".as_ptr());
                            break '_fail;
                        } else if !((*config).zindex > 0 as Integer) {
                            api_err_exp(
                                err,
                                c"zindex".as_ptr(),
                                c"positive Integer".as_ptr(),
                                ::core::ptr::null::<::core::ffi::c_char>(),
                            );
                            break '_fail;
                        }
                        (*fconfig).zindex = (*config).zindex as ::core::ffi::c_int;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__title,
                    ) {
                        if is_split {
                            api_err_conflict(err, c"title".as_ptr(), c"non-float window".as_ptr());
                            break '_fail;
                        }
                        parse_bordertext((*config).title, kBorderTextTitle, fconfig, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_fail;
                        } else if !parse_bordertext_pos(
                            wp,
                            (*config).title_pos,
                            kBorderTextTitle,
                            fconfig,
                            err,
                        ) {
                            break '_fail;
                        }
                    } else if has_key((*config).is_set__win_config_, 22 as ::core::ffi::c_int) {
                        api_err_required(err, c"'title' requires 'title_pos'".as_ptr());
                        break '_fail;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__footer,
                    ) {
                        if is_split {
                            api_err_conflict(err, c"footer".as_ptr(), c"non-float window".as_ptr());
                            break '_fail;
                        }
                        parse_bordertext((*config).footer, kBorderTextFooter, fconfig, err);
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            break '_fail;
                        } else if !parse_bordertext_pos(
                            wp,
                            (*config).footer_pos,
                            kBorderTextFooter,
                            fconfig,
                            err,
                        ) {
                            break '_fail;
                        }
                    } else if has_key((*config).is_set__win_config_, 23 as ::core::ffi::c_int) {
                        api_err_required(err, c"'footer' requires 'footer_pos'".as_ptr());
                        break '_fail;
                    }
                    border_style = object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed { boolean: false },
                    };
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__border,
                    ) {
                        if is_split {
                            api_err_conflict(err, c"border".as_ptr(), c"non-float window".as_ptr());
                            break '_fail;
                        }
                        border_style = (*config).border;
                        if border_style.type_0 as ::core::ffi::c_uint
                            != kObjectTypeNil as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            parse_border_style(border_style, fconfig, err);
                            if (*err).type_0 as ::core::ffi::c_int
                                != kErrorTypeNone as ::core::ffi::c_int
                            {
                                break '_fail;
                            }
                        }
                    } else if *p_winborder.get() as ::core::ffi::c_int != NUL
                        && (wp.is_null() || !(*wp).w_floating)
                        && !parse_winborder(fconfig, p_winborder.get(), err)
                    {
                        break '_fail;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__style,
                    ) {
                        if *(*config)
                            .style
                            .data
                            .offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == NUL
                        {
                            (*fconfig).style = kWinStyleUnused;
                        } else if striequal((*config).style.data, c"minimal".as_ptr()) {
                            (*fconfig).style = kWinStyleMinimal;
                        } else if true {
                            api_err_invalid(
                                err,
                                c"style".as_ptr(),
                                (*config).style.data,
                                0 as int64_t,
                                true,
                            );
                            break '_fail;
                        }
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__noautocmd,
                    ) {
                        if !wp.is_null()
                            && (*config).noautocmd as ::core::ffi::c_int
                                != (*fconfig).noautocmd as ::core::ffi::c_int
                        {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                c"'noautocmd' cannot be changed on existing window".as_ptr(),
                            );
                            break '_fail;
                        }
                        (*fconfig).noautocmd = (*config).noautocmd as bool;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__fixed,
                    ) {
                        (*fconfig).fixed = (*config).fixed as bool;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config__hide,
                    ) {
                        (*fconfig).hide = (*config).hide as bool;
                    }
                    if has_key(
                        (*config).is_set__win_config_,
                        KEYSET_OPTIDX_win_config___cmdline_offset,
                    ) {
                        (*fconfig)._cmdline_offset =
                            (*config)._cmdline_offset as ::core::ffi::c_int;
                    }
                    return true;
                }
            }
        }
        merge_win_config(
            fconfig,
            if !wp.is_null() {
                (*wp).w_config
            } else {
                WinConfig {
                    window: 0,
                    bufpos: lpos_T {
                        lnum: -1 as linenr_T,
                        col: 0 as colnr_T,
                    },
                    height: 0 as ::core::ffi::c_int,
                    width: 0 as ::core::ffi::c_int,
                    row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                    col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
                    anchor: 0 as FloatAnchor,
                    relative: kFloatRelativeEditor,
                    external: false,
                    focusable: true,
                    mouse: true,
                    split: kWinSplitLeft,
                    zindex: kZIndexFloatDefault as ::core::ffi::c_int,
                    style: kWinStyleUnused,
                    border: false,
                    shadow: false,
                    border_chars: [[0; 32]; 8],
                    border_hl_ids: [0; 8],
                    border_attr: [0; 8],
                    title: false,
                    title_pos: kAlignLeft,
                    title_chunks: VirtText {
                        size: 0,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<VirtTextChunk>(),
                    },
                    title_width: 0,
                    footer: false,
                    footer_pos: kAlignLeft,
                    footer_chunks: VirtText {
                        size: 0,
                        capacity: 0,
                        items: ::core::ptr::null_mut::<VirtTextChunk>(),
                    },
                    footer_width: 0,
                    noautocmd: false,
                    fixed: false,
                    hide: false,
                    _cmdline_offset: INT_MAX,
                }
            },
        );
        return false;
    }
}
