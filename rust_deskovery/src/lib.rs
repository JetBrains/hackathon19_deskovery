#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;
#[allow(dead_code)]
mod generated_images;

use wifi::{Port, PortResult, Device};
use data::{DeskoveryData, ServerData};


use odometry::OdometryComputer;
use core::f64::consts::PI;
use compat::{display_text_xy, display_text, PRX_BR, PRX_BL, PRX_FR, PRX_FL, robot_idle};
use compat::{
    delay_ms, display_bg_control, led_control, left_ticks, prxData, radar_range, right_ticks,
    LCD5110_clear, LCD5110_set_XY, LCD5110_write_char, LCD5110_write_pict, deskovery_motor,
    uart_output, uart_input,
};
use crate::compat::{debug_output, system_ticks};
use crate::generated_images::{RUST_LOGO_BYTES, CLION_LOGO_NORM_BYTES, CLION_LOGO_BYTES}; //todo make safe

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
    let odo_computer = OdometryComputer::new();
    let port = RobotBrains {
        odo_computer,
        deskovery_data: [Default::default(); 10],
        data_q_len: 0,
        server_data: None,
        left_motor: 0,
        right_motor: 0,
        sample_timestamp: 0,
        screen_draw: RobotBrains::data_screen_draw,
    };
    let mut device = Device::new(port);
    unsafe {
        display_bg_control(80);
        LCD5110_write_pict(&RUST_LOGO_BYTES as *const _);
    }
    unsafe {
        let mut c: i8 = 0;
        delay_ms(1500);
        while uart_input(&mut c, 1) != 0 {
            debug_output(&(c as u8) as *const u8, 1);
        }
    }


    match device.connect_to_wifi_if_needed() {
        _ => {}
    };
    loop {
        let arr = device.brains.deskovery_data;
        unsafe { led_control(true); }
        device.brains.server_data = device.make_post_request(&arr[0..device.brains.data_q_len],
//                                                             "185.135.234.139", 8000).ok();
                                                             "104.236.228.23", 8000).ok();
        unsafe { led_control(false); }
        device.brains.data_q_len = 0;
        device.brains.robot_loop();
    }
}

fn adjust_motor(josticAxis: i32) -> i32 {
    let v = josticAxis.abs();
    if v < 50 { return 0; }
    return -josticAxis.signum() * (200 + (v - 50) * 800 / 1000);
}

pub struct RobotBrains {
    odo_computer: OdometryComputer,
    deskovery_data: [DeskoveryData; 10],
    data_q_len: usize,
    server_data: Option<ServerData>,
    left_motor: i32,
    right_motor: i32,
    sample_timestamp: u32,
    screen_draw: fn(&Self),
}

impl RobotBrains {
    pub fn robot_loop(&mut self) {
        robot_idle();
        match self.server_data {
            Some(data) => {
                //tractor control
                self.left_motor = adjust_motor(data.x);
                self.right_motor = adjust_motor(data.y);
                //auto control
//                self.left_motor = -data.y / 2 + data.x / 2;
//                self.right_motor = -data.y / 2 - data.x / 2;
                unsafe { deskovery_motor(self.left_motor, self.right_motor, false); }
                if data.b1 {
                    self.screen_draw = Self::data_screen_draw;
                } else if data.b2 {
                    self.screen_draw = Self::clion_screen_draw;
                } else if data.b3 {
                    self.screen_draw = Self::rust_screen_draw;
                } else if data.b4 {
                    self.screen_draw = Self::clion_neg_screen_draw;
                }
            }
            None => {}
        }
        (self.screen_draw)(self);
        unsafe {
            self.odo_computer.update(left_ticks(), right_ticks());
            if (system_ticks() - self.sample_timestamp) > 250 {
                if self.data_q_len < self.deskovery_data.len() {
                    self.deskovery_data[self.data_q_len] = DeskoveryData {
                        x: self.odo_computer.position().x as i32,
                        y: self.odo_computer.position().y as i32,
                        th: (self.odo_computer.position().theta * 180.0 / PI) as i32,
                        ps1: prxData.alarms[0],
                        ps2: prxData.alarms[1],
                        ps3: prxData.alarms[2],
                        ps4: prxData.alarms[3],
                        dto: radar_range(),
                    };
                    self.data_q_len += 1;
                }
                self.sample_timestamp = system_ticks();
            }
        }
    }
    pub fn rust_screen_draw(&self) {
        unsafe {
            LCD5110_write_pict(RUST_LOGO_BYTES.as_ptr());
        }
    }
    pub fn clion_screen_draw(&self) {
        unsafe {
            LCD5110_write_pict(CLION_LOGO_NORM_BYTES.as_ptr());
        }
    }
    pub fn clion_neg_screen_draw(&self) {
        unsafe {
            LCD5110_write_pict(CLION_LOGO_BYTES.as_ptr());
        }
    }
    pub fn data_screen_draw(&self) {
        unsafe {
            LCD5110_clear();
            output_data_line(0, 0, "LM: ", || self.left_motor);
            output_data_line(0, 1, "RM: ", || self.right_motor);
            LCD5110_set_XY(12, 0);
            LCD5110_write_char(alarm_char(PRX_BR));
            LCD5110_write_char(alarm_char(PRX_BL));
            LCD5110_set_XY(12, 1);
            LCD5110_write_char(alarm_char(PRX_FR));
            LCD5110_write_char(alarm_char(PRX_FL));
        }
    }
}

impl Port for RobotBrains {
    fn write(&mut self, message: &[u8]) -> PortResult<()> {
        unsafe {
            uart_output(message.as_ptr() as *const i8, message.len() as i32);
        }
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> PortResult<usize> {
        let size = unsafe { uart_input(buf.as_mut_ptr() as *mut i8, buf.len() as i32) };
        if size == 0 {
            self.robot_loop();
        } else {
            unsafe { debug_output(buf.as_ptr(), size as u32); }
        }
        Ok(size as usize)
    }
}

