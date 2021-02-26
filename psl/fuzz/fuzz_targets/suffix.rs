#![no_main]

use libfuzzer_sys::fuzz_target;
use psl::{Psl, List};
use std::str::from_utf8;

fuzz_target!(|data: &[u8]| {
    if let Ok(data) = from_utf8(data) {
        if let Some(suffix) = List.suffix(data) {
            suffix.to_str();
        }
    }
});
