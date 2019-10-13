#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(core_intrinsics)]

mod compat;
#[allow(dead_code)]
use wifi::{Port, PortResult, Device};
use data::{DeskoveryData, ServerData};


use odometry::OdometryComputer;
use core::f64::consts::PI;
use compat::{display_text_xy, PRX_BR, PRX_BL, PRX_FR, PRX_FL, robot_idle};
use compat::{
    delay_ms, display_bg_control, led_control, left_ticks, prxData, radar_range, right_ticks,
    ILI9341_Fill_Screen, ILI9341_Draw_Char, ILI9341_Draw_Image, deskovery_motor,
    uart_output, uart_input, WHITE, BLACK, debug_output, system_ticks, SCREEN_HORIZONTAL_2,
    ferris, jb_logo, cl_logo};
use crate::compat::{ILI9341_Draw_Filled_Circle, ILI9341_Draw_Hollow_Circle, ILI9341_Draw_Filled_Rectangle_Coord};  //todo make safe

fn output_data_line<F>(x: u16, y: u16, label: &str, dataGetter: F)
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

    display_text_xy((label.len() * 5) as u16, y as u16, core::str::from_utf8(&buf[index..]).unwrap());
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

fn alarm_char(alarm_idx: u32) -> i8 {
    unsafe {
        if prxData.alarms[alarm_idx as usize] {
            'A' as i8
        } else {
            '.' as i8
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
        screen_changed: true
    };
    let mut device = Device::new(port);
    unsafe {
        display_bg_control(80);
        ILI9341_Draw_Image(jb_logo.as_ptr(), SCREEN_HORIZONTAL_2 as u8);
//        ILI9341_Draw_Image(cl_logo.as_ptr(),SCREEN_HORIZONTAL_2 as u8);
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
                                                             "192.168.0.101", 8000).ok();
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
    screen_draw: fn(&Self, bool),
    screen_changed: bool,
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
                let newScreen = self.screen_draw;
                if data.b1 {
                    self.screen_draw = Self::data_screen_draw;
                } else if data.b2 {
                    self.screen_draw = Self::clion_screen_draw;
                } else if data.b3 {
                    self.screen_draw = Self::rust_screen_draw;
                } else if data.b4 {
                    self.screen_draw = Self::jb_screen_draw;
                }
                if &self.screen_draw != &newScreen {
                    self.screen_changed = true;
                    self.screen_draw = newScreen;
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
    pub fn jb_screen_draw(&self, firstDraw: bool) {
        if firstDraw {
            unsafe {
                ILI9341_Draw_Image(jb_logo.as_ptr(), SCREEN_HORIZONTAL_2 as u8);
            }
        }
    }
    pub fn rust_screen_draw(&self, firstDraw: bool) {
        if firstDraw {
            //        unsafe {
//            ILI9341_Draw_Image(ferris.as_ptr(), SCREEN_HORIZONTAL_2 as u8);
//        }
            self.jb_screen_draw(firstDraw);
        }
    }
    pub fn clion_screen_draw(&self, firstDraw: bool) {
        if firstDraw {

//        unsafe {
//            ILI9341_Draw_Image(cl_logo.as_ptr(), SCREEN_HORIZONTAL_2 as u8);
//        }
        }
        self.jb_screen_draw(firstDraw);
    }
    pub fn data_screen_draw(&self, firstDraw: bool) {
        unsafe {
            if firstDraw {
                ILI9341_Fill_Screen(WHITE as u16);
                ILI9341_Draw_Filled_Circle(260, 180, 60, 0xFFE0);
                ILI9341_Draw_Hollow_Circle(260, 180, 60, 0);
            }
            //todo clear text under
            output_data_line(0, 0, "LM: ", || self.left_motor);
            output_data_line(0, 1, "RM: ", || self.right_motor);
            output_data_line(0, 2, "RDR: ", || radar_range());

            ILI9341_Draw_Filled_Rectangle_Coord(220, 140, 240, 160, alarm_color(PRX_BR));
            ILI9341_Draw_Filled_Rectangle_Coord(280, 140, 300, 160, alarm_color(PRX_BL));
            ILI9341_Draw_Filled_Rectangle_Coord(220, 200, 240, 220, alarm_color(PRX_FR));
            ILI9341_Draw_Filled_Rectangle_Coord(280, 200, 300, 220, alarm_color(PRX_FL));
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

