extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

use futures::{Future, Stream};

use hyper::{Response, StatusCode};
use hyper::{Client, Uri};

use gotham::http::response::create_response;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham::handler::{HandlerFuture, IntoHandlerError, IntoResponse};

use tokio_core::reactor::Handle;

type ResponseContentFuture = Box<Future<Item = Vec<Post>, Error = hyper::Error>>;

fn http_get(handle: &Handle) -> ResponseContentFuture {
    let client = Client::new(handle);
    let posts_url: Uri = "http://jsonplaceholder.typicode.com/posts".parse().unwrap();

    let f = client
        .get(posts_url)
        .and_then(|posts_response| {
            posts_response.body().concat2().and_then(|full_body| {
                Ok(serde_json::from_slice(&full_body).expect("expected serialized posts"))
            })
        })
        .and_then(move |mut posts: Vec<Post>| {
            let comments_url: Uri = "http://jsonplaceholder.typicode.com/posts/1/comments"
                .parse()
                .unwrap();

            client.get(comments_url).and_then(|comments_response| {
                comments_response.body().concat2().and_then(|full_body| {
                    let vcomments =
                        serde_json::from_slice(&full_body).expect("expected serialized comments");

                    posts[0].comments = vcomments;

                    Ok(posts)
                })
            })
        });

    Box::new(f)
}

fn get_posts_handler(state: State) -> Box<HandlerFuture> {
    let data_future: ResponseContentFuture = {
        let handle = Handle::borrow_from(&state).clone();

        let f = http_get(&handle);

        Box::new(f)
    };

    Box::new(data_future.then(move |result| match result {
        Ok(data) => {
            let res = create_response(
                &state,
                StatusCode::Ok,
                Some((
                    serde_json::to_string(&data)
                        .expect("expected serialized post")
                        .into_bytes(),
                    mime::APPLICATION_JSON,
                )),
            );

            Ok((state, res))
        }
        Err(err) => Err((state, err.into_handler_error())),
    }))
}

#[derive(Serialize, Deserialize)]
struct Post {
    #[serde(rename = "userId")]
    user_id: u8,
    id: u8,
    title: String,
    body: String,
    #[serde(default)]
    comments: Vec<Comment>,
}

#[derive(Serialize, Deserialize)]
struct Comment {
    #[serde(rename = "postId")]
    post_id: u8,
    id: u8,
    name: String,
    email: String,
    body: String,
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
                mime::APPLICATION_JSON,
            )),
        )
    }
}

fn sync_get_post_handler(state: State) -> (State, Post) {
    let post = Post {
        user_id: 12,
        id: 19,
        title: "Some post".to_string(),
        body: "This is my first post, qskdsqdksqdshd".to_string(),
        comments: vec![],
    };

    (state, post)
}

fn router() -> Router {
    build_simple_router(|route| {
        route.get("/posts").to(get_posts_handler);
        route.get("/syncpost").to(sync_get_post_handler)
    })
}

fn main() {
    let addr = "127.0.0.1:3038";
    println!("Listening for requests at {}", addr);

    gotham::start(addr, router())
}
