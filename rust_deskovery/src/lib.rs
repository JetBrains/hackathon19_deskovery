#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;
#[allow(dead_code)]
mod generated_images;

use odometry::{OdometryComputer, Position};
use core::f64::consts::PI;
use compat::{display_text_xy, debug_print, display_text, PRX_BR, PRX_BL, PRX_FR, PRX_FL, robot_idle};
use compat::{
    delay_ms, display_bg_control, led_control, left_ticks, prxData, radar_range, right_ticks,
    LCD5110_clear, LCD5110_set_XY, LCD5110_write_char, LCD5110_write_pict, deskovery_motor,
};
use crate::compat::system_ticks; //todo make safe

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

enum DriveMode {
    Spiral,
    Leaves,
}

#[no_mangle]
pub extern "C" fn rust_main() {
    let mut brightness: i32 = 0;

    let mut odo_computer = OdometryComputer::new();
    let mut close_to_start_point = true;
    let mut left_power = 200;
    let mut right_power = 600;
    let mut current_iteration = 0;
    let mut required_iterations_before_change = 0;

    let mut start_x = 0;
    let mut start_y = 0;

    let drive_mode = DriveMode::Spiral;

    unsafe {
        delay_ms(1500);
    }
    loop {
        robot_idle();
        unsafe {
            /*
                        delay_ms(300);
                        brightness = (brightness + 10) % 100;
                        display_bg_control(brightness);
            */
            deskovery_motor(left_power, right_power, false);
            LCD5110_clear();

//            LCD5110_write_pict( &generated_images::RUST_LOGO_BYTES as *const u8);
//            delay_ms(1000);
//            LCD5110_write_pict( &generated_images::CLION_LOGO_NORM_BYTES as *const u8);
//            delay_ms(1000);

            output_data_line(0, 0, "Left: ", || left_power);
            output_data_line(0, 1, "Right: ", || right_power);
//            output_data_line(0, 0, "Dist: ", || radar_range());
//            output_data_line(0, 1, "L: ", || left_ticks());
//            output_data_line(0, 2, "R: ", || right_ticks());
//            LCD5110_set_XY(12, 0);
//            LCD5110_write_char(alarm_char(PRX_BR));
//            LCD5110_write_char(alarm_char(PRX_BL));
//            LCD5110_set_XY(12, 1);
//            LCD5110_write_char(alarm_char(PRX_FR));
//            LCD5110_write_char(alarm_char(PRX_FL));
            odo_computer.update(left_ticks(), right_ticks());

            match drive_mode {
                DriveMode::Spiral => {
                    if required_iterations_before_change == current_iteration {
                        left_power += 5;
                        required_iterations_before_change += 5;
                    } else {
                        current_iteration += 1;
                    }
                }
                DriveMode::Leaves => {
                    if ((odo_computer.position().x as i32 - start_x).pow(2) +
                        (odo_computer.position().y as i32 - start_y).pow(2)) < 900
                    {
                        if !close_to_start_point {
                            close_to_start_point = true;
                            start_x = odo_computer.position().x as i32;
                            start_y = odo_computer.position().y as i32;
                            left_power = -600;
                            right_power = -400;
                        }
                    } else {
                        close_to_start_point = false;
                    }
                }
            }
        }
        let position = odo_computer.position();
        output_data_line(0, 3, "X: ", || position.x as i32);
        output_data_line(0, 4, "Y: ", || position.y as i32);
        output_data_line(0, 5, "T: ", || (position.theta / PI * 180.0) as i32);

        debug_print("Hello, Deskovery\n\r");
    }
}
