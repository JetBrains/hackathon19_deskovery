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
use rocket::config::Environment;


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
    pub th: i32,
    // theta
    pub ps1: bool,
    // proximity_sensor
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
    let mut d = data.d.lock().unwrap_or_else(|e| e.into_inner());
    let descovery_data = &mut d.deskovery;

    if descovery_data.len() >= 1000 {
        descovery_data.clear();
    }

    let deskovery_data = deskovery_data_json.0;
    println!("RECEIVED: {:?}", deskovery_data);
    descovery_data.extend(deskovery_data.clone().into_iter());

    for data in &deskovery_data {
        let field_x = min(max(data.x + FIELD_SIZE as i32 / 2, 0), FIELD_SIZE as i32 - 1) as usize;
        let field_y = min(max(data.y + FIELD_SIZE as i32 / 2, 0), FIELD_SIZE as i32 - 1) as usize;
        let index = min(field_x * FIELD_SIZE + field_y, d.field_map.len() - 1);
        d.field_map[index] = FIELD_COLOR;
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
    let d = data.d.lock().unwrap_or_else(|e| e.into_inner());
    Response::build()
        .header(ContentType::Plain)
        .sized_body(Cursor::new(serde_json::to_string(&d.deskovery).unwrap()))
        .finalize()
}

#[post("/delete_map_data")]
fn delete_map_data(data: State<MyData>) -> Response<'static> {
    let mut d = data.d.lock().unwrap_or_else(|e| e.into_inner());
    d.deskovery.clear();
    d.field_map = [0; FIELD_SIZE * FIELD_SIZE];

    Response::build()
        .header(ContentType::Plain)
        .sized_body(Cursor::new("Cleaned"))
        .finalize()
}

#[get("/map")]
fn get_map(data: State<MyData>) -> Response<'static> {
    let d = data.d.lock().unwrap_or_else(|e| e.into_inner());

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
    let mut c = rocket::Config::new(Environment::Staging);
    c.set_keep_alive(0);
    rocket::custom(c)
        .manage(MyData::new())
        .mount("/", routes![index, poll, push, get_map, get_map_data, delete_map_data])
        .launch();
}

#[cfg(test)]
mod test {
    use crate::{DeskoveryData, FIELD_SIZE};
    use bmp::{Image, Pixel};
    use image::{ImageBuffer, ImageFormat};
    use std::collections::HashSet;

