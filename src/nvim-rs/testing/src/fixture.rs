//! Test fixtures
//!
//! This module provides fixture management for Neovim tests,
//! including setup/teardown and test data helpers.

use std::ffi::{c_char, c_int};

// =============================================================================
// Fixture State
// =============================================================================

/// State of a fixture.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FixtureState {
    /// Fixture not initialized
    #[default]
    Uninitialized = 0,
    /// Fixture is being set up
    SettingUp = 1,
    /// Fixture is ready
    Ready = 2,
    /// Fixture is being torn down
    TearingDown = 3,
    /// Fixture has been cleaned up
    CleanedUp = 4,
    /// Fixture setup failed
    Failed = 5,
}

impl FixtureState {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::SettingUp,
            2 => Self::Ready,
            3 => Self::TearingDown,
            4 => Self::CleanedUp,
            5 => Self::Failed,
            _ => Self::Uninitialized,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if fixture is usable.
    #[must_use]
    pub const fn is_usable(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// FFI: Check if fixture is usable.
#[no_mangle]
pub extern "C" fn rs_fixture_state_is_usable(state: c_int) -> c_int {
    c_int::from(FixtureState::from_c_int(state).is_usable())
}

// =============================================================================
// Fixture Info
// =============================================================================

/// Information about a fixture.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FixtureInfo {
    /// Fixture ID
    pub id: c_int,
    /// Current state
    pub state: c_int,
    /// Reference count
    pub ref_count: c_int,
    /// Is shared between tests
    pub shared: bool,
    /// Requires cleanup
    pub needs_cleanup: bool,
    /// Setup time (ms)
    pub setup_time_ms: i64,
}

impl FixtureInfo {
    /// Create new fixture info.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            id,
            state: FixtureState::Uninitialized as c_int,
            ref_count: 0,
            shared: false,
            needs_cleanup: false,
            setup_time_ms: 0,
        }
    }

    /// Get current state.
    #[must_use]
    pub const fn get_state(&self) -> FixtureState {
        FixtureState::from_c_int(self.state)
    }

    /// Acquire a reference.
    pub fn acquire(&mut self) {
        self.ref_count += 1;
    }

    /// Release a reference.
    pub fn release(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }

    /// Check if fixture has references.
    #[must_use]
    pub const fn has_refs(&self) -> bool {
        self.ref_count > 0
    }
}

/// FFI: Create fixture info.
#[no_mangle]
pub extern "C" fn rs_fixture_info_new(id: c_int) -> FixtureInfo {
    FixtureInfo::new(id)
}

/// FFI: Acquire fixture reference.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_fixture_acquire(info: *mut FixtureInfo) {
    if !info.is_null() {
        (*info).acquire();
    }
}

/// FFI: Release fixture reference.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_fixture_release(info: *mut FixtureInfo) {
    if !info.is_null() {
        (*info).release();
    }
}

/// FFI: Check if fixture has references.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_fixture_has_refs(info: *const FixtureInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from((*info).has_refs())
}

// =============================================================================
// Buffer Fixture
// =============================================================================

/// Fixture for buffer testing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferFixture {
    /// Info
    pub info: FixtureInfo,
    /// Buffer ID
    pub buffer_id: c_int,
    /// Number of lines
    pub line_count: c_int,
    /// Has content
    pub has_content: bool,
    /// Content type
    pub content_type: c_int,
}

/// Buffer content types for fixtures.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BufferContentType {
    /// Empty buffer
    #[default]
    Empty = 0,
    /// Simple text lines
    Text = 1,
    /// Source code (syntax testing)
    Code = 2,
    /// Long lines
    LongLines = 3,
    /// Unicode content
    Unicode = 4,
    /// Binary-like content
    Binary = 5,
}

impl BufferContentType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Text,
            2 => Self::Code,
            3 => Self::LongLines,
            4 => Self::Unicode,
            5 => Self::Binary,
            _ => Self::Empty,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

