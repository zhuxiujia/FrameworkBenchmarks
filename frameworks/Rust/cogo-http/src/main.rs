#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::borrow::Cow;
use std::fmt::Write;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use cdbc::PoolConnection;
use cdbc_pg::{PgConnection, PgPool, Postgres};

use cogo::std::http::server::{HttpService, HttpServiceFactory, Request, Response};

use oorandom::Rand32;
use smallvec::SmallVec;
use yarte::{ywrite_html, Serialize};


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

struct PgConnectionPool {
    pool: Arc<PgPool>,
    idx: AtomicUsize,
    clients: Vec<Arc<PoolConnection<Postgres>>>,
}

impl PgConnectionPool {
    fn new(db_url: &str, size: usize) -> PgConnectionPool {
        let pool=Arc::new(PgPool::connect(db_url).unwrap());

        let mut clients = Vec::with_capacity(size);
        for _ in 0..size {
            let client = pool.acquire().unwrap();
            clients.push(Arc::new(client));
        }

        PgConnectionPool {
            pool:pool,
            idx: AtomicUsize::new(0),
            clients,
        }
    }

    fn get_connection(&self) -> (Arc<PoolConnection<Postgres>>, usize) {
        let idx = self.idx.fetch_add(1, Ordering::Relaxed);
        let len = self.clients.len();
        (self.clients[idx % len].clone(), idx)
    }
}



struct Techempower {
    db: Arc<PoolConnection<Postgres>>,
    rng: Rand32,
}

impl HttpService for Techempower {
    fn call(&mut self, req: Request, rsp: &mut Response) -> io::Result<()> {
        // Bare-bones router
        match req.path() {
            "/json" => {
                rsp.header("Content-Type: application/json");
                let msg = HeloMessage {
                    message: "Hello, World!",
                };
                msg.to_bytes_mut(rsp.body_mut());
            }
            "/plaintext" => {
                rsp.header("Content-Type: text/plain").body("Hello, World!");
            }
            // "/db" => {
            //     rsp.header("Content-Type: application/json");
            //     let random_id = (self.rng.rand_u32() % 10_000 + 1) as i32;
            //     let world = self.db.get_world(random_id).unwrap();
            //     world.to_bytes_mut(rsp.body_mut())
            // }
            // "/fortunes" => {
            //     rsp.header("Content-Type: text/html; charset=utf-8");
            //     let fortunes = self.db.tell_fortune().unwrap();
            //     let mut body = Vec::with_capacity(2048);
            //     ywrite_html!(body, "{{> fortune }}");
            //     rsp.body_vec(body);
            // }
            // p if p.starts_with("/queries") => {
            //     rsp.header("Content-Type: application/json");
            //     let q = utils::get_query_param(p) as usize;
            //     let worlds = self.db.get_worlds(q, &mut self.rng).unwrap();
            //     worlds.to_bytes_mut(rsp.body_mut());
            // }
            // p if p.starts_with("/updates") => {
            //     rsp.header("Content-Type: application/json");
            //     let q = utils::get_query_param(p) as usize;
            //     let worlds = self.db.updates(q, &mut self.rng).unwrap();
            //     worlds.to_bytes_mut(rsp.body_mut());
            // }
            _ => {
                rsp.status_code("404", "Not Found");
            }
        }

        Ok(())
    }
}

struct HttpServer {
    db_pool: PgConnectionPool,
}

impl HttpServiceFactory for HttpServer {
    type Service = Techempower;

    fn new_service(&self) -> Self::Service {
        let (db, idx) = self.db_pool.get_connection();
        let rng = Rand32::new(idx as u64);
        Techempower { db, rng }
    }
}

fn main() {
    cogo::config()
        .set_pool_capacity(10000)
        .set_stack_size(0x1000);
    println!("Starting http server: 127.0.0.1:8080");
    let server = HttpServer {
        db_pool: PgConnectionPool::new(
            "postgres://benchmarkdbuser:benchmarkdbpass@tfb-database:5432/hello_world",
            num_cpus::get(),
        ),
    };
    server.start("0.0.0.0:8080").unwrap().join().unwrap();
}
