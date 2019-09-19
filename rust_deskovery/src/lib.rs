#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;
mod generated_images;
mod odometry;

use crate::odometry::{OdometryComputer, Position};
use compat::{debug_print, display_text, display_text_xy, PRX_BL, PRX_BR, PRX_FL, PRX_FR};
use compat::{
    delay_ms, display_bg_control, idle, led_control, left_ticks, prxData, radar_range, right_ticks,
    LCD5110_clear, LCD5110_set_XY, LCD5110_write_char, LCD5110_write_pict, /*deskovery_motor*/
}; //todo make safe

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
    //    let s = "Hello, Embedded World";

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
        unsafe {
            idle();
            delay_ms(300);
            brightness = (brightness + 10) % 100;
            display_bg_control(brightness);
            LCD5110_clear();

            output_data_line(0, 0, "Dist: ", || radar_range());
            output_data_line(0, 1, "L: ", || left_ticks());
            output_data_line(0, 2, "R: ", || right_ticks());

            LCD5110_write_pict(&generated_images::RUST_LOGO_BYTES as *const u8);
            //            LCD5110_clear();
            //            display_text(0, 0, "This is RUST!");
            //
            //            output_data_line(1, "B: ", || brightness);
            //            output_data_line(2, "Rng: ", || radar_range());
            //            output_data_line(3, "Left : ", || left_ticks());
            //            output_data_line(4, "Right: ", || right_ticks());
            //
            //            LCD5110_set_XY(12, 0);
            //            LCD5110_write_char(alarm_char(PRX_BR));
            //            LCD5110_write_char(alarm_char(PRX_BL));
            LCD5110_set_XY(12, 1);
            //            LCD5110_write_char(alarm_char(PRX_FR));
            //            LCD5110_write_char(alarm_char(PRX_FL));
            //            odo_computer.update(left_ticks(), right_ticks());

            //            deskovery_motor(400, 400, false);
            //todo test odometry
            debug_print("Hello, Deskovery\n\r");

            // TODO: f64 printing
            // output_data_line(4, "x:     ", || position.x);
            // output_data_line(4, "y:     ", || position.y);

            /*
                        void debug_output(const unsigned char *p, unsigned int len); //todo implement
            */
        }
    }
}
