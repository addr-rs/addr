use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::{env, mem, str};

use psl::Type;
use psl_lexer::request;
use rspec::report::ExampleResult;

#[test]
fn list_behaviour() {
    rspec::run(&rspec::describe("the official test", (), |ctx| {
        let tests = "https://raw.githubusercontent.com/publicsuffix/list/master/tests/tests.txt";
        let body = request(tests).unwrap_or_else(|_| {
            let root = env::var("CARGO_MANIFEST_DIR").unwrap();
            let path = Path::new(&root).join("tests").join("tests.txt");
            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents
        });

        let mut parse = false;

        for (i, line) in body.lines().enumerate() {
            match line {
                line if line.trim().is_empty() => {
                    parse = true;
                    continue;
                }
                line if line.starts_with("//") => {
                    continue;
                }
                line => {
                    if !parse {
                        continue;
                    }
                    let mut test = line.split_whitespace().peekable();
                    if test.peek().is_none() {
                        continue;
                    }
                    let input = match test.next() {
                        Some("null") => "",
                        Some(res) => res,
                        None => {
                            panic!(format!(
                                "line {} of the test file doesn't seem to be valid",
                                i
                            ));
                        }
                    };
                    if !expected_tld(input) {
                        continue;
                    }
                    let (expected_root, expected_suffix) = match test.next() {
                        Some("null") => (None, None),
                        Some(root) => {
                            let suffix = {
                                let parts: Vec<&str> = root.split('.').rev().collect();
                                (&parts[..parts.len() - 1])
                                    .iter()
                                    .rev()
                                    .map(|part| *part)
                                    .collect::<Vec<_>>()
                                    .join(".")
                            };
                            (Some(root.to_string()), Some(suffix.to_string()))
                        }
                        None => {
                            panic!(format!(
                                "line {} of the test file doesn't seem to be valid",
                                i
                            ));
                        }
                    };
                    let (found_root, found_suffix) =
                        if input.starts_with(".") || input.contains("..") {
                            (None, None)
                        } else {
                            psl::domain(input.to_lowercase().as_bytes())
                                .map(|d| {
                                    let domain = str::from_utf8(d.as_bytes()).unwrap().to_string();
                                    let suffix =
                                        str::from_utf8(d.suffix().as_bytes()).unwrap().to_string();
                                    (Some(domain), Some(suffix))
                                })
                                .unwrap_or((None, None))
                        };
                    ctx.when(msg(format!("input is `{}`", input)), |ctx| {
                        let full_domain = expected_root.is_some();

                        ctx.it(msg(format!("means the root domain {}", val(&expected_root))), move |_| {
                            if expected_root == found_root {
                                ExampleResult::Success
                            } else {
                                let msg = format!("expected `{:?}` but found `{:?}` on line {} of `test_psl.txt`", expected_root, found_root, i+1);
                                ExampleResult::Failure(Some(msg))
                            }
                        });

                        if full_domain {
                            ctx.it(msg(format!("also means the suffix {}", val(&expected_suffix))), move |_| {
                                if expected_suffix == found_suffix {
                                    ExampleResult::Success
                                } else {
                                    let msg = format!("expected `{:?}` but found `{:?}` on line {} of `test_psl.txt`", expected_suffix, found_suffix, i+1);
                                    ExampleResult::Failure(Some(msg))
                                }
                            });
                        }
                    });
                }
            }
        }
    }));

    rspec::run(&rspec::describe("suffix tests", (), |ctx| {
        let extra = vec![
            (
                "gp-id-ter-acc-1.to.gp-kl-cas-11-ses001-ses-1.wdsl.5m.za",
                "za",
            ),
            ("yokohama.jp", "jp"),
            ("kobe.jp", "jp"),
            ("foo.bar.platformsh.site", "bar.platformsh.site"),
            ("bar.platformsh.site", "bar.platformsh.site"),
            ("platform.sh", "sh"),
            ("sh", "sh"),
            (".", "."),
            ("example.com.", "com."),
        ];

        for (input, expected) in extra {
            if !expected_tld(input) {
                continue;
            }
            ctx.when(msg(format!("input is `{}`", input)), |ctx| {
                let expected_suffix = Some(expected);
                ctx.it(
                    msg(format!(
                        "means the suffix {}",
                        val(&expected_suffix.map(ToString::to_string))
                    )),
                    move |_| {
                        let suffix = psl::suffix(input.as_bytes()).unwrap();
                        if suffix.as_bytes() == expected.as_bytes() {
                            ExampleResult::Success
                        } else {
                            let msg = format!(
                                "expected `{:?}` but found `{:?}`",
                                expected_suffix,
                                Some(str::from_utf8(suffix.as_bytes()).unwrap().to_string())
                            );
                            ExampleResult::Failure(Some(msg))
                        }
                    },
                );
            });
        }
    }));

    rspec::run(&rspec::describe("suffix type tests", (), |ctx| {
        let extra = vec![
            (
                "gp-id-ter-acc-1.to.gp-kl-cas-11-ses001-ses-1.wdsl.5m.za",
                false,
                None,
            ),
            ("yokohama.jp", true, Some(Type::Icann)),
            ("kobe.jp", true, Some(Type::Icann)),
            ("foo.bar.platformsh.site", true, Some(Type::Private)),
            ("bar.platformsh.site", true, Some(Type::Private)),
            ("platform.sh", true, Some(Type::Icann)),
            ("sh", true, Some(Type::Icann)),
            (".", false, None),
            ("example.gafregsrse", false, None),
        ];

        for (input, known_suffix, typ) in extra {
            if !expected_tld(input) {
                continue;
            }
            ctx.when(msg(format!("input is `{}`", input)), |ctx| {
                ctx.it(
                    msg(format!(
                        "means known suffix {}",
                        val(&Some(known_suffix.to_string()))
                    )),
                    move |_| {
                        let suffix = psl::suffix(input.as_bytes()).unwrap();
                        assert_eq!(suffix.typ(), typ);
                        if suffix.is_known() == known_suffix {
                            ExampleResult::Success
                        } else {
                            let msg = format!(
                                "expected `{:?}` but found `{:?}`",
                                known_suffix,
                                suffix.is_known()
                            );
                            ExampleResult::Failure(Some(msg))
                        }
                    },
                );
            });
        }
    }));
}

// Converts a String to &'static str
//
// This will leak memory but that's OK for our testing purposes
fn msg(s: String) -> &'static str {
    unsafe {
        let ret = mem::transmute(&s as &str);
        mem::forget(s);
        ret
    }
}

fn val(s: &Option<String>) -> String {
    match *s {
        Some(ref v) => format!("should be `{}`", v),
        None => format!("is invalid"),
    }
}

fn expected_tld(input: &str) -> bool {
    let var = if let Ok(var) = env::var("PSL_TLD") {
        var
    } else {
        String::new()
    };
    var.trim().is_empty() || input.trim().trim_end_matches('.').ends_with(&var)
}
