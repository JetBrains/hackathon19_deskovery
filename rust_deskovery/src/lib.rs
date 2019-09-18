#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
use core::panic::PanicInfo;

// TODO this is fugly
pub mod libc {
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i64;
    pub type c_ulong = u64;
    pub type c_uchar = u8;
    pub type c_char = i8;
}
include!("descovery_bindings.rs");

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //    let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //    writeln!(host_stderr, "{}", info).ok();
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_main() {
    //    let s = "Hello, Embedded World";

    unsafe {
        //        outputStr(s.as_ptr(), s.len());
        let mut brightness: i32 = 0;
        loop {
            led_control(true);
            delay_ms(300);
            led_control(false);
            brightness = (brightness + 10) % 102;
            display_bg_control(brightness);
            LCD5110_set_XY(0, 0);
            LCD5110_write_string(b"This is RUST!" as *const u8);
        }
    }
}
