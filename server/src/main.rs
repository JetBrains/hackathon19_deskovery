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
use std::cmp::{min, max};
use image::{ImageFormat, ImageBuffer, Rgba, Luma, ConvertBuffer, png, ColorType};
use image::DynamicImage::ImageLuma8;
use bmp::{Image, Pixel};
use std::fs::File;


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

struct _MyData {
    controller: ControllerData,
    deskovery: Vec<DeskoveryData>,
    field_map: [u8; FIELD_SIZE * FIELD_SIZE],
}

impl _MyData {
    fn new() -> _MyData {
        _MyData {
            controller: ControllerData { x: 0, y: 0 },
            deskovery: vec![],
            field_map: [0u8; FIELD_SIZE * FIELD_SIZE],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ControllerData {
    x: i16,
    y: i16,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct DeskoveryData {
    x: i32,
    // mm
    y: i32,
    // mm
    theta: i32,
    // degree
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

#[post("/poll", format = "json", data = "<deskovery_data_json>")]
fn poll(data: State<MyData>, deskovery_data_json: Json<DeskoveryData>) -> String {
    let mut d = data.d.lock().unwrap();
    let deskovery_data = deskovery_data_json.0;
    d.deskovery.push(deskovery_data);

    let field_x = min(max(deskovery_data.x + FIELD_SIZE as i32 / 2, 0), FIELD_SIZE as i32) as usize;
    let field_y = min(max(deskovery_data.y + FIELD_SIZE as i32 / 2, 0), FIELD_SIZE as i32) as usize;
    d.field_map[field_x * FIELD_SIZE + field_y] = 1;

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

#[get("/map")]
fn get_map(data: State<MyData>) -> Response<'static> {
    let mut d = data.d.lock().unwrap();

    let v = vec![
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 255, 255, 255, 0, 0, 0,
        0, 0, 255, 255, 255, 0, 0, 0,
        0, 0, 255, 255, 255, 0, 0, 0,
        0, 0, 255, 255, 255, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0
    ];

    let mut my_bmp = Image::new(8, 8);
    for (x, y) in my_bmp.coordinates() {
        my_bmp.set_pixel(x, y, px!(x, y, v[(x * 8 + y) as usize]));
    }
    my_bmp.save("img.bmp");

    let mut buffer = vec![];
    buffer.resize(1024, 0);
    let mut f = File::open("img.bmp").unwrap();
    f.read(&mut buffer);

    Response::build()
        .header(ContentType::BMP)
        .sized_body(Cursor::new(buffer))
        .finalize()
}

fn main() {
    rocket::ignite()
        .manage(MyData::new())
        .mount("/", routes![index, poll, push, get_map])
        .launch();
}
