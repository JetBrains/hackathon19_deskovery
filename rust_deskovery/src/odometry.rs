use core::intrinsics::{cosf64, sinf64};

const PI: f64 = core::f64::consts::PI;

const WHEEL_RADIUS_CM: f64 = 3.5;
const WHEEL_BASE_CM: f64 = 14.0;
const WHEEL_TICKS_PER_CIRCLE: i32 = 740;
const WHEEL_CIRCLE_LEN_CM: f64 = 2.0 * PI * WHEEL_RADIUS_CM;
const WHEEL_TICK_IN_CM: f64 = WHEEL_TICKS_PER_CIRCLE as f64 / WHEEL_CIRCLE_LEN_CM as f64;
const WHEEL_RADIUS_TICKS: f64 = WHEEL_RADIUS_CM * WHEEL_TICK_IN_CM;
const WHEEL_BASE_TICKS: f64 = WHEEL_BASE_CM * WHEEL_TICK_IN_CM;

fn sin(x: f64) -> f64 {
    unsafe { sinf64(x) }
}
fn cos(x: f64) -> f64 {
    unsafe { cosf64(x) }
}

pub struct Position {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

pub struct OdometryComputer<'a> {
    pub position: &'a mut Position,
    pub old_left_ticks: i32,
    pub old_right_ticks: i32,
}

impl<'a> OdometryComputer<'a> {
    pub fn update(&mut self, left_ticks: i32, right_ticks: i32) {
        let dL = left_ticks - self.old_left_ticks;
        self.old_left_ticks = left_ticks;
        let dR = right_ticks - self.old_right_ticks;
        self.old_right_ticks = right_ticks;
        let dTrackL: f64 = dL as f64 * PI / 180.0 * WHEEL_RADIUS_TICKS;
        let dTrackR = dR as f64 * PI / 180.0 * WHEEL_RADIUS_TICKS;
        let dTrack = dTrackR - dTrackL;
        let dTrackAvr = (dTrackR + dTrackL) / 2.0;
        let dTurnAngle = dTrack / WHEEL_BASE_TICKS;
        let turnRadius = dTrackAvr / dTurnAngle;
        let dx;
        let dy;
        if turnRadius.is_infinite() || turnRadius.is_nan() {
            dx = dTrackAvr * cos(self.position.theta);
            dy = dTrackAvr * sin(self.position.theta);
        } else {
            let turnAngle = self.position.theta - PI / 2.0;
            dx = turnRadius * (cos(turnAngle + dTurnAngle) - cos(turnAngle));
            dy = turnRadius * (sin(turnAngle + dTurnAngle) - sin(turnAngle));
        }
        self.position.x += dx;
        self.position.y += dy;
        self.position.theta += dTurnAngle;
        if self.position.theta < 0.0 {
            self.position.theta += 2.0 * PI;
        } else if self.position.theta > 2.0 * PI {
            self.position.theta -= 2.0 * PI;
        }
    }
}
