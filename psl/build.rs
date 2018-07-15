use std::env::{self, VarError};

fn main() {
    let not_set: Vec<_> = vec!["PSL_TLD", "PSL_TLDS"]
        .into_iter()
        .map(|key| {
            println!("cargo:rerun-if-env-changed={}", key);
            env::var(key)
        })
        .filter(|x| *x == Err(VarError::NotPresent))
        .collect();

    if not_set.len() == 2 {
        if let Ok(profile) = env::var("PROFILE") {
            if profile == "debug" {
                println!("cargo:rustc-env=PSL_TLD=com");
            }
        }
    }
}
