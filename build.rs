fn main() {
    if cfg!(addr_docs_rs) {
        println!("cargo:rustc-env=PSL_TLDS=com,中国,cn,рф");
        return;
    }
}
