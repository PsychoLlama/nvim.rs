//! Backtracking (BT) regex engine pattern compilation.
//!
//! This module implements the pattern compilation phase of the BT regex engine.
//! It converts regex patterns into bytecode using a two-pass algorithm:
//!
//! 1. **First pass**: Calculate the size of the bytecode without emitting
//! 2. **Second pass**: Emit the actual bytecode
//!
//! # Bytecode Format
//!
//! The BT engine uses a linked-list bytecode format where each instruction
//! consists of:
//! - 1 byte opcode
//! - 2 bytes "next" pointer (offset to next instruction)
//! - Optional operand data (varies by opcode)
//!
//! # Key Functions
//!
//! - [`RegCompiler`]: Main compiler state machine
//! - [`emit_node`]: Emit a new bytecode node
//! - [`emit_char`]: Emit a character/byte to the bytecode stream

use std::ffi::c_int;
use std::ptr;

use crate::bt_opcodes::{END, HASWIDTH, SIMPLE, SPSTART};

// =============================================================================
// Constants
// =============================================================================

/// Magic byte at the start of compiled regex
pub const REGMAGIC: u8 = 0x1d;

/// Special value meaning "just count, don't emit"
pub const JUST_CALC_SIZE: *mut u8 = std::ptr::dangling_mut::<u8>();

/// Maximum size of a compiled regex pattern
pub const MAX_REGSIZE: usize = 65534;

/// Offset for the "next" pointer in a node
pub const NEXT_OFFSET: usize = 1;

/// Size of a minimal node (opcode + next pointer)
pub const NODE_SIZE: usize = 3;

// =============================================================================
// Compilation Flags
// =============================================================================

/// Flags for regex compilation
#[derive(Clone, Copy, Debug, Default)]
pub struct CompileFlags {
    /// Pattern contains something that has width
    pub has_width: bool,
    /// Pattern is simple enough for STAR/PLUS operand
    pub simple: bool,
    /// Pattern starts with * or +
    pub sp_start: bool,
    /// Pattern contains \n
    pub has_nl: bool,
    /// Pattern contains look-behind
    pub has_lookbh: bool,
}

impl CompileFlags {
    /// Convert to C-style flag bits
    pub fn to_bits(&self) -> c_int {
        let mut flags = 0;
        if self.has_width {
            flags |= HASWIDTH;
        }
        if self.simple {
            flags |= SIMPLE;
        }
        if self.sp_start {
            flags |= SPSTART;
        }
        if self.has_nl {
            // Note: HASNL is defined in bt_opcodes
            flags |= 0x8; // HASNL
        }
        if self.has_lookbh {
            flags |= 0x10; // HASLOOKBH
        }
        flags
    }

    /// Create from C-style flag bits
    pub fn from_bits(bits: c_int) -> Self {
        Self {
            has_width: (bits & HASWIDTH) != 0,
            simple: (bits & SIMPLE) != 0,
            sp_start: (bits & SPSTART) != 0,
            has_nl: (bits & 0x8) != 0,      // HASNL
            has_lookbh: (bits & 0x10) != 0, // HASLOOKBH
        }
    }

    /// Combine flags (OR operation)
    pub fn combine(&mut self, other: &Self) {
        self.has_width |= other.has_width;
        self.simple |= other.simple;
        self.sp_start |= other.sp_start;
        self.has_nl |= other.has_nl;
        self.has_lookbh |= other.has_lookbh;
    }
}

// =============================================================================
// Bytecode Helpers
// =============================================================================

/// Get the opcode from a bytecode node
///
/// # Safety
/// `p` must be a valid pointer to a bytecode node
#[inline]
pub unsafe fn op(p: *const u8) -> c_int {
    if p.is_null() {
        END
    } else {
        c_int::from(*p)
    }
}

/// Get the "next" offset from a bytecode node
///
/// Returns the pointer to the next node, or NULL if none.
///
/// # Safety
/// `p` must be a valid pointer to a bytecode node with at least 3 bytes
#[inline]
pub unsafe fn next(p: *const u8) -> *const u8 {
    if p.is_null() {
        return ptr::null();
    }

    // Next pointer is stored as 2 bytes at offset 1
    let offset = ((*p.add(1) as u16) << 8) | (*p.add(2) as u16);

    if offset == 0 {
        ptr::null()
    } else {
        p.offset(offset as isize)
    }
}

