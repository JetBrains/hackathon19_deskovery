#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate bmp;

use std::io::{Cursor, Read};
use std::sync::Mutex;

use rocket::data::{FromDataSimple, Outcome};
use rocket::http::{ContentType, Status};
use rocket::{Data, State};
use rocket::{Outcome::*, Request, Response};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use bmp::{Image, Pixel};
use std::fs::File;
use std::cmp::{min, max};


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

const FIELD_SIZE: usize = 1000;
const FIELD_COLOR: u8 = 255;

struct _MyData {
    controller: ControllerData,
    deskovery: Vec<DeskoveryData>,
    field_map: [u8; FIELD_SIZE * FIELD_SIZE],
}

impl _MyData {
    fn new() -> _MyData {
        _MyData {
            controller: ControllerData { x: 0, y: 0, b1: false, b2: false, b3: false, b4: false },
            deskovery: vec![],
            field_map: [0u8; FIELD_SIZE * FIELD_SIZE],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ControllerData {
    x: i16,
    y: i16,
    pub b1: bool,
    pub b2: bool,
    pub b3: bool,
    pub b4: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub struct DeskoveryData {
    pub x: i32,
    pub y: i32,
    pub th: i32, // theta
    pub ps1: bool, // proximity_sensor
    pub ps2: bool,
    pub ps3: bool,
    pub ps4: bool,
    pub dto: i32, // distance to object
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

#[post("/poll", format = "json", data = "<deskovery_data_json>")]
fn poll(data: State<MyData>, deskovery_data_json: Json<Vec<DeskoveryData>>) -> String {
    let mut d = data.d.lock().unwrap();
    let descovery_data = &mut d.deskovery;

    if descovery_data.len() >= 1000 {
        descovery_data.clear();
    }

    let deskovery_data = deskovery_data_json.0;
    println!("RECEIVED: {:?}", deskovery_data);
    descovery_data.extend(deskovery_data.clone().into_iter());

    for data in &deskovery_data {
        let field_x = min(max(data.x + FIELD_SIZE as i32 / 2, 0), FIELD_SIZE as i32) as usize;
        let field_y = min(max(data.y + FIELD_SIZE as i32 / 2, 0), FIELD_SIZE as i32) as usize;
        d.field_map[field_x * FIELD_SIZE + field_y] = FIELD_COLOR;
    }

    let out = serde_json::to_string(&d.controller).unwrap();
    println!("SENDING: {:?}", &out);
    out
}

#[get("/push?<x>&<y>&<b1>&<b2>&<b3>&<b4>")]
fn push(data: State<MyData>, x: String, y: String, b1: String, b2: String, b3: String, b4: String) {
    println!("Accept x: {}; y: {}", x, y);
    match (x.parse::<f64>().ok(), y.parse::<f64>().ok()) {
        (Some(x), Some(y)) => {
            let x = (x * 1000.0) as i16;
            let y = (y * 1000.0) as i16;
            let mut d = data.d.lock().unwrap();
            d.controller.x = x;
            d.controller.y = y;
            d.controller.b1 = b1 == "true";
            d.controller.b2 = b2 == "true";
            d.controller.b3 = b3 == "true";
            d.controller.b4 = b4 == "true";
        }
        _ => println!("Invalid input!"),
    }
}

#[get("/map_data")]
fn get_map_data(data: State<MyData>) -> Response<'static> {
    let d = data.d.lock().unwrap();
    Response::build()
        .header(ContentType::Plain)
        .sized_body(Cursor::new(serde_json::to_string(&d.deskovery).unwrap()))
        .finalize()
}

#[post("/delete_map_data")]
fn delete_map_data(data: State<MyData>) -> Response<'static> {
    let mut d = data.d.lock().unwrap();
    d.deskovery.clear();
    d.field_map = [0; FIELD_SIZE * FIELD_SIZE];

    Response::build()
        .header(ContentType::Plain)
        .sized_body(Cursor::new("Cleaned"))
        .finalize()
}

#[get("/map")]
fn get_map(data: State<MyData>) -> Response<'static> {
    let d = data.d.lock().unwrap();

//    let v = vec![
//        0, 0, 0, 0, 0, 0, 0, 0,
//        0, 0, 0, 0, 0, 0, 0, 0,
//        0, 0, 255, 255, 255, 0, 0, 0,
//        0, 0, 255, 255, 255, 0, 0, 0,
//        0, 0, 255, 255, 255, 0, 0, 0,
//        0, 0, 255, 255, 255, 0, 0, 0,
//        0, 0, 0, 0, 0, 0, 0, 0,
//        0, 0, 0, 0, 0, 0, 0, 0
//    ];

    let mut my_bmp = Image::new(FIELD_SIZE as u32, FIELD_SIZE as u32);
    for (x, y) in my_bmp.coordinates() {
        my_bmp.set_pixel(x, y, px!(x, y, d.field_map[(x * FIELD_SIZE as u32 + y) as usize]));
    }
    my_bmp.save("img.bmp").unwrap();

    let mut buffer = vec![];
    buffer.resize(FIELD_SIZE * FIELD_SIZE * 4, 0);
    let mut f = File::open("img.bmp").unwrap();
    f.read(&mut buffer).unwrap();

//    let zz = serde_json::to_string(&d.deskovery).unwrap();

    Response::build()
        .header(ContentType::BMP)
        .sized_body(Cursor::new(buffer))
        .finalize()
}

fn main() {
    rocket::ignite()
        .manage(MyData::new())
        .mount("/", routes![index, poll, push, get_map, get_map_data, delete_map_data])
        .launch();
}
