#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use core::panic::PanicInfo;

const WHEEL_RADIUS_CM: f64 = 3.5;
const WHEEL_BASE_CM: f64 = 14.0;

// TODO this is fugly
pub mod libc {
    pub type c_int = i32;
    pub type c_uint = u32;
    pub type c_long = i64;
    pub type c_ulong = u64;
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

#[no_mangle]
pub extern "C" fn rust_main() {
    //    let s = "Hello, Embedded World";

    unsafe {
        //        outputStr(s.as_ptr(), s.len());
        let mut brightness: i32 = 0;

        let mut odo_computer = OdometryComputer {
            lastSample: system_ticks() as f64,
            x: 0.0,
            y: 0.0,
            theta: 0.0,
            oldTachoL: 0,
            oldTachoR: 0
        };

        loop {
            led_control(true);
            delay_ms(300);
            led_control(false);
            brightness = (brightness + 10) % 102;
            display_bg_control(brightness);
            LCD5110_set_XY(0, 0);
            LCD5110_write_string(b"This is RUST!" as *const u8);
            odo_computer.compute_odometry(left_ticks(), right_ticks(), system_ticks() as f64);
        }
    }
}

pub const PI: f64 = core::f64::consts::PI;

//fn is_infinite(x: f64) -> bool {
//    x.abs() == core::f64::INFINITY
//}
//
//fn is_nan(x: f64) -> bool {
//    x != x
//}

struct OdometryComputer {
    lastSample: f64,
    x: f64,
    y: f64,
    theta: f64,
    oldTachoL: i64,
    oldTachoR: i64,
}

fn sin(x: f64) -> f64 {
    x - x*x*x / 6.0 + x*x*x*x*x / 120.0
}

fn cos(x: f64) -> f64 {
    1.0 - x*x / 2.0 + x*x*x*x / 24.0
}

impl OdometryComputer {
    fn compute_odometry(&mut self, tachoL: i64, tachoR: i64, timeStamp: f64) {
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
        let newSample = timeStamp;
        // let dTime = newSample - self.lastSample;
        self.lastSample = newSample;
        self.x += dx;
        self.y += dy;
        self.theta += dTurnAngle;
        if self.theta < 0.0 {
            self.theta += 2.0 * PI;
        } else if self.theta > 2.0 * PI {
            self.theta -= 2.0 * PI;
        }

        /* TODO: is it needed?
        let orientation: Quaternion = odometry.getPose().getPose().getOrientation();
        roll_pitch_yaw(0, 0, theta, orientation);

        let linear: Vector3 = odometry.getTwist().getTwist().getLinear();
        linear.setX(dx / dTime);
        linear.setY(dy / dTime);
        linear.setZ(0);
        let angular: Vector3 = odometry.getTwist().getTwist().getAngular();
        angular.setX(0);
        angular.setY(0);
        angular.setZ(dTurnAngle / dTime);

        let translation: Vector3 = transformStamped.getTransform().getTranslation();
        translation.setX(x);
        translation.setY(y);
        translation.setZ(0);
        roll_pitch_yaw(0.0, 0.0, theta, transformStamped.getTransform().getRotation());
        */
    }

    /*
    fn roll_pitch_yaw(roll: f64, pitch: f64, yaw: f64, q: Quaternion) {
        let halfYaw = yaw * 0.5;
        let halfPitch = pitch * 0.5;
        let halfRoll = roll * 0.5;
        let cosYaw = halfYaw.cos();
        let sinYaw = halfYaw.sin();
        let cosPitch = halfPitch.cos();
        let sinPitch = halfPitch.sin();
        let cosRoll = halfRoll.cos();
        let sinRoll = halfRoll.sin();
        q.setX(sinRoll * cosPitch * cosYaw - cosRoll * sinPitch * sinYaw); // x
        q.setY(cosRoll * sinPitch * cosYaw + sinRoll * cosPitch * sinYaw); // y
        q.setZ(cosRoll * cosPitch * sinYaw - sinRoll * sinPitch * cosYaw); // z
        q.setW(cosRoll * cosPitch * cosYaw + sinRoll * sinPitch * sinYaw);
    }
    */
}
