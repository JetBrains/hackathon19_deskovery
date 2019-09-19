use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub struct DeskoveryData {
    pub x: i32,
    pub y: i32,
    pub th: i32,
    pub ps1: bool,
    pub ps2: bool,
    pub ps3: bool,
    pub ps4: bool,
    pub dto: u32,
}

#[derive(Serialize, Deserialize)]
pub enum RequestType {
    Move
}

#[derive(Serialize, Deserialize)]
pub struct ServerData {
    pub x: i32,
    pub y: i32,
//    pub theta: f64,
}
