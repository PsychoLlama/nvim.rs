//! Channel management for Neovim RPC
//!
//! This module provides type definitions for Neovim's channel infrastructure,
//! which handles communication between Neovim and external processes (plugins,
//! GUIs, remote clients).
//!
//! # Channel Types
//!
//! - `kChannelStreamProc` - Process (job) channel
//! - `kChannelStreamSocket` - TCP/pipe socket channel
//! - `kChannelStreamStdio` - Standard I/O channel
//! - `kChannelStreamStderr` - Standard error channel
//! - `kChannelStreamInternal` - Internal loopback channel

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Channel ID for stdio
pub const CHAN_STDIO: u64 = 1;

/// Channel ID for stderr
pub const CHAN_STDERR: u64 = 2;

// =============================================================================
// Enums
// =============================================================================

/// Channel stream type
///
/// Determines the underlying transport mechanism for a channel.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStreamType {
    /// Process (job) channel
    Proc = 0,
    /// TCP/pipe socket channel
    Socket = 1,
    /// Standard I/O channel
    Stdio = 2,
    /// Standard error channel
    Stderr = 3,
    /// Internal loopback channel
    Internal = 4,
}

impl ChannelStreamType {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Proc),
            1 => Some(Self::Socket),
            2 => Some(Self::Stdio),
            3 => Some(Self::Stderr),
            4 => Some(Self::Internal),
            _ => None,
        }
    }
}

/// Channel part for partial close operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPart {
    /// stdin of a process channel
    Stdin = 0,
    /// stdout of a process channel
    Stdout = 1,
    /// stderr of a process channel
    Stderr = 2,
    /// RPC layer
    Rpc = 3,
    /// All parts (full close)
    All = 4,
}

impl ChannelPart {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Stdin),
            1 => Some(Self::Stdout),
            2 => Some(Self::Stderr),
            3 => Some(Self::Rpc),
            4 => Some(Self::All),
            _ => None,
        }
    }
}

/// Channel stdin mode
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStdinMode {
    /// stdin is connected via pipe
    Pipe = 0,
    /// stdin is disconnected
    Null = 1,
}

/// Client type for RPC channels
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientType {
    /// Unknown client type
    Unknown = -1,
    /// Remote client
    Remote = 0,
    /// UI client
    Ui = 1,
    /// Embedder client
    Embedder = 2,
    /// Host client
    Host = 3,
    /// Plugin client
    Plugin = 4,
    /// Msgpack-rpc client
    MsgpackRpc = 5,
}

impl ClientType {
    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Remote,
            1 => Self::Ui,
            2 => Self::Embedder,
            3 => Self::Host,
            4 => Self::Plugin,
            5 => Self::MsgpackRpc,
            _ => Self::Unknown,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_stream_type_from_c_int() {
        assert_eq!(
            ChannelStreamType::from_c_int(0),
            Some(ChannelStreamType::Proc)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(1),
            Some(ChannelStreamType::Socket)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(2),
            Some(ChannelStreamType::Stdio)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(3),
            Some(ChannelStreamType::Stderr)
        );
        assert_eq!(
            ChannelStreamType::from_c_int(4),
            Some(ChannelStreamType::Internal)
        );
        assert_eq!(ChannelStreamType::from_c_int(99), None);
    }

    #[test]
    fn test_channel_part_from_c_int() {
        assert_eq!(ChannelPart::from_c_int(0), Some(ChannelPart::Stdin));
        assert_eq!(ChannelPart::from_c_int(1), Some(ChannelPart::Stdout));
        assert_eq!(ChannelPart::from_c_int(2), Some(ChannelPart::Stderr));
        assert_eq!(ChannelPart::from_c_int(3), Some(ChannelPart::Rpc));
        assert_eq!(ChannelPart::from_c_int(4), Some(ChannelPart::All));
        assert_eq!(ChannelPart::from_c_int(99), None);
    }

    #[test]
    fn test_client_type_from_c_int() {
        assert_eq!(ClientType::from_c_int(0), ClientType::Remote);
        assert_eq!(ClientType::from_c_int(1), ClientType::Ui);
        assert_eq!(ClientType::from_c_int(5), ClientType::MsgpackRpc);
        assert_eq!(ClientType::from_c_int(99), ClientType::Unknown);
        assert_eq!(ClientType::from_c_int(-1), ClientType::Unknown);
    }

    #[test]
    fn test_constants() {
        assert_eq!(CHAN_STDIO, 1);
        assert_eq!(CHAN_STDERR, 2);
    }
}
