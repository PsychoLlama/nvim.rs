//! Find patterns in included files: `find_pattern_in_path()` and helpers.
//!
//! Full Rust port of the `find_pattern_in_path` ecosystem from search_shim.c.
//! Exports `find_pattern_in_path` directly, replacing the C implementation.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type ColnrT = c_int;
type WinHandle = *mut c_void;
type BufHandle = *mut c_void;
/// Opaque handle to a C `regmatch_T` (single-line matcher)
type RegmatchHandle = *mut c_void;
/// C FILE pointer
type FilePtr = *mut c_void;

// =============================================================================
// Constants
// =============================================================================

const FORWARD: c_int = 1;
const FAIL: c_int = 0;
const OK: c_int = 1;

// FindPatternInPath `type` values
const FIND_DEFINE: c_int = 2;
const CHECK_PATH: c_int = 3;

// Action values
const ACTION_SHOW: c_int = 1;
const ACTION_SPLIT: c_int = 3;
const ACTION_SHOW_ALL: c_int = 4;
const ACTION_EXPAND: c_int = 5;

// FNAME flags for path functions
const FNAME_EXP: c_int = 2;
const FNAME_INCL: c_int = 8;
const FNAME_REL: c_int = 16;

// Path comparison result
const K_EQUAL_FILES: c_int = 1;

// SHM flags
const SHM_COMPLETIONSCAN: c_int = b'C' as c_int;

// Highlight group indices (from HlF enum order in highlight_defs.h)
const HLF_D: c_int = 5; // Directory
const HLF_N: c_int = 13; // LineNr
const HLF_R: c_int = 19; // Question (used for scan messages)

// UPD_VALID
const UPD_VALID: c_int = 10;

// IOSIZE / LSIZE
const IOSIZE: usize = 1025;
const LSIZE: c_int = 512;

// TAB
const TAB: u8 = b'\t';

// RE_MAGIC
const RE_MAGIC: c_int = 1;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Global state
    static mut got_int: bool;

    // Current window handle (global)
    static curwin: WinHandle;

    // IObuff global (link_name avoids conflicts with other crates declaring IObuff)
    #[link_name = "IObuff"]
    static mut IObuff_ps: [c_char; IOSIZE];

    // Options
    static p_verbose: c_int;

    // Regexp (single-line)
    fn vim_regcomp(pat: *const c_char, magic: c_int) -> *mut c_void; // returns regprog_T*
    fn vim_regexec(rmp: RegmatchHandle, line: *const c_char, col: ColnrT) -> c_int;
    fn vim_regfree(prog: *mut c_void);

    // Charset
    fn skipwhite(p: *const c_char) -> *const c_char;
    fn vim_iswordc(c: u8) -> c_int;
    fn vim_iswordp(p: *const c_char) -> c_int;
    fn vim_isfilec(c: u8) -> c_int;

    // File I/O
    fn os_fopen(path: *const c_char, mode: *const c_char) -> FilePtr;
    fn vim_fgets(buf: *mut c_char, size: c_int, fp: FilePtr) -> c_int;
    fn fclose(fp: FilePtr) -> c_int;

    // Memory
    fn xmalloc(size: usize) -> *mut c_void;
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;

    // Strings
    fn vim_snprintf(dst: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strncpy(dst: *mut c_char, src: *const c_char, n: usize) -> *mut c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Multibyte comparison
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;

    // Messaging
    fn msg_putchar(c: c_int);
    fn msg_puts(s: *const c_char);
    fn msg_puts_title(s: *const c_char);
    fn msg_puts_hl(s: *const c_char, hlf: c_int, force: c_int);
    fn msg_home_replace(fname: *const c_char);
    fn msg_outtrans(str: *const c_char, attr: c_int, right: bool);
    fn msg_prt_line(s: *const c_char, list: c_int);
    fn msg_end();
    fn msg_trunc(s: *const c_char, force: c_int, attr: c_int);
    fn gotocmdline(clr: bool);
    fn smsg(attr: c_int, fmt: *const c_char, ...);
    fn msg_hist_off_set(v: c_int);
    fn emsg(s: *const c_char) -> c_int;
    fn msg(s: *const c_char, attr: c_int);

    // Path functions
    fn find_file_name_in_path(
        ptr: *const c_char,
        len: usize,
        options: c_int,
        count: c_int,
        rel_fname: *const c_char,
    ) -> *mut c_char;
    fn file_name_in_line(
        ptr: *const c_char,
        col: c_int,
        options: c_int,
        count: c_int,
        rel_fname: *const c_char,
        didmatch: *mut c_int,
    ) -> *mut c_char;
    fn path_full_compare(
        s1: *const c_char,
        s2: *const c_char,
        checkname: c_int,
        expandenv: c_int,
    ) -> c_int;

    // Memline / buffer
    fn ml_get(lnum: LinenrT) -> *mut c_char;

    // Completion
    fn ins_compl_add_infercase(
        str: *const c_char,
        len: c_int,
        icase: c_int,
        fname: *const c_char,
        dir: c_int,
        cont_s_ipos: c_int,
        flags: c_int,
    ) -> c_int;

    // Shortmess
    fn shortmess(x: c_int) -> bool;

    // Verbose
    fn verbose_enter();
    fn verbose_leave();

    // Window / cursor
    fn setpcmark();
    fn check_cursor(win: WinHandle);
    fn validate_cursor(win: WinHandle);
    fn redraw_later(win: WinHandle, upd: c_int);
    fn win_enter(win: WinHandle, undo_sync: c_int);
    fn win_split(size: c_int, flags: c_int) -> c_int;
    fn getfile(
        fnum: c_int,
        fname: *const c_char,
        sfname: *const c_char,
        setpm: c_int,
        lnum: LinenrT,
        forceit: c_int,
    ) -> c_int;
    fn prepare_tagpreview(undo_sync: c_int);
    // Interrupts
    fn line_breakcheck();

    // Get leader length
    fn get_leader_len(
        line: *const c_char,
        flags: *mut c_void,
        do_middle: c_int,
        flags2: c_int,
    ) -> c_int;

    // ignorecase
    fn ignorecase(pat: *const c_char) -> c_int;

    // C accessor functions provided by search_shim.c
    fn nvim_search_get_p_ic() -> c_int;
    fn nvim_get_g_do_tagpreview() -> c_int;
    fn nvim_fpip_get_curbuf_fname() -> *mut c_char;
    fn nvim_fpip_get_curbuf_ffname() -> *mut c_char;
    fn nvim_fpip_get_curbuf_b_p_inc() -> *mut c_char;
    fn nvim_fpip_get_curbuf_b_p_def() -> *mut c_char;
    fn nvim_fpip_get_p_inc() -> *mut c_char;
    fn nvim_fpip_get_p_def() -> *mut c_char;
    fn nvim_fpip_get_p_js() -> c_int;
    fn nvim_fpip_get_curbuf_ml_line_count() -> LinenrT;
    fn nvim_fpip_curwin_cursor_lnum() -> LinenrT;
    fn nvim_fpip_reset_binding_curwin();
    fn nvim_fpip_curwin_set_cursor_col(col: ColnrT);
    fn nvim_fpip_curwin_set_curswant();
    fn nvim_fpip_curwin_ptr() -> WinHandle;
    fn nvim_fpip_get_buf_fnum(buf: BufHandle) -> c_int;
    fn nvim_fpip_set_curwin_cursor_lnum(lnum: LinenrT);
    fn nvim_fpip_msg_silent() -> c_int;

    // rs_ functions from other Rust crates
    fn rs_magic_isset() -> c_int;
    fn rs_compl_status_adding() -> c_int;
    fn rs_compl_status_sol() -> c_int;
    fn rs_ins_compl_len() -> c_int;
    fn rs_ins_compl_interrupted() -> c_int;
    fn rs_ins_compl_check_keys(frequency: c_int, in_compl_func: c_int);
    fn rs_find_word_start(ptr: *mut c_char) -> *mut c_char;
    fn rs_find_word_end(ptr: *mut c_char) -> *mut c_char;
    fn rs_win_valid(win: WinHandle) -> c_int;
}

