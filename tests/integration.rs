use addr::{dns, domain};

#[test]
fn addr_parsing() {
    rspec::run(&rspec::given("a domain", (), |ctx| {
        ctx.it("should allow non-fully qualified domain names", move |_| {
            assert!(domain::Name::parse("example.com").is_ok())
        });

        ctx.it("should allow fully qualified domain names", move |_| {
            assert!(domain::Name::parse("example.com.").is_ok())
        });

        ctx.it("should allow sub-domains", move |_| {
            assert!(domain::Name::parse("www.example.com.").is_ok())
        });

        ctx.it("should not allow more than 1 trailing dot", move |_| {
            assert!(domain::Name::parse("example.com..").is_err());
        });

        ctx.it("should allow single-label domains", move |_| {
            let domains = vec![
                // real TLDs
                "com",
                "saarland",
                "museum.",
                // non-existant TLDs
                "localhost",
                "madeup",
                "with-dot.",
            ];
            for domain in domains {
                assert!(domain::Name::parse(domain).is_ok());
            }
        });

        ctx.it(
            "should not have the same result with or without the trailing dot",
            move |_| {
                assert_ne!(
                    domain::Name::parse("example.com.").unwrap(),
                    domain::Name::parse("example.com").unwrap()
                );
            },
        );

        ctx.it("should not have empty labels", move |_| {
            assert!(domain::Name::parse("exa..mple.com").is_err());
        });

        ctx.it("should not contain spaces", move |_| {
            assert!(domain::Name::parse("exa mple.com").is_err());
        });

        ctx.it("should not start with a dash", move |_| {
            assert!(domain::Name::parse("-example.com").is_err());
        });

        ctx.it("should not end with a dash", move |_| {
            assert!(domain::Name::parse("example-.com").is_err());
        });

        ctx.it("should not contain /", move |_| {
            assert!(domain::Name::parse("exa/mple.com").is_err());
        });

        ctx.it("should not have a label > 63 characters", move |_| {
            let mut too_long_domain = String::from("a");
            for _ in 0..64 {
                too_long_domain.push_str("a");
            }
            too_long_domain.push_str(".com");
            assert!(domain::Name::parse(too_long_domain.as_str()).is_err());
        });

        ctx.it("should not be an IPv4 address", move |_| {
            assert!(domain::Name::parse("127.38.53.247").is_err());
        });

        ctx.it("should not be an IPv6 address", move |_| {
            assert!(domain::Name::parse("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123").is_err());
        });

        ctx.it(
            "should allow numbers only labels that are not the tld",
            move |_| {
                assert!(domain::Name::parse("127.com").is_ok());
            },
        );

        ctx.it("should not allow number only tlds", move |_| {
            assert!(domain::Name::parse("example.127").is_err());
        });

        ctx.it("should not have more than 127 labels", move |_| {
            let mut too_many_labels_domain = String::from("a");
            for _ in 0..126 {
                too_many_labels_domain.push_str(".a");
            }
            too_many_labels_domain.push_str(".com");
            assert!(domain::Name::parse(too_many_labels_domain.as_str()).is_err());
        });

        ctx.it("should not have more than 253 characters", move |_| {
            let mut too_many_chars_domain = String::from("aaaaa");
            for _ in 0..50 {
                too_many_chars_domain.push_str(".aaaaaa");
            }
            too_many_chars_domain.push_str(".com");
            assert!(domain::Name::parse(too_many_chars_domain.as_str()).is_err());
        });
    }));

    rspec::run(&rspec::given("a DNS name", (), |ctx| {
        ctx.it("should allow extended characters", move |_| {
            let names = vec![
                "example.com.",
                "_tcp.example.com.",
                "_telnet._tcp.example.com.",
                "*.example.com.",
                "!.example.com.",
            ];
            for name in names {
                assert!(dns::Name::parse(name).is_ok());
            }
        });

        ctx.it(
            "should allow extracting the correct root and suffix where possible",
            move |_| {
                let names = vec![
                    ("_tcp.example.com.", Some("example.com."), Some("com.")),
                    (
                        "_telnet._tcp.example.com.",
                        Some("example.com."),
                        Some("com."),
                    ),
                    ("example.com", Some("example.com"), Some("com")),
                ];
                for (input, root, suffix) in names {
                    let name = dns::Name::parse(input).unwrap();
                    assert_eq!(name.root(), root);
                    assert_eq!(name.suffix(), suffix);
                }
            },
        );

        ctx.it("should not require a valid root domain", move |_| {
            let names = vec!["_tcp.com.", "_telnet._tcp.com.", "*.com.", "ex!mple.com."];
            for name in names {
                assert!(dns::Name::parse(name).is_ok());
            }
        });

        ctx.it("should not allow more than 1 trailing dot", move |_| {
            assert!(dns::Name::parse("example.com..").is_err());
        });
    }));
}