impl BufferFixture {
    /// Create new buffer fixture.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            info: FixtureInfo::new(id),
            buffer_id: 0,
            line_count: 0,
            has_content: false,
            content_type: BufferContentType::Empty as c_int,
        }
    }

    /// Create buffer fixture with content.
    #[must_use]
    pub const fn with_content(
        id: c_int,
        line_count: c_int,
        content_type: BufferContentType,
    ) -> Self {
        Self {
            info: FixtureInfo::new(id),
            buffer_id: 0,
            line_count,
            has_content: line_count > 0,
            content_type: content_type as c_int,
        }
    }
}

/// FFI: Create buffer fixture.
#[no_mangle]
pub extern "C" fn rs_buffer_fixture_new(id: c_int) -> BufferFixture {
    BufferFixture::new(id)
}

/// FFI: Create buffer fixture with content.
#[no_mangle]
pub extern "C" fn rs_buffer_fixture_with_content(
    id: c_int,
    line_count: c_int,
    content_type: c_int,
) -> BufferFixture {
    BufferFixture::with_content(id, line_count, BufferContentType::from_c_int(content_type))
}

// =============================================================================
// Window Fixture
// =============================================================================

/// Fixture for window testing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowFixture {
    /// Info
    pub info: FixtureInfo,
    /// Window ID
    pub window_id: c_int,
    /// Width
    pub width: c_int,
    /// Height
    pub height: c_int,
    /// Associated buffer ID
    pub buffer_id: c_int,
}

impl WindowFixture {
    /// Create new window fixture.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            info: FixtureInfo::new(id),
            window_id: 0,
            width: 80,
            height: 24,
            buffer_id: 0,
        }
    }

    /// Create window fixture with size.
    #[must_use]
    pub const fn with_size(id: c_int, width: c_int, height: c_int) -> Self {
        Self {
            info: FixtureInfo::new(id),
            window_id: 0,
            width,
            height,
            buffer_id: 0,
        }
    }
}

/// FFI: Create window fixture.
#[no_mangle]
pub extern "C" fn rs_window_fixture_new(id: c_int) -> WindowFixture {
    WindowFixture::new(id)
}

/// FFI: Create window fixture with size.
#[no_mangle]
pub extern "C" fn rs_window_fixture_with_size(
    id: c_int,
    width: c_int,
    height: c_int,
) -> WindowFixture {
    WindowFixture::with_size(id, width, height)
}

// =============================================================================
// Test Data Generators
// =============================================================================

/// Generate test line content.
///
/// Returns number of characters written (excluding null terminator).
///
/// # Safety
/// `buf` must point to at least `buf_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_fixture_gen_line(
    buf: *mut c_char,
    buf_size: c_int,
    line_num: c_int,
) -> c_int {
    if buf.is_null() || buf_size <= 0 {
        return 0;
    }

    // Generate "Line N: content..."
    let prefix = b"Line ";
    let mut written = 0;

    // Write "Line "
    for &c in prefix {
        if written >= buf_size - 1 {
            break;
        }
        *buf.add(written as usize) = c as c_char;
        written += 1;
    }

    // Write line number
    let mut num = line_num;
    let mut digits = [0u8; 12];
    let digit_count = if num == 0 {
        digits[0] = b'0';
        1
    } else {
        let mut count = 0;
        while num > 0 {
            digits[count] = b'0' + (num % 10) as u8;
            count += 1;
            num /= 10;
        }
        count
    };

    // Write digits in reverse
    for i in (0..digit_count).rev() {
        if written >= buf_size - 1 {
            break;
        }
        *buf.add(written as usize) = digits[i] as c_char;
        written += 1;
    }

    // Write ": content"
    let suffix = b": test content\n";
    for &c in suffix {
        if written >= buf_size - 1 {
            break;
        }
        *buf.add(written as usize) = c as c_char;
        written += 1;
    }

    // Null terminate
    *buf.add(written as usize) = 0;

    written
}

/// Generate repeated character string.
///
/// # Safety
/// `buf` must point to at least `buf_size` bytes.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_fixture_gen_repeat(
    buf: *mut c_char,
    buf_size: c_int,
    ch: c_int,
    count: c_int,
) -> c_int {
    if buf.is_null() || buf_size <= 0 {
        return 0;
    }

    let actual_count = count.min(buf_size - 1);
    for i in 0..actual_count {
        *buf.add(i as usize) = ch as c_char;
    }
    *buf.add(actual_count as usize) = 0;

    actual_count
}

