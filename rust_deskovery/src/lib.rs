#![cfg_attr(not(test), no_std)]
#![feature(lang_items, core_intrinsics)]

mod compat;
mod data;
mod odometry;
mod wifi;

use data::{DeskoveryData, ServerData};
use wifi::{Device, Port, PortResult};

use compat::*;
use core::f64::consts::PI;
use odometry::OdometryComputer;

fn output_data_line<F>(
    x: u16,
    y: u16,
    label: &str,
    data_getter: F,
    data_len: usize,
    color: u16,
    bg_color: u16,
    size: u8,
) where
    F: FnOnce() -> i32,
{
    display_text_xy(x, y, label, color, bg_color, size);
    output_value(
        x + label.len() as u16 * 6 * size as u16, /*todo constant font width*/
        y,
        data_getter,
        data_len,
        color,
        bg_color,
        size,
    );
}

fn output_value<F>(
    x: u16,
    y: u16,
    data_getter: F,
    data_len: usize,
    color: u16,
    bg_color: u16,
    size: u8,
) where
    F: FnOnce() -> i32,
{
    let mut buf: [u8; 20] = [0; 20];
    let str_center = buf.len() / 2;
    let mut index = str_center;
    let mut val = data_getter();
    let sign = val < 0;
    if sign {
        val = -val;
    }
    loop {
        index -= 1;
        buf[index] = (val % 10 + 48) as u8;
        val /= 10;
        if (val == 0) | (index == 0) {
            break;
        }
    }
    if sign & (index != 0) {
        index -= 1;
        buf[index] = b'-';
    }
    for i in str_center..(index + data_len) {
        buf[i] = 32;
    }
    display_text_xy(
        x,
        y,
        core::str::from_utf8(&buf[index..(index + data_len)]).unwrap(),
        color,
        bg_color,
        size,
    );
}

fn alarm_color(alarm_idx: u32) -> u16 {
    unsafe {
        if prxData.alarms[alarm_idx as usize] {
            0xF800 //RED
        } else {
            0xAFE5 //Green-Yellow
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_main() {
    let odo_computer = OdometryComputer::default();
    let port = RobotBrains {
        odo_computer,
        deskovery_data: [Default::default(); 10],
        data_q_len: 0,
        server_data: None,
        left_motor: 0,
        right_motor: 0,
        sample_timestamp: 0,
    };
    let mut device = Device::new(port);
    display_bg_control(80);
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
    unsafe {
        draw_image(*screen_back);
    }
    loop {
        let arr = device.brains.deskovery_data;
        led_control(true);

        let mut final_data = &arr[0..device.brains.data_q_len];
        let mut data_str_result = serde_json_core::ser::to_string::<[u8; 1500], _>(final_data);
        while data_str_result.is_err() {
            final_data = &final_data[1..final_data.len()];
            data_str_result = serde_json_core::ser::to_string::<[u8; 1500], _>(final_data);
        }
        device.brains.data_q_len = 0;

        let data_str = data_str_result.unwrap();

        device.brains.server_data = device
            .make_post_request(&data_str, "192.168.0.101", 8000)
            .ok();
        led_control(false);
        device.brains.robot_loop();
    }
}

fn adjust_motor(controller_axis: i32) -> i32 {
    let v = controller_axis.abs();
    if v < 50 {
        0
    } else {
        -controller_axis.signum() * (200 + (v - 50) * 500 / 1000)
    }
}

pub struct RobotBrains {
    odo_computer: OdometryComputer,
    deskovery_data: [DeskoveryData; 10],
    data_q_len: usize,
    server_data: Option<ServerData>,
    left_motor: i32,
    right_motor: i32,
    sample_timestamp: u32,
}

impl RobotBrains {
    pub fn robot_loop(&mut self) {
        idle();
        if let Some(data) = self.server_data {
            //tractor control
            self.left_motor = adjust_motor(data.x);
            self.right_motor = adjust_motor(data.y);
            //auto control
            //                self.left_motor = -data.y / 2 + data.x / 2;
            //                self.right_motor = -data.y / 2 - data.x / 2;
            deskovery_motor(self.left_motor, self.right_motor, false);
        }
        self.screen_draw();
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
    pub fn jb_screen_draw(&mut self) {
        unsafe {
            draw_image(jb_logo.as_ptr());
        }
    }

    pub fn screen_draw(&mut self) {
        output_data_line(
            120,
            155,
            "L: ",
            || self.left_motor,
            4,
            RED as u16,
            WHITE as u16,
            2,
        );
        output_data_line(
            220,
            155,
            "R: ",
            || self.right_motor,
            4,
            DARKGREEN as u16,
            WHITE as u16,
            2,
        );
        let range = radar_range();
        let color = match range {
            -1 => LIGHTGREY,
            0..=300 => RED,
            301..=500 => BLUE,
            _ => NAVY,
        } as u16;
        output_data_line(122, 212, "Radar: ", || range, 5, WHITE as u16, color, 3);
        let position = self.odo_computer.position();

        output_data_line(
            122,
            175,
            "P: ",
            || position.x as i32,
            6,
            BLACK as u16,
            WHITE as u16,
            2,
        );
        output_data_line(
            230,
            175,
            ", ",
            || position.y as i32,
            6,
            BLACK as u16,
            WHITE as u16,
            2,
        );
        output_data_line(
            122,
            191,
            "     B: ",
            || (position.theta * 180.0 / PI) as i32,
            4,
            BLACK as u16,
            WHITE as u16,
            2,
        );

        draw_filled_rectangle_coord(35, 165, 45, 175, alarm_color(PRX_BR));
        draw_filled_rectangle_coord(85, 165, 95, 175, alarm_color(PRX_BL));
        draw_filled_rectangle_coord(35, 210, 45, 220, alarm_color(PRX_FR));
        draw_filled_rectangle_coord(85, 210, 95, 220, alarm_color(PRX_FL));
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
            unsafe {
                debug_output(buf.as_ptr(), size as u32);
            }
        }
        Ok(size as usize)
    }
}
