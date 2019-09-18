use std::env;
use std::path::PathBuf;

fn generate_rust_bindings(out_path: PathBuf) {
    let result_path = out_path.join("descovery_bindings.rs");

    // TODO uncomment?
    //    if result_path.is_file() {
    //        return;
    //    }

    let builder = bindgen::Builder::default()
        .header("../Core/Inc/rust_header.h")
        .use_core()
        // without this it generates `pub type __uint32_t = ::std::os::raw::c_uint;`
        .whitelist_recursively(false)
//        .whitelist_var("huart2")
//        .whitelist_var("HAL_LD2_GPIO_Port")
//        .whitelist_var("HAL_LD2_Pin")
        // todo this is a copy-paste from makefile
//        .clang_arg("-DUSE_HAL_DRIVER")
//        .clang_arg("-DSTM32F446xx")
//        .clang_arg("--target=thumbv7em-none-eabi")
//        .clang_arg("--verbose")
//        .clang_arg("-nostdinc")
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
