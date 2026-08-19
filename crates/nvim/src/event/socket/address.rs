//! Deciding what an `--listen`/`--server` address means, and rendering the
//! port back into it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

/// The size of a `SocketWatcher`'s address buffer.
pub const SOCKET_ADDR_LEN: usize = 256;

/// The offset of the colon that separates host from port, or `None` when
/// `address` names a local socket rather than a TCP endpoint.
///
/// The *last* colon wins, so a bracketless IPv6 literal is read as
/// `[::1]:port` would be — `::1:6666` is host `::1`, port `6666`.
pub fn tcp_host_end(address: &[u8]) -> Option<usize> {
    // A Windows drive-letter path ("X:\..." or "X:/...") is a local path.
    // Kept on every platform: the addresses come from the command line and
    // from `serverstart()`, so the answer must not depend on the host.
    if let [drive, b':', sep, ..] = address
        && drive.is_ascii_alphabetic()
        && (*sep == b'\\' || *sep == b'/')
    {
        return None;
    }
    // A leading colon is a socket path, not an empty host.
    address.iter().rposition(|&b| b == b':').filter(|&i| i > 0)
}

/// Whether `address` is a bare name rather than something to connect to.
///
/// `serverstart("foo")` and `--listen foo` mean "make me a socket called
/// foo" in the runtime directory. Anything carrying a separator — a colon, a
/// forward slash or a backslash — is taken as an address or a path, on every
/// platform, because it may have come from another machine's command line.
pub fn is_bare_server_name(address: &[u8]) -> bool {
    !address
        .iter()
        .any(|&byte| matches!(byte, b':' | b'/' | b'\\'))
}

/// `":<port>"`, NUL-terminated, for appending to a bound address.
///
/// The address a server reports through `v:servername` has to name the port
/// the kernel actually assigned, which is not known until the socket is bound.
pub fn port_suffix(port: u16) -> [u8; 8] {
    let mut digits = [0u8; 5];
    let mut count = 0;
    let mut rest = port;
    loop {
        digits[count] = b'0' + (rest % 10) as u8;
        count += 1;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    let mut out = [0u8; 8];
    out[0] = b':';
    for (i, digit) in digits[..count].iter().rev().enumerate() {
        out[1 + i] = *digit;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{is_bare_server_name, port_suffix, tcp_host_end};

    fn host_end(address: &str) -> Option<usize> {
        tcp_host_end(address.as_bytes())
    }

    #[test]
    fn a_host_and_port_splits_at_the_colon() {
        assert_eq!(host_end("127.0.0.1:6666"), Some(9));
        assert_eq!(host_end("localhost:0"), Some(9));
    }

    #[test]
    fn a_trailing_colon_is_a_host_with_no_port() {
        assert_eq!(host_end("localhost:"), Some(9));
    }

    #[test]
    fn a_path_without_a_colon_is_a_local_socket() {
        assert_eq!(host_end("/tmp/nvim.sock"), None);
        assert_eq!(host_end(""), None);
    }

    #[test]
    fn a_leading_colon_is_a_local_socket() {
        assert_eq!(host_end(":6666"), None);
    }

    #[test]
    fn a_windows_drive_letter_is_a_local_path() {
        assert_eq!(host_end("C:\\Users\\me\\nvim.sock"), None);
        assert_eq!(host_end("c:/users/me/nvim.sock"), None);
    }

    #[test]
    fn a_two_character_drive_prefix_is_still_an_endpoint() {
        // No separator after the colon, so this is host "C", port "6666".
        assert_eq!(host_end("C:6666"), Some(1));
    }

    #[test]
    fn the_last_colon_wins() {
        assert_eq!(host_end("::1:6666"), Some(3));
    }

    #[test]
    fn a_bare_name_has_no_separator_of_any_kind() {
        assert!(is_bare_server_name(b"nvim"));
        assert!(is_bare_server_name(b""));
        assert!(!is_bare_server_name(b"127.0.0.1:6666"));
        assert!(!is_bare_server_name(b"/tmp/nvim.sock"));
        assert!(is_bare_server_name(b"nvim.sock"));
        // A backslash counts on every platform, not just Windows.
        assert!(!is_bare_server_name(br"C:\pipe\nvim"));
    }

    #[test]
    fn a_port_renders_with_a_leading_colon_and_a_nul() {
        assert_eq!(&port_suffix(0)[..3], b":0\0");
        assert_eq!(&port_suffix(6666)[..6], b":6666\0");
        assert_eq!(&port_suffix(65535)[..7], b":65535\0");
    }
}
