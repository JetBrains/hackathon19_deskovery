#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> String {
    "Hello, world!".to_string()
}

#[get("/poll")]
fn poll() -> String {
    "Hello, world!".to_string()
}

fn main() {
    rocket::ignite().mount("/", routes![index, poll]).launch();
}