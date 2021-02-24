extern crate addr;
extern crate psl;
extern crate rspec;

use std::str::FromStr;

use addr::errors::ErrorKind;
use addr::{DnsName, DomainName, Email, Host};
use psl::{List, Psl};

#[test]
fn addr_parsing() {
    let list = List::new();

    rspec::run(&rspec::given("a domain", (), |ctx| {
        ctx.it("should allow non-fully qualified domain names", move |_| {
            assert!(DomainName::from_str("example.com").is_ok())
        });

        ctx.it("should allow fully qualified domain names", move |_| {
            assert!(DomainName::from_str("example.com.").is_ok())
        });

        ctx.it("should allow sub-domains", move |_| {
            assert!(DomainName::from_str("www.example.com.").is_ok())
        });

        ctx.it("should not allow more than 1 trailing dot", move |_| {
            assert!(DomainName::from_str("example.com..").is_err());
            match *DomainName::from_str("example.com..").unwrap_err().kind() {
                ErrorKind::InvalidDomain(ref domain) => assert_eq!(domain, "example.com.."),
                _ => assert!(false),
            }
        });

        ctx.it(
            "should allow a single label with a single trailing dot",
            move |_| {
                assert!(list.suffix("com.").is_some());
            },
        );

        ctx.it(
            "should always have a suffix for single-label domains",
            move |_| {
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
                    let suffix = list.suffix(domain).unwrap();
                    assert_eq!(suffix, domain);
                    assert!(list.domain(domain).is_none());
                }
            },
        );

        ctx.it(
            "should not have the same result with or without the trailing dot",
            move |_| {
                assert_ne!(
                    DomainName::from_str("example.com.").unwrap(),
                    DomainName::from_str("example.com").unwrap()
                );
            },
        );

        ctx.it("should not have empty labels", move |_| {
            assert!(DomainName::from_str("exa..mple.com").is_err());
        });

        ctx.it("should not contain spaces", move |_| {
            assert!(DomainName::from_str("exa mple.com").is_err());
        });

        ctx.it("should not start with a dash", move |_| {
            assert!(DomainName::from_str("-example.com").is_err());
        });

        ctx.it("should not end with a dash", move |_| {
            assert!(DomainName::from_str("example-.com").is_err());
        });

        ctx.it("should not contain /", move |_| {
            assert!(DomainName::from_str("exa/mple.com").is_err());
        });

        ctx.it("should not have a label > 63 characters", move |_| {
            let mut too_long_domain = String::from("a");
            for _ in 0..64 {
                too_long_domain.push_str("a");
            }
            too_long_domain.push_str(".com");
            assert!(DomainName::from_str(&too_long_domain).is_err());
        });

        ctx.it("should not be an IPv4 address", move |_| {
            assert!(DomainName::from_str("127.38.53.247").is_err());
        });

        ctx.it("should not be an IPv6 address", move |_| {
            assert!(DomainName::from_str("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123").is_err());
        });

        ctx.it(
            "should allow numbers only labels that are not the tld",
            move |_| {
                assert!(DomainName::from_str("127.com").is_ok());
            },
        );

        ctx.it("should not allow number only tlds", move |_| {
            assert!(DomainName::from_str("example.127").is_err());
        });

        ctx.it("should not have more than 127 labels", move |_| {
            let mut too_many_labels_domain = String::from("a");
            for _ in 0..126 {
                too_many_labels_domain.push_str(".a");
            }
            too_many_labels_domain.push_str(".com");
            assert!(DomainName::from_str(&too_many_labels_domain).is_err());
        });

        ctx.it("should not have more than 253 characters", move |_| {
            let mut too_many_chars_domain = String::from("aaaaa");
            for _ in 0..50 {
                too_many_chars_domain.push_str(".aaaaaa");
            }
            too_many_chars_domain.push_str(".com");
            assert!(DomainName::from_str(&too_many_chars_domain).is_err());
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
                assert!(DnsName::from_str(name).is_ok());
            }
        });

        ctx.it(
            "should allow extracting the correct domain name where possible",
            move |_| {
                let names = vec![
                    ("_tcp.example.com.", "example.com."),
                    ("_telnet._tcp.example.com.", "example.com."),
                    ("*.example.com.", "example.com."),
                ];
                for (name, domain) in names {
                    let name = DnsName::from_str(name).unwrap();
                    let root = name.root();
                    assert_eq!(root, domain);
                }
            },
        );

        ctx.it("should have a valid root domain", move |_| {
            let names = vec!["_tcp.com.", "_telnet._tcp.com.", "*.com.", "ex!mple.com."];
            for name in names {
                assert!(DnsName::from_str(name).is_err());
            }
        });

        ctx.it("should not allow more than 1 trailing dot", move |_| {
            assert!(DnsName::from_str("example.com..").is_err());
            match *DnsName::from_str("example.com..").unwrap_err().kind() {
                ErrorKind::InvalidDomain(ref domain) => assert_eq!(domain, "example.com.."),
                _ => assert!(false),
            }
        });
    }));

    rspec::run(&rspec::given("a host", (), |ctx| {
        ctx.it("can be an IPv4 address", move |_| {
            assert!(Host::from_str("127.38.53.247").is_ok());
        });

        ctx.it("can be an IPv6 address", move |_| {
            assert!(Host::from_str("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123").is_ok());
        });

        ctx.it("can be a domain name", move |_| {
            assert!(Host::from_str("example.com").is_ok());
        });

        ctx.it(
            "cannot be neither an IP address nor a domain name",
            move |_| {
                assert!(Host::from_str("23.56").is_err());
            },
        );

        ctx.it(
            "an IPv4 address should parse into an IP object",
            move |_| {
                assert!(Host::from_str("127.38.53.247").unwrap().is_ip());
            },
        );

        ctx.it(
            "an IPv6 address should parse into an IP object",
            move |_| {
                assert!(Host::from_str("fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123")
                    .unwrap()
                    .is_ip());
            },
        );

        ctx.it(
            "a domain name should parse into a domain object",
            move |_| {
                assert!(Host::from_str("example.com").unwrap().is_domain());
            },
        );
    }));

    rspec::run(&rspec::given("a parsed email", (), |ctx| {
        ctx.it("should allow valid email addresses", move |_| {
            let emails = vec![
                "prettyandsimple@example.com",
                "very.common@example.com",
                "disposable.style.email.with+symbol@example.com",
                "other.email-with-dash@example.com",
                "x@example.com",
                "example-indeed@strange-example.com",
                "#!$%&'*+-/=?^_`{}|~@example.org",
                "example@s.solutions",
                "user@[fd79:cdcb:38cc:9dd:f686:e06d:32f3:c123]",
                r#""Abc\@def"@example.com"#,
                r#""Fred Bloggs"@example.com"#,
                r#""Joe\\Blow"@example.com"#,
                r#""Abc@def"@example.com"#,
                r#"customer/department=shipping@example.com"#,
                "$A12345@example.com",
                "!def!xyz%abc@example.com",
                "_somename@example.com",
            ];
            for email in emails {
                assert!(Email::from_str(email).is_ok());
            }
        });

        ctx.it("should reject invalid email addresses", move |_| {
            let emails = vec![
                "Abc.example.com",
                "A@b@c@example.com",
                r#"a"b(c)d,e:f;g<h>i[j\k]l@example.com"#,
                r#""just"not"right@example.com"#,
                r#"this is"not\allowed@example.com"#,
                r#"this\ still\"not\\allowed@example.com"#,
                "1234567890123456789012345678901234567890123456789012345678901234+x@example.com",
                "john..doe@example.com",
                "john.doe@example..com",
                " prettyandsimple@example.com",
                "prettyandsimple@example.com ",
                "@example.com",
            ];
            for email in emails {
                assert!(Email::from_str(email).is_err());
            }
        });

        ctx.it("should allow parsing IDN email addresses", move |_| {
            let emails = vec![
                r#"Pelé@example.com"#,
                r#"δοκιμή@παράδειγμα.δοκιμή"#,
                r#"我買@屋企.香港"#,
                r#"甲斐@黒川.日本"#,
                r#"чебурашка@ящик-с-апельсинами.рф"#,
                r#"संपर्क@डाटामेल.भारत"#,
                r#"用户@例子.广告"#,
            ];
            for email in emails {
                assert!(Email::from_str(email).is_ok());
            }
        });
    }));
}