// =============================================================================
// Regmatch helper struct (mirrors C regmatch_T layout we need)
// =============================================================================

/// Opaque handle to C `regmatch_T`. We access it only through raw pointers
/// because the layout is complex and we only need startp/endp/regprog/rm_ic.
struct Regmatch {
    /// Heap-allocated `regmatch_T` (layout owned by C, size = size_of::<regmatch_T>())
    ptr: *mut c_void,
}

/// Size of regmatch_T as computed from the C side
const REGMATCH_T_SIZE: usize = 128; // conservative upper bound; actual is ~80 bytes

impl Regmatch {
    fn new_zeroed() -> Self {
        let ptr = unsafe { xcalloc(1, REGMATCH_T_SIZE) };
        Regmatch { ptr }
    }

    fn regprog(&self) -> *mut c_void {
        // regprog is the first field of regmatch_T
        unsafe { *(self.ptr as *mut *mut c_void) }
    }

    fn set_regprog(&mut self, prog: *mut c_void) {
        unsafe {
            *(self.ptr as *mut *mut c_void) = prog;
        }
    }

    fn set_rm_ic(&mut self, ic: c_int) {
        // rm_ic follows regprog (pointer) -- offset depends on arch.
        // On 64-bit: regprog is 8 bytes, then rm_ic (int) at offset 8.
        unsafe {
            let rm_ic_ptr = (self.ptr as *mut u8).add(8) as *mut c_int;
            *rm_ic_ptr = ic;
        }
    }

    fn startp(&self, idx: usize) -> *mut c_char {
        // startp[] array starts after regprog (8) + rm_ic (4) + padding (4) = 16
        // startp is array of pointers: offset 16 + idx * 8
        unsafe {
            let base = (self.ptr as *const u8).add(16 + idx * 8) as *const *mut c_char;
            *base
        }
    }

    fn endp(&self, idx: usize) -> *mut c_char {
        // endp[] follows startp[NSUBEXP] where NSUBEXP=10
        // offset = 16 + 10*8 + idx*8 = 16 + 80 + idx*8 = 96 + idx*8
        unsafe {
            let base = (self.ptr as *const u8).add(96 + idx * 8) as *const *mut c_char;
            *base
        }
    }

    fn regexec(&self, line: *const c_char, col: ColnrT) -> bool {
        unsafe { vim_regexec(self.ptr, line, col) != 0 }
    }

