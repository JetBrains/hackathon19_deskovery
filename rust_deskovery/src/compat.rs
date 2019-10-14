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

pub fn display_text_xy(x: u16, y: u16, s: &str, color: u16, bg_color: u16, size: u8) {
    unsafe {
        let bytes = s.as_bytes();
        ILI9341_Draw_Text_Len(bytes.as_ptr() as *const i8, s.len() as u8, x, y, color, size, bg_color);
    }
}

pub fn debug_print(s: &str) {
    unsafe {
        debug_output(s.as_ptr(), s.len() as libc::c_uint);
    }
}

pub fn idle() { unsafe { delegate_idle(); } }

pub fn draw_image(bytes: *const i8) {
    unsafe {
        ILI9341_Draw_Image(bytes, SCREEN_HORIZONTAL_2 as u8);
    }
}

pub fn led_control(q: bool)
{
    unsafe { delegate_led_control(q); }
}


pub fn delay_ms(ms: i32) { unsafe { delegate_delay_ms(ms); } }

pub fn display_bg_control(brighness: i32) { unsafe { delegate_display_bg_control(brighness); } }

pub fn left_ticks() -> i32 { unsafe { return delegate_left_ticks(); } }

pub fn radar_range() -> i32 { unsafe { return delegate_radar_range(); } }

pub fn right_ticks() -> i32 { unsafe { return delegate_right_ticks(); } }

pub fn fill_screen(color: u16) { unsafe { return ILI9341_Fill_Screen(color); } }

pub fn deskovery_motor(pwrLeft: i32, pwrRight: i32, recovery: bool) {
    unsafe { delegate_deskovery_motor(pwrLeft, pwrRight, recovery); }
}

pub fn system_ticks() -> u32 { unsafe { return delegate_system_ticks(); } }

pub fn draw_filled_circle(x: u16, y: u16, r: u16, color: u16) {
    unsafe { ILI9341_Draw_Filled_Circle(x, y, r, color); }
}

pub fn draw_hollow_circle(x: u16, y: u16, r: u16, color: u16) {
    unsafe { ILI9341_Draw_Hollow_Circle(x, y, r, color); }
}

pub fn draw_filled_rectangle_coord(x0: u16, y0: u16, x1: u16, y1: u16, color: u16) {
    unsafe { ILI9341_Draw_Filled_Rectangle_Coord(x0, y0, x1, y1, color); }
}

