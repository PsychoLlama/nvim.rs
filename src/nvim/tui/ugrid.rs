extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn xcalloc(count: size_t, size: size_t) -> *mut ::core::ffi::c_void;
}
pub type size_t = usize;
pub type int32_t = i32;
pub type uint32_t = u32;
pub type schar_T = uint32_t;
pub type sattr_T = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UCell {
    pub data: schar_T,
    pub attr: sattr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UGrid {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub cells: *mut *mut UCell,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 52] = unsafe {
    ::core::mem::transmute::<[u8; 52], [::core::ffi::c_char; 52]>(
        *b"void ugrid_scroll(UGrid *, int, int, int, int, int)\0",
    )
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[no_mangle]
pub unsafe extern "C" fn ugrid_init(mut grid: *mut UGrid) {
    (*grid).cells = ::core::ptr::null_mut::<*mut UCell>();
}
#[no_mangle]
pub unsafe extern "C" fn ugrid_free(mut grid: *mut UGrid) {
    destroy_cells(grid);
}
#[no_mangle]
pub unsafe extern "C" fn ugrid_resize(
    mut grid: *mut UGrid,
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    destroy_cells(grid);
    (*grid).cells = xmalloc((height as size_t).wrapping_mul(::core::mem::size_of::<*mut UCell>()))
        as *mut *mut UCell;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < height {
        *(*grid).cells.offset(i as isize) =
            xcalloc(width as size_t, ::core::mem::size_of::<UCell>()) as *mut UCell;
        i += 1;
    }
    (*grid).width = width;
    (*grid).height = height;
}
#[no_mangle]
pub unsafe extern "C" fn ugrid_clear(mut grid: *mut UGrid) {
    clear_region(
        grid,
        0 as ::core::ffi::c_int,
        (*grid).height - 1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        (*grid).width - 1 as ::core::ffi::c_int,
        0 as sattr_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ugrid_clear_chunk(
    mut grid: *mut UGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut endcol: ::core::ffi::c_int,
    mut attr: sattr_T,
) {
    clear_region(grid, row, row, col, endcol - 1 as ::core::ffi::c_int, attr);
}
#[no_mangle]
pub unsafe extern "C" fn ugrid_goto(
    mut grid: *mut UGrid,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    (*grid).row = row;
    (*grid).col = col;
}
#[no_mangle]
pub unsafe extern "C" fn ugrid_scroll(
    mut grid: *mut UGrid,
    mut top: ::core::ffi::c_int,
    mut bot: ::core::ffi::c_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut count: ::core::ffi::c_int,
) {
    let mut start: ::core::ffi::c_int = 0;
    let mut stop: ::core::ffi::c_int = 0;
    let mut step: ::core::ffi::c_int = 0;
    if count > 0 as ::core::ffi::c_int {
        start = top;
        stop = bot - count + 1 as ::core::ffi::c_int;
        step = 1 as ::core::ffi::c_int;
    } else {
        start = bot;
        stop = top - count - 1 as ::core::ffi::c_int;
        step = -1 as ::core::ffi::c_int;
    }
    let mut i: ::core::ffi::c_int = start;
    while i != stop {
        let mut target_row: *mut UCell = (*(*grid).cells.offset(i as isize)).offset(left as isize);
        let mut source_row: *mut UCell =
            (*(*grid).cells.offset((i + count) as isize)).offset(left as isize);
        '_c2rust_label: {
            if right >= left && left >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"right >= left && left >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/tui/ugrid.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    66 as ::core::ffi::c_uint,
                    __ASSERT_FUNCTION.as_ptr(),
                );
            }
        };
        memcpy(
            target_row as *mut ::core::ffi::c_void,
            source_row as *const ::core::ffi::c_void,
            ::core::mem::size_of::<UCell>().wrapping_mul(
                (right as size_t)
                    .wrapping_sub(left as size_t)
                    .wrapping_add(1 as size_t),
            ),
        );
        i += step;
    }
}
unsafe extern "C" fn clear_region(
    mut grid: *mut UGrid,
    mut top: ::core::ffi::c_int,
    mut bot: ::core::ffi::c_int,
    mut left: ::core::ffi::c_int,
    mut right: ::core::ffi::c_int,
    mut attr: sattr_T,
) {
    let mut row: ::core::ffi::c_int = top;
    while row <= bot {
        let mut row_cells: *mut UCell = *(*grid).cells.offset(row as isize);
        let mut curcol: ::core::ffi::c_int = left;
        while curcol < right + 1 as ::core::ffi::c_int {
            let mut cell: *mut UCell = row_cells.offset(curcol as isize);
            (*cell).data = ' ' as ::core::ffi::c_int as schar_T;
            (*cell).attr = attr;
            curcol += 1;
        }
        row += 1;
    }
}
unsafe extern "C" fn destroy_cells(mut grid: *mut UGrid) {
    if !(*grid).cells.is_null() {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*grid).height {
            xfree(*(*grid).cells.offset(i as isize) as *mut ::core::ffi::c_void);
            i += 1;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*grid).cells as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        *ptr_;
    }
}
