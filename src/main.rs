extern crate actix_web;
use actix_web::server;

mod mypaste {
    use actix_web::{App, HttpRequest};

    fn paste(_req: &HttpRequest) -> String {
        "Hello Paste!".to_owned()
    }

    pub(crate) fn app() -> App {
        App::new().prefix("/").resource("/paste", |r| r.f(paste))
    }
}

fn main() {
    server::new(|| mypaste::app())
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
