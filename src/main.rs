extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use hyper::{Response, StatusCode};

use gotham::http::response::create_response;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::State;
use gotham::handler::IntoResponse;

// https://jsonplaceholder.typicode.com/posts/1
#[derive(Serialize, Deserialize)]
struct Post {
    user_id: u8,
    id: u8,
    title: String,
    body: String
}

impl IntoResponse for Post {
    fn into_response(self, state: &State) -> Response {
        create_response(
            state,
            StatusCode::Ok,
            Some((
                serde_json::to_string(&self)
                    .expect("expected serialized post")
                    .into_bytes(),
                mime::APPLICATION_JSON
            ))
        )
    }
}

fn get_post_handler(state: State) -> (State, Post) {
    let post = Post {
        user_id: 12,
        id: 19,
        title: "Some post".to_string(),
        body: "This is my first post, qskdsqdksqdshd".to_string()
    };

    (state, post)
}

fn router() -> Router {
    build_simple_router(|route| {
        route.get("/post").to(get_post_handler)
    })
}

fn main() {
    let addr = "127.0.0.1:3038";
    println!("Listening for requests at {}", addr);

    gotham::start(addr, router())
}
