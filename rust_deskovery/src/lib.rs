#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;
mod generated_images;

use odometry::{OdometryComputer, Position};
use core::f64::consts::PI;
use compat::{display_text_xy, debug_print, display_text, PRX_BR, PRX_BL, PRX_FR, PRX_FL, robot_idle};
use compat::{
    delay_ms, display_bg_control, led_control, left_ticks, prxData, radar_range, right_ticks,
    LCD5110_clear, LCD5110_set_XY, LCD5110_write_char, LCD5110_write_pict, deskovery_motor,
};
use crate::compat::{sensor_radar_range, set_radar_roi}; //todo make safe

pub struct Screen {
    screen: [u8; 504]
}

impl Screen {
    pub fn new() -> Self {
        Screen { screen: [0; 504] }
    }

    pub fn clear(&mut self) {
        self.screen = [0; 504];
    }

    pub fn pixel(&mut self, x: u32, y: u32) {
        if x >= 84 || y >= 48 { return; }
        let idx = x + 84 * (5 - (y / 8));
        self.screen[idx as usize] |= [0x80, 0x40, 0x20, 0x10, 0x8, 0x4, 0x2, 0x1][(y % 8) as usize];
    }
    pub fn draw(&self) {
        unsafe { LCD5110_write_pict(&self.screen as *const u8); }
    }
}


fn output_data_line<F>(x: u8, y: u8, label: &str, dataGetter: F)
    where
        F: FnOnce() -> i32,
{
    display_text_xy(x, y, label);
    let mut buf: [u8; 10] = [0; 10];
    let mut index = buf.len();
    let mut val = dataGetter();
    let sign = val < 0;
    if sign {
        val = -val;
    }
    loop {
        index -= 1;
        buf[index] = (val % 10 + 48) as u8;
        val = val / 10;
        if val == 0 {
            break;
        }
    }
    if sign {
        index -= 1;
        buf[index] = '-' as u8;
    }

    display_text(core::str::from_utf8(&buf[index..]).unwrap());
}

fn alarm_char(alarm_idx: u32) -> u8 {
    unsafe {
        if prxData.alarms[alarm_idx as usize] {
            'A' as u8
        } else {
            '.' as u8
        }
    }
}


#[no_mangle]
pub extern "C" fn rust_main() {
    let mut screen = Screen::new();
    unsafe {
        display_bg_control(99);
//        deskovery_motor(200, 300, false);
    }
    loop {
        screen.clear();
        for i in (0..15) {
            set_radar_roi(i, 0, i, 15);
            robot_idle();
            let mut y = sensor_radar_range();
            if y > 0 {
                y /= 15;
                for j in (0..4) {
                    screen.pixel(i * 5 + 2 + j, y as u32);
                }
            }
            screen.draw();
        }
    }
}