/// Get the operand pointer from a bytecode node
///
/// The operand starts at offset 3 (after opcode + next pointer)
///
/// # Safety
/// `p` must be a valid pointer to a bytecode node
#[inline]
pub unsafe fn operand(p: *const u8) -> *const u8 {
    if p.is_null() {
        ptr::null()
    } else {
        p.add(NODE_SIZE)
    }
}

/// Get the operand pointer as mutable
///
/// # Safety
/// `p` must be a valid mutable pointer to a bytecode node
#[inline]
pub unsafe fn operand_mut(p: *mut u8) -> *mut u8 {
    if p.is_null() {
        ptr::null_mut()
    } else {
        p.add(NODE_SIZE)
    }
}

// =============================================================================
// RegCompiler - Main compilation state
// =============================================================================

/// Pattern compiler state.
///
/// This tracks the current position in the bytecode stream and
/// manages the two-pass compilation process.
pub struct RegCompiler {
    /// Current output position in bytecode
    /// Set to JUST_CALC_SIZE for first pass
    regcode: *mut u8,

    /// Size of bytecode generated
    regsize: usize,

    /// Input pattern
    pattern: *const u8,

    /// Current position in input pattern
    regparse: *const u8,

    /// Number of parentheses encountered
    regnpar: c_int,

    /// Number of \z() parentheses
    regnzpar: c_int,

    /// Number of complex brace quantifiers
    num_complex_braces: c_int,

    /// Pattern is too long
    reg_toolong: bool,

    /// RE flags (RE_MAGIC, etc.)
    #[allow(dead_code)]
    re_flags: c_int,

    /// Detected pattern flags (RF_HASNL, etc.)
    regflags: c_int,

    /// Has \z special
    re_has_z: bool,
}

impl RegCompiler {
    /// Create a new compiler for the given pattern.
    pub fn new(pattern: *const u8, re_flags: c_int) -> Self {
        Self {
            regcode: ptr::null_mut(),
            regsize: 0,
            pattern,
            regparse: pattern,
            regnpar: 1, // Start at 1, 0 is whole match
            regnzpar: 0,
            num_complex_braces: 0,
            reg_toolong: false,
            re_flags,
            regflags: 0,
            re_has_z: false,
        }
    }

    /// Start compilation (reset state for a pass).
    pub fn start(&mut self, counting: bool) {
        self.regparse = self.pattern;
        self.regsize = 0;
        self.regnpar = 1;
        self.regnzpar = 0;
        self.num_complex_braces = 0;
        self.reg_toolong = false;
        self.regflags = 0;
        self.re_has_z = false;

        if counting {
            self.regcode = JUST_CALC_SIZE;
        }
    }

    /// Check if we're in counting mode (first pass)
    #[inline]
    pub fn is_counting(&self) -> bool {
        self.regcode == JUST_CALC_SIZE
    }

    /// Emit a single byte to the bytecode stream.
    pub fn emit_byte(&mut self, b: u8) {
        if self.is_counting() {
            self.regsize += 1;
        } else if !self.regcode.is_null() {
            unsafe {
                *self.regcode = b;
                self.regcode = self.regcode.add(1);
            }
        }
    }

    /// Emit a bytecode node with the given opcode.
    ///
    /// Returns the location of the node (for later patching).
    pub fn emit_node(&mut self, opcode: c_int) -> *mut u8 {
        let ret = if self.is_counting() {
            self.regsize += NODE_SIZE;
            JUST_CALC_SIZE
        } else {
            let ret = self.regcode;
            unsafe {
                // Opcode
                *self.regcode = opcode as u8;
                self.regcode = self.regcode.add(1);

                // Next pointer (initially 0)
                *self.regcode = 0;
                self.regcode = self.regcode.add(1);
                *self.regcode = 0;
                self.regcode = self.regcode.add(1);
            }
            ret
        };

        // Check for pattern too long
        if self.regsize > MAX_REGSIZE {
            self.reg_toolong = true;
        }

        ret
    }

    /// Emit a node with a character argument.
    pub fn emit_node_with_arg(&mut self, opcode: c_int, arg: c_int) -> *mut u8 {
        let ret = self.emit_node(opcode);

        // Emit 4-byte argument
        self.emit_byte(((arg >> 24) & 0xff) as u8);
        self.emit_byte(((arg >> 16) & 0xff) as u8);
        self.emit_byte(((arg >> 8) & 0xff) as u8);
        self.emit_byte((arg & 0xff) as u8);

        ret
    }

