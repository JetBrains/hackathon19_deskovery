#![allow(dead_code)]

use core::panic::PanicInfo;

pub mod libc {
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i32;
    pub type c_ulong = u32;
    pub type c_ushort = u16;
    pub type c_uchar = u8;
    pub type c_char = i8;
    //    pub type c_double = f64;
}

include!("descovery_bindings.rs");

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //    let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //    writeln!(host_stderr, "{}", info).ok();
    unsafe {
        loop {
            Error_Handler();
        }
    }
}

pub fn display_text_xy(x: u16, y: u16, s: &str, color: u16, bg_color: u16) {
    unsafe {
        let size = 4;
        let bytes = s.as_bytes();
        ILI9341_Draw_Text_Len(bytes.as_ptr() as *const i8, s.len() as u8 ,x as u8 * size, y as u8 * 8 * size, color, size as u16, bg_color);
    }
}

pub fn debug_print(s: &str) {
    unsafe {
        debug_output(s.as_ptr(), s.len() as libc::c_uint);
    }
}

pub fn robot_idle() { unsafe { idle(); } }

pub fn draw_image(bytes: *const i8) {
    unsafe {
        ILI9341_Draw_Image(bytes, SCREEN_HORIZONTAL_2 as u8);
    }
}
