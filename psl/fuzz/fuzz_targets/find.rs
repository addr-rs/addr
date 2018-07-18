#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate psl;

use psl::{Psl, List};

fuzz_target!(|data: &[u8]| {
    List.find(data);
});
