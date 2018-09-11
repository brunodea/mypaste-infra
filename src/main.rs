extern crate actix_web;
extern crate futures;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate md5;
extern crate base64;

use actix_web::server;

trait Content {
    fn hash(&self) -> String;
}

trait Cache<T: Content> {
    fn set(&mut self, value: T);
    fn get(&self, key: String) -> Option<&T>;
}

mod cache {
    use super::Content;
    use super::Cache;
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
    use super::Content;
    use md5;
    use base64;

    const HASHLEN: usize = 7usize;

    #[derive(Debug, Serialize, Deserialize)]
    pub(crate) struct PasteContent {
        content: String,
    }

    impl PasteContent {
        pub(crate) fn new(content: &str) -> Self {
            PasteContent {
                content: content.to_string()
            }
        }
    }

    impl Content for PasteContent {
        fn hash(&self) -> String {
            let md5_value = md5::compute(&self.content);
            let mut base64_value = base64::encode(&format!("{:x}", md5_value));
            base64_value.truncate(HASHLEN);
            base64_value
        }
    }
}

mod paste_app {
    use actix_web::{http, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse};
    use futures::future::Future;
    use super::Content; // just to add the trait to scope
    use super::Cache; // just to add the trait to scope
    use content::PasteContent;
    use cache::MemoryCache;
    use std::cell::RefCell;

    pub(crate) struct AppState {
        cache: RefCell<MemoryCache<PasteContent>>
    }

    fn paste(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
        req.json()
            .and_then(|content: PasteContent| {
                println!("/paste/hash: {}", content.hash());
                let mut memcache = req.state().cache.borrow_mut();
                memcache.set(content);
                Ok(HttpResponse::Ok().json(PasteContent::new("Success!")))
            }).responder()
    }

    fn read(req: &HttpRequest<AppState>) -> HttpResponse {
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

    pub(crate) fn app() -> App<AppState> {
        let state = AppState {
            cache: RefCell::new(MemoryCache::<PasteContent>::new())
        };
        App::with_state(state)
            .resource("/paste", |r| r.method(http::Method::POST).f(paste))
            .resource("/paste/{id}", |r| r.method(http::Method::GET).f(read))
    }
}

fn main() {
    server::new(|| paste_app::app())
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
