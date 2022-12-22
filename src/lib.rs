use std::any::Any;

use resvg::{
    tiny_skia,
    usvg::{self, Tree},
};
use worker::*;

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or_else(|| "unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.
    let router = Router::new();

    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    router
        .get_async("/", |req, ctx| async move {
            let Some(title) = get_query_param(&req, "title") else {
                return Response::error("Not found", 404);
            };

            // let Some(query) = ctx.param("query") else {
            //     return Response::ok("Hello from Rust!");
            // };

            handle_image(title).await
        })
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .run(req, env)
        .await
}

fn get_query_param(req: &Request, param: &str) -> Option<String> {
    let url = req.url().expect("URL to parse");
    url.query_pairs()
        .find(|(k, _)| k == param)
        .map(|(_, val)| val)
        .as_deref()
        .map(|s| s.to_owned())
}

async fn handle_image(title: String) -> Result<Response> {
    let tree = Tree::from_str(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 800 600\" width=\"800\" height=\"600\"><circle cx=\"50\" cy=\"50\" r=\"50\" /></svg>",
        &usvg::Options::default(),
    )
    .expect("Parse svg");

    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    let mut headers = Headers::new();
    headers.set("content-type", "image/png")?;

    Ok(Response::from_bytes(pixmap.encode_png().unwrap())?.with_headers(headers))
}
