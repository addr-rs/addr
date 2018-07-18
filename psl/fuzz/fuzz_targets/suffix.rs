#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate psl;

use std::str::from_utf8;

use psl::{Psl, List};

fuzz_target!(|data: &[u8]| {
    if let Ok(data) = from_utf8(data) {
        if let Some(suffix) = List.suffix(data) {
            suffix.to_str();
        }
    }
});