/// Generate numbered sequence.
///
/// # Safety
/// `buf` must point to at least `count` i64 values.
#[no_mangle]
pub unsafe extern "C" fn rs_fixture_gen_sequence(buf: *mut i64, count: c_int, start: i64) {
    if buf.is_null() || count <= 0 {
        return;
    }

    for i in 0..count {
        *buf.add(i as usize) = start + i64::from(i);
    }
}

// =============================================================================
// Temp Path Helpers
// =============================================================================

/// Generate a unique temp file path component.
///
/// Returns number of characters written.
///
/// # Safety
/// `buf` must point to at least `buf_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_fixture_temp_name(
    buf: *mut c_char,
    buf_size: c_int,
    id: c_int,
) -> c_int {
    if buf.is_null() || buf_size <= 0 {
        return 0;
    }

    let prefix = b"nvim_test_";
    let mut written = 0;

    for &c in prefix {
        if written >= buf_size - 1 {
            break;
        }
        *buf.add(written as usize) = c as c_char;
        written += 1;
    }

    // Write id as hex
    let hex = b"0123456789abcdef";
    let mut num = id as u32;
    let mut digits = [0u8; 8];
    let digit_count = if num == 0 {
        digits[0] = b'0';
        1
    } else {
        let mut count = 0;
        while num > 0 {
            digits[count] = hex[(num & 0xF) as usize];
            count += 1;
            num >>= 4;
        }
        count
    };

    for i in (0..digit_count).rev() {
        if written >= buf_size - 1 {
            break;
        }
        *buf.add(written as usize) = digits[i] as c_char;
        written += 1;
    }

    *buf.add(written as usize) = 0;
    written
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_state() {
        assert!(!FixtureState::Uninitialized.is_usable());
        assert!(FixtureState::Ready.is_usable());
        assert!(!FixtureState::Failed.is_usable());
    }

    #[test]
    fn test_fixture_info() {
        let mut info = FixtureInfo::new(1);
        assert!(!info.has_refs());

        info.acquire();
        assert!(info.has_refs());
        assert_eq!(info.ref_count, 1);

        info.release();
        assert!(!info.has_refs());
    }

    #[test]
    fn test_buffer_fixture() {
        let fixture = BufferFixture::new(1);
        assert!(!fixture.has_content);

        let fixture = BufferFixture::with_content(1, 100, BufferContentType::Code);
        assert!(fixture.has_content);
        assert_eq!(fixture.line_count, 100);
    }

    #[test]
    fn test_window_fixture() {
        let fixture = WindowFixture::new(1);
        assert_eq!(fixture.width, 80);
        assert_eq!(fixture.height, 24);

        let fixture = WindowFixture::with_size(1, 120, 40);
        assert_eq!(fixture.width, 120);
        assert_eq!(fixture.height, 40);
    }

    #[test]
    fn test_gen_line() {
        let mut buf = [0i8; 64];
        unsafe {
            let len = rs_fixture_gen_line(buf.as_mut_ptr(), 64, 42);
            assert!(len > 0);
        }
    }

    #[test]
    fn test_gen_repeat() {
        let mut buf = [0i8; 16];
        unsafe {
            let len = rs_fixture_gen_repeat(buf.as_mut_ptr(), 16, c_int::from(b'x'), 10);
            assert_eq!(len, 10);
            assert_eq!(buf[0], b'x' as i8);
            assert_eq!(buf[9], b'x' as i8);
            assert_eq!(buf[10], 0);
        }
    }

    #[test]
    fn test_gen_sequence() {
        let mut buf = [0i64; 5];
        unsafe {
            rs_fixture_gen_sequence(buf.as_mut_ptr(), 5, 10);
            assert_eq!(buf, [10, 11, 12, 13, 14]);
        }
    }

    #[test]
    fn test_temp_name() {
        let mut buf = [0i8; 32];
        unsafe {
            let len = rs_fixture_temp_name(buf.as_mut_ptr(), 32, 0x1234);
            assert!(len > 0);
        }
    }
}
