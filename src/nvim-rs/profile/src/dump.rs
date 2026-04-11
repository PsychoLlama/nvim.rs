//! Profile dump functions: func_dump_profile and script_dump_profile.
//!
//! Implements formatting and output of profiling results for functions and
//! scripts. Replaces C implementations in profile.c.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};

use crate::types::{FileHandle, UFuncHandle};
use crate::Proftime;

// K_SPECIAL = 0x80 (first byte of special key code sequences)
const K_SPECIAL: u8 = 0x80;

extern "C" {
    // profile_msg returns a static buffer (not reentrant)
    fn profile_msg(tm: Proftime) -> *const c_char;
    fn profile_equal(tm1: Proftime, tm2: Proftime) -> bool;
    fn profile_cmp(tm1: Proftime, tm2: Proftime) -> c_int;

    // profile_init pointer-based setters (scriptitem_T * as *mut c_void)
    fn nvim_si_ptr_set_pr_count(si: *mut c_void, val: c_int);
    fn nvim_si_ptr_set_pr_total(si: *mut c_void, val: Proftime);
    fn nvim_si_ptr_set_pr_self(si: *mut c_void, val: Proftime);
    fn nvim_si_ptr_ga_init_prl(si: *mut c_void);
    fn nvim_si_ptr_set_prl_idx(si: *mut c_void, val: c_int);
    fn nvim_si_ptr_set_prof_on(si: *mut c_void, val: bool);
    fn nvim_si_ptr_set_pr_nest(si: *mut c_void, val: c_int);

    // File output
    fn nvim_profile_fputs(s: *const c_char, fd: FileHandle);
    fn nvim_profile_fclose(fd: FileHandle);

    // ufunc_T field accessors
    fn nvim_ufunc_get_tm_count(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_get_tm_total(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_get_tm_self(fp: UFuncHandle) -> Proftime;
    fn nvim_ufunc_get_name(fp: UFuncHandle) -> *const c_char;
    fn nvim_ufunc_get_name_first_byte(fp: UFuncHandle) -> u8;
    fn nvim_ufunc_get_script_ctx_sid(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_get_script_ctx_lnum(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_get_script_ctx_chan(fp: UFuncHandle) -> u64;
    fn nvim_ufunc_get_lines_len(fp: UFuncHandle) -> c_int;
    fn nvim_ufunc_funcline_is_null(fp: UFuncHandle, idx: c_int) -> c_int;
    fn nvim_ufunc_get_funcline(fp: UFuncHandle, i: c_int) -> *const c_char;
    fn nvim_ufunc_get_tml_count_i(fp: UFuncHandle, i: c_int) -> c_int;
    fn nvim_ufunc_get_tml_total_i(fp: UFuncHandle, i: c_int) -> Proftime;
    fn nvim_ufunc_get_tml_self_i(fp: UFuncHandle, i: c_int) -> Proftime;

    // get_scriptname
    fn nvim_rt_get_scriptname(sc_sid: c_int, sc_chan: u64, should_free: *mut bool) -> *mut c_char;
    fn nvim_profile_xfree_ptr(ptr: *mut c_void);

    // Collect profiled funcs from hashtable
    fn nvim_profile_collect_profiled_funcs(out_array: *mut *mut UFuncHandle, out_count: *mut c_int);

    // scriptitem_T dump accessors
    fn nvim_get_script_items_len() -> c_int;
    fn nvim_si_get_prof_on(sid: c_int) -> c_int;
    fn nvim_si_get_pr_count(sid: c_int) -> c_int;
    fn nvim_si_get_pr_total(sid: c_int) -> Proftime;
    fn nvim_si_get_pr_self(sid: c_int) -> Proftime;
    fn nvim_si_get_name(sid: c_int) -> *const c_char;
    fn nvim_si_fopen(sid: c_int) -> FileHandle;
    fn nvim_si_prl_ga_len(sid: c_int) -> c_int;
    fn nvim_si_prl_item_get_count(sid: c_int, idx: c_int) -> c_int;
    fn nvim_si_prl_item_get_total(sid: c_int, idx: c_int) -> Proftime;
    fn nvim_si_prl_item_get_self(sid: c_int, idx: c_int) -> Proftime;
    fn nvim_profile_vim_fgets(buf: *mut c_char, size: c_int, fd: FileHandle) -> c_int;
    fn nvim_profile_get_iobuff() -> *mut c_char;
    fn nvim_profile_get_iosize() -> c_int;

    // xmalloc/xfree for C allocations
    fn xfree(ptr: *mut c_void);
}

/// Write a string to file handle using fputs via C wrapper.
///
/// # Safety
///
/// `fd` must be a valid FILE* wrapped in c_void.
unsafe fn write_str(fd: FileHandle, s: &str) {
    // Ensure NUL termination by using a temporary CString-like approach.
    // We write a null-terminated byte slice.
    let mut bytes = s.as_bytes().to_vec();
    bytes.push(0);
    nvim_profile_fputs(bytes.as_ptr().cast::<c_char>(), fd);
}

/// Format and write count + total + self times for one function or script line.
///
/// Mirrors the C `prof_func_line` function exactly.
///
/// # Safety
///
/// `fd` must be a valid FILE*.
unsafe fn prof_func_line_write(
    fd: FileHandle,
    count: c_int,
    total: Proftime,
    self_time: Proftime,
    prefer_self: bool,
) {
    if count > 0 {
        let count_str = format!("{:5} ", count);
        write_str(fd, &count_str);

        // total column (11 chars + space)
        if prefer_self && profile_equal(total, self_time) {
            write_str(fd, "           ");
        } else {
            let msg = CStr::from_ptr(profile_msg(total)).to_string_lossy();
            let s = format!("{} ", msg);
            write_str(fd, &s);
        }

        // self column (11 chars + space)
        if !prefer_self && profile_equal(total, self_time) {
            write_str(fd, "           ");
        } else {
            let msg = CStr::from_ptr(profile_msg(self_time)).to_string_lossy();
            let s = format!("{} ", msg);
            write_str(fd, &s);
        }
    } else {
        write_str(fd, "                            ");
    }
}

/// Write sorted function list to fd. Mirrors `prof_sort_list`.
///
/// # Safety
///
/// `fd` must be a valid FILE*. `sorttab` must be a valid slice of UFuncHandle.
unsafe fn prof_sort_list_write(
    fd: FileHandle,
    sorttab: &[UFuncHandle],
    title: &str,
    prefer_self: bool,
) {
    write_str(fd, &format!("FUNCTIONS SORTED ON {} TIME\n", title));
    write_str(fd, "count  total (s)   self (s)  function\n");

    let limit = if sorttab.len() > 20 {
        20
    } else {
        sorttab.len()
    };
    for fp in &sorttab[..limit] {
        let fp = *fp;
        let count = nvim_ufunc_get_tm_count(fp);
        let total = nvim_ufunc_get_tm_total(fp);
        let self_time = nvim_ufunc_get_tm_self(fp);
        prof_func_line_write(fd, count, total, self_time, prefer_self);

        let first_byte = nvim_ufunc_get_name_first_byte(fp);
        let name_ptr = nvim_ufunc_get_name(fp);
        if first_byte == K_SPECIAL {
            // Skip 3 bytes: K_SPECIAL + KS + KE
            let snr_name = CStr::from_ptr(name_ptr.add(3)).to_string_lossy();
            write_str(fd, &format!(" <SNR>{}()\n", snr_name));
        } else {
            let name = CStr::from_ptr(name_ptr).to_string_lossy();
            write_str(fd, &format!(" {}()\n", name));
        }
    }
    write_str(fd, "\n");
}

/// Dump profiling results for all functions to `fd`.
/// Replaces C `func_dump_profile`.
///
/// # Safety
///
/// `fd` must be a valid FILE*.
pub unsafe fn rs_func_dump_profile_impl(fd: FileHandle) {
    let mut arr_ptr: *mut UFuncHandle = std::ptr::null_mut();
    let mut count: c_int = 0;
    nvim_profile_collect_profiled_funcs(
        std::ptr::addr_of_mut!(arr_ptr).cast::<*mut *mut c_void>(),
        std::ptr::addr_of_mut!(count),
    );

    if count == 0 {
        return;
    }

    // Build a Vec from the C-allocated array.
    let funcs: Vec<UFuncHandle> = std::slice::from_raw_parts(arr_ptr, count as usize).to_vec();

    // Free the C-allocated array (the pointers inside are owned by C).
    xfree(arr_ptr.cast::<c_void>());

    // Print per-function profiling data.
    for &fp in &funcs {
        let first_byte = nvim_ufunc_get_name_first_byte(fp);
        let name_ptr = nvim_ufunc_get_name(fp);
        if first_byte == K_SPECIAL {
            let snr_name = CStr::from_ptr(name_ptr.add(3)).to_string_lossy();
            write_str(fd, &format!("FUNCTION  <SNR>{}()\n", snr_name));
        } else {
            let name = CStr::from_ptr(name_ptr).to_string_lossy();
            write_str(fd, &format!("FUNCTION  {}()\n", name));
        }

        let sid = nvim_ufunc_get_script_ctx_sid(fp);
        if sid != 0 {
            let chan = nvim_ufunc_get_script_ctx_chan(fp);
            let lnum = nvim_ufunc_get_script_ctx_lnum(fp);
            let mut should_free = false;
            let sname_ptr = nvim_rt_get_scriptname(sid, chan, std::ptr::addr_of_mut!(should_free));
            let sname = CStr::from_ptr(sname_ptr).to_string_lossy();
            write_str(fd, &format!("    Defined: {}:{}\n", sname, lnum));
            if should_free {
                nvim_profile_xfree_ptr(sname_ptr.cast::<c_void>());
            }
        }

        let tm_count = nvim_ufunc_get_tm_count(fp);
        if tm_count == 1 {
            write_str(fd, "Called 1 time\n");
        } else {
            write_str(fd, &format!("Called {} times\n", tm_count));
        }

        let tm_total = nvim_ufunc_get_tm_total(fp);
        let msg_total = CStr::from_ptr(profile_msg(tm_total)).to_string_lossy();
        write_str(fd, &format!("Total time: {}\n", msg_total));

        let tm_self = nvim_ufunc_get_tm_self(fp);
        let msg_self = CStr::from_ptr(profile_msg(tm_self)).to_string_lossy();
        write_str(fd, &format!(" Self time: {}\n", msg_self));

        write_str(fd, "\n");
        write_str(fd, "count  total (s)   self (s)\n");

        let lines_len = nvim_ufunc_get_lines_len(fp);
        for i in 0..lines_len {
            if nvim_ufunc_funcline_is_null(fp, i) != 0 {
                continue;
            }
            let tml_count = nvim_ufunc_get_tml_count_i(fp, i);
            let tml_total = nvim_ufunc_get_tml_total_i(fp, i);
            let tml_self = nvim_ufunc_get_tml_self_i(fp, i);
            prof_func_line_write(fd, tml_count, tml_total, tml_self, true);

            let line_ptr = nvim_ufunc_get_funcline(fp, i);
            let line = CStr::from_ptr(line_ptr).to_string_lossy();
            write_str(fd, &format!("{}\n", line));
        }
        write_str(fd, "\n");
    }

    // Sort and print by total time.
    let mut sorttab_total = funcs.clone();
    sorttab_total.sort_by(|&a, &b| {
        let ta = nvim_ufunc_get_tm_total(a);
        let tb = nvim_ufunc_get_tm_total(b);
        profile_cmp(ta, tb).cmp(&0)
    });
    prof_sort_list_write(fd, &sorttab_total, "TOTAL", false);

    // Sort and print by self time.
    let mut sorttab_self = funcs;
    sorttab_self.sort_by(|&a, &b| {
        let ta = nvim_ufunc_get_tm_self(a);
        let tb = nvim_ufunc_get_tm_self(b);
        profile_cmp(ta, tb).cmp(&0)
    });
    prof_sort_list_write(fd, &sorttab_self, "SELF", true);
}

/// Dump profiling results for all scripts to `fd`.
/// Replaces C `script_dump_profile`.
///
/// # Safety
///
/// `fd` must be a valid FILE*.
pub unsafe fn rs_script_dump_profile_impl(fd: FileHandle) {
    let items_len = nvim_get_script_items_len();
    let iobuf = nvim_profile_get_iobuff();
    let iosize = nvim_profile_get_iosize();

    for id in 1..=items_len {
        if nvim_si_get_prof_on(id) == 0 {
            continue;
        }

        let name_ptr = nvim_si_get_name(id);
        let name = CStr::from_ptr(name_ptr).to_string_lossy();
        write_str(fd, &format!("SCRIPT  {}\n", name));

        let pr_count = nvim_si_get_pr_count(id);
        if pr_count == 1 {
            write_str(fd, "Sourced 1 time\n");
        } else {
            write_str(fd, &format!("Sourced {} times\n", pr_count));
        }

        let pr_total = nvim_si_get_pr_total(id);
        let msg_total = CStr::from_ptr(profile_msg(pr_total)).to_string_lossy();
        write_str(fd, &format!("Total time: {}\n", msg_total));

        let pr_self = nvim_si_get_pr_self(id);
        let msg_self = CStr::from_ptr(profile_msg(pr_self)).to_string_lossy();
        write_str(fd, &format!(" Self time: {}\n", msg_self));

        write_str(fd, "\n");
        write_str(fd, "count  total (s)   self (s)\n");

        let sfd = nvim_si_fopen(id);
        if sfd.is_null() {
            write_str(fd, "Cannot open file!\n");
        } else {
            let prl_len = nvim_si_prl_ga_len(id);
            let mut i: c_int = 0;
            loop {
                if nvim_profile_vim_fgets(iobuf, iosize, sfd) != 0 {
                    break;
                }

                // When a line has been truncated, append NL, taking care of
                // multi-byte characters.
                let n = (iosize - 2) as usize;
                let buf = std::slice::from_raw_parts_mut(iobuf as *mut u8, iosize as usize);
                if buf[n] != 0 && buf[n] != b'\n' {
                    let mut idx = n;
                    // Move to the first byte of this char.
                    while idx > 0 && (buf[idx] & 0xc0) == 0x80 {
                        idx -= 1;
                    }
                    buf[idx] = b'\n';
                    buf[idx + 1] = 0;
                }

                if i < prl_len {
                    let pp_count = nvim_si_prl_item_get_count(id, i);
                    if pp_count > 0 {
                        let count_str = format!("{:5} ", pp_count);
                        write_str(fd, &count_str);

                        let pp_total = nvim_si_prl_item_get_total(id, i);
                        let pp_self = nvim_si_prl_item_get_self(id, i);
                        if profile_equal(pp_total, pp_self) {
                            write_str(fd, "           ");
                        } else {
                            let msg = CStr::from_ptr(profile_msg(pp_total)).to_string_lossy();
                            write_str(fd, &format!("{} ", msg));
                        }
                        let msg = CStr::from_ptr(profile_msg(pp_self)).to_string_lossy();
                        write_str(fd, &format!("{} ", msg));
                    } else {
                        write_str(fd, "                            ");
                    }
                } else {
                    write_str(fd, "                            ");
                }

                // Write the line content from IObuff (already NUL-terminated).
                nvim_profile_fputs(iobuf.cast::<c_char>(), fd);

                i += 1;
            }
            nvim_profile_fclose(sfd);
        }
        write_str(fd, "\n");
    }
}

/// Export: dump profiling results for all functions.
///
/// Called by `nvim_profile_func_dump` C wrapper, which replaces the old call
/// to C `func_dump_profile`.
///
/// # Safety
///
/// `fd` must be a valid FILE* cast to *mut c_void.
#[export_name = "rs_func_dump_profile"]
pub unsafe extern "C" fn rs_func_dump_profile(fd: FileHandle) {
    rs_func_dump_profile_impl(fd);
}

/// Export: dump profiling results for all scripts.
///
/// Called by `nvim_profile_script_dump` C wrapper, which replaces the old call
/// to C `script_dump_profile`.
///
/// # Safety
///
/// `fd` must be a valid FILE* cast to *mut c_void.
#[export_name = "rs_script_dump_profile"]
pub unsafe extern "C" fn rs_script_dump_profile(fd: FileHandle) {
    rs_script_dump_profile_impl(fd);
}

/// Initialize profiling for a script item.
/// Replaces C `profile_init`. Called via `nvim_rt_profile_init` in runtime_ffi.c.
///
/// # Safety
///
/// `si` must be a valid `scriptitem_T *` pointer.
#[export_name = "profile_init"]
pub unsafe extern "C" fn rs_profile_init(si: *mut c_void) {
    nvim_si_ptr_set_pr_count(si, 0);
    nvim_si_ptr_set_pr_total(si, crate::rs_profile_zero());
    nvim_si_ptr_set_pr_self(si, crate::rs_profile_zero());
    nvim_si_ptr_ga_init_prl(si);
    nvim_si_ptr_set_prl_idx(si, -1);
    nvim_si_ptr_set_prof_on(si, true);
    nvim_si_ptr_set_pr_nest(si, 0);
}
