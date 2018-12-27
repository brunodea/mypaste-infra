use std::io::{Cursor, Read};
use std::sync::{Arc, Mutex};

use rocket::{
    self,
    data::{self, FromDataSimple},
    get,
    http::{ContentType, Status},
    post,
    response::Response,
    routes, Data, Outcome, Request, Rocket, State,
};

use serde_json;

use crate::cache::{Cache, MemoryCache};
use crate::content::{Content, PasteContent};

type SharedCache = Arc<Mutex<Box<MemoryCache>>>;

fn memory_cache() -> SharedCache {
    Arc::new(Mutex::new(Box::new(MemoryCache::new())))
}

/// This impl allows having PasteContent as a parameter in handlers.
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

/// Gets the stored paste content for a certain hash id.
/// Returns 403 Not Found in case the hash id isn't found.
#[get("/<hash_id>")]
fn paste_get(hash_id: String, cache: State<SharedCache>) -> Response {
    let cache = cache.inner().lock().unwrap();
    let (body, status) = if let Some(paste) = cache.get(hash_id.to_string()) {
        (serde_json::to_string(paste).unwrap(), Status::Ok)
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

/// Stores a PasteContent to the cache.
/// The request is expected to have a JSON body that represents some PasteContent.
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

fn create_rocket() -> Rocket {
    rocket::ignite()
        .manage(memory_cache())
        .mount("/paste", routes![paste_get, paste_post])
}

/// Launches
pub(crate) fn start() {
    create_rocket().launch();
}

#[cfg(test)]
mod test {
    use super::*;
    use rocket::local::Client;

    #[test]
    fn get_non_existing_hash_responds_with_not_found() {
        let _rocket = create_rocket();
        let client = Client::new(_rocket).expect("valid rocket instance");

        let request = client.get("/paste/NonExistingHash");
        let mut response = request.dispatch();

        assert_eq!(response.status(), Status::NotFound);
        assert!(response.body_string().unwrap().contains("NonExistingHash"));
    }

    #[test]
    fn get_existing_hash_responds_with_paste_content_for_hash() {
        let _rocket = create_rocket();
        let client = Client::new(_rocket).expect("valid rocket instance");

        let request = client
            .post("/paste")
            .header(ContentType::JSON)
            .body("{\"PlainText\":\"Test of Content\"}");
        let mut response = request.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body_string(), Some("\"YTA0NzY\"".to_string()));

        let request = client.get(format!("/paste/{}", "YTA0NzY"));
        let mut response = request.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            serde_json::from_str::<PasteContent>(&response.body_string().unwrap()).unwrap(),
            PasteContent::PlainText("Test of Content".to_string())
        );

        // Making sure the request doesn't have a valid response if the path
        // contains some valid hash.
        let request = client.get(format!("/paste/A{}", "YTA0NzY"));
        let response = request.dispatch();
        assert_eq!(response.status(), Status::NotFound);
        let request = client.get(format!("/paste/{}B", "YTA0NzY"));
        let response = request.dispatch();
        assert_eq!(response.status(), Status::NotFound);
        let request = client.get(format!("/paste/{}/invalid", "YTA0NzY"));
        let response = request.dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
}