    /// Insert a node before an existing location.
    ///
    /// Shifts the existing bytecode and inserts a new node.
    ///
    /// # Safety
    /// `before` must be a valid pointer within the bytecode buffer or null.
    pub unsafe fn insert_node(&mut self, opcode: c_int, before: *mut u8) {
        if self.is_counting() {
            self.regsize += NODE_SIZE;
        } else if !before.is_null() && !self.regcode.is_null() {
            // Shift existing bytes forward
            let shift = NODE_SIZE;
            let len = self.regcode.offset_from(before) as usize;
            if len > 0 {
                std::ptr::copy(before, before.add(shift), len);
            }
            self.regcode = self.regcode.add(shift);

            // Write new node at the insertion point
            *before = opcode as u8;
            *before.add(1) = 0; // next high byte
            *before.add(2) = 0; // next low byte
        }
    }

    /// Insert a node with a 4-byte argument before an existing location.
    ///
    /// # Safety
    /// `before` must be a valid pointer within the bytecode buffer or null.
    pub unsafe fn insert_node_with_arg(&mut self, opcode: c_int, arg: c_int, before: *mut u8) {
        if self.is_counting() {
            self.regsize += NODE_SIZE + 4;
        } else if !before.is_null() && !self.regcode.is_null() {
            // Shift existing bytes forward
            let shift = NODE_SIZE + 4;
            let len = self.regcode.offset_from(before) as usize;
            if len > 0 {
                std::ptr::copy(before, before.add(shift), len);
            }
            self.regcode = self.regcode.add(shift);

            // Write new node at the insertion point
            *before = opcode as u8;
            *before.add(1) = 0; // next high byte
            *before.add(2) = 0; // next low byte

            // Write argument
            *before.add(3) = ((arg >> 24) & 0xff) as u8;
            *before.add(4) = ((arg >> 16) & 0xff) as u8;
            *before.add(5) = ((arg >> 8) & 0xff) as u8;
            *before.add(6) = (arg & 0xff) as u8;
        }
    }

    /// Set the "next" pointer of a node to point to another node.
    ///
    /// # Safety
    /// `from` and `to` must be valid pointers within the bytecode buffer or null.
    pub unsafe fn set_next(&mut self, from: *mut u8, to: *mut u8) {
        if self.is_counting() || from.is_null() {
            return;
        }

        let offset = if to.is_null() {
            0i16
        } else {
            to.offset_from(from) as i16
        };

        // Store as big-endian 16-bit value
        *from.add(1) = ((offset >> 8) & 0xff) as u8;
        *from.add(2) = (offset & 0xff) as u8;
    }

    /// Chain a list of branches together via their "next" pointers.
    ///
    /// Walks the chain from `first` and sets the final node's next to `last`.
    ///
    /// # Safety
    /// `first` and `last` must be valid pointers within the bytecode buffer or null.
    pub unsafe fn chain(&mut self, first: *mut u8, last: *mut u8) {
        if self.is_counting() || first.is_null() {
            return;
        }

        let mut scan = first;
        loop {
            let temp = next(scan) as *mut u8;
            if temp.is_null() {
                break;
            }
            scan = temp;
        }
        self.set_next(scan, last);
    }

    /// Get the current bytecode size
    pub fn size(&self) -> usize {
        self.regsize
    }

    /// Check if compilation failed due to pattern being too long
    pub fn is_too_long(&self) -> bool {
        self.reg_toolong
    }

    /// Get the number of parentheses encountered
    pub fn paren_count(&self) -> c_int {
        self.regnpar
    }

    /// Increment and get the parenthesis number
    pub fn next_paren(&mut self) -> c_int {
        let n = self.regnpar;
        self.regnpar += 1;
        n
    }

    /// Get detected pattern flags
    pub fn flags(&self) -> c_int {
        self.regflags
    }

    /// Add a flag
    pub fn add_flag(&mut self, flag: c_int) {
        self.regflags |= flag;
    }

    /// Check if pattern has \z specials
    pub fn has_z(&self) -> bool {
        self.re_has_z
    }

    /// Mark pattern as having \z specials
    pub fn set_has_z(&mut self) {
        self.re_has_z = true;
    }

