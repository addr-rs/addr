extern crate rustc_version;

use std::env::{self, VarError};

use rustc_version::{version, Version};

fn main() {
    let profile = env::var("PROFILE").unwrap();

    let string_match = "PSL_STRING_MATCH";

    if version().unwrap() < Version::parse("1.27.0").unwrap() || profile == "debug" {
        println!("cargo:rustc-env={}=1", string_match);
    }

    let not_set: Vec<_> = vec!["PSL_TLD", "PSL_TLDS", string_match]
        .into_iter()
        .map(|key| { println!("cargo:rerun-if-env-changed={}", key); key })
        .filter(|x| *x != string_match)
        .map(|x| env::var(x))
        .filter(|x| *x == Err(VarError::NotPresent))
        .collect();

    if not_set.len() == 2 {
        if profile == "debug" {
            println!("cargo:rustc-env=PSL_TLD=com");
        }
    }
}
