use std::env;
use std::path::{Path, PathBuf};

fn generate_rust_bindings<P: AsRef<Path>>(out_path: P) {
    let result_path = out_path.as_ref().join("descovery_bindings.rs");

    let includes = [
        "/usr/local/lib/gcc/arm-none-eabi/7.3.1/include",
        "C:\\tools\\GNU Tools Arm Embedded\\7 2018-q2-update\\lib\\gcc\\arm-none-eabi\\7.3.1\\include",
        "/usr/local/Cellar/arm-none-eabi-gcc/8-2018-q4-major/gcc/lib/gcc/arm-none-eabi/8.2.1/include"
    ];

    let builder = bindgen::Builder::default()
        .header("../Core/Inc/rust_header.h")
        .use_core()
        .ctypes_prefix("crate::compat::libc")
        .clang_args(includes.iter().map(|include| format!("-I{}", include)))
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
