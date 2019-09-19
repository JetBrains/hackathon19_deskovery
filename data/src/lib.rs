use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct DeskoveryData {

}

#[derive(Serialize, Deserialize)]
pub struct ServerData {
    x: i32,
    y: i32
}