    #[test]
    fn test() {
        let mock_data = r#"[{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":87},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":92},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":86},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":85},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":87},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":84},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":87},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":88},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":85},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":83},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":87},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":100},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":82},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":81},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":82},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":88},{"x":36,"y":0,"th":0,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":84},{"x":44,"y":-2,"th":288,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":830},{"x":47,"y":-15,"th":47,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":739},{"x":47,"y":-15,"th":47,"ps1":true,"ps2":true,"ps3":true,"ps4":true,"dto":-1},{"x":47,"y":-15,"th":47,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":77},{"x":348,"y":-195,"th":315,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":67},{"x":381,"y":-90,"th":202,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":63},{"x":375,"y":-93,"th":207,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":65},{"x":386,"y":23,"th":321,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":69},{"x":1153,"y":-538,"th":326,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":57},{"x":2018,"y":-1052,"th":331,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":45},{"x":2120,"y":-978,"th":100,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":39},{"x":2044,"y":-841,"th":0,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":-1},{"x":2036,"y":-841,"th":220,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":434},{"x":2036,"y":-840,"th":232,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":455},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":426},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":423},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":419},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":380},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":380},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":373},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":377},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":312},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":321},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":359},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":399},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":413},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":391},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":408},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":411},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":407},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":396},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":377},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":384},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":372},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":367},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":380},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":384},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":393},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":397},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":388},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":380},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":387},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":393},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":388},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":389},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":383},{"x":2021,"y":-1277,"th":269,"ps1":false,"ps2":false,"ps3":false,"ps4":false,"dto":394}]"#;
        let deskovery_data: Vec<DeskoveryData> = serde_json::from_str(mock_data).unwrap();

        if deskovery_data.is_empty() {
            return;
        }

        let mut min_x: Option<i32> = None;
        let mut max_x: Option<i32> = None;
        let mut min_y: Option<i32> = None;
        let mut max_y: Option<i32> = None;

        for data in &deskovery_data {
            if min_x.is_none() || data.x < min_x.unwrap() {
                min_x = Some(data.x);
            }
            if max_x.is_none() || data.x > max_x.unwrap() {
                max_x = Some(data.x);
            }
            if min_y.is_none() || data.y < min_y.unwrap() {
                min_y = Some(data.y);
            }
            if max_y.is_none() || data.y > max_y.unwrap() {
                max_y = Some(data.y);
            }
        }

        let min_x: i32 = min_x.unwrap();
        let max_x: i32 = max_x.unwrap();
        let min_y: i32 = min_y.unwrap();
        let max_y: i32 = max_y.unwrap();

        let scale_x = FIELD_SIZE as f64 / (max_x - min_x) as f64;
        let scale_y = FIELD_SIZE as f64 / (max_y - min_y) as f64;


        let mut test_bmp = Image::new(FIELD_SIZE as u32 + 10, FIELD_SIZE as u32 + 10);
        for i in 0..FIELD_SIZE as u32 + 10 {
            for j in 0..FIELD_SIZE as u32 + 10 {
                test_bmp.set_pixel(i, j, Pixel {
                    r: 255,
                    g: 255,
                    b: 255,
                });
            }
        }

        let mut prev_x: Option<u32> = None;
        let mut prev_y: Option<u32> = None;

        deskovery_data.iter().for_each(|DeskoveryData { x, y, .. }| {
            let image_x = ((x - min_x) as f64 * scale_x) as u32;
            let image_y = ((y - min_y) as f64 * scale_y) as u32;
            dbg!(image_x, image_y, "====");
            draw_square(&mut test_bmp, image_x, image_y);

            // Bresenham's algorithm
            if prev_x.is_some() && prev_y.is_some() {
                let prev_x = prev_x.unwrap();
                let prev_y = prev_y.unwrap();
                let delta_x: i32 = image_x as i32 - prev_x as i32;
                let delta_y: i32 = image_y as i32 - prev_y as i32;
                let delta_err = (delta_x as f64 / delta_y as f64).abs();

                let mut error = 0.0; // No error at start
                let mut y: u32 = prev_y;
                for x in prev_x..image_x {
                    draw_square(&mut test_bmp, x, y);
                    error = error + delta_err;
                    if error >= 0.5 {
                        y = (y as i32 + delta_y.signum()) as u32;
                        error = error - 1.0
                    }
                }
            }

            prev_x = Some(image_x);
            prev_y = Some(image_y);
        });

        test_bmp.save("test.bmp").unwrap();
    }

    fn draw_square(image: &mut Image, center_x: u32, center_y: u32) {
        let mut square_axes = Vec::with_capacity(9);

        square_axes.push((center_x, center_y));
        square_axes.push((center_x + 1, center_y));
        square_axes.push((center_x + 1, center_y + 1));
        square_axes.push((center_x, center_y + 1));

        let square_size = 4;

//        let square_start_x = center_x as i32 - square_size;
//        let square_start_y = center_y as i32 - square_size;

        if center_x != 0 {
            square_axes.push((center_x - 1, center_y));
            square_axes.push((center_x - 1, center_y + 1));
            if center_y != 0 {
                square_axes.push((center_x - 1, center_y - 1));
            }
        }

        if center_y != 0 {
            square_axes.push((center_x + 1, center_y - 1));
            square_axes.push((center_x, center_y - 1));
        }

        square_axes.into_iter()
            .for_each(|(x, y)| {
                image.set_pixel(x, y, Pixel {
                    r: 0,
                    g: 0,
                    b: 0,
                })
            });
    }
}
