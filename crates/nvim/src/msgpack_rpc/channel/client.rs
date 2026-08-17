#![forbid(unsafe_code)]

//! What kind of peer is on the other end of an RPC channel.
//!
//! A client announces itself with `nvim_set_client_info`; the `type` key of
//! that call is the only thing this decides on.

use core::ffi::CStr;

use crate::types::ClientType;

/// The `ClientType` values, nested so they stay out of the flat namespace the
/// unit-test header generator collects top-level constants into.
mod kind {
    use super::ClientType;

    pub const UNKNOWN: ClientType = -1;
    pub const REMOTE: ClientType = 0;
    pub const UI: ClientType = 1;
    pub const EMBEDDER: ClientType = 2;
    pub const HOST: ClientType = 3;
    pub const PLUGIN: ClientType = 4;
    pub const MSGPACK_RPC: ClientType = 5;
}

/// Classifies a peer from the `type` it declared.
///
/// A peer that declared nothing is treated as `remote` — the same as one that
/// said so explicitly — because that is the conservative reading: responses
/// are then only matched against the call at the top of the stack.
///
/// A `type` this editor does not recognise is *not* treated as remote. It
/// reaches `channel_info()` as `client_type` and is otherwise inert.
pub fn classify_client(declared: Option<&CStr>) -> ClientType {
    match declared.map(CStr::to_bytes) {
        None | Some(b"remote") => kind::REMOTE,
        Some(b"msgpack-rpc") => kind::MSGPACK_RPC,
        Some(b"ui") => kind::UI,
        Some(b"embedder") => kind::EMBEDDER,
        Some(b"host") => kind::HOST,
        Some(b"plugin") => kind::PLUGIN,
        Some(_) => kind::UNKNOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_type_reads_as_remote() {
        assert_eq!(classify_client(None), kind::REMOTE);
        assert_eq!(classify_client(Some(c"remote")), kind::REMOTE);
    }

    #[test]
    fn every_declared_type_has_a_value() {
        assert_eq!(classify_client(Some(c"msgpack-rpc")), kind::MSGPACK_RPC);
        assert_eq!(classify_client(Some(c"ui")), kind::UI);
        assert_eq!(classify_client(Some(c"embedder")), kind::EMBEDDER);
        assert_eq!(classify_client(Some(c"host")), kind::HOST);
        assert_eq!(classify_client(Some(c"plugin")), kind::PLUGIN);
    }

    #[test]
    fn an_unrecognised_type_is_not_remote() {
        assert_eq!(classify_client(Some(c"")), kind::UNKNOWN);
        assert_eq!(classify_client(Some(c"Remote")), kind::UNKNOWN);
        assert_eq!(classify_client(Some(c"msgpack_rpc")), kind::UNKNOWN);
    }
}
