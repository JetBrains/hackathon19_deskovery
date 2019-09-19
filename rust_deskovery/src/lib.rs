#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![feature(intrinsics)]

use core::intrinsics::{cosf64, sinf64};
use core::panic::PanicInfo;

const WHEEL_RADIUS_CM: f64 = 3.5;
const WHEEL_BASE_CM: f64 = 14.0;

const PI: f64 = core::f64::consts::PI;

// TODO this is fugly
pub mod libc {
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i32;
    pub type c_ulong = u32;
    pub type c_uchar = u8;
    pub type c_char = i8;
    pub type c_double = f64;
}
include!("descovery_bindings.rs");

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    //    let mut host_stderr = HStderr::new();

    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    //    writeln!(host_stderr, "{}", info).ok();
    loop {}
}

fn display_text(x: u8, y: u8, s: &str) {
    unsafe {
        LCD5110_set_XY(x, y);
        let bytes = s.as_bytes();
        LCD5110_write_bytes(bytes.as_ptr() as *const u8, bytes.len() as u32);
    }
}

fn output_data_line<F>(y: u8, label: &str, dataGetter: F)
where
    F: FnOnce() -> i32,
{
    display_text(0, y, label);
    let mut buf: [u8; 10] = [0; 10];
    let mut index = buf.len() - 1;
    let mut val = dataGetter();
    let sign = val < 0;
    if sign {
        val = -val;
    }
    loop {
        buf[index] = (val % 10 + 48) as u8;
        index -= 1;
        val = val / 10;
        if val == 0 {
            break;
        }
    }
    if sign {
        buf[index] = '-' as u8;
        index -= 1;
    }
    unsafe {
        LCD5110_write_bytes(
            buf[index + 1..].as_ptr() as *const u8,
            (buf.len() - index - 1) as u32,
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
            theta: 0.0
        };
        let mut odo_computer = OdometryComputer {
            position: &mut position,
            oldTachoL: 0,
            oldTachoR: 0,
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

            let left_ticks = left_ticks(); //todo fix types
            let right_ticks = right_ticks(); //todo fix types
            let system_ticks = system_ticks();

            output_data_line(1, "B:     ", || brightness);
            output_data_line(2, "Rng:   ", || radar_range());
            output_data_line(3, "Left:  ", || left_ticks);
            output_data_line(4, "Right: ", || right_ticks);

            LCD5110_set_XY(3, 5);
            LCD5110_write_char(alarm_char(0));
            LCD5110_write_char(alarm_char(1));
            LCD5110_write_char(alarm_char(2));
            LCD5110_write_char(alarm_char(3));
            odo_computer.compute_odometry(left_ticks, right_ticks);

            // TODO: f64 printing
            // output_data_line(4, "x:     ", || position.x);
            // output_data_line(4, "y:     ", || position.y);

            /*
                        void debug_output(const unsigned char *p, unsigned int len); //todo implement
            */
        }
    }
}

struct Position {
    x: f64,
    y: f64,
    theta: f64,
}

struct OdometryComputer<'a> {
    position: &'a mut Position,
    oldTachoL: i32,
    oldTachoR: i32,
}

fn sin(x: f64) -> f64 {
    //    x - x*x*x / 6.0 + x*x*x*x*x / 120.0
    unsafe { sinf64(x) }
}

fn cos(x: f64) -> f64 {
    //    1.0 - x*x / 2.0 + x*x*x*x / 24.0
    unsafe { cosf64(x) }
}

impl OdometryComputer {
    fn compute_odometry(&mut self, tachoL: i32, tachoR: i32) {
        let dL = tachoL - self.oldTachoL;
        self.oldTachoL = tachoL;
        let dR = tachoR - self.oldTachoR;
        self.oldTachoR = tachoR;
        let dTrackL: f64 = dL as f64 * PI / 180.0 * WHEEL_RADIUS_CM;
        let dTrackR = dR as f64 * PI / 180.0 * WHEEL_RADIUS_CM;
        let dTrack = dTrackR - dTrackL;
        let dTrackAvr = (dTrackR + dTrackL) / 2.0;
        let dTurnAngle = dTrack / WHEEL_BASE_CM;
        let turnRadius = dTrackAvr / dTurnAngle;
        let dx;
        let dy;
        if turnRadius.is_infinite() || turnRadius.is_nan() {
            dx = dTrackAvr * cos(self.theta);
            dy = dTrackAvr * sin(self.theta);
        } else {
            let turnAngle = self.theta - PI / 2.0;
            dx = turnRadius * (cos(turnAngle + dTurnAngle) - cos(turnAngle));
            dy = turnRadius * (sin(turnAngle + dTurnAngle) - sin(turnAngle));
        }
        self.x += dx;
        self.y += dy;
        self.theta += dTurnAngle;
        if self.theta < 0.0 {
            self.theta += 2.0 * PI;
        } else if self.theta > 2.0 * PI {
            self.theta -= 2.0 * PI;
        }
    }
}
