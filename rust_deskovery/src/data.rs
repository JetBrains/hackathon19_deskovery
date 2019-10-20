use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
pub struct DeskoveryData {
    pub x: i32,
    pub y: i32,
    pub th: i32,
    pub ps1: bool,
    pub ps2: bool,
    pub ps3: bool,
    pub ps4: bool,
    pub dto: i32,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ServerData {
    pub x: i32,
    pub y: i32,
    pub b1: bool,
    pub b2: bool,
    pub b3: bool,
    pub b4: bool,
}
