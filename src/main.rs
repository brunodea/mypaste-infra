extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate md5;

mod cache;
mod content;
mod paste_app;

use std::env;

fn main() {
    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let cache = paste_app::memory_cache();
    actix_web::server::new(move || paste_app::paste_app(cache.clone()))
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
