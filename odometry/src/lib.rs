#![no_std]
#![feature(core_intrinsics)]

use core::intrinsics::{cosf64, sinf64, fabsf64};

const PI: f64 = core::f64::consts::PI;

const WHEEL_RADIUS_MM: f64 = 35.0;
const WHEEL_BASE_MM: f64 = 140.0;
const WHEEL_TICKS_PER_CIRCLE: i32 = 720;

const WHEEL_CIRCLE_LEN_MM: f64 = 2.0 * PI * WHEEL_RADIUS_MM;
const WHEEL_TICK_IN_MM: f64 = WHEEL_TICKS_PER_CIRCLE as f64 / WHEEL_CIRCLE_LEN_MM as f64;
const WHEEL_RADIUS_TICKS: f64 = WHEEL_RADIUS_MM * WHEEL_TICK_IN_MM;
const WHEEL_BASE_TICKS: f64 = WHEEL_BASE_MM * WHEEL_TICK_IN_MM;

fn sin(x: f64) -> f64 {
    unsafe { sinf64(x) }
}

fn cos(x: f64) -> f64 {
    unsafe { cosf64(x) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub x: i32,     // mm
    pub y: i32,     // mm
    pub theta: i32, // degree
}

pub struct OdometryComputer {
    position: Position,
    old_left_mm: i32,
    old_right_mm: i32,
}

impl OdometryComputer {
    pub fn new() -> OdometryComputer {
        OdometryComputer {
            position: Position { x: 0, y: 0, theta: 0 },
            old_left_mm: 0,
            old_right_mm: 0,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn update(&mut self, left_ticks: i32, right_ticks: i32) {
        let left_mm = left_ticks as f64 / WHEEL_TICK_IN_MM;
        let right_mm = right_ticks as f64 / WHEEL_TICK_IN_MM;

        // NB: all further computations operate with mm

        let d_left = left_mm - self.old_left_mm as f64;
        self.old_left_mm = left_mm as i32;
        let d_right = right_mm - self.old_right_mm as f64;
        self.old_right_mm = right_mm as i32;

        let d_track = d_right - d_left;
        let d_track_avr = (d_right + d_left) / 2.0;
        let d_turn_angle = d_track / WHEEL_BASE_MM;
        let turn_radius = d_track_avr / d_turn_angle;
        let dx;
        let dy;
        let theta_radians = self.position.theta as f64 * PI / 180.0;
        if turn_radius.is_infinite() || turn_radius.is_nan() {
            dx = d_track_avr * cos(theta_radians);
            dy = d_track_avr * sin(theta_radians);
        } else {
            let turn_angle = theta_radians - PI / 2.0;
            dx = turn_radius * (cos(turn_angle + d_turn_angle) - cos(turn_angle));
            dy = turn_radius * (sin(turn_angle + d_turn_angle) - sin(turn_angle));
        }
        self.position.x += dx as i32;
        self.position.y += dy as i32;

        self.position.theta += (d_turn_angle / PI * 180.0) as i32;
        if self.position.theta < 0 {
            self.position.theta += 360;
        } else if self.position.theta > 360 {
            self.position.theta -= 360;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Position, OdometryComputer, WHEEL_TICKS_PER_CIRCLE, WHEEL_CIRCLE_LEN_MM};

    fn do_test(left_ticks: i32, right_ticks: i32, expected: Position) {
        let mut odo_computer = OdometryComputer::new();
        odo_computer.update(left_ticks, right_ticks);
        let actual = odo_computer.position();
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_forward() {
        do_test(
            WHEEL_TICKS_PER_CIRCLE,
            WHEEL_TICKS_PER_CIRCLE,
            Position { x: WHEEL_CIRCLE_LEN_MM as i32, y: 0, theta: 0 }
        );
    }

    #[test]
    fn test_backward() {
        do_test(
            -WHEEL_TICKS_PER_CIRCLE,
            -WHEEL_TICKS_PER_CIRCLE,
            Position { x: -WHEEL_CIRCLE_LEN_MM as i32, y: 0, theta: 0 }
        );
    }


    #[test]
    fn test_forward_twice() {
        let mut odo_computer = OdometryComputer::new();
        odo_computer.update(WHEEL_TICKS_PER_CIRCLE, WHEEL_TICKS_PER_CIRCLE);
        odo_computer.update(2 * WHEEL_TICKS_PER_CIRCLE, 2 * WHEEL_TICKS_PER_CIRCLE);
        let actual = odo_computer.position();
        assert_eq!(actual, Position { x: (2.0 * WHEEL_CIRCLE_LEN_MM) as i32, y: 0, theta: 0 });
    }

    #[test]
    fn test_turn() {
        do_test(
            500,
            0,
            Position { x: 62, y: -37, theta: 297 }
        );
    }

    #[test]
    fn test_stand() {
        let mut odo_computer = OdometryComputer::new();
        odo_computer.update(500, 100);
        assert_eq!(odo_computer.position().theta, 311);
        odo_computer.update(500, 100);
        assert_eq!(odo_computer.position().theta, 311);
        odo_computer.update(500, 100);
        assert_eq!(odo_computer.position().theta, 311);
        odo_computer.update(500, 100);
        assert_eq!(odo_computer.position().theta, 311);
        odo_computer.update(500, 100);
        assert_eq!(odo_computer.position().theta, 311);
    }
}