    fn free_regprog(&mut self) {
        let prog = self.regprog();
        if !prog.is_null() {
            unsafe {
                vim_regfree(prog);
            }
            self.set_regprog(std::ptr::null_mut());
        }
    }
}

impl Drop for Regmatch {
    fn drop(&mut self) {
        self.free_regprog();
        unsafe {
            xfree(self.ptr);
        }
    }
}

// =============================================================================
// SearchedFile array
// =============================================================================

struct SearchedFile {
    fp: FilePtr,
    name: *mut c_char,
    lnum: LinenrT,
    matched: bool,
}

impl SearchedFile {
    fn zeroed() -> Self {
        SearchedFile {
            fp: std::ptr::null_mut(),
            name: std::ptr::null_mut(),
            lnum: 0,
            matched: false,
        }
    }
}

// =============================================================================
// Helper: copy a buffer line into a scratch buffer
// =============================================================================

unsafe fn get_line_and_copy(lnum: LinenrT, buf: *mut c_char) -> *mut c_char {
    xstrlcpy(buf, ml_get(lnum), LSIZE as usize);
    buf
}

// =============================================================================
// Helper: show_pat_in_path
// =============================================================================

unsafe fn show_pat_in_path(
    line: *mut c_char,
    type_: c_int,
    did_show: bool,
    action: c_int,
    fp: FilePtr,
    lnum: *mut LinenrT,
    count: c_int,
) {
    if did_show {
        msg_putchar(b'\n' as c_int);
    } else if nvim_fpip_msg_silent() == 0 {
        gotocmdline(true);
    }
    if got_int {
        return;
    }
    let mut line = line;
    let mut linelen = strlen(line);
    loop {
        let p = line.add(linelen.saturating_sub(1));
        if !fp.is_null() {
            let p_val = *p;
            if p >= line && p_val == b'\n' as c_char {
                *p = b'\0' as c_char;
                linelen = linelen.saturating_sub(1);
            }
            let p2 = if linelen > 0 {
                line.add(linelen.saturating_sub(1))
            } else {
                line
            };
            if p2 >= line && *p2 == b'\r' as c_char {
                *p2 = b'\0' as c_char;
            }
            // Overwrite trailing chars
            *line.add(linelen) = b'\0' as c_char;
        }
        if action == ACTION_SHOW_ALL {
            // print "  N: " prefix
            let fmt = c"%3d: ".as_ptr();
            vim_snprintf(
                std::ptr::addr_of_mut!(IObuff_ps).cast::<c_char>(),
                IOSIZE,
                fmt,
                count,
            );
            msg_puts(std::ptr::addr_of!(IObuff_ps).cast::<c_char>());
            let fmt2 = c"%4d".as_ptr();
            vim_snprintf(
                std::ptr::addr_of_mut!(IObuff_ps).cast::<c_char>(),
                IOSIZE,
                fmt2,
                *lnum,
            );
            msg_puts_hl(std::ptr::addr_of!(IObuff_ps).cast::<c_char>(), HLF_N, 0);
            msg_puts(c" ".as_ptr());
        }
        msg_prt_line(line, 0);
        let p_last = if linelen > 0 {
            *line.add(linelen.saturating_sub(1))
        } else {
            0
        };
        if got_int || type_ != FIND_DEFINE || p_last != b'\\' as c_char {
            break;
        }
        if !fp.is_null() {
            if vim_fgets(line, LSIZE, fp) != 0 {
                break;
            }
            linelen = strlen(line);
            *lnum += 1;
        } else {
            *lnum += 1;
            if *lnum > nvim_fpip_get_curbuf_ml_line_count() {
                break;
            }
            line = ml_get(*lnum);
            linelen = strlen(line);
        }
        msg_putchar(b'\n' as c_int);
    }
}

// =============================================================================
// Main exported function
// =============================================================================

