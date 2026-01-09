//! Backup file management for Neovim.
//!
//! This module handles:
//! - Backup file naming and extension handling
//! - Backup strategy selection (copy vs rename)
//! - Patchmode (.orig) file handling
//! - Backup directory management

use std::ffi::c_int;
use std::path::{Path, PathBuf};

// =============================================================================
// Backup Strategy
// =============================================================================

/// Strategy for creating backup files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BackupStrategy {
    /// Copy the original file to the backup location
    #[default]
    Copy,
    /// Rename the original file to the backup name
    Rename,
}

/// Flags for 'backupcopy' option (bkc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BackupCopyFlags(u32);

impl BackupCopyFlags {
    /// Always make a copy
    pub const YES: u32 = 0x01;
    /// Automatically choose copy or rename
    pub const AUTO: u32 = 0x02;
    /// Break symbolic links when writing
    pub const BREAKSYMLINK: u32 = 0x04;
    /// Break hard links when writing
    pub const BREAKHARDLINK: u32 = 0x08;

    /// Create from raw flags.
    pub fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    /// Check if the YES flag is set.
    pub fn is_yes(&self) -> bool {
        self.0 & Self::YES != 0
    }

    /// Check if the AUTO flag is set.
    pub fn is_auto(&self) -> bool {
        self.0 & Self::AUTO != 0
    }

    /// Check if should break symlinks.
    pub fn should_break_symlink(&self) -> bool {
        self.0 & Self::BREAKSYMLINK != 0
    }

    /// Check if should break hardlinks.
    pub fn should_break_hardlink(&self) -> bool {
        self.0 & Self::BREAKHARDLINK != 0
    }

    /// Determine the initial backup strategy based on flags and context.
    pub fn initial_strategy(
        &self,
        is_append: bool,
        is_hardlink: bool,
        is_symlink: bool,
    ) -> BackupStrategy {
        if self.is_yes() || is_append {
            return BackupStrategy::Copy;
        }

        if self.is_auto() {
            // In auto mode, use copy for hardlinks and symlinks
            if is_hardlink || is_symlink {
                return BackupStrategy::Copy;
            }
        }

        BackupStrategy::Rename
    }
}

// =============================================================================
// Backup File Naming
// =============================================================================

/// Default backup extension.
pub const DEFAULT_BACKUP_EXT: &str = ".bak";

/// Generate a backup filename from the original filename.
///
/// # Arguments
/// * `original` - Original filename
/// * `extension` - Backup extension (e.g., ".bak")
/// * `prepend_dot` - Whether to prepend a dot to the backup name
///
/// # Returns
/// The backup filename
pub fn make_backup_name(original: &Path, extension: &str, prepend_dot: bool) -> PathBuf {
    let ext = if extension.is_empty() {
        DEFAULT_BACKUP_EXT
    } else {
        extension
    };

    let mut backup_name = original.to_path_buf();

    if let Some(file_name) = original.file_name() {
        let name_str = file_name.to_string_lossy();
        let new_name = if prepend_dot && !name_str.starts_with('.') {
            format!(".{}{}", name_str, ext)
        } else {
            format!("{}{}", name_str, ext)
        };
        backup_name.set_file_name(new_name);
    } else {
        // Handle edge case of no filename
        let current = backup_name.to_string_lossy().to_string();
        backup_name = PathBuf::from(format!("{}{}", current, ext));
    }

    backup_name
}

/// Generate a unique backup filename by appending a character.
///
/// If the backup file already exists, this tries 'a' through 'z' to
/// find a unique name.
///
/// # Arguments
/// * `base_name` - Base backup filename
/// * `check_exists` - Function to check if a path exists
///
/// # Returns
/// A unique backup filename, or None if all options are exhausted
pub fn make_unique_backup_name<F>(base_name: &Path, mut check_exists: F) -> Option<PathBuf>
where
    F: FnMut(&Path) -> bool,
{
    // First, try the base name
    if !check_exists(base_name) {
        return Some(base_name.to_path_buf());
    }

    // Try appending a character from 'z' down to 'a'
    // (matching C code which decrements from 'z')
    let base_str = base_name.to_string_lossy();

    for c in ('a'..='z').rev() {
        // Insert the character before the extension
        let candidate = if let Some(ext_pos) = base_str.rfind('.') {
            // Insert before the extension (but after the base name)
            // For "file.bak", we want "filez.bak"
            let (prefix, ext) = base_str.split_at(ext_pos);
            if let Some(last_dot) = prefix.rfind('.') {
                // Multiple extensions: insert before the last one
                let (p1, p2) = prefix.split_at(last_dot);
                format!("{}{}{}{}", p1, c, p2, ext)
            } else {
                format!("{}{}{}", prefix, c, ext)
            }
        } else {
            format!("{}{}", base_str, c)
        };

        let candidate_path = PathBuf::from(&candidate);
        if !check_exists(&candidate_path) {
            return Some(candidate_path);
        }
    }

    None
}

