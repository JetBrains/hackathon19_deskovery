#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::Response;
use rocket::State;
use rocket::http::ContentType;
use std::io::Cursor;
use std::sync::atomic::{AtomicPtr, AtomicU32, Ordering};
use std::sync::Mutex;

struct Data {
    d: Mutex<_Data>
}

impl Data {
    fn new() -> Self {
        Data { d: Mutex::new(_Data::new()) }
    }
}

struct _Data {
    x: i16,
    y: i16,
}

impl _Data {
    fn new() -> _Data {
        _Data { x: 0, y: 0 }
    }
}

#[get("/")]
fn index() -> Response<'static> {
    let bdy = include_str!("index.html").to_string();
    Response::build().header(ContentType::HTML).sized_body(Cursor::new(bdy)).finalize()
}

#[get("/poll")]
fn poll(data: State<Data>) -> String {
    let d = data.d.lock().unwrap();
    format!("x: {}; y: {}", d.x, d.y)
}

#[get("/push?<x>&<y>")]
fn push(data: State<Data>, x: String, y: String) {
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
    rocket::ignite().manage(Data::new()).mount("/", routes![index, poll, push]).launch();
}