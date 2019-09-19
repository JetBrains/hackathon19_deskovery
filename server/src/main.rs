#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::io::{Cursor, Read};
use std::sync::Mutex;

use rocket::data::{FromDataSimple, Outcome};
use rocket::http::{ContentType, Status};
use rocket::{Data, State};
use rocket::{Outcome::*, Request, Response};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};

struct MyData {
    d: Mutex<_MyData>,
}

impl MyData {
    fn new() -> Self {
        MyData {
            d: Mutex::new(_MyData::new()),
        }
    }
}

struct _MyData {
    controller: ControllerData,
    deskovery: Vec<DeskoveryData>,
}

impl _MyData {
    fn new() -> _MyData {
        _MyData {
            controller: ControllerData { x: 0, y: 0 },
            deskovery: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ControllerData {
    x: i16,
    y: i16,
}

#[derive(Serialize, Deserialize, Default)]
struct DeskoveryData {
    x: f64,
    y: f64,
    theta: f64,
    proximity_sensor_1: bool,
    proximity_sensor_2: bool,
    proximity_sensor_3: bool,
    proximity_sensor_4: bool,
    distance_to_obstacle: u32,
}

impl FromDataSimple for DeskoveryData {
    type Error = ();

    fn from_data(_: &Request, data: Data) -> Outcome<Self, Self::Error> {
        let mut data_vec = Vec::new();
        if let Err(_) = data.open().read_to_end(&mut data_vec) {
            return Failure((Status::InternalServerError, ()));
        }
        if let Ok(data) = serde_json::from_slice(&data_vec) {
            Success(data)
        } else {
            Failure((Status::UnprocessableEntity, ()))
        }
    }
}

#[get("/")]
fn index() -> Response<'static> {
    let bdy = include_str!("index.html").to_string();
    Response::build()
        .header(ContentType::HTML)
        .sized_body(Cursor::new(bdy))
        .finalize()
}

#[post("/poll", format = "json", data = "<deskovery_data>")]
fn poll(data: State<MyData>, deskovery_data: Json<DeskoveryData>) -> String {
    let mut d = data.d.lock().unwrap();
    d.deskovery.push(deskovery_data.0);
    serde_json::to_string(&d.controller).unwrap()
}

#[get("/push?<x>&<y>")]
fn push(data: State<MyData>, x: String, y: String) {
    println!("Accept x: {}; y: {}", x, y);
    match (x.parse::<f64>().ok(), y.parse::<f64>().ok()) {
        (Some(x), Some(y)) => {
            let x = (x * 1000.0) as i16;
            let y = (y * 1000.0) as i16;
            let mut d = data.d.lock().unwrap();
            d.controller.x = x;
            d.controller.y = y;
        }
        _ => println!("Invalid input!"),
    }
}

fn main() {
    rocket::ignite()
        .manage(MyData::new())
        .mount("/", routes![index, poll, push])
        .launch();
}
