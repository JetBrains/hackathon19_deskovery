#![no_std]

use core::panic::PanicInfo;

// TODO this is fugly
pub mod libc {
    pub type c_int = i32;
    pub type c_long = i64;
    pub type c_uchar = u8;
    pub type c_char = i8;
}

include!(concat!(env!("OUT_DIR"), "/descovery_bindings.rs"));

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //    let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //    writeln!(host_stderr, "{}", info).ok();
    loop {}
}