    /// Insert a BRACE_LIMITS node before the given location.
    ///
    /// BRACE_LIMITS stores two 4-byte values: minval and maxval.
    /// Total node size: 3 (opcode + next) + 8 (two 4-byte limits) = 11 bytes.
    ///
    /// # Safety
    /// `before` must be a valid pointer within the bytecode buffer or null.
    pub unsafe fn insert_limits(
        &mut self,
        opcode: c_int,
        minval: i32,
        maxval: i32,
        before: *mut u8,
    ) {
        if self.is_counting() {
            self.regsize += NODE_SIZE + 8; // opcode + next + 2x 4-byte limits
        } else if !before.is_null() && !self.regcode.is_null() {
            // Shift existing bytes forward
            let shift = NODE_SIZE + 8;
            let len = self.regcode.offset_from(before) as usize;
            if len > 0 {
                std::ptr::copy(before, before.add(shift), len);
            }
            self.regcode = self.regcode.add(shift);

            // Write new node at the insertion point
            *before = opcode as u8;
            *before.add(1) = 0; // next high byte
            *before.add(2) = 0; // next low byte

            // Write minval (4 bytes, big-endian)
            *before.add(3) = ((minval >> 24) & 0xff) as u8;
            *before.add(4) = ((minval >> 16) & 0xff) as u8;
            *before.add(5) = ((minval >> 8) & 0xff) as u8;
            *before.add(6) = (minval & 0xff) as u8;

            // Write maxval (4 bytes, big-endian)
            *before.add(7) = ((maxval >> 24) & 0xff) as u8;
            *before.add(8) = ((maxval >> 16) & 0xff) as u8;
            *before.add(9) = ((maxval >> 8) & 0xff) as u8;
            *before.add(10) = (maxval & 0xff) as u8;
        }
    }

    /// Get and increment the complex braces counter.
    pub fn next_complex_brace(&mut self) -> c_int {
        let n = self.num_complex_braces;
        self.num_complex_braces += 1;
        n
    }

    /// Get current complex braces count.
    pub fn complex_brace_count(&self) -> c_int {
        self.num_complex_braces
    }
}

impl Default for RegCompiler {
    fn default() -> Self {
        Self::new(ptr::null(), 0)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new regex compiler.
#[no_mangle]
pub extern "C" fn rs_bt_compiler_new(pattern: *const u8, re_flags: c_int) -> *mut RegCompiler {
    Box::into_raw(Box::new(RegCompiler::new(pattern, re_flags)))
}

/// Free a regex compiler.
///
/// # Safety
/// `compiler` must be a valid pointer from `rs_bt_compiler_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_compiler_free(compiler: *mut RegCompiler) {
    if !compiler.is_null() {
        drop(Box::from_raw(compiler));
    }
}

/// Start a compilation pass.
///
/// # Safety
/// `compiler` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_compiler_start(compiler: *mut RegCompiler, counting: c_int) {
    if !compiler.is_null() {
        (*compiler).start(counting != 0);
    }
}

/// Emit a single byte.
///
/// # Safety
/// `compiler` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_emit_byte(compiler: *mut RegCompiler, b: u8) {
    if !compiler.is_null() {
        (*compiler).emit_byte(b);
    }
}

/// Emit a bytecode node.
///
/// # Safety
/// `compiler` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_emit_node(compiler: *mut RegCompiler, opcode: c_int) -> *mut u8 {
    if compiler.is_null() {
        return ptr::null_mut();
    }
    (*compiler).emit_node(opcode)
}

/// Emit a bytecode node with a 4-byte argument.
///
/// # Safety
/// `compiler` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_emit_node_arg(
    compiler: *mut RegCompiler,
    opcode: c_int,
    arg: c_int,
) -> *mut u8 {
    if compiler.is_null() {
        return ptr::null_mut();
    }
    (*compiler).emit_node_with_arg(opcode, arg)
}

/// Insert a node before another node.
///
/// # Safety
/// `compiler` and `before` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_insert_node(
    compiler: *mut RegCompiler,
    opcode: c_int,
    before: *mut u8,
) {
    if !compiler.is_null() {
        (*compiler).insert_node(opcode, before);
    }
}

/// Set the next pointer of a node.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_next(compiler: *mut RegCompiler, from: *mut u8, to: *mut u8) {
    if !compiler.is_null() {
        (*compiler).set_next(from, to);
    }
}

/// Chain nodes together.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_chain(compiler: *mut RegCompiler, first: *mut u8, last: *mut u8) {
    if !compiler.is_null() {
        (*compiler).chain(first, last);
    }
}

/// Get the bytecode size.
///
/// # Safety
/// `compiler` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_compiler_size(compiler: *const RegCompiler) -> usize {
    if compiler.is_null() {
        0
    } else {
        (*compiler).size()
    }
}

/// Check if pattern is too long.
///
/// # Safety
/// `compiler` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_compiler_is_too_long(compiler: *const RegCompiler) -> c_int {
    if compiler.is_null() {
        0
    } else {
        c_int::from((*compiler).is_too_long())
    }
}

/// Get the opcode from a bytecode node.
///
/// # Safety
/// `p` must be null or point to valid bytecode.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_op(p: *const u8) -> c_int {
    op(p)
}

