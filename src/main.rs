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
            let content = match self {
                PasteContent::Text(text) => text,
            };
            let md5_value = md5::compute(&content);
            let mut base64_value = base64::encode(&format!("{:x}", md5_value));
            base64_value.truncate(HASHLEN);
            base64_value
        }
    }
}

mod cache {
    use content::Content;
    use std::collections::HashMap;

    pub(crate) trait Cache<T: Content> {
        fn set(&mut self, value: T);
        fn get(&self, key: String) -> Option<&T>;
    }

    pub(crate) struct MemoryCache<T: Content> {
        map: HashMap<String, T>,
    }

    impl<T: Content> MemoryCache<T> {
        pub(crate) fn new() -> Self {
            MemoryCache {
                map: HashMap::<String, T>::new(),
            }
        }
    }

    impl<T: Content> Cache<T> for MemoryCache<T> {
        fn set(&mut self, value: T) {
            self.map.insert(value.hash(), value);
        }

        fn get(&self, key: String) -> Option<&T> {
            self.map.get(&key)
        }
    }
}

mod paste_app {
    use actix_web::{http, middleware::Logger, pred, App, HttpRequest, HttpResponse, Json, State};
    use cache::{Cache, MemoryCache};
    use content::{Content, PasteContent};
    use std::cell::RefCell;

    pub(crate) fn app() -> App<AppState> {
        let state = AppState::new(Box::new(MemoryCache::<PasteContent>::new()));
        App::with_state(state)
            .middleware(Logger::default())
            .middleware(Logger::new("%a %s{User-Agent}i"))
            .resource("/paste", |r| r.method(http::Method::POST).with(paste_post))
            .resource("/paste/{id}", |r| r.method(http::Method::GET).f(paste_get))
            /*.default_resource(|r| {
                r.method(http::Method::GET).f(p404);
                r.route()
                    .filter(pred::Not(pred::Get()))
                    .f(|_| HttpResponse::MethodNotAllowed());
            })*/
    }

    pub(crate) struct AppState {
        cache: RefCell<Box<Cache<PasteContent>>>,
    }

    impl AppState {
        fn new(cache: Box<Cache<PasteContent>>) -> Self {
            AppState {
                cache: RefCell::new(cache),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct PasteResponse {
        hash: String,
    }

    fn paste_post((content, state): (Json<PasteContent>, State<AppState>)) -> HttpResponse {
        let content = content.into_inner();
        let hash = content.hash();
        let mut memcache = state.cache.borrow_mut();
        memcache.set(content);
        println!("hash: {}", hash);
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(hash)
    }

    fn paste_get(req: &HttpRequest<AppState>) -> HttpResponse {
        let hash_id = req.match_info().get("id").unwrap();
        let state = req.state();
        let res = match state.cache.borrow().get(hash_id.to_string()) {
            Some(ref paste) => format!("{:?}", paste),
            None => "Not found!".to_string(),
        };
        HttpResponse::Ok().content_type("text/plain").body(res)
    }

    fn p404(_: &HttpRequest<AppState>) -> HttpResponse {
        HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Custom not found 404!")
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();

    actix_web::server::new(move || paste_app::app())
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
