//! Errors returned by this library

error_chain! {
    errors {
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

        Uts46(t: ::idna::Errors) {
            description("UTS #46 processing failed")
            display("UTS #46 processing error: '{:?}'", t)
        }
    }
}
