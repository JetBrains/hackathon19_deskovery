#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::io::{Cursor, Read};
use std::sync::Mutex;
use rocket_contrib::json::Json;
use serde::{Deserialize};
use rocket::{Request, Response, Outcome::*};
use rocket::data::{FromData, Outcome, Transform, Transformed};
use rocket::{State, Data};
use rocket::http::{ContentType, Status};


struct MyData {
    d: Mutex<_MyData>,
    deskovery_data_vec: Mutex<Vec<DeskoveryData>>,
}

impl MyData {
    fn new() -> Self {
        MyData {
            d: Mutex::new(_MyData::new()),
            deskovery_data_vec: Mutex::new(vec![])
        }
    }
}

struct _MyData {
    x: i16,
    y: i16,
}

impl _MyData {
    fn new() -> _MyData {
        _MyData { x: 0, y: 0 }
    }
}

#[derive(Deserialize)]
struct DeskoveryData {
    x: f64,
    y: f64,
    theta: f64,
}

impl DeskoveryData {
    fn new() -> DeskoveryData {
        DeskoveryData { x: 0.0, y: 0.0, theta: 0.0 }
    }
}

const DESKOVERY_DATA_LIMIT: u64 = 256;

impl<'a> FromData<'a> for DeskoveryData {
    type Error = ();
    type Owned = String;
    type Borrowed = str;

    fn transform(request: &Request, data: Data) -> Transform<Outcome<Self::Owned, Self::Error>> {
        let mut stream = data.open().take(DESKOVERY_DATA_LIMIT);
        let mut string = String::with_capacity((DESKOVERY_DATA_LIMIT / 2) as usize);
        let outcome: Outcome<Self::Owned, Self::Error> = match stream.read_to_string(&mut string) {
            Ok(_) => Success(string),
            Err(e) => Failure((Status::InternalServerError, ()))
        };

        Transform::Borrowed(outcome)
    }

    fn from_data(request: &Request, outcome: Transformed<'a, Self>) ->  Outcome<Self, Self::Error>  {
        let string = outcome.borrowed()?;

        let splits: Vec<&str> = string.split(" ").collect();
        if splits.len() != 3 || splits.iter().any(|s| s.is_empty()) {
            return Failure((Status::UnprocessableEntity, ()));
        }

        Success(DeskoveryData {
            x: splits[0].parse::<f64>().unwrap(),
            y: splits[1].parse::<f64>().unwrap(),
            theta: splits[2].parse::<f64>().unwrap()
        })
    }
}


#[get("/")]
fn index() -> Response<'static> {
    let bdy = include_str!("index.html").to_string();
    Response::build().header(ContentType::HTML).sized_body(Cursor::new(bdy)).finalize()
}

#[post("/poll", format = "json", data = "<deskovery_data>")]
fn poll(data: State<MyData>, deskovery_data: Json<DeskoveryData>) -> String {
    let d = data.d.lock().unwrap();
    let mut deskovery_data_vec = data.deskovery_data_vec.lock().unwrap();
    deskovery_data_vec.push(deskovery_data.0);
    format!("x: {}; y: {}", d.x, d.y)
}

#[get("/push?<x>&<y>")]
fn push(data: State<MyData>, x: String, y: String) {
    println!("Accept x: {}; y: {}", x, y);
    match (x.parse::<f64>().ok(), y.parse::<f64>().ok()) {
        (Some(x), Some(y)) => {
            let x = (x * 1000.0) as i16;
            let y = (y * 1000.0) as i16;
            let mut d = data.d.lock().unwrap();
            d.x = x;
            d.y = y;
        }
        _ => println!("Invalid input!")
    }
}

fn main() {
    rocket::ignite().manage(MyData::new()).mount("/", routes![index, poll, push]).launch();
}