use std::io::Read;

use crate::cache::{Cache, MemoryCache};
use crate::content::{Content, PasteContent};

use rocket::{
    self,
    data::{self, FromDataSimple},
    http::{ContentType, Status},
    response, Data, Outcome, Request, State,
};
use serde_json;
use std::sync::{Arc, Mutex};

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
fn paste_get(hash_id: String, cache: State<SharedCache>) -> response::status::Accepted<String> {
    let cache = cache.inner().lock().unwrap();
    let value = match cache.get(hash_id.to_string()) {
        Some(paste) => serde_json::to_string(paste).unwrap(),
        None => "Not found!".to_string(),
    };

    response::status::Accepted(Some(value))
}

#[post("/", format = "application/json", data = "<content>")]
fn paste_post(
    content: PasteContent,
    cache: State<SharedCache>,
) -> response::status::Accepted<String> {
    let hash = content.hash();

    let mut cache = cache.inner().lock().unwrap();
    cache.set(content);

    response::status::Accepted(Some(format!("{:?}", hash)))
}

pub(crate) fn start() {
    rocket::ignite()
        .manage(memory_cache())
        .mount("/paste", routes![paste_get, paste_post])
        .launch();
}
