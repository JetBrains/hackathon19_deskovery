#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;

#[allow(dead_code)]
use wifi::{Port, PortResult, Device};
use data::{DeskoveryData, ServerData};

use odometry::OdometryComputer;
use core::f64::consts::PI;
use compat::{display_text_xy, PRX_BR, PRX_BL, PRX_FR, PRX_FL, idle};
use compat::{
    delay_ms, display_bg_control, led_control, left_ticks, prxData, radar_range, right_ticks,
    fill_screen, draw_image, deskovery_motor,
    uart_output, uart_input, WHITE, debug_output, system_ticks, ferris, jb_logo, cl_logo,
    draw_filled_circle, draw_hollow_circle, draw_filled_rectangle_coord, DARKGREEN, DARKCYAN, LIGHTGREY, RED, NAVY, BLUE};

fn output_data_line<F>(x: u16, y: u16, label: &str, dataGetter: F, dataLen: usize, color: u16, bg_color: u16, size: u8)
    where
        F: FnOnce() -> i32,
{
    display_text_xy(x, y, label, color, bg_color, size);
    output_value(x + label.len() as u16 * 6 * size as u16 /*todo constant font width*/, y, dataGetter, dataLen, color, bg_color, size);
}

fn output_value<F>(x: u16, y: u16, dataGetter: F, dataLen: usize, color: u16, bg_color: u16, size: u8) where F: FnOnce() -> i32 {
    let mut buf: [u8; 20] = [0; 20];
    let strCenter = buf.len() / 2;
    let mut index = strCenter;
    let mut val = dataGetter();
    let sign = val < 0;
    if sign {
        val = -val;
    }
    loop {
        index -= 1;
        buf[index] = (val % 10 + 48) as u8;
        val = val / 10;
        if (val == 0) | (index == 0) {
            break;
        }
    }
    if sign & (index != 0) {
        index -= 1;
        buf[index] = '-' as u8;
    }
    for i in strCenter..(index + dataLen) {
        buf[i] = 32;
    }
    display_text_xy(x, y, core::str::from_utf8(&buf[index..(index + dataLen)]).unwrap(), color, bg_color, size);
}

fn alarm_color(alarm_idx: u32) -> u16 {
    unsafe {
        if prxData.alarms[alarm_idx as usize] {
            0xF800//RED
        } else {
            0xAFE5 //Green-Yellow
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
        old_screen_draw: RobotBrains::clion_screen_draw,
        screen_draw: RobotBrains::data_screen_draw,
    };
    let mut device = Device::new(port);
    display_bg_control(80);
    unsafe {
        draw_image(jb_logo.as_ptr());
    }
    let mut c: u8;
    delay_ms(1500);
    unsafe {
        let mut c: u8 = 0;
        delay_ms(1500);
        while uart_input(&mut c, 1) != 0 {
            debug_output(&c, 1);
        }
    }

    match device.connect_to_wifi_if_needed() {
        _ => {}
    };
    loop {
        let arr = device.brains.deskovery_data;
        led_control(true);

        let mut final_data = &arr[0..device.brains.data_q_len];
        let mut data_str_result = serde_json_core::ser::to_string::<[u8; 1500], _>(final_data);
        device.brains.data_q_len = 0;
        while data_str_result.is_err() {
            final_data = &final_data[1..final_data.len()];
            data_str_result = serde_json_core::ser::to_string::<[u8; 1500], _>(final_data);
        }

        let data_str = data_str_result.unwrap();

        device.brains.server_data = device.make_post_request(&data_str,
//                                                             "185.135.234.139", 8000).ok();
                                                             "192.168.0.101", 8000).ok();
        led_control(false);
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
    screen_draw: fn(&mut Self),
    old_screen_draw: fn(&mut Self),
}

impl RobotBrains {
    pub fn robot_loop(&mut self) {
        idle();
        match self.server_data {
            Some(data) => {
                //tractor control
                self.left_motor = adjust_motor(data.x);
                self.right_motor = adjust_motor(data.y);
                //auto control
//                self.left_motor = -data.y / 2 + data.x / 2;
//                self.right_motor = -data.y / 2 - data.x / 2;
                deskovery_motor(self.left_motor, self.right_motor, false);
                if data.b1 {
                    self.screen_draw = Self::data_screen_draw;
                } else if data.b2 {
                    self.screen_draw = Self::clion_screen_draw;
                } else if data.b3 {
                    self.screen_draw = Self::rust_screen_draw;
                } else if data.b4 {
                    self.screen_draw = Self::jb_screen_draw;
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
    pub fn rust_screen_draw(&mut self) {
        unsafe {
            draw_image(ferris.as_ptr());
        }
//            self.jb_screen_draw();
    }
    pub fn clion_screen_draw(&mut self) {
        unsafe {
            draw_image(cl_logo.as_ptr());
        }
//            self.jb_screen_draw();
    }
    pub fn jb_screen_draw(&mut self) {
        unsafe {
            draw_image(jb_logo.as_ptr());
        }
    }

    pub fn data_screen_draw(&mut self) {
        if self.old_screen_draw as *const u8 != self.screen_draw as *const u8 {
            self.old_screen_draw = self.screen_draw;
            fill_screen(WHITE  as u16);
            draw_filled_circle(160, 120, 60, 0xFFE0);
            draw_hollow_circle(160, 120, 60, 0);
        }
        output_value(0, 0, || self.left_motor, 4, DARKGREEN as u16, WHITE as u16, 4);
        output_value(256, 0, || self.right_motor, 4, DARKCYAN as u16, WHITE as u16, 4);
        let range = radar_range();
        let color = match range {
            -1 => LIGHTGREY,
            0..=300 => RED,
            301..=500 => NAVY,
            _ => BLUE
        } as u16;
        output_data_line(72, 208, "Radar: ", || range, 5, WHITE as u16, color, 3);
        draw_filled_rectangle_coord(120, 80, 140, 100, alarm_color(PRX_BR));
        draw_filled_rectangle_coord(180, 80, 200, 100, alarm_color(PRX_BL));
        draw_filled_rectangle_coord(120, 140, 140, 160, alarm_color(PRX_FR));
        draw_filled_rectangle_coord(180, 140, 200, 160, alarm_color(PRX_FL));
    }
}

impl Port for RobotBrains {
    fn write(&mut self, message: &[u8]) -> PortResult<()> {
        unsafe {
            uart_output(message.as_ptr(), message.len() as u32);
        }
        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> PortResult<usize> {
        let size = unsafe { uart_input(buf.as_mut_ptr(), buf.len() as u32) };
        if size == 0 {
            self.robot_loop();
        } else {
            unsafe { debug_output(buf.as_ptr(), size as u32); }
        }
        Ok(size as usize)
    }
}

