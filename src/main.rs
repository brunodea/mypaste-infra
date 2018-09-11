extern crate actix_web;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use actix_web::server;

mod mypaste {
    use actix_web::{http, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse};
    use futures::future::Future;

    #[derive(Debug, Serialize, Deserialize)]
    struct PasteContent {
        content: String,
    }

    fn paste(req: &HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
        req.json()
            .from_err() // convert all errors into Error
            .and_then(|content: PasteContent| {
                println!("request content: {:?}", content);
                Ok(HttpResponse::Ok().json(PasteContent {
                    content: "Worked!".to_string(),
                }))
            }).responder()
    }

    pub(crate) fn app() -> App {
        App::new().resource("/paste", |r| r.method(http::Method::POST).f(paste))
    }
}

fn main() {
    server::new(|| mypaste::app())
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
