use std::env;
use std::path::{Path, PathBuf};

fn generate_rust_bindings<P: AsRef<Path>>(out_path: P) {
    let result_path = out_path.as_ref().join("descovery_bindings.rs");

    let builder = bindgen::Builder::default()
        .header("../Core/Inc/rust_header.h")
        .use_core()
        .ctypes_prefix("crate::compat::libc")
        .clang_arg("--target=thumbv7em-none-eabi")
        .clang_arg("--verbose")
        .clang_arg("-nostdinc");

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(result_path)
        .expect("Couldn't write bindings!");
}

fn main() {
    let out_path = PathBuf::from("src/compat");
    if !out_path.exists() {
        std::fs::create_dir_all(&out_path).expect(&format!(
            "Failed to create an output directory: {:?}",
            out_path
        ));
    }

    if env::var("TARGET").unwrap() == "thumbv7em-none-eabihf" {
        generate_rust_bindings(&out_path);
    }
}
