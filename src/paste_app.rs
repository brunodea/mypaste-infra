use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};

use rocket::{
    self,
    data::{self, FromDataSimple},
    get,
    http::{ContentType, Status},
    post,
    response::Response,
    routes, Data, Outcome, Request, State,
};

use serde_json;

use crate::cache::{Cache, MemoryCache};
use crate::content::{Content, PasteContent};

type SharedCache = Arc<Mutex<Box<MemoryCache>>>;

fn memory_cache() -> SharedCache {
    Arc::new(Mutex::new(Box::new(MemoryCache::new())))
}

impl FromDataSimple for PasteContent {
    type Error = String;

    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        // Ensure the content type is correct.
        let json_ct = ContentType::new("application", "json");
        if req.content_type() != Some(&json_ct) {
            return Outcome::Forward(data);
        }

        const LIMIT: u64 = 512;

        // Read json data to a String.
        let mut paste_content_str = String::new();
        if let Err(e) = data
            .open()
            .take(LIMIT)
            .read_to_string(&mut paste_content_str)
        {
            return Outcome::Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        // Parse JSON to PasteContent
        match serde_json::from_str::<PasteContent>(&paste_content_str) {
            Ok(content) => Outcome::Success(content),
            Err(_) => Outcome::Failure((Status::UnprocessableEntity, "Invalid JSON".into())),
        }
    }
}

#[get("/<hash_id>")]
fn paste_get(hash_id: String, cache: State<SharedCache>) -> Response {
    let cache = cache.inner().lock().unwrap();
    let (body, status) = if let Some(paste) = cache.get(hash_id.to_string()) {
        (serde_json::to_string(paste).unwrap(), Status::Accepted)
    } else {
        (
            format!("Unable to find data for {}.", hash_id),
            Status::NotFound,
        )
    };
    Response::build()
        .sized_body(Cursor::new(body))
        .status(status)
        .finalize()
}

#[post("/", format = "application/json", data = "<content>")]
fn paste_post(content: PasteContent, cache: State<SharedCache>) -> Response {
    let hash = content.hash();

    let mut cache = cache.inner().lock().unwrap();
    cache.set(content);

    Response::build()
        .sized_body(Cursor::new(format!("{:?}", hash)))
        .status(Status::Ok)
        .finalize()
}

pub(crate) fn start() {
    rocket::ignite()
        .manage(memory_cache())
        .mount("/paste", routes![paste_get, paste_post])
        .launch();
}