/// Get the next pointer from a bytecode node.
///
/// # Safety
/// `p` must be null or point to valid bytecode.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_next(p: *const u8) -> *const u8 {
    next(p)
}

/// Get the operand pointer from a bytecode node.
///
/// # Safety
/// `p` must be null or point to valid bytecode.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_operand(p: *const u8) -> *const u8 {
    operand(p)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bt_opcodes::{BRANCH, EXACTLY};

    #[test]
    fn test_compile_flags_bits() {
        let flags = CompileFlags {
            has_width: true,
            simple: true,
            sp_start: false,
            has_nl: true,
            has_lookbh: false,
        };

        let bits = flags.to_bits();
        assert_eq!(bits & HASWIDTH, HASWIDTH);
        assert_eq!(bits & SIMPLE, SIMPLE);
        assert_eq!(bits & SPSTART, 0);
        assert_eq!(bits & 0x8, 0x8); // HASNL
        assert_eq!(bits & 0x10, 0); // HASLOOKBH

        let recovered = CompileFlags::from_bits(bits);
        assert!(recovered.has_width);
        assert!(recovered.simple);
        assert!(!recovered.sp_start);
        assert!(recovered.has_nl);
        assert!(!recovered.has_lookbh);
    }

    #[test]
    fn test_flags_combine() {
        let mut flags1 = CompileFlags {
            has_width: true,
            simple: false,
            sp_start: false,
            has_nl: false,
            has_lookbh: false,
        };

        let flags2 = CompileFlags {
            has_width: false,
            simple: true,
            sp_start: false,
            has_nl: true,
            has_lookbh: false,
        };

        flags1.combine(&flags2);
        assert!(flags1.has_width);
        assert!(flags1.simple);
        assert!(!flags1.sp_start);
        assert!(flags1.has_nl);
        assert!(!flags1.has_lookbh);
    }

    #[test]
    fn test_constants() {
        assert_eq!(REGMAGIC, 0x1d);
        assert_eq!(NODE_SIZE, 3);
        assert_eq!(MAX_REGSIZE, 65534);
    }

    #[test]
    fn test_compiler_counting_mode() {
        let mut compiler = RegCompiler::new(ptr::null(), 0);
        compiler.start(true);
        assert!(compiler.is_counting());

        // Emit some nodes
        compiler.emit_node(BRANCH);
        compiler.emit_node(EXACTLY);
        compiler.emit_byte(b'a');
        compiler.emit_byte(0);
        compiler.emit_node(END);

        // Size should be calculated
        assert_eq!(compiler.size(), NODE_SIZE * 3 + 2);
    }

    #[test]
    fn test_compiler_emit_to_buffer() {
        let mut buffer = [0u8; 32];
        let mut compiler = RegCompiler::new(ptr::null(), 0);

        // Setup for emit mode
        compiler.regcode = buffer.as_mut_ptr();

        let node = compiler.emit_node(BRANCH);
        assert_eq!(node, buffer.as_mut_ptr());

        unsafe {
            assert_eq!(*buffer.as_ptr(), BRANCH as u8);
            assert_eq!(*buffer.as_ptr().add(1), 0); // next high
            assert_eq!(*buffer.as_ptr().add(2), 0); // next low
        }
    }

    #[test]
    fn test_op_null() {
        unsafe {
            assert_eq!(op(ptr::null()), END);
        }
    }

    #[test]
    fn test_next_null() {
        unsafe {
            assert!(next(ptr::null()).is_null());
        }
    }

    #[test]
    fn test_next_zero_offset() {
        let node = [BRANCH as u8, 0, 0];
        unsafe {
            assert!(next(node.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_operand() {
        let node = [EXACTLY as u8, 0, 0, b'a', b'b', b'c', 0];
        unsafe {
            let op_ptr = operand(node.as_ptr());
            assert_eq!(*op_ptr, b'a');
            assert_eq!(*op_ptr.add(1), b'b');
            assert_eq!(*op_ptr.add(2), b'c');
        }
    }

    #[test]
    fn test_paren_counting() {
        let mut compiler = RegCompiler::new(ptr::null(), 0);
        assert_eq!(compiler.paren_count(), 1); // Starts at 1

        let p1 = compiler.next_paren();
        assert_eq!(p1, 1);
        assert_eq!(compiler.paren_count(), 2);

        let p2 = compiler.next_paren();
        assert_eq!(p2, 2);
        assert_eq!(compiler.paren_count(), 3);
    }
}