/// Generate a backup filename in a specific directory.
///
/// # Arguments
/// * `original` - Original filename
/// * `backup_dir` - Directory for backup files
/// * `extension` - Backup extension
///
/// # Returns
/// The backup filename in the specified directory
pub fn make_backup_in_dir(original: &Path, backup_dir: &Path, extension: &str) -> PathBuf {
    let ext = if extension.is_empty() {
        DEFAULT_BACKUP_EXT
    } else {
        extension
    };

    let file_name = original
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "backup".to_string());

    backup_dir.join(format!("{}{}", file_name, ext))
}

// =============================================================================
// Patchmode (.orig) Files
// =============================================================================

/// Default patchmode extension.
pub const DEFAULT_PATCHMODE_EXT: &str = ".orig";

/// Generate a patchmode filename.
///
/// # Arguments
/// * `original` - Original filename
/// * `extension` - Patchmode extension (e.g., ".orig")
///
/// # Returns
/// The patchmode filename
pub fn make_patchmode_name(original: &Path, extension: &str) -> PathBuf {
    let ext = if extension.is_empty() {
        DEFAULT_PATCHMODE_EXT
    } else {
        extension
    };

    let mut patchmode_name = original.to_path_buf();
    let current = patchmode_name.to_string_lossy().to_string();
    patchmode_name = PathBuf::from(format!("{}{}", current, ext));

    patchmode_name
}

// =============================================================================
// Backup Directory Parsing
// =============================================================================

/// Parse a backup directory specification.
///
/// Backup directory specs can be:
/// - "." - Same directory as the file
/// - "/path/to/dir" - Absolute path
/// - "~" - Home directory
/// - "./relative" - Relative path
///
/// # Arguments
/// * `spec` - Directory specification string
///
/// # Returns
/// Interpreted path
pub fn parse_backup_dir(spec: &str) -> Option<PathBuf> {
    if spec.is_empty() {
        return None;
    }

    if spec == "." {
        return Some(PathBuf::from("."));
    }

    if spec.starts_with('~') {
        // Home directory expansion would typically use an OS function
        // For now, just return as-is for the caller to expand
        return Some(PathBuf::from(spec));
    }

    Some(PathBuf::from(spec))
}

/// Iterator over backup directories from a comma-separated list.
pub struct BackupDirIterator<'a> {
    remaining: &'a str,
}

impl<'a> BackupDirIterator<'a> {
    /// Create a new iterator over backup directories.
    pub fn new(dirs: &'a str) -> Self {
        Self { remaining: dirs }
    }
}

