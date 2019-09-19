use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DeskoveryData {
    x: f64,
    y: f64,
    theta: f64,
    proximity_sensor_1: bool,
    proximity_sensor_2: bool,
    proximity_sensor_3: bool,
    proximity_sensor_4: bool,
    distance_to_obstacle: u32,
}

#[derive(Serialize, Deserialize)]
pub enum RequestType {
    Move
}

#[derive(Serialize, Deserialize)]
pub struct ServerData {
    x: i32,
    y: i32,
    theta: f64,
}
