use std::env;
use std::path::PathBuf;

fn generate_rust_bindings(out_path: PathBuf) {
    //let result_path = out_path.join("descovery_bindings.rs");
    let result_path = PathBuf::from("src").join("descovery_bindings.rs");

    let includes = ["/usr/local/lib/gcc/arm-none-eabi/7.3.1/include","C:\\tools\\GNU Tools Arm Embedded\\7 2018-q2-update\\lib\\gcc\\arm-none-eabi\\7.3.1\\include"];

    let builder = bindgen::Builder::default()
        .header("../Core/Inc/rust_header.h")
        .use_core()
        .ctypes_prefix("crate::libc")
        .clang_args(includes.iter().map(|include| format!("-I{}", include)))
        .clang_arg("--target=thumbv7em-none-eabi")
        .clang_arg("--verbose")
        .clang_arg("-nostdinc")
        ;

    let bindings = builder.generate().expect("Unable to generate bindings");

    bindings
        .write_to_file(result_path)
        .expect("Couldn't write bindings!");
}

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    if target == "thumbv7em-none-eabihf" {
        generate_rust_bindings(out_path);
    }
}
