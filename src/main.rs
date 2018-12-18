#![feature(proc_macro_hygiene, decl_macro)]

extern crate base64;
extern crate md5;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rocket;

mod cache;
mod content;
mod paste_app;

fn main() {
    paste_app::start();
}
