use actix_web::{http, middleware::Logger, App, HttpRequest, HttpResponse, Json, State};
use crate::cache::{Cache, MemoryCache};
use crate::content::{Content, PasteContent};
use std::sync::{Arc, Mutex};

pub(crate) type SharedCache = Arc<Mutex<Box<MemoryCache>>>;

pub(crate) fn memory_cache() -> SharedCache {
    Arc::new(Mutex::new(Box::new(MemoryCache::new())))
}

pub(crate) fn paste_app(state: SharedCache) -> App<SharedCache> {
    App::with_state(state)
        .middleware(Logger::default())
        .middleware(Logger::new("%a %{User-Agent}i"))
        .resource("/paste", |r| r.method(http::Method::POST).with(paste_post))
        .resource("/paste/{id}", |r| r.method(http::Method::GET).f(paste_get))
}

#[derive(Debug, Serialize, Deserialize)]
struct PasteResponse {
    hash: String,
}

fn paste_post((content, state): (Json<PasteContent>, State<SharedCache>)) -> HttpResponse {
    let content = content.into_inner();
    let hash = content.hash();
    let mut memcache = state.lock().unwrap();
    memcache.set(content);
    HttpResponse::Ok()
        .content_type("application/json")
        .json(PasteResponse { hash })
}

fn paste_get(req: &HttpRequest<SharedCache>) -> HttpResponse {
    let hash_id = req.match_info().get("id").unwrap();
    let cache = req.state().lock().unwrap();
    match cache.get(hash_id.to_string()) {
        Some(ref paste) => HttpResponse::Ok().json(paste),
        None => HttpResponse::Ok()
            .content_type("text/plain")
            .body("Not found!"),
    }
}
