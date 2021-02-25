//! Errors returned by this library

#[cfg(feature = "remote_list")]
use std::net::TcpStream;

error_chain::error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Url(::url::ParseError);
        Tls(::native_tls::Error) #[cfg(feature = "remote_list")];
        Handshake(::native_tls::HandshakeError<TcpStream>) #[cfg(feature = "remote_list")];
    }

    errors {
        UnsupportedScheme { }

        InvalidList { }

        NoHost { }

        InvalidHost { }

        InvalidEmail { }

        InvalidRule(t: String) {
            description("invalid rule")
            display("invalid rule: '{}'", t)
        }

        InvalidDomain(t: String) {
            description("invalid domain")
            display("invalid domain: '{}'", t)
        }
    }
}
