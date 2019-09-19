#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;
mod odometry;

use crate::odometry::{OdometryComputer, Position};
use compat::display_text;
use compat::{
    delay_ms, display_bg_control, idle, led_control, left_ticks, prxData, radar_range, right_ticks,
    LCD5110_clear, LCD5110_set_XY, LCD5110_write_bytes, LCD5110_write_char,
}; //todo make safe

fn output_data_line<F>(y: u8, label: &str, dataGetter: F)
where
    F: FnOnce() -> i32,
{
    display_text(0, y, label);
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
    unsafe {
        LCD5110_write_bytes(
            buf[index..].as_ptr() as *const u8,
            (buf.len() - index) as u32,
        );
    }
}

fn alarm_char(alarm_idx: usize) -> u8 {
    unsafe {
        if prxData.alarms[alarm_idx] {
            'A' as u8
        } else {
            '.' as u8
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_main() {
    //    let s = "Hello, Embedded World";

    unsafe {
        //        outputStr(s.as_ptr(), s.len());
        let mut brightness: i32 = 0;

        let mut position = Position {
            x: 0.0,
            y: 0.0,
            theta: 0.0,
        };
        let mut odo_computer = OdometryComputer {
            position: &mut position,
            old_left_ticks: 0,
            old_right_ticks: 0,
        };

        loop {
            idle();
            led_control(true);
            delay_ms(300);
            led_control(false);
            brightness = (brightness + 10) % 100;
            display_bg_control(brightness);
            LCD5110_clear();
            display_text(0, 0, "This is RUST!");

            output_data_line(1, "B: ", || brightness);
            output_data_line(2, "Rng: ", || radar_range());
            output_data_line(3, "Left : ", || left_ticks());
            output_data_line(4, "Right: ", || right_ticks());

            LCD5110_set_XY(3, 5);
            LCD5110_write_char(alarm_char(0));
            LCD5110_write_char(alarm_char(1));
            LCD5110_write_char(alarm_char(2));
            LCD5110_write_char(alarm_char(3));
            odo_computer.update(left_ticks(), right_ticks());

            // TODO: f64 printing
            // output_data_line(4, "x:     ", || position.x);
            // output_data_line(4, "y:     ", || position.y);

            /*
                        void debug_output(const unsigned char *p, unsigned int len); //todo implement
            */
        }
    }
}
