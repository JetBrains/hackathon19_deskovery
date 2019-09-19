use image::GenericImageView;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn generate_rust_bindings(out_path: PathBuf) {
    let result_path = out_path.join("descovery_bindings.rs");

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

#[allow(dead_code)]
enum OutputMode {
    NetPBM,
    ScreenBytes,
}

const DISPLAY_WIDTH: usize = 84;
const DISPLAY_HEIGHT: usize = 48;
const DISPLAY_BYTES_COUNT: usize = 504;
const BITS_LEN: usize = 8;
const BITS: [u8; BITS_LEN] = [
    0b0000_0001,
    0b0000_0010,
    0b0000_0100,
    0b0000_1000,
    0b0001_0000,
    0b0010_0000,
    0b0100_0000,
    0b1000_0000,
];

fn get_image_screen_pixels<P: AsRef<Path>>(
    image_path: P,
    screen_width: usize,
    screen_height: usize,
    output_mode: OutputMode,
) -> Vec<u8> {
    let image = image::open(image_path).unwrap();
    let (image_width, image_height) = image.dimensions();
    let image_width = image_width as usize;
    let image_height = image_height as usize;
    assert!(image_width <= screen_width);
    assert!(image_height <= screen_height);

    let image_offset_x = (screen_width - image_width) / 2;
    let image_offset_y = (screen_height - image_height) / 2;

    let mut result = match output_mode {
        OutputMode::NetPBM => vec![0; screen_width * screen_height],
        OutputMode::ScreenBytes => vec![0; DISPLAY_BYTES_COUNT],
    };

    let middle_value = 255u8 / 2;
    let grayscale_bytes = image.to_luma().into_raw();

    for screen_row_index in 0..screen_height {
        for screen_column_index in 0..screen_width {
            if (screen_row_index >= image_offset_y
                && screen_row_index < image_offset_y + image_height)
                && (screen_column_index >= image_offset_x
                    && screen_column_index < image_offset_x + image_width)
            {
                let image_pixel = grayscale_bytes[(screen_row_index - image_offset_y)
                    * image_width
                    + (screen_column_index - image_offset_x)];
                if image_pixel >= middle_value {
                    match output_mode {
                        OutputMode::NetPBM => {
                            result[screen_column_index + screen_width * screen_row_index] = 1
                        }
                        OutputMode::ScreenBytes => {
                            result[screen_column_index
                                + screen_width * (screen_row_index / BITS_LEN)] |=
                                BITS[screen_row_index % BITS_LEN];
                        }
                    };
                }
            };
        }
    }

    result
}

fn generate_image_data(out_path: PathBuf) {
    let lines = ["images/clion_logo.png", "images/rust_logo.png"]
        .into_iter()
        .map(|image_path| {
            let image_bytes = get_image_screen_pixels(
                image_path,
                DISPLAY_WIDTH,
                DISPLAY_HEIGHT,
                OutputMode::ScreenBytes,
            );

            let separator_index = image_path.find('/').unwrap();
            let extension_index = image_path.rfind('.').unwrap();
            let variable_name = format!(
                "{}_BYTES",
                &image_path[separator_index + 1..extension_index]
            )
            .to_uppercase();
            format!(
                "pub const {}: [u8; {}] = {:?};",
                variable_name,
                image_bytes.len(),
                image_bytes
            )
        })
        .collect::<Vec<_>>();

    fs::write(out_path.join("generated_images.rs"), lines.join("\n")).unwrap()
}

fn main() {
    let out_path = PathBuf::from("src");

    if env::var("TARGET").unwrap() == "thumbv7em-none-eabihf" {
        generate_rust_bindings(out_path.clone());
        generate_image_data(out_path)
    }
}
