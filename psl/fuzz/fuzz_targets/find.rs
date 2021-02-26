#![no_main]

use libfuzzer_sys::fuzz_target;
use psl::{Psl, List};

fuzz_target!(|data: &[u8]| {
    List.find(data);
});
