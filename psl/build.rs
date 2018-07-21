extern crate rustc_version;

use std::env;

use rustc_version::{version, Version};

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let string_match = "PSL_STRING_MATCH";
    let string_match_not_set = env::var(string_match).is_err();
    println!("cargo:rerun-if-env-changed={}", string_match);

    if version().unwrap() < Version::parse("1.27.0").unwrap() || (profile == "debug" && string_match_not_set) {
        println!("cargo:rustc-env={}=1", string_match);
    }
}
