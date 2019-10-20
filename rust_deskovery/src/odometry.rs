use core::intrinsics::{cosf64, fabsf64, sinf64};

const PI: f64 = core::f64::consts::PI;
const EPS: f64 = 0.000_001;

const WHEEL_RADIUS_MM: f64 = 35.0;
const WHEEL_BASE_MM: f64 = 140.0;
const WHEEL_TICKS_PER_CIRCLE: i32 = 720;

const WHEEL_CIRCLE_LEN_MM: f64 = 2.0 * PI * WHEEL_RADIUS_MM;
const WHEEL_TICK_IN_MM: f64 = WHEEL_TICKS_PER_CIRCLE as f64 / WHEEL_CIRCLE_LEN_MM as f64;
//const WHEEL_RADIUS_TICKS: f64 = WHEEL_RADIUS_MM * WHEEL_TICK_IN_MM;
//const WHEEL_BASE_TICKS: f64 = WHEEL_BASE_MM * WHEEL_TICK_IN_MM;

fn sin(x: f64) -> f64 {
    unsafe { sinf64(x) }
}

fn cos(x: f64) -> f64 {
    unsafe { cosf64(x) }
}

fn abs(x: f64) -> f64 {
    unsafe { fabsf64(x) }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub theta: f64,
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        abs(self.x - other.x) < EPS
            && abs(self.y - other.y) < EPS
            && abs(self.theta - other.theta) < EPS
    }
}

impl Eq for Position {}

#[derive(Default)]
pub struct OdometryComputer {
    position: Position,
    old_left_mm: f64,
    old_right_mm: f64,
}

impl OdometryComputer {
    pub fn position(&self) -> Position {
        self.position
    }

    pub fn update(&mut self, left_ticks: i32, right_ticks: i32) {
        let left_mm = left_ticks as f64 / WHEEL_TICK_IN_MM;
        let right_mm = right_ticks as f64 / WHEEL_TICK_IN_MM;

        // NB: all further computations operate with mm

        let d_left = left_mm - self.old_left_mm;
        self.old_left_mm = left_mm;
        let d_right = right_mm - self.old_right_mm;
        self.old_right_mm = right_mm;

        let d_track = d_right - d_left;
        let d_track_avr = (d_right + d_left) / 2.0;
        let d_turn_angle = d_track / WHEEL_BASE_MM;
        let turn_radius = d_track_avr / d_turn_angle;
        let (dx, dy) = if turn_radius.is_infinite() || turn_radius.is_nan() {
            let dx = d_track_avr * cos(self.position.theta);
            let dy = d_track_avr * sin(self.position.theta);
            (dx, dy)
        } else {
            let turn_angle = self.position.theta - PI / 2.0;
            let dx = turn_radius * (cos(turn_angle + d_turn_angle) - cos(turn_angle));
            let dy = turn_radius * (sin(turn_angle + d_turn_angle) - sin(turn_angle));
            (dx, dy)
        };
        self.position.x += dx;
        self.position.y += dy;
        self.position.theta += d_turn_angle;
        if self.position.theta < 0.0 {
            self.position.theta += 2.0 * PI;
        } else if self.position.theta > 2.0 * PI {
            self.position.theta -= 2.0 * PI;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::odometry::{
        OdometryComputer, Position, WHEEL_CIRCLE_LEN_MM, WHEEL_TICKS_PER_CIRCLE,
    };

    fn do_test(left_ticks: i32, right_ticks: i32, expected: Position) {
        let mut odo_computer = OdometryComputer::default();
        odo_computer.update(left_ticks, right_ticks);
        let actual = odo_computer.position();
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_forward() {
        do_test(
            WHEEL_TICKS_PER_CIRCLE,
            WHEEL_TICKS_PER_CIRCLE,
            Position {
                x: WHEEL_CIRCLE_LEN_MM,
                y: 0.0,
                theta: 0.0,
            },
        );
    }

    #[test]
    fn test_backward() {
        do_test(
            -WHEEL_TICKS_PER_CIRCLE,
            -WHEEL_TICKS_PER_CIRCLE,
            Position {
                x: -WHEEL_CIRCLE_LEN_MM,
                y: 0.0,
                theta: 0.0,
            },
        );
    }

    #[test]
    fn test_forward_twice() {
        let mut odo_computer = OdometryComputer::default();
        odo_computer.update(WHEEL_TICKS_PER_CIRCLE, WHEEL_TICKS_PER_CIRCLE);
        odo_computer.update(2 * WHEEL_TICKS_PER_CIRCLE, 2 * WHEEL_TICKS_PER_CIRCLE);
        let actual = odo_computer.position();
        assert_eq!(
            actual,
            Position {
                x: 2.0 * WHEEL_CIRCLE_LEN_MM,
                y: 0.0,
                theta: 0.0
            }
        );
    }

    #[test]
    fn test_turn() {
        do_test(
            10,
            0,
            Position {
                x: 1.5270419524192766,
                y: -0.01665810440636073,
                theta: 6.261368691529657,
            },
        );
    }
}
