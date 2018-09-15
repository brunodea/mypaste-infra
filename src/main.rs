extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate md5;

mod content {
    use base64;
    use md5;

    pub(crate) trait Content {
        fn hash(&self) -> String;
    }

    const HASHLEN: usize = 7usize;

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) enum PasteContent {
        Text(String),
    }

    impl Content for PasteContent {
        fn hash(&self) -> String {
            let PasteContent::Text(content) = &self;
            let md5_value = md5::compute(&content);
            let mut base64_value = base64::encode(&format!("{:x}", md5_value));
            base64_value.truncate(HASHLEN);
            base64_value
        }
    }
}

mod cache {
    use content::{Content, PasteContent};
    use std::collections::HashMap;

    pub(crate) trait Cache {
        fn set(&mut self, value: PasteContent);
        fn get(&self, key: String) -> Option<&PasteContent>;
    }

    pub(crate) struct MemoryCache {
        map: HashMap<String, PasteContent>,
    }

    impl MemoryCache {
        pub(crate) fn new() -> Self {
            MemoryCache {
                map: HashMap::<String, PasteContent>::new(),
            }
        }
    }

    impl Cache for MemoryCache {
        fn set(&mut self, value: PasteContent) {
            self.map.insert(value.hash(), value);
        }

        fn get(&self, key: String) -> Option<&PasteContent> {
            self.map.get(&key)
        }
    }
}

mod paste_app {
    use actix_web::{http, middleware::Logger, App, HttpRequest, HttpResponse, Json, State};
    use cache::{Cache, MemoryCache};
    use content::{Content, PasteContent};
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
        let res = match cache.get(hash_id.to_string()) {
            Some(ref paste) => format!("{:?}", paste),
            None => "Not found!".to_string(),
        };
        HttpResponse::Ok().content_type("text/plain").body(res)
    }
}

fn main() {
    use paste_app;
    use std::env;

    env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    let cache = paste_app::memory_cache();
    actix_web::server::new(move || paste_app::paste_app(cache.clone()))
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
