#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;


// #[macro_use]
// extern crate cdbc;

use std::borrow::Cow;
use std::fmt::Write;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
// use cdbc::{Executor, PoolConnection};
// use cdbc_pg::{PgConnection, PgPool, PgPoolOptions, Postgres};





use oorandom::Rand32;
use smallvec::SmallVec;
use yarte::{ywrite_html, Serialize};

use cogo_http::route::Route;
use cogo_http::server::{Request, Response};

#[derive(Serialize)]
struct HeloMessage {
    message: &'static str,
}

#[derive(Serialize)]
struct WorldRow {
    id: i32,
    randomnumber: i32,
}

#[derive(Serialize)]
pub struct Fortune {
    id: i32,
    message: Cow<'static, str>,
}


fn main() {
    cogo::config()
        .set_pool_capacity(10000)
        .set_stack_size(0x1000);
    let mut route = Route::new();

    route.handle_fn("/json",|req: Request, rsp: Response| {
        rsp.headers.set_raw("Content-Type",vec![b"application/json".to_vec()]);
        rsp.headers.set_raw("Server",vec![b"cogo".to_vec()]);
        let msg = HeloMessage {
            message: "Hello, World!",
        };
        let mut buf =vec![];
        msg.to_bytes_mut(&mut buf);
        rsp.send(&buf);
    });
    route.handle_fn("/plaintext",|req: Request, rsp: Response| {
      rsp.headers.set_raw("Content-Type",vec![b"text/plain".to_vec()]);
      rsp.headers.set_raw("Server",vec![b"cogo".to_vec()]);
      rsp.send("Hello, World!".as_bytes());
    });
    route.handle_fn("/db",|req: Request, rsp: Response| {
        rsp.headers.set_raw("Content-Type",vec![b"application/json".to_vec()]);
        let mut rng = Rand32::new(0);
        let random_id = (rng.rand_u32() % 10_000 + 1) as i32;
        let world = {
            WorldRow{
                id: random_id,
                randomnumber: random_id
            }
        };
        let mut buf =vec![];
        world.to_bytes_mut(&mut buf);
        rsp.send(&buf);
    });
    route.handle_fn("/fortunes",|req: Request, rsp: Response| {
        rsp.headers.set_raw("Content-Type",vec![b"text/html; charset=utf-8".to_vec()]);
        let mut rng = Rand32::new(0);
        let fortunes = {
            let mut v=vec![];
            let random_id = (rng.rand_u32() % 10_000 + 1) as i32;
            v.push(Fortune{
                id: random_id,
                message: Cow::Owned("ff".parse().unwrap())
            });
            v
        };
        let mut body = Vec::with_capacity(2048);
        ywrite_html!(body, "{{> fortune }}");
        rsp.send(&body);
    });
    route.handle_fn("/queries",|req: Request, rsp: Response| {
        rsp.headers.set_raw("Content-Type",vec![b"application/json".to_vec()]);
        let mut rng = Rand32::new(0);
        let worlds = {
            let mut vec:SmallVec<[WorldRow;1]> =SmallVec::new();
            let random_id = (rng.rand_u32() % 10_000 + 1) as i32;
            vec.push(WorldRow{
                id: random_id,
                randomnumber: random_id
            });
            vec
        };
        let mut buf =vec![];
        worlds.to_bytes_mut(&mut buf);
        rsp.send(&buf);
    });
    route.handle_fn("/updates",|req: Request, rsp: Response| {
        rsp.headers.set_raw("Content-Type",vec![b"application/json".to_vec()]);
        let mut rng = Rand32::new(0);
        let worlds = {
            let mut vec:SmallVec<[WorldRow;1]> =SmallVec::new();
            let random_id = (rng.rand_u32() % 10_000 + 1) as i32;
            vec.push(WorldRow{
                id: random_id,
                randomnumber: random_id
            });
            vec
        };
        let mut buf =vec![];
        worlds.to_bytes_mut(&mut buf);
        rsp.send(&buf);
    });
    let route = Arc::new(route);
    let _listening = cogo_http::Server::http("0.0.0.0:8080").unwrap()
        .handle_stack(route.clone(),0x1000);
    println!("Starting http server: 127.0.0.1:8080");
}