impl<'a> Iterator for BackupDirIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        // Find the next comma
        let (current, rest) = match self.remaining.find(',') {
            Some(pos) => (&self.remaining[..pos], &self.remaining[pos + 1..]),
            None => (self.remaining, ""),
        };

        self.remaining = rest;

        // Skip empty entries
        let trimmed = current.trim();
        if trimmed.is_empty() {
            self.next()
        } else {
            Some(trimmed)
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI wrapper for make_backup_name.
///
/// # Safety
/// - `original` must be a valid null-terminated C string
/// - `extension` must be a valid null-terminated C string
/// - `output` must be a valid buffer of at least `output_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_make_backup_name(
    original: *const u8,
    original_len: usize,
    extension: *const u8,
    extension_len: usize,
    prepend_dot: c_int,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if original.is_null() || output.is_null() {
        return 0;
    }

    let original_slice = std::slice::from_raw_parts(original, original_len);
    let original_str = match std::str::from_utf8(original_slice) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let ext_str = if extension.is_null() || extension_len == 0 {
        DEFAULT_BACKUP_EXT
    } else {
        let ext_slice = std::slice::from_raw_parts(extension, extension_len);
        match std::str::from_utf8(ext_slice) {
            Ok(s) => s,
            Err(_) => DEFAULT_BACKUP_EXT,
        }
    };

    let backup_name = make_backup_name(Path::new(original_str), ext_str, prepend_dot != 0);
    let backup_bytes = backup_name.to_string_lossy();
    let backup_bytes = backup_bytes.as_bytes();

    if backup_bytes.len() >= output_len {
        return 0; // Buffer too small
    }

    let out_slice = std::slice::from_raw_parts_mut(output, output_len);
    out_slice[..backup_bytes.len()].copy_from_slice(backup_bytes);
    out_slice[backup_bytes.len()] = 0; // Null terminate

    backup_bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_copy_flags() {
        let flags = BackupCopyFlags::from_bits(BackupCopyFlags::YES);
        assert!(flags.is_yes());
        assert!(!flags.is_auto());

        let flags = BackupCopyFlags::from_bits(BackupCopyFlags::AUTO);
        assert!(!flags.is_yes());
        assert!(flags.is_auto());

        let flags = BackupCopyFlags::from_bits(
            BackupCopyFlags::BREAKSYMLINK | BackupCopyFlags::BREAKHARDLINK,
        );
        assert!(flags.should_break_symlink());
        assert!(flags.should_break_hardlink());
    }

    #[test]
    fn test_initial_strategy() {
        let flags_yes = BackupCopyFlags::from_bits(BackupCopyFlags::YES);
        assert_eq!(
            flags_yes.initial_strategy(false, false, false),
            BackupStrategy::Copy
        );

        let flags_auto = BackupCopyFlags::from_bits(BackupCopyFlags::AUTO);
        // Auto with hardlink should copy
        assert_eq!(
            flags_auto.initial_strategy(false, true, false),
            BackupStrategy::Copy
        );
        // Auto without links should rename
        assert_eq!(
            flags_auto.initial_strategy(false, false, false),
            BackupStrategy::Rename
        );

        // Append always copies
        let flags_none = BackupCopyFlags::from_bits(0);
        assert_eq!(
            flags_none.initial_strategy(true, false, false),
            BackupStrategy::Copy
        );
    }

    #[test]
    fn test_make_backup_name() {
        let original = Path::new("/path/to/file.txt");

        // Default extension
        let backup = make_backup_name(original, "", false);
        assert_eq!(backup, PathBuf::from("/path/to/file.txt.bak"));

        // Custom extension
        let backup = make_backup_name(original, ".backup", false);
        assert_eq!(backup, PathBuf::from("/path/to/file.txt.backup"));

        // Prepend dot
        let backup = make_backup_name(original, ".bak", true);
        assert_eq!(backup, PathBuf::from("/path/to/.file.txt.bak"));

        // File already starts with dot
        let original = Path::new("/path/to/.hidden");
        let backup = make_backup_name(original, ".bak", true);
        assert_eq!(backup, PathBuf::from("/path/to/.hidden.bak"));
    }

    #[test]
    fn test_make_unique_backup_name() {
        let base = Path::new("/path/to/file.txt.bak");

        // No existing files
        let result = make_unique_backup_name(base, |_| false);
        assert_eq!(result, Some(PathBuf::from("/path/to/file.txt.bak")));

        // Base exists, first alternative doesn't
        let mut calls = 0;
        let result = make_unique_backup_name(base, |p| {
            calls += 1;
            // Only the base exists
            p.to_string_lossy().ends_with(".bak") && !p.to_string_lossy().contains('z')
        });
        assert!(result.is_some());
        let result_str = result.unwrap().to_string_lossy().to_string();
        assert!(result_str.contains('z'));
    }

    #[test]
    fn test_make_backup_in_dir() {
        let original = Path::new("/path/to/file.txt");
        let backup_dir = Path::new("/backups");

        let backup = make_backup_in_dir(original, backup_dir, ".bak");
        assert_eq!(backup, PathBuf::from("/backups/file.txt.bak"));
    }

    #[test]
    fn test_make_patchmode_name() {
        let original = Path::new("/path/to/file.txt");

        let patchmode = make_patchmode_name(original, "");
        assert_eq!(patchmode, PathBuf::from("/path/to/file.txt.orig"));

        let patchmode = make_patchmode_name(original, ".original");
        assert_eq!(patchmode, PathBuf::from("/path/to/file.txt.original"));
    }

    #[test]
    fn test_parse_backup_dir() {
        assert_eq!(parse_backup_dir("."), Some(PathBuf::from(".")));
        assert_eq!(
            parse_backup_dir("/tmp/backups"),
            Some(PathBuf::from("/tmp/backups"))
        );
        assert_eq!(
            parse_backup_dir("~/backups"),
            Some(PathBuf::from("~/backups"))
        );
        assert_eq!(parse_backup_dir(""), None);
    }

    #[test]
    fn test_backup_dir_iterator() {
        let dirs = "/tmp,/var/backup,~/.backups";
        let collected: Vec<&str> = BackupDirIterator::new(dirs).collect();
        assert_eq!(collected, vec!["/tmp", "/var/backup", "~/.backups"]);

        // With empty entries
        let dirs = "/tmp,,/var/backup,";
        let collected: Vec<&str> = BackupDirIterator::new(dirs).collect();
        assert_eq!(collected, vec!["/tmp", "/var/backup"]);

        // Single entry
        let dirs = "/tmp";
        let collected: Vec<&str> = BackupDirIterator::new(dirs).collect();
        assert_eq!(collected, vec!["/tmp"]);

        // Empty string
        let dirs = "";
        let collected: Vec<&str> = BackupDirIterator::new(dirs).collect();
        assert!(collected.is_empty());
    }
}