/// Rust replacement for `find_pattern_in_path()`.
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[unsafe(export_name = "find_pattern_in_path")]
pub unsafe extern "C" fn rs_find_pattern_in_path(
    ptr: *const c_char,
    dir: c_int,
    len: usize,
    whole: c_int,
    skip_comments: c_int,
    type_: c_int,
    count: c_int,
    action: c_int,
    start_lnum: LinenrT,
    end_lnum: LinenrT,
    forceit: c_int,
    silent: c_int,
) {
    let whole = whole != 0;
    let skip_comments = skip_comments != 0;
    let forceit = forceit != 0;
    let silent = silent != 0;

    // ---------- Init phase ----------

    // Compile search pattern (unless CHECK_PATH or FIND_DEFINE at start-of-line)
    let mut regmatch = Regmatch::new_zeroed();
    if type_ != CHECK_PATH && type_ != FIND_DEFINE && rs_compl_status_sol() == 0 {
        let patsize = len + 5;
        let pat = xmalloc(patsize) as *mut c_char;
        let pat_fmt = if whole {
            c"\\<%.*s\\>".as_ptr()
        } else {
            c"%.*s".as_ptr()
        };
        vim_snprintf(pat, patsize, pat_fmt, len as c_int, ptr);
        let rm_ic = ignorecase(pat);
        regmatch.set_rm_ic(rm_ic);
        let prog = vim_regcomp(pat, if rs_magic_isset() != 0 { RE_MAGIC } else { 0 });
        xfree(pat as *mut c_void);
        if prog.is_null() {
            return;
        }
        regmatch.set_regprog(prog);
    }

    // Compile include pattern
    let mut incl_regmatch = Regmatch::new_zeroed();
    let curbuf_b_p_inc = nvim_fpip_get_curbuf_b_p_inc();
    let p_inc_global = nvim_fpip_get_p_inc();
    let inc_opt: *mut c_char = if *curbuf_b_p_inc != 0 {
        curbuf_b_p_inc
    } else {
        p_inc_global
    };

    if *inc_opt != 0 {
        let prog = vim_regcomp(inc_opt, if rs_magic_isset() != 0 { RE_MAGIC } else { 0 });
        if prog.is_null() {
            return;
        }
        incl_regmatch.set_regprog(prog);
        incl_regmatch.set_rm_ic(0);
    }

    // Compile define pattern
    let mut def_regmatch = Regmatch::new_zeroed();
    let curbuf_b_p_def = nvim_fpip_get_curbuf_b_p_def();
    let p_def_global = nvim_fpip_get_p_def();
    if type_ == FIND_DEFINE && (*curbuf_b_p_def != 0 || *p_def_global != 0) {
        let def_pat = if *curbuf_b_p_def != 0 {
            curbuf_b_p_def
        } else {
            p_def_global
        };
        let prog = vim_regcomp(def_pat, if rs_magic_isset() != 0 { RE_MAGIC } else { 0 });
        if prog.is_null() {
            return;
        }
        def_regmatch.set_regprog(prog);
        def_regmatch.set_rm_ic(0);
    }

    // Initialize file array
    let max_path_depth_init: usize = 50;
    let mut max_path_depth = max_path_depth_init;
    let mut old_files = max_path_depth;
    let mut files: Vec<SearchedFile> = (0..max_path_depth)
        .map(|_| SearchedFile::zeroed())
        .collect();

    // Clamp lnum range
    let ml_line_count = nvim_fpip_get_curbuf_ml_line_count();
    let end_lnum = end_lnum.min(ml_line_count);
    let mut lnum = start_lnum.min(end_lnum);

    // Local copies of state
    let mut depth: i32 = -1;
    let mut depth_displayed: i32 = -1;
    let mut match_count: c_int = 1;
    let mut curr_fname = nvim_fpip_get_curbuf_fname();
    let mut prev_fname: *mut c_char = std::ptr::null_mut();
    let mut did_show = false;
    let mut found = false;
    let mut dir_local = dir;
    let l_g_do_tagpreview = nvim_get_g_do_tagpreview();

    // Scratch line buffer
    let file_line = xmalloc(LSIZE as usize) as *mut c_char;
    let mut line = get_line_and_copy(lnum, file_line);

    // ---------- Main loop ----------
    'outer: loop {
        if incl_regmatch.regprog().is_null() || !incl_regmatch.regexec(line, 0) {
            // No include match -- check for pattern match on this line.
            // We use a nested label to handle the `goto search_line` case.
            let mut search_line_p = line;
            'search_line: loop {
                let mut define_matched = false;
                if !def_regmatch.regprog().is_null() && def_regmatch.regexec(line, 0) {
                    let mut p = def_regmatch.endp(0);
                    while *p != 0 && vim_iswordc(*p as u8) == 0 {
                        p = p.add(1);
                    }
                    search_line_p = p;
                    define_matched = true;
                }

                let mut matched = false;
                let mut startp: *mut c_char = std::ptr::null_mut();

                if def_regmatch.regprog().is_null() || define_matched {
                    if define_matched || rs_compl_status_sol() != 0 {
                        startp = skipwhite(search_line_p) as *mut c_char;
                        let p_ic_val = nvim_search_get_p_ic();
                        matched = if p_ic_val != 0 {
                            mb_strnicmp(startp, ptr, len) == 0
                        } else {
                            strncmp(startp, ptr, len) == 0
                        };
                        if matched
                            && define_matched
                            && whole
                            && vim_iswordc(*startp.add(len) as u8) != 0
                        {
                            matched = false;
                        }
                    } else if !regmatch.regprog().is_null() {
                        let col = (search_line_p as usize - line as usize) as ColnrT;
                        if regmatch.regexec(line, col) {
                            matched = true;
                            startp = regmatch.startp(0);
                            if skip_comments {
                                // Skip if in comment
                                if (*line != b'#' as c_char
                                    || strncmp(skipwhite(line.add(1)), c"define".as_ptr(), 6) != 0)
                                    && get_leader_len(line, std::ptr::null_mut(), 0, 1) != 0
                                {
                                    matched = false;
                                }
                                let p2 = skipwhite(line) as *mut c_char;
                                if matched
                                    || (*p2 == b'/' as c_char && *p2.add(1) == b'*' as c_char)
                                    || *p2 == b'*' as c_char
                                {
                                    let mut p3 = line;
                                    while *p3 != 0 && p3 < startp {
                                        if matched
                                            && *p3 == b'/' as c_char
                                            && (*p3.add(1) == b'*' as c_char
                                                || *p3.add(1) == b'/' as c_char)
                                        {
                                            matched = false;
                                            if *p3.add(1) == b'/' as c_char {
                                                break;
                                            }
                                            p3 = p3.add(1);
                                        } else if !matched
                                            && *p3 == b'*' as c_char
                                            && *p3.add(1) == b'/' as c_char
                                        {
                                            matched = true;
                                            p3 = p3.add(1);
                                        }
                                        p3 = p3.add(1);
                                    }
                                }
                            }
                        }
                    }
                }

                if matched {
                    // Handle the matched line
                    // This section handles `goto exit_matched` via `break 'matched`
                    let done = 'matched: {
                        if action == ACTION_EXPAND {
                            if depth == -1 && lnum == nvim_fpip_curwin_cursor_lnum() {
                                // break outer loop
                                break 'outer;
                            }
                            found = true;
                            let aux_start = startp;
                            let mut p = startp;
                            if rs_compl_status_adding() != 0 {
                                let compl_len = rs_ins_compl_len();
                                if strlen(p) as c_int >= compl_len {
                                    p = p.add(compl_len as usize);
                                    if vim_iswordp(p) != 0 {
                                        break 'matched false; // goto exit_matched
                                    }
                                    p = rs_find_word_start(p);
                                }
                            }
                            p = rs_find_word_end(p);
                            let mut i = (p as usize - aux_start as usize) as c_int;
                            let mut aux = aux_start;
                            if rs_compl_status_adding() != 0 && i == rs_ins_compl_len() {
                                // Copy current match into IObuff, fetch next line
                                strncpy(
                                    std::ptr::addr_of_mut!(IObuff_ps).cast::<c_char>(),
                                    aux,
                                    i as usize,
                                );
                                let got_next = if depth < 0 {
                                    if lnum >= end_lnum {
                                        break 'matched false; // goto exit_matched
                                    }
                                    lnum += 1;
                                    line = get_line_and_copy(lnum, file_line);
                                    false
                                } else if vim_fgets(file_line, LSIZE, files[depth as usize].fp) != 0
                                {
                                    break 'matched false; // goto exit_matched
                                } else {
                                    line = file_line;
                                    false
                                };
                                let _ = got_next;
                                // already = skipwhite(line) -- track this
                                let already_p = skipwhite(line) as *mut c_char;
                                // Next iteration should use already_p as line start
                                aux = already_p;
                                p = already_p;
                                p = rs_find_word_start(p);
                                p = rs_find_word_end(p);
                                if p > aux {
                                    let iobuf = std::ptr::addr_of_mut!(IObuff_ps).cast::<c_char>();
                                    let aux_char = *aux;
                                    let iobuff_last = *iobuf.add(i as usize - 1) as u8;
                                    if aux_char != b')' as c_char && iobuff_last != TAB {
                                        if iobuff_last != b' ' {
                                            *iobuf.add(i as usize) = b' ' as c_char;
                                            i += 1;
                                        }
                                        if nvim_fpip_get_p_js() != 0 {
                                            let prev = *iobuf.add(i as usize - 2) as u8;
                                            if prev == b'.' || prev == b'?' || prev == b'!' {
                                                *iobuf.add(i as usize) = b' ' as c_char;
                                                i += 1;
                                            }
                                        }
                                    }
                                    let span = p as usize - aux as usize;
                                    let available = IOSIZE.saturating_sub(i as usize);
                                    let copy_len = span.min(available.saturating_sub(1));
                                    strncpy(iobuf.add(i as usize), aux, copy_len);
                                    i += copy_len as c_int;
                                }
                                *std::ptr::addr_of_mut!(IObuff_ps)
                                    .cast::<c_char>()
                                    .add(i as usize) = 0;
                                aux = std::ptr::addr_of_mut!(IObuff_ps).cast::<c_char>();
                                if i == rs_ins_compl_len() {
                                    break 'matched false; // goto exit_matched
                                }

                                // After fetching next line, handle `already` path:
                                // The C code sets `already = aux` so the outer loop
                                // doesn't advance the line again. We replicate by
                                // continuing the outer loop without advancing.
                                let curbuf_fname = nvim_fpip_get_curbuf_fname();
                                let fname_arg = if curr_fname == curbuf_fname {
                                    std::ptr::null()
                                } else {
                                    curr_fname as *const c_char
                                };
                                let add_r = ins_compl_add_infercase(
                                    aux,
                                    i,
                                    nvim_search_get_p_ic(),
                                    fname_arg,
                                    dir_local,
                                    0, // cont_s_ipos not set in this sub-path
                                    0,
                                );
                                if add_r == OK {
                                    dir_local = FORWARD;
                                } else if add_r == FAIL {
                                    break 'outer;
                                }
                                // skip line advance
                                line_breakcheck();
                                rs_ins_compl_check_keys(30, 0);
                                if got_int || rs_ins_compl_interrupted() != 0 {
                                    break 'outer;
                                }
                                continue 'outer;
                            }
                            let curbuf_fname = nvim_fpip_get_curbuf_fname();
                            let fname_arg = if curr_fname == curbuf_fname {
                                std::ptr::null()
                            } else {
                                curr_fname as *const c_char
                            };
                            let cont_s_ipos = 0; // set above only in the adding+fetched-next-line path
                            let add_r = ins_compl_add_infercase(
                                aux,
                                i,
                                nvim_search_get_p_ic(),
                                fname_arg,
                                dir_local,
                                cont_s_ipos,
                                0,
                            );
                            if add_r == OK {
                                dir_local = FORWARD;
                            } else if add_r == FAIL {
                                break 'outer;
                            }
                        } else if action == ACTION_SHOW_ALL {
                            found = true;
                            if !did_show {
                                gotocmdline(true);
                            }
                            if curr_fname != prev_fname {
                                if did_show {
                                    msg_putchar(b'\n' as c_int);
                                }
                                if !got_int {
                                    msg_home_replace(curr_fname);
                                }
                                prev_fname = curr_fname;
                            }
                            did_show = true;
                            if !got_int {
                                let fp_arg = if depth == -1 {
                                    std::ptr::null_mut()
                                } else {
                                    files[depth as usize].fp
                                };
                                let lnum_ref = if depth == -1 {
                                    &mut lnum as *mut LinenrT
                                } else {
                                    &mut files[depth as usize].lnum as *mut LinenrT
                                };
                                show_pat_in_path(
                                    line,
                                    type_,
                                    true,
                                    action,
                                    fp_arg,
                                    lnum_ref,
                                    match_count,
                                );
                                match_count += 1;
                            }
                            for f in files.iter_mut().take(depth as usize + 1) {
                                f.matched = true;
                            }
                        } else {
                            // ACTION_GOTO, ACTION_SPLIT, ACTION_SHOW (count-based)
                            let count_val = count - match_count;
                            // decrement global match_count to track st->count
                            match_count += 1;
                            if count_val <= 0 {
                                found = true;
                                let cursor_lnum = nvim_fpip_curwin_cursor_lnum();
                                if depth == -1 && lnum == cursor_lnum && l_g_do_tagpreview == 0 {
                                    emsg(c"E387: Match is on current line".as_ptr());
                                } else if action == ACTION_SHOW {
                                    let fp_arg = if depth == -1 {
                                        std::ptr::null_mut()
                                    } else {
                                        files[depth as usize].fp
                                    };
                                    let lnum_ref = if depth == -1 {
                                        &mut lnum as *mut LinenrT
                                    } else {
                                        &mut files[depth as usize].lnum as *mut LinenrT
                                    };
                                    show_pat_in_path(
                                        line, type_, did_show, action, fp_arg, lnum_ref, 1,
                                    );
                                    did_show = true;
                                } else {
                                    let mut curwin_save: WinHandle = std::ptr::null_mut();
                                    if l_g_do_tagpreview != 0 {
                                        curwin_save = nvim_fpip_curwin_ptr();
                                        prepare_tagpreview(1);
                                    }
                                    if action == ACTION_SPLIT {
                                        if win_split(0, 0) == FAIL {
                                            break 'outer;
                                        }
                                        nvim_fpip_reset_binding_curwin();
                                    }
                                    if depth == -1 {
                                        if l_g_do_tagpreview != 0 {
                                            if rs_win_valid(curwin_save) == 0 {
                                                break 'outer;
                                            }
                                            let buf_fnum = nvim_fpip_get_buf_fnum(curwin_save);
                                            if getfile(
                                                buf_fnum,
                                                std::ptr::null(),
                                                std::ptr::null(),
                                                1,
                                                lnum,
                                                forceit as c_int,
                                            ) > 0
                                            {
                                                break 'outer;
                                            }
                                        } else {
                                            setpcmark();
                                        }
                                        nvim_fpip_set_curwin_cursor_lnum(lnum);
                                        check_cursor(curwin);
                                    } else {
                                        let fname = files[depth as usize].name;
                                        let file_lnum = files[depth as usize].lnum;
                                        if getfile(
                                            0,
                                            fname,
                                            std::ptr::null(),
                                            1,
                                            file_lnum,
                                            forceit as c_int,
                                        ) > 0
                                        {
                                            break 'outer;
                                        }
                                        nvim_fpip_set_curwin_cursor_lnum(file_lnum);
                                    }
                                    if action != ACTION_SHOW {
                                        nvim_fpip_curwin_set_cursor_col(
                                            (startp as usize - line as usize) as ColnrT,
                                        );
                                        nvim_fpip_curwin_set_curswant();
                                    }
                                    if l_g_do_tagpreview != 0 {
                                        let cur = nvim_fpip_curwin_ptr();
                                        if cur != curwin_save && rs_win_valid(curwin_save) != 0 {
                                            validate_cursor(curwin);
                                            redraw_later(curwin, UPD_VALID);
                                            win_enter(curwin_save, 1);
                                        }
                                    }
                                    break 'outer;
                                }
                                // For ACTION_SHOW: also check/update cursor col
                                if action != ACTION_SHOW {
                                    nvim_fpip_curwin_set_cursor_col(
                                        (startp as usize - line as usize) as ColnrT,
                                    );
                                    nvim_fpip_curwin_set_curswant();
                                }
                                break 'outer;
                            }
                        }
                        true // matched handling done normally
                    };
                    // exit_matched: label equivalent
                    // If `done` is false we jumped out of matched handling early.
                    let _ = done;

                    // After exit_matched: if expanding and not sol, try searching
                    // again on the same line (goto search_line)
                    if def_regmatch.regprog().is_null()
                        && action == ACTION_EXPAND
                        && rs_compl_status_sol() == 0
                        && !startp.is_null()
                        && *startp != 0
                        && *startp.add(utfc_ptr2len(startp) as usize) != 0
                    {
                        // goto search_line: re-enter 'search_line loop
                        search_line_p = startp.add(utfc_ptr2len(startp) as usize);
                        continue 'search_line;
                    }
                }

                // Done with search_line processing for this iteration
                break 'search_line;
            } // end 'search_line loop
        } else {
            // Include regex matched -- process include file
            let p_fname: *const c_char = {
                let curbuf_fname = nvim_fpip_get_curbuf_fname();
                if curr_fname == curbuf_fname {
                    nvim_fpip_get_curbuf_ffname()
                } else {
                    curr_fname
                }
            };

            // Determine new_fname for the included file
            let has_zs = strstr(inc_opt, c"\\zs".as_ptr());
            let mut new_fname: *mut c_char = if !has_zs.is_null() {
                let startp = incl_regmatch.startp(0);
                let endp = incl_regmatch.endp(0);
                let flen = endp as usize - startp as usize;
                find_file_name_in_path(startp, flen, FNAME_EXP | FNAME_INCL | FNAME_REL, 1, p_fname)
            } else {
                file_name_in_line(
                    incl_regmatch.endp(0),
                    0,
                    FNAME_EXP | FNAME_INCL | FNAME_REL,
                    1,
                    p_fname,
                    std::ptr::null_mut(),
                )
            };

            let mut already_searched = false;
            if !new_fname.is_null() {
                let mut i = 0usize;
                loop {
                    if i == (depth + 1) as usize {
                        i = old_files;
                    }
                    if i == max_path_depth {
                        break;
                    }
                    if path_full_compare(new_fname, files[i].name, 1, 1) & K_EQUAL_FILES != 0 {
                        if type_ != CHECK_PATH && action == ACTION_SHOW_ALL && files[i].matched {
                            msg_putchar(b'\n' as c_int);
                            if !got_int {
                                msg_home_replace(new_fname);
                                msg_puts(c" (includes previously listed match)".as_ptr()
                                    as *const c_char);
                                prev_fname = std::ptr::null_mut();
                            }
                        }
                        xfree(new_fname as *mut c_void);
                        new_fname = std::ptr::null_mut();
                        already_searched = true;
                        break;
                    }
                    i += 1;
                }
            }

            if type_ == CHECK_PATH
                && (action == ACTION_SHOW_ALL || (new_fname.is_null() && !already_searched))
            {
                if did_show {
                    msg_putchar(b'\n' as c_int);
                } else {
                    gotocmdline(true);
                    msg_puts_title(c"--- Included files ".as_ptr());
                    if action != ACTION_SHOW_ALL {
                        msg_puts_title(c"not found ".as_ptr());
                    }
                    msg_puts_title(c"in path ---\n".as_ptr());
                }
                did_show = true;
                let mut dd = depth_displayed;
                while dd < depth && !got_int {
                    dd += 1;
                    for _ in 0..dd {
                        msg_puts(c"  ".as_ptr());
                    }
                    msg_home_replace(files[dd as usize].name);
                    msg_puts(c" -->\n".as_ptr());
                }
                depth_displayed = dd;
                if !got_int {
                    for _ in 0..=(depth_displayed) {
                        msg_puts(c"  ".as_ptr());
                    }
                    if !new_fname.is_null() {
                        msg_outtrans(new_fname, HLF_D, false);
                    } else {
                        // Show the text from the include line
                        let (p_show, i_show): (*const c_char, usize) = if !has_zs.is_null() {
                            let s = incl_regmatch.startp(0);
                            let e = incl_regmatch.endp(0);
                            (s, (e as usize - s as usize))
                        } else {
                            let mut p2 = incl_regmatch.endp(0);
                            while *p2 != 0 && vim_isfilec(*p2 as u8) == 0 {
                                p2 = p2.add(1);
                            }
                            let mut i2 = 0usize;
                            while vim_isfilec(*p2.add(i2) as u8) != 0 {
                                i2 += 1;
                            }
                            if i2 == 0 {
                                p2 = incl_regmatch.endp(0);
                                i2 = strlen(p2);
                            } else {
                                let base = incl_regmatch.endp(0);
                                if p2 > base {
                                    if *p2.offset(-1) == b'"' as c_char
                                        || *p2.offset(-1) == b'<' as c_char
                                    {
                                        p2 = p2.offset(-1);
                                        i2 += 1;
                                    }
                                    if *p2.add(i2) == b'"' as c_char
                                        || *p2.add(i2) == b'>' as c_char
                                    {
                                        i2 += 1;
                                    }
                                }
                            }
                            (p2, i2)
                        };
                        let save_char = *p_show.add(i_show);
                        *(p_show.add(i_show) as *mut c_char) = 0;
                        msg_outtrans(p_show, HLF_D, false);
                        *(p_show.add(i_show) as *mut c_char) = save_char;
                    }
                    if new_fname.is_null() && action == ACTION_SHOW_ALL {
                        if already_searched {
                            msg_puts(c"  (Already listed)".as_ptr());
                        } else {
                            msg_puts(c"  NOT FOUND".as_ptr());
                        }
                    }
                }
            }

            if !new_fname.is_null() {
                // Grow files array if needed
                if depth as usize + 1 == old_files {
                    let new_size = max_path_depth * 2;
                    let mut bigger: Vec<SearchedFile> =
                        (0..new_size).map(|_| SearchedFile::zeroed()).collect();
                    for i in 0..=(depth as usize) {
                        bigger[i].fp = files[i].fp;
                        bigger[i].name = files[i].name;
                        bigger[i].lnum = files[i].lnum;
                        bigger[i].matched = files[i].matched;
                        // prevent double-free
                        files[i].fp = std::ptr::null_mut();
                        files[i].name = std::ptr::null_mut();
                    }
                    for i in 0..max_path_depth {
                        bigger[i + max_path_depth].fp = files[old_files + i].fp;
                        bigger[i + max_path_depth].name = files[old_files + i].name;
                        bigger[i + max_path_depth].lnum = files[old_files + i].lnum;
                        bigger[i + max_path_depth].matched = files[old_files + i].matched;
                        files[old_files + i].fp = std::ptr::null_mut();
                        files[old_files + i].name = std::ptr::null_mut();
                    }
                    old_files += max_path_depth;
                    max_path_depth = new_size;
                    // Drain remaining (old_files..old_max range) -- already transferred
                    files = bigger;
                }

                let fp = os_fopen(new_fname, c"r".as_ptr());
                if fp.is_null() {
                    xfree(new_fname as *mut c_void);
                } else {
                    depth += 1;
                    if depth as usize == old_files {
                        xfree(files[old_files].name as *mut c_void);
                        old_files += 1;
                    }
                    files[depth as usize].fp = fp;
                    files[depth as usize].name = new_fname;
                    curr_fname = new_fname;
                    files[depth as usize].lnum = 0;
                    files[depth as usize].matched = false;
                    if action == ACTION_EXPAND && !shortmess(SHM_COMPLETIONSCAN) && !silent {
                        msg_hist_off_set(1);
                        let fmt = c"Scanning included file: %s".as_ptr();
                        vim_snprintf(
                            std::ptr::addr_of_mut!(IObuff_ps).cast::<c_char>(),
                            IOSIZE,
                            fmt,
                            new_fname,
                        );
                        msg_trunc(std::ptr::addr_of!(IObuff_ps).cast::<c_char>(), 1, HLF_R);
                    } else if p_verbose >= 5 {
                        verbose_enter();
                        smsg(0, c"Searching included file %s".as_ptr(), new_fname);
                        verbose_leave();
                    }
                }
            }
        }

        // Advance to next line
        line_breakcheck();
        if action == ACTION_EXPAND {
            rs_ins_compl_check_keys(30, 0);
        }
        if got_int || rs_ins_compl_interrupted() != 0 {
            break 'outer;
        }

        // Read next line from included file (pop stack when file exhausted)
        while depth >= 0 && vim_fgets(file_line, LSIZE, files[depth as usize].fp) != 0 {
            fclose(files[depth as usize].fp);
            files[depth as usize].fp = std::ptr::null_mut();
            old_files -= 1;
            files[old_files].name = files[depth as usize].name;
            files[old_files].matched = files[depth as usize].matched;
            files[depth as usize].name = std::ptr::null_mut();
            depth -= 1;
            curr_fname = if depth == -1 {
                nvim_fpip_get_curbuf_fname()
            } else {
                files[depth as usize].name
            };
            if depth_displayed > depth {
                depth_displayed = depth;
            }
        }

        if depth >= 0 {
            // Reading from included file
            files[depth as usize].lnum += 1;
            line = file_line;
            let mut i = strlen(line) as usize;
            if i > 0 && *line.add(i - 1) == b'\n' as c_char {
                i -= 1;
                *line.add(i) = 0;
            }
            if i > 0 && *line.add(i - 1) == b'\r' as c_char {
                i -= 1;
                *line.add(i) = 0;
            }
        } else {
            // Reading from current buffer
            lnum += 1;
            if lnum > end_lnum {
                break 'outer;
            }
            line = get_line_and_copy(lnum, file_line);
        }
    } // end 'outer loop

    // Cleanup open file handles and names
    if depth >= 0 {
        for f in files.iter_mut().take(depth as usize + 1) {
            if !f.fp.is_null() {
                fclose(f.fp);
            }
            xfree(f.name as *mut c_void);
        }
    }
    for f in files
        .iter_mut()
        .skip(old_files)
        .take(max_path_depth - old_files)
    {
        xfree(f.name as *mut c_void);
    }

    xfree(file_line as *mut c_void);

    // Final messages
    if type_ == CHECK_PATH {
        if !did_show {
            if action != ACTION_SHOW_ALL {
                msg(c"All included files were found".as_ptr(), 0);
            } else {
                msg(c"No included files".as_ptr(), 0);
            }
        }
    } else if !found && action != ACTION_EXPAND && !silent {
        if got_int || rs_ins_compl_interrupted() != 0 {
            emsg(c"Interrupted".as_ptr());
        } else if type_ == FIND_DEFINE {
            emsg(c"E388: Couldn't find definition".as_ptr());
        } else {
            emsg(c"E389: Couldn't find pattern".as_ptr());
        }
    }

    if action == ACTION_SHOW || action == ACTION_SHOW_ALL {
        msg_end();
    }
}
