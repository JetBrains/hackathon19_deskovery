#![allow(dead_code)]
use core::panic::PanicInfo;

pub mod libc {
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i32;
    pub type c_ulong = u32;
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
        loop{
            Error_Handler();
        }
    }
}

pub fn display_text(s: &str) {
    unsafe {
        let bytes = s.as_bytes();
        LCD5110_write_bytes(bytes.as_ptr() as *const u8, bytes.len() as u32);
    }
}
pub fn display_text_xy(x: u8, y: u8, s: &str) {
    unsafe {
        LCD5110_set_XY(x, y);
    }
    display_text(s);
}

pub fn debug_print(s: &str) {
    unsafe {
        debug_output(s.as_ptr(),s.len() as libc::c_uint);
    }
}
