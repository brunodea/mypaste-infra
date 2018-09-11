extern crate actix_web;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate md5;
extern crate base64;

use actix_web::{http, server, App};

trait Content {
    fn hash(&self) -> String;
}

trait Cache<T: Content> {
    fn set(&mut self, value: T);
    fn get(&self, key: String) -> Option<&T>;
}

mod cache {
    use ::Content;
    use ::Cache;
    use std::collections::HashMap;

    pub(crate) struct MemoryCache<T: Content> {
        map: HashMap<String, T>
    }

    impl<T: Content> MemoryCache<T> {
        pub(crate) fn new() -> Self {
            MemoryCache {
                map: HashMap::<String, T>::new()
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

mod content {
    use ::Content;
    use md5;
    use base64;

    const HASHLEN: usize = 7usize;

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) enum PasteContent {
        Text(String)
    }

    impl Content for PasteContent {
        fn hash(&self) -> String {
            let content = match self {
                PasteContent::Text(text) => {
                    text
                }
            };
            let md5_value = md5::compute(&content);
            let mut base64_value = base64::encode(&format!("{:x}", md5_value));
            base64_value.truncate(HASHLEN);
            base64_value
        }
    }
}

mod paste_app {
    use ::Cache;
    use ::Content;
    use ::content::PasteContent;
    use actix_web::{AsyncResponder, Error, HttpRequest, HttpResponse, HttpMessage};
    use std::cell::RefCell;
    use futures::future::Future;

    pub(crate) struct AppState {
        cache: RefCell<Box<Cache<PasteContent>>>
    }

    impl AppState {
        pub(crate) fn new(cache: Box<Cache<PasteContent>>) -> Self {
            AppState {
                cache: RefCell::new(cache)
            }
        }
    }

    pub(crate) fn paste_post(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
        let state = req.state();
        req.json()
            .from_err()
            .and_then(|content: PasteContent| {
                println!("/paste/hash: {}", content.hash());
                let mut memcache = state.cache.borrow_mut();
                memcache.set(content);
                Ok(HttpResponse::Ok().into())
            }).responder()
    }

    pub(crate) fn paste_get(req: &HttpRequest<AppState>) -> HttpResponse {
        let hash_id = req.match_info().get("id").unwrap();
        let state = req.state();
        let res = match state.cache.borrow().get(hash_id.to_string()) {
            Some(ref paste) => format!("{:?}", paste),
            None => "Not found!".to_string()
        };
        HttpResponse::Ok()
            .content_type("text/plain")
            .body(res)
    }

}

fn app() -> App<paste_app::AppState> {
    let state = paste_app::AppState::new(Box::new(cache::MemoryCache::<content::PasteContent>::new()));

    App::with_state(state)
        .resource("/paste", |r| r.method(http::Method::POST).f(paste_app::paste_post))
        .resource("/paste/{id}", |r| r.method(http::Method::GET).f(paste_app::paste_get))
}

fn main() {
    server::new(|| app())
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
