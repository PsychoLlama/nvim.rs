//! Deciding what an `--listen`/`--server` address means.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::try_getdigits_at;
use std::ffi::{CStr, CString};

/// What an address to listen on names, parsed whole.
#[derive(Debug, PartialEq, Eq)]
pub enum Endpoint {
    /// A local socket (a Unix domain socket, a Windows named pipe) at this
    /// path.
    Pipe(CString),
    /// A TCP endpoint: the host to resolve, and the port (the service
    /// handed to the resolver), or `None` for whichever one the kernel has
    /// free (the address ended in a colon).
    Tcp {
        host: CString,
        port: Option<CString>,
    },
}

/// The text after a TCP endpoint's colon, which is not a port number.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPort(pub Vec<u8>);

impl Endpoint {
    /// Split `address` into what it names. Nothing is cut: an address too
    /// long for a socket path fails when it is bound, and a host too long to
    /// be one fails its lookup.
    pub fn parse(address: &CStr) -> Result<Self, InvalidPort> {
        let bytes = address.to_bytes();
        let Some(host_end) = tcp_host_end(bytes) else {
            return Ok(Endpoint::Pipe(address.to_owned()));
        };
        let (host, port) = (&bytes[..host_end], &bytes[host_end + 1..]);
        if !port_in_range(port) {
            return Err(InvalidPort(port.to_vec()));
        }
        let no_nul = "a C string's slice holds no NUL";
        let host = CString::new(host).expect(no_nul);
        let port = (!port.is_empty()).then(|| CString::new(port).expect(no_nul));
        Ok(Endpoint::Tcp { host, port })
    }
}

/// Upstream's check on a port: the number it starts with (read as
/// `strtoimax` reads one, so none at all reads as 0) is a port number.
///
/// Anything after that number is left to the resolver, which refuses it as a
/// service — `https://example.com` is host `https`, service
/// `//example.com`, and fails its lookup, which server_spec pins.
fn port_in_range(text: &[u8]) -> bool {
    try_getdigits_at(text, 0).is_some_and(|(port, _)| (0..=i64::from(u16::MAX)).contains(&port))
}

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

#[cfg(test)]
mod tests {
    use super::{Endpoint, InvalidPort, is_bare_server_name, tcp_host_end};
    use std::ffi::CString;

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

    fn parse(address: &str) -> Result<Endpoint, InvalidPort> {
        Endpoint::parse(&CString::new(address).unwrap())
    }

    fn tcp(host: &str, port: Option<&str>) -> Result<Endpoint, InvalidPort> {
        let host = CString::new(host).unwrap();
        let port = port.map(|port| CString::new(port).unwrap());
        Ok(Endpoint::Tcp { host, port })
    }

    #[test]
    fn an_endpoint_with_a_port_is_tcp() {
        assert_eq!(parse("127.0.0.1:6666"), tcp("127.0.0.1", Some("6666")));
        assert_eq!(parse("::1:0"), tcp("::1", Some("0")));
        assert_eq!(parse("localhost:65535"), tcp("localhost", Some("65535")));
    }

    #[test]
    fn a_trailing_colon_asks_for_any_free_port() {
        assert_eq!(parse("localhost:"), tcp("localhost", None));
    }

    #[test]
    fn a_port_out_of_range_is_refused() {
        let invalid = |port: &str| Err(InvalidPort(port.as_bytes().to_vec()));
        assert_eq!(parse("127.0.0.1:65536"), invalid("65536"));
        assert_eq!(parse("127.0.0.1:-1"), invalid("-1"));
        let huge = "99999999999999999999999";
        assert_eq!(parse(&format!("127.0.0.1:{huge}")), invalid(huge));
    }

    #[test]
    fn text_after_the_number_is_left_to_the_resolver() {
        // As upstream: only the leading number is checked, so a service the
        // resolver will refuse still parses.
        assert_eq!(
            parse("https://example.com"),
            tcp("https", Some("//example.com"))
        );
        assert_eq!(parse("localhost:12ab"), tcp("localhost", Some("12ab")));
        assert_eq!(parse("localhost:http"), tcp("localhost", Some("http")));
    }

    #[test]
    fn anything_else_is_a_local_socket_path() {
        let pipe = |path: &str| Ok(Endpoint::Pipe(CString::new(path).unwrap()));
        assert_eq!(parse("/tmp/nvim.sock"), pipe("/tmp/nvim.sock"));
        assert_eq!(parse(":6666"), pipe(":6666"));
        assert_eq!(
            parse(r"C:\Users\me\nvim.sock"),
            pipe(r"C:\Users\me\nvim.sock")
        );
    }

    #[test]
    fn a_long_address_is_kept_whole() {
        let host = "h".repeat(300);
        assert_eq!(parse(&format!("{host}:6666")), tcp(&host, Some("6666")));
        let path = format!("/tmp/{}", "x".repeat(300));
        assert_eq!(
            parse(&path),
            Ok(Endpoint::Pipe(CString::new(path.clone()).unwrap()))
        );
    }
}
